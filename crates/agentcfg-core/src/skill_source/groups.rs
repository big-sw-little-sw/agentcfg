//! Root `skills.toml` parsing for Skill Source-local **Skill Groups**.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::InvalidSkillGroupDefinitionReason;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SkillGroupsInSource {
    Absent {
        skills_toml: PathBuf,
    },
    Present {
        skills_toml: PathBuf,
        groups: BTreeMap<String, Vec<String>>,
    },
}

#[derive(Debug)]
pub(crate) enum SkillGroupsMetadataError {
    Io {
        skills_toml: PathBuf,
        source: std::io::Error,
    },
    Parse {
        skills_toml: PathBuf,
        source: toml::de::Error,
    },
    InvalidDefinitions(Vec<InvalidSkillGroupDefinitionFact>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InvalidSkillGroupDefinitionFact {
    pub skill_group_name: Option<String>,
    pub skills_toml: PathBuf,
    pub reason: InvalidSkillGroupDefinitionReason,
}

pub(crate) fn read_skill_groups_in_source(
    resolved_root: &Path,
) -> std::result::Result<SkillGroupsInSource, SkillGroupsMetadataError> {
    let skills_toml = resolved_root.join("skills.toml");
    if !skills_toml.is_file() {
        return Ok(SkillGroupsInSource::Absent { skills_toml });
    }

    let contents =
        fs::read_to_string(&skills_toml).map_err(|source| SkillGroupsMetadataError::Io {
            skills_toml: skills_toml.clone(),
            source,
        })?;

    let groups = if contents.trim().is_empty() {
        BTreeMap::new()
    } else {
        let raw = toml::from_str::<RawSkillsToml>(&contents).map_err(|source| {
            SkillGroupsMetadataError::Parse {
                skills_toml: skills_toml.clone(),
                source,
            }
        })?;
        validate_groups_metadata(&skills_toml, raw.groups.unwrap_or_default())?
    };

    Ok(SkillGroupsInSource::Present {
        skills_toml,
        groups,
    })
}

fn validate_groups_metadata(
    skills_toml: &Path,
    groups: BTreeMap<String, Vec<String>>,
) -> std::result::Result<BTreeMap<String, Vec<String>>, SkillGroupsMetadataError> {
    let mut validated = BTreeMap::new();
    let mut invalid = Vec::new();

    for (group_name, members) in groups {
        if group_name.is_empty() {
            invalid.push(invalid_definition(
                skills_toml,
                None,
                InvalidSkillGroupDefinitionReason::EmptyGroupName,
            ));
            continue;
        }
        if group_name != group_name.trim() {
            invalid.push(invalid_definition(
                skills_toml,
                Some(group_name),
                InvalidSkillGroupDefinitionReason::WhitespaceGroupName,
            ));
            continue;
        }
        if members.is_empty() {
            invalid.push(invalid_definition(
                skills_toml,
                Some(group_name),
                InvalidSkillGroupDefinitionReason::EmptyMemberList,
            ));
            continue;
        }

        let mut seen_members = BTreeSet::new();
        let mut member_errors = false;
        for member in members {
            if member.is_empty() {
                invalid.push(invalid_definition(
                    skills_toml,
                    Some(group_name.clone()),
                    InvalidSkillGroupDefinitionReason::EmptyMember,
                ));
                member_errors = true;
                continue;
            }
            if member != member.trim() {
                invalid.push(invalid_definition(
                    skills_toml,
                    Some(group_name.clone()),
                    InvalidSkillGroupDefinitionReason::WhitespaceMember,
                ));
                member_errors = true;
                continue;
            }
            if !seen_members.insert(member.clone()) {
                invalid.push(invalid_definition(
                    skills_toml,
                    Some(group_name.clone()),
                    InvalidSkillGroupDefinitionReason::DuplicateMember,
                ));
                member_errors = true;
            }
        }

        if member_errors {
            continue;
        }

        validated.insert(group_name, seen_members.into_iter().collect());
    }

    if !invalid.is_empty() {
        invalid.sort_by(|left, right| {
            (left.skill_group_name.as_deref(), left.reason)
                .cmp(&(right.skill_group_name.as_deref(), right.reason))
        });
        return Err(SkillGroupsMetadataError::InvalidDefinitions(invalid));
    }

    Ok(validated)
}

fn invalid_definition(
    skills_toml: &Path,
    skill_group_name: Option<String>,
    reason: InvalidSkillGroupDefinitionReason,
) -> InvalidSkillGroupDefinitionFact {
    InvalidSkillGroupDefinitionFact {
        skill_group_name,
        skills_toml: skills_toml.to_path_buf(),
        reason,
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSkillsToml {
    #[serde(default)]
    groups: Option<BTreeMap<String, Vec<String>>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn skill_source_groups_absent_when_skills_toml_missing() {
        let tempdir = TempDir::new().unwrap();
        let root = tempdir.path();

        let metadata = read_skill_groups_in_source(root).unwrap();

        assert!(matches!(
            metadata,
            SkillGroupsInSource::Absent { skills_toml } if skills_toml == root.join("skills.toml")
        ));
    }

    #[test]
    fn skill_source_groups_empty_file_defines_zero_groups() {
        let tempdir = TempDir::new().unwrap();
        let root = tempdir.path();
        fs::write(root.join("skills.toml"), "").unwrap();

        let metadata = read_skill_groups_in_source(root).unwrap();

        assert!(matches!(
            metadata,
            SkillGroupsInSource::Present { groups, .. } if groups.is_empty()
        ));
    }

    #[test]
    fn skill_source_groups_parses_valid_groups() {
        let tempdir = TempDir::new().unwrap();
        let root = tempdir.path();
        fs::write(
            root.join("skills.toml"),
            r#"
[groups]
design = ["alpha", "beta"]
"#,
        )
        .unwrap();

        let metadata = read_skill_groups_in_source(root).unwrap();

        let SkillGroupsInSource::Present { groups, .. } = metadata else {
            panic!("expected present metadata");
        };
        assert_eq!(
            groups.get("design").map(|members| members.as_slice()),
            Some(["alpha".to_string(), "beta".to_string()].as_slice())
        );
    }

    #[test]
    fn skill_source_groups_rejects_unknown_top_level_key() {
        let tempdir = TempDir::new().unwrap();
        let root = tempdir.path();
        fs::write(
            root.join("skills.toml"),
            r#"
version = 1
"#,
        )
        .unwrap();

        let error = read_skill_groups_in_source(root).unwrap_err();

        assert!(matches!(error, SkillGroupsMetadataError::Parse { .. }));
    }

    #[test]
    fn skill_source_groups_rejects_empty_group_name() {
        let tempdir = TempDir::new().unwrap();
        let root = tempdir.path();
        fs::write(
            root.join("skills.toml"),
            r#"
[groups]
"" = ["alpha"]
"#,
        )
        .unwrap();

        let error = read_skill_groups_in_source(root).unwrap_err();

        let SkillGroupsMetadataError::InvalidDefinitions(invalid_skill_group_definitions) = error
        else {
            panic!("expected invalid metadata definitions, got {error:?}");
        };
        assert!(
            invalid_skill_group_definitions
                .iter()
                .any(|entry| { entry.reason == InvalidSkillGroupDefinitionReason::EmptyGroupName })
        );
    }
}
