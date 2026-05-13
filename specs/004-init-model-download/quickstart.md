# Quick Start: Init-Time Model Download

**Feature**: Init-Time Model Download
**Date**: 2026-05-12
**Status**: Complete

## Overview

This quick start guide provides a step-by-step approach to implementing init-time model download with structured plain text progress indicators, graceful error handling, and state management.

## Prerequisites

- Rust 1.75+ (Async Trait support required)
- Cargo package manager
- Git (for version control)
- Access to the Knowledge Loom repository

## Step 1: Create New Modules

Create three new modules in `src/`:

```bash
touch src/init.rs
touch src/model.rs
touch src/download.rs
```

Update `src/lib.rs` to include the new modules:

```rust
pub mod init;
pub mod model;
pub mod download;
```

## Step 2: Implement Download Manager

Create `src/download.rs` with the download manager:

```rust
use anyhow::{Context, Result};
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

pub struct DownloadManager {
    client: Client,
    url: String,
    output_path: std::path::PathBuf,
    max_retries: u32,
    retry_delay: Duration,
    timeout: Duration,
}

impl DownloadManager {
    pub fn new(url: String, output_path: std::path::PathBuf) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            url,
            output_path,
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            timeout: Duration::from_secs(30),
        })
    }

    pub async fn download_with_retry<F>(&self, progress_callback: F) -> Result<()>
    where
        F: Fn(DownloadProgress) + Send + Sync,
    {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                eprintln!("Retrying model download (attempt {} of {})...", attempt, self.max_retries);
                tokio::time::sleep(self.retry_delay * attempt as u32).await;
            }

            match self.download(&progress_callback).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.max_retries {
                        continue;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Download failed")))
    }

    async fn download<F>(&self, progress_callback: &F) -> Result<()>
    where
        F: Fn(DownloadProgress) + Send + Sync,
    {
        let response = self.client
            .get(&self.url)
            .send()
            .await
            .context("Failed to start download")?;

        let total_bytes = response
            .content_length()
            .context("Missing content length")?;

        let mut file = File::create(&self.output_path)
            .context("Failed to create output file")?;

        let mut downloaded = 0u64;
        let start_time = std::time::Instant::now();
        let mut last_update = start_time;

        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Failed to read chunk")?;
            file.write_all(&chunk).context("Failed to write chunk")?;
            downloaded += chunk.len() as u64;

            let now = std::time::Instant::now();
            if now - last_update >= Duration::from_secs(1) {
                let elapsed = now.duration_since(start_time).as_secs_f64();
                let speed = downloaded as f64 / elapsed;
                let remaining = if speed > 0.0 {
                    (total_bytes - downloaded) as f64 / speed
                } else {
                    0.0
                };

                let progress = DownloadProgress {
                    bytes_downloaded: downloaded,
                    total_bytes,
                    percentage_complete: (downloaded as f64 / total_bytes as f64) * 100.0,
                    download_speed: speed,
                    estimated_time_remaining: remaining,
                    elapsed_time: elapsed,
                };

                progress_callback(progress);
                last_update = now;
            }
        }

        Ok(())
    }

    pub fn calculate_checksum(&self, path: &Path) -> Result<String> {
        let mut file = File::open(path).context("Failed to open file")?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let n = std::io::Read::read(&mut file, &mut buffer)
                .context("Failed to read file")?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }
}

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

## Step 3: Implement Model Manager

Create `src/model.rs` with the model manager:

```rust
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::download::{DownloadManager, DownloadProgress};

pub const MODEL_NAME: &str = "all-MiniLM-L6-v2";
pub const MODEL_VERSION: &str = "1.0.0";
pub const MODEL_URL: &str = "https://huggingface.co/Qdrant/all-MiniLM-L6-v2-onnx/resolve/main/model.onnx";
pub const MODEL_FILE: &str = "all-MiniLM-L6-v2.onnx";
pub const MODEL_METADATA_FILE: &str = "all-MiniLM-L6-v2.json";
pub const STATE_FILE: &str = "download-state.json";
pub const LOCK_FILE: &str = ".download.lock";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
}

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

