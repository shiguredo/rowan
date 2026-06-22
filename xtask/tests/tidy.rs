use std::process::Command;

#[test]
fn test_formatting() {
    let status = Command::new("cargo").args(["fmt", "--all", "--", "--check"]).status().unwrap();
    assert!(status.success());
}
