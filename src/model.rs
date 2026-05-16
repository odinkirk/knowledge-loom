// Model module for Knowledge Loom model management
// This module handles model validation, metadata, and state management

#![allow(dead_code)]

use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Model name constant
pub const MODEL_NAME: &str = "all-MiniLM-L6-v2";

/// Model version constant
pub const MODEL_VERSION: &str = "1.0.0";

/// Model download URL constant
pub const MODEL_URL: &str =
    "https://huggingface.co/qdrant/all-MiniLM-L6-v2-onnx/resolve/main/model.onnx";

/// Model file name constant
pub const MODEL_FILE: &str = "all-MiniLM-L6-v2.onnx";

/// Download state file name constant
pub const STATE_FILE: &str = "download-state.json";

/// Lock file name constant
pub const LOCK_FILE: &str = "download.lock";

/// Error types for model management operations
#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Invalid download state: {details}")]
    InvalidState { details: String },

    #[error("Model validation failed: Checksum mismatch")]
    ChecksumMismatch,

    #[error("Model metadata not found: {path}")]
    MetadataNotFound { path: String },

    #[error("Model download is already in progress")]
    LockTimeout,

    #[error("Permission denied: Cannot write to {path}")]
    PermissionDenied { path: String },

    #[error("Insufficient disk space: {required_mb}MB required, {available_mb}MB available")]
    InsufficientDiskSpace { required_mb: u64, available_mb: u64 },

    #[error("Path not found: {path}")]
    PathNotFound { path: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Error types for download operations
#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Download interrupted by user")]
    Interrupted,

    #[error("Download failed after {retries} retries")]
    MaxRetriesExceeded { retries: u32 },

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },
}

/// Error types for initialization operations
#[derive(Error, Debug)]
pub enum InitError {
    #[error("Model error: {0}")]
    Model(#[from] ModelError),

    #[error("Download error: {0}")]
    Download(#[from] DownloadError),

    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
}

/// Download status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
}

/// Download state structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadState {
    pub status: DownloadStatus,
    pub progress_percentage: f64,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub download_speed: f64,
    pub error_message: Option<String>,
    pub last_updated: DateTime<Utc>,
    pub model_name: String,
    pub model_version: String,
}

impl DownloadState {
    /// Create a new download state
    pub fn new(model_name: String, model_version: String, total_bytes: u64) -> Self {
        Self {
            status: DownloadStatus::NotStarted,
            progress_percentage: 0.0,
            bytes_downloaded: 0,
            total_bytes,
            download_speed: 0.0,
            error_message: None,
            last_updated: Utc::now(),
            model_name,
            model_version,
        }
    }

    /// Update download progress
    pub fn update_progress(&mut self, bytes_downloaded: u64, download_speed: f64) {
        self.bytes_downloaded = bytes_downloaded;
        self.download_speed = download_speed;
        self.progress_percentage = if self.total_bytes > 0 {
            (bytes_downloaded as f64 / self.total_bytes as f64) * 100.0
        } else {
            0.0
        };
        self.last_updated = Utc::now();
    }

    /// Set download status
    pub fn set_status(&mut self, status: DownloadStatus) {
        self.status = status;
        self.last_updated = Utc::now();
    }

    /// Set download error
    pub fn set_error(&mut self, error_message: String) {
        self.status = DownloadStatus::Failed;
        self.error_message = Some(error_message);
        self.last_updated = Utc::now();
    }
}

/// Model metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub model_name: String,
    pub model_version: String,
    pub file_path: String,
    pub file_size: u64,
    pub sha256_checksum: String,
    pub download_timestamp: DateTime<Utc>,
    pub download_url: String,
    pub validated: bool,
}

impl ModelMetadata {
    /// Create new model metadata
    pub fn new(
        model_name: String,
        model_version: String,
        file_path: String,
        file_size: u64,
        sha256_checksum: String,
        download_url: String,
    ) -> Self {
        Self {
            model_name,
            model_version,
            file_path,
            file_size,
            sha256_checksum,
            download_timestamp: Utc::now(),
            download_url,
            validated: false,
        }
    }

    /// Mark model as validated
    pub fn mark_validated(&mut self) {
        self.validated = true;
    }

