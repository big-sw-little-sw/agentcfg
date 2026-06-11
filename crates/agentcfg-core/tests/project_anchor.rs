use std::path::PathBuf;

use agentcfg_core::{
    build_workflow_context, clients_set, config_show, init, ClientsSetRequest, ConfigShowRequest,
    InitRequest, InstallLevel,
};

#[test]
fn project_level_mutation_blocks_before_write_when_unanchored() {
    let root = test_dir("mutation-blocker");
    let nested = root.join("scratch").join("work");
    std::fs::create_dir_all(&nested).expect("create nested dir");

    let context = build_workflow_context(nested.clone(), None).expect("build context");
    assert!(!context.is_anchored());

    let result = clients_set(ClientsSetRequest {
        install_level: InstallLevel::Project,
        context,
        config_layer: None,
        clients: vec![agentcfg_core::Client::Cursor],
    });

    assert_eq!(result.blockers.len(), 1);
    assert_eq!(result.blockers[0].code, "project-unanchored");
    assert_eq!(result.blockers[0].suggested_actions.len(), 2);
    assert!(!nested.join(".agentcfg").join("agentcfg.toml").exists());
}

#[test]
fn config_show_reports_unanchored_diagnostic_without_creating_markers() {
    let root = test_dir("config-show-unanchored");
    let nested = root.join("scratch").join("work");
    std::fs::create_dir_all(&nested).expect("create nested dir");

    let context = build_workflow_context(nested.clone(), None).expect("build context");
    let result = config_show(ConfigShowRequest::project(context));

    assert!(result.blockers.is_empty());
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].code, "project-unanchored");
    assert!(!nested.join(".agentcfg").exists());
    assert!(!nested.join("agentcfg.toml").exists());
}

#[test]
fn init_creates_markers_and_enables_project_level_mutation_in_non_git_fixture() {
    let root = test_dir("init-enables-mutation");
    let nested = root.join("project").join("app");
    std::fs::create_dir_all(&nested).expect("create nested dir");

    let init_result = init(InitRequest {
        cwd: nested.clone(),
        explicit_project_root: None,
    });
    assert!(init_result.blockers.is_empty());
    assert!(nested.join(".agentcfg").is_dir());
    assert!(!nested.join("agentcfg.lock").exists());
    assert!(!nested.join(".agentcfg").join("agentcfg.lock").exists());
    assert!(!nested.join(".agentcfg").join("state").exists());

    let context = build_workflow_context(nested.clone(), None).expect("build context");
    assert!(context.is_anchored());

    let mutation = clients_set(ClientsSetRequest {
        install_level: InstallLevel::Project,
        context,
        config_layer: None,
        clients: vec![agentcfg_core::Client::Cursor],
    });
    assert!(mutation.blockers.is_empty());
    assert!(nested.join(".agentcfg").join("agentcfg.toml").exists());
}

#[test]
fn init_is_idempotent_when_project_markers_already_exist() {
    let root = test_dir("init-idempotent");
    std::fs::create_dir_all(root.join(".agentcfg")).expect("create marker dir");

    let first = init(InitRequest {
        cwd: root.clone(),
        explicit_project_root: None,
    });
    let second = init(InitRequest {
        cwd: root.clone(),
        explicit_project_root: None,
    });

    assert!(first.blockers.is_empty());
    assert!(second.blockers.is_empty());
    assert!(!second.data.created_markers);
}

#[test]
fn explicit_project_root_override_allows_mutation_without_markers_in_cwd() {
    let root = test_dir("explicit-root-mutation");
    let marked = root.join("marked");
    let unmarked = root.join("unmarked").join("work");
    std::fs::create_dir_all(marked.join(".agentcfg")).expect("create marker dir");
    std::fs::create_dir_all(&unmarked).expect("create unmarked dir");

    let context =
        build_workflow_context(unmarked.clone(), Some(marked.clone())).expect("build context");
    let result = clients_set(ClientsSetRequest {
        install_level: InstallLevel::Project,
        context,
        config_layer: None,
        clients: vec![agentcfg_core::Client::Cursor],
    });

    assert!(result.blockers.is_empty());
    assert!(marked.join(".agentcfg").join("agentcfg.toml").exists());
    assert!(!unmarked.join(".agentcfg").exists());
}

fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("agentcfg-tests")
        .join(format!("project-anchor-{name}-{}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("remove previous dir");
    }
    std::fs::create_dir_all(&dir).expect("create dir");
    dir
}