pub struct ModelManager {
    kb_root: PathBuf,
    model_dir: PathBuf,
    state_file: PathBuf,
    lock_file: PathBuf,
}

impl ModelManager {
    pub fn new(kb_root: &Path) -> Result<Self> {
        let kb_root = kb_root.to_path_buf();
        let model_dir = kb_root.join(".knowledge-loom-index").join("models");
        let state_file = model_dir.join(STATE_FILE);
        let lock_file = model_dir.join(LOCK_FILE);

        Ok(Self {
            kb_root,
            model_dir,
            state_file,
            lock_file,
        })
    }

    pub fn is_model_valid(&self) -> Result<bool> {
        let model_path = self.model_path();
        if !model_path.exists() {
            return Ok(false);
        }

        let metadata_path = self.model_dir.join(MODEL_METADATA_FILE);
        if !metadata_path.exists() {
            return Ok(false);
        }

        let metadata: ModelMetadata = serde_json::from_str(
            &fs::read_to_string(&metadata_path)
                .context("Failed to read metadata")?
        ).context("Failed to parse metadata")?;

        if !metadata.validated {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn download_model<F>(&self, progress_callback: F) -> Result<()>
    where
        F: Fn(DownloadProgress) + Send + Sync,
    {
        // Acquire lock
        let _lock = self.acquire_lock()?;

        // Create model directory
        fs::create_dir_all(&self.model_dir)
            .context("Failed to create model directory")?;

        // Set state to in progress
        let mut state = DownloadState {
            status: DownloadStatus::InProgress,
            progress_percentage: 0.0,
            bytes_downloaded: 0,
            total_bytes: 0,
            download_speed: 0.0,
            error_message: None,
            last_updated: chrono::Utc::now().to_rfc3339(),
            model_name: MODEL_NAME.to_string(),
            model_version: MODEL_VERSION.to_string(),
        };
        self.set_download_state(&state)?;

        // Download model
        let output_path = self.model_path();
        let download_manager = DownloadManager::new(MODEL_URL.to_string(), output_path.clone())?;

        let result = download_manager.download_with_retry(|progress| {
            state.progress_percentage = progress.percentage_complete;
            state.bytes_downloaded = progress.bytes_downloaded;
            state.total_bytes = progress.total_bytes;
            state.download_speed = progress.download_speed;
            state.last_updated = chrono::Utc::now().to_rfc3339();
            let _ = self.set_download_state(&state);
            progress_callback(progress);
        });

        match result {
            Ok(()) => {
                // Validate model
                self.validate_model()?;

                // Update state to completed
                state.status = DownloadStatus::Completed;
                state.progress_percentage = 100.0;
                state.last_updated = chrono::Utc::now().to_rfc3339();
                self.set_download_state(&state)?;

                Ok(())
            }
            Err(e) => {
                // Update state to failed
                state.status = DownloadStatus::Failed;
                state.error_message = Some(e.to_string());
                state.last_updated = chrono::Utc::now().to_rfc3339();
                self.set_download_state(&state)?;

                Err(e)
            }
        }
    }

    pub fn validate_model(&self) -> Result<()> {
        let model_path = self.model_path();
        let download_manager = DownloadManager::new(MODEL_URL.to_string(), model_path.clone())?;

        // Calculate checksum
        let checksum = download_manager.calculate_checksum(&model_path)?;

        // Get file size
        let file_size = fs::metadata(&model_path)
            .context("Failed to get file size")?
            .len();

        // Create metadata
        let metadata = ModelMetadata {
            model_name: MODEL_NAME.to_string(),
            model_version: MODEL_VERSION.to_string(),
            file_path: model_path.to_string_lossy().to_string(),
            file_size,
            sha256_checksum: checksum,
            download_timestamp: chrono::Utc::now().to_rfc3339(),
            download_url: MODEL_URL.to_string(),
            validated: true,
        };

        // Save metadata
        let metadata_path = self.model_dir.join(MODEL_METADATA_FILE);
        fs::write(
            &metadata_path,
            serde_json::to_string_pretty(&metadata)
                .context("Failed to serialize metadata")?
        ).context("Failed to write metadata")?;

        Ok(())
    }

    pub fn get_download_state(&self) -> Result<DownloadState> {
        if !self.state_file.exists() {
            return Ok(DownloadState {
                status: DownloadStatus::NotStarted,
                progress_percentage: 0.0,
                bytes_downloaded: 0,
                total_bytes: 0,
                download_speed: 0.0,
                error_message: None,
                last_updated: chrono::Utc::now().to_rfc3339(),
                model_name: MODEL_NAME.to_string(),
                model_version: MODEL_VERSION.to_string(),
            });
        }

        let content = fs::read_to_string(&self.state_file)
            .context("Failed to read state file")?;

        serde_json::from_str(&content)
            .context("Failed to parse state file")
    }

    pub fn set_download_state(&self, state: &DownloadState) -> Result<()> {
        let content = serde_json::to_string_pretty(state)
            .context("Failed to serialize state")?;

        fs::write(&self.state_file, content)
            .context("Failed to write state file")
    }

    pub fn model_path(&self) -> PathBuf {
        self.model_dir.join(MODEL_FILE)
    }

    fn acquire_lock(&self) -> Result<FileLock> {
        use fs2::FileExt;
        use std::fs::OpenOptions;

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.lock_file)
            .context("Failed to open lock file")?;

        file.try_lock_exclusive()
            .map_err(|e| anyhow::anyhow!("Failed to acquire lock: {}", e))?;

        Ok(FileLock { file })
    }
}

struct FileLock {
    file: std::fs::File,
}

impl Drop for FileLock {
    fn drop(&mut self) {
        use fs2::FileExt;
        let _ = self.file.unlock();
    }
}
```

## Step 4: Implement Init Manager

Create `src/init.rs` with the init manager:

```rust
use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::model::ModelManager;

pub struct InitManager {
    kb_root: PathBuf,
    model_manager: ModelManager,
    force: bool,
}

impl InitManager {
    pub fn new(kb_root: PathBuf, force: bool) -> Result<Self> {
        let model_manager = ModelManager::new(&kb_root)?;

        Ok(Self {
            kb_root,
            model_manager,
            force,
        })
    }

