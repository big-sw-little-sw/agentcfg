//! V1 skill config parsing and validation.
//!
//! Config declares **Skill Sources** (`[[skill_sources]]`), **Skill Selection**
//! (`include` selects **Included Skills**; `groups` selects **Skill Groups**),
//! and **Skill Aliases** that map a Source Skill Name to a **Discovery Name**.
//!
//! This module owns the persisted TOML shape and returns validated domain models
//! so workflow and Skill Source resolution code do not need to inspect raw config tables.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::layer_level::ConfigLayer;
use crate::{ConfigError, Error, Result};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    layer: ConfigLayer,
    skill_sources: Vec<SkillSourceConfig>,
    skill_aliases: BTreeMap<String, String>,
    skills: SkillsConfig,
}

impl Config {
    pub fn layer(&self) -> ConfigLayer {
        self.layer
    }

    pub fn skill_sources(&self) -> &[SkillSourceConfig] {
        &self.skill_sources
    }

    pub fn skill_aliases(&self) -> &BTreeMap<String, String> {
        &self.skill_aliases
    }

    pub fn skills(&self) -> &SkillsConfig {
        &self.skills
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillSourceConfig {
    id: String,
    kind: SkillSourceKind,
    included_skill_names: Vec<String>,
    skill_group_names: Vec<String>,
}

impl SkillSourceConfig {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn kind(&self) -> &SkillSourceKind {
        &self.kind
    }

    pub fn included_skill_names(&self) -> &[String] {
        &self.included_skill_names
    }

    pub fn skill_group_names(&self) -> &[String] {
        &self.skill_group_names
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SkillSourceKind {
    Path { path: PathBuf },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillsConfig {
    clients: ClientSelection,
}

impl SkillsConfig {
    pub fn clients(&self) -> &ClientSelection {
        &self.clients
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClientSelection {
    Explicit(Vec<String>),
    AllSupported,
}

pub fn parse_config_str(
    layer: ConfigLayer,
    path: impl Into<PathBuf>,
    contents: &str,
) -> Result<Config> {
    let path = path.into();
    let raw = toml::from_str::<RawConfig>(contents).map_err(|source| ConfigError::Parse {
        path: path.clone(),
        source,
    })?;

    validate_config(layer, path, raw)
}

pub fn load_config(layer: ConfigLayer, path: impl AsRef<Path>) -> Result<Config> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })?;

    parse_config_str(layer, path, &contents)
}

fn validate_config(layer: ConfigLayer, path: PathBuf, raw: RawConfig) -> Result<Config> {
    let persisted_scope_value = raw
        .scope
        .ok_or_else(|| missing_field(&path, layer, "scope"))?;

    if persisted_scope_value != layer.persisted_scope_value() {
        return Err(ConfigError::PersistedScopeValueMismatch {
            path,
            expected_layer: layer,
            expected_persisted_scope_value: layer.persisted_scope_value(),
            actual_persisted_scope_value: persisted_scope_value,
        }
        .into());
    }

    let skill_sources = validate_skill_sources(&path, layer, raw.skill_sources)?;
    let skill_aliases = validate_skill_aliases(&path, layer, raw.skill_aliases, &skill_sources)?;

    let skills = validate_skills(
        &path,
        layer,
        raw.skills
            .ok_or_else(|| missing_field(&path, layer, "skills"))?,
    )?;

    Ok(Config {
        layer,
        skill_sources,
        skill_aliases,
        skills,
    })
}

fn validate_skill_sources(
    path: &Path,
    layer: ConfigLayer,
    raw_skill_sources: Vec<RawSkillSource>,
) -> Result<Vec<SkillSourceConfig>> {
    let mut ids = BTreeSet::new();
    let mut skill_sources = Vec::with_capacity(raw_skill_sources.len());

    for raw_skill_source in raw_skill_sources {
        let skill_source = validate_skill_source(path, layer, raw_skill_source)?;
        if !ids.insert(skill_source.id.clone()) {
            return Err(ConfigError::DuplicateSkillSourceId {
                path: path.to_path_buf(),
                layer,
                skill_source_id: skill_source.id,
            }
            .into());
        }
        skill_sources.push(skill_source);
    }

    Ok(skill_sources)
}

fn validate_skill_source(
    path: &Path,
    layer: ConfigLayer,
    raw: RawSkillSource,
) -> Result<SkillSourceConfig> {
    let id = raw
        .id
        .ok_or_else(|| missing_field(path, layer, "skill_sources[].id"))?;
    let id = id.trim().to_string();

    if id.is_empty() {
        return Err(empty_field(path, layer, "skill_sources[].id"));
    }

    if raw.exclude.is_some() {
        return Err(ConfigError::UnsupportedField {
            path: path.to_path_buf(),
            layer,
            field: "skill_sources[].exclude",
        }
        .into());
    }

    let kind = raw
        .kind
        .ok_or_else(|| missing_field(path, layer, "skill_sources[].type"))?;

    let skill_source_kind = match kind.as_str() {
        "path" => {
            let source_path = raw
                .path
                .ok_or_else(|| missing_field(path, layer, "skill_sources[].path"))?;
            SkillSourceKind::Path { path: source_path }
        }
        _ => {
            return Err(ConfigError::UnsupportedSkillSourceKind {
                path: path.to_path_buf(),
                layer,
                skill_source_id: Some(id),
                kind,
            }
            .into());
        }
    };

    let included_skill_names =
        validate_optional_list(path, layer, "skill_sources[].include", raw.include)?;
    let skill_group_names =
        validate_optional_list(path, layer, "skill_sources[].groups", raw.groups)?;

    Ok(SkillSourceConfig {
        id,
        kind: skill_source_kind,
        included_skill_names,
        skill_group_names,
    })
}

fn validate_optional_list(
    path: &Path,
    layer: ConfigLayer,
    field: &'static str,
    value: Option<Vec<String>>,
) -> Result<Vec<String>> {
    match value {
        Some(values) if values.is_empty() => Err(empty_field(path, layer, field)),
        Some(values) => Ok(values),
        None => Ok(Vec::new()),
    }
}

fn validate_skill_aliases(
    path: &Path,
    layer: ConfigLayer,
    raw_aliases: BTreeMap<String, String>,
    skill_sources: &[SkillSourceConfig],
) -> Result<BTreeMap<String, String>> {
    let skill_source_ids = skill_sources
        .iter()
        .map(|skill_source| skill_source.id())
        .collect::<BTreeSet<_>>();

    for (skill_alias_key, discovery_name) in &raw_aliases {
        let Some((skill_source_id, source_skill_name)) = skill_alias_key.split_once(':') else {
            return Err(ConfigError::InvalidSkillAliasKey {
                path: path.to_path_buf(),
                layer,
                skill_alias_key: skill_alias_key.clone(),
            }
            .into());
        };

        if skill_source_id.trim().is_empty() || source_skill_name.trim().is_empty() {
            return Err(ConfigError::InvalidSkillAliasKey {
                path: path.to_path_buf(),
                layer,
                skill_alias_key: skill_alias_key.clone(),
            }
            .into());
        }

        if !skill_source_ids.contains(skill_source_id) {
            return Err(ConfigError::UnknownSkillAliasSkillSource {
                path: path.to_path_buf(),
                layer,
                skill_alias_key: skill_alias_key.clone(),
                skill_source_id: skill_source_id.to_string(),
            }
            .into());
        }

        if discovery_name.trim().is_empty() {
            return Err(empty_field(path, layer, "skill_aliases[]"));
        }
    }

    Ok(raw_aliases)
}

fn validate_skills(path: &Path, layer: ConfigLayer, raw: RawSkills) -> Result<SkillsConfig> {
    let clients = raw
        .clients
        .ok_or_else(|| missing_field(path, layer, "skills.clients"))?;

    let clients = match clients {
        RawClientSelection::String(value) if value == "all" => ClientSelection::AllSupported,
        RawClientSelection::String(value) => {
            return Err(ConfigError::InvalidFieldValue {
                path: path.to_path_buf(),
                layer,
                field: "skills.clients",
                value,
            }
            .into());
        }
        RawClientSelection::List(clients) if clients.is_empty() => {
            return Err(empty_field(path, layer, "skills.clients"));
        }
        RawClientSelection::List(clients) => ClientSelection::Explicit(clients),
    };

    Ok(SkillsConfig { clients })
}

fn missing_field(path: &Path, layer: ConfigLayer, field: &'static str) -> Error {
    ConfigError::MissingRequiredField {
        path: path.to_path_buf(),
        layer,
        field,
    }
    .into()
}

fn empty_field(path: &Path, layer: ConfigLayer, field: &'static str) -> Error {
    ConfigError::EmptyRequiredField {
        path: path.to_path_buf(),
        layer,
        field,
    }
    .into()
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    scope: Option<String>,
    #[serde(default)]
    skill_sources: Vec<RawSkillSource>,
    #[serde(default)]
    skill_aliases: BTreeMap<String, String>,
    skills: Option<RawSkills>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSkillSource {
    id: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
    path: Option<PathBuf>,
    include: Option<Vec<String>>,
    groups: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSkills {
    clients: Option<RawClientSelection>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawClientSelection {
    String(String),
    List(Vec<String>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_paths::ConfigFilePaths;
    use std::io::Write;

    const VALID_SKILL_SOURCE_CONFIG: &str = r#"
[[skill_sources]]
id = "personal"
type = "path"
path = "../skills"
include = ["do-code-review"]
groups = ["design"]

[skill_aliases]
"personal:legacy-review" = "code-review"

[skills]
clients = ["codex", "claude", "opencode"]
"#;

    #[test]
    fn skill_source_config_parses_path_skill_source() {
        let config = parse_layer_config(ConfigLayer::SharedProject, "shared-project");

        let skill_source = &config.skill_sources()[0];
        assert_eq!(skill_source.id(), "personal");
        assert!(matches!(
            skill_source.kind(),
            SkillSourceKind::Path { path } if path == Path::new("../skills")
        ));
    }

    #[test]
    fn skill_selection_include_and_groups_are_preserved() {
        let config = parse_layer_config(ConfigLayer::SharedProject, "shared-project");

        assert_eq!(
            config.skill_sources()[0].included_skill_names(),
            ["do-code-review"]
        );
        assert_eq!(config.skill_sources()[0].skill_group_names(), ["design"]);
    }

    #[test]
    fn skill_alias_sets_discovery_name() {
        let config = parse_layer_config(ConfigLayer::SharedProject, "shared-project");

        assert_eq!(
            config.skill_aliases().get("personal:legacy-review"),
            Some(&"code-review".to_string())
        );
    }

    #[test]
    fn parses_valid_shared_project_config() {
        let config = parse_layer_config(ConfigLayer::SharedProject, "shared-project");

        assert_eq!(config.layer(), ConfigLayer::SharedProject);
        assert_eq!(config.skill_sources().len(), 1);
        assert_eq!(config.skill_sources()[0].id(), "personal");
        assert_eq!(
            config.skill_sources()[0].kind(),
            &SkillSourceKind::Path {
                path: PathBuf::from("../skills")
            }
        );
        assert_eq!(
            config.skill_sources()[0].included_skill_names(),
            ["do-code-review"]
        );
        assert_eq!(config.skill_sources()[0].skill_group_names(), ["design"]);
        assert_eq!(
            config.skill_aliases().get("personal:legacy-review"),
            Some(&"code-review".to_string())
        );
        assert_eq!(
            config.skills().clients(),
            &ClientSelection::Explicit(vec![
                "codex".to_string(),
                "claude".to_string(),
                "opencode".to_string()
            ])
        );
    }

    #[test]
    fn parses_valid_user_project_config() {
        let config = parse_layer_config(ConfigLayer::UserProject, "user-project");

        assert_eq!(config.layer(), ConfigLayer::UserProject);
    }

    #[test]
    fn parses_valid_user_config() {
        let config = parse_layer_config(ConfigLayer::User, "user");

        assert_eq!(config.layer(), ConfigLayer::User);
    }

    #[test]
    fn wraps_toml_parse_failure_with_path_context() {
        let error = parse_config_str(ConfigLayer::User, "user.toml", "scope =").unwrap_err();

        assert!(matches!(
            error,
            Error::Config(ConfigError::Parse { ref path, .. }) if path == Path::new("user.toml")
        ));
    }

    #[test]
    fn rejects_persisted_scope_value_mismatch() {
        let error = parse_config_str(
            ConfigLayer::User,
            "user.toml",
            &config_contents("shared-project"),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            Error::Config(ConfigError::PersistedScopeValueMismatch {
                expected_layer: ConfigLayer::User,
                expected_persisted_scope_value: "user",
                ref actual_persisted_scope_value,
                ..
            }) if actual_persisted_scope_value == "shared-project"
        ));
    }

    #[test]
    fn rejects_missing_top_level_scope() {
        let error = parse_config_str(ConfigLayer::User, "user.toml", VALID_SKILL_SOURCE_CONFIG)
            .unwrap_err();

        assert_missing_field(error, "scope");
    }

    #[test]
    fn rejects_missing_skill_source_id() {
        let contents = r#"
scope = "user"

[[skill_sources]]
type = "path"
path = "../skills"

[skills]
clients = ["codex"]
"#;

        let error = parse_config_str(ConfigLayer::User, "user.toml", contents).unwrap_err();

        assert_missing_field(error, "skill_sources[].id");
    }

    #[test]
    fn rejects_empty_skill_source_id() {
        let contents = r#"
scope = "user"

[[skill_sources]]
id = "   "
type = "path"
path = "../skills"

[skills]
clients = ["codex"]
"#;

        let error = parse_config_str(ConfigLayer::User, "user.toml", contents).unwrap_err();

        assert_empty_field(error, "skill_sources[].id");
    }

    #[test]
    fn rejects_duplicate_skill_source_ids_after_trimming() {
        let contents = r#"
scope = "user"

[[skill_sources]]
id = "personal"
type = "path"
path = "../skills"

[[skill_sources]]
id = " personal "
type = "path"
path = "../other-skills"

[skills]
clients = ["codex"]
"#;

        let error = parse_config_str(ConfigLayer::User, "user.toml", contents).unwrap_err();

        assert!(matches!(
            error,
            Error::Config(ConfigError::DuplicateSkillSourceId {
                skill_source_id,
                ..
            }) if skill_source_id == "personal"
        ));
    }

    #[test]
    fn parses_omitted_skill_selection_as_empty_lists() {
        let contents = r#"
scope = "user"

[[skill_sources]]
id = "personal"
type = "path"
path = "../skills"

[skills]
clients = ["codex"]
"#;

        let config = parse_config_str(ConfigLayer::User, "user.toml", contents).unwrap();

        assert!(config.skill_sources()[0].included_skill_names().is_empty());
        assert!(config.skill_sources()[0].skill_group_names().is_empty());
    }

    #[test]
    fn rejects_explicit_empty_included_skills() {
        let contents = r#"
scope = "user"

[[skill_sources]]
id = "personal"
type = "path"
path = "../skills"
include = []

[skills]
clients = ["codex"]
"#;

        let error = parse_config_str(ConfigLayer::User, "user.toml", contents).unwrap_err();

        assert_empty_field(error, "skill_sources[].include");
    }

    #[test]
    fn rejects_explicit_empty_skill_groups() {
        let contents = r#"
scope = "user"

[[skill_sources]]
id = "personal"
type = "path"
path = "../skills"
groups = []

[skills]
clients = ["codex"]
"#;

        let error = parse_config_str(ConfigLayer::User, "user.toml", contents).unwrap_err();

        assert_empty_field(error, "skill_sources[].groups");
    }

    #[test]
    fn rejects_malformed_skill_alias_key() {
        let contents = r#"
scope = "user"

[[skill_sources]]
id = "personal"
type = "path"
path = "../skills"

[skill_aliases]
"legacy-review" = "code-review"

[skills]
clients = ["codex"]
"#;

        let error = parse_config_str(ConfigLayer::User, "user.toml", contents).unwrap_err();

        assert!(matches!(
            error,
            Error::Config(ConfigError::InvalidSkillAliasKey {
                skill_alias_key,
                ..
            }) if skill_alias_key == "legacy-review"
        ));
    }

    #[test]
    fn rejects_skill_alias_for_unknown_skill_source() {
        let contents = r#"
scope = "user"

[[skill_sources]]
id = "personal"
type = "path"
path = "../skills"

[skill_aliases]
"community:legacy-review" = "code-review"

[skills]
clients = ["codex"]
"#;

        let error = parse_config_str(ConfigLayer::User, "user.toml", contents).unwrap_err();

        assert!(matches!(
            error,
            Error::Config(ConfigError::UnknownSkillAliasSkillSource {
                skill_alias_key,
                skill_source_id,
                ..
            }) if skill_alias_key == "community:legacy-review" && skill_source_id == "community"
        ));
    }

    #[test]
    fn rejects_empty_skill_alias_discovery_name() {
        let contents = r#"
scope = "user"

[[skill_sources]]
id = "personal"
type = "path"
path = "../skills"

[skill_aliases]
"personal:legacy-review" = ""

[skills]
clients = ["codex"]
"#;

        let error = parse_config_str(ConfigLayer::User, "user.toml", contents).unwrap_err();

        assert_empty_field(error, "skill_aliases[]");
    }

    #[test]
    fn rejects_missing_skills_table() {
        let contents = r#"
scope = "user"

[[skill_sources]]
id = "personal"
type = "path"
path = "../skills"
"#;

        let error = parse_config_str(ConfigLayer::User, "user.toml", contents).unwrap_err();

        assert_missing_field(error, "skills");
    }

    #[test]
    fn rejects_missing_skills_clients() {
        let contents = r#"
scope = "user"

[skills]
"#;

        let error = parse_config_str(ConfigLayer::User, "user.toml", contents).unwrap_err();

        assert_missing_field(error, "skills.clients");
    }

    #[test]
    fn rejects_empty_skills_clients() {
        let contents = r#"
scope = "user"

[skills]
clients = []
"#;

        let error = parse_config_str(ConfigLayer::User, "user.toml", contents).unwrap_err();

        assert_empty_field(error, "skills.clients");
    }

    #[test]
    fn accepts_all_supported_clients() {
        let contents = r#"
scope = "user"

[skills]
clients = "all"
"#;

        let config = parse_config_str(ConfigLayer::User, "user.toml", contents).unwrap();

        assert_eq!(config.skills().clients(), &ClientSelection::AllSupported);
    }

    #[test]
    fn rejects_unsupported_skill_source_exclude() {
        let contents = r#"
scope = "user"

[[skill_sources]]
id = "personal"
type = "path"
path = "../skills"
exclude = ["draft"]

[skills]
clients = ["codex"]
"#;

        let error = parse_config_str(ConfigLayer::User, "user.toml", contents).unwrap_err();

        assert!(matches!(
            error,
            Error::Config(ConfigError::UnsupportedField {
                field: "skill_sources[].exclude",
                ..
            })
        ));
    }

    #[test]
    fn rejects_unsupported_skill_source_kind_before_git_sources_exist() {
        let contents = r#"
scope = "user"

[[skill_sources]]
id = "personal"
type = "git"
path = "../skills"

[skills]
clients = ["codex"]
"#;

        let error = parse_config_str(ConfigLayer::User, "user.toml", contents).unwrap_err();

        assert!(matches!(
            &error,
            Error::Config(ConfigError::UnsupportedSkillSourceKind {
                skill_source_id: Some(skill_source_id),
                kind,
                ..
            }) if skill_source_id == "personal" && kind == "git"
        ));
        assert!(
            error
                .to_string()
                .contains("git Skill Source support is planned for a later phase")
        );
    }

    #[test]
    fn load_config_reads_config_file_paths() {
        let tempdir = tempfile::tempdir().unwrap();
        let paths = ConfigFilePaths::for_shared_project(tempdir.path());

        let mut file = fs::File::create(paths.config_file()).unwrap();
        file.write_all(config_contents("shared-project").as_bytes())
            .unwrap();

        let config = load_config(paths.layer(), paths.config_file()).unwrap();

        assert_eq!(config.layer(), ConfigLayer::SharedProject);
        assert_eq!(config.skill_sources()[0].id(), "personal");
    }

    fn parse_layer_config(layer: ConfigLayer, persisted_scope_value: &str) -> Config {
        parse_config_str(
            layer,
            "agentcfg.toml",
            &config_contents(persisted_scope_value),
        )
        .unwrap()
    }

    fn config_contents(persisted_scope_value: &str) -> String {
        format!("scope = \"{persisted_scope_value}\"\n{VALID_SKILL_SOURCE_CONFIG}")
    }

    fn assert_missing_field(error: Error, expected_field: &'static str) {
        assert!(matches!(
            error,
            Error::Config(ConfigError::MissingRequiredField { field, .. })
                if field == expected_field
        ));
    }

    fn assert_empty_field(error: Error, expected_field: &'static str) {
        assert!(matches!(
            error,
            Error::Config(ConfigError::EmptyRequiredField { field, .. })
                if field == expected_field
        ));
    }
}