    /// Check if model version matches expected version
    ///
    /// This function compares the model version with the expected version.
    ///
    /// # Arguments
    ///
    /// * `expected_version` - The expected model version
    ///
    /// # Returns
    ///
    /// * `true` - If the model version matches the expected version
    /// * `false` - If the model version doesn't match
    pub fn is_version_match(&self, expected_version: &str) -> bool {
        self.model_version == expected_version
    }

    /// Check if model name matches expected name
    ///
    /// This function compares the model name with the expected name.
    ///
    /// # Arguments
    ///
    /// * `expected_name` - The expected model name
    ///
    /// # Returns
    ///
    /// * `true` - If the model name matches the expected name
    /// * `false` - If the model name doesn't match
    pub fn is_name_match(&self, expected_name: &str) -> bool {
        self.model_name == expected_name
    }

    /// Validate model metadata against expected values
    ///
    /// This function validates the model metadata against the expected name and version.
    /// It returns an error if the name or version doesn't match.
    ///
    /// # Arguments
    ///
    /// * `expected_name` - The expected model name
    /// * `expected_version` - The expected model version
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the metadata matches the expected values
    /// * `Err(ModelError)` - If the name or version doesn't match
    pub fn validate_metadata(
        &self,
        expected_name: &str,
        expected_version: &str,
    ) -> Result<(), ModelError> {
        if !self.is_name_match(expected_name) {
            return Err(ModelError::InvalidState {
                details: format!(
                    "Model name mismatch: expected {}, found {}",
                    expected_name, self.model_name
                ),
            });
        }

        if !self.is_version_match(expected_version) {
            return Err(ModelError::InvalidState {
                details: format!(
                    "Model version mismatch: expected {}, found {}",
                    expected_version, self.model_version
                ),
            });
        }

        Ok(())
    }
}

/// Model manager for handling model validation, metadata, and state management
pub struct ModelManager {
    kb_root: PathBuf,
}

impl ModelManager {
    /// Create a new model manager
    pub fn new(kb_root: PathBuf) -> Self {
        Self { kb_root }
    }

    /// Get the path to the model file
    pub fn model_path(&self) -> PathBuf {
        self.kb_root
            .join(".knowledge-loom-index")
            .join("models")
            .join(MODEL_FILE)
    }

    /// Get the path to the model metadata file
    pub fn metadata_path(&self) -> PathBuf {
        self.kb_root
            .join(".knowledge-loom-index")
            .join("models")
            .join(format!("{}.json", MODEL_NAME))
    }

    /// Get the path to the download state file
    pub fn state_path(&self) -> PathBuf {
        self.kb_root
            .join(".knowledge-loom-index")
            .join("models")
            .join(STATE_FILE)
    }

    /// Get the path to the lock file
    pub fn lock_path(&self) -> PathBuf {
        self.kb_root
            .join(".knowledge-loom-index")
            .join("models")
            .join(LOCK_FILE)
    }

    /// Check if the model is valid (exists, validated, correct version)
    pub fn is_model_valid(&self) -> anyhow::Result<bool> {
        let model_path = self.model_path();
        if !model_path.exists() {
            return Ok(false);
        }

        let metadata_path = self.metadata_path();
        if !metadata_path.exists() {
            return Ok(false);
        }

        let metadata: ModelMetadata = serde_json::from_str(
            &std::fs::read_to_string(&metadata_path).context("Failed to read metadata")?,
        )
        .context("Failed to parse metadata")?;

        // Check version mismatch
        if !metadata.is_version_match(MODEL_VERSION) {
            eprintln!(
                "Model version mismatch: expected {}, found {}",
                MODEL_VERSION, metadata.model_version
            );
            return Ok(false);
        }

        if !metadata.validated {
            return Ok(false);
        }

        Ok(true)
    }

    /// Get the download state
    ///
    /// This function retrieves the current download state from the state file.
    /// If the state file doesn't exist, it returns None.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(DownloadState))` - If the state file exists and contains valid state
    /// * `Ok(None)` - If the state file doesn't exist
    /// * `Err(anyhow::Error)` - If the state file cannot be read or parsed
    pub fn get_download_state(&self) -> anyhow::Result<Option<DownloadState>> {
        let state_path = self.state_path();
        if !state_path.exists() {
            return Ok(None);
        }

        let content =
            std::fs::read_to_string(&state_path).context("Failed to read download state")?;

        let state: DownloadState =
            serde_json::from_str(&content).context("Failed to parse download state")?;

        Ok(Some(state))
    }

