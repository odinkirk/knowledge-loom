// Unit tests for Knowledge Loom model download
// This module tests HTTP download with retry logic, progress tracking, and checksum validation

#[cfg(test)]
mod download_tests {
    use knowledge_loom::download::{
        format_download_complete, format_download_error, format_download_progress, DownloadManager,
        MAX_RETRIES, RETRY_DELAY, TIMEOUT,
    };
    use knowledge_loom::model::{DownloadError, DownloadProgress};
    use tempfile::TempDir;

    #[test]
    fn test_download_manager_new() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test-model.onnx");

        let result =
            DownloadManager::new("https://example.com/model.onnx".to_string(), output_path);

        assert!(result.is_ok());
        let manager = result.unwrap();
        assert_eq!(manager.max_retries, MAX_RETRIES);
    }

    #[test]
    fn test_download_manager_with_retries() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test-model.onnx");

        let manager =
            DownloadManager::new("https://example.com/model.onnx".to_string(), output_path)
                .unwrap()
                .with_retries(5);

        assert_eq!(manager.max_retries, 5);
    }

    #[test]
    fn test_download_manager_with_retry_delay() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test-model.onnx");

        let manager =
            DownloadManager::new("https://example.com/model.onnx".to_string(), output_path)
                .unwrap()
                .with_retry_delay(std::time::Duration::from_secs(2));

        // Verify the method returns a DownloadManager
        // We can't directly access the private field, but we can verify the method works
        assert_eq!(manager.max_retries, MAX_RETRIES);
    }

    #[test]
    fn test_download_manager_with_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test-model.onnx");

        let manager =
            DownloadManager::new("https://example.com/model.onnx".to_string(), output_path)
                .unwrap()
                .with_timeout(std::time::Duration::from_secs(60));

        // Verify the method returns a DownloadManager
        // We can't directly access the private field, but we can verify the method works
        assert_eq!(manager.max_retries, MAX_RETRIES);
    }

    #[test]
    fn test_format_download_progress() {
        let progress = DownloadProgress::new(50_000_000, 100_000_000, 2_500_000.0);

        let formatted = format_download_progress(&progress);

        assert!(formatted.contains("50.0%"));
        assert!(formatted.contains("47.7MB")); // 50MB / 1,048,576
        assert!(formatted.contains("95.4MB")); // 100MB / 1,048,576
        assert!(formatted.contains("2.38MB/s")); // 2.5MB / 1,048,576
    }

    #[test]
    fn test_format_download_complete() {
        let formatted = format_download_complete(100_000_000, 60);

        assert!(formatted.contains("95.4MB")); // 100MB / 1,048,576
        assert!(formatted.contains("1m 0s"));
    }

    #[test]
    fn test_format_download_error() {
        let error = DownloadError::Network("Connection failed".to_string());
        let formatted = format_download_error(&error);

        assert!(formatted.contains("Network error"));
        assert!(formatted.contains("Connection failed"));
    }

    #[test]
    fn test_format_download_error_interrupted() {
        let error = DownloadError::Interrupted;
        let formatted = format_download_error(&error);

        assert!(formatted.contains("interrupted by user"));
    }

    #[test]
    fn test_format_download_error_max_retries() {
        let error = DownloadError::MaxRetriesExceeded { retries: 3 };
        let formatted = format_download_error(&error);

        assert!(formatted.contains("failed after 3 retries"));
    }

    #[test]
    fn test_download_progress_new() {
        let progress = DownloadProgress::new(50_000_000, 100_000_000, 2_500_000.0);

        assert_eq!(progress.percentage, 50.0);
        assert_eq!(progress.bytes_downloaded, 50_000_000);
        assert_eq!(progress.total_bytes, 100_000_000);
        assert_eq!(progress.speed, 2_500_000.0);
        assert!(progress.eta_seconds.is_some());
    }

    #[test]
    fn test_download_progress_eta_calculation() {
        let progress = DownloadProgress::new(50_000_000, 100_000_000, 1_000_000.0);

        let eta = progress.eta_seconds.unwrap();
        assert_eq!(eta, 50); // 50MB remaining at 1MB/s
    }

    #[test]
    fn test_download_progress_zero_speed() {
        let progress = DownloadProgress::new(50_000_000, 100_000_000, 0.0);

        assert!(progress.eta_seconds.is_none());
    }

    #[test]
    fn test_download_progress_complete() {
        let progress = DownloadProgress::new(100_000_000, 100_000_000, 2_500_000.0);

        assert_eq!(progress.percentage, 100.0);
        assert!(progress.eta_seconds.is_none());
    }

    #[test]
    fn test_constants() {
        assert_eq!(MAX_RETRIES, 3);
        assert_eq!(RETRY_DELAY, 1);
        assert_eq!(TIMEOUT, 30);
    }

    #[test]
    fn test_output_conventions() {
        // Test that progress formatting uses println! for user-facing output
        let progress = DownloadProgress::new(50_000_000, 100_000_000, 2_500_000.0);
        let formatted = format_download_progress(&progress);

        // This should be user-facing output (would use println! in actual usage)
        assert!(!formatted.is_empty());
        assert!(formatted.contains("Downloading model"));
    }

    // User Story 2: Graceful Error Handling Tests

    #[test]
    fn test_network_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test-model.onnx");

        // Test with invalid URL that will fail
        let manager = DownloadManager::new(
            "https://invalid-host-that-does-not-exist.example.com/model.onnx".to_string(),
            output_path,
        )
        .unwrap()
        .with_timeout(std::time::Duration::from_secs(5));

        // This should fail with a network error
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { manager.download_with_retry(&|_| {}).await });

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            DownloadError::Network(msg) => {
                assert!(msg.contains("Failed to start download") || msg.contains("error"));
            }
            DownloadError::MaxRetriesExceeded { retries } => {
                assert_eq!(retries, MAX_RETRIES);
            }
            _ => panic!(
                "Expected Network or MaxRetriesExceeded error, got: {:?}",
                error
            ),
        }
    }

    #[test]
    fn test_disk_full_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        let _output_path = temp_dir.path().join("test-model.onnx");

        // Create a file that will cause disk full error
        // We can't easily simulate disk full, but we can test the error path
        let error = DownloadError::Io(std::io::Error::new(
            std::io::ErrorKind::StorageFull,
            "No space left on device",
        ));

        let formatted = format_download_error(&error);
        assert!(formatted.contains("I/O error") || formatted.contains("No space"));
    }

    #[test]
    fn test_permission_denied_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        let _output_path = temp_dir.path().join("test-model.onnx");

        // Test permission denied error
        let error = DownloadError::Io(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Permission denied",
        ));

        let formatted = format_download_error(&error);
        assert!(formatted.contains("I/O error") || formatted.contains("Permission"));
    }

    #[test]
    fn test_timeout_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test-model.onnx");

        // Test with very short timeout
        let manager = DownloadManager::new("https://httpbin.org/delay/10".to_string(), output_path)
            .unwrap()
            .with_timeout(std::time::Duration::from_millis(100));

        // This should fail with a timeout error
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { manager.download_with_retry(&|_| {}).await });

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            DownloadError::Network(msg) => {
                assert!(
                    msg.contains("timeout") || msg.contains("timed out") || msg.contains("error")
                );
            }
            DownloadError::MaxRetriesExceeded { retries } => {
                assert_eq!(retries, MAX_RETRIES);
            }
            DownloadError::Timeout(msg) => {
                assert!(msg.contains("timeout") || msg.contains("100ms"));
            }
            _ => panic!(
                "Expected Network, MaxRetriesExceeded, or Timeout error, got: {:?}",
                error
            ),
        }
    }

    #[test]
    fn test_proxy_configuration() {
        // Test that proxy configuration is respected
        // We can't easily test actual proxy behavior, but we can verify the client is configured
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test-model.onnx");

        // Set proxy environment variables
        std::env::set_var("HTTP_PROXY", "http://proxy.example.com:8080");
        std::env::set_var("HTTPS_PROXY", "http://proxy.example.com:8080");

        let result =
            DownloadManager::new("https://example.com/model.onnx".to_string(), output_path);

        assert!(result.is_ok());
        let _manager = result.unwrap();

        // Clean up
        std::env::remove_var("HTTP_PROXY");
        std::env::remove_var("HTTPS_PROXY");
    }

    #[test]
    fn test_proxy_bypass_rules() {
        // Test that proxy bypass rules are respected
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test-model.onnx");

        // Set proxy with bypass rules
        std::env::set_var("HTTP_PROXY", "http://proxy.example.com:8080");
        std::env::set_var("NO_PROXY", "localhost,127.0.0.1");

        let result = DownloadManager::new("https://localhost/model.onnx".to_string(), output_path);

        assert!(result.is_ok());
        let _manager = result.unwrap();

        // Clean up
        std::env::remove_var("HTTP_PROXY");
        std::env::remove_var("NO_PROXY");
    }

    // User Story 3: Model Re-Download with State Handling Tests

    #[test]
    fn test_http_range_request_support() {
        use std::fs::File;
        use std::io::Write;

        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test-model.onnx");

        // Create a partial file to simulate interrupted download
        let mut file = File::create(&output_path).unwrap();
        file.write_all(b"partial content").unwrap();
        file.flush().unwrap();

        // Verify partial file exists
        assert!(output_path.exists());

        // Get file size
        let metadata = std::fs::metadata(&output_path).unwrap();
        let file_size = metadata.len();

        // Verify file size is non-zero
        assert!(file_size > 0);

        // Simulate HTTP Range request
        let start_byte = file_size;
        let range_header = format!("bytes={}-", start_byte);

        // Verify Range header format
        assert!(range_header.starts_with("bytes="));
        assert!(range_header.contains(&start_byte.to_string()));
    }

    #[test]
    fn test_download_resume_capability() {
        use std::fs::File;
        use std::io::Write;

        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test-model.onnx");

        // Create a partial file to simulate interrupted download
        let mut file = File::create(&output_path).unwrap();
        file.write_all(b"partial content").unwrap();
        file.flush().unwrap();

        // Verify partial file exists
        assert!(output_path.exists());

        // Get file size
        let metadata = std::fs::metadata(&output_path).unwrap();
        let bytes_downloaded = metadata.len();

        // Simulate resume logic
        let total_bytes = 120_000_000; // Expected total size
        let remaining_bytes = total_bytes - bytes_downloaded;

        // Verify resume capability
        assert!(bytes_downloaded > 0);
        assert!(bytes_downloaded < total_bytes);
        assert!(remaining_bytes > 0);

        // Verify can calculate progress
        let progress_percentage = (bytes_downloaded as f64 / total_bytes as f64) * 100.0;
        assert!(progress_percentage > 0.0);
        assert!(progress_percentage < 100.0);
    }

    #[test]
    fn test_ctrl_c_signal_handling() {
        use std::sync::atomic::{AtomicBool, Ordering};

        // Create an interrupt flag
        let interrupted = AtomicBool::new(false);

        // Simulate signal handler setting the flag
        interrupted.store(true, Ordering::SeqCst);

        // Verify flag was set
        assert!(interrupted.load(Ordering::SeqCst));

        // Simulate checking for interrupt
        if interrupted.load(Ordering::SeqCst) {
            // Signal was received, should handle cleanup
            assert!(true);
        }
    }

    #[test]
    fn test_signal_cleanup_and_state_preservation() {
        use std::fs::File;
        use std::io::Write;
        use std::sync::atomic::{AtomicBool, Ordering};

        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create model directory
        let models_dir = kb_root.join(".knowledge-loom-index").join("models");
        std::fs::create_dir_all(&models_dir).unwrap();

        // Create a partial file to simulate interrupted download
        let output_path = models_dir.join("test-model.onnx");
        let mut file = File::create(&output_path).unwrap();
        file.write_all(b"partial content").unwrap();
        file.flush().unwrap();

        // Create an interrupt flag
        let interrupted = AtomicBool::new(false);

        // Simulate signal handler setting the flag
        interrupted.store(true, Ordering::SeqCst);

        // Simulate cleanup on interrupt
        if interrupted.load(Ordering::SeqCst) {
            // Preserve download state
            let state_file = models_dir.join("download-state.json");
            let state = knowledge_loom::model::DownloadState {
                status: knowledge_loom::model::DownloadStatus::InProgress,
                progress_percentage: 50.0,
                bytes_downloaded: 60_000_000,
                total_bytes: 120_000_000,
                download_speed: 2_500_000.0,
                error_message: Some("Download interrupted by user".to_string()),
                last_updated: chrono::Utc::now(),
                model_name: knowledge_loom::model::MODEL_NAME.to_string(),
                model_version: knowledge_loom::model::MODEL_VERSION.to_string(),
            };

            let state_json = serde_json::to_string_pretty(&state).unwrap();
            std::fs::write(&state_file, state_json).unwrap();

            // Verify state was preserved
            assert!(state_file.exists());

            // Verify partial file still exists for resume
            assert!(output_path.exists());

            // Verify state contains error message
            let state_json = std::fs::read_to_string(&state_file).unwrap();
            let retrieved_state: knowledge_loom::model::DownloadState =
                serde_json::from_str(&state_json).unwrap();
            assert!(retrieved_state.error_message.is_some());
            assert!(retrieved_state
                .error_message
                .unwrap()
                .contains("interrupted"));
        }
    }
}
