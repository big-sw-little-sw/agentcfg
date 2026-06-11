use std::process::Command;

#[test]
fn init_creates_markers_and_enables_clients_set_in_non_git_fixture() {
    let project_root = test_project("init-then-mutate");
    let nested = project_root.join("app");
    std::fs::create_dir_all(&nested).expect("create nested dir");

    let init = agentcfg()
        .args(["init"])
        .current_dir(&nested)
        .output()
        .expect("run init");

    assert!(
        init.status.success(),
        "expected init success, stderr:\n{}",
        String::from_utf8_lossy(&init.stderr)
    );
    assert!(nested.join(".agentcfg").is_dir());
    assert!(!nested.join("agentcfg.lock").exists());

    let set = agentcfg()
        .args(["clients", "set", "cursor"])
        .current_dir(&nested)
        .output()
        .expect("run clients set");

    assert!(
        set.status.success(),
        "expected mutation success, stderr:\n{}",
        String::from_utf8_lossy(&set.stderr)
    );
    assert!(nested.join(".agentcfg/agentcfg.toml").exists());
}

#[test]
fn project_level_mutation_blocks_in_unanchored_directory() {
    let project_root = test_project("unanchored-mutation");
    let nested = project_root.join("scratch").join("work");
    std::fs::create_dir_all(&nested).expect("create nested dir");

    let output = agentcfg()
        .args(["clients", "set", "cursor"])
        .current_dir(&nested)
        .output()
        .expect("run clients set");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not anchored to a Project"));
    assert!(stderr.contains("agentcfg init"));
    assert!(stderr.contains("--project-root"));
    assert!(!nested.join(".agentcfg").exists());
}

#[test]
fn project_level_mutation_blocker_emits_json_diagnostics() {
    let project_root = test_project("unanchored-json");
    let nested = project_root.join("scratch").join("work");
    std::fs::create_dir_all(&nested).expect("create nested dir");

    let output = agentcfg()
        .args(["clients", "set", "cursor", "--format", "json"])
        .current_dir(&nested)
        .output()
        .expect("run clients set");

    assert!(!output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is json");
    assert_eq!(value["blockers"][0]["code"], "project-unanchored");
    assert_eq!(
        value["blockers"][0]["suggested_actions"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn explicit_project_root_override_allows_mutation_from_unmarked_directory() {
    let root = test_project("explicit-root");
    let marked = root.join("marked");
    let unmarked = root.join("unmarked").join("work");
    std::fs::create_dir_all(marked.join(".agentcfg")).expect("create marker dir");
    std::fs::create_dir_all(&unmarked).expect("create unmarked dir");

    let output = agentcfg()
        .args([
            "clients",
            "set",
            "cursor",
            "--project-root",
            marked.to_str().expect("utf8 path"),
        ])
        .current_dir(&unmarked)
        .output()
        .expect("run clients set");

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(marked.join(".agentcfg/agentcfg.toml").exists());
    assert!(!unmarked.join(".agentcfg").exists());
}

#[test]
fn config_show_reports_unanchored_note_in_text_output() {
    let project_root = test_project("config-show-unanchored");
    let nested = project_root.join("scratch").join("work");
    std::fs::create_dir_all(&nested).expect("create nested dir");

    let output = agentcfg()
        .args(["config", "show"])
        .current_dir(&nested)
        .output()
        .expect("run config show");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("note: Working directory is not anchored"));
    assert!(stdout.contains("hint: agentcfg init"));
    assert!(!nested.join(".agentcfg").exists());
}

#[test]
fn init_emits_json_result() {
    let project_root = test_project("init-json");
    let output = agentcfg()
        .args(["init", "--format", "json"])
        .current_dir(&project_root)
        .output()
        .expect("run init");

    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is json");
    assert_eq!(value["workflow"], "init");
    assert_eq!(value["data"]["created_markers"], true);
}

fn agentcfg() -> Command {
    Command::new(env!("CARGO_BIN_EXE_agentcfg"))
}

fn test_project(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir()
        .join("agentcfg-cli-tests")
        .join(format!("project-root-{name}-{}", std::process::id()));
    if root.exists() {
        std::fs::remove_dir_all(&root).expect("remove previous root");
    }
    std::fs::create_dir_all(&root).expect("create root");
    root
}
