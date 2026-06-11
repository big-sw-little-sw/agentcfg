use std::process::Command;

use serde_json::json;

#[test]
fn clients_show_reports_project_layers_as_text() {
    let project_root = test_project("cli-show-text");
    std::fs::write(
        project_root.join("agentcfg.toml"),
        "version = 1\nconfig-layer = \"shared-project\"\nclients = [\"cursor\"]\n",
    )
    .expect("write shared config");

    let output = agentcfg()
        .args(["clients", "show"])
        .current_dir(&project_root)
        .output()
        .expect("run agentcfg");

    assert!(
        output.status.success(),
        "expected success, stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is utf8"),
        "Default Client Selection\n\
         Install Level: project\n\
         Config Layers:\n\
         - Shared Project Config: cursor (agentcfg.toml)\n\
         - User Project Config: none (.agentcfg/agentcfg.toml)\n"
    );
}

#[test]
fn clients_set_defaults_to_user_project_config() {
    let project_root = test_project("cli-set-default-layer");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create project marker");

    let output = agentcfg()
        .args(["clients", "set", "codex"])
        .current_dir(&project_root)
        .output()
        .expect("run agentcfg");

    assert!(output.status.success());
    assert!(project_root.join(".agentcfg/agentcfg.toml").exists());
    assert!(!project_root.join("agentcfg.toml").exists());
}

#[test]
fn clients_set_writes_shared_project_config_with_flag() {
    let project_root = test_project("cli-set-shared");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create project marker");

    let output = agentcfg()
        .args(["clients", "set", "pi", "--config-layer", "shared-project"])
        .current_dir(&project_root)
        .output()
        .expect("run agentcfg");

    assert!(output.status.success());
    assert!(project_root.join("agentcfg.toml").exists());
}

#[test]
fn clients_show_supports_user_level_with_flag() {
    let config_home = test_dir("cli-user-level");
    std::fs::create_dir_all(config_home.join("agentcfg")).expect("create config dir");
    std::fs::write(
        config_home.join("agentcfg/agentcfg.toml"),
        "version = 1\nconfig-layer = \"user\"\nclients = [\"cline\"]\n",
    )
    .expect("write user config");

    let output = agentcfg()
        .args(["clients", "show", "--level", "user", "--format", "json"])
        .env("XDG_CONFIG_HOME", &config_home)
        .current_dir(test_project("cli-user-level-cwd"))
        .output()
        .expect("run agentcfg");

    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is json");
    assert_eq!(value["data"]["install_level"], "user");
    assert_eq!(value["data"]["config_layers"].as_array().unwrap().len(), 1);
    assert_eq!(
        value["data"]["config_layers"][0]["default_clients"],
        json!(["cline"])
    );
}

#[test]
fn clients_set_rejects_unknown_client_with_nonzero_exit() {
    let project_root = test_project("cli-unknown-client");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create project marker");

    let output = agentcfg()
        .args(["clients", "set", "not-a-client"])
        .current_dir(&project_root)
        .output()
        .expect("run agentcfg");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is utf8"),
        ""
    );
}

#[test]
fn clients_add_and_remove_emit_json_results() {
    let project_root = test_project("cli-add-remove-json");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create user project dir");
    std::fs::write(
        project_root.join(".agentcfg/agentcfg.toml"),
        "version = 1\nconfig-layer = \"user-project\"\nclients = [\"codex\"]\n",
    )
    .expect("write user project config");

    let add = agentcfg()
        .args(["clients", "add", "cursor", "--format", "json"])
        .current_dir(&project_root)
        .output()
        .expect("run add");

    assert!(add.status.success());
    let add_value: serde_json::Value =
        serde_json::from_slice(&add.stdout).expect("add stdout is json");
    assert_eq!(add_value["workflow"], "clients_add");

    let remove = agentcfg()
        .args(["clients", "remove", "codex", "--format", "json"])
        .current_dir(&project_root)
        .output()
        .expect("run remove");

    assert!(remove.status.success());
    let remove_value: serde_json::Value =
        serde_json::from_slice(&remove.stdout).expect("remove stdout is json");
    assert_eq!(remove_value["workflow"], "clients_remove");
}

fn agentcfg() -> Command {
    Command::new(env!("CARGO_BIN_EXE_agentcfg"))
}

fn test_project(name: &str) -> std::path::PathBuf {
    test_dir(name)
}

fn test_dir(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir()
        .join("agentcfg-cli-tests")
        .join(format!("{name}-{}", std::process::id()));
    if root.exists() {
        std::fs::remove_dir_all(&root).expect("remove previous root");
    }
    std::fs::create_dir_all(&root).expect("create root");
    root
}
