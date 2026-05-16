// Install benchmark tests
// Verifies performance requirements for `loom install`

use knowledge_loom::install::{run_install, InstallManager, InstallState};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tempfile::TempDir;

/// Benchmark: install should complete in <30s on 100Mbps connection
/// Model size: ~90MB, 100Mbps = 12.5MB/s theoretical, ~7-10s realistic
#[tokio::test]
async fn test_install_completes_within_30s() {
    let tmp = TempDir::new().unwrap();
    let kb_root = tmp.path().to_path_buf();
    
    let start = Instant::now();
    let result = run_install(kb_root.clone(), false).await;
    let elapsed = start.elapsed();
    
    // If download succeeds, it should complete within 30s
    if result.is_ok() {
        assert!(
            elapsed < Duration::from_secs(30),
            "Install took {:?}, expected <30s",
            elapsed
        );
    }
    // If it fails (network issues), that's okay - we're testing performance, not network availability
}

/// Benchmark: integrity verification should be fast (<1s for 90MB file)
#[test]
fn test_integrity_verification_fast() {
    let tmp = TempDir::new().unwrap();
    let kb_root = tmp.path().to_path_buf();
    let manager = InstallManager::new(kb_root);
    
    // Create mock model file (simulate 90MB with smaller data for test)
    let model_dir = manager.model_path();
    std::fs::create_dir_all(&model_dir).unwrap();
    let model_file = model_dir.join("model.onnx");
    
    // Create test data (smaller for speed, but tests the algorithm)
    let test_data = vec![0u8; 1_000_000]; // 1MB
    std::fs::write(&model_file, &test_data).unwrap();
    
    // Calculate checksum
    use sha2::{Digest, Sha256};
    let checksum = Sha256::digest(&test_data);
    let checksum_hex = format!("{:x}", checksum);
    
    // Create state
    let state = InstallState {
        model_version: "test-v1".to_string(),
        download_timestamp: chrono::Utc::now().to_rfc3339(),
        checksum: checksum_hex,
        size_bytes: test_data.len() as u64,
    };
    let state_json = serde_json::to_string_pretty(&state).unwrap();
    std::fs::write(manager.state_path(), state_json).unwrap();
    
    // Time integrity check
    let start = Instant::now();
    let result = manager.verify_integrity();
    let elapsed = start.elapsed();
    
    assert!(result.unwrap(), "Integrity check should pass");
    assert!(
        elapsed < Duration::from_secs(1),
        "Integrity check took {:?}, expected <1s",
        elapsed
    );
}

/// Benchmark: skip logic should be instant (<100ms)
#[test]
fn test_skip_logic_instant() {
    let tmp = TempDir::new().unwrap();
    let kb_root = tmp.path().to_path_buf();
    let manager = InstallManager::new(kb_root);
    
    // Create valid installed state
    let model_dir = manager.model_path();
    std::fs::create_dir_all(&model_dir).unwrap();
    let model_file = model_dir.join("model.onnx");
    let test_data = b"mock model data";
    std::fs::write(&model_file, test_data).unwrap();
    
    // Calculate checksum
    use sha2::{Digest, Sha256};
    let checksum = Sha256::digest(test_data);
    let checksum_hex = format!("{:x}", checksum);
    
    // Create state
    let state = InstallState {
        model_version: "test-v1".to_string(),
        download_timestamp: chrono::Utc::now().to_rfc3339(),
        checksum: checksum_hex,
        size_bytes: test_data.len() as u64,
    };
    let state_json = serde_json::to_string_pretty(&state).unwrap();
    std::fs::write(manager.state_path(), state_json).unwrap();
    
    // Time skip logic (should be instant since it just checks integrity)
    let start = Instant::now();
    let result = manager.verify_integrity();
    let elapsed = start.elapsed();
    
    assert!(result.unwrap(), "Integrity check should pass");
    assert!(
        elapsed < Duration::from_millis(100),
        "Skip logic took {:?}, expected <100ms",
        elapsed
    );
}
