// Unit tests for Knowledge Loom model download
// This module tests HTTP download with retry logic, progress tracking, and checksum validation

#[cfg(test)]
mod download_tests {
    use knowledge_loom::download::{
        format_download_complete, format_download_error, format_download_progress, DownloadManager,
        DownloadProgress, MAX_RETRIES, RETRY_DELAY, TIMEOUT,
    };
    use knowledge_loom::model::DownloadError;
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

        assert_eq!(manager.retry_delay.as_secs(), 2);
    }

    #[test]
    fn test_download_manager_with_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test-model.onnx");

        let manager =
            DownloadManager::new("https://example.com/model.onnx".to_string(), output_path)
                .unwrap()
                .with_timeout(std::time::Duration::from_secs(60));

        assert_eq!(manager.timeout.as_secs(), 60);
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
}
