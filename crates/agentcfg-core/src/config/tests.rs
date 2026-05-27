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
