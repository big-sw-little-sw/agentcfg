//! V1 skill config parsing and validation.
//!
//! This module owns the persisted TOML shape and returns validated domain
//! models so workflow and source-resolution code do not need to inspect raw
//! config tables.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::registry::normalize_client_id;
use crate::scope::ConfigLayer;
use crate::{ConfigError, Error, Result};

struct ValidationContext<'a> {
    path: &'a Path,
    layer: ConfigLayer,
}

impl ValidationContext<'_> {
    fn path_buf(&self) -> PathBuf {
        self.path.to_path_buf()
    }

    fn missing_field(&self, field: &'static str) -> Error {
        ConfigError::MissingRequiredField {
            path: self.path_buf(),
            layer: self.layer,
            field,
        }
        .into()
    }

    fn empty_field(&self, field: &'static str) -> Error {
        ConfigError::EmptyRequiredField {
            path: self.path_buf(),
            layer: self.layer,
            field,
        }
        .into()
    }
}

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
    let ctx = ValidationContext {
        path: &path,
        layer,
    };

    let scope = raw.scope.ok_or_else(|| ctx.missing_field("scope"))?;

    if ConfigLayer::from_persisted_scope(&scope).is_none() {
        return Err(ConfigError::InvalidFieldValue {
            path: ctx.path_buf(),
            layer: ctx.layer,
            field: "scope",
            value: scope,
        }
        .into());
    }

    if scope != layer.persisted_scope() {
        return Err(ConfigError::ScopeMismatch {
            path,
            expected_layer: layer,
            expected_scope: layer.persisted_scope(),
            actual_scope: scope,
        }
        .into());
    }

    let sources = validate_sources(&ctx, raw.skill_sources)?;
    let skill_aliases = validate_skill_aliases(&ctx, raw.skill_aliases, &sources)?;
    let skills = validate_skills(
        &ctx,
        raw.skills.ok_or_else(|| ctx.missing_field("skills"))?,
    )?;

    Ok(Config {
        layer,
        sources,
        skill_aliases,
        skills,
    })
}

fn validate_sources(
    ctx: &ValidationContext<'_>,
    raw_sources: Vec<RawSkillSource>,
) -> Result<Vec<SkillSourceConfig>> {
    let mut ids = BTreeSet::new();
    let mut sources = Vec::with_capacity(raw_sources.len());

    for raw_source in raw_sources {
        let source = validate_source(ctx, raw_source)?;
        if !ids.insert(source.id.clone()) {
            return Err(ConfigError::DuplicateSourceId {
                path: ctx.path_buf(),
                layer: ctx.layer,
                source_id: source.id,
            }
            .into());
        }
        sources.push(source);
    }

    Ok(sources)
}

fn validate_source(
    ctx: &ValidationContext<'_>,
    raw: RawSkillSource,
) -> Result<SkillSourceConfig> {
    let id = raw
        .id
        .ok_or_else(|| ctx.missing_field("skill_sources[].id"))?;
    let id = id.trim().to_string();

    if id.is_empty() {
        return Err(ctx.empty_field("skill_sources[].id"));
    }

    if raw.exclude.is_some() {
        return Err(ConfigError::UnsupportedField {
            path: ctx.path_buf(),
            layer: ctx.layer,
            field: "skill_sources[].exclude",
        }
        .into());
    }

    let kind = raw
        .kind
        .ok_or_else(|| ctx.missing_field("skill_sources[].type"))?;

    let source = match kind.as_str() {
        "path" => {
            let source_path = raw
                .path
                .ok_or_else(|| ctx.missing_field("skill_sources[].path"))?;
            SkillSourceKind::Path { path: source_path }
        }
        "git" => {
            return Err(ConfigError::UnsupportedSourceKind {
                path: ctx.path_buf(),
                layer: ctx.layer,
                source_id: Some(id),
                kind,
            }
            .into());
        }
        _ => {
            return Err(ConfigError::UnsupportedSourceKind {
                path: ctx.path_buf(),
                layer: ctx.layer,
                source_id: Some(id),
                kind,
            }
            .into());
        }
    };

    let include =
        validate_optional_list(ctx, "skill_sources[].include", raw.include)?;
    let groups = validate_optional_list(ctx, "skill_sources[].groups", raw.groups)?;

    Ok(SkillSourceConfig {
        id,
        source,
        include,
        groups,
    })
}

