use std::path::PathBuf;

use agentcfg_core::{
    active_config_layers, config_layer_path, resolve_project_root, user_config_path, ConfigLayerId,
    InstallLevel,
};

#[test]
fn resolve_project_root_uses_git_repository_root() {
    let root = test_dir("git-root");
    let nested = root.join("packages").join("app");
    std::fs::create_dir_all(&nested).expect("create nested dir");
    std::fs::create_dir_all(root.join(".git")).expect("create git dir");

    assert_eq!(resolve_project_root(&nested), root);
}

#[test]
fn resolve_project_root_falls_back_to_start_when_no_git_repository() {
    let root = test_dir("no-git-root");
    std::fs::create_dir_all(&root).expect("create dir");

    assert_eq!(resolve_project_root(&root), root);
}

#[test]
fn project_config_layer_paths_are_under_project_root() {
    let project_root = PathBuf::from("/tmp/example-project");

    assert_eq!(
        config_layer_path(&project_root, ConfigLayerId::SharedProject),
        project_root.join("agentcfg.toml")
    );
    assert_eq!(
        config_layer_path(&project_root, ConfigLayerId::UserProject),
        project_root.join(".agentcfg").join("agentcfg.toml")
    );
}

#[test]
fn user_config_path_uses_xdg_config_home_when_set() {
    let config_home = test_dir("xdg-config-home");
    std::env::set_var("XDG_CONFIG_HOME", &config_home);

    assert_eq!(
        user_config_path(),
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
