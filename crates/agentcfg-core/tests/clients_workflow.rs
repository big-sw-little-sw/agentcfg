use agentcfg_core::{
    clients_add, clients_remove, clients_set, clients_show, parse_client_name, Client,
    ClientsAddRequest, ClientsRemoveRequest, ClientsSetRequest, ClientsShowRequest, ConfigLayerId,
    InstallLevel, PersistedClientSelection,
};

#[test]
fn parse_client_name_accepts_known_v1_clients() {
    assert_eq!(
        parse_client_name("claude-code").unwrap(),
        Client::ClaudeCode
    );
    assert_eq!(parse_client_name("open-code").unwrap(), Client::OpenCode);
}

#[test]
fn parse_client_name_rejects_unknown_clients() {
    assert!(parse_client_name("vscode").is_err());
}

#[test]
fn clients_show_reports_project_layers_in_order() {
    let project_root = test_project("show-project-layers");
    std::fs::write(
        project_root.join("agentcfg.toml"),
        "version = 1\nconfig-layer = \"shared-project\"\nclients = [\"cursor\"]\n",
    )
    .expect("write shared config");

    let result = clients_show(ClientsShowRequest {
        install_level: InstallLevel::Project,
        project_root: project_root.clone(),
        config_layer: None,
    });

    assert!(result.blockers.is_empty());
    assert_eq!(result.data.config_layers.len(), 2);
    assert_eq!(
        result.data.config_layers[0].id,
        ConfigLayerId::SharedProject
    );
    assert_eq!(
        result.data.config_layers[0].default_clients,
        Some(PersistedClientSelection::Explicit(vec![Client::Cursor]))
    );
    assert_eq!(result.data.config_layers[1].id, ConfigLayerId::UserProject);
    assert_eq!(result.data.config_layers[1].default_clients, None);
}

#[test]
fn clients_set_defaults_to_user_project_config_at_project_level() {
    let project_root = test_project("set-default-layer");

    let result = clients_set(ClientsSetRequest {
        install_level: InstallLevel::Project,
        project_root: project_root.clone(),
        config_layer: None,
        clients: vec![Client::Codex],
    });

    assert!(result.blockers.is_empty());
    assert_eq!(result.data.config_layer.id, ConfigLayerId::UserProject);
    assert!(project_root
        .join(".agentcfg")
        .join("agentcfg.toml")
        .exists());
    assert!(!project_root.join("agentcfg.toml").exists());
}

#[test]
fn clients_set_writes_shared_project_config_when_requested() {
    let project_root = test_project("set-shared-layer");

    let result = clients_set(ClientsSetRequest {
        install_level: InstallLevel::Project,
        project_root: project_root.clone(),
        config_layer: Some(ConfigLayerId::SharedProject),
        clients: vec![Client::Pi],
    });

    assert!(result.blockers.is_empty());
    assert_eq!(result.data.config_layer.id, ConfigLayerId::SharedProject);
    assert!(project_root.join("agentcfg.toml").exists());
}

#[test]
fn clients_add_and_remove_mutate_only_selected_layer() {
    let project_root = test_project("add-remove-layer");
    std::fs::write(
        project_root.join("agentcfg.toml"),
        "version = 1\nconfig-layer = \"shared-project\"\nclients = [\"cursor\"]\n",
    )
    .expect("write shared config");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create user project dir");
    std::fs::write(
        project_root.join(".agentcfg").join("agentcfg.toml"),
        "version = 1\nconfig-layer = \"user-project\"\nclients = [\"codex\"]\n",
    )
    .expect("write user project config");

    clients_add(ClientsAddRequest {
        install_level: InstallLevel::Project,
        project_root: project_root.clone(),
        config_layer: Some(ConfigLayerId::UserProject),
        clients: vec![Client::ClaudeCode],
    });

    let shared = std::fs::read_to_string(project_root.join("agentcfg.toml")).expect("read shared");
    let user = std::fs::read_to_string(project_root.join(".agentcfg/agentcfg.toml"))
        .expect("read user project");
    assert!(shared.contains("[\"cursor\"]"));
    assert!(user.contains("codex") && user.contains("claude-code"));

    clients_remove(ClientsRemoveRequest {
        install_level: InstallLevel::Project,
        project_root: project_root.clone(),
        config_layer: Some(ConfigLayerId::SharedProject),
        clients: vec![Client::Cursor],
    });

    let shared = std::fs::read_to_string(project_root.join("agentcfg.toml")).expect("read shared");
    assert!(!shared.contains("cursor"));
}

#[test]
fn clients_mutations_do_not_write_managed_artifacts() {
    let project_root = test_project("no-managed-artifacts");
    let managed_state = project_root.join(".agentcfg").join("state");

    let result = clients_set(ClientsSetRequest {
        install_level: InstallLevel::Project,
        project_root,
        config_layer: None,
        clients: vec![Client::Cursor],
    });

    assert!(result.blockers.is_empty());

    assert!(!managed_state.exists());
}

fn test_project(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir()
        .join("agentcfg-tests")
        .join(format!("clients-{name}-{}", std::process::id()));
    if root.exists() {
        std::fs::remove_dir_all(&root).expect("remove previous root");
    }
    std::fs::create_dir_all(&root).expect("create root");
    root
}