fn validate_optional_list(
    ctx: &ValidationContext<'_>,
    field: &'static str,
    value: Option<Vec<String>>,
) -> Result<Vec<String>> {
    match value {
        None => Ok(Vec::new()),
        Some(values) if values.is_empty() => Err(ctx.empty_field(field)),
        Some(values) => validate_non_empty_list_elements(ctx, field, values),
    }
}

fn validate_non_empty_list_elements(
    ctx: &ValidationContext<'_>,
    field: &'static str,
    values: Vec<String>,
) -> Result<Vec<String>> {
    let mut normalized = Vec::with_capacity(values.len());
    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ctx.empty_field(field));
        }
        normalized.push(trimmed.to_string());
    }
    Ok(normalized)
}

fn validate_skill_aliases(
    ctx: &ValidationContext<'_>,
    raw_aliases: BTreeMap<String, String>,
    sources: &[SkillSourceConfig],
) -> Result<BTreeMap<String, String>> {
    let source_ids = sources
        .iter()
        .map(|source| source.id())
        .collect::<BTreeSet<_>>();

    for (source_skill, installed_name) in &raw_aliases {
        let Some((source_id, skill_name)) = source_skill.split_once(':') else {
            return Err(ConfigError::InvalidAliasKey {
                path: ctx.path_buf(),
                layer: ctx.layer,
                alias_key: source_skill.clone(),
            }
            .into());
        };

        if source_id.trim().is_empty() || skill_name.trim().is_empty() {
            return Err(ConfigError::InvalidAliasKey {
                path: ctx.path_buf(),
                layer: ctx.layer,
                alias_key: source_skill.clone(),
            }
            .into());
        }

        if !source_ids.contains(source_id.trim()) {
            return Err(ConfigError::UnknownAliasSource {
                path: ctx.path_buf(),
                layer: ctx.layer,
                alias_key: source_skill.clone(),
                source_id: source_id.trim().to_string(),
            }
            .into());
        }

        if installed_name.trim().is_empty() {
            return Err(ctx.empty_field("skill_aliases[]"));
        }
    }

    Ok(raw_aliases)
}

fn validate_skills(ctx: &ValidationContext<'_>, raw: RawSkills) -> Result<SkillsConfig> {
    let clients = raw
        .clients
        .ok_or_else(|| ctx.missing_field("skills.clients"))?;

    let clients = match clients {
        RawClientSelection::String(value) if value.trim() == "all" => {
            ClientSelection::AllSupported
        }
        RawClientSelection::String(value) => {
            return Err(ConfigError::InvalidFieldValue {
                path: ctx.path_buf(),
                layer: ctx.layer,
                field: "skills.clients",
                value,
            }
            .into());
        }
        RawClientSelection::List(clients) if clients.is_empty() => {
            return Err(ctx.empty_field("skills.clients"));
        }
        RawClientSelection::List(clients) => {
            let clients = validate_non_empty_list_elements(ctx, "skills.clients", clients)?;
            ClientSelection::Explicit(validate_client_ids(ctx, clients)?)
        }
    };

    Ok(SkillsConfig { clients })
}

