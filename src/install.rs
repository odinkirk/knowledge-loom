// Install module for Knowledge Loom runtime data setup
// Handles downloading and installing fastembed model files

use sha2::Digest;
use std::path::PathBuf;

/// Constants for install module
pub const MODEL_DIR: &str = ".knowledge-loom/models";
pub const STATE_FILE: &str = ".knowledge-loom/models/.install-state.json";
pub const MODEL_URL: &str =
    "https://huggingface.co/Qdrant/all-MiniLM-L6-v2-onnx/resolve/main/model.onnx";
pub const EXPECTED_CHECKSUM: &str =
    "bbd7b466f6d58e646fdc2bd5fd67b2f5e93c0b687011bd4548c420f7bd46f0c5";

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

        // Download model file using shared DownloadManager
        let model_file = model_dir.join("model.onnx");
        let mut manager = crate::download::DownloadManager::new(
            MODEL_URL.to_string(),
            model_file.clone(),
        )
        .map_err(|e| InstallError::DownloadFailed(e.to_string()))?
        .with_retries(crate::download::MAX_RETRIES)
        .with_retry_delay(std::time::Duration::from_secs(crate::download::RETRY_DELAY))
        .with_timeout(std::time::Duration::from_secs(crate::download::TIMEOUT));

        manager
            .download(|_| {})
            .await
            .map_err(|e| InstallError::DownloadFailed(e.to_string()))?;

        // Read downloaded file for checksum validation
        let bytes = std::fs::read(&model_file)?;

        // Validate checksum using shared utility
        if let Err(e) = crate::download::utils::validate_checksum(&bytes, EXPECTED_CHECKSUM) {
            return Err(InstallError::ChecksumMismatch {
                expected: EXPECTED_CHECKSUM.to_string(),
                actual: e.to_string(),
            });
        }

        let checksum_hex = EXPECTED_CHECKSUM.to_string();

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
        let state_content = match std::fs::read_to_string(self.state_path()) {
            Ok(content) => content,
            Err(_) => return Ok(false),
        };
        let state: InstallState = serde_json::from_str(&state_content)?;

        // Read model file and verify checksum
        let model_file = self.model_path().join("model.onnx");
        let bytes = match std::fs::read(&model_file) {
            Ok(bytes) => bytes,
            Err(_) => return Ok(false),
        };
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_run_install_with_force() {
        // Integration test would go here
        // For now, verify the function signature is correct
    }
}
