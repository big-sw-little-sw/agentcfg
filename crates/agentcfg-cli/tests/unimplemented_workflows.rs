use std::process::Command;

#[test]
fn preview_exits_nonzero_when_not_implemented() {
    let output = Command::new(env!("CARGO_BIN_EXE_agentcfg"))
        .args(["preview"])
        .output()
        .expect("run preview");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("preview is not implemented yet"),
        "stderr was: {stderr}"
    );
}

#[test]
fn apply_exits_nonzero_when_not_implemented() {
    let output = Command::new(env!("CARGO_BIN_EXE_agentcfg"))
        .args(["apply"])
        .output()
        .expect("run apply");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("apply is not implemented yet"),
        "stderr was: {stderr}"
    );
}
