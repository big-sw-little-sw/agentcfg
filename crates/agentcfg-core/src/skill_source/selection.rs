//! **Skill Selection** for path **Skill Sources**: implicit all-skills, explicit `include`, and diagnostics.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::config::{Config, SkillSourceKind};
use crate::layer_level::ConfigLayer;
use crate::skill_source::path::{discover_skills_in_source, DiscoveredSkill};
use crate::skill_source::DiscoveryDepth;
use crate::{MissingIncludedSkill, Result, SkillSelectionError};

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
    let mut missing = Vec::new();

    for input in inputs {
        let layer = input.config.layer();
        for skill_source in input.config.skill_sources() {
            let SkillSourceKind::Path { path: configured_path } = skill_source.kind();

            let discovery = discover_skills_in_source(
                skill_source.id(),
                input.config_file,
                configured_path,
                skill_source.discovery_depth(),
            )?;

            let by_name: BTreeMap<&str, &DiscoveredSkill> = discovery
                .discovered_skills
                .iter()
                .map(|skill| (skill.source_skill_name.as_str(), skill))
                .collect();

            let include = skill_source.included_skill_names();
            let groups = skill_source.skill_group_names();

            if include.is_empty() && groups.is_empty() {
                if discovery.discovered_skills.is_empty() {
                    warnings.push(SkillSelectionWarning::EmptyDiscovery(EmptyDiscovery {
                        layer,
                        skill_source_id: skill_source.id().to_string(),
                        resolved_root: discovery.resolved_root,
                        discovery_depth: skill_source.discovery_depth(),
                    }));
                } else {
                    for skill in &discovery.discovered_skills {
                        selected_skills.push(selected_from_discovery(
                            layer,
                            skill_source.id(),
                            skill,
                        ));
                    }
                }
            } else if !include.is_empty() {
                for source_skill_name in include {
                    if let Some(skill) = by_name.get(source_skill_name.as_str()) {
                        selected_skills.push(selected_from_discovery(
                            layer,
                            skill_source.id(),
                            skill,
                        ));
                    } else {
                        missing.push(MissingIncludedSkill {
                            layer,
                            skill_source_id: skill_source.id().to_string(),
                            source_skill_name: source_skill_name.clone(),
                            resolved_root: discovery.resolved_root.clone(),
                            discovery_depth: skill_source.discovery_depth(),
                        });
                    }
                }
            }
        }
    }

    if !missing.is_empty() {
        return Err(SkillSelectionError::MissingIncludedSkills { missing }.into());
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
    use super::*;
    use crate::config::parse_config_str;
    use crate::Error;
    use std::fs;
    use tempfile::TempDir;

    const SKILL_FILE: &str = "SKILL.md";

    fn write_skill(skill_dir: &Path) {
        fs::create_dir_all(skill_dir).unwrap();
        fs::write(skill_dir.join(SKILL_FILE), "skill").unwrap();
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
        assert_eq!(selection.selected_skills[0].layer, ConfigLayer::SharedProject);
        assert_eq!(selection.selected_skills[0].source_skill_name, "shared-only");
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

        assert!(matches!(
            error,
            Error::SkillSelection(SkillSelectionError::MissingIncludedSkills { ref missing })
                if missing.len() == 1
                    && missing[0].layer == ConfigLayer::User
                    && missing[0].skill_source_id == "personal"
                    && missing[0].source_skill_name == "missing"
                    && missing[0].discovery_depth == DiscoveryDepth::DEFAULT
        ));
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

        let Error::SkillSelection(SkillSelectionError::MissingIncludedSkills { missing }) = error
        else {
            panic!("expected MissingIncludedSkills, got {error:?}");
        };
        assert_eq!(missing.len(), 2);
        assert!(
            missing
                .iter()
                .any(|entry| entry.skill_source_id == "src-a" && entry.source_skill_name == "missing-a")
        );
        assert!(
            missing
                .iter()
                .any(|entry| entry.skill_source_id == "src-b" && entry.source_skill_name == "missing-b")
        );
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
    fn skill_selection_groups_only_selects_nothing_without_warning() {
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

        let selection = parse_and_select(ConfigLayer::User, &config_file, &contents).unwrap();

        assert!(selection.selected_skills.is_empty());
        assert!(selection.warnings.is_empty());
    }

    #[test]
    fn skill_selection_include_and_groups_resolves_only_include() {
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
include = ["alpha"]
groups = ["design"]
"#,
        );

        let selection = parse_and_select(ConfigLayer::User, &config_file, &contents).unwrap();

        assert_eq!(selection.selected_skills.len(), 1);
        assert_eq!(selection.selected_skills[0].source_skill_name, "alpha");
        assert!(selection.warnings.is_empty());
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
