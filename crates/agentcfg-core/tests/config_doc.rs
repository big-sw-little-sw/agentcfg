use agentcfg_core::{
    read_default_clients, write_default_clients, Client, ConfigLayerId, PersistedClientSelection,
};

#[test]
fn read_default_clients_returns_none_for_missing_file() {
    let path = test_path("missing-config");

    assert_eq!(
        read_default_clients(path.as_path()).expect("read missing file"),
        None
    );
}

#[test]
fn read_default_clients_parses_explicit_client_list() {
    let path = test_path("explicit-clients");
    std::fs::write(
        &path,
        r#"
version = 1
config-layer = "user-project"
clients = ["codex", "cursor"]
"#,
    )
    .expect("write config");

    assert_eq!(
        read_default_clients(path.as_path()).expect("read config"),
        Some(PersistedClientSelection::Explicit(vec![
            Client::Codex,
            Client::Cursor,
        ]))
    );
}

#[test]
fn read_default_clients_rejects_unknown_client_names() {
    let path = test_path("unknown-client");
    std::fs::write(
        &path,
        r#"
version = 1
config-layer = "user-project"
clients = ["not-a-client"]
"#,
    )
    .expect("write config");

    let error = read_default_clients(path.as_path()).unwrap_err();
    assert!(error.to_string().contains("not-a-client"));
}

#[test]
fn write_default_clients_creates_user_project_config_with_parent_directory() {
    let project_root = test_dir("write-user-project");
    let path = project_root.join(".agentcfg").join("agentcfg.toml");

    write_default_clients(
        path.as_path(),
        ConfigLayerId::UserProject,
        &PersistedClientSelection::Explicit(vec![Client::Cursor]),
    )
    .expect("write config");

    let content = std::fs::read_to_string(&path).expect("read config");
    assert!(content.contains("config-layer = \"user-project\""));
    assert!(content.contains("clients = [\"cursor\"]"));
}

#[test]
fn write_default_clients_preserves_toml_comments() {
    let path = test_path("preserve-comments");
    std::fs::write(
        &path,
        r#"# project defaults
version = 1
config-layer = "shared-project"
# end header
"#,
    )
    .expect("write config");

    write_default_clients(
        path.as_path(),
        ConfigLayerId::SharedProject,
        &PersistedClientSelection::Explicit(vec![Client::Codex]),
    )
    .expect("write config");

    let content = std::fs::read_to_string(&path).expect("read config");
    assert!(content.contains("# project defaults"));
    assert!(content.contains("# end header"));
    assert!(content.contains("clients = [\"codex\"]"));
}

#[test]
fn write_default_clients_preserves_unrelated_toml_fields() {
    let path = test_path("preserve-fields");
    std::fs::write(
        &path,
        r#"
version = 1
config-layer = "shared-project"
note = "keep-me"

[[skills.sources]]
id = "team"
path = "../skills"
"#,
    )
    .expect("write config");

    write_default_clients(
        path.as_path(),
        ConfigLayerId::SharedProject,
        &PersistedClientSelection::Explicit(vec![Client::Codex]),
    )
    .expect("write config");

    let content = std::fs::read_to_string(&path).expect("read config");
    assert!(content.contains("note = \"keep-me\""));
    assert!(content.contains("id = \"team\""));
    assert!(content.contains("clients = [\"codex\"]"));
}

fn test_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir()
        .join("agentcfg-tests")
        .join(format!("config-doc-{name}-{}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("remove previous dir");
    }
    std::fs::create_dir_all(&dir).expect("create dir");
    dir
}

fn test_path(name: &str) -> std::path::PathBuf {
    test_dir(name).join("agentcfg.toml")
}