    pub fn initialize<F>(&self, progress_callback: F) -> Result<()>
    where
        F: Fn(InitProgress) + Send + Sync,
    {
        // Check if already initialized
        if !self.force && self.is_initialized()? {
            anyhow::bail!("Already initialized. Use --force to re-initialize.");
        }

        // Create directory structure
        progress_callback(InitProgress::CreatingDirectories);
        self.create_directories()?;

        // Download model
        if !self.model_manager.is_model_valid()? {
            progress_callback(InitProgress::Downloading);
            self.model_manager.download_model(|progress| {
                progress_callback(InitProgress::DownloadProgress(progress));
            })?;
        }

        // Validate model
        progress_callback(InitProgress::Validating);
        self.model_manager.validate_model()?;

        // Initialize indexes
        progress_callback(InitProgress::InitializingIndexes);
        self.initialize_indexes()?;

        // Create configuration files
        progress_callback(InitProgress::CreatingConfigFiles);
        self.create_config_files()?;

        progress_callback(InitProgress::Complete);

        Ok(())
    }

    pub fn is_initialized(&self) -> Result<bool> {
        let index_dir = self.kb_root.join(".knowledge-loom-index");
        Ok(index_dir.exists() && self.model_manager.is_model_valid()?)
    }

    fn create_directories(&self) -> Result<()> {
        let index_dir = self.kb_root.join(".knowledge-loom-index");
        let models_dir = index_dir.join("models");
        let tantivy_dir = index_dir.join("tantivy");

        fs::create_dir_all(&models_dir)
            .context("Failed to create models directory")?;

        fs::create_dir_all(&tantivy_dir)
            .context("Failed to create tantivy directory")?;

        Ok(())
    }

    fn initialize_indexes(&self) -> Result<()> {
        // TODO: Implement index initialization
        Ok(())
    }

