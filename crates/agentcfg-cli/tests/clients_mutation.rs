use std::process::Command;

use serde_json::json;

#[test]
fn set_writes_user_project_config_and_reports_json() {
    let project_root = test_project_root("clients-set-json");

    let output = Command::new(env!("CARGO_BIN_EXE_agentcfg"))
        .args(["clients", "set", "codex", "cursor", "--format", "json"])
        .current_dir(&project_root)
        .output()
        .expect("run agentcfg");

    assert!(
        output.status.success(),
        "expected success, stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let canonical_project_root =
        std::fs::canonicalize(&project_root).expect("canonicalize project root");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).expect("stdout is json"),
        json!({
            "workflow": "clients_set",
            "status": "success",
            "diagnostics": [],
            "blockers": [],
            "suggested_actions": [{
                "command": "agentcfg install --level project",
                "reason": "Materialize changed Default Client Selection."
            }],
            "progress_events": [],
            "data": {
                "install_level": "project",
                "config_layer_id": "user-project",
                "config_path": canonical_project_root.join(".agentcfg").join("agentcfg.toml"),
                "default_clients": ["codex", "cursor"]
            }
        })
    );
    assert_eq!(
        std::fs::read_to_string(project_root.join(".agentcfg").join("agentcfg.toml"))
            .expect("read user project config"),
        "default_clients = [\"codex\", \"cursor\"]\n"
    );
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr is utf8"),
        ""
    );
}

#[test]
fn add_can_target_shared_project_config() {
    let project_root = test_project_root("clients-add-shared");
    std::fs::write(
        project_root.join("agentcfg.toml"),
        "default_clients = [\"codex\"]\n",
    )
    .expect("write shared config");

    let output = Command::new(env!("CARGO_BIN_EXE_agentcfg"))
        .args([
            "clients",
            "add",
            "claude-code",
            "--config-layer",
            "shared-project",
        ])
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
        "Default Client Selection updated\n\
         Config Layer: Shared Project Config\n\
         Default Clients: codex, claude-code\n\
         Next: agentcfg install --level project (Materialize changed Default Client Selection.)\n"
    );
    assert_eq!(
        std::fs::read_to_string(project_root.join("agentcfg.toml")).expect("read shared config"),
        "default_clients = [\"codex\", \"claude-code\"]\n"
    );
    assert!(!project_root
        .join(".agentcfg")
        .join("agentcfg.toml")
        .exists());
}

#[test]
fn remove_can_target_user_config_with_level_user() {
    let project_root = test_project_root("clients-remove-user");
    let config_home = project_root.join("xdg-config-home");
    let user_config_dir = config_home.join("agentcfg");
    std::fs::create_dir_all(&user_config_dir).expect("create user config dir");
    std::fs::write(
        user_config_dir.join("agentcfg.toml"),
        "default_clients = [\"codex\", \"cursor\"]\n",
    )
    .expect("write user config");

    let output = Command::new(env!("CARGO_BIN_EXE_agentcfg"))
        .args(["clients", "remove", "cursor", "--level", "user"])
        .current_dir(&project_root)
        .env("XDG_CONFIG_HOME", &config_home)
        .output()
        .expect("run agentcfg");

    assert!(
        output.status.success(),
        "expected success, stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is utf8"),
        "Default Client Selection updated\n\
         Config Layer: User Config\n\
         Default Clients: codex\n\
         Next: agentcfg install --level user (Materialize changed Default Client Selection.)\n"
    );
    assert_eq!(
        std::fs::read_to_string(user_config_dir.join("agentcfg.toml")).expect("read user config"),
        "default_clients = [\"codex\"]\n"
    );
    assert!(!project_root
        .join(".agentcfg")
        .join("agentcfg.toml")
        .exists());
}

#[test]
fn rejects_user_config_layer_without_level_user() {
    let output = Command::new(env!("CARGO_BIN_EXE_agentcfg"))
        .args(["clients", "set", "codex", "--config-layer", "user"])
        .current_dir(test_project_root("clients-user-layer-without-level"))
        .output()
        .expect("run agentcfg");

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is utf8"),
        ""
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr is utf8");
    assert!(stderr.contains("--config-layer user can only be used with --level user"));
}

#[test]
fn rejects_unknown_client_names() {
    let output = Command::new(env!("CARGO_BIN_EXE_agentcfg"))
        .args(["clients", "set", "vscode"])
        .current_dir(test_project_root("clients-unknown-client"))
        .output()
        .expect("run agentcfg");

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is utf8"),
        ""
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr is utf8");
    assert!(stderr.contains("unknown Client `vscode`"));
}

fn test_project_root(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir()
        .join("agentcfg-cli-tests")
        .join(format!("{}-{}", name, std::process::id()));
    if root.exists() {
        std::fs::remove_dir_all(&root).expect("remove previous test root");
    }
    std::fs::create_dir_all(&root).expect("create test root");
    root
}
