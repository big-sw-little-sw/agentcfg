use std::process::Command;

#[test]
fn invalid_command_form_exits_two() {
    let output = Command::new(env!("CARGO_BIN_EXE_agentcfg"))
        .args(["doctor", "--user"])
        .output()
        .expect("failed to run agentcfg");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty());
    assert!(!stderr.contains("usage error: error:"));
}
