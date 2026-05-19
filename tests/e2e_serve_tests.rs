use std::process::{Command, Stdio};
use tempfile::tempdir;

mod e2e_helpers;
use e2e_helpers::run_loom_cmd;

#[test]
fn test_serve_requires_kb_root() {
    // Test that serve command requires KB_ROOT environment variable
    let output = Command::new(env!("CARGO_BIN_EXE_loom"))
        .arg("serve")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute loom serve");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Serve should fail without KB_ROOT
    assert!(
        stderr.contains("KB_ROOT"),
        "loom serve should require KB_ROOT. stderr: {}",
        stderr
    );
}

#[test]
fn test_serve_requires_client() {
    // Test that serve command fails when no MCP client is connected
    // This is expected behavior - serve needs an MCP client to communicate with
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let kb_root = temp_dir.path();

    // Initialize first
    let init_output = run_loom_cmd(&["init"], kb_root);
    assert_eq!(init_output.exit_code, 0);

    // Start serve - it will fail because no client is connected
    let binary_path = env!("CARGO_BIN_EXE_loom");
    let output = Command::new(binary_path)
        .arg("serve")
        .env("KB_ROOT", kb_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute loom serve");

    // Serve will panic when no client connects - this is a known limitation
    // The test documents this behavior for future improvement
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Document that serve currently panics without a client
    // Future improvement: handle this more gracefully
    assert!(
        stderr.contains("ConnectionClosed") || stderr.contains("panicked"),
        "loom serve should fail when no client connects. stderr: {}",
        stderr
    );
}