fn validate_client_ids(
    ctx: &ValidationContext<'_>,
    clients: Vec<String>,
) -> Result<Vec<String>> {
    let mut seen = BTreeSet::new();
    let mut normalized = Vec::with_capacity(clients.len());

    for client in clients {
        let Some(client_id) = normalize_client_id(&client) else {
            return Err(ConfigError::InvalidFieldValue {
                path: ctx.path_buf(),
                layer: ctx.layer,
                field: "skills.clients",
                value: client,
            }
            .into());
        };

        if !seen.insert(client_id) {
            return Err(ConfigError::InvalidFieldValue {
                path: ctx.path_buf(),
                layer: ctx.layer,
                field: "skills.clients",
                value: format!("duplicate client `{client_id}`"),
            }
            .into());
        }

        normalized.push(client_id.to_string());
    }

    Ok(normalized)
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
    fn parses_omitted_source_selection_as_empty_lists() {
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

        assert!(config.sources()[0].include().is_empty());
        assert!(config.sources()[0].groups().is_empty());
    }

    #[test]
    fn rejects_explicit_empty_source_include() {
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
    fn rejects_explicit_empty_source_groups() {
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
            Error::Config(ConfigError::InvalidAliasKey {
                alias_key,
                ..
            }) if alias_key == "legacy-review"
        ));
    }

    #[test]
    fn rejects_skill_alias_for_unknown_source() {
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
            Error::Config(ConfigError::UnknownAliasSource {
                alias_key,
                source_id,
                ..
            }) if alias_key == "community:legacy-review" && source_id == "community"
        ));
    }

    #[test]
    fn rejects_empty_skill_alias_target() {
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
        assert!(error.to_string().contains("git sources are not implemented yet"));
    }

    #[test]
    fn rejects_unknown_scope_value() {
        let error = parse_config_str(ConfigLayer::User, "user.toml", &config_contents("bogus"))
            .unwrap_err();

        assert!(matches!(
            error,
            Error::Config(ConfigError::InvalidFieldValue {
                field: "scope",
                value,
                ..
            }) if value == "bogus"
        ));
    }

    #[test]
    fn rejects_unknown_client_id() {
        let contents = r#"
scope = "user"

[skills]
clients = ["not-a-client"]
"#;

        let error = parse_config_str(ConfigLayer::User, "user.toml", contents).unwrap_err();

        assert!(matches!(
            error,
            Error::Config(ConfigError::InvalidFieldValue {
                field: "skills.clients",
                value,
                ..
            }) if value == "not-a-client"
        ));
    }

    #[test]
    fn rejects_invalid_clients_string() {
        let contents = r#"
scope = "user"

[skills]
clients = "bogus"
"#;

        let error = parse_config_str(ConfigLayer::User, "user.toml", contents).unwrap_err();

        assert!(matches!(
            error,
            Error::Config(ConfigError::InvalidFieldValue {
                field: "skills.clients",
                value,
                ..
            }) if value == "bogus"
        ));
    }

    #[test]
    fn rejects_empty_include_entry() {
        let contents = r#"
scope = "user"

[[skill_sources]]
id = "personal"
type = "path"
path = "../skills"
include = [""]

[skills]
clients = ["codex"]
"#;

        let error = parse_config_str(ConfigLayer::User, "user.toml", contents).unwrap_err();

        assert_empty_field(error, "skill_sources[].include");
    }

    #[test]
    fn accepts_alias_source_id_with_surrounding_whitespace() {
        let contents = r#"
scope = "user"

[[skill_sources]]
id = "personal"
type = "path"
path = "../skills"

[skill_aliases]
" personal:legacy-review" = "code-review"

[skills]
clients = ["codex"]
"#;

        parse_config_str(ConfigLayer::User, "user.toml", contents).unwrap();
    }

    #[test]
    fn rejects_unknown_top_level_field() {
        let contents = r#"
scope = "user"
unknown = true

[skills]
clients = ["codex"]
"#;

        let error = parse_config_str(ConfigLayer::User, "user.toml", contents).unwrap_err();

        assert!(matches!(
            error,
            Error::Config(ConfigError::Parse { .. })
        ));
    }

    #[test]
    fn rejects_duplicate_client_ids() {
        let contents = r#"
scope = "user"

[skills]
clients = ["codex", "codex"]
"#;

        let error = parse_config_str(ConfigLayer::User, "user.toml", contents).unwrap_err();

        assert!(matches!(
            error,
            Error::Config(ConfigError::InvalidFieldValue {
                field: "skills.clients",
                ..
            })
        ));
        assert!(error.to_string().contains("duplicate client"));
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