    /// Set the download state
    ///
    /// This function persists the download state to the state file.
    /// It creates the parent directory if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `state` - The download state to persist
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the state was successfully persisted
    /// * `Err(anyhow::Error)` - If the state cannot be serialized or written
    pub fn set_download_state(&self, state: &DownloadState) -> anyhow::Result<()> {
        let state_path = self.state_path();

        // Create parent directory if it doesn't exist
        if let Some(parent) = state_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create state directory")?;
        }

        let content =
            serde_json::to_string_pretty(state).context("Failed to serialize download state")?;

        // Write to temporary file first, then rename atomically
        // This prevents corruption if the process crashes during write
        let temp_path = state_path.with_extension(".tmp");
        std::fs::write(&temp_path, content).context("Failed to write temporary state file")?;

        // Rename atomically (this is guaranteed to be atomic on most filesystems)
        std::fs::rename(&temp_path, &state_path).context("Failed to rename state file")?;

        Ok(())
    }

    /// Get the model metadata
    ///
    /// This function retrieves the model metadata from the metadata file.
    /// If the metadata file doesn't exist, it returns None.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(ModelMetadata))` - If the metadata file exists and contains valid metadata
    /// * `Ok(None)` - If the metadata file doesn't exist
    /// * `Err(anyhow::Error)` - If the metadata file cannot be read or parsed
    pub fn get_model_metadata(&self) -> anyhow::Result<Option<ModelMetadata>> {
        let metadata_path = self.metadata_path();
        if !metadata_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&metadata_path).context("Failed to read metadata")?;

        let metadata: ModelMetadata =
            serde_json::from_str(&content).context("Failed to parse metadata")?;

        Ok(Some(metadata))
    }

    /// Set the model metadata
    ///
    /// This function persists the model metadata to the metadata file.
    /// It creates the parent directory if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `metadata` - The model metadata to persist
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the metadata was successfully persisted
    /// * `Err(anyhow::Error)` - If the metadata cannot be serialized or written
    pub fn set_model_metadata(&self, metadata: &ModelMetadata) -> anyhow::Result<()> {
        let metadata_path = self.metadata_path();

        // Create parent directory if it doesn't exist
        if let Some(parent) = metadata_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create metadata directory")?;
        }

        let content =
            serde_json::to_string_pretty(metadata).context("Failed to serialize metadata")?;

        std::fs::write(&metadata_path, content).context("Failed to write metadata")?;

        Ok(())
    }

    /// Delete the model file
    ///
    /// This function deletes the model file and its metadata file if they exist.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the model and metadata were successfully deleted
    /// * `Err(anyhow::Error)` - If the files cannot be deleted
    pub fn delete_model(&self) -> anyhow::Result<()> {
        let model_path = self.model_path();
        if model_path.exists() {
            std::fs::remove_file(&model_path).context("Failed to delete model file")?;
        }

        let metadata_path = self.metadata_path();
        if metadata_path.exists() {
            std::fs::remove_file(&metadata_path).context("Failed to delete metadata file")?;
        }

        Ok(())
    }

    /// Validate the model file
    ///
    /// This function validates the model file by calculating its SHA-256 checksum
    /// and comparing it with the expected checksum. If the checksums don't match,
    /// it returns an error with detailed information about the mismatch.
    ///
    /// # Arguments
    ///
    /// * `expected_checksum` - The expected SHA-256 checksum of the model file
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the model file exists and the checksum matches
    /// * `Ok(false)` - If the model file doesn't exist
    /// * `Err(anyhow::Error)` - If the checksum doesn't match or validation fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The model file cannot be read
    /// - The checksum calculation fails
    /// - The checksum doesn't match the expected value
    pub fn validate_model(&self, expected_checksum: &str) -> anyhow::Result<bool> {
        let model_path = self.model_path();
        if !model_path.exists() {
            return Ok(false);
        }

        // Calculate checksum
        let checksum = self.calculate_checksum()?;

        // Compare with expected checksum
        if checksum != expected_checksum {
            return Err(anyhow::anyhow!(
                "Checksum mismatch: expected {}, got {}",
                expected_checksum,
                checksum
            ));
        }

        // Update metadata to mark as validated
        // Create metadata if it doesn't exist (e.g., after manual download)
        if let Some(mut metadata) = self.get_model_metadata()? {
            metadata.mark_validated();
            self.set_model_metadata(&metadata)?;
        } else {
            // Create new metadata with validation status
            let file_size = std::fs::metadata(self.model_path())
                .context("Failed to get model file size")?
                .len();
            let metadata = ModelMetadata::new(
                MODEL_NAME.to_string(),
                MODEL_VERSION.to_string(),
                self.model_path().to_string_lossy().to_string(),
                file_size,
                checksum.clone(),
                MODEL_URL.to_string(),
            );
            let mut metadata = metadata;
            metadata.mark_validated();
            self.set_model_metadata(&metadata)?;
        }

        Ok(true)
    }

    /// Calculate SHA-256 checksum of the model file
    fn calculate_checksum(&self) -> anyhow::Result<String> {
        use sha2::{Digest, Sha256};
        use std::io::Read;

        let model_path = self.model_path();
        let mut file = std::fs::File::open(&model_path).context("Failed to open model file")?;

        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let n = file
                .read(&mut buffer)
                .context("Failed to read model file")?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }
}

