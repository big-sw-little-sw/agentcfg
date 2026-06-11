use std::path::PathBuf;

use agentcfg_core::{
    active_config_layers, resolve_project_root, user_config_path, ConfigLayerId, InstallLevel,
    ProjectAnchorSource, UserConfigPathError, WorkflowContext,
};

#[test]
fn resolve_project_root_uses_git_repository_root() {
    let root = test_dir("git-root");
    let nested = root.join("packages").join("app");
    std::fs::create_dir_all(&nested).expect("create nested dir");
    std::fs::create_dir_all(root.join(".git")).expect("create git dir");

    let discovered = resolve_project_root(&nested);
    assert_eq!(discovered.root, root);
    assert_eq!(discovered.anchor, Some(ProjectAnchorSource::GitRoot));
}

#[test]
fn resolve_project_root_is_unanchored_without_git_or_markers() {
    let root = test_dir("no-git-root");
    std::fs::create_dir_all(&root).expect("create dir");

    let discovered = resolve_project_root(&root);
    assert_eq!(discovered.root, root);
    assert_eq!(discovered.anchor, None);
}

#[test]
fn project_config_layer_paths_are_under_project_root() {
    let context = WorkflowContext::from_project_root(PathBuf::from("/tmp/example-project"));

    assert_eq!(
        context
            .config_layer_path(ConfigLayerId::SharedProject)
            .expect("shared project path"),
        PathBuf::from("/tmp/example-project/agentcfg.toml")
    );
    assert_eq!(
        context
            .config_layer_path(ConfigLayerId::UserProject)
            .expect("user project path"),
        PathBuf::from("/tmp/example-project/.agentcfg/agentcfg.toml")
    );
}

#[test]
fn user_config_path_uses_xdg_config_home_when_set() {
    let config_home = test_dir("xdg-config-home");
    std::env::set_var("XDG_CONFIG_HOME", &config_home);

    assert_eq!(
        user_config_path().expect("user config path"),
        config_home.join("agentcfg").join("agentcfg.toml")
    );

    std::env::remove_var("XDG_CONFIG_HOME");
}

#[test]
fn user_config_path_errors_when_home_env_vars_are_missing() {
    let saved_xdg = std::env::var("XDG_CONFIG_HOME").ok();
    let saved_home = std::env::var("HOME").ok();
    std::env::remove_var("XDG_CONFIG_HOME");
    std::env::remove_var("HOME");

    assert_eq!(
        user_config_path().unwrap_err(),
        UserConfigPathError::MissingHomeEnv
    );

    restore_env_var("XDG_CONFIG_HOME", saved_xdg);
    restore_env_var("HOME", saved_home);
}

#[test]
fn user_config_layer_path_ignores_project_root() {
    let config_home = test_dir("user-layer-context");
    std::env::set_var("XDG_CONFIG_HOME", &config_home);

    let context = WorkflowContext::from_project_root(PathBuf::from("/tmp/unrelated-project"));
    assert_eq!(
        context
            .config_layer_path(ConfigLayerId::User)
            .expect("user config path"),
        config_home.join("agentcfg").join("agentcfg.toml")
    );

    std::env::remove_var("XDG_CONFIG_HOME");
}

#[test]
fn active_project_level_layers_are_shared_then_user_project() {
    assert_eq!(
        active_config_layers(InstallLevel::Project),
        vec![ConfigLayerId::SharedProject, ConfigLayerId::UserProject]
    );
}

#[test]
fn active_user_level_layers_are_user_config_only() {
    assert_eq!(
        active_config_layers(InstallLevel::User),
        vec![ConfigLayerId::User]
    );
}

fn restore_env_var(name: &str, value: Option<String>) {
    match value {
        Some(value) => std::env::set_var(name, value),
        None => std::env::remove_var(name),
    }
}

fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("agentcfg-tests")
        .join(format!("locations-{name}-{}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("remove previous dir");
    }
    std::fs::create_dir_all(&dir).expect("create dir");
    dir
}
