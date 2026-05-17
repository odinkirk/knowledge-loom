use std::process::{Command, Stdio};
use tempfile::tempdir;

mod e2e_helpers;
use e2e_helpers::run_loom_cmd;

#[test]
fn test_shell_requires_kb_root() {
    // Test that shell command requires KB_ROOT environment variable
    let output = Command::new(env!("CARGO_BIN_EXE_loom"))
        .arg("shell")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute loom shell");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Shell should fail without KB_ROOT
    assert!(
        stderr.contains("KB_ROOT"),
        "loom shell should require KB_ROOT. stderr: {}",
        stderr
    );
}

#[test]
fn test_shell_requires_client() {
    // Test that shell command starts and provides interactive shell
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let kb_root = temp_dir.path();

    // Initialize first
    let init_output = run_loom_cmd(&["init"], kb_root);
    assert_eq!(init_output.exit_code, 0);

    // Start shell - it should start the MCP server and provide a shell
    let binary_path = env!("CARGO_BIN_EXE_loom");
    let output = Command::new(binary_path)
        .arg("shell")
        .env("KB_ROOT", kb_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute loom shell");

    // Shell should start successfully
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check that shell started (should mention MCP server or shell)
    assert!(
        stderr.contains("shell") || stderr.contains("MCP"),
        "loom shell should start. stderr: {}",
        stderr
    );
}
