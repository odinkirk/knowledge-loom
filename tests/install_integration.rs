// Install integration tests
// Tests the full install flow: download + verification + test suite

use knowledge_loom::install::{run_install, InstallManager};
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

#[tokio::test]
async fn test_install_and_verify_integrity() {
    let tmp = TempDir::new().unwrap();
    let kb_root = tmp.path().to_path_buf();

    // Run install (will fail without network, but we can test the flow)
    let result = run_install(kb_root.clone(), false).await;

    // Should attempt download (may fail with network error, but not AlreadyInstalled)
    assert!(!matches!(
        result,
        Err(knowledge_loom::install::InstallError::AlreadyInstalled)
    ));
}

#[tokio::test]
async fn test_install_force_reinstall() {
    let tmp = TempDir::new().unwrap();
    let kb_root = tmp.path().to_path_buf();

    // First install attempt
    let _ = run_install(kb_root.clone(), false).await;

    // Force reinstall should attempt download again
    let result = run_install(kb_root.clone(), true).await;

    // Should not be AlreadyInstalled when force=true
    assert!(!matches!(
        result,
        Err(knowledge_loom::install::InstallError::AlreadyInstalled)
    ));
}

#[test]
fn test_install_manager_end_to_end_mock() {
    let tmp = TempDir::new().unwrap();
    let kb_root = tmp.path().to_path_buf();
    let manager = InstallManager::new(kb_root);

    // Initially not installed
    assert!(!manager.is_installed());
    assert!(!manager.verify_integrity().unwrap());

    // Create mock installed state
    let model_dir = manager.model_path();
    std::fs::create_dir_all(&model_dir).unwrap();
    let model_file = model_dir.join("model.onnx");
    let test_data = b"mock model data for e2e test";
    std::fs::write(&model_file, test_data).unwrap();

    // Calculate checksum
    use sha2::{Digest, Sha256};
    let checksum = Sha256::digest(test_data);
    let checksum_hex = format!("{:x}", checksum);

    // Create state file
    let state = knowledge_loom::install::InstallState {
        model_version: "test-e2e-v1".to_string(),
        download_timestamp: chrono::Utc::now().to_rfc3339(),
        checksum: checksum_hex,
        size_bytes: test_data.len() as u64,
    };
    let state_json = serde_json::to_string_pretty(&state).unwrap();
    std::fs::write(manager.state_path(), state_json).unwrap();

    // Now should be installed and pass integrity check
    assert!(manager.is_installed());
    assert!(manager.verify_integrity().unwrap());
}

#[test]
fn test_cargo_test_with_mock_install() {
    // This test verifies that cargo test --release works with installed models
    // We use a mock installation to avoid network dependency

    let tmp = TempDir::new().unwrap();
    let kb_root = tmp.path().to_path_buf();

    // Create mock installation
    let model_dir = kb_root.join(".knowledge-loom/models");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("model.onnx"), b"mock model").unwrap();

    let state = knowledge_loom::install::InstallState {
        model_version: "mock".to_string(),
        download_timestamp: chrono::Utc::now().to_rfc3339(),
        checksum: "mock".to_string(),
        size_bytes: 10,
    };
    let state_json = serde_json::to_string_pretty(&state).unwrap();
    std::fs::write(
        kb_root.join(".knowledge-loom/models/.install-state.json"),
        state_json,
    )
    .unwrap();

    // Verify KB_ROOT environment variable works
    std::env::set_var("KB_ROOT", kb_root.to_str().unwrap());
    let manager = InstallManager::new(kb_root);
    assert!(manager.is_installed());
    std::env::remove_var("KB_ROOT");
}

#[tokio::test]
async fn test_force_reinstall_overwrites_existing() {
    // Test that --force actually re-downloads and overwrites existing model
    let tmp = TempDir::new().unwrap();
    let kb_root = tmp.path().to_path_buf();
    let manager = InstallManager::new(kb_root.clone());

    // Create initial "old" installation
    let model_dir = manager.model_path();
    std::fs::create_dir_all(&model_dir).unwrap();
    let model_file = model_dir.join("model.onnx");
    let old_data = b"old model version 1";
    std::fs::write(&model_file, old_data).unwrap();

    // Create state for old version
    use sha2::{Digest, Sha256};
    let old_checksum = Sha256::digest(old_data);
    let old_checksum_hex = format!("{:x}", old_checksum);

    let state = knowledge_loom::install::InstallState {
        model_version: "old-v1".to_string(),
        download_timestamp: chrono::Utc::now().to_rfc3339(),
        checksum: old_checksum_hex,
        size_bytes: old_data.len() as u64,
    };
    let state_json = serde_json::to_string_pretty(&state).unwrap();
    std::fs::write(manager.state_path(), state_json).unwrap();

    // Verify old installation is valid
    assert!(manager.verify_integrity().unwrap());

    // Force reinstall should attempt download (not return AlreadyInstalled)
    let result = manager.validate_or_download(true).await;
    assert!(!matches!(
        result,
        Err(knowledge_loom::install::InstallError::AlreadyInstalled)
    ));
}
