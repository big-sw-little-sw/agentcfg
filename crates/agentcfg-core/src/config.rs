//! V1 skill config parsing and validation.
//!
//! This module owns the persisted TOML shape and returns validated domain
//! models so workflow and source-resolution code do not need to inspect raw
//! config tables.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::scope::ConfigLayer;
use crate::{ConfigError, Error, Result};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    layer: ConfigLayer,
    sources: Vec<SkillSourceConfig>,
    skill_aliases: BTreeMap<String, String>,
    skills: SkillsConfig,
}

impl Config {
    pub fn layer(&self) -> ConfigLayer {
        self.layer
    }

    pub fn sources(&self) -> &[SkillSourceConfig] {
        &self.sources
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
    source: SkillSourceKind,
    include: Vec<String>,
    groups: Vec<String>,
}

impl SkillSourceConfig {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn source(&self) -> &SkillSourceKind {
        &self.source
    }

    pub fn include(&self) -> &[String] {
        &self.include
    }

    pub fn groups(&self) -> &[String] {
        &self.groups
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
    let scope = raw
        .scope
        .ok_or_else(|| missing_field(&path, layer, "scope"))?;

    if scope != layer.persisted_scope() {
        return Err(ConfigError::ScopeMismatch {
            path,
            expected_layer: layer,
            expected_scope: layer.persisted_scope(),
            actual_scope: scope,
        }
        .into());
    }

    let sources = validate_sources(&path, layer, raw.skill_sources)?;

    let skills = validate_skills(
        &path,
        layer,
        raw.skills
            .ok_or_else(|| missing_field(&path, layer, "skills"))?,
    )?;

    Ok(Config {
        layer,
        sources,
        skill_aliases: raw.skill_aliases,
        skills,
    })
}

fn validate_sources(
    path: &Path,
    layer: ConfigLayer,
    raw_sources: Vec<RawSkillSource>,
) -> Result<Vec<SkillSourceConfig>> {
    let mut ids = BTreeSet::new();
    let mut sources = Vec::with_capacity(raw_sources.len());

    for raw_source in raw_sources {
        let source = validate_source(path, layer, raw_source)?;
        if !ids.insert(source.id.clone()) {
            return Err(ConfigError::DuplicateSourceId {
                path: path.to_path_buf(),
                layer,
                source_id: source.id,
            }
            .into());
        }
        sources.push(source);
    }

    Ok(sources)
}

fn validate_source(
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

    let source = match kind.as_str() {
        "path" => {
            let source_path = raw
                .path
                .ok_or_else(|| missing_field(path, layer, "skill_sources[].path"))?;
            SkillSourceKind::Path { path: source_path }
        }
        _ => {
            return Err(ConfigError::UnsupportedSourceKind {
                path: path.to_path_buf(),
                layer,
                source_id: Some(id),
                kind,
            }
            .into());
        }
    };

    Ok(SkillSourceConfig {
        id,
        source,
        include: raw.include,
        groups: raw.groups,
    })
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
    #[serde(default)]
    include: Vec<String>,
    #[serde(default)]
    groups: Vec<String>,
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

    const VALID_SOURCE: &str = r#"
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
    fn parses_valid_shared_project_config() {
        let config = parse_layer_config(ConfigLayer::SharedProject, "shared-project");

        assert_eq!(config.layer(), ConfigLayer::SharedProject);
        assert_eq!(config.sources().len(), 1);
        assert_eq!(config.sources()[0].id(), "personal");
        assert_eq!(
            config.sources()[0].source(),
            &SkillSourceKind::Path {
                path: PathBuf::from("../skills")
            }
        );
        assert_eq!(config.sources()[0].include(), ["do-code-review"]);
        assert_eq!(config.sources()[0].groups(), ["design"]);
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
    fn rejects_scope_mismatch() {
        let error = parse_config_str(
            ConfigLayer::User,
            "user.toml",
            &config_contents("shared-project"),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            Error::Config(ConfigError::ScopeMismatch {
                expected_layer: ConfigLayer::User,
                expected_scope: "user",
                ref actual_scope,
                ..
            }) if actual_scope == "shared-project"
        ));
    }

    #[test]
    fn rejects_missing_top_level_scope() {
        let error = parse_config_str(ConfigLayer::User, "user.toml", VALID_SOURCE).unwrap_err();

        assert_missing_field(error, "scope");
    }

    #[test]
    fn rejects_missing_source_id() {
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
    fn rejects_empty_source_id() {
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
    fn rejects_duplicate_source_ids_after_trimming() {
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
            Error::Config(ConfigError::DuplicateSourceId {
                source_id,
                ..
            }) if source_id == "personal"
        ));
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
    fn rejects_unsupported_source_exclude() {
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
    fn rejects_unsupported_source_kind_before_git_sources_exist() {
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
            Error::Config(ConfigError::UnsupportedSourceKind {
                source_id: Some(source_id),
                kind,
                ..
            }) if source_id == "personal" && kind == "git"
        ));
        assert!(
            error
                .to_string()
                .contains("git source support is planned for a later phase")
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
        assert_eq!(config.sources()[0].id(), "personal");
    }

    fn parse_layer_config(layer: ConfigLayer, scope: &str) -> Config {
        parse_config_str(layer, "agentcfg.toml", &config_contents(scope)).unwrap()
    }

    fn config_contents(scope: &str) -> String {
        format!("scope = \"{scope}\"\n{VALID_SOURCE}")
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
