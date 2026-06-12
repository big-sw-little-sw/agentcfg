use std::process::Command;

use serde_json::json;

#[test]
fn skills_deselect_targets_entry_by_id_without_source() {
    let project_root = test_project("cli-deselect-by-id");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create project marker");
    std::fs::write(
        project_root.join(".agentcfg/agentcfg.toml"),
        r#"
version = 1
config-layer = "user-project"
clients = ["codex"]

[[skills]]
id = "local"
source = "./skills"
include = ["find-bugs"]
"#,
    )
    .expect("write user project config");

    let output = agentcfg()
        .args(["skills", "deselect", "find-bugs", "--id", "local"])
        .current_dir(&project_root)
        .output()
        .expect("run agentcfg");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");
    assert!(stdout.contains("Entry Id: local"));
}

#[test]
fn skills_select_persists_entry_id() {
    let project_root = test_project("cli-select-entry-id");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create project marker");
    std::fs::write(
        project_root.join(".agentcfg/agentcfg.toml"),
        "version = 1\nconfig-layer = \"user-project\"\nclients = [\"codex\"]\n",
    )
    .expect("write user project config");

    let output = agentcfg()
        .args([
            "skills",
            "select",
            "find-bugs",
            "--id",
            "local",
            "--source",
            "./skills",
        ])
        .current_dir(&project_root)
        .output()
        .expect("run agentcfg");

    assert!(output.status.success());
    let content = std::fs::read_to_string(project_root.join(".agentcfg/agentcfg.toml"))
        .expect("read user project config");
    assert!(content.contains("id = \"local\""));
}

#[test]
fn skills_select_rejects_duplicate_entry_id() {
    let project_root = test_project("cli-select-duplicate-entry-id");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create project marker");
    std::fs::write(
        project_root.join(".agentcfg/agentcfg.toml"),
        r#"
version = 1
config-layer = "user-project"
clients = ["codex"]

[[skills]]
id = "team"
source = "./skills"
include = "all"
"#,
    )
    .expect("write user project config");

    let output = agentcfg()
        .args([
            "skills",
            "select",
            "find-bugs",
            "--id",
            "team",
            "--source",
            "./other",
        ])
        .current_dir(&project_root)
        .output()
        .expect("run agentcfg");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already declared"));
    assert!(stderr.contains("team"));

    let content = std::fs::read_to_string(project_root.join(".agentcfg/agentcfg.toml"))
        .expect("read user project config");
    assert!(!content.contains("find-bugs"));
}

#[test]
fn skills_select_writes_explicit_included_skill_as_text() {
    let project_root = test_project("cli-select-text");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create project marker");
    std::fs::write(
        project_root.join(".agentcfg/agentcfg.toml"),
        "version = 1\nconfig-layer = \"user-project\"\nclients = [\"codex\"]\n",
    )
    .expect("write user project config");

    let output = agentcfg()
        .args(["skills", "select", "find-bugs", "--source", "./skills"])
        .current_dir(&project_root)
        .output()
        .expect("run agentcfg");

    assert!(
        output.status.success(),
        "expected success, stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");
    assert!(stdout.contains("Skill selected"));
    assert!(stdout.contains("find-bugs"));
    assert!(stdout.contains("./skills"));
    assert!(stdout.contains("codex"));
    assert!(stdout.contains("Next: agentcfg install"));
    assert!(stdout.contains("agentcfg skills clients"));
}

#[test]
fn skills_select_persists_github_shorthand_with_ref() {
    let project_root = test_project("cli-select-github-ref");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create project marker");
    std::fs::write(
        project_root.join(".agentcfg/agentcfg.toml"),
        "version = 1\nconfig-layer = \"user-project\"\nclients = [\"codex\"]\n",
    )
    .expect("write user project config");

    let output = agentcfg()
        .args([
            "skills",
            "select",
            "dotagents",
            "--source",
            "getsentry/dotagents",
            "--ref",
            "v1.0.0",
        ])
        .current_dir(&project_root)
        .output()
        .expect("run agentcfg");

    assert!(output.status.success());
    let content = std::fs::read_to_string(project_root.join(".agentcfg/agentcfg.toml"))
        .expect("read user project config");
    assert!(content.contains("source = \"getsentry/dotagents\""));
    assert!(content.contains("ref = \"v1.0.0\""));
}

#[test]
fn skills_select_blocks_without_default_client_selection() {
    let project_root = test_project("cli-select-no-clients");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create project marker");

    let output = agentcfg()
        .args(["skills", "select", "find-bugs", "--source", "./skills"])
        .current_dir(&project_root)
        .output()
        .expect("run agentcfg");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("final client selection"));
    assert!(stderr.contains("agentcfg clients set"));
}

#[test]
fn skills_deselect_emits_install_and_prune_suggestions() {
    let project_root = test_project("cli-deselect-text");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create project marker");
    std::fs::write(
        project_root.join(".agentcfg/agentcfg.toml"),
        r#"
version = 1
config-layer = "user-project"
clients = ["codex"]

[[skills]]
source = "./skills"
include = ["find-bugs"]
"#,
    )
    .expect("write user project config");

    let output = agentcfg()
        .args(["skills", "deselect", "find-bugs", "--source", "./skills"])
        .current_dir(&project_root)
        .output()
        .expect("run agentcfg");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");
    assert!(stdout.contains("Skill deselected"));
    assert!(stdout.contains("Next: agentcfg install"));
    assert!(stdout.contains("Next: agentcfg prune"));
}

#[test]
fn skills_select_emits_json_workflow_result() {
    let project_root = test_project("cli-select-json");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create project marker");
    std::fs::write(
        project_root.join(".agentcfg/agentcfg.toml"),
        "version = 1\nconfig-layer = \"user-project\"\nclients = [\"cursor\"]\n",
    )
    .expect("write user project config");

    let output = agentcfg()
        .args([
            "skills",
            "select",
            "find-bugs",
            "--source",
            "./skills",
            "--format",
            "json",
        ])
        .current_dir(&project_root)
        .output()
        .expect("run agentcfg");

    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is json");
    assert_eq!(value["workflow"], "select_skill");
    assert_eq!(value["data"]["source"], "./skills");
    assert_eq!(value["data"]["source_skill_name"], "find-bugs");
    assert_eq!(value["suggested_actions"][0]["command"], "agentcfg install");
}

#[test]
fn skills_deselect_json_includes_prune_suggestion() {
    let project_root = test_project("cli-deselect-json");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create project marker");
    std::fs::write(
        project_root.join(".agentcfg/agentcfg.toml"),
        r#"
version = 1
config-layer = "user-project"
clients = ["codex"]

[[skills]]
source = "./skills"
include = ["find-bugs"]
"#,
    )
    .expect("write user project config");

    let output = agentcfg()
        .args([
            "skills",
            "deselect",
            "find-bugs",
            "--source",
            "./skills",
            "--format",
            "json",
        ])
        .current_dir(&project_root)
        .output()
        .expect("run agentcfg");

    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is json");
    assert_eq!(value["workflow"], "deselect_skill");
    assert_eq!(value["suggested_actions"].as_array().unwrap().len(), 2);
    assert_eq!(
        value["suggested_actions"][1]["command"],
        json!("agentcfg prune")
    );
}

fn agentcfg() -> Command {
    Command::new(env!("CARGO_BIN_EXE_agentcfg"))
}

fn test_project(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir()
        .join("agentcfg-cli-tests")
        .join(format!("skills-{name}-{}", std::process::id()));
    if root.exists() {
        std::fs::remove_dir_all(&root).expect("remove previous root");
    }
    std::fs::create_dir_all(&root).expect("create root");
    root
}
