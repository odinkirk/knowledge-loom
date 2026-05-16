// Install module tests

use knowledge_loom::install::{InstallManager, InstallState};
use std::path::PathBuf;
use tempfile::TempDir;

fn setup_test_manager() -> (TempDir, InstallManager) {
    let tmp = TempDir::new().unwrap();
    let manager = InstallManager::new(tmp.path().to_path_buf());
    (tmp, manager)
}

#[test]
fn test_install_manager_new() {
    let (_tmp, manager) = setup_test_manager();
    assert_eq!(manager.kb_root(), &PathBuf::from(_tmp.path()));
}

#[test]
fn test_install_manager_model_path() {
    let (_tmp, manager) = setup_test_manager();
    let expected = _tmp.path().join(".knowledge-loom/models");
    assert_eq!(manager.model_path(), expected);
}

#[test]
fn test_install_manager_is_installed_false_initially() {
    let (_tmp, manager) = setup_test_manager();
    assert!(!manager.is_installed());
}

#[test]
fn test_install_manager_state_path() {
    let (_tmp, manager) = setup_test_manager();
    let expected = _tmp.path().join(".knowledge-loom/models/.install-state.json");
    assert_eq!(manager.state_path(), expected);
}

#[tokio::test]
async fn test_install_manager_download_model_mock() {
    let (_tmp, manager) = setup_test_manager();
    
    // Create a mock model file instead of actual download
    let model_dir = manager.model_path();
    std::fs::create_dir_all(&model_dir).unwrap();
    let model_file = model_dir.join("model.onnx");
    std::fs::write(&model_file, b"mock model data").unwrap();
    
    // Create mock state
    let state = InstallState {
        model_version: "test-v1".to_string(),
        download_timestamp: chrono::Utc::now().to_rfc3339(),
        checksum: "abc123".to_string(),
        size_bytes: 15,
    };
    let state_json = serde_json::to_string_pretty(&state).unwrap();
    std::fs::write(manager.state_path(), state_json).unwrap();
    
    assert!(manager.is_installed());
}

#[test]
fn test_install_manager_checksum_validation() {
    let (_tmp, manager) = setup_test_manager();
    
    // Create model directory and file
    let model_dir = manager.model_path();
    std::fs::create_dir_all(&model_dir).unwrap();
    let model_file = model_dir.join("model.onnx");
    let test_data = b"test data for checksum";
    std::fs::write(&model_file, test_data).unwrap();
    
    // Calculate expected checksum
    use sha2::{Digest, Sha256};
    let checksum = Sha256::digest(test_data);
    let checksum_hex = format!("{:x}", checksum);
    
    // Create state with correct checksum
    let state = InstallState {
        model_version: "test-v1".to_string(),
        download_timestamp: chrono::Utc::now().to_rfc3339(),
        checksum: checksum_hex.clone(),
        size_bytes: test_data.len() as u64,
    };
    let state_json = serde_json::to_string_pretty(&state).unwrap();
    std::fs::write(manager.state_path(), state_json).unwrap();
    
    // Verify integrity should pass
    assert!(manager.verify_integrity().unwrap());
    
    // Corrupt the file
    std::fs::write(&model_file, b"corrupted data").unwrap();
    
    // Verify integrity should fail
    assert!(!manager.verify_integrity().unwrap());
}

#[tokio::test]
async fn test_validate_or_download_corrupted_triggers_redownload() {
    let (_tmp, manager) = setup_test_manager();
    
    // Create model directory and file with wrong data
    let model_dir = manager.model_path();
    std::fs::create_dir_all(&model_dir).unwrap();
    let model_file = model_dir.join("model.onnx");
    std::fs::write(&model_file, b"corrupted data").unwrap();
    
    // Create state with wrong checksum (simulating corruption)
    let state = InstallState {
        model_version: "test-v1".to_string(),
        download_timestamp: chrono::Utc::now().to_rfc3339(),
        checksum: "wrongchecksum".to_string(),
        size_bytes: 16,
    };
    let state_json = serde_json::to_string_pretty(&state).unwrap();
    std::fs::write(manager.state_path(), state_json).unwrap();
    
    // validate_or_download with force should succeed (would re-download in real scenario)
    // For this test, we just verify it doesn't return AlreadyInstalled error
    let result = manager.validate_or_download(true).await;
    // Should not be AlreadyInstalled error when force=true
    assert!(!matches!(result, Err(knowledge_loom::install::InstallError::AlreadyInstalled)));
}

