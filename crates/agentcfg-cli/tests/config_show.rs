use std::process::Command;

use serde_json::json;

#[test]
fn config_show_reports_missing_project_config_files_as_text() {
    let project_root = test_project_root("text-output");
    assert!(!project_root.join("agentcfg.toml").exists());
    assert!(!project_root
        .join(".agentcfg")
        .join("agentcfg.toml")
        .exists());

    let output = Command::new(env!("CARGO_BIN_EXE_agentcfg"))
        .args(["config", "show"])
        .current_dir(project_root)
        .output()
        .expect("run agentcfg");

    assert!(
        output.status.success(),
        "expected success, stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is utf8"),
        "Agent Configuration\n\
         Install Level: project\n\
         Config Layers:\n\
         - Shared Project Config: missing (agentcfg.toml)\n\
         - User Project Config: missing (.agentcfg/agentcfg.toml)\n"
    );
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr is utf8"),
        ""
    );
}

#[test]
fn config_show_reports_missing_project_config_files_as_json() {
    let project_root = test_project_root("json-output");
    assert!(!project_root.join("agentcfg.toml").exists());
    assert!(!project_root
        .join(".agentcfg")
        .join("agentcfg.toml")
        .exists());

    let output = Command::new(env!("CARGO_BIN_EXE_agentcfg"))
        .args(["config", "show", "--format", "json"])
        .current_dir(&project_root)
        .output()
        .expect("run agentcfg");

    assert!(
        output.status.success(),
        "expected success, stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let project_root = std::fs::canonicalize(project_root).expect("canonicalize test root");
    let shared_path = project_root.join("agentcfg.toml");
    let user_path = project_root.join(".agentcfg").join("agentcfg.toml");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).expect("stdout is json"),
        json!({
            "workflow": "config_show",
            "status": "success",
            "diagnostics": [],
            "blockers": [],
            "suggested_actions": [],
            "progress_events": [],
            "data": {
                "install_level": "project",
                "config_layers": [
                    {
                        "id": "shared-project",
                        "name": "Shared Project Config",
                        "path": shared_path,
                        "state": "missing"
                    },
                    {
                        "id": "user-project",
                        "name": "User Project Config",
                        "path": user_path,
                        "state": "missing"
                    }
                ]
            }
        })
    );
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr is utf8"),
        ""
    );
}

#[test]
fn config_show_rejects_invalid_format_without_stdout() {
    let output = Command::new(env!("CARGO_BIN_EXE_agentcfg"))
        .args(["config", "show", "--format", "yaml"])
        .current_dir(test_project_root("invalid-format"))
        .output()
        .expect("run agentcfg");

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is utf8"),
        ""
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr is utf8");
    assert!(stderr.contains("invalid value"));
    assert!(stderr.contains("yaml"));
    assert!(stderr.contains("--format"));
    assert!(stderr.contains("text"));
    assert!(stderr.contains("json"));
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
