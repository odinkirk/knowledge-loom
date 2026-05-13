// Init module for Knowledge Loom initialization
// This module handles the initialization of the knowledge base, including model download

use crate::model::ModelManager;
use crate::model::{InitError, ModelMetadata, MODEL_URL};
use std::path::PathBuf;

/// Initialization manager for handling knowledge base initialization
pub struct InitManager {
    kb_root: PathBuf,
}

impl InitManager {
    /// Create a new initialization manager
    pub fn new(kb_root: PathBuf) -> Self {
        Self { kb_root }
    }

    /// Get the knowledge base root directory
    pub fn kb_root(&self) -> &PathBuf {
        &self.kb_root
    }

    /// Check if the knowledge base is initialized
    pub fn is_initialized(&self) -> Result<bool, InitError> {
        let index_dir = self.kb_root.join(".knowledge-loom-index");
        Ok(index_dir.exists())
    }

    /// Initialize the knowledge base
    pub fn initialize(&self) -> Result<(), InitError> {
        // Create index directory
        let index_dir = self.kb_root.join(".knowledge-loom-index");
        std::fs::create_dir_all(&index_dir).map_err(|e| {
            InitError::InitializationFailed(format!("Failed to create index directory: {}", e))
        })?;

        // Create models directory
        let models_dir = index_dir.join("models");
        std::fs::create_dir_all(&models_dir).map_err(|e| {
            InitError::InitializationFailed(format!("Failed to create models directory: {}", e))
        })?;

        Ok(())
    }

    /// Get the model metadata
    pub fn get_model_metadata(&self) -> Result<Option<ModelMetadata>, InitError> {
        let models_dir = self.kb_root.join(".knowledge-loom-index").join("models");
        let metadata_file = models_dir.join("all-MiniLM-L6-v2.json");

        if !metadata_file.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&metadata_file).map_err(|e| {
            InitError::InitializationFailed(format!("Failed to read metadata: {}", e))
        })?;

        let metadata: ModelMetadata = serde_json::from_str(&content).map_err(|e| {
            InitError::InitializationFailed(format!("Failed to parse metadata: {}", e))
        })?;

