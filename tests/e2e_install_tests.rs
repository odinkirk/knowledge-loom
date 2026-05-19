use std::env;
use std::fs;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

mod e2e_helpers;
use e2e_helpers::{assert_contains, assert_exit_code, assert_no_panic, run_loom_cmd};

/// Helper to check if network tests should run
/// Set LOOM_TEST_NETWORK=1 to enable network-dependent tests
fn should_run_network_test() -> bool {
    env::var("LOOM_TEST_NETWORK").unwrap_or_default() == "1"
}

#[test]
fn test_install_clean_directory() {
    if !should_run_network_test() {
        eprintln!("Skipping network test: set LOOM_TEST_NETWORK=1 to enable");
        return;
    }

    let temp_dir = tempdir().expect("Failed to create temp directory");
    let kb_root = temp_dir.path();

    // Create .knowledge-loom/ directory structure
    let knowledge_loom_dir = kb_root.join(".knowledge-loom");
    fs::create_dir_all(&knowledge_loom_dir).expect("Failed to create .knowledge-loom directory");

    let output = run_loom_cmd(&["install"], kb_root);

    assert_exit_code(&output, 0);
    assert_no_panic(&output);

    // Verify model exists
    let model_dir = kb_root.join(".knowledge-loom").join("models");
    assert!(
        model_dir.exists(),
        "Model directory should exist after install"
    );
}

#[test]
fn test_install_skip_valid_model() {
    if !should_run_network_test() {
        eprintln!("Skipping network test: set LOOM_TEST_NETWORK=1 to enable");
        return;
    }

    let temp_dir = tempdir().expect("Failed to create temp directory");
    let kb_root = temp_dir.path();

    let knowledge_loom_dir = kb_root.join(".knowledge-loom");
    fs::create_dir_all(&knowledge_loom_dir).expect("Failed to create .knowledge-loom directory");

    // First install
    let output1 = run_loom_cmd(&["install"], kb_root);
    assert_exit_code(&output1, 0);

    // Second install should skip
    let output2 = run_loom_cmd(&["install"], kb_root);
    assert_contains(&output2, "already installed");
}

#[test]
fn test_install_force_redownload() {
    if !should_run_network_test() {
        eprintln!("Skipping network test: set LOOM_TEST_NETWORK=1 to enable");
        return;
    }

    let temp_dir = tempdir().expect("Failed to create temp directory");
    let kb_root = temp_dir.path();

    let knowledge_loom_dir = kb_root.join(".knowledge-loom");
    fs::create_dir_all(&knowledge_loom_dir).expect("Failed to create .knowledge-loom directory");

    // First install
    let output1 = run_loom_cmd(&["install"], kb_root);
    assert_exit_code(&output1, 0);

    // Get timestamp of model file (not directory - directory mtime is filesystem-dependent)
    let model_file = kb_root
        .join(".knowledge-loom")
        .join("models")
        .join("model.onnx");
    let first_timestamp = fs::metadata(&model_file)
        .expect("Failed to get model metadata")
        .modified()
        .expect("Failed to get modified time");

    // Wait a bit to ensure timestamp difference
    thread::sleep(Duration::from_secs(2));

    // Force re-download
    let output2 = run_loom_cmd(&["install", "--force"], kb_root);
    assert_exit_code(&output2, 0);

    // Verify model was re-downloaded (timestamp should be newer)
    let second_timestamp = fs::metadata(&model_file)
        .expect("Failed to get model metadata")
        .modified()
        .expect("Failed to get modified time");

    assert!(
        second_timestamp > first_timestamp,
        "Model should be re-downloaded with --force flag"
    );
}

#[test]
fn test_install_corrupted_model() {
    if !should_run_network_test() {
        eprintln!("Skipping network test: set LOOM_TEST_NETWORK=1 to enable");
        return;
    }

    let temp_dir = tempdir().expect("Failed to create temp directory");
    let kb_root = temp_dir.path();

    let knowledge_loom_dir = kb_root.join(".knowledge-loom");
    let model_dir = knowledge_loom_dir.join("models");
    fs::create_dir_all(&model_dir).expect("Failed to create model directory");

    // Write garbage to model file
    let model_file = model_dir.join("model.onnx");
    fs::write(&model_file, "corrupted data").expect("Failed to write corrupted model");

    // Install should detect corruption and re-download
    let output = run_loom_cmd(&["install"], kb_root);

    assert_exit_code(&output, 0);
    assert_no_panic(&output);

    // Verify model was re-downloaded (file size should be larger than garbage)
    let metadata = fs::metadata(&model_file).expect("Failed to get model metadata");
    assert!(
        metadata.len() > 100,
        "Model should be re-downloaded (size should be > 100 bytes)"
    );
}