/// Download progress structure
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub percentage: f64,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub speed: f64,
    pub eta_seconds: Option<u64>,
}

impl DownloadProgress {
    pub fn new(bytes_downloaded: u64, total_bytes: u64, speed: f64) -> Self {
        let percentage = if total_bytes > 0 {
            (bytes_downloaded as f64 / total_bytes as f64) * 100.0
        } else {
            0.0
        };

        let eta_seconds = if speed > 0.0 && bytes_downloaded < total_bytes {
            let remaining_bytes = total_bytes.saturating_sub(bytes_downloaded);
            Some((remaining_bytes as f64 / speed) as u64)
        } else {
            None
        };

        Self {
            percentage,
            bytes_downloaded,
            total_bytes,
            speed,
            eta_seconds,
        }
    }
}

/// Format error with manual download instructions
///
/// This function formats an error message with manual download instructions
/// to help users recover from download failures.
///
/// # Arguments
///
/// * `error` - The error that occurred
/// * `kb_root` - The knowledge base root directory
///
/// # Returns
///
/// Returns a formatted error message with manual download instructions.
///
/// # Examples
///
/// ```no_run
/// use knowledge_loom::model::format_error_with_instructions;
/// use knowledge_loom::model::DownloadError;
/// use std::path::PathBuf;
///
/// let error = DownloadError::Network("Connection timeout".to_string());
/// let kb_root = PathBuf::from("/path/to/kb");
/// let formatted = format_error_with_instructions(&error, &kb_root);
/// println!("{}", formatted);
/// ```
pub fn format_error_with_instructions<E: std::fmt::Display>(
    error: &E,
    kb_root: &std::path::Path,
) -> String {
    let error_message = format!("{}", error);
    let instructions = format_manual_download_instructions(kb_root);

    format!(
        "{}\n\n{}\n\nFor manual download instructions, see above.",
        error_message, instructions
    )
}

/// Format manual download instructions
///
/// This function formats manual download instructions for displaying to users.
///
/// # Arguments
///
/// * `kb_root` - The knowledge base root directory
///
/// # Returns
///
/// Returns formatted manual download instructions.
fn format_manual_download_instructions(kb_root: &std::path::Path) -> String {
    let models_dir = kb_root.join(".knowledge-loom-index").join("models");
    let model_file = models_dir.join("all-MiniLM-L6-v2.onnx");

    format!(
        "Manual Model Download Instructions:\n\
         \n\
         Step 1: Download the model file\n\
           Download URL: https://huggingface.co/Qdrant/all-MiniLM-L6-v2-onnx/resolve/main/model.onnx\n\
           Model name: all-MiniLM-L6-v2\n\
           Expected size: ~120MB\n\
         \n\
           You can download using:\n\
           - curl: curl -L -o \"{}\" \"https://huggingface.co/Qdrant/all-MiniLM-L6-v2-onnx/resolve/main/model.onnx\"\n\
           - wget: wget -O \"{}\" \"https://huggingface.co/Qdrant/all-MiniLM-L6-v2-onnx/resolve/main/model.onnx\"\n\
         \n\
         Step 2: Create the models directory\n\
           mkdir -p \"{}\"\n\
         \n\
         Step 3: Move the downloaded file to the models directory\n\
           mv all-MiniLM-L6-v2.onnx \"{}\"\n\
         \n\
         Step 4: Run initialization again\n\
           Run 'loom init' again to complete the initialization process.",
        model_file.display(),
        model_file.display(),
        models_dir.display(),
        model_file.display()
    )
}
