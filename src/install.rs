// Install module for Knowledge Loom runtime data setup
// Handles downloading and installing fastembed model files

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Constants for install module
pub const MODEL_DIR: &str = ".knowledge-loom/models";
pub const STATE_FILE: &str = ".knowledge-loom/models/.install-state.json";
pub const MODEL_URL: &str = "https://huggingface.co/Qdrant/all-MiniLM-L6-v2-onnx/resolve/main/model.onnx";

/// Install error types
#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("Model already installed. Use --force to re-download.")]
    AlreadyInstalled,

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Disk full: required {required} MB, available {available} MB")]
    DiskFull { required: u64, available: u64 },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Install result type
pub type Result<T> = std::result::Result<T, InstallError>;

/// Install manager for runtime data setup
pub struct InstallManager {
    kb_root: PathBuf,
}

impl InstallManager {
    /// Create a new install manager
    pub fn new(kb_root: PathBuf) -> Self {
        Self { kb_root }
    }

    /// Get the knowledge base root directory
    pub fn kb_root(&self) -> &PathBuf {
        &self.kb_root
    }

    /// Get the model directory path
    pub fn model_path(&self) -> PathBuf {
        self.kb_root.join(MODEL_DIR)
    }

    /// Get the state file path
    pub fn state_path(&self) -> PathBuf {
        self.kb_root.join(STATE_FILE)
    }

    /// Check if model is already installed
    pub fn is_installed(&self) -> bool {
        self.model_path().exists()
    }

    /// Download the model file
    pub async fn download_model(&self, force: bool) -> Result<InstallSummary> {
        // Check if already installed
        if self.is_installed() && !force {
            return Err(InstallError::AlreadyInstalled);
        }

        // Create model directory
        let model_dir = self.model_path();
        std::fs::create_dir_all(&model_dir)?;

        // Download model file
        let model_file = model_dir.join("model.onnx");
        let client = reqwest::Client::new();
        let response = client
            .get(MODEL_URL)
            .send()
            .await
            .map_err(|e| InstallError::NetworkError(e.to_string()))?;

        let total_size = response
            .content_length()
            .ok_or_else(|| InstallError::DownloadFailed("Unknown content length".to_string()))?;

        let bytes = response
            .bytes()
            .await
            .map_err(|e| InstallError::DownloadFailed(e.to_string()))?;

        // Write model file
        std::fs::write(&model_file, &bytes)?;

        // Calculate checksum
        let checksum = sha2::Sha256::digest(&bytes);
        let checksum_hex = format!("{:x}", checksum);

        // Save state
        let state = InstallState {
            model_version: "all-MiniLM-L6-v2".to_string(),
            download_timestamp: chrono::Utc::now().to_rfc3339(),
            checksum: checksum_hex.clone(),
            size_bytes: bytes.len() as u64,
        };

        let state_json = serde_json::to_string_pretty(&state)?;
        std::fs::write(self.state_path(), state_json)?;

        Ok(InstallSummary {
            model_version: state.model_version,
            size_bytes: state.size_bytes,
            target_location: model_file.display().to_string(),
            checksum: checksum_hex,
        })
    }

    /// Verify model integrity
    pub fn verify_integrity(&self) -> Result<bool> {
        if !self.is_installed() {
            return Ok(false);
        }

        // Load state
        let state_content = std::fs::read_to_string(self.state_path())?;
        let state: InstallState = serde_json::from_str(&state_content)?;

        // Read model file and verify checksum
        let model_file = self.model_path().join("model.onnx");
        let bytes = std::fs::read(&model_file)?;
        let checksum = sha2::Sha256::digest(&bytes);
        let checksum_hex = format!("{:x}", checksum);

        if checksum_hex != state.checksum {
            return Ok(false);
        }

        Ok(true)
    }

    /// Validate or download model
    pub async fn validate_or_download(&self, force: bool) -> Result<InstallSummary> {
        if self.verify_integrity()? && !force {
            // Model is valid, skip download
            return Err(InstallError::AlreadyInstalled);
        }

        // Download model
        self.download_model(force).await
    }
}

/// Install state persisted to disk
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct InstallState {
    pub model_version: String,
    pub download_timestamp: String,
    pub checksum: String,
    pub size_bytes: u64,
}

/// Install summary for user output
#[derive(Debug)]
pub struct InstallSummary {
    pub model_version: String,
    pub size_bytes: u64,
    pub target_location: String,
    pub checksum: String,
}

/// Run the install command
pub async fn run_install(kb_root: PathBuf, force: bool) -> Result<InstallSummary> {
    let manager = InstallManager::new(kb_root);
    manager.validate_or_download(force).await
}
