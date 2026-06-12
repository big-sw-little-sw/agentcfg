use agentcfg_core::{
    deselect_skill, read_skill_configuration, select_skill, validate_entry_ids,
    write_skill_configuration, ConfigLayerId, DeselectSkillRequest, InstallLevel,
    SelectSkillRequest, SkillConfiguration, SkillConfigurationEntry, SkillSelection,
    WorkflowContext,
};

#[test]
fn select_skill_writes_explicit_included_skill_to_user_project_config() {
    let project_root = test_project("select-explicit-skill");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create project marker");
    std::fs::write(
        project_root.join(".agentcfg/agentcfg.toml"),
        "version = 1\nconfig-layer = \"user-project\"\nclients = [\"codex\"]\n",
    )
    .expect("write user project config");

    let result = select_skill(SelectSkillRequest {
        install_level: InstallLevel::Project,
        context: WorkflowContext::from_project_root(project_root.clone()),
        config_layer: None,
        source_skill_name: "find-bugs".to_string(),
        entry_id: None,
        source: Some("./skills".to_string()),
        git_ref: None,
    });

    assert!(result.blockers.is_empty(), "{:?}", result.blockers);
    assert!(result.data.changed);

    let content = std::fs::read_to_string(project_root.join(".agentcfg/agentcfg.toml"))
        .expect("read user project config");
    assert!(content.contains("source = \"./skills\""));
    assert!(content.contains("include = [\"find-bugs\"]"));
}

#[test]
fn select_skill_persists_entry_id_on_new_entry() {
    let project_root = test_project("select-entry-id");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create project marker");
    std::fs::write(
        project_root.join(".agentcfg/agentcfg.toml"),
        "version = 1\nconfig-layer = \"user-project\"\nclients = [\"codex\"]\n",
    )
    .expect("write user project config");

    let result = select_skill(SelectSkillRequest {
        install_level: InstallLevel::Project,
        context: WorkflowContext::from_project_root(project_root.clone()),
        config_layer: None,
        source_skill_name: "find-bugs".to_string(),
        entry_id: Some("local".to_string()),
        source: Some("./skills".to_string()),
        git_ref: None,
    });

    assert!(result.blockers.is_empty());
    assert_eq!(result.data.entry_id.as_deref(), Some("local"));

    let content = std::fs::read_to_string(project_root.join(".agentcfg/agentcfg.toml"))
        .expect("read user project config");
    assert!(content.contains("id = \"local\""));
}

#[test]
fn select_skill_appends_to_existing_entry_by_id() {
    let project_root = test_project("select-append-by-id");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create project marker");
    std::fs::write(
        project_root.join(".agentcfg/agentcfg.toml"),
        r#"
version = 1
config-layer = "user-project"
clients = ["codex"]

[[skills]]
id = "local"
source = "./skills"
include = ["find-bugs"]
"#,
    )
    .expect("write user project config");

    let result = select_skill(SelectSkillRequest {
        install_level: InstallLevel::Project,
        context: WorkflowContext::from_project_root(project_root.clone()),
        config_layer: None,
        source_skill_name: "code-review".to_string(),
        entry_id: Some("local".to_string()),
        source: None,
        git_ref: None,
    });

    assert!(result.blockers.is_empty());
    let content = std::fs::read_to_string(project_root.join(".agentcfg/agentcfg.toml"))
        .expect("read user project config");
    assert_eq!(content.matches("[[skills]]").count(), 1);
    assert!(content.contains("include = [\"find-bugs\", \"code-review\"]"));
}

#[test]
fn select_skill_rejects_disagreeing_entry_id_and_source() {
    let project_root = test_project("select-id-source-mismatch");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create project marker");
    std::fs::write(
        project_root.join(".agentcfg/agentcfg.toml"),
        r#"
version = 1
config-layer = "user-project"
clients = ["codex"]

[[skills]]
id = "local"
source = "./skills"
include = ["find-bugs"]
"#,
    )
    .expect("write user project config");

    let result = select_skill(SelectSkillRequest {
        install_level: InstallLevel::Project,
        context: WorkflowContext::from_project_root(project_root.clone()),
        config_layer: None,
        source_skill_name: "code-review".to_string(),
        entry_id: Some("local".to_string()),
        source: Some("./other".to_string()),
        git_ref: None,
    });

    assert_eq!(result.blockers.len(), 1);
    assert_eq!(result.blockers[0].code, "entry-selector-mismatch");
}

#[test]
fn read_skill_configuration_rejects_duplicate_entry_ids() {
    let path = test_path("duplicate-entry-ids");
    std::fs::write(
        &path,
        r#"
version = 1
config-layer = "user-project"

[[skills]]
id = "team"
source = "./skills"
include = ["find-bugs"]

[[skills]]
id = "team"
source = "./other"
include = ["code-review"]
"#,
    )
    .expect("write config");

    let error = read_skill_configuration(path.as_path()).unwrap_err();
    assert!(error
        .to_string()
        .contains("duplicate Skill Configuration Entry Id"));
}

#[test]
fn validate_entry_ids_accepts_unique_optional_ids() {
    let entries = vec![
        SkillConfigurationEntry {
            id: Some("a".to_string()),
            source: "./one".to_string(),
            git_ref: None,
            include: SkillSelection::Explicit(vec!["find-bugs".to_string()]),
            clients: None,
            exclude: Vec::new(),
            aliases: Default::default(),
        },
        SkillConfigurationEntry {
            id: None,
            source: "./two".to_string(),
            git_ref: None,
            include: SkillSelection::Explicit(vec!["code-review".to_string()]),
            clients: None,
            exclude: Vec::new(),
            aliases: Default::default(),
        },
    ];

    validate_entry_ids(&entries).expect("unique ids");
}

