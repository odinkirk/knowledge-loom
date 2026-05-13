// Integration tests for Knowledge Loom initialization
// This module tests the init functionality including model download

#[cfg(test)]
mod init_tests {
    use knowledge_loom::init::InitManager;
    use tempfile::TempDir;

    #[test]
    fn test_init_manager_new() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        let init_manager = InitManager::new(kb_root.to_path_buf());

        assert_eq!(init_manager.kb_root(), kb_root);
    }

    #[test]
    fn test_init_manager_is_initialized_false() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        let init_manager = InitManager::new(kb_root.to_path_buf());

        let is_initialized = init_manager.is_initialized();

        assert!(is_initialized.is_ok());
        assert!(!is_initialized.unwrap());
    }

    #[test]
    fn test_init_manager_is_initialized_true() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create index directory
        let index_dir = kb_root.join(".knowledge-loom-index");
        std::fs::create_dir_all(&index_dir).unwrap();

        let init_manager = InitManager::new(kb_root.to_path_buf());

        let is_initialized = init_manager.is_initialized();

        assert!(is_initialized.is_ok());
        assert!(is_initialized.unwrap());
    }

    #[test]
    fn test_init_manager_initialize() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        let init_manager = InitManager::new(kb_root.to_path_buf());

        let result = init_manager.initialize();

        assert!(result.is_ok());

        // Verify directories were created
        let index_dir = kb_root.join(".knowledge-loom-index");
        assert!(index_dir.exists());

        let models_dir = index_dir.join("models");
        assert!(models_dir.exists());
    }

    #[test]
    fn test_init_manager_get_model_metadata_none() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        let init_manager = InitManager::new(kb_root.to_path_buf());

        let metadata = init_manager.get_model_metadata();

        assert!(metadata.is_ok());
        assert!(metadata.unwrap().is_none());
    }

    #[test]
    fn test_init_manager_get_model_metadata_some() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create model directory
        let models_dir = kb_root.join(".knowledge-loom-index").join("models");
        std::fs::create_dir_all(&models_dir).unwrap();

        // Create metadata file
        let metadata_file = models_dir.join("all-MiniLM-L6-v2.json");
        let metadata = knowledge_loom::model::ModelMetadata::new(
            "all-MiniLM-L6-v2".to_string(),
            "1.0.0".to_string(),
            "/path/to/model.onnx".to_string(),
            1000,
            "test_checksum".to_string(),
            "https://example.com/model.onnx".to_string(),
        );

        let metadata_json = serde_json::to_string_pretty(&metadata).unwrap();
        std::fs::write(&metadata_file, metadata_json).unwrap();

        let init_manager = InitManager::new(kb_root.to_path_buf());

        let result = init_manager.get_model_metadata();

        assert!(result.is_ok());
        let metadata = result.unwrap();
        assert!(metadata.is_some());
        let metadata = metadata.unwrap();
        assert_eq!(metadata.model_name, "all-MiniLM-L6-v2");
    }
}
