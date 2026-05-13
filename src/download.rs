// Download module for Knowledge Loom model download
// This module handles HTTP download with retry logic, progress tracking, and checksum validation

use crate::model::{DownloadError, DownloadProgress};
use reqwest::Client;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Maximum number of download retries
pub const MAX_RETRIES: u32 = 3;

/// Base retry delay in seconds
pub const RETRY_DELAY: u64 = 1;

/// Download timeout in seconds
pub const TIMEOUT: u64 = 30;

/// Buffer size for download chunks
pub const BUFFER_SIZE: usize = 8192;

/// Progress update interval in milliseconds
pub const PROGRESS_UPDATE_INTERVAL: u64 = 1000;

/// Global flag for tracking interrupt signals (Ctrl+C)
pub static INTERRUPTED: AtomicBool = AtomicBool::new(false);

/// Format download progress as structured plain text
pub fn format_download_progress(progress: &DownloadProgress) -> String {
    let percentage = progress.percentage;
    let downloaded_mb = progress.bytes_downloaded as f64 / 1_048_576.0;
    let total_mb = progress.total_bytes as f64 / 1_048_576.0;
    let speed_mb = progress.speed / 1_048_576.0;

    let eta_str = if let Some(eta) = progress.eta_seconds {
        if eta >= 60 {
            let minutes = eta / 60;
            let seconds = eta % 60;
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", eta)
        }
    } else {
        "calculating".to_string()
    };

    format!(
        "Downloading model: {:.1}% ({:.1}MB/{:.1}MB) - {:.2}MB/s - {} remaining",
        percentage, downloaded_mb, total_mb, speed_mb, eta_str
    )
}

/// Format download completion message
pub fn format_download_complete(file_size: u64, duration_secs: u64) -> String {
    let file_size_mb = file_size as f64 / 1_048_576.0;
    let duration_str = if duration_secs >= 60 {
        let minutes = duration_secs / 60;
        let seconds = duration_secs % 60;
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", duration_secs)
    };

    format!(
        "Download complete: {:.1}MB in {}",
        file_size_mb, duration_str
    )
}

/// Format download error message
pub fn format_download_error(error: &DownloadError) -> String {
    match error {
        DownloadError::Network(msg) => format!("Network error: {}", msg),
        DownloadError::Http(msg) => format!("HTTP error: {}", msg),
        DownloadError::Interrupted => "Download interrupted by user".to_string(),
        DownloadError::MaxRetriesExceeded { retries } => {
            format!("Download failed after {} retries", retries)
        }
        DownloadError::Timeout(msg) => format!("Timeout: {}", msg),
        DownloadError::Io(e) => format!("IO error: {}", e),
        DownloadError::Reqwest(e) => format!("HTTP client error: {}", e),
    }
}

/// Acquire file lock to prevent concurrent downloads
pub fn acquire_lock(lock_path: &PathBuf) -> Result<std::fs::File, DownloadError> {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(lock_path)
        .map_err(DownloadError::Io)?;

    file.try_lock()
        .map_err(|e| DownloadError::Network(format!("Failed to acquire lock: {}", e)))?;

    Ok(file)
}

/// Release file lock
pub fn release_lock(file: std::fs::File) -> Result<(), DownloadError> {
    file.unlock()
        .map_err(|e| DownloadError::Network(format!("Failed to release lock: {}", e)))?;
    Ok(())
}

/// Setup signal handler for Ctrl+C
pub fn setup_signal_handler() -> Result<(), DownloadError> {
    #[cfg(unix)]
    {
        use signal_hook::consts::SIGINT;
        let flag = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(SIGINT, flag).map_err(|e| {
            DownloadError::Network(format!("Failed to register signal handler: {}", e))
        })?;
    }
    Ok(())
}

/// Check if download was interrupted
pub fn is_interrupted() -> bool {
    INTERRUPTED.load(Ordering::SeqCst)
}

/// Reset interrupt flag
pub fn reset_interrupt_flag() {
    INTERRUPTED.store(false, Ordering::SeqCst);
}

/// Download manager for handling HTTP downloads with retry logic
pub struct DownloadManager {
    client: Client,
    url: String,
    output_path: PathBuf,
    pub max_retries: u32,
    retry_delay: Duration,
    timeout: Duration,
}

impl DownloadManager {
    /// Create a new download manager
    pub fn new(url: String, output_path: PathBuf) -> Result<Self, DownloadError> {
        // Note: reqwest automatically respects HTTP_PROXY, HTTPS_PROXY, and NO_PROXY
        // environment variables for proxy configuration
        let client = Client::builder()
            .timeout(Duration::from_secs(TIMEOUT))
            .build()
            .map_err(|e| DownloadError::Network(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            url,
            output_path,
            max_retries: MAX_RETRIES,
            retry_delay: Duration::from_secs(RETRY_DELAY),
            timeout: Duration::from_secs(TIMEOUT),
        })
    }