#[test]
fn select_skill_persists_github_shorthand_with_ref() {
    let project_root = test_project("select-github-ref");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create project marker");
    std::fs::write(
        project_root.join(".agentcfg/agentcfg.toml"),
        "version = 1\nconfig-layer = \"user-project\"\nclients = [\"codex\"]\n",
    )
    .expect("write user project config");

    let result = select_skill(SelectSkillRequest {
        install_level: InstallLevel::Project,
        context: WorkflowContext::from_project_root(project_root.clone()),
        config_layer: None,
        source_skill_name: "dotagents".to_string(),
        entry_id: None,
        source: Some("getsentry/dotagents".to_string()),
        git_ref: Some("v1.0.0".to_string()),
    });

    assert!(result.blockers.is_empty());
    let content = std::fs::read_to_string(project_root.join(".agentcfg/agentcfg.toml"))
        .expect("read user project config");
    assert!(content.contains("ref = \"v1.0.0\""));
}

#[test]
fn read_skill_configuration_deserializes_inline_dotagents_style_rows() {
    let path = test_path("read-inline-skills");
    std::fs::write(
        &path,
        r#"
version = 1
config-layer = "user-project"

[[skills]]
id = "local"
source = "./skills"
include = ["find-bugs"]
"#,
    )
    .expect("write config");

    let configuration = read_skill_configuration(path.as_path()).expect("read skills");
    assert_eq!(configuration.entries[0].id.as_deref(), Some("local"));
}

#[test]
fn select_skill_blocks_before_write_when_no_final_client_selection() {
    let project_root = test_project("select-no-clients");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create project marker");

    let result = select_skill(SelectSkillRequest {
        install_level: InstallLevel::Project,
        context: WorkflowContext::from_project_root(project_root.clone()),
        config_layer: None,
        source_skill_name: "find-bugs".to_string(),
        entry_id: None,
        source: Some("./skills".to_string()),
        git_ref: None,
    });

    assert_eq!(result.blockers.len(), 1);
    assert_eq!(result.blockers[0].code, "no-client-selection");
}

#[test]
fn select_skill_appends_to_compatible_existing_entry() {
    let project_root = test_project("select-append-compatible");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create project marker");
    std::fs::write(
        project_root.join(".agentcfg/agentcfg.toml"),
        r#"
version = 1
config-layer = "user-project"
clients = ["codex"]

[[skills]]
source = "./skills"
include = ["find-bugs"]
"#,
    )
    .expect("write user project config");

    select_skill(SelectSkillRequest {
        install_level: InstallLevel::Project,
        context: WorkflowContext::from_project_root(project_root.clone()),
        config_layer: None,
        source_skill_name: "code-review".to_string(),
        entry_id: None,
        source: Some("./skills".to_string()),
        git_ref: None,
    });

    let content = std::fs::read_to_string(project_root.join(".agentcfg/agentcfg.toml"))
        .expect("read user project config");
    assert!(content.contains("include = [\"find-bugs\", \"code-review\"]"));
}

#[test]
fn deselect_skill_targets_entry_by_id_without_source() {
    let project_root = test_project("deselect-by-id");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create project marker");
    std::fs::write(
        project_root.join(".agentcfg/agentcfg.toml"),
        r#"
version = 1
config-layer = "user-project"
clients = ["codex"]

[[skills]]
id = "local"
source = "./skills"
include = ["find-bugs"]
"#,
    )
    .expect("write user project config");

    let result = deselect_skill(DeselectSkillRequest {
        install_level: InstallLevel::Project,
        context: WorkflowContext::from_project_root(project_root.clone()),
        config_layer: None,
        source_skill_name: "find-bugs".to_string(),
        entry_id: Some("local".to_string()),
        source: None,
        git_ref: None,
    });

    assert!(result.blockers.is_empty());
    assert!(result.data.changed);
    let content = std::fs::read_to_string(project_root.join(".agentcfg/agentcfg.toml"))
        .expect("read user project config");
    assert!(!content.contains("find-bugs"));
}

#[test]
fn write_skill_configuration_preserves_unrelated_toml_comments() {
    let path = test_path("write-preserve-comments");
    std::fs::write(
        &path,
        r#"# project defaults
version = 1
config-layer = "shared-project"
clients = ["codex"]
# end header
"#,
    )
    .expect("write config");

    write_skill_configuration(
        path.as_path(),
        ConfigLayerId::SharedProject,
        &SkillConfiguration {
            entries: vec![SkillConfigurationEntry {
                id: None,
                source: "./skills".to_string(),
                git_ref: None,
                include: SkillSelection::Explicit(vec!["find-bugs".to_string()]),
                clients: None,
                exclude: Vec::new(),
                aliases: Default::default(),
            }],
        },
    )
    .expect("write skills");

    let content = std::fs::read_to_string(&path).expect("read config");
    assert!(content.contains("# project defaults"));
    assert!(content.contains("source = \"./skills\""));
}

fn test_project(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir()
        .join("agentcfg-tests")
        .join(format!("skills-{name}-{}", std::process::id()));
    if root.exists() {
        std::fs::remove_dir_all(&root).expect("remove previous root");
    }
    std::fs::create_dir_all(&root).expect("create root");
    root
}

fn test_path(name: &str) -> std::path::PathBuf {
    test_project(name).join("agentcfg.toml")
}
