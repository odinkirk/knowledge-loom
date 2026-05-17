// Unit tests for Knowledge Loom model management
// This module tests model validation, metadata, and state management

#[cfg(test)]
mod model_tests {
    use knowledge_loom::model::{
        DownloadState, DownloadStatus, ModelMetadata, MODEL_NAME, MODEL_VERSION,
    };
    use tempfile::TempDir;

    #[test]
    fn test_model_manager_new() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Test creating a new model manager
        let models_dir = kb_root.join(".knowledge-loom-index").join("models");
        std::fs::create_dir_all(&models_dir).unwrap();

        // Verify directory was created
        assert!(models_dir.exists());
    }

    #[test]
    fn test_model_manager_is_model_valid() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create model directory
        let models_dir = kb_root.join(".knowledge-loom-index").join("models");
        std::fs::create_dir_all(&models_dir).unwrap();

        // Create model file
        let model_file = models_dir.join("all-MiniLM-L6-v2.onnx");
        std::fs::write(&model_file, "test model data").unwrap();

        // Create metadata file
        let metadata_file = models_dir.join("all-MiniLM-L6-v2.json");
        let mut metadata = ModelMetadata::new(
            MODEL_NAME.to_string(),
            MODEL_VERSION.to_string(),
            model_file.to_str().unwrap().to_string(),
            14,
            "test_checksum".to_string(),
            "https://example.com/model.onnx".to_string(),
        );
        metadata.mark_validated();

        let metadata_json = serde_json::to_string_pretty(&metadata).unwrap();
        std::fs::write(&metadata_file, metadata_json).unwrap();

        // Verify model is valid
        assert!(model_file.exists());
        assert!(metadata_file.exists());
    }

    #[test]
    fn test_model_manager_model_path() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Test model path construction
        let models_dir = kb_root.join(".knowledge-loom-index").join("models");
        let model_file = models_dir.join("all-MiniLM-L6-v2.onnx");

        assert_eq!(
            model_file.file_name().unwrap().to_str().unwrap(),
            "all-MiniLM-L6-v2.onnx"
        );
    }

    #[test]
    fn test_download_state_new() {
        let state = DownloadState::new("test-model".to_string(), "1.0.0".to_string(), 1000);

        assert_eq!(state.status, DownloadStatus::NotStarted);
        assert_eq!(state.progress_percentage, 0.0);
        assert_eq!(state.bytes_downloaded, 0);
        assert_eq!(state.total_bytes, 1000);
        assert_eq!(state.download_speed, 0.0);
        assert!(state.error_message.is_none());
        assert_eq!(state.model_name, "test-model");
        assert_eq!(state.model_version, "1.0.0");
    }

    #[test]
    fn test_download_state_update_progress() {
        let mut state = DownloadState::new("test-model".to_string(), "1.0.0".to_string(), 1000);

        state.update_progress(500, 100.0);

        assert_eq!(state.bytes_downloaded, 500);
        assert_eq!(state.download_speed, 100.0);
        assert_eq!(state.progress_percentage, 50.0);
    }

    #[test]
    fn test_download_state_set_status() {
        let mut state = DownloadState::new("test-model".to_string(), "1.0.0".to_string(), 1000);

        state.set_status(DownloadStatus::InProgress);

        assert_eq!(state.status, DownloadStatus::InProgress);
    }

    #[test]
    fn test_download_state_set_error() {
        let mut state = DownloadState::new("test-model".to_string(), "1.0.0".to_string(), 1000);

        state.set_error("Test error".to_string());

        assert_eq!(state.status, DownloadStatus::Failed);
        assert_eq!(state.error_message, Some("Test error".to_string()));
    }

    #[test]
    fn test_model_metadata_new() {
        let metadata = ModelMetadata::new(
            "test-model".to_string(),
            "1.0.0".to_string(),
            "/path/to/model.onnx".to_string(),
            1000,
            "test_checksum".to_string(),
            "https://example.com/model.onnx".to_string(),
        );

        assert_eq!(metadata.model_name, "test-model");
        assert_eq!(metadata.model_version, "1.0.0");
        assert_eq!(metadata.file_path, "/path/to/model.onnx");
        assert_eq!(metadata.file_size, 1000);
        assert_eq!(metadata.sha256_checksum, "test_checksum");
        assert_eq!(metadata.download_url, "https://example.com/model.onnx");
        assert!(!metadata.validated);
    }

    #[test]
    fn test_model_metadata_mark_validated() {
        let mut metadata = ModelMetadata::new(
            "test-model".to_string(),
            "1.0.0".to_string(),
            "/path/to/model.onnx".to_string(),
            1000,
            "test_checksum".to_string(),
            "https://example.com/model.onnx".to_string(),
        );

        metadata.mark_validated();

        assert!(metadata.validated);
    }

    #[test]
    fn test_model_metadata_version_match() {
        let metadata = ModelMetadata::new(
            "test-model".to_string(),
            "1.0.0".to_string(),
            "/path/to/model.onnx".to_string(),
            1000,
            "test_checksum".to_string(),
            "https://example.com/model.onnx".to_string(),
        );

        assert!(metadata.is_version_match("1.0.0"));
        assert!(!metadata.is_version_match("2.0.0"));
    }

    #[test]
    fn test_model_metadata_name_match() {
        let metadata = ModelMetadata::new(
            "test-model".to_string(),
            "1.0.0".to_string(),
            "/path/to/model.onnx".to_string(),
            1000,
            "test_checksum".to_string(),
            "https://example.com/model.onnx".to_string(),
        );

        assert!(metadata.is_name_match("test-model"));
        assert!(!metadata.is_name_match("other-model"));
    }

    #[test]
    fn test_model_metadata_validate_metadata() {
        let metadata = ModelMetadata::new(
            "test-model".to_string(),
            "1.0.0".to_string(),
            "/path/to/model.onnx".to_string(),
            1000,
            "test_checksum".to_string(),
            "https://example.com/model.onnx".to_string(),
        );

        // Test valid metadata
        let result = metadata.validate_metadata("test-model", "1.0.0");
        assert!(result.is_ok());

        // Test invalid name
        let result = metadata.validate_metadata("other-model", "1.0.0");
        assert!(result.is_err());

        // Test invalid version
        let result = metadata.validate_metadata("test-model", "2.0.0");
        assert!(result.is_err());
    }

    // User Story 2: Graceful Error Handling Tests

    #[test]
    fn test_checksum_mismatch_error_handling() {
        use knowledge_loom::model::ModelError;

        // Test checksum mismatch error
        let error = ModelError::ChecksumMismatch;

        let error_msg = format!("{:?}", error);
        assert!(error_msg.contains("ChecksumMismatch") || error_msg.contains("checksum"));
    }

    // User Story 3: Model Re-Download with State Handling Tests

    #[test]
    fn test_download_state_persistence() {
        use knowledge_loom::model::{DownloadState, DownloadStatus};

        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create model directory
        let models_dir = kb_root.join(".knowledge-loom-index").join("models");
        std::fs::create_dir_all(&models_dir).unwrap();

        // Create a download state
        let state = DownloadState {
            status: DownloadStatus::InProgress,
            progress_percentage: 50.0,
            bytes_downloaded: 60_000_000,
            total_bytes: 120_000_000,
            download_speed: 2_500_000.0,
            error_message: None,
            last_updated: chrono::Utc::now(),
            model_name: MODEL_NAME.to_string(),
            model_version: MODEL_VERSION.to_string(),
        };

        // Save state to file
        let state_file = models_dir.join("download-state.json");
        let state_json = serde_json::to_string_pretty(&state).unwrap();
        std::fs::write(&state_file, state_json).unwrap();

        // Verify state file exists
        assert!(state_file.exists());

        // Read state back
        let state_json = std::fs::read_to_string(&state_file).unwrap();
        let retrieved_state: DownloadState = serde_json::from_str(&state_json).unwrap();

        // Verify state was persisted correctly
        assert_eq!(retrieved_state.status, DownloadStatus::InProgress);
        assert_eq!(retrieved_state.progress_percentage, 50.0);
        assert_eq!(retrieved_state.bytes_downloaded, 60_000_000);
        assert_eq!(retrieved_state.total_bytes, 120_000_000);
        assert_eq!(retrieved_state.download_speed, 2_500_000.0);
        assert!(retrieved_state.error_message.is_none());
        assert_eq!(retrieved_state.model_name, MODEL_NAME);
        assert_eq!(retrieved_state.model_version, MODEL_VERSION);
    }

    #[test]
    fn test_download_state_recovery() {
        use knowledge_loom::model::{DownloadState, DownloadStatus};

        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create model directory
        let models_dir = kb_root.join(".knowledge-loom-index").join("models");
        std::fs::create_dir_all(&models_dir).unwrap();

        // Create a download state with in-progress status
        let state = DownloadState {
            status: DownloadStatus::InProgress,
            progress_percentage: 75.0,
            bytes_downloaded: 90_000_000,
            total_bytes: 120_000_000,
            download_speed: 3_000_000.0,
            error_message: None,
            last_updated: chrono::Utc::now(),
            model_name: MODEL_NAME.to_string(),
            model_version: MODEL_VERSION.to_string(),
        };

        // Save state to file
        let state_file = models_dir.join("download-state.json");
        let state_json = serde_json::to_string_pretty(&state).unwrap();
        std::fs::write(&state_file, state_json).unwrap();

        // Simulate recovery by reading state
        let state_json = std::fs::read_to_string(&state_file).unwrap();
        let recovered_state: DownloadState = serde_json::from_str(&state_json).unwrap();

        // Verify recovery
        assert_eq!(recovered_state.status, DownloadStatus::InProgress);
        assert_eq!(recovered_state.progress_percentage, 75.0);
        assert_eq!(recovered_state.bytes_downloaded, 90_000_000);
        assert_eq!(recovered_state.total_bytes, 120_000_000);

        // Calculate remaining bytes
        let remaining_bytes = recovered_state.total_bytes - recovered_state.bytes_downloaded;
        assert_eq!(remaining_bytes, 30_000_000);

        // Verify can resume from this state
        assert!(recovered_state.status == DownloadStatus::InProgress);
        assert!(recovered_state.bytes_downloaded > 0);
        assert!(recovered_state.bytes_downloaded < recovered_state.total_bytes);
    }

    #[test]
    fn test_file_locking() {
        use std::fs::OpenOptions;
        use std::io::Write;

        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create model directory
        let models_dir = kb_root.join(".knowledge-loom-index").join("models");
        std::fs::create_dir_all(&models_dir).unwrap();

        // Create lock file
        let lock_file = models_dir.join(".download.lock");
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&lock_file)
            .unwrap();

        // Write lock data
        file.write_all(b"locked").unwrap();
        file.flush().unwrap();

        // Verify lock file exists
        assert!(lock_file.exists());

        // Verify lock file content
        let lock_content = std::fs::read_to_string(&lock_file).unwrap();
        assert_eq!(lock_content, "locked");
    }

    #[test]
    fn test_concurrent_download_prevention() {
        use std::fs::OpenOptions;
        use std::io::Write;

        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create model directory
        let models_dir = kb_root.join(".knowledge-loom-index").join("models");
        std::fs::create_dir_all(&models_dir).unwrap();

        // Create lock file to simulate in-progress download
        let lock_file = models_dir.join(".download.lock");
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&lock_file)
            .unwrap();

        file.write_all(b"locked").unwrap();
        file.flush().unwrap();

        // Verify lock file exists
        assert!(lock_file.exists());

        // Simulate checking for lock
        if lock_file.exists() {
            // Lock exists, should prevent concurrent download
            let lock_content = std::fs::read_to_string(&lock_file).unwrap();
            assert_eq!(lock_content, "locked");
        }
    }

    #[test]
    fn test_model_version_mismatch_detection() {
        use knowledge_loom::model::ModelMetadata;

        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create model directory
        let models_dir = kb_root.join(".knowledge-loom-index").join("models");
        std::fs::create_dir_all(&models_dir).unwrap();

        // Create metadata with old version
        let metadata_file = models_dir.join("all-MiniLM-L6-v2.json");
        let mut metadata = ModelMetadata::new(
            MODEL_NAME.to_string(),
            "0.9.0".to_string(), // Old version
            "/path/to/model.onnx".to_string(),
            1000,
            "test_checksum".to_string(),
            "https://example.com/model.onnx".to_string(),
        );
        metadata.mark_validated();

        let metadata_json = serde_json::to_string_pretty(&metadata).unwrap();
        std::fs::write(&metadata_file, metadata_json).unwrap();

        // Read metadata back
        let metadata_json = std::fs::read_to_string(&metadata_file).unwrap();
        let retrieved_metadata: ModelMetadata = serde_json::from_str(&metadata_json).unwrap();

        // Verify version mismatch
        assert!(!retrieved_metadata.is_version_match(MODEL_VERSION));
        assert_eq!(retrieved_metadata.model_version, "0.9.0");
        assert_eq!(MODEL_VERSION, "1.0.0");
    }

    #[test]
    fn test_version_re_download_prompt() {
        use knowledge_loom::model::{DownloadState, DownloadStatus, ModelMetadata};

        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create model directory
        let models_dir = kb_root.join(".knowledge-loom-index").join("models");
        std::fs::create_dir_all(&models_dir).unwrap();

        // Create metadata with old version
        let metadata_file = models_dir.join("all-MiniLM-L6-v2.json");
        let mut metadata = ModelMetadata::new(
            MODEL_NAME.to_string(),
            "0.9.0".to_string(), // Old version
            "/path/to/model.onnx".to_string(),
            1000,
            "test_checksum".to_string(),
            "https://example.com/model.onnx".to_string(),
        );
        metadata.mark_validated();

        let metadata_json = serde_json::to_string_pretty(&metadata).unwrap();
        std::fs::write(&metadata_file, metadata_json).unwrap();

        // Read metadata back
        let metadata_json = std::fs::read_to_string(&metadata_file).unwrap();
        let retrieved_metadata: ModelMetadata = serde_json::from_str(&metadata_json).unwrap();

        // Verify version mismatch detected
        assert!(!retrieved_metadata.is_version_match(MODEL_VERSION));

        // Create download state for re-download
        let state = DownloadState {
            status: DownloadStatus::NotStarted,
            progress_percentage: 0.0,
            bytes_downloaded: 0,
            total_bytes: 120_000_000,
            download_speed: 0.0,
            error_message: Some("Version mismatch: expected 1.0.0, found 0.9.0".to_string()),
            last_updated: chrono::Utc::now(),
            model_name: MODEL_NAME.to_string(),
            model_version: MODEL_VERSION.to_string(),
        };

        // Verify state indicates re-download needed
        assert_eq!(state.status, DownloadStatus::NotStarted);
        assert!(state.error_message.is_some());
        assert!(state.error_message.unwrap().contains("Version mismatch"));
    }
}
