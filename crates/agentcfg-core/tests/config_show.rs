use agentcfg_core::{
    config_show, ConfigLayerId, ConfigLayerState, ConfigShowRequest, InstallLevel, WorkflowContext,
    WorkflowStatus,
};

#[test]
fn config_show_reports_missing_project_config_files() {
    let project_root = test_project_root("missing-project-config-files");

    let result = config_show(ConfigShowRequest::project(
        WorkflowContext::from_project_root(project_root.clone()),
    ));

    assert_eq!(result.workflow, "config_show");
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
    assert_eq!(shared_project.state, ConfigLayerState::Missing);

    let user_project = &result.data.config_layers[1];
    assert_eq!(user_project.id, ConfigLayerId::UserProject);
    assert_eq!(user_project.name, "User Project Config");
    assert_eq!(
        user_project.path,
        project_root.join(".agentcfg").join("agentcfg.toml")
    );
    assert_eq!(user_project.state, ConfigLayerState::Missing);
}

#[test]
fn config_show_reports_existing_empty_project_config_files() {
    let project_root = test_project_root("existing-empty-project-config-files");
    std::fs::write(project_root.join("agentcfg.toml"), "").expect("write shared config");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create user project dir");
    std::fs::write(project_root.join(".agentcfg").join("agentcfg.toml"), "")
        .expect("write user project config");

    let result = config_show(ConfigShowRequest::project(
        WorkflowContext::from_project_root(project_root),
    ));

    assert_eq!(result.data.config_layers.len(), 2);
    assert_eq!(result.data.config_layers[0].state, ConfigLayerState::Empty);
    assert_eq!(result.data.config_layers[1].state, ConfigLayerState::Empty);
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
