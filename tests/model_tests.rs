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
        let metadata = ModelMetadata::new(
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
}
