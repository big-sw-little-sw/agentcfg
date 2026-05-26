use std::fs;
use std::path::Path;
use std::process::{Command, Output};

#[test]
fn default_init_creates_user_project_config() {
    let temp = tempfile::tempdir().unwrap();

    let output = run_agentcfg(temp.path(), ["init"]);

    assert_success(&output);
    let config_file = temp.path().join(".agentcfg").join("config.toml");
    assert!(config_file.is_file());
    assert_config_scope(&config_file, "user-project");
    assert!(stdout(&output).contains("Created config:"));
}

#[test]
fn project_init_creates_shared_config_without_user_project_dir() {
    let temp = tempfile::tempdir().unwrap();

    let output = run_agentcfg(temp.path(), ["init", "--project"]);

    assert_success(&output);
    let config_file = temp.path().join("agentcfg.toml");
    assert!(config_file.is_file());
    assert_config_scope(&config_file, "shared-project");
    assert!(!temp.path().join(".agentcfg").exists());
}

#[test]
fn user_init_creates_user_config_without_project_dir() {
    let temp = tempfile::tempdir().unwrap();
    let config_home = temp.path().join("xdg-config");
    let state_home = temp.path().join("xdg-state");
    let home = temp.path().join("home");

    let output = Command::new(env!("CARGO_BIN_EXE_agentcfg"))
        .args(["init", "--user"])
        .current_dir(temp.path())
        .env("XDG_CONFIG_HOME", &config_home)
        .env("XDG_STATE_HOME", &state_home)
        .env("HOME", &home)
        .output()
        .expect("failed to run agentcfg");

    assert_success(&output);
    let config_file = config_home.join("agentcfg").join("config.toml");
    assert!(config_file.is_file());
    assert_config_scope(&config_file, "user");
    assert!(!temp.path().join(".agentcfg").exists());
}

#[test]
fn init_refuses_to_overwrite_existing_config() {
    let temp = tempfile::tempdir().unwrap();
    let agentcfg_dir = temp.path().join(".agentcfg");
    let config_file = agentcfg_dir.join("config.toml");
    fs::create_dir(&agentcfg_dir).unwrap();
    fs::write(&config_file, "existing").unwrap();

    let output = run_agentcfg(temp.path(), ["init"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("config already exists"));
    assert_eq!(fs::read_to_string(config_file).unwrap(), "existing");
}

#[test]
fn init_reports_unmanaged_artifacts_without_modifying_targets() {
    let temp = tempfile::tempdir().unwrap();
    let skill = temp.path().join(".agents").join("skills").join("review");
    fs::create_dir_all(&skill).unwrap();
    fs::write(skill.join("SKILL.md"), "review").unwrap();

    let output = run_agentcfg(temp.path(), ["init", "--project"]);

    assert_success(&output);
    let stderr = stderr(&output);
    assert!(stderr.contains("warning: unmanaged skill artifact exists"));
    assert!(stderr.contains("review"));
    assert_eq!(
        fs::read_to_string(skill.join("SKILL.md")).unwrap(),
        "review"
    );
    assert!(!temp.path().join(".claude").exists());
    assert!(!temp.path().join(".cline").exists());
}

fn run_agentcfg<const N: usize>(cwd: &Path, args: [&str; N]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_agentcfg"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("failed to run agentcfg")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "expected success\nstdout:\n{}\nstderr:\n{}",
        stdout(output),
        stderr(output)
    );
}

fn assert_config_scope(path: &Path, scope: &str) {
    assert!(
        fs::read_to_string(path)
            .unwrap()
            .contains(&format!("scope = \"{scope}\""))
    );
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
