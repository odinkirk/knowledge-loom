# API Contract: Model Download

**Feature**: Init-Time Model Download
**Date**: 2026-05-12
**Status**: Complete

## Overview

This document defines the internal API contract for model download, including function signatures, data structures, error handling, and state management.

## Module Structure

```
src/
├── init.rs              # Initialization utilities
├── model.rs              # Model download and management
└── download.rs           # Download progress and error handling
```

## Public API

### Model Manager (`src/model.rs`)

#### `ModelManager`

Manages model download, validation, and state.

**Fields**:

```rust
pub struct ModelManager {
    kb_root: PathBuf,
    model_dir: PathBuf,
    state_file: PathBuf,
    lock_file: PathBuf,
}
```

**Methods**:

```rust
impl ModelManager {
    /// Create a new ModelManager
    pub fn new(kb_root: &Path) -> Result<Self>;

    /// Check if model is downloaded and valid
    pub fn is_model_valid(&self) -> Result<bool>;

    /// Download model with progress callback
    pub fn download_model<F>(&self, progress_callback: F) -> Result<()>
    where
        F: Fn(DownloadProgress) + Send + Sync;

    /// Validate model file
    pub fn validate_model(&self) -> Result<()>;

    /// Get model download state
    pub fn get_download_state(&self) -> Result<DownloadState>;

    /// Set model download state
    pub fn set_download_state(&self, state: &DownloadState) -> Result<()>;

    /// Delete model file
    pub fn delete_model(&self) -> Result<()>;

    /// Get model file path
    pub fn model_path(&self) -> PathBuf;

    /// Get model metadata
    pub fn get_model_metadata(&self) -> Result<ModelMetadata>;
}
```

### Download Manager (`src/download.rs`)

#### `DownloadManager`

Handles HTTP download with progress tracking and retry logic.

**Fields**:

```rust
pub struct DownloadManager {
    client: reqwest::Client,
    url: String,
    output_path: PathBuf,
    max_retries: u32,
    retry_delay: Duration,
    timeout: Duration,
}
```

**Methods**:

```rust
impl DownloadManager {
    /// Create a new DownloadManager
    pub fn new(url: String, output_path: PathBuf) -> Result<Self>;

    /// Download file with progress callback
    pub async fn download<F>(&self, progress_callback: F) -> Result<()>
    where
        F: Fn(DownloadProgress) + Send + Sync;

    /// Download file with retry logic
    pub async fn download_with_retry<F>(&self, progress_callback: F) -> Result<()>
    where
        F: Fn(DownloadProgress) + Send + Sync;

    /// Resume interrupted download
    pub async fn resume_download<F>(&self, progress_callback: F) -> Result<()>
    where
        F: Fn(DownloadProgress) + Send + Sync;

    /// Calculate SHA-256 checksum
    pub fn calculate_checksum(&self, path: &Path) -> Result<String>;

    /// Validate file size
    pub fn validate_file_size(&self, path: &Path, expected_size: u64) -> Result<()>;
}
```

### Init Manager (`src/init.rs`)

#### `InitManager`

Handles initialization of knowledge base.

**Fields**:

```rust
pub struct InitManager {
    kb_root: PathBuf,
    model_manager: ModelManager,
    force: bool,
}
```

**Methods**:

```rust
impl InitManager {
    /// Create a new InitManager
    pub fn new(kb_root: PathBuf, force: bool) -> Result<Self>;

    /// Initialize knowledge base
    pub fn initialize<F>(&self, progress_callback: F) -> Result<()>
    where
        F: Fn(InitProgress) + Send + Sync;

    /// Check if already initialized
    pub fn is_initialized(&self) -> Result<bool>;

    /// Create directory structure
    pub fn create_directories(&self) -> Result<()>;

    /// Initialize indexes
    pub fn initialize_indexes(&self) -> Result<()>;

    /// Create configuration files
    pub fn create_config_files(&self) -> Result<()>;
}
```

## Data Structures

### `DownloadState`

Represents the current download state.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadState {
    pub status: DownloadStatus,
    pub progress_percentage: f64,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub download_speed: f64,
    pub error_message: Option<String>,
    pub last_updated: String,
    pub model_name: String,
    pub model_version: String,
}
```

### `DownloadStatus`

Download status enum.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
}
```

### `DownloadProgress`

Real-time download progress information.

```rust
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub percentage_complete: f64,
    pub download_speed: f64,
    pub estimated_time_remaining: f64,
    pub elapsed_time: f64,
}
```

### `ModelMetadata`

Model file metadata.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub model_name: String,
    pub model_version: String,
    pub file_path: String,
    pub file_size: u64,
    pub sha256_checksum: String,
    pub download_timestamp: String,
    pub download_url: String,
    pub validated: bool,
}
```

### `InitProgress`

Initialization progress information.

```rust
#[derive(Debug, Clone)]
pub enum InitProgress {
    Downloading(DownloadProgress),
    Validating,
    CreatingDirectories,
    InitializingIndexes,
    CreatingConfigFiles,
    Complete,
}
```

## Error Types

### `ModelError`

Model-related errors.

```rust
#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("Model not found: {0}")]
    NotFound(String),

    #[error("Model validation failed: {0}")]
    ValidationFailed(String),

    #[error("Model download failed: {0}")]
    DownloadFailed(String),

    #[error("Model state error: {0}")]
    StateError(String),

    #[error("Model lock error: {0}")]
    LockError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