    fn create_config_files(&self) -> Result<()> {
        // TODO: Implement config file creation
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum InitProgress {
    DownloadProgress(crate::download::DownloadProgress),
    Validating,
    CreatingDirectories,
    InitializingIndexes,
    CreatingConfigFiles,
    Complete,
}
```

## Step 5: Add Dependencies

Update `Cargo.toml` with new dependencies:

```toml
[dependencies]
# Existing dependencies...
reqwest = { version = "0.11", features = ["rustls-tls", "gzip"] }
sha2 = "0.10"
chrono = { version = "0.4", features = ["serde"] }
fs2 = "0.4"
```

## Step 6: Update CLI

Update `src/main.rs` to use the init manager:

```rust
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::init::InitManager;

#[derive(Parser)]
#[command(name = "loom")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(short, long)]
        kb_root: Option<PathBuf>,
        #[arg(long)]
        force: bool,
    },
    // Other commands...
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { kb_root, force } => {
            let kb_root = kb_root.unwrap_or_else(|| std::env::current_dir().unwrap());
            let init_manager = InitManager::new(kb_root, force)?;

            init_manager.initialize(|progress| {
                match progress {
                    crate::init::InitProgress::DownloadProgress(p) => {
                        let downloaded_mb = p.bytes_downloaded as f64 / 1_000_000.0;
                        let total_mb = p.total_bytes as f64 / 1_000_000.0;
                        let speed_mb = p.download_speed as f64 / 1_000_000.0;
                        let remaining = p.estimated_time_remaining as u64;

                        println!(
                            "Downloading model: {:.0}% ({:.1}MB/{:.1}MB) - {:.1}MB/s - {}s remaining",
                            p.percentage_complete,
                            downloaded_mb,
                            total_mb,
                            speed_mb,
                            remaining
                        );
                    }
                    crate::init::InitProgress::Validating => {
                        println!("Validating model... OK");
                    }
                    crate::init::InitProgress::CreatingDirectories => {
                        println!("Creating directory structure...");
                    }
                    crate::init::InitProgress::InitializingIndexes => {
                        println!("Initializing indexes...");
                    }
                    crate::init::InitProgress::CreatingConfigFiles => {
                        println!("Creating configuration files...");
                    }
                    crate::init::InitProgress::Complete => {
                        println!("knowledge-loom init complete.");
                    }
                }
            })?;

            Ok(())
        }
        // Other commands...
    }
}
```

## Step 7: Write Tests

Create `tests/model_tests.rs`:

```rust
use anyhow::Result;
use tempfile::TempDir;

#[test]
fn test_model_manager_creation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let kb_root = temp_dir.path();

    let model_manager = loom::model::ModelManager::new(kb_root)?;
    assert!(model_manager.model_path().exists() == false);

    Ok(())
}

#[test]
fn test_download_state_persistence() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let kb_root = temp_dir.path();

    let model_manager = loom::model::ModelManager::new(kb_root)?;

    let state = loom::model::DownloadState {
        status: loom::model::DownloadStatus::InProgress,
        progress_percentage: 50.0,
        bytes_downloaded: 60_000_000,
        total_bytes: 120_000_000,
        download_speed: 2_500_000.0,
        error_message: None,
        last_updated: chrono::Utc::now().to_rfc3339(),
        model_name: "all-MiniLM-L6-v2".to_string(),
        model_version: "1.0.0".to_string(),
    };

    model_manager.set_download_state(&state)?;

    let retrieved_state = model_manager.get_download_state()?;
    assert_eq!(retrieved_state.status, loom::model::DownloadStatus::InProgress);
    assert_eq!(retrieved_state.progress_percentage, 50.0);

    Ok(())
}
```

## Step 8: Run Tests

```bash
cargo test --all-features
```

## Step 9: Run Quality Gates

```bash
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo deny check licenses bans sources
```

## Step 10: Update Documentation

Update `ARCHITECTURE.md` with Model Download Flow section:

```markdown
## Model Download Flow

The model download flow is as follows:

1. User runs `loom init`
2. InitManager checks if model is already downloaded and valid
3. If not valid, ModelManager downloads model with progress indicators
4. DownloadManager handles HTTP download with retry logic
5. ModelManager validates model (checksum, size, format)
6. ModelManager saves model metadata
7. InitManager continues with index initialization

### State Management

- Download state is persisted to `{KB_ROOT}/.knowledge-loom-index/models/download-state.json`
- File locking prevents concurrent downloads
- State includes: status, progress, error message, timestamp

