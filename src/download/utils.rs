// Shared download utilities for Knowledge Loom
// Extracted from download.rs to prevent code duplication

#![allow(dead_code)]

use crate::download::{DownloadManager, DownloadProgress, MAX_RETRIES, RETRY_DELAY, TIMEOUT};
use crate::model::DownloadError;
use std::path::Path;
use std::time::Duration;

/// Download a file with retry logic and progress tracking
///
/// This is a simplified wrapper around DownloadManager for common use cases.
///
/// # Arguments
///
/// * `url` - The URL to download from
/// * `output_path` - Where to save the downloaded file
/// * `progress_callback` - Optional callback for progress updates
///
/// # Returns
///
/// * `Ok(())` - If download completed successfully
/// * `Err(DownloadError)` - If download failed
pub async fn download_with_retry(
    url: &str,
    output_path: &Path,
    progress_callback: Option<impl Fn(DownloadProgress) + Send + Sync>,
) -> Result<(), DownloadError> {
    let manager = DownloadManager::new(url.to_string(), output_path.to_path_buf())?
        .with_retries(MAX_RETRIES)
        .with_retry_delay(Duration::from_secs(RETRY_DELAY))
        .with_timeout(Duration::from_secs(TIMEOUT));

    match progress_callback {
        Some(callback) => manager.download(callback).await,
        None => manager.download(|_| {}).await,
    }
}

/// Check available disk space before download
///
/// # Arguments
///
/// * `output_path` - Where the file will be saved
/// * `required_bytes` - Estimated file size in bytes
///
/// # Returns
///
/// * `Ok(())` - If sufficient disk space is available
/// * `Err(DownloadError)` - If disk space is insufficient
#[cfg(unix)]
pub fn check_disk_space(output_path: &Path, required_bytes: u64) -> Result<(), DownloadError> {
    use nix::sys::statvfs::statvfs;

    let dir = output_path.parent().unwrap_or_else(|| Path::new("."));

    let stat = statvfs(dir).map_err(|e| {
        DownloadError::Io(std::io::Error::other(format!(
            "Failed to get disk space: {}",
            e
        )))
    })?;

    let available_bytes = stat.blocks_available() as u64 * stat.block_size() as u64;

    if available_bytes < required_bytes {
        let required_mb = required_bytes / 1_048_576;
        let available_mb = available_bytes / 1_048_576;
        Err(DownloadError::Network(format!(
            "Insufficient disk space: required {} MB, available {} MB",
            required_mb, available_mb
        )))
    } else {
        Ok(())
    }
}

/// Calculate SHA-256 checksum of data
///
/// # Arguments
///
/// * `data` - Bytes to calculate checksum for
///
/// # Returns
///
/// Hexadecimal string representation of the checksum
pub fn calculate_checksum(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let checksum = Sha256::digest(data);
    format!("{:x}", checksum)
}

/// Validate checksum against expected value
///
/// # Arguments
///
/// * `data` - Bytes to validate
/// * `expected` - Expected checksum hex string
///
/// # Returns
///
/// * `Ok(())` - If checksum matches
/// * `Err(DownloadError)` - If checksum doesn't match
pub fn validate_checksum(data: &[u8], expected: &str) -> Result<(), DownloadError> {
    let actual = calculate_checksum(data);
    if actual != expected {
        Err(DownloadError::ChecksumMismatch {
            expected: expected.to_string(),
            actual,
        })
    } else {
        Ok(())
    }
}
