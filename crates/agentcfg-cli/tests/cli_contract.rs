use std::process::Command;

#[test]
fn help_exits_zero() {
    let output = Command::new(env!("CARGO_BIN_EXE_agentcfg"))
        .arg("--help")
        .output()
        .expect("failed to run agentcfg");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
}

#[test]
fn version_exits_zero() {
    let output = Command::new(env!("CARGO_BIN_EXE_agentcfg"))
        .arg("--version")
        .output()
        .expect("failed to run agentcfg");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
}

#[test]
fn invalid_command_form_exits_two_without_double_error_prefix() {
    let output = Command::new(env!("CARGO_BIN_EXE_agentcfg"))
        .args(["doctor", "--user"])
        .output()
        .expect("failed to run agentcfg");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty());
    assert!(
        !stderr.contains("usage error: error:"),
        "unexpected stderr: {stderr}"
    );
}