```

### `DownloadError`

Download-related errors.

```rust
#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Disk full: {0}")]
    DiskFull(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### `InitError`

Initialization-related errors.

```rust
#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error("Already initialized: {0}")]
    AlreadyInitialized(String),

    #[error("Initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Model error: {0}")]
    Model(#[from] ModelError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

## Constants

### Model Configuration

```rust
pub const MODEL_NAME: &str = "all-MiniLM-L6-v2";
pub const MODEL_VERSION: &str = "1.0.0";
pub const MODEL_URL: &str = "https://huggingface.co/Qdrant/all-MiniLM-L6-v2-onnx/resolve/main/model.onnx";
pub const MODEL_FILE: &str = "all-MiniLM-L6-v2.onnx";
pub const MODEL_METADATA_FILE: &str = "all-MiniLM-L6-v2.json";
pub const EXPECTED_MODEL_SIZE: u64 = 120_000_000; // ~120MB
pub const EXPECTED_CHECKSUM: &str = "a1b2c3d4e5f6..."; // To be determined
```

### Download Configuration

```rust
pub const MAX_RETRIES: u32 = 3;
pub const RETRY_DELAY: Duration = Duration::from_secs(1);
pub const TIMEOUT: Duration = Duration::from_secs(30);
pub const BUFFER_SIZE: usize = 8192; // 8KB chunks
pub const PROGRESS_UPDATE_INTERVAL: Duration = Duration::from_secs(1);
```

### State Configuration

```rust
pub const STATE_FILE: &str = "download-state.json";
pub const LOCK_FILE: &str = ".download.lock";
pub const LOCK_TIMEOUT: Duration = Duration::from_secs(0); // Fail immediately
```

## Helper Functions

### Progress Formatting

```rust
/// Format download progress as structured plain text
pub fn format_download_progress(progress: &DownloadProgress) -> String {
    let downloaded_mb = progress.bytes_downloaded as f64 / 1_000_000.0;
    let total_mb = progress.total_bytes as f64 / 1_000_000.0;
    let speed_mb = progress.download_speed as f64 / 1_000_000.0;
    let remaining = progress.estimated_time_remaining as u64;

    format!(
        "Downloading model: {:.0}% ({:.1}MB/{:.1}MB) - {:.1}MB/s - {}s remaining",
        progress.percentage_complete,
        downloaded_mb,
        total_mb,
        speed_mb,
        remaining
    )
}

/// Format download completion as structured plain text
pub fn format_download_complete(progress: &DownloadProgress) -> String {
    let total_mb = progress.total_bytes as f64 / 1_000_000.0;
    let speed_mb = progress.download_speed as f64 / 1_000_000.0;
    let elapsed = progress.elapsed_time as u64;

    format!(
        "Model download complete: 100% ({:.1}MB/{:.1}MB) - {:.1}MB/s - {}s total",
        total_mb, total_mb, speed_mb, elapsed
    )
}

/// Format download error as structured plain text
pub fn format_download_error(error: &DownloadError) -> String {
    format!("Model download failed: {}", error)
}
```

### Checksum Calculation

```rust
/// Calculate SHA-256 checksum of a file
pub fn calculate_sha256_checksum(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}
```

### File Locking

```rust
/// Acquire exclusive file lock
pub fn acquire_lock(lock_path: &Path) -> Result<FileLock> {
    use fs2::FileExt;
    use std::fs::OpenOptions;

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(lock_path)?;

    file.try_lock_exclusive()
        .map_err(|e| ModelError::LockError(e.to_string()))?;

    Ok(FileLock { file })
}

/// Release file lock
pub fn release_lock(lock: FileLock) {
    use fs2::FileExt;
    let _ = lock.file.unlock();
}

struct FileLock {
    file: File,
}
```

## Testing

### Unit Tests

- Model manager methods (all public methods)
- Download manager methods (all public methods)
- Init manager methods (all public methods)
- Progress formatting (all formatting functions)
- Checksum calculation (all checksum functions)
- File locking (all locking functions)

### Integration Tests

- Model download end-to-end
- Model validation end-to-end
- Initialization end-to-end
- Error handling (all error scenarios)
- State persistence and recovery
- Concurrent download prevention

### Mock Objects

- Mock HTTP client for testing download logic
- Mock file system for testing file operations
- Mock progress callback for testing progress updates

## References

- [Feature specification](../spec.md)
- [Research document](../research.md)
- [Data model](../data-model.md)
- [CLI contract](./cli-init.md)
- [Knowledge Loom constitution](../../.specify/memory/constitution.md)
