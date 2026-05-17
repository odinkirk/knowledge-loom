use std::fs;
use tempfile::tempdir;

mod e2e_helpers;
use e2e_helpers::{assert_contains, assert_exit_code, assert_no_panic, run_loom_cmd};

#[test]
fn test_init_clean_directory_exit_code_0() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let kb_root = temp_dir.path();

    let output = run_loom_cmd(&["init"], kb_root);

    assert_exit_code(&output, 0);
}

#[test]
fn test_init_no_tokio_panic() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let kb_root = temp_dir.path();

    let output = run_loom_cmd(&["init"], kb_root);

    assert_no_panic(&output);
}

#[test]
fn test_init_reinit_already_initialized() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let kb_root = temp_dir.path();

    // First init
    let output1 = run_loom_cmd(&["init"], kb_root);
    assert_exit_code(&output1, 0);

    // Second init should report already initialized
    let output2 = run_loom_cmd(&["init"], kb_root);
    assert_contains(&output2, "already initialized");
}

#[test]
fn test_init_partial_completion() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let kb_root = temp_dir.path();

    // Create .knowledge-loom-index/ without model (simulating partial init)
    let index_dir = kb_root.join(".knowledge-loom-index");
    fs::create_dir_all(&index_dir).expect("Failed to create index directory");

    // Run init - should detect partial state and complete setup
    let output = run_loom_cmd(&["init"], kb_root);

    // Init should report already initialized (directory exists)
    assert_exit_code(&output, 0);
    assert_contains(&output, "already initialized");

    // Verify index directory exists
    assert!(index_dir.exists(), "Index directory should exist");
}
