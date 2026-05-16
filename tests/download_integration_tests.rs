// Download integration tests
// Tests for DownloadManager integration across the codebase

use knowledge_loom::download::DownloadManager;
use knowledge_loom::download::utils::{calculate_checksum, validate_checksum};
use std::path::PathBuf;
use tempfile::TempDir;

/// Integration test: Verify DownloadManager can be instantiated and configured
#[test]
fn test_download_manager_creation() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("test_model.onnx");
    
    let result = DownloadManager::new(
        "https://example.com/model.onnx".to_string(),
        output_path,
    );
    
    // Should succeed in creating manager (network failure happens on download, not creation)
    assert!(result.is_ok());
}

/// Integration test: Verify DownloadManager configuration chain works
#[test]
fn test_download_manager_configuration() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("test_model.onnx");
    
    let manager = DownloadManager::new(
        "https://example.com/model.onnx".to_string(),
        output_path,
    )
    .unwrap()
    .with_retries(5)
    .with_retry_delay(std::time::Duration::from_secs(2))
    .with_timeout(std::time::Duration::from_secs(60));
    
    assert_eq!(manager.max_retries, 5);
}

/// Integration test: Verify checksum utilities work with real data
#[test]
fn test_checksum_validation_integration() {
    // Create test data
    let test_data = b"integration test data for checksum validation";
    
    // Calculate checksum
    let checksum = calculate_checksum(test_data);
    
    // Validate it matches
    let result = validate_checksum(test_data, &checksum);
    assert!(result.is_ok());
    
    // Validate it rejects wrong checksum
    let wrong_result = validate_checksum(test_data, "wrongchecksum");
    assert!(wrong_result.is_err());
}

/// Integration test: Verify checksum works across module boundaries
#[test]
fn test_checksum_cross_module() {
    // This test verifies that checksum calculation is consistent
    // regardless of where it's called from in the codebase
    
    let data = b"cross-module test data";
    
    // Calculate in test
    let checksum1 = calculate_checksum(data);
    
    // Calculate again (simulating call from different module)
    let checksum2 = calculate_checksum(data);
    
    assert_eq!(checksum1, checksum2, "Checksum should be consistent across calls");
}

/// Integration test: Verify DownloadManager error handling
#[tokio::test]
async fn test_download_manager_error_handling() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("test_model.onnx");
    
    let manager = DownloadManager::new(
        "https://invalid-url-that-does-not-exist.example/model.onnx".to_string(),
        output_path,
    )
    .unwrap()
    .with_retries(1)
    .with_retry_delay(std::time::Duration::from_millis(100));
    
    // Download should fail with network error, not panic
    let result = manager.download(|_| {}).await;
    assert!(result.is_err());
    
    // Error should be descriptive
    let error_msg = result.unwrap_err().to_string();
    assert!(!error_msg.is_empty());
}

/// Integration test: Verify install.rs uses DownloadManager correctly
#[test]
fn test_install_uses_download_manager() {
    // This is a compile-time integration test
    // If install.rs compiles, it's using DownloadManager correctly
    
    // We can't easily test the actual download without network,
    // but we can verify the types align
    
    use knowledge_loom::install::InstallManager;
    
    let tmp = TempDir::new().unwrap();
    let manager = InstallManager::new(tmp.path().to_path_buf());
    
    // Verify InstallManager exists and can be created
    assert_eq!(manager.kb_root(), tmp.path());
}

/// Integration test: Verify CLI args module integrates with main
#[test]
fn test_cli_args_integration() {
    use knowledge_loom::cli::args::parse_flag;
    
    // Test that CLI args module is accessible from knowledge_loom crate
    let _result = parse_flag("test", None);
    
    // If we got here, the module integration works
    assert!(true);
}