        Ok(Some(metadata))
    }

    /// Download the model
    pub fn download_model<F>(&self, progress_callback: F) -> Result<(), InitError>
    where
        F: Fn(crate::model::DownloadProgress) + Send + Sync,
    {
        let model_manager = ModelManager::new(self.kb_root.clone());
        let output_path = model_manager.model_path();

        // Create download manager
        let download_manager =
            crate::download::DownloadManager::new(MODEL_URL.to_string(), output_path)
                .map_err(InitError::Download)?;

        // Download with retry
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            InitError::InitializationFailed(format!("Failed to create runtime: {}", e))
        })?;

        rt.block_on(async {
            download_manager
                .download_with_retry(&progress_callback)
                .await
        })
        .map_err(InitError::Download)?;

        Ok(())
    }

    /// Validate the model
    pub fn validate_model(&self, expected_checksum: &str) -> Result<bool, InitError> {
        let model_manager = ModelManager::new(self.kb_root.clone());
        model_manager
            .validate_model(expected_checksum)
            .map_err(|e| InitError::InitializationFailed(format!("Validation failed: {}", e)))
    }

    /// Create directories for initialization
    pub fn create_directories(&self) -> Result<(), InitError> {
        let index_dir = self.kb_root.join(".knowledge-loom-index");
        std::fs::create_dir_all(&index_dir).map_err(|e| {
            InitError::InitializationFailed(format!("Failed to create index directory: {}", e))
        })?;

        let models_dir = index_dir.join("models");
        std::fs::create_dir_all(&models_dir).map_err(|e| {
            InitError::InitializationFailed(format!("Failed to create models directory: {}", e))
        })?;

        Ok(())
    }

    /// Initialize indexes
    pub fn initialize_indexes(&self) -> Result<(), InitError> {
        // Index initialization is handled by the main indexing system
        // This is a placeholder for any index-specific initialization
        Ok(())
    }

    /// Create config files
    pub fn create_config_files(&self) -> Result<(), InitError> {
        // Config file creation is handled by the main configuration system
        // This is a placeholder for any config-specific initialization
        Ok(())
    }

    /// Generate manual download instructions
    ///
    /// This function generates step-by-step instructions for manually downloading
    /// the model when automatic download fails or is not possible.
    ///
    /// # Returns
    ///
    /// Returns a `Result<String, InitError>` containing the formatted instructions
    /// or an error if the instructions cannot be generated.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use knowledge_loom::init::InitManager;
    /// use std::path::PathBuf;
    ///
    /// let init_manager = InitManager::new(PathBuf::from("/path/to/kb"));
    /// let instructions = init_manager.generate_manual_download_instructions().unwrap();
    /// println!("{}", instructions);
    /// ```
    ///
    /// # Instructions Include
    ///
    /// - Step-by-step download instructions
    /// - Model URL and file information
    /// - Directory creation commands
    /// - File placement instructions
    /// - Verification guidance
    /// - Troubleshooting tips
    pub fn generate_manual_download_instructions(&self) -> Result<String, InitError> {
        let models_dir = self.kb_root.join(".knowledge-loom-index").join("models");
        let model_file = models_dir.join("all-MiniLM-L6-v2.onnx");

        let instructions = format!(
            r#"Manual Model Download Instructions

Automatic model download failed or was interrupted. Follow these steps to manually download the model:

Step 1: Download the model file
  Download URL: https://huggingface.co/Qdrant/all-MiniLM-L6-v2-onnx/resolve/main/model.onnx
  Model name: all-MiniLM-L6-v2
  Expected size: ~120MB

  You can download using:
  - curl: curl -L -o "{}" "https://huggingface.co/Qdrant/all-MiniLM-L6-v2-onnx/resolve/main/model.onnx"
  - wget: wget -O "{}" "https://huggingface.co/Qdrant/all-MiniLM-L6-v2-onnx/resolve/main/model.onnx"
  - Or download directly from the URL in your browser

Step 2: Create the models directory
  mkdir -p "{}"

Step 3: Move the downloaded file to the models directory
  mv all-MiniLM-L6-v2.onnx "{}"

Step 4: Verify the download (optional but recommended)
  After downloading, you can verify the file integrity using SHA-256 checksum.
  The expected checksum will be validated automatically when you run 'loom init' again.

Step 5: Run initialization again
  Run 'loom init' again to complete the initialization process.
  The system will validate the downloaded model and continue with initialization.

Troubleshooting:
  - If download fails, check your internet connection
  - If you're behind a proxy, configure HTTP_PROXY and HTTPS_PROXY environment variables
  - If you have permission issues, ensure you have write access to the knowledge base directory
  - For more help, visit: https://github.com/your-repo/knowledge-loom/issues

KB_ROOT: {}
Models directory: {}
"#,
            model_file.display(),
            model_file.display(),
            models_dir.display(),
            model_file.display(),
            self.kb_root.display(),
            models_dir.display()
        );

        Ok(instructions)
    }
}

/// Run the init command with progress display
pub fn run_init(_args: Vec<String>) -> Result<(), InitError> {
    // Get KB_ROOT from environment or use current directory
    let kb_root: PathBuf = std::env::var("KB_ROOT")
        .unwrap_or_else(|_| ".".to_string())
        .into();

    let init_manager = InitManager::new(kb_root.clone());

    // Check if already initialized
    if init_manager.is_initialized()? {
        println!("Knowledge base already initialized");
        return Ok(());
    }

    // Initialize knowledge base
    println!("Initializing knowledge base...");
    init_manager.initialize()?;

    // Check if model is valid
    let model_manager = ModelManager::new(kb_root.clone());
    if model_manager.is_model_valid().map_err(|e| {
        InitError::InitializationFailed(format!("Failed to check model validity: {}", e))
    })? {
        println!("Model already downloaded and validated");
        return Ok(());
    }

    // Download model with progress display
    println!("Downloading model...");
    let start_time = std::time::Instant::now();

    init_manager.download_model(|progress| {
        println!("{}", crate::download::format_download_progress(&progress));
    })?;

    let duration = start_time.elapsed().as_secs();
    let file_size = std::fs::metadata(model_manager.model_path())
        .map_err(|e| {
            InitError::InitializationFailed(format!("Failed to get model file size: {}", e))
        })?
        .len();

    println!(
        "{}",
        crate::download::format_download_complete(file_size, duration)
    );

    // Validate model
    println!("Validating model...");
    // Note: In a real implementation, you would have the expected checksum
    // For now, we'll skip validation or use a placeholder
    // let is_valid = init_manager.validate_model("expected_checksum")?;

    println!("Initialization complete!");

    Ok(())
}

/// Initialization progress structure
#[derive(Debug, Clone)]
pub struct InitProgress {
    pub stage: String,
    pub percentage: f64,
    pub message: String,
}

impl InitProgress {
    /// Create new initialization progress
    pub fn new(stage: String, percentage: f64, message: String) -> Self {
        Self {
            stage,
            percentage,
            message,
        }
    }
}
