//! Select Skill and Deselect Skill mutation workflows.

use std::path::PathBuf;

use crate::clients::{resolve_mutation_layer, ClientsLayerReport};
use crate::config_doc::{ConfigDocError, PersistedClientSelection};
use crate::locations::{layer_label, layer_relative_path_label};
use crate::project_anchor::project_anchor_blocker;
use crate::skills_config::{
    final_client_selection_for_new_entry, read_skill_configuration, validate_entry_ids,
    write_skill_configuration, SkillConfiguration, SkillConfigurationEntry, SkillSelection,
};
use crate::{
    ConfigLayerId, Diagnostic, InstallLevel, SuggestedAction, UserConfigPathError, WorkflowContext,
    WorkflowResult, WorkflowStatus,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectSkillRequest {
    pub install_level: InstallLevel,
    pub context: WorkflowContext,
    pub config_layer: Option<ConfigLayerId>,
    pub source_skill_name: String,
    pub entry_id: Option<String>,
    pub source: Option<String>,
    pub git_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeselectSkillRequest {
    pub install_level: InstallLevel,
    pub context: WorkflowContext,
    pub config_layer: Option<ConfigLayerId>,
    pub source_skill_name: String,
    pub entry_id: Option<String>,
    pub source: Option<String>,
    pub git_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SkillMutationData {
    pub install_level: InstallLevel,
    pub config_layer: ClientsLayerReport,
    pub entry_id: Option<String>,
    pub source: String,
    pub git_ref: Option<String>,
    pub source_skill_name: String,
    pub clients: PersistedClientSelection,
    pub changed: bool,
}

pub fn select_skill(request: SelectSkillRequest) -> WorkflowResult<SkillMutationData> {
    mutate_skill_configuration(
        "select_skill",
        request.install_level,
        request.context,
        request.config_layer,
        |configuration, default_clients| {
            let selector = EntrySelector {
                entry_id: request.entry_id.as_deref(),
                source: request.source.as_deref(),
                git_ref: request.git_ref.as_deref(),
            };
            selector.validate_for_select()?;

            let final_clients = default_clients
                .clone()
                .ok_or_else(no_client_selection_blocker)?;
            if !has_final_client_selection(&final_clients) {
                return Err(no_client_selection_blocker());
            }

            let changed = if skill_already_selected(
                &configuration.entries,
                &selector,
                &request.source_skill_name,
            ) {
                false
            } else if let Some(index) =
                find_compatible_entry_index(&configuration.entries, &selector)
            {
                verify_locator_agreement(&configuration.entries[index], &selector)?;
                let entry = &mut configuration.entries[index];
                let SkillSelection::Explicit(skills) = &mut entry.include else {
                    return Err(incompatible_entry_blocker());
                };
                skills.push(request.source_skill_name.clone());
                true
            } else {
                let source = selector
                    .source
                    .ok_or_else(missing_source_locator_blocker)?
                    .to_string();
                if let Some(entry_id) = request.entry_id.as_deref() {
                    ensure_entry_id_available(&configuration.entries, entry_id)?;
                }
                configuration.entries.push(SkillConfigurationEntry {
                    id: request.entry_id.clone(),
                    source,
                    git_ref: request.git_ref.clone(),
                    include: SkillSelection::Explicit(vec![request.source_skill_name.clone()]),
                    clients: None,
                    exclude: Vec::new(),
                    aliases: Default::default(),
                });
                true
            };

            let (entry_id, source, git_ref) = mutation_target(&configuration.entries, &selector);

            Ok((
                entry_id,
                source,
                git_ref,
                request.source_skill_name.clone(),
                final_clients,
                changed,
            ))
        },
        true,
    )
}

pub fn deselect_skill(request: DeselectSkillRequest) -> WorkflowResult<SkillMutationData> {
    mutate_skill_configuration(
        "deselect_skill",
        request.install_level,
        request.context,
        request.config_layer,
        |configuration, default_clients| {
            let selector = EntrySelector {
                entry_id: request.entry_id.as_deref(),
                source: request.source.as_deref(),
                git_ref: request.git_ref.as_deref(),
            };
            selector.validate_for_deselect()?;

            let Some((entry_index, final_clients)) = find_entry_with_skill(
                &configuration.entries,
                &selector,
                &request.source_skill_name,
                default_clients.as_ref(),
            ) else {
                let (entry_id, source, git_ref) = empty_mutation_target(&selector);
                return Ok((
                    entry_id,
                    source,
                    git_ref,
                    request.source_skill_name.clone(),
                    default_clients.unwrap_or(PersistedClientSelection::Explicit(Vec::new())),
                    false,
                ));
            };

            verify_locator_agreement(&configuration.entries[entry_index], &selector)?;
            let target = mutation_target_from_entry(&configuration.entries[entry_index]);

            let entry = &mut configuration.entries[entry_index];
            if let SkillSelection::Explicit(skills) = &mut entry.include {
                skills.retain(|skill| skill != &request.source_skill_name);
                if skills.is_empty() {
                    configuration.entries.remove(entry_index);
                }
            }

            Ok((
                target.0,
                target.1,
                target.2,
                request.source_skill_name.clone(),
                final_clients,
                true,
            ))
        },
        false,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EntrySelector<'a> {
    entry_id: Option<&'a str>,
    source: Option<&'a str>,
    git_ref: Option<&'a str>,
}

impl EntrySelector<'_> {
    fn validate_for_select(&self) -> Result<(), Diagnostic> {
        if self.entry_id.is_none() && self.source.is_none() {
            return Err(missing_entry_selector_blocker());
        }
        Ok(())
    }

    fn validate_for_deselect(&self) -> Result<(), Diagnostic> {
        if self.entry_id.is_none() && self.source.is_none() {
            return Err(missing_entry_selector_blocker());
        }
        Ok(())
    }
}

fn mutate_skill_configuration(
    workflow: &'static str,
    install_level: InstallLevel,
    context: WorkflowContext,
    config_layer: Option<ConfigLayerId>,
    transform: impl FnOnce(
        &mut SkillConfiguration,
        Option<PersistedClientSelection>,
    ) -> Result<
        (
            Option<String>,
            String,
            Option<String>,
            String,
            PersistedClientSelection,
            bool,
        ),
        Diagnostic,
    >,
    install_only: bool,
) -> WorkflowResult<SkillMutationData> {
    let layer = match resolve_mutation_layer(install_level, config_layer) {
        Ok(layer) => layer,
        Err(blocker) => {
            return blocked_result(
                workflow,
                install_level,
                &context,
                config_layer.unwrap_or(ConfigLayerId::UserProject),
                vec![blocker],
            );
        }
    };

    if install_level == InstallLevel::Project {
        if let Some(blocker) = project_anchor_blocker(&context) {
            return blocked_result(workflow, install_level, &context, layer, vec![blocker]);
        }
    }

    let path = match context.config_layer_path(layer) {
        Ok(path) => path,
        Err(error) => {
            return blocked_result(
                workflow,
                install_level,
                &context,
                layer,
                vec![user_config_path_blocker(error)],
            );
        }
    };

    let default_clients = match final_client_selection_for_new_entry(&path) {
        Ok(default_clients) => default_clients,
        Err(error) => {
            return blocked_result(
                workflow,
                install_level,
                &context,
                layer,
                vec![config_read_blocker(layer, &path, error)],
            );
        }
    };

    let mut configuration = match read_skill_configuration(&path) {
        Ok(configuration) => configuration,
        Err(error) => {
            return blocked_result(
                workflow,
                install_level,
                &context,
                layer,
                vec![config_read_blocker(layer, &path, error)],
            );
        }
    };

    let (entry_id, source, git_ref, source_skill_name, clients, changed) =
        match transform(&mut configuration, default_clients.clone()) {
            Ok(value) => value,
            Err(blocker) => {
                return blocked_result(workflow, install_level, &context, layer, vec![blocker]);
            }
        };

    if changed {
        if let Err(error) = validate_entry_ids(&configuration.entries) {
            return blocked_result(
                workflow,
                install_level,
                &context,
                layer,
                vec![config_read_blocker(layer, &path, error)],
            );
        }
        if let Err(error) = write_skill_configuration(&path, layer, &configuration) {
            return blocked_result(
                workflow,
                install_level,
                &context,
                layer,
                vec![config_write_blocker(layer, &path, error)],
            );
        }
    }

    successful_mutation(SuccessfulMutationInput {
        workflow,
        install_level,
        layer,
        path,
        entry_id,
        source,
        git_ref,
        source_skill_name,
        clients,
        changed,
        install_only,
    })
}

fn find_compatible_entry_index(
    entries: &[SkillConfigurationEntry],
    selector: &EntrySelector<'_>,
) -> Option<usize> {
    if let Some(entry_id) = selector.entry_id {
        return entries.iter().position(|entry| {
            entry.is_compatible_with_explicit_selection(
                Some(entry_id),
                selector.source.unwrap_or(&entry.source),
                selector.git_ref.or(entry.git_ref.as_deref()),
            )
        });
    }

    let source = selector.source?;
    entries.iter().position(|entry| {
        entry.is_compatible_with_explicit_selection(None, source, selector.git_ref)
    })
}

fn skill_already_selected(
    entries: &[SkillConfigurationEntry],
    selector: &EntrySelector<'_>,
    skill: &str,
) -> bool {
    matching_entry_indexes(entries, selector)
        .iter()
        .any(|index| {
            matches!(
                    &entries[*index].include,
                SkillSelection::Explicit(skills) if skills.contains(&skill.to_string())
            )
        })
}

fn find_entry_with_skill(
    entries: &[SkillConfigurationEntry],
    selector: &EntrySelector<'_>,
    skill: &str,
    default_clients: Option<&PersistedClientSelection>,
) -> Option<(usize, PersistedClientSelection)> {
    matching_entry_indexes(entries, selector)
        .into_iter()
        .find_map(|index| {
            let entry = &entries[index];
            let SkillSelection::Explicit(skills) = &entry.include else {
                return None;
            };
            if !skills.contains(&skill.to_string()) {
                return None;
            }
            Some((
                index,
                entry
                    .final_client_selection(default_clients)
                    .unwrap_or(PersistedClientSelection::Explicit(Vec::new())),
            ))
        })
}

fn matching_entry_indexes(
    entries: &[SkillConfigurationEntry],
    selector: &EntrySelector<'_>,
) -> Vec<usize> {
    entries
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            if let Some(entry_id) = selector.entry_id {
                if !entry.matches_entry_id(entry_id) {
                    return None;
                }
            } else {
                let source = selector.source?;
                if !entry.matches_source(source, selector.git_ref) {
                    return None;
                }
            }
            Some(index)
        })
        .collect()
}

fn verify_locator_agreement(
    entry: &SkillConfigurationEntry,
    selector: &EntrySelector<'_>,
) -> Result<(), Diagnostic> {
    if !entry.locator_agrees_with(selector.source, selector.git_ref) {
        return Err(selector_mismatch_blocker(
            selector.entry_id,
            selector.source,
            selector.git_ref,
        ));
    }
    Ok(())
}

fn ensure_entry_id_available(
    entries: &[SkillConfigurationEntry],
    entry_id: &str,
) -> Result<(), Diagnostic> {
    if entries.iter().any(|entry| entry.matches_entry_id(entry_id)) {
        return Err(duplicate_entry_id_blocker(entry_id));
    }
    Ok(())
}

fn mutation_target(
    entries: &[SkillConfigurationEntry],
    selector: &EntrySelector<'_>,
) -> (Option<String>, String, Option<String>) {
    if let Some(index) = matching_entry_indexes(entries, selector).into_iter().next() {
        return mutation_target_from_entry(&entries[index]);
    }
    empty_mutation_target(selector)
}

fn mutation_target_from_entry(
    entry: &SkillConfigurationEntry,
) -> (Option<String>, String, Option<String>) {
    (
        entry.id.clone(),
        entry.source.clone(),
        entry.git_ref.clone(),
    )
}

fn empty_mutation_target(selector: &EntrySelector<'_>) -> (Option<String>, String, Option<String>) {
    (
        selector.entry_id.map(str::to_string),
        selector.source.unwrap_or_default().to_string(),
        selector.git_ref.map(str::to_string),
    )
}

fn has_final_client_selection(clients: &PersistedClientSelection) -> bool {
    match clients {
        PersistedClientSelection::All => true,
        PersistedClientSelection::Explicit(values) => !values.is_empty(),
    }
}

struct SuccessfulMutationInput {
    workflow: &'static str,
    install_level: InstallLevel,
    layer: ConfigLayerId,
    path: PathBuf,
    entry_id: Option<String>,
    source: String,
    git_ref: Option<String>,
    source_skill_name: String,
    clients: PersistedClientSelection,
    changed: bool,
    install_only: bool,
}

fn successful_mutation(input: SuccessfulMutationInput) -> WorkflowResult<SkillMutationData> {
    let SuccessfulMutationInput {
        workflow,
        install_level,
        layer,
        path,
        entry_id,
        source,
        git_ref,
        source_skill_name,
        clients,
        changed,
        install_only,
    } = input;

    let mut suggested_actions = Vec::new();
    if changed {
        suggested_actions.push(SuggestedAction {
            command: "agentcfg install".to_string(),
            reason: if install_only {
                "Materialize Skill Selection changes.".to_string()
            } else {
                "Materialize binding changes.".to_string()
            },
        });
        if !install_only {
            suggested_actions.push(SuggestedAction {
                command: "agentcfg prune".to_string(),
                reason: "Remove stale Managed State when ownership can be derived.".to_string(),
            });
        }
    }

    WorkflowResult {
        workflow,
        status: WorkflowStatus::Success,
        diagnostics: Vec::new(),
        blockers: Vec::new(),
        suggested_actions,
        progress_events: Vec::new(),
        data: SkillMutationData {
            install_level,
            config_layer: ClientsLayerReport {
                id: layer,
                name: layer_label(layer),
                path,
                default_clients: None,
            },
            entry_id,
            source,
            git_ref,
            source_skill_name,
            clients,
            changed,
        },
    }
}

fn blocked_result(
    workflow: &'static str,
    install_level: InstallLevel,
    context: &WorkflowContext,
    layer: ConfigLayerId,
    blockers: Vec<Diagnostic>,
) -> WorkflowResult<SkillMutationData> {
    let path = context
        .config_layer_path(layer)
        .unwrap_or_else(|_| PathBuf::from("<unresolved>"));
    WorkflowResult {
        workflow,
        status: WorkflowStatus::Success,
        diagnostics: Vec::new(),
        blockers,
        suggested_actions: Vec::new(),
        progress_events: Vec::new(),
        data: SkillMutationData {
            install_level,
            config_layer: ClientsLayerReport {
                id: layer,
                name: layer_label(layer),
                path,
                default_clients: None,
            },
            entry_id: None,
            source: String::new(),
            git_ref: None,
            source_skill_name: String::new(),
            clients: PersistedClientSelection::Explicit(Vec::new()),
            changed: false,
        },
    }
}

fn no_client_selection_blocker() -> Diagnostic {
    Diagnostic {
        code: "no-client-selection".to_string(),
        message: "Cannot select Skill without a final client selection.".to_string(),
        context: Vec::new(),
        suggested_actions: vec![SuggestedAction {
            command: "agentcfg clients set <client>...".to_string(),
            reason: "Set Default Client Selection before selecting Skills.".to_string(),
        }],
    }
}

fn missing_entry_selector_blocker() -> Diagnostic {
    Diagnostic {
        code: "missing-entry-selector".to_string(),
        message: "Provide --id or --source to select a Skill Configuration Entry.".to_string(),
        context: Vec::new(),
        suggested_actions: Vec::new(),
    }
}

fn missing_source_locator_blocker() -> Diagnostic {
    Diagnostic {
        code: "missing-source-locator".to_string(),
        message: "Provide --source when creating a new Skill Configuration Entry.".to_string(),
        context: Vec::new(),
        suggested_actions: Vec::new(),
    }
}

fn selector_mismatch_blocker(
    entry_id: Option<&str>,
    source: Option<&str>,
    git_ref: Option<&str>,
) -> Diagnostic {
    let mut context = Vec::new();
    if let Some(entry_id) = entry_id {
        context.push(("entry-id".to_string(), entry_id.to_string()));
    }
    if let Some(source) = source {
        context.push(("source".to_string(), source.to_string()));
    }
    if let Some(git_ref) = git_ref {
        context.push(("ref".to_string(), git_ref.to_string()));
    }
    Diagnostic {
        code: "entry-selector-mismatch".to_string(),
        message: "Skill Configuration Entry Id and Skill Source locator disagree.".to_string(),
        context,
        suggested_actions: Vec::new(),
    }
}

fn duplicate_entry_id_blocker(entry_id: &str) -> Diagnostic {
    Diagnostic {
        code: "duplicate-entry-id".to_string(),
        message: format!("Skill Configuration Entry Id '{entry_id}' is already declared."),
        context: vec![("entry-id".to_string(), entry_id.to_string())],
        suggested_actions: Vec::new(),
    }
}

fn incompatible_entry_blocker() -> Diagnostic {
    Diagnostic {
        code: "incompatible-skill-entry".to_string(),
        message:
            "Cannot append explicit Included Skill to an incompatible Skill Configuration Entry."
                .to_string(),
        context: Vec::new(),
        suggested_actions: Vec::new(),
    }
}

fn user_config_path_blocker(error: UserConfigPathError) -> Diagnostic {
    Diagnostic {
        code: "user-config-path-unresolved".to_string(),
        message: format!("Cannot resolve User Config path: {error}"),
        context: vec![(
            "config-layer".to_string(),
            layer_relative_path_label(ConfigLayerId::User).to_string(),
        )],
        suggested_actions: Vec::new(),
    }
}

fn config_read_blocker(
    layer: ConfigLayerId,
    path: &std::path::Path,
    error: ConfigDocError,
) -> Diagnostic {
    Diagnostic {
        code: "config-read-failed".to_string(),
        message: format!(
            "Cannot read {} at {}: {error}",
            layer_label(layer),
            path.display()
        ),
        context: vec![(
            "config-layer".to_string(),
            layer_relative_path_label(layer).to_string(),
        )],
        suggested_actions: Vec::new(),
    }
}

fn config_write_blocker(
    layer: ConfigLayerId,
    path: &std::path::Path,
    error: ConfigDocError,
) -> Diagnostic {
    Diagnostic {
        code: "config-write-failed".to_string(),
        message: format!(
            "Cannot write {} at {}: {error}",
            layer_label(layer),
            path.display()
        ),
        context: vec![(
            "config-layer".to_string(),
            layer_relative_path_label(layer).to_string(),
        )],
        suggested_actions: Vec::new(),
    }
}
