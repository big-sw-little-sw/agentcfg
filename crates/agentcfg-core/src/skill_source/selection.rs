//! **Skill Selection** for path **Skill Sources**: implicit all-skills, explicit `include`, **Skill Groups**, and diagnostics.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::config::{Config, SkillSourceConfig, SkillSourceKind};
use crate::layer_level::ConfigLayer;
use crate::skill_source::DiscoveryDepth;
use crate::skill_source::groups::{
    InvalidSkillGroupDefinitionFact, SkillGroupsInSource, SkillGroupsMetadataError,
    read_skill_groups_in_source,
};
use crate::skill_source::path::{
    DiscoveredSkill, DiscoveredSkillsInPathSource, discover_skills_in_source,
};
use crate::{
    Error, InvalidSkillGroupDefinition, MissingIncludedSkill, MissingSkillGroup,
    MissingSkillGroupCause, MissingSkillGroupMember, Result, SkillSelectionError,
    SkillSourceMetadataParseError,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillSelectionInput<'a> {
    pub config_file: &'a Path,
    pub config: &'a Config,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillSelection {
    pub selected_skills: Vec<SelectedSkill>,
    pub warnings: Vec<SkillSelectionWarning>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedSkill {
    pub layer: ConfigLayer,
    pub skill_source_id: String,
    pub source_skill_name: String,
    pub skill_dir: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SkillSelectionWarning {
    EmptyDiscovery(EmptyDiscovery),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmptyDiscovery {
    pub layer: ConfigLayer,
    pub skill_source_id: String,
    pub resolved_root: PathBuf,
    pub discovery_depth: DiscoveryDepth,
}

pub fn resolve_skill_selection(inputs: &[SkillSelectionInput<'_>]) -> Result<SkillSelection> {
    let mut selected_skills = Vec::new();
    let mut warnings = Vec::new();
    let mut errors = SelectionErrors::default();

    for input in inputs {
        let layer = input.config.layer();
        for skill_source in input.config.skill_sources() {
            match resolve_source_selection(input, layer, skill_source)? {
                SourceSelectionOutcome::Selected {
                    mut selected,
                    warning,
                } => {
                    selected_skills.append(&mut selected);
                    warnings.extend(warning);
                }
                SourceSelectionOutcome::Invalid(source_errors) => {
                    errors.append(source_errors);
                }
            }
        }
    }

    if errors.has_errors() {
        return Err(errors.into_invalid_selection().into());
    }

    selected_skills.sort_by(|left, right| {
        (
            left.layer,
            left.skill_source_id.as_str(),
            left.source_skill_name.as_str(),
        )
            .cmp(&(
                right.layer,
                right.skill_source_id.as_str(),
                right.source_skill_name.as_str(),
            ))
    });

    Ok(SkillSelection {
        selected_skills,
        warnings,
    })
}

enum SourceSelectionOutcome {
    Selected {
        selected: Vec<SelectedSkill>,
        warning: Option<SkillSelectionWarning>,
    },
    Invalid(SelectionErrors),
}

#[derive(Default)]
struct SelectionErrors {
    missing_included_skills: Vec<MissingIncludedSkill>,
    missing_skill_groups: Vec<MissingSkillGroup>,
    invalid_skill_group_definitions: Vec<InvalidSkillGroupDefinition>,
    missing_skill_group_members: Vec<MissingSkillGroupMember>,
    metadata_parse_errors: Vec<SkillSourceMetadataParseError>,
}

impl SelectionErrors {
    fn append(&mut self, mut other: SelectionErrors) {
        self.missing_included_skills
            .append(&mut other.missing_included_skills);
        self.missing_skill_groups
            .append(&mut other.missing_skill_groups);
        self.invalid_skill_group_definitions
            .append(&mut other.invalid_skill_group_definitions);
        self.missing_skill_group_members
            .append(&mut other.missing_skill_group_members);
        self.metadata_parse_errors
            .append(&mut other.metadata_parse_errors);
    }

    fn has_errors(&self) -> bool {
        !self.missing_included_skills.is_empty()
            || !self.missing_skill_groups.is_empty()
            || !self.invalid_skill_group_definitions.is_empty()
            || !self.missing_skill_group_members.is_empty()
            || !self.metadata_parse_errors.is_empty()
    }

    fn into_invalid_selection(mut self) -> SkillSelectionError {
        sort_missing_included_skills(&mut self.missing_included_skills);
        sort_missing_skill_groups(&mut self.missing_skill_groups);
        sort_invalid_skill_group_definitions(&mut self.invalid_skill_group_definitions);
        sort_missing_skill_group_members(&mut self.missing_skill_group_members);
        sort_metadata_parse_errors(&mut self.metadata_parse_errors);

        SkillSelectionError::InvalidSelection {
            missing_included_skills: self.missing_included_skills,
            missing_skill_groups: self.missing_skill_groups,
            invalid_skill_group_definitions: self.invalid_skill_group_definitions,
            missing_skill_group_members: self.missing_skill_group_members,
            metadata_parse_errors: self.metadata_parse_errors,
        }
    }
}

fn resolve_source_selection(
    input: &SkillSelectionInput<'_>,
    layer: ConfigLayer,
    skill_source: &SkillSourceConfig,
) -> Result<SourceSelectionOutcome> {
    let SkillSourceKind::Path {
        path: configured_path,
    } = skill_source.kind();

    let discovery = discover_skills_in_source(
        skill_source.id(),
        input.config_file,
        configured_path,
        skill_source.discovery_depth(),
    )?;

    if skill_source.included_skill_names().is_empty() && skill_source.skill_group_names().is_empty()
    {
        return Ok(select_implicit_source(layer, skill_source, discovery));
    }

    select_explicit_source(layer, skill_source, &discovery)
}

fn select_implicit_source(
    layer: ConfigLayer,
    skill_source: &SkillSourceConfig,
    discovery: DiscoveredSkillsInPathSource,
) -> SourceSelectionOutcome {
    if discovery.discovered_skills.is_empty() {
        return SourceSelectionOutcome::Selected {
            selected: Vec::new(),
            warning: Some(SkillSelectionWarning::EmptyDiscovery(EmptyDiscovery {
                layer,
                skill_source_id: skill_source.id().to_string(),
                resolved_root: discovery.resolved_root,
                discovery_depth: skill_source.discovery_depth(),
            })),
        };
    }

    SourceSelectionOutcome::Selected {
        selected: discovery
            .discovered_skills
            .iter()
            .map(|skill| selected_from_discovery(layer, skill_source.id(), skill))
            .collect(),
        warning: None,
    }
}

fn select_explicit_source(
    layer: ConfigLayer,
    skill_source: &SkillSourceConfig,
    discovery: &DiscoveredSkillsInPathSource,
) -> Result<SourceSelectionOutcome> {
    let by_name = discovered_by_name(&discovery.discovered_skills);
    let mut selected_for_source: BTreeMap<String, &DiscoveredSkill> = BTreeMap::new();
    let mut errors = SelectionErrors::default();

    select_included_skills(
        layer,
        skill_source,
        discovery,
        &by_name,
        &mut selected_for_source,
        &mut errors,
    );

    if !skill_source.skill_group_names().is_empty() {
        expand_skill_groups(
            layer,
            skill_source,
            discovery,
            &by_name,
            &mut selected_for_source,
            &mut errors,
        )?;
    }

    if errors.has_errors() {
        return Ok(SourceSelectionOutcome::Invalid(errors));
    }

    Ok(SourceSelectionOutcome::Selected {
        selected: selected_for_source
            .values()
            .map(|skill| selected_from_discovery(layer, skill_source.id(), skill))
            .collect(),
        warning: None,
    })
}

fn discovered_by_name(discovered_skills: &[DiscoveredSkill]) -> BTreeMap<&str, &DiscoveredSkill> {
    discovered_skills
        .iter()
        .map(|skill| (skill.source_skill_name.as_str(), skill))
        .collect()
}

fn select_included_skills<'a>(
    layer: ConfigLayer,
    skill_source: &SkillSourceConfig,
    discovery: &DiscoveredSkillsInPathSource,
    by_name: &BTreeMap<&str, &'a DiscoveredSkill>,
    selected_for_source: &mut BTreeMap<String, &'a DiscoveredSkill>,
    errors: &mut SelectionErrors,
) {
    for source_skill_name in skill_source.included_skill_names() {
        if let Some(skill) = by_name.get(source_skill_name.as_str()) {
            selected_for_source.insert(source_skill_name.clone(), skill);
        } else {
            errors.missing_included_skills.push(MissingIncludedSkill {
                layer,
                skill_source_id: skill_source.id().to_string(),
                source_skill_name: source_skill_name.clone(),
                resolved_root: discovery.resolved_root.clone(),
                discovery_depth: skill_source.discovery_depth(),
            });
        }
    }
}

fn expand_skill_groups<'a>(
    layer: ConfigLayer,
    skill_source: &SkillSourceConfig,
    discovery: &DiscoveredSkillsInPathSource,
    by_name: &BTreeMap<&str, &'a DiscoveredSkill>,
    selected_for_source: &mut BTreeMap<String, &'a DiscoveredSkill>,
    errors: &mut SelectionErrors,
) -> Result<()> {
    match read_skill_groups_in_source(&discovery.resolved_root) {
        Ok(SkillGroupsInSource::Absent { skills_toml }) => {
            add_missing_groups(
                layer,
                skill_source,
                discovery,
                skills_toml,
                MissingSkillGroupCause::SkillsTomlAbsent,
                errors,
            );
            Ok(())
        }
        Ok(SkillGroupsInSource::Present {
            skills_toml,
            groups,
        }) => {
            expand_present_groups(
                layer,
                skill_source,
                discovery,
                skills_toml,
                groups,
                by_name,
                selected_for_source,
                errors,
            );
            Ok(())
        }
        Err(SkillGroupsMetadataError::InvalidDefinitions(invalid_definitions)) => {
            errors.invalid_skill_group_definitions.extend(
                invalid_definitions
                    .into_iter()
                    .map(|fact| invalid_definition_with_source(layer, skill_source.id(), fact)),
            );
            Ok(())
        }
        Err(SkillGroupsMetadataError::Io {
            skills_toml,
            source,
        }) => Err(Error::Io {
            path: skills_toml,
            source,
        }),
        Err(SkillGroupsMetadataError::Parse {
            skills_toml,
            source,
        }) => {
            errors
                .metadata_parse_errors
                .push(SkillSourceMetadataParseError {
                    layer,
                    skill_source_id: skill_source.id().to_string(),
                    skills_toml,
                    source,
                });
            Ok(())
        }
    }
}

fn add_missing_groups(
    layer: ConfigLayer,
    skill_source: &SkillSourceConfig,
    discovery: &DiscoveredSkillsInPathSource,
    skills_toml: PathBuf,
    cause: MissingSkillGroupCause,
    errors: &mut SelectionErrors,
) {
    for skill_group_name in skill_source.skill_group_names() {
        errors.missing_skill_groups.push(MissingSkillGroup {
            layer,
            skill_source_id: skill_source.id().to_string(),
            skill_group_name: skill_group_name.clone(),
            resolved_root: discovery.resolved_root.clone(),
            skills_toml: skills_toml.clone(),
            cause,
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn expand_present_groups<'a>(
    layer: ConfigLayer,
    skill_source: &SkillSourceConfig,
    discovery: &DiscoveredSkillsInPathSource,
    skills_toml: PathBuf,
    defined_groups: BTreeMap<String, Vec<String>>,
    by_name: &BTreeMap<&str, &'a DiscoveredSkill>,
    selected_for_source: &mut BTreeMap<String, &'a DiscoveredSkill>,
    errors: &mut SelectionErrors,
) {
    for skill_group_name in skill_source.skill_group_names() {
        let Some(members) = defined_groups.get(skill_group_name.as_str()).cloned() else {
            errors.missing_skill_groups.push(MissingSkillGroup {
                layer,
                skill_source_id: skill_source.id().to_string(),
                skill_group_name: skill_group_name.clone(),
                resolved_root: discovery.resolved_root.clone(),
                skills_toml: skills_toml.clone(),
                cause: MissingSkillGroupCause::SkillsTomlPresent,
            });
            continue;
        };

        for source_skill_name in members {
            if let Some(skill) = by_name.get(source_skill_name.as_str()) {
                selected_for_source.insert(source_skill_name.clone(), skill);
            } else {
                errors
                    .missing_skill_group_members
                    .push(MissingSkillGroupMember {
                        layer,
                        skill_source_id: skill_source.id().to_string(),
                        skill_group_name: skill_group_name.clone(),
                        source_skill_name,
                        resolved_root: discovery.resolved_root.clone(),
                        discovery_depth: skill_source.discovery_depth(),
                    });
            }
        }
    }
}

fn invalid_definition_with_source(
    layer: ConfigLayer,
    skill_source_id: &str,
    fact: InvalidSkillGroupDefinitionFact,
) -> InvalidSkillGroupDefinition {
    InvalidSkillGroupDefinition {
        layer,
        skill_source_id: skill_source_id.to_string(),
        skill_group_name: fact.skill_group_name,
        skills_toml: fact.skills_toml,
        reason: fact.reason,
    }
}

fn sort_missing_included_skills(missing: &mut [MissingIncludedSkill]) {
    missing.sort_by(|left, right| {
        (
            left.layer,
            left.skill_source_id.as_str(),
            left.source_skill_name.as_str(),
        )
            .cmp(&(
                right.layer,
                right.skill_source_id.as_str(),
                right.source_skill_name.as_str(),
            ))
    });
}

fn sort_missing_skill_groups(missing: &mut [MissingSkillGroup]) {
    missing.sort_by(|left, right| {
        (
            left.layer,
            left.skill_source_id.as_str(),
            left.skill_group_name.as_str(),
        )
            .cmp(&(
                right.layer,
                right.skill_source_id.as_str(),
                right.skill_group_name.as_str(),
            ))
    });
}

fn sort_invalid_skill_group_definitions(invalid: &mut [crate::InvalidSkillGroupDefinition]) {
    invalid.sort_by(|left, right| {
        (
            left.layer,
            left.skill_source_id.as_str(),
            left.skill_group_name.as_deref(),
        )
            .cmp(&(
                right.layer,
                right.skill_source_id.as_str(),
                right.skill_group_name.as_deref(),
            ))
    });
}

fn sort_missing_skill_group_members(missing: &mut [MissingSkillGroupMember]) {
    missing.sort_by(|left, right| {
        (
            left.layer,
            left.skill_source_id.as_str(),
            left.skill_group_name.as_str(),
            left.source_skill_name.as_str(),
        )
            .cmp(&(
                right.layer,
                right.skill_source_id.as_str(),
                right.skill_group_name.as_str(),
                right.source_skill_name.as_str(),
            ))
    });
}

fn sort_metadata_parse_errors(errors: &mut [SkillSourceMetadataParseError]) {
    errors.sort_by(|left, right| {
        (
            left.layer,
            left.skill_source_id.as_str(),
            left.skills_toml.as_path(),
        )
            .cmp(&(
                right.layer,
                right.skill_source_id.as_str(),
                right.skills_toml.as_path(),
            ))
    });
}

fn selected_from_discovery(
    layer: ConfigLayer,
    skill_source_id: &str,
    skill: &DiscoveredSkill,
) -> SelectedSkill {
    SelectedSkill {
        layer,
        skill_source_id: skill_source_id.to_string(),
        source_skill_name: skill.source_skill_name.clone(),
        skill_dir: skill.skill_dir.clone(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(irrefutable_let_patterns)]

    use super::*;
    use crate::config::parse_config_str;
    use crate::{
        Error, InvalidSkillGroupDefinitionReason, MissingSkillGroupCause, SkillSelectionError,
    };
    use std::fs;
    use tempfile::TempDir;

    const SKILL_FILE: &str = "SKILL.md";

    fn write_skill(skill_dir: &Path) {
        fs::create_dir_all(skill_dir).unwrap();
        fs::write(skill_dir.join(SKILL_FILE), "skill").unwrap();
    }

    fn write_skills_toml(root: &Path, contents: &str) {
        fs::write(root.join("skills.toml"), contents).unwrap();
    }

    fn parse_and_select(
        layer: ConfigLayer,
        config_file: &Path,
        contents: &str,
    ) -> Result<SkillSelection> {
        let config = parse_config_str(layer, config_file, contents)?;
        resolve_skill_selection(&[SkillSelectionInput {
            config_file,
            config: &config,
        }])
    }

    fn minimal_config(scope: &str, skill_sources_body: &str) -> String {
        format!(
            r#"
scope = "{scope}"

{skill_sources_body}

[skills]
clients = ["codex"]
"#
        )
    }

    fn invalid_selection(error: Error) -> SkillSelectionError {
        let Error::SkillSelection(selection_error) = error else {
            panic!("expected SkillSelection error, got {error:?}");
        };
        selection_error
    }

    #[test]
    fn skill_selection_implicit_all_discovered_skills() {
        let tempdir = TempDir::new().unwrap();
        let skills_dir = tempdir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        write_skill(&skills_dir.join("alpha"));
        write_skill(&skills_dir.join("beta"));

        let config_file = tempdir.path().join("agentcfg.toml");
        let contents = minimal_config(
            "user",
            r#"
[[skill_sources]]
id = "personal"
type = "path"
path = "skills"
"#,
        );

        let selection = parse_and_select(ConfigLayer::User, &config_file, &contents).unwrap();

        assert_eq!(selection.warnings.len(), 0);
        assert_eq!(selection.selected_skills.len(), 2);
        let names: Vec<_> = selection
            .selected_skills
            .iter()
            .map(|skill| skill.source_skill_name.as_str())
            .collect();
        assert_eq!(names, vec!["alpha", "beta"]);
    }

    #[test]
    fn skill_selection_explicit_include_selects_only_listed_skills() {
        let tempdir = TempDir::new().unwrap();
        let skills_dir = tempdir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        write_skill(&skills_dir.join("alpha"));
        write_skill(&skills_dir.join("beta"));

        let config_file = tempdir.path().join("agentcfg.toml");
        let contents = minimal_config(
            "user",
            r#"
[[skill_sources]]
id = "personal"
type = "path"
path = "skills"
include = ["beta"]
"#,
        );

        let selection = parse_and_select(ConfigLayer::User, &config_file, &contents).unwrap();

        assert_eq!(selection.selected_skills.len(), 1);
        assert_eq!(selection.selected_skills[0].source_skill_name, "beta");
    }

    #[test]
    fn skill_selection_sorts_by_layer_source_and_name_not_include_order() {
        let tempdir = TempDir::new().unwrap();
        let skills_dir = tempdir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        write_skill(&skills_dir.join("alpha"));
        write_skill(&skills_dir.join("beta"));
        write_skill(&skills_dir.join("zebra"));

        let config_file = tempdir.path().join("agentcfg.toml");
        let contents = minimal_config(
            "user",
            r#"
[[skill_sources]]
id = "personal"
type = "path"
path = "skills"
include = ["zebra", "alpha", "beta"]
"#,
        );

        let selection = parse_and_select(ConfigLayer::User, &config_file, &contents).unwrap();

        let names: Vec<_> = selection
            .selected_skills
            .iter()
            .map(|skill| skill.source_skill_name.as_str())
            .collect();
        assert_eq!(names, vec!["alpha", "beta", "zebra"]);
    }

    #[test]
    fn skill_selection_preserves_layer_for_same_skill_source_id() {
        let tempdir = TempDir::new().unwrap();
        let shared_skills = tempdir.path().join("shared-skills");
        let user_skills = tempdir.path().join("user-skills");
        fs::create_dir_all(&shared_skills).unwrap();
        fs::create_dir_all(&user_skills).unwrap();
        write_skill(&shared_skills.join("shared-only"));
        write_skill(&user_skills.join("user-only"));

        let shared_config = tempdir.path().join("shared.toml");
        let user_config = tempdir.path().join("user.toml");

        let shared = parse_config_str(
            ConfigLayer::SharedProject,
            &shared_config,
            &minimal_config(
                "shared-project",
                r#"
[[skill_sources]]
id = "repo"
type = "path"
path = "shared-skills"
"#,
            ),
        )
        .unwrap();
        let user = parse_config_str(
            ConfigLayer::User,
            &user_config,
            &minimal_config(
                "user",
                r#"
[[skill_sources]]
id = "repo"
type = "path"
path = "user-skills"
"#,
            ),
        )
        .unwrap();

        let selection = resolve_skill_selection(&[
            SkillSelectionInput {
                config_file: &shared_config,
                config: &shared,
            },
            SkillSelectionInput {
                config_file: &user_config,
                config: &user,
            },
        ])
        .unwrap();

        assert_eq!(selection.selected_skills.len(), 2);
        assert_eq!(
            selection.selected_skills[0].layer,
            ConfigLayer::SharedProject
        );
        assert_eq!(
            selection.selected_skills[0].source_skill_name,
            "shared-only"
        );
        assert_eq!(selection.selected_skills[1].layer, ConfigLayer::User);
        assert_eq!(selection.selected_skills[1].source_skill_name, "user-only");
    }

    #[test]
    fn skill_selection_missing_include_returns_error() {
        let tempdir = TempDir::new().unwrap();
        let skills_dir = tempdir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        write_skill(&skills_dir.join("present"));

        let config_file = tempdir.path().join("agentcfg.toml");
        let contents = minimal_config(
            "user",
            r#"
[[skill_sources]]
id = "personal"
type = "path"
path = "skills"
include = ["missing"]
"#,
        );

        let error = parse_and_select(ConfigLayer::User, &config_file, &contents).unwrap_err();
        let SkillSelectionError::InvalidSelection {
            missing_included_skills,
            ..
        } = invalid_selection(error)
        else {
            panic!("expected InvalidSelection");
        };

        assert_eq!(missing_included_skills.len(), 1);
        assert_eq!(missing_included_skills[0].layer, ConfigLayer::User);
        assert_eq!(missing_included_skills[0].skill_source_id, "personal");
        assert_eq!(missing_included_skills[0].source_skill_name, "missing");
        assert_eq!(
            missing_included_skills[0].discovery_depth,
            DiscoveryDepth::DEFAULT
        );
    }

    #[test]
    fn skill_selection_collects_missing_includes_across_inputs() {
        let tempdir = TempDir::new().unwrap();
        let skills_a = tempdir.path().join("skills-a");
        let skills_b = tempdir.path().join("skills-b");
        fs::create_dir_all(&skills_a).unwrap();
        fs::create_dir_all(&skills_b).unwrap();

        let config_a = tempdir.path().join("a.toml");
        let config_b = tempdir.path().join("b.toml");

        let config_a_parsed = parse_config_str(
            ConfigLayer::User,
            &config_a,
            &minimal_config(
                "user",
                r#"
[[skill_sources]]
id = "src-a"
type = "path"
path = "skills-a"
include = ["missing-a"]
"#,
            ),
        )
        .unwrap();
        let config_b_parsed = parse_config_str(
            ConfigLayer::UserProject,
            &config_b,
            &minimal_config(
                "user-project",
                r#"
[[skill_sources]]
id = "src-b"
type = "path"
path = "skills-b"
include = ["missing-b"]
"#,
            ),
        )
        .unwrap();

        let error = resolve_skill_selection(&[
            SkillSelectionInput {
                config_file: &config_a,
                config: &config_a_parsed,
            },
            SkillSelectionInput {
                config_file: &config_b,
                config: &config_b_parsed,
            },
        ])
        .unwrap_err();

        let SkillSelectionError::InvalidSelection {
            missing_included_skills,
            ..
        } = invalid_selection(error)
        else {
            panic!("expected InvalidSelection");
        };
        assert_eq!(missing_included_skills.len(), 2);
        assert!(missing_included_skills.iter().any(
            |entry| entry.skill_source_id == "src-a" && entry.source_skill_name == "missing-a"
        ));
        assert!(missing_included_skills.iter().any(
            |entry| entry.skill_source_id == "src-b" && entry.source_skill_name == "missing-b"
        ));
    }

    #[test]
    fn skill_selection_empty_discovery_warning_for_implicit_selection() {
        let tempdir = TempDir::new().unwrap();
        let skills_dir = tempdir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();

        let config_file = tempdir.path().join("agentcfg.toml");
        let contents = minimal_config(
            "user",
            r#"
[[skill_sources]]
id = "personal"
type = "path"
path = "skills"
"#,
        );

        let selection = parse_and_select(ConfigLayer::User, &config_file, &contents).unwrap();

        assert!(selection.selected_skills.is_empty());
        assert_eq!(selection.warnings.len(), 1);
        assert!(matches!(
            selection.warnings[0],
            SkillSelectionWarning::EmptyDiscovery(EmptyDiscovery {
                layer: ConfigLayer::User,
                ref skill_source_id,
                ref resolved_root,
                discovery_depth,
            }) if skill_source_id == "personal"
                && resolved_root == &skills_dir
                && discovery_depth == DiscoveryDepth::DEFAULT
        ));
    }

    #[test]
    fn skill_selection_no_warning_for_zero_configured_skill_sources() {
        let tempdir = TempDir::new().unwrap();
        let config_file = tempdir.path().join("agentcfg.toml");
        let contents = minimal_config("user", "");

        let selection = parse_and_select(ConfigLayer::User, &config_file, &contents).unwrap();

        assert!(selection.selected_skills.is_empty());
        assert!(selection.warnings.is_empty());
    }

    #[test]
    fn skill_selection_group_selects_discovered_skills() {
        let tempdir = TempDir::new().unwrap();
        let skills_dir = tempdir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        write_skill(&skills_dir.join("alpha"));
        write_skill(&skills_dir.join("beta"));
        write_skills_toml(
            &skills_dir,
            r#"
[groups]
design = ["beta", "alpha"]
"#,
        );

        let config_file = tempdir.path().join("agentcfg.toml");
        let contents = minimal_config(
            "user",
            r#"
[[skill_sources]]
id = "personal"
type = "path"
path = "skills"
groups = ["design"]
"#,
        );

        let selection = parse_and_select(ConfigLayer::User, &config_file, &contents).unwrap();

        let names: Vec<_> = selection
            .selected_skills
            .iter()
            .map(|skill| skill.source_skill_name.as_str())
            .collect();
        assert_eq!(names, vec!["alpha", "beta"]);
    }

    #[test]
    fn skill_selection_include_and_groups_union_and_deduplicate() {
        let tempdir = TempDir::new().unwrap();
        let skills_dir = tempdir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        write_skill(&skills_dir.join("alpha"));
        write_skill(&skills_dir.join("beta"));
        write_skills_toml(
            &skills_dir,
            r#"
[groups]
design = ["beta"]
"#,
        );

        let config_file = tempdir.path().join("agentcfg.toml");
        let contents = minimal_config(
            "user",
            r#"
[[skill_sources]]
id = "personal"
type = "path"
path = "skills"
include = ["alpha"]
groups = ["design"]
"#,
        );

        let selection = parse_and_select(ConfigLayer::User, &config_file, &contents).unwrap();

        let names: Vec<_> = selection
            .selected_skills
            .iter()
            .map(|skill| skill.source_skill_name.as_str())
            .collect();
        assert_eq!(names, vec!["alpha", "beta"]);
    }

    #[test]
    fn skill_selection_missing_skills_toml_reports_absent_cause() {
        let tempdir = TempDir::new().unwrap();
        let skills_dir = tempdir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        write_skill(&skills_dir.join("alpha"));

        let config_file = tempdir.path().join("agentcfg.toml");
        let contents = minimal_config(
            "user",
            r#"
[[skill_sources]]
id = "personal"
type = "path"
path = "skills"
groups = ["design"]
"#,
        );

        let error = parse_and_select(ConfigLayer::User, &config_file, &contents).unwrap_err();
        let SkillSelectionError::InvalidSelection {
            missing_skill_groups,
            ..
        } = invalid_selection(error)
        else {
            panic!("expected InvalidSelection");
        };

        assert_eq!(missing_skill_groups.len(), 1);
        assert_eq!(missing_skill_groups[0].skill_group_name, "design");
        assert_eq!(
            missing_skill_groups[0].cause,
            MissingSkillGroupCause::SkillsTomlAbsent
        );
    }

    #[test]
    fn skill_selection_empty_skills_toml_reports_present_cause_for_selected_group() {
        let tempdir = TempDir::new().unwrap();
        let skills_dir = tempdir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        write_skill(&skills_dir.join("alpha"));
        write_skills_toml(&skills_dir, "");

        let config_file = tempdir.path().join("agentcfg.toml");
        let contents = minimal_config(
            "user",
            r#"
[[skill_sources]]
id = "personal"
type = "path"
path = "skills"
groups = ["design"]
"#,
        );

        let error = parse_and_select(ConfigLayer::User, &config_file, &contents).unwrap_err();
        let SkillSelectionError::InvalidSelection {
            missing_skill_groups,
            ..
        } = invalid_selection(error)
        else {
            panic!("expected InvalidSelection");
        };

        assert_eq!(missing_skill_groups.len(), 1);
        assert_eq!(
            missing_skill_groups[0].cause,
            MissingSkillGroupCause::SkillsTomlPresent
        );
    }

    #[test]
    fn skill_selection_include_only_does_not_read_invalid_skills_toml() {
        let tempdir = TempDir::new().unwrap();
        let skills_dir = tempdir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        write_skill(&skills_dir.join("alpha"));
        write_skills_toml(&skills_dir, "not valid toml [[[");

        let config_file = tempdir.path().join("agentcfg.toml");
        let contents = minimal_config(
            "user",
            r#"
[[skill_sources]]
id = "personal"
type = "path"
path = "skills"
include = ["alpha"]
"#,
        );

        let selection = parse_and_select(ConfigLayer::User, &config_file, &contents).unwrap();
        assert_eq!(selection.selected_skills.len(), 1);
    }

    #[test]
    fn skill_selection_implicit_all_does_not_read_invalid_skills_toml() {
        let tempdir = TempDir::new().unwrap();
        let skills_dir = tempdir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        write_skill(&skills_dir.join("alpha"));
        write_skills_toml(&skills_dir, "not valid toml [[[");

        let config_file = tempdir.path().join("agentcfg.toml");
        let contents = minimal_config(
            "user",
            r#"
[[skill_sources]]
id = "personal"
type = "path"
path = "skills"
"#,
        );

        let selection = parse_and_select(ConfigLayer::User, &config_file, &contents).unwrap();
        assert_eq!(selection.selected_skills.len(), 1);
    }

    #[test]
    fn skill_selection_selected_group_with_unknown_metadata_key_fails() {
        let tempdir = TempDir::new().unwrap();
        let skills_dir = tempdir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        write_skill(&skills_dir.join("alpha"));
        write_skills_toml(&skills_dir, "version = 1");

        let config_file = tempdir.path().join("agentcfg.toml");
        let contents = minimal_config(
            "user",
            r#"
[[skill_sources]]
id = "personal"
type = "path"
path = "skills"
groups = ["design"]
"#,
        );

        let error = parse_and_select(ConfigLayer::User, &config_file, &contents).unwrap_err();
        let SkillSelectionError::InvalidSelection {
            metadata_parse_errors,
            ..
        } = invalid_selection(error)
        else {
            panic!("expected InvalidSelection");
        };

        assert_eq!(metadata_parse_errors.len(), 1);
        assert_eq!(metadata_parse_errors[0].layer, ConfigLayer::User);
        assert_eq!(metadata_parse_errors[0].skill_source_id, "personal");
        assert_eq!(
            metadata_parse_errors[0].skills_toml,
            skills_dir.join("skills.toml")
        );
    }

    #[test]
    fn skill_selection_invalid_metadata_reports_invalid_group_definition() {
        let tempdir = TempDir::new().unwrap();
        let skills_dir = tempdir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        write_skill(&skills_dir.join("alpha"));
        write_skills_toml(
            &skills_dir,
            r#"
[groups]
design = []
"#,
        );

        let config_file = tempdir.path().join("agentcfg.toml");
        let contents = minimal_config(
            "user",
            r#"
[[skill_sources]]
id = "personal"
type = "path"
path = "skills"
groups = ["design"]
"#,
        );

        let error = parse_and_select(ConfigLayer::User, &config_file, &contents).unwrap_err();
        let SkillSelectionError::InvalidSelection {
            invalid_skill_group_definitions,
            ..
        } = invalid_selection(error)
        else {
            panic!("expected InvalidSelection");
        };

        assert!(invalid_skill_group_definitions.iter().any(|entry| {
            entry.skill_group_name.as_deref() == Some("design")
                && entry.reason == InvalidSkillGroupDefinitionReason::EmptyMemberList
        }));
    }

    #[test]
    fn skill_selection_rejects_whitespace_group_name_in_metadata() {
        let tempdir = TempDir::new().unwrap();
        let skills_dir = tempdir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        write_skill(&skills_dir.join("alpha"));
        write_skills_toml(
            &skills_dir,
            r#"
[groups]
" design" = ["alpha"]
"#,
        );

        let config_file = tempdir.path().join("agentcfg.toml");
        let contents = minimal_config(
            "user",
            r#"
[[skill_sources]]
id = "personal"
type = "path"
path = "skills"
groups = ["design"]
"#,
        );

        let error = parse_and_select(ConfigLayer::User, &config_file, &contents).unwrap_err();
        let SkillSelectionError::InvalidSelection {
            invalid_skill_group_definitions,
            ..
        } = invalid_selection(error)
        else {
            panic!("expected InvalidSelection");
        };

        assert!(invalid_skill_group_definitions.iter().any(|entry| {
            entry.skill_group_name.as_deref() == Some(" design")
                && entry.reason == InvalidSkillGroupDefinitionReason::WhitespaceGroupName
        }));
    }

    #[test]
    fn skill_selection_rejects_whitespace_group_member_in_metadata() {
        let tempdir = TempDir::new().unwrap();
        let skills_dir = tempdir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        write_skill(&skills_dir.join("alpha"));
        write_skills_toml(
            &skills_dir,
            r#"
[groups]
design = [" alpha"]
"#,
        );

        let config_file = tempdir.path().join("agentcfg.toml");
        let contents = minimal_config(
            "user",
            r#"
[[skill_sources]]
id = "personal"
type = "path"
path = "skills"
groups = ["design"]
"#,
        );

        let error = parse_and_select(ConfigLayer::User, &config_file, &contents).unwrap_err();
        let SkillSelectionError::InvalidSelection {
            invalid_skill_group_definitions,
            ..
        } = invalid_selection(error)
        else {
            panic!("expected InvalidSelection");
        };

        assert!(invalid_skill_group_definitions.iter().any(|entry| {
            entry.skill_group_name.as_deref() == Some("design")
                && entry.reason == InvalidSkillGroupDefinitionReason::WhitespaceMember
        }));
    }

    #[test]
    fn skill_selection_rejects_duplicate_group_member_in_metadata() {
        let tempdir = TempDir::new().unwrap();
        let skills_dir = tempdir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        write_skill(&skills_dir.join("alpha"));
        write_skills_toml(
            &skills_dir,
            r#"
[groups]
design = ["alpha", "alpha"]
"#,
        );

        let config_file = tempdir.path().join("agentcfg.toml");
        let contents = minimal_config(
            "user",
            r#"
[[skill_sources]]
id = "personal"
type = "path"
path = "skills"
groups = ["design"]
"#,
        );

        let error = parse_and_select(ConfigLayer::User, &config_file, &contents).unwrap_err();
        let SkillSelectionError::InvalidSelection {
            invalid_skill_group_definitions,
            ..
        } = invalid_selection(error)
        else {
            panic!("expected InvalidSelection");
        };

        assert!(invalid_skill_group_definitions.iter().any(|entry| {
            entry.skill_group_name.as_deref() == Some("design")
                && entry.reason == InvalidSkillGroupDefinitionReason::DuplicateMember
        }));
    }

    #[test]
    fn skill_selection_duplicate_group_keys_report_metadata_parse_context() {
        let tempdir = TempDir::new().unwrap();
        let skills_dir = tempdir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        write_skill(&skills_dir.join("alpha"));
        write_skills_toml(
            &skills_dir,
            r#"
[groups]
design = ["alpha"]
design = ["beta"]
"#,
        );

        let config_file = tempdir.path().join("agentcfg.toml");
        let contents = minimal_config(
            "user-project",
            r#"
[[skill_sources]]
id = "team"
type = "path"
path = "skills"
groups = ["design"]
"#,
        );

        let error =
            parse_and_select(ConfigLayer::UserProject, &config_file, &contents).unwrap_err();

        let SkillSelectionError::InvalidSelection {
            metadata_parse_errors,
            ..
        } = invalid_selection(error)
        else {
            panic!("expected InvalidSelection");
        };

        assert_eq!(metadata_parse_errors.len(), 1);
        assert_eq!(metadata_parse_errors[0].layer, ConfigLayer::UserProject);
        assert_eq!(metadata_parse_errors[0].skill_source_id, "team");
        assert_eq!(
            metadata_parse_errors[0].skills_toml,
            skills_dir.join("skills.toml")
        );
    }

    #[test]
    fn skill_selection_malformed_metadata_skips_one_source_and_continues() {
        let tempdir = TempDir::new().unwrap();
        let bad_skills = tempdir.path().join("bad-skills");
        let other_skills = tempdir.path().join("other-skills");
        fs::create_dir_all(&bad_skills).unwrap();
        fs::create_dir_all(&other_skills).unwrap();
        write_skill(&bad_skills.join("alpha"));
        write_skills_toml(&bad_skills, "not valid toml [[");

        let config_file = tempdir.path().join("agentcfg.toml");
        let contents = minimal_config(
            "user",
            r#"
[[skill_sources]]
id = "bad"
type = "path"
path = "bad-skills"
groups = ["design"]

[[skill_sources]]
id = "other"
type = "path"
path = "other-skills"
include = ["missing"]
"#,
        );

        let error = parse_and_select(ConfigLayer::User, &config_file, &contents).unwrap_err();
        let SkillSelectionError::InvalidSelection {
            missing_included_skills,
            metadata_parse_errors,
            ..
        } = invalid_selection(error)
        else {
            panic!("expected InvalidSelection with collected source errors");
        };

        assert!(
            metadata_parse_errors
                .iter()
                .any(|entry| entry.skill_source_id == "bad")
        );
        assert!(
            missing_included_skills
                .iter()
                .any(|entry| entry.skill_source_id == "other")
        );
    }

    #[test]
    fn skill_selection_collects_same_source_include_and_metadata_errors() {
        let tempdir = TempDir::new().unwrap();
        let skills_dir = tempdir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        write_skill(&skills_dir.join("alpha"));
        write_skills_toml(&skills_dir, "not valid toml [[");

        let config_file = tempdir.path().join("agentcfg.toml");
        let contents = minimal_config(
            "user",
            r#"
[[skill_sources]]
id = "personal"
type = "path"
path = "skills"
include = ["missing"]
groups = ["design"]
"#,
        );

        let error = parse_and_select(ConfigLayer::User, &config_file, &contents).unwrap_err();
        let SkillSelectionError::InvalidSelection {
            missing_included_skills,
            metadata_parse_errors,
            ..
        } = invalid_selection(error)
        else {
            panic!("expected InvalidSelection with same-source errors");
        };

        assert_eq!(missing_included_skills.len(), 1);
        assert_eq!(missing_included_skills[0].skill_source_id, "personal");
        assert_eq!(missing_included_skills[0].source_skill_name, "missing");
        assert_eq!(metadata_parse_errors.len(), 1);
        assert_eq!(metadata_parse_errors[0].skill_source_id, "personal");
        assert_eq!(
            metadata_parse_errors[0].skills_toml,
            skills_dir.join("skills.toml")
        );
    }

    #[test]
    fn skill_selection_missing_group_member_returns_error() {
        let tempdir = TempDir::new().unwrap();
        let skills_dir = tempdir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        write_skill(&skills_dir.join("alpha"));
        write_skills_toml(
            &skills_dir,
            r#"
[groups]
design = ["missing"]
"#,
        );

        let config_file = tempdir.path().join("agentcfg.toml");
        let contents = minimal_config(
            "user",
            r#"
[[skill_sources]]
id = "personal"
type = "path"
path = "skills"
groups = ["design"]
"#,
        );

        let error = parse_and_select(ConfigLayer::User, &config_file, &contents).unwrap_err();
        let SkillSelectionError::InvalidSelection {
            missing_skill_group_members,
            ..
        } = invalid_selection(error)
        else {
            panic!("expected InvalidSelection");
        };

        assert_eq!(missing_skill_group_members.len(), 1);
        assert_eq!(missing_skill_group_members[0].source_skill_name, "missing");
    }

    #[test]
    fn skill_selection_unselected_group_may_reference_missing_member() {
        let tempdir = TempDir::new().unwrap();
        let skills_dir = tempdir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        write_skill(&skills_dir.join("alpha"));
        write_skills_toml(
            &skills_dir,
            r#"
[groups]
design = ["alpha"]
other = ["missing"]
"#,
        );

        let config_file = tempdir.path().join("agentcfg.toml");
        let contents = minimal_config(
            "user",
            r#"
[[skill_sources]]
id = "personal"
type = "path"
path = "skills"
groups = ["design"]
"#,
        );

        let selection = parse_and_select(ConfigLayer::User, &config_file, &contents).unwrap();
        assert_eq!(selection.selected_skills.len(), 1);
        assert_eq!(selection.selected_skills[0].source_skill_name, "alpha");
    }

    #[test]
    fn skill_selection_aggregate_errors_are_sorted_deterministically() {
        let tempdir = TempDir::new().unwrap();
        let skills_a = tempdir.path().join("skills-a");
        let skills_b = tempdir.path().join("skills-b");
        fs::create_dir_all(&skills_a).unwrap();
        fs::create_dir_all(&skills_b).unwrap();

        let config_a = tempdir.path().join("a.toml");
        let config_b = tempdir.path().join("b.toml");

        let config_a_parsed = parse_config_str(
            ConfigLayer::UserProject,
            &config_a,
            &minimal_config(
                "user-project",
                r#"
[[skill_sources]]
id = "src-b"
type = "path"
path = "skills-b"
include = ["missing-b"]
"#,
            ),
        )
        .unwrap();
        let config_b_parsed = parse_config_str(
            ConfigLayer::User,
            &config_b,
            &minimal_config(
                "user",
                r#"
[[skill_sources]]
id = "src-a"
type = "path"
path = "skills-a"
include = ["missing-a"]
"#,
            ),
        )
        .unwrap();

        let error = resolve_skill_selection(&[
            SkillSelectionInput {
                config_file: &config_a,
                config: &config_a_parsed,
            },
            SkillSelectionInput {
                config_file: &config_b,
                config: &config_b_parsed,
            },
        ])
        .unwrap_err();

        let SkillSelectionError::InvalidSelection {
            missing_included_skills,
            ..
        } = invalid_selection(error)
        else {
            panic!("expected InvalidSelection");
        };

        assert_eq!(
            missing_included_skills
                .iter()
                .map(|entry| (entry.layer, entry.skill_source_id.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (ConfigLayer::UserProject, "src-b"),
                (ConfigLayer::User, "src-a"),
            ]
        );
    }

    #[test]
    fn skill_selection_does_not_reject_duplicate_names_across_skill_sources() {
        let tempdir = TempDir::new().unwrap();
        let skills_a = tempdir.path().join("skills-a");
        let skills_b = tempdir.path().join("skills-b");
        fs::create_dir_all(&skills_a).unwrap();
        fs::create_dir_all(&skills_b).unwrap();
        write_skill(&skills_a.join("dup"));
        write_skill(&skills_b.join("dup"));

        let config_file = tempdir.path().join("agentcfg.toml");
        let contents = minimal_config(
            "user",
            r#"
[[skill_sources]]
id = "src-a"
type = "path"
path = "skills-a"

[[skill_sources]]
id = "src-b"
type = "path"
path = "skills-b"
"#,
        );

        let selection = parse_and_select(ConfigLayer::User, &config_file, &contents).unwrap();

        assert_eq!(selection.selected_skills.len(), 2);
        assert!(
            selection
                .selected_skills
                .iter()
                .all(|skill| skill.source_skill_name == "dup")
        );
        assert_ne!(
            selection.selected_skills[0].skill_source_id,
            selection.selected_skills[1].skill_source_id
        );
    }
}
