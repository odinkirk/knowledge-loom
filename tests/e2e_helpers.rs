use std::path::Path;
use std::process::{Command, Stdio};

/// Output from running a loom command
#[allow(dead_code)]
pub struct CommandOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Run a loom command with the given arguments and KB_ROOT
#[allow(dead_code)]
pub fn run_loom_cmd(args: &[&str], kb_root: &Path) -> CommandOutput {
    let binary_path = env!("CARGO_BIN_EXE_loom");

    let mut cmd = Command::new(binary_path);
    cmd.args(args);
    cmd.env("KB_ROOT", kb_root);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd.output().expect("Failed to execute loom command");

    CommandOutput {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

/// Assert the command exited with the expected code
#[allow(dead_code)]
pub fn assert_exit_code(output: &CommandOutput, expected: i32) {
    assert_eq!(
        output.exit_code, expected,
        "Expected exit code {}, got {}. stderr: {}",
        expected, output.exit_code, output.stderr
    );
}

/// Assert the command did not panic (check stderr for panic messages)
#[allow(dead_code)]
pub fn assert_no_panic(output: &CommandOutput) {
    assert!(
        !output.stderr.contains("panicked"),
        "Command panicked. stderr: {}",
        output.stderr
    );
    assert!(
        !output
            .stderr
            .contains("Cannot start a runtime from within a runtime"),
        "Tokio runtime panic detected. stderr: {}",
        output.stderr
    );
}

/// Assert the output contains the expected string
#[allow(dead_code)]
pub fn assert_contains(output: &CommandOutput, expected: &str) {
    assert!(
        output.stdout.contains(expected) || output.stderr.contains(expected),
        "Expected output to contain '{}'. stdout: {}, stderr: {}",
        expected,
        output.stdout,
        output.stderr
    );
}
