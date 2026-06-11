//! Default Client Selection inspection and mutation workflows.

use std::path::PathBuf;

use crate::client::Client;
use crate::config_doc::PersistedClientSelection;
use crate::config_doc::{read_default_clients, write_default_clients, ConfigDocError};
use crate::locations::{active_config_layers, layer_label, layer_relative_path_label};
use crate::{
    project_anchor_blocker, ConfigLayerId, Diagnostic, InstallLevel, SuggestedAction,
    UserConfigPathError, WorkflowContext, WorkflowResult, WorkflowStatus,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientsShowRequest {
    pub install_level: InstallLevel,
    pub context: WorkflowContext,
    pub config_layer: Option<ConfigLayerId>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ClientsShowData {
    pub install_level: InstallLevel,
    pub config_layers: Vec<ClientsLayerReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ClientsLayerReport {
    pub id: ConfigLayerId,
    pub name: &'static str,
    pub path: PathBuf,
    pub default_clients: Option<PersistedClientSelection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientsSetRequest {
    pub install_level: InstallLevel,
    pub context: WorkflowContext,
    pub config_layer: Option<ConfigLayerId>,
    pub clients: Vec<Client>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientsAddRequest {
    pub install_level: InstallLevel,
    pub context: WorkflowContext,
    pub config_layer: Option<ConfigLayerId>,
    pub clients: Vec<Client>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientsRemoveRequest {
    pub install_level: InstallLevel,
    pub context: WorkflowContext,
    pub config_layer: Option<ConfigLayerId>,
    pub clients: Vec<Client>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ClientsMutationData {
    pub install_level: InstallLevel,
    pub config_layer: ClientsLayerReport,
    pub default_clients: PersistedClientSelection,
    pub changed: bool,
}

pub fn clients_show(request: ClientsShowRequest) -> WorkflowResult<ClientsShowData> {
    let layers = selected_layers(request.install_level, request.config_layer);
    let mut blockers = Vec::new();
    let mut config_layers = Vec::new();

    for layer in layers {
        let path = match request.context.config_layer_path(layer) {
            Ok(path) => path,
            Err(error) => {
                blockers.push(user_config_path_blocker(error));
                continue;
            }
        };
        match read_default_clients(&path) {
            Ok(default_clients) => config_layers.push(ClientsLayerReport {
                id: layer,
                name: layer_label(layer),
                path,
                default_clients,
            }),
            Err(error) => blockers.push(config_read_blocker(layer, &path, error)),
        }
    }

    WorkflowResult {
        workflow: "clients_show",
        status: WorkflowStatus::Success,
        diagnostics: Vec::new(),
        blockers,
        suggested_actions: Vec::new(),
        progress_events: Vec::new(),
        data: ClientsShowData {
            install_level: request.install_level,
            config_layers,
        },
    }
}

pub fn clients_set(request: ClientsSetRequest) -> WorkflowResult<ClientsMutationData> {
    let selection = PersistedClientSelection::Explicit(request.clients.clone());
    mutate_default_clients(
        "clients_set",
        request.install_level,
        request.context,
        request.config_layer,
        move |current| {
            if current.as_ref() == Some(&selection) {
                Ok((selection.clone(), false))
            } else {
                Ok((selection.clone(), true))
            }
        },
    )
}

pub fn clients_add(request: ClientsAddRequest) -> WorkflowResult<ClientsMutationData> {
    let clients = request.clients.clone();
    mutate_default_clients(
        "clients_add",
        request.install_level,
        request.context,
        request.config_layer,
        move |current| {
            let mut next = match current {
                Some(PersistedClientSelection::Explicit(existing)) => existing,
                Some(PersistedClientSelection::All) => {
                    return Err(Diagnostic {
                        code: "clients-all-selected".to_string(),
                        message: "Cannot add clients when Default Client Selection is \"all\"."
                            .to_string(),
                        context: Vec::new(),
                        suggested_actions: Vec::new(),
                    });
                }
                None => Vec::new(),
            };

            let mut changed = false;
            for client in clients {
                if !next.contains(&client) {
                    next.push(client);
                    changed = true;
                }
            }

            Ok((PersistedClientSelection::Explicit(next), changed))
        },
    )
}

pub fn clients_remove(request: ClientsRemoveRequest) -> WorkflowResult<ClientsMutationData> {
    let clients = request.clients.clone();
    mutate_default_clients(
        "clients_remove",
        request.install_level,
        request.context,
        request.config_layer,
        move |current| {
            let Some(PersistedClientSelection::Explicit(mut existing)) = current else {
                return Ok((PersistedClientSelection::Explicit(Vec::new()), false));
            };

            let before = existing.len();
            existing.retain(|client| !clients.contains(client));
            let changed = existing.len() != before;
            Ok((PersistedClientSelection::Explicit(existing), changed))
        },
    )
}

fn mutate_default_clients(
    workflow: &'static str,
    install_level: InstallLevel,
    context: WorkflowContext,
    config_layer: Option<ConfigLayerId>,
    transform: impl FnOnce(
        Option<PersistedClientSelection>,
    ) -> Result<(PersistedClientSelection, bool), Diagnostic>,
) -> WorkflowResult<ClientsMutationData> {
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
    let current = match read_default_clients(&path) {
        Ok(current) => current,
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

    let (next, changed) = match transform(current) {
        Ok(value) => value,
        Err(blocker) => {
            return blocked_result(workflow, install_level, &context, layer, vec![blocker]);
        }
    };
    if changed {
        if let Err(error) = write_default_clients(&path, layer, &next) {
            return blocked_result(
                workflow,
                install_level,
                &context,
                layer,
                vec![config_write_blocker(layer, &path, error)],
            );
        }
    }

    successful_mutation(workflow, install_level, layer, path, next, changed)
}

fn successful_mutation(
    workflow: &'static str,
    install_level: InstallLevel,
    layer: ConfigLayerId,
    path: PathBuf,
    default_clients: PersistedClientSelection,
    changed: bool,
) -> WorkflowResult<ClientsMutationData> {
    let mut suggested_actions = Vec::new();
    if changed {
        suggested_actions.push(SuggestedAction {
            command: "agentcfg install".to_string(),
            reason: "Materialize Default Client Selection changes.".to_string(),
        });
    }

    WorkflowResult {
        workflow,
        status: WorkflowStatus::Success,
        diagnostics: Vec::new(),
        blockers: Vec::new(),
        suggested_actions,
        progress_events: Vec::new(),
        data: ClientsMutationData {
            install_level,
            config_layer: ClientsLayerReport {
                id: layer,
                name: layer_label(layer),
                path,
                default_clients: Some(default_clients.clone()),
            },
            default_clients,
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
) -> WorkflowResult<ClientsMutationData> {
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
        data: ClientsMutationData {
            install_level,
            config_layer: ClientsLayerReport {
                id: layer,
                name: layer_label(layer),
                path,
                default_clients: None,
            },
            default_clients: PersistedClientSelection::Explicit(Vec::new()),
            changed: false,
        },
    }
}

fn selected_layers(
    install_level: InstallLevel,
    config_layer: Option<ConfigLayerId>,
) -> Vec<ConfigLayerId> {
    match config_layer {
        Some(layer) => vec![layer],
        None => active_config_layers(install_level),
    }
}

pub fn resolve_mutation_layer(
    install_level: InstallLevel,
    config_layer: Option<ConfigLayerId>,
) -> Result<ConfigLayerId, Diagnostic> {
    match install_level {
        InstallLevel::Project => match config_layer {
            None => Ok(ConfigLayerId::UserProject),
            Some(ConfigLayerId::SharedProject | ConfigLayerId::UserProject) => {
                Ok(config_layer.unwrap())
            }
            Some(ConfigLayerId::User) => Err(invalid_layer_blocker(
                "--config-layer user requires --level user",
            )),
        },
        InstallLevel::User => match config_layer {
            None => Ok(ConfigLayerId::User),
            Some(ConfigLayerId::User) => Ok(ConfigLayerId::User),
            Some(_) => Err(invalid_layer_blocker(
                "User Level mutations use User Config only; omit --config-layer",
            )),
        },
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

fn invalid_layer_blocker(message: &str) -> Diagnostic {
    Diagnostic {
        code: "invalid-config-layer".to_string(),
        message: message.to_string(),
        context: Vec::new(),
        suggested_actions: Vec::new(),
    }
}
