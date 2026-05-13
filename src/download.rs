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
///
/// This function formats a download error message with optional manual download
/// instructions for critical errors that prevent automatic download.
///
/// # Arguments
///
/// * `error` - The download error that occurred
///
/// # Returns
///
/// Returns a formatted error message with manual download instructions for critical errors.
pub fn format_download_error(error: &DownloadError) -> String {
    let base_message = match error {
        DownloadError::Network(msg) => format!("Network error: {}", msg),
        DownloadError::Http(msg) => format!("HTTP error: {}", msg),
        DownloadError::Interrupted => "Download interrupted by user".to_string(),
        DownloadError::MaxRetriesExceeded { retries } => {
            format!("Download failed after {} retries", retries)
        }
        DownloadError::Timeout(msg) => format!("Timeout: {}", msg),
        DownloadError::Io(e) => format!("IO error: {}", e),
        DownloadError::Reqwest(e) => format!("HTTP client error: {}", e),
    };

    // Add manual download instructions for critical errors
    let instructions = match error {
        DownloadError::Network(_) | DownloadError::Http(_) | DownloadError::MaxRetriesExceeded { .. } => {
            Some(format!(
                "\n\n{}\n\nFor manual download instructions, run 'loom init --help' or visit the documentation.",
                get_manual_download_instructions_summary()
            ))
        }
        _ => None,
    };

    if let Some(instructions) = instructions {
        format!("{}{}", base_message, instructions)
    } else {
        base_message
    }
}

/// Get a summary of manual download instructions
///
/// This function returns a brief summary of manual download instructions
/// for inclusion in error messages.
///
/// # Returns
///
/// Returns a formatted summary of manual download instructions.
fn get_manual_download_instructions_summary() -> String {
    "Manual download is available as a fallback:\n\
     \n\
     1. Download the model from: https://huggingface.co/Qdrant/all-MiniLM-L6-v2-onnx/resolve/main/model.onnx\n\
     2. Place it in: .knowledge-loom-index/models/all-MiniLM-L6-v2.onnx\n\
     3. Run 'loom init' again to complete initialization".to_string()
}

/// Acquire file lock to prevent concurrent downloads
///
/// This function creates a lock file and acquires an exclusive lock on it.
/// If the lock file already exists and is locked, it returns an error.
///
/// # Arguments
///
/// * `lock_path` - The path to the lock file
///
/// # Returns
///
/// * `Ok(std::fs::File)` - If the lock was successfully acquired
/// * `Err(DownloadError)` - If the lock cannot be acquired
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
///
/// This function releases the lock on the lock file.
///
/// # Arguments
///
/// * `file` - The file handle to release the lock from
///
/// # Returns
///
/// * `Ok(())` - If the lock was successfully released
/// * `Err(DownloadError)` - If the lock cannot be released
pub fn release_lock(file: std::fs::File) -> Result<(), DownloadError> {
    file.unlock()
        .map_err(|e| DownloadError::Network(format!("Failed to release lock: {}", e)))?;
    Ok(())
}

/// Setup signal handler for Ctrl+C
///
/// This function sets up a signal handler for SIGINT (Ctrl+C) on Unix systems.
/// When the signal is received, it sets the INTERRUPTED flag.
///
/// # Returns
///
/// * `Ok(())` - If the signal handler was successfully set up
/// * `Err(DownloadError)` - If the signal handler cannot be set up
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
///
/// This function checks if the INTERRUPTED flag is set, indicating that
/// the download was interrupted by the user (Ctrl+C).
///
/// # Returns
///
/// * `true` - If the download was interrupted
/// * `false` - If the download was not interrupted
pub fn is_interrupted() -> bool {
    INTERRUPTED.load(Ordering::SeqCst)
}

