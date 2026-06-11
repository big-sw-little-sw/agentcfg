use agentcfg_core::{
    clients_show, ClientId, ClientsShowRequest, ConfigLayerId, ConfigLayerState, InstallLevel,
    WorkflowName, WorkflowStatus,
};

#[test]
fn reports_default_client_selection_by_project_config_layer() {
    let project_root = test_project_root("show-project-default-clients");
    std::fs::write(
        project_root.join("agentcfg.toml"),
        "default_clients = [\"codex\"]\n",
    )
    .expect("write shared config");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create user project dir");
    std::fs::write(
        project_root.join(".agentcfg").join("agentcfg.toml"),
        "default_clients = [\"cursor\", \"claude-code\"]\n",
    )
    .expect("write user project config");

    let result = clients_show(ClientsShowRequest::for_project_root(project_root.clone()));

    assert_eq!(result.workflow, WorkflowName::ClientsShow);
    assert_eq!(result.status, WorkflowStatus::Success);
    assert!(result.diagnostics.is_empty());
    assert!(result.blockers.is_empty());
    assert!(result.suggested_actions.is_empty());
    assert!(result.progress_events.is_empty());

    assert_eq!(result.data.install_level, InstallLevel::Project);
    assert_eq!(result.data.config_layers.len(), 2);

    let shared_project = &result.data.config_layers[0];
    assert_eq!(shared_project.id, ConfigLayerId::SharedProject);
    assert_eq!(shared_project.name, "Shared Project Config");
    assert_eq!(shared_project.path, project_root.join("agentcfg.toml"));
    assert_eq!(shared_project.state, ConfigLayerState::Authored);
    assert_eq!(shared_project.default_clients, vec![ClientId::Codex]);

    let user_project = &result.data.config_layers[1];
    assert_eq!(user_project.id, ConfigLayerId::UserProject);
    assert_eq!(user_project.name, "User Project Config");
    assert_eq!(
        user_project.path,
        project_root.join(".agentcfg").join("agentcfg.toml")
    );
    assert_eq!(user_project.state, ConfigLayerState::Authored);
    assert_eq!(
        user_project.default_clients,
        vec![ClientId::Cursor, ClientId::ClaudeCode]
    );
}

#[test]
fn resolves_project_root_from_nested_git_working_tree() {
    let project_root = test_project_root("nested-git-working-tree");
    std::fs::create_dir(project_root.join(".git")).expect("create git marker");
    std::fs::create_dir_all(project_root.join("crates").join("nested")).expect("create nested dir");
    std::fs::write(
        project_root.join("agentcfg.toml"),
        "default_clients = [\"codex\"]\n",
    )
    .expect("write shared config");

    let result = clients_show(ClientsShowRequest::for_project_cwd(
        project_root.join("crates").join("nested"),
    ));

    assert_eq!(
        result.data.config_layers[0].path,
        project_root.join("agentcfg.toml")
    );
    assert_eq!(
        result.data.config_layers[0].default_clients,
        vec![ClientId::Codex]
    );
    assert_eq!(
        result.data.config_layers[1].path,
        project_root.join(".agentcfg").join("agentcfg.toml")
    );
}

#[test]
fn reads_user_config_only_at_user_level() {
    let project_root = test_project_root("user-level-ignores-project");
    std::fs::write(
        project_root.join("agentcfg.toml"),
        "default_clients = [\"codex\"]\n",
    )
    .expect("write shared config");
    let config_home = project_root.join("xdg-config-home");
    let user_config_dir = config_home.join("agentcfg");
    std::fs::create_dir_all(&user_config_dir).expect("create user config dir");
    std::fs::write(
        user_config_dir.join("agentcfg.toml"),
        "default_clients = [\"opencode\", \"pi\"]\n",
    )
    .expect("write user config");

    let result = clients_show(ClientsShowRequest::for_user_config_home(
        config_home.clone(),
    ));

    assert_eq!(result.data.install_level, InstallLevel::User);
    assert_eq!(result.data.config_layers.len(), 1);
    assert_eq!(result.data.config_layers[0].id, ConfigLayerId::User);
    assert_eq!(result.data.config_layers[0].name, "User Config");
    assert_eq!(
        result.data.config_layers[0].path,
        config_home.join("agentcfg").join("agentcfg.toml")
    );
    assert_eq!(
        result.data.config_layers[0].default_clients,
        vec![ClientId::Opencode, ClientId::Pi]
    );
}

#[test]
fn blocks_unknown_default_client_names() {
    let project_root = test_project_root("unknown-default-client");
    std::fs::write(
        project_root.join("agentcfg.toml"),
        "default_clients = [\"codex\", \"vscode\"]\n",
    )
    .expect("write shared config");

    let result = clients_show(ClientsShowRequest::for_project_root(project_root.clone()));

    assert_eq!(result.status, WorkflowStatus::Blocked);
    assert_eq!(result.blockers.len(), 1);
    assert_eq!(result.blockers[0].code, "unknown-client");
    assert_eq!(
        result.blockers[0].message,
        "Unknown Client in Default Client Selection."
    );
    assert_eq!(
        result.blockers[0].context,
        vec![
            ("client".to_string(), "vscode".to_string()),
            (
                "path".to_string(),
                project_root
                    .join("agentcfg.toml")
                    .to_string_lossy()
                    .into_owned()
            )
        ]
    );
}

fn test_project_root(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join("agentcfg-tests").join(format!(
        "{}-{}",
        name,
        std::process::id()
    ));
    if root.exists() {
        std::fs::remove_dir_all(&root).expect("remove previous test root");
    }
    std::fs::create_dir_all(&root).expect("create test root");
    root
}
