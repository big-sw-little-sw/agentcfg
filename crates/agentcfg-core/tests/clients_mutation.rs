use agentcfg_core::{
    clients_add, clients_remove, clients_set, ClientId, ClientsMutationRequest, ConfigLayerId,
    WorkflowName, WorkflowStatus,
};

#[test]
fn set_defaults_project_mutation_to_user_project_config() {
    let project_root = test_project_root("set-defaults-to-user-project");
    std::fs::write(
        project_root.join("agentcfg.toml"),
        "default_clients = [\"codex\"]\n",
    )
    .expect("write shared config");

    let result = clients_set(ClientsMutationRequest::for_project_root(
        project_root.clone(),
        vec![ClientId::Cursor],
    ));

    assert_eq!(result.workflow, WorkflowName::ClientsSet);
    assert_eq!(result.status, WorkflowStatus::Success);
    assert_eq!(result.data.config_layer_id, ConfigLayerId::UserProject);
    assert_eq!(
        result.data.config_path,
        project_root.join(".agentcfg").join("agentcfg.toml")
    );
    assert_eq!(result.data.default_clients, vec![ClientId::Cursor]);
    assert_eq!(
        result.suggested_actions[0].command,
        "agentcfg install --level project"
    );

    assert_eq!(
        std::fs::read_to_string(project_root.join("agentcfg.toml")).expect("read shared config"),
        "default_clients = [\"codex\"]\n"
    );
    assert_eq!(
        std::fs::read_to_string(project_root.join(".agentcfg").join("agentcfg.toml"))
            .expect("read user project config"),
        "default_clients = [\"cursor\"]\n"
    );
    assert!(!project_root.join(".agentcfg").join("state").exists());
    assert!(!project_root.join("agentcfg.lock").exists());
    assert!(!project_root
        .join(".agentcfg")
        .join("agentcfg.lock")
        .exists());
}

#[test]
fn add_mutates_only_the_selected_shared_project_config() {
    let project_root = test_project_root("add-shared-project-client");
    std::fs::write(
        project_root.join("agentcfg.toml"),
        "default_clients = [\"codex\"]\n",
    )
    .expect("write shared config");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create user project dir");
    std::fs::write(
        project_root.join(".agentcfg").join("agentcfg.toml"),
        "default_clients = [\"cursor\"]\n",
    )
    .expect("write user project config");

    let result = clients_add(
        ClientsMutationRequest::for_project_root(project_root.clone(), vec![ClientId::ClaudeCode])
            .with_config_layer(ConfigLayerId::SharedProject),
    );

    assert_eq!(result.workflow, WorkflowName::ClientsAdd);
    assert_eq!(result.status, WorkflowStatus::Success);
    assert_eq!(result.data.config_layer_id, ConfigLayerId::SharedProject);
    assert_eq!(
        result.data.default_clients,
        vec![ClientId::Codex, ClientId::ClaudeCode]
    );
    assert_eq!(
        std::fs::read_to_string(project_root.join("agentcfg.toml")).expect("read shared config"),
        "default_clients = [\"codex\", \"claude-code\"]\n"
    );
    assert_eq!(
        std::fs::read_to_string(project_root.join(".agentcfg").join("agentcfg.toml"))
            .expect("read user project config"),
        "default_clients = [\"cursor\"]\n"
    );
}

#[test]
fn remove_mutates_only_the_selected_user_project_config() {
    let project_root = test_project_root("remove-user-project-client");
    std::fs::write(
        project_root.join("agentcfg.toml"),
        "default_clients = [\"codex\", \"cursor\"]\n",
    )
    .expect("write shared config");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create user project dir");
    std::fs::write(
        project_root.join(".agentcfg").join("agentcfg.toml"),
        "default_clients = [\"codex\", \"cursor\", \"cline\"]\n",
    )
    .expect("write user project config");

    let result = clients_remove(ClientsMutationRequest::for_project_root(
        project_root.clone(),
        vec![ClientId::Cursor],
    ));

    assert_eq!(result.workflow, WorkflowName::ClientsRemove);
    assert_eq!(result.status, WorkflowStatus::Success);
    assert_eq!(result.data.config_layer_id, ConfigLayerId::UserProject);
    assert_eq!(
        result.data.default_clients,
        vec![ClientId::Codex, ClientId::Cline]
    );
    assert_eq!(
        std::fs::read_to_string(project_root.join("agentcfg.toml")).expect("read shared config"),
        "default_clients = [\"codex\", \"cursor\"]\n"
    );
    assert_eq!(
        std::fs::read_to_string(project_root.join(".agentcfg").join("agentcfg.toml"))
            .expect("read user project config"),
        "default_clients = [\"codex\", \"cline\"]\n"
    );
}

#[test]
fn mutation_preserves_unrelated_config_fields() {
    let project_root = test_project_root("preserve-unrelated-fields");
    let user_project_config = project_root.join(".agentcfg").join("agentcfg.toml");
    std::fs::create_dir_all(user_project_config.parent().expect("config parent"))
        .expect("create user project dir");
    std::fs::write(
        &user_project_config,
        "schema_version = 1\n\n[metadata]\nowner = \"local\"\n",
    )
    .expect("write user project config");

    let result = clients_set(ClientsMutationRequest::for_project_root(
        project_root,
        vec![ClientId::Codex],
    ));

    assert_eq!(result.status, WorkflowStatus::Success);
    let written = std::fs::read_to_string(user_project_config).expect("read user project config");
    assert!(written.contains("schema_version = 1"));
    assert!(written.contains("[metadata]\nowner = \"local\""));
    assert!(written.contains("default_clients = [\"codex\"]"));
}

#[test]
fn mutation_blocks_invalid_default_client_selection_without_rewriting_file() {
    let project_root = test_project_root("invalid-default-clients-preserved");
    let user_project_config = project_root.join(".agentcfg").join("agentcfg.toml");
    std::fs::create_dir_all(user_project_config.parent().expect("config parent"))
        .expect("create user project dir");
    let original = "default_clients = \"codex\"\n\n[metadata]\nowner = \"local\"\n";
    std::fs::write(&user_project_config, original).expect("write user project config");

    let result = clients_add(ClientsMutationRequest::for_project_root(
        project_root,
        vec![ClientId::Cursor],
    ));

    assert_eq!(result.status, WorkflowStatus::Blocked);
    assert_eq!(result.blockers[0].code, "invalid-default-client-selection");
    assert!(result.suggested_actions.is_empty());
    assert_eq!(
        std::fs::read_to_string(user_project_config).expect("read user project config"),
        original
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