/// Reset interrupt flag
///
/// This function resets the INTERRUPTED flag to false.
/// This should be called before starting a new download.
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
    ///
    /// This function downloads a file from the configured URL with progress tracking.
    /// It supports HTTP Range requests for resuming interrupted downloads.
    /// If a partial file exists at the output path, it will resume the download
    /// from the last byte downloaded.
    ///
    /// # Arguments
    ///
    /// * `progress_callback` - A callback function that receives download progress updates
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the download completed successfully
    /// * `Err(DownloadError)` - If the download failed
    ///
    /// # HTTP Range Support
    ///
    /// This function supports HTTP Range requests for resuming interrupted downloads.
    /// If a partial file exists at the output path, it will:
    /// 1. Check the file size to determine the number of bytes already downloaded
    /// 2. Add a Range header to the HTTP request: `Range: bytes={start_byte}-`
    /// 3. Append the downloaded data to the existing file
    ///
    /// If the server doesn't support Range requests, it will return a 200 status
    /// instead of 206 (Partial Content), and the download will start from the beginning.
    pub async fn download<F>(&self, progress_callback: F) -> Result<(), DownloadError>
    where
        F: Fn(DownloadProgress) + Send + Sync,
    {
        // Check for interrupt
        if is_interrupted() {
            return Err(DownloadError::Interrupted);
        }

        // Check if partial file exists for resume
        // HTTP Range request support: resume from last byte downloaded
        let start_byte = if self.output_path.exists() {
            let metadata = std::fs::metadata(&self.output_path).map_err(DownloadError::Io)?;
            metadata.len()
        } else {
            0
        };

        // Build request with Range header if resuming
        let mut request = self.client.get(&self.url);
        if start_byte > 0 {
            // Add Range header for HTTP Range request support
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
        // HTTP Range requests return 206 (Partial Content) for successful resume
        // Regular downloads return 200 (OK)
        let status = response.status();
        if !status.is_success() && status != reqwest::StatusCode::PARTIAL_CONTENT {
            return Err(DownloadError::Http(format!("HTTP error: {}", status)));
        }

        // Get total bytes from Content-Range or Content-Length
        // For HTTP Range requests (206), parse Content-Range header: "bytes 0-100/200" or "bytes 100-200/*"
        // For regular downloads (200), use Content-Length header
        let total_bytes = if status == reqwest::StatusCode::PARTIAL_CONTENT {
            // Parse Content-Range header to extract total file size
            // Format: "bytes {start}-{end}/{total}" or "bytes {start}-{end}/*"
            if let Some(content_range) = response.headers().get("Content-Range") {
                let range_str = content_range.to_str().unwrap_or("");
                // Extract total size after the slash
                if let Some(slash_pos) = range_str.find('/') {
                    let total_str = &range_str[slash_pos + 1..];
                    // "*" indicates unknown total size, use 0 as fallback
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
            // For regular downloads, use Content-Length header
            response
                .content_length()
                .ok_or_else(|| DownloadError::Network("Missing content length".to_string()))?
        };

        // Initialize download tracking
        let mut bytes_downloaded = start_byte;
        let start_time = std::time::Instant::now();

        // Get byte stream from response
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
        // Process download chunks
        while let Some(chunk_result) = stream.next().await {
            // Check for interrupt signal (Ctrl+C)
            if is_interrupted() {
                return Err(DownloadError::Interrupted);
            }

            // Get chunk from stream, handling network errors
            let chunk = chunk_result
                .map_err(|e| DownloadError::Network(format!("Download chunk error: {}", e)))?;

            // Write chunk to file with enhanced error handling
            // Check for disk full and permission denied errors
            file.write_all(&chunk).map_err(|e| {
                if e.kind() == std::io::ErrorKind::StorageFull {
                    // Disk full error - provide clear error message
                    DownloadError::Io(std::io::Error::new(
                        std::io::ErrorKind::StorageFull,
                        format!(
                            "No space left on device while writing to {}: {}",
                            self.output_path.display(),
                            e
                        ),
                    ))
                } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                    // Permission denied error - provide clear error message
                    DownloadError::Io(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        format!(
                            "Permission denied writing to {}: {}",
                            self.output_path.display(),
                            e
                        ),
                    ))
                } else {
                    // Other IO errors - pass through
                    DownloadError::Io(e)
                }
            })?;

            // Update download progress
            bytes_downloaded += chunk.len() as u64;

            // Calculate download speed (bytes per second)
            // Avoid division by zero by checking elapsed time
            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 {
                bytes_downloaded as f64 / elapsed
            } else {
                0.0
            };

            // Report progress to callback
            // Progress includes: bytes downloaded, total bytes, and current speed
            let progress = DownloadProgress::new(bytes_downloaded, total_bytes, speed);
            progress_callback(progress);
        }

        Ok(())
    }

    /// Download file with retry logic
    ///
    /// This function implements exponential backoff retry logic for handling
    /// transient network failures. It will retry up to max_retries times with
    /// increasing delays between attempts.
    pub async fn download_with_retry<F>(&self, progress_callback: F) -> Result<(), DownloadError>
    where
        F: Fn(DownloadProgress) + Send + Sync,
    {
        let mut last_error = None;

        // Retry loop with exponential backoff
        for attempt in 0..=self.max_retries {
            // Check for interrupt signal (Ctrl+C)
            if is_interrupted() {
                return Err(DownloadError::Interrupted);
            }

            // Add delay before retry (exponential backoff)
            // Delay increases with each attempt: 1s, 2s, 4s, etc.
            if attempt > 0 {
                eprintln!(
                    "Retrying model download (attempt {} of {})...",
                    attempt, self.max_retries
                );
                tokio::time::sleep(self.retry_delay * attempt).await;
            }

            // Attempt download
            match self.download(&progress_callback).await {
                // Success - return immediately
                Ok(()) => return Ok(()),
                // Failure - store error and continue to next attempt
                Err(e) => {
                    last_error = Some(e);
                    // Continue retrying if we haven't exhausted retries
                    if attempt < self.max_retries {
                        continue;
                    }
                }
            }
        }

        // All retries exhausted - return the last error
        Err(last_error.unwrap_or_else(|| {
            DownloadError::Network("Download failed: unknown error".to_string())
        }))
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