    /// Set maximum number of retries
    pub fn with_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set retry delay
    pub fn with_retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }

    /// Set download timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Download file with progress callback
    pub async fn download<F>(&self, progress_callback: F) -> Result<(), DownloadError>
    where
        F: Fn(DownloadProgress) + Send + Sync,
    {
        // Check for interrupt
        if is_interrupted() {
            return Err(DownloadError::Interrupted);
        }

        // Check if partial file exists for resume
        let start_byte = if self.output_path.exists() {
            let metadata = std::fs::metadata(&self.output_path).map_err(DownloadError::Io)?;
            metadata.len()
        } else {
            0
        };

        // Build request with Range header if resuming
        let mut request = self.client.get(&self.url);
        if start_byte > 0 {
            request = request.header("Range", format!("bytes={}-", start_byte));
        }

        let response = request.timeout(self.timeout).send().await.map_err(|e| {
            if e.is_timeout() {
                DownloadError::Timeout(format!("Download timeout after {:?}", self.timeout))
            } else if e.is_connect() {
                DownloadError::Network(format!("Connection failed: {}", e))
            } else {
                DownloadError::Network(format!("Failed to start download: {}", e))
            }
        })?;

        // Check response status
        let status = response.status();
        if !status.is_success() && status != reqwest::StatusCode::PARTIAL_CONTENT {
            return Err(DownloadError::Http(format!("HTTP error: {}", status)));
        }

        // Get total bytes from Content-Range or Content-Length
        let total_bytes = if status == reqwest::StatusCode::PARTIAL_CONTENT {
            // Parse Content-Range header: "bytes 0-100/200" or "bytes 100-200/*"
            if let Some(content_range) = response.headers().get("Content-Range") {
                let range_str = content_range.to_str().unwrap_or("");
                if let Some(slash_pos) = range_str.find('/') {
                    let total_str = &range_str[slash_pos + 1..];
                    if total_str != "*" {
                        total_str.parse::<u64>().unwrap_or(0)
                    } else {
                        0
                    }
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            response
                .content_length()
                .ok_or_else(|| DownloadError::Network("Missing content length".to_string()))?
        };

        let mut bytes_downloaded = start_byte;
        let start_time = std::time::Instant::now();

        let mut stream = response.bytes_stream();

        // Create parent directory if it doesn't exist
        // Enhanced error handling for permission denied and disk full errors
        if let Some(parent) = self.output_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    DownloadError::Io(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        format!(
                            "Permission denied creating directory {}: {}",
                            parent.display(),
                            e
                        ),
                    ))
                } else {
                    DownloadError::Io(e)
                }
            })?;
        }

        // Open file in append mode if resuming, otherwise create new
        // Enhanced error handling for permission denied and disk full errors
        let mut file = if start_byte > 0 {
            std::fs::OpenOptions::new()
                .append(true)
                .open(&self.output_path)
                .map_err(|e| {
                    if e.kind() == std::io::ErrorKind::PermissionDenied {
                        DownloadError::Io(std::io::Error::new(
                            std::io::ErrorKind::PermissionDenied,
                            format!(
                                "Permission denied accessing file {}: {}",
                                self.output_path.display(),
                                e
                            ),
                        ))
                    } else {
                        DownloadError::Io(e)
                    }
                })?
        } else {
            std::fs::File::create(&self.output_path).map_err(|e| {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    DownloadError::Io(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        format!(
                            "Permission denied creating file {}: {}",
                            self.output_path.display(),
                            e
                        ),
                    ))
                } else if e.kind() == std::io::ErrorKind::StorageFull {
                    DownloadError::Io(std::io::Error::new(
                        std::io::ErrorKind::StorageFull,
                        format!(
                            "No space left on device for file {}: {}",
                            self.output_path.display(),
                            e
                        ),
                    ))
                } else {
                    DownloadError::Io(e)
                }
            })?
        };

        use futures_util::StreamExt;
        while let Some(chunk_result) = stream.next().await {
            // Check for interrupt
            if is_interrupted() {
                return Err(DownloadError::Interrupted);
            }

            let chunk = chunk_result
                .map_err(|e| DownloadError::Network(format!("Download chunk error: {}", e)))?;

            file.write_all(&chunk).map_err(|e| {
                if e.kind() == std::io::ErrorKind::StorageFull {
                    DownloadError::Io(std::io::Error::new(
                        std::io::ErrorKind::StorageFull,
                        format!(
                            "No space left on device while writing to {}: {}",
                            self.output_path.display(),
                            e
                        ),
                    ))
                } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                    DownloadError::Io(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        format!(
                            "Permission denied writing to {}: {}",
                            self.output_path.display(),
                            e
                        ),
                    ))
                } else {
                    DownloadError::Io(e)
                }
            })?;

            bytes_downloaded += chunk.len() as u64;

            // Calculate download speed
            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 {
                bytes_downloaded as f64 / elapsed
            } else {
                0.0
            };

            // Report progress
            let progress = DownloadProgress::new(bytes_downloaded, total_bytes, speed);
            progress_callback(progress);
        }

        Ok(())
    }

    /// Download file with retry logic
    pub async fn download_with_retry<F>(&self, progress_callback: F) -> Result<(), DownloadError>
    where
        F: Fn(DownloadProgress) + Send + Sync,
    {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            // Check for interrupt
            if is_interrupted() {
                return Err(DownloadError::Interrupted);
            }

            if attempt > 0 {
                eprintln!(
                    "Retrying model download (attempt {} of {})...",
                    attempt, self.max_retries
                );
                tokio::time::sleep(self.retry_delay * attempt).await;
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

        Err(
            last_error.unwrap_or_else(|| DownloadError::MaxRetriesExceeded {
                retries: self.max_retries,
            }),
        )
    }
}

/// Calculate SHA-256 checksum of downloaded file
pub fn calculate_checksum(output_path: &PathBuf) -> Result<String, DownloadError> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let mut file = std::fs::File::open(output_path).map_err(DownloadError::Io)?;

    let mut hasher = Sha256::new();
    let mut buffer = [0u8; BUFFER_SIZE];

    loop {
        let n = file.read(&mut buffer).map_err(DownloadError::Io)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}
