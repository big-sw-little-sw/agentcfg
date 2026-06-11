use std::process::Command;

#[test]
fn reports_project_default_clients_as_text() {
    let project_root = test_project_root("clients-show-text");
    std::fs::write(
        project_root.join("agentcfg.toml"),
        "default_clients = [\"codex\"]\n",
    )
    .expect("write shared config");
    std::fs::create_dir_all(project_root.join(".agentcfg")).expect("create user project dir");
    std::fs::write(
        project_root.join(".agentcfg").join("agentcfg.toml"),
        "default_clients = [\"cursor\", \"claude-code\"]\n",
    )
    .expect("write user project config");

    let output = Command::new(env!("CARGO_BIN_EXE_agentcfg"))
        .args(["clients", "show"])
        .current_dir(&project_root)
        .output()
        .expect("run agentcfg");

    assert!(
        output.status.success(),
        "expected success, stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let project_root = std::fs::canonicalize(project_root).expect("canonicalize project root");
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is utf8"),
        format!(
            "Default Client Selection\n\
             Install Level: project\n\
             Config Layers:\n\
             - Shared Project Config: codex ({})\n\
             - User Project Config: cursor, claude-code ({})\n",
            project_root.join("agentcfg.toml").display(),
            project_root
                .join(".agentcfg")
                .join("agentcfg.toml")
                .display()
        )
    );
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr is utf8"),
        ""
    );
}

#[test]
fn can_inspect_user_config_with_level_user() {
    let project_root = test_project_root("clients-show-user");
    let config_home = project_root.join("xdg-config-home");
    let user_config_dir = config_home.join("agentcfg");
    std::fs::create_dir_all(&user_config_dir).expect("create user config dir");
    std::fs::write(
        user_config_dir.join("agentcfg.toml"),
        "default_clients = [\"opencode\", \"pi\"]\n",
    )
    .expect("write user config");

    let output = Command::new(env!("CARGO_BIN_EXE_agentcfg"))
        .args(["clients", "show", "--level", "user"])
        .current_dir(project_root)
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
        format!(
            "Default Client Selection\n\
             Install Level: user\n\
             Config Layers:\n\
             - User Config: opencode, pi ({})\n",
            user_config_dir.join("agentcfg.toml").display()
        )
    );
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
