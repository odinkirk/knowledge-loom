// Install module for Knowledge Loom runtime data setup
// Handles downloading and installing fastembed model files

#![allow(dead_code)]

use sha2::Digest;
use std::io::Read;
use std::path::PathBuf;

/// Constants for install module
pub const MODEL_DIR: &str = ".knowledge-loom/models";
pub const STATE_FILE: &str = ".knowledge-loom/models/.install-state.json";
pub const MODEL_URL: &str =
    "https://huggingface.co/Xenova/bge-small-en-v1.5/resolve/main/onnx/model.onnx";
pub const EXPECTED_CHECKSUM: &str =
    "828e1496d7fabb79cfa4dcd84fa38625c0d3d21da474a00f08db0f559940cf35";

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

    /// Check if model is already installed (checks for model.onnx file)
    pub fn is_installed(&self) -> bool {
        self.model_path().join("model.onnx").exists()
    }

    /// Download the model file
    pub async fn download_model(&self, _force: bool) -> Result<InstallSummary> {
        // Create model directory
        let model_dir = self.model_path();
        std::fs::create_dir_all(&model_dir)?;

        // Remove existing file to ensure clean download (avoids resume from corrupted file)
        let model_file = model_dir.join("model.onnx");
        if model_file.exists() {
            std::fs::remove_file(&model_file)?;
        }

        // Download model file using shared DownloadManager
        let manager =
            crate::download::DownloadManager::new(MODEL_URL.to_string(), model_file.clone())
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
        let actual_checksum = crate::download::utils::calculate_checksum(&bytes);
        if actual_checksum != EXPECTED_CHECKSUM {
            return Err(InstallError::ChecksumMismatch {
                expected: EXPECTED_CHECKSUM.to_string(),
                actual: actual_checksum,
            });
        }

        let checksum_hex = EXPECTED_CHECKSUM.to_string();

        // Save state
        let state = InstallState {
            model_version: "bge-small-en-v1.5".to_string(),
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

    /// Verify model integrity using streaming checksum calculation
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

        // Read model file and verify checksum using streaming
        let model_file = self.model_path().join("model.onnx");
        let file = match std::fs::File::open(&model_file) {
            Ok(file) => file,
            Err(_) => return Ok(false),
        };

        // Stream checksum calculation (8KB chunks)
        let mut hasher = sha2::Sha256::new();
        let mut reader = std::io::BufReader::new(file);
        let mut buffer = [0u8; 8192];
        loop {
            let n = match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => return Ok(false),
            };
            hasher.update(&buffer[..n]);
        }
        let checksum_hex = format!("{:x}", hasher.finalize());

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
    fn test_install_manager_paths() {
        let kb_root = PathBuf::from("/tmp/test-kb");
        let manager = InstallManager::new(kb_root.clone());
        assert_eq!(manager.kb_root(), &kb_root);
        assert_eq!(manager.model_path(), kb_root.join(MODEL_DIR));
        assert_eq!(manager.state_path(), kb_root.join(STATE_FILE));
        assert!(
            !manager.is_installed(),
            "Fresh manager should report not installed"
        );
    }
}