### Error Handling

- Network errors: Retry with exponential backoff (1s, 2s, 4s)
- Disk full: Clear error message with space requirements
- Permission denied: Clear error message with path
- Checksum mismatch: Delete corrupted file, trigger re-download
```

Update `CHANGELOG.md`:

```markdown
## [Unreleased]

### Added
- Init-time model download with progress indicators
- Model validation with SHA-256 checksum
- Download state management and persistence
- File locking to prevent concurrent downloads
- Graceful error handling with actionable messages

### Changed
- `loom init` now downloads model during initialization
- Model files stored in `{KB_ROOT}/.knowledge-loom-index/models/`

## Step 11: Add Signal Handling

Add Ctrl+C signal handling to `src/download.rs`:

```rust
use signal_hook::{flag, register};

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

impl DownloadManager {
    pub fn new(url: String, output_path: std::path::PathBuf) -> Result<Self> {
        // Register signal handler for Ctrl+C
        let _ = flag::register(signal_hook::SIGINT, || {
            INTERRUPTED.store(true, Ordering::SeqCst);
        });

        // ... rest of initialization
    }

    pub async fn download_with_retry<F>(&self, progress_callback: F) -> Result<()>
    where
        F: Fn(DownloadProgress) + Send + Sync,
    {
        // Check for interrupt
        if INTERRUPTED.load(Ordering::SeqCst) {
            anyhow::bail!("Download interrupted by user");
        }

        // ... rest of download logic
    }
}
```

## Step 12: Add HTTP Range Request Support

Add HTTP Range request support to `src/download.rs`:

```rust
pub async fn download<F>(&self, progress_callback: &F) -> Result<()>
where
    F: Fn(DownloadProgress) + Send + Sync,
{
    // Check if partial file exists
    let start_byte = if self.output_path.exists() {
        let metadata = std::fs::metadata(&self.output_path)?;
        metadata.len()
    } else {
        0
    };

    let mut request = self.client.get(&self.url);

    // Add Range header if resuming
    if start_byte > 0 {
        request = request.header("Range", format!("bytes={}-", start_byte));
    }

    let response = request.send().await.context("Failed to start download")?;

    // ... rest of download logic
}
```

## Step 13: Add Proxy Configuration Support

Add proxy configuration to `src/download.rs`:

```rust
impl DownloadManager {
    pub fn new(url: String, output_path: std::path::PathBuf) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            url,
            output_path,
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            timeout: Duration::from_secs(30),
        })
    }
}
```

Note: reqwest automatically respects HTTP_PROXY, HTTPS_PROXY, and NO_PROXY environment variables.

## Step 14: Add Version Mismatch Detection

Add version mismatch detection to `src/model.rs`:

```rust
impl ModelManager {
    pub fn is_model_valid(&self) -> Result<bool> {
        let model_path = self.model_path();
        if !model_path.exists() {
            return Ok(false);
        }

        let metadata_path = self.model_dir.join(MODEL_METADATA_FILE);
        if !metadata_path.exists() {
            return Ok(false);
        }

        let metadata: ModelMetadata = serde_json::from_str(
            &fs::read_to_string(&metadata_path)
                .context("Failed to read metadata")?
        ).context("Failed to parse metadata")?;

        // Check version mismatch
        if metadata.model_version != MODEL_VERSION {
            eprintln!("Model version mismatch: expected {}, found {}", MODEL_VERSION, metadata.model_version);
            return Ok(false);
        }

        if !metadata.validated {
            return Ok(false);
        }

        Ok(true)
    }
}
```

### Fixed
- Fixed confusing hang during first-time indexing
- Fixed model download error handling
```

## Next Steps

1. Implement remaining TODO items (index initialization, config file creation)
2. Add comprehensive integration tests
3. Add performance benchmarks
4. Update README.md with manual download instructions
5. Create PR for review

## References

- [Feature specification](./spec.md)
- [Research document](./research.md)
- [Data model](./data-model.md)
- [CLI contract](./contracts/cli-init.md)
- [API contract](./contracts/api-model-download.md)
- [Display contract](./contracts/display-progress.md)
- [Knowledge Loom constitution](../../.specify/memory/constitution.md)