#[tokio::test]
async fn test_validate_or_download_missing_triggers_download() {
    let (_tmp, manager) = setup_test_manager();
    
    // Don't create any files - simulate missing model
    
    // validate_or_download should not return AlreadyInstalled when nothing exists
    let result = manager.validate_or_download(false).await;
    // Should attempt download (will fail without network, but shouldn't be AlreadyInstalled)
    assert!(!matches!(result, Err(knowledge_loom::install::InstallError::AlreadyInstalled)));
}

#[tokio::test]
async fn test_force_flag_triggers_redownload() {
    let (_tmp, manager) = setup_test_manager();
    
    // Create valid installed state
    let model_dir = manager.model_path();
    std::fs::create_dir_all(&model_dir).unwrap();
    let model_file = model_dir.join("model.onnx");
    let test_data = b"valid model data";
    std::fs::write(&model_file, test_data).unwrap();
    
    // Calculate checksum
    use sha2::{Digest, Sha256};
    let checksum = Sha256::digest(test_data);
    let checksum_hex = format!("{:x}", checksum);
    
    // Create valid state
    let state = InstallState {
        model_version: "test-v1".to_string(),
        download_timestamp: chrono::Utc::now().to_rfc3339(),
        checksum: checksum_hex,
        size_bytes: test_data.len() as u64,
    };
    let state_json = serde_json::to_string_pretty(&state).unwrap();
    std::fs::write(manager.state_path(), state_json).unwrap();
    
    // Without force, should return AlreadyInstalled
    let result_no_force = manager.validate_or_download(false).await;
    assert!(matches!(result_no_force, Err(knowledge_loom::install::InstallError::AlreadyInstalled)));
    
    // With force, should attempt download (not AlreadyInstalled)
    let result_force = manager.validate_or_download(true).await;
    assert!(!matches!(result_force, Err(knowledge_loom::install::InstallError::AlreadyInstalled)));
}

#[tokio::test]
async fn test_skip_download_when_model_valid() {
    let (_tmp, manager) = setup_test_manager();
    
    // Create valid installed state
    let model_dir = manager.model_path();
    std::fs::create_dir_all(&model_dir).unwrap();
    let model_file = model_dir.join("model.onnx");
    let test_data = b"valid model data for skip test";
    std::fs::write(&model_file, test_data).unwrap();
    
    // Calculate checksum
    use sha2::{Digest, Sha256};
    let checksum = Sha256::digest(test_data);
    let checksum_hex = format!("{:x}", checksum);
    
    // Create valid state
    let state = InstallState {
        model_version: "test-v1".to_string(),
        download_timestamp: chrono::Utc::now().to_rfc3339(),
        checksum: checksum_hex,
        size_bytes: test_data.len() as u64,
    };
    let state_json = serde_json::to_string_pretty(&state).unwrap();
    std::fs::write(manager.state_path(), state_json).unwrap();
    
    // Verify integrity passes
    assert!(manager.verify_integrity().unwrap());
    
    // validate_or_download without force should return AlreadyInstalled
    let result = manager.validate_or_download(false).await;
    assert!(matches!(result, Err(knowledge_loom::install::InstallError::AlreadyInstalled)));
}

#[tokio::test]
async fn test_error_when_force_not_provided_and_model_exists() {
    let (_tmp, manager) = setup_test_manager();
    
    // Create valid installed state
    let model_dir = manager.model_path();
    std::fs::create_dir_all(&model_dir).unwrap();
    let model_file = model_dir.join("model.onnx");
    let test_data = b"valid model data";
    std::fs::write(&model_file, test_data).unwrap();
    
    // Calculate checksum
    use sha2::{Digest, Sha256};
    let checksum = Sha256::digest(test_data);
    let checksum_hex = format!("{:x}", checksum);
    
    // Create valid state
    let state = InstallState {
        model_version: "test-v1".to_string(),
        download_timestamp: chrono::Utc::now().to_rfc3339(),
        checksum: checksum_hex,
        size_bytes: test_data.len() as u64,
    };
    let state_json = serde_json::to_string_pretty(&state).unwrap();
    std::fs::write(manager.state_path(), state_json).unwrap();
    
    // Should return AlreadyInstalled error with helpful message
    let result = manager.validate_or_download(false).await;
    assert!(matches!(result, Err(knowledge_loom::install::InstallError::AlreadyInstalled)));
}
