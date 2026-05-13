# Data Model: Init-Time Model Download

**Feature**: Init-Time Model Download
**Date**: 2026-05-12
**Status**: Complete

## Overview

This document defines the data model for init-time model download, including entities, fields, relationships, validation rules, and state transitions.

## Entities

### 1. Model Download State

**Purpose**: Represents the current status of model download, including progress and error information.

**Storage**: File-based JSON at `{KB_ROOT}/.knowledge-loom-index/models/download-state.json`

**Fields**:

| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| `status` | String | Current download status | Must be one of: `not_started`, `in_progress`, `completed`, `failed` |
| `progress_percentage` | Number | Download progress (0-100) | Must be between 0 and 100, inclusive |
| `bytes_downloaded` | Number | Bytes downloaded so far | Must be non-negative integer |
| `total_bytes` | Number | Total bytes to download | Must be positive integer |
| `download_speed` | Number | Current download speed in bytes/second | Must be non-negative number |
| `error_message` | String | Error message if status is `failed` | Required if status is `failed`, optional otherwise |
| `last_updated` | String | ISO 8601 timestamp of last update | Must be valid ISO 8601 timestamp |
| `model_name` | String | Name of the model being downloaded | Must be non-empty string |
| `model_version` | String | Version of the model being downloaded | Must be non-empty string |

**State Transitions**:

```
not_started → in_progress → completed
                ↓
              failed
```

**Transition Rules**:
- `not_started` → `in_progress`: Download starts
- `in_progress` → `completed`: Download completes successfully
- `in_progress` → `failed`: Download fails (network error, disk full, etc.)
- `failed` → `in_progress`: Retry download
- `completed` → `in_progress`: Re-download (corrupted model, wrong version)

**Example**:

```json
{
  "status": "in_progress",
  "progress_percentage": 45,
  "bytes_downloaded": 54000000,
  "total_bytes": 120000000,
  "download_speed": 2500000,
  "error_message": null,
  "last_updated": "2026-05-12T14:30:45Z",
  "model_name": "all-MiniLM-L6-v2",
  "model_version": "1.0.0"
}
```

### 2. Model File

**Purpose**: Represents the downloaded model file with metadata.

**Storage**: File-based at `{KB_ROOT}/.knowledge-loom-index/models/all-MiniLM-L6-v2.onnx`

**Metadata Storage**: File-based JSON at `{KB_ROOT}/.knowledge-loom-index/models/all-MiniLM-L6-v2.json`

**Fields**:

| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| `model_name` | String | Name of the model | Must be `all-MiniLM-L6-v2` |
| `model_version` | String | Version of the model | Must be non-empty string |
| `file_path` | String | Path to the model file | Must be valid file path |
| `file_size` | Number | Size of the model file in bytes | Must be positive integer |
| `sha256_checksum` | String | SHA-256 checksum of the model file | Must be 64-character hexadecimal string |
| `download_timestamp` | String | ISO 8601 timestamp of download | Must be valid ISO 8601 timestamp |
| `download_url` | String | URL from which the model was downloaded | Must be valid HTTPS URL |
| `validated` | Boolean | Whether the model has been validated | Must be `true` for model to be usable |

**Validation Rules**:
- File size must match expected size (~120MB)
- SHA-256 checksum must match expected checksum
- File must be valid ONNX format
- File must be readable and accessible

**Example**:

```json
{
  "model_name": "all-MiniLM-L6-v2",
  "model_version": "1.0.0",
  "file_path": "/path/to/.knowledge-loom-index/models/all-MiniLM-L6-v2.onnx",
  "file_size": 120000000,
  "sha256_checksum": "a1b2c3d4e5f6...64-character-hex-string",
  "download_timestamp": "2026-05-12T14:30:45Z",
  "download_url": "https://huggingface.co/Qdrant/all-MiniLM-L6-v2-onnx/resolve/main/model.onnx",
  "validated": true
}
```

### 3. Download Progress Information

**Purpose**: Real-time information about download progress for display to users.

**Storage**: In-memory during download, persisted to Model Download State

**Fields**:

| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| `bytes_downloaded` | Number | Bytes downloaded so far | Must be non-negative integer |
| `total_bytes` | Number | Total bytes to download | Must be positive integer |
| `percentage_complete` | Number | Download progress (0-100) | Must be between 0 and 100, inclusive |
| `download_speed` | Number | Current download speed in bytes/second | Must be non-negative number |
| `estimated_time_remaining` | Number | Estimated time remaining in seconds | Must be non-negative number |
| `elapsed_time` | Number | Elapsed time in seconds | Must be non-negative number |

**Display Format**:

Structured plain text format: `"Downloading model: {percentage}% ({downloaded_mb}MB/{total_mb}MB) - {speed_mb}MB/s - {remaining}s remaining"`

**Example**:

```
Downloading model: 45% (54MB/120MB) - 2.5MB/s - 26s remaining
```

**Completion Format**:

```
Model download complete: 100% (120MB/120MB) - 2.5MB/s - 48s total
```

**Error Format**:

```
Model download failed: Network error: Connection timeout
```

### 4. Signal Handler

**Purpose**: Handles Ctrl+C signal for graceful shutdown and cleanup.

**Storage**: In-memory signal handler registration

**Fields**:

| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| `signal_type` | Signal | Signal type being handled | Must be SIGINT (Ctrl+C) |
| `cleanup_action` | String | Action to take on signal | Must be one of: `cleanup_partial_files`, `preserve_state`, `exit` |
| `state_preserved` | Boolean | Whether download state was preserved | Must be true if state was saved |
| `cleanup_time_ms` | Number | Time taken for cleanup in milliseconds | Must be <500ms |

**Behavior**:
- Catch SIGINT signal (Ctrl+C)
- Clean up partial files
- Preserve download state for resume
- Exit gracefully

### 5. Proxy Configuration

**Purpose**: System proxy environment variables for corporate environments.

**Storage**: Environment variables (HTTP_PROXY, HTTPS_PROXY, NO_PROXY)

**Fields**:

| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| `http_proxy` | String | HTTP proxy URL | Must be valid URL if present |
| `https_proxy` | String | HTTPS proxy URL | Must be valid URL if present |
| `no_proxy` | String | Comma-separated list of bypass rules | Must be valid hostnames/IPs if present |
| `proxy_type` | String | Type of proxy (http, https, socks) | Must be one of: `http`, `https`, `socks` |

**Behavior**:
- Respect system proxy environment variables
- Apply proxy to all HTTP/HTTPS requests
- Bypass local addresses per NO_PROXY rules
- Fallback to direct connection if proxy not configured

### 6. Version Metadata

**Purpose**: Model version information for detecting version mismatches.

**Storage**: Part of Model File metadata JSON

**Fields**:

| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| `expected_version` | String | Expected model version | Must be `1.0.0` |
| `actual_version` | String | Actual model version from metadata | Must be non-empty string |
| `version_match` | Boolean | Whether versions match | Must be true for model to be usable |
| `mismatch_action` | String | Action to take on mismatch | Must be one of: `prompt_re_download`, `auto_re_download`, `block` |

**Behavior**:
- Compare actual version with expected version
- Prompt user for re-download on mismatch
- Block model usage if version mismatch detected
- Update expected version when model is updated

## Relationships

### Entity Relationships

```
Model Download State
    ↓ (references)
Model File
    ↓ (has metadata)
Download Progress Information
    ↓ (managed by)
Signal Handler
    ↓ (uses)
Proxy Configuration
    ↓ (includes)
Version Metadata
```

**Relationship Details**:

1. **Model Download State → Model File**:
   - One-to-one relationship
   - Model Download State references the model being downloaded
   - Model File exists only after download completes

2. **Model File → Download Progress Information**:
   - One-to-many relationship (historical progress)
   - Model File has current progress during download
   - Progress information is transient (not persisted after completion)

3. **Model Download State → Download Progress Information**:
   - One-to-one relationship (current progress)
   - Model Download State contains current progress information
   - Progress is updated during download and persisted to state

4. **Download Manager → Signal Handler**:
   - One-to-one relationship
   - Download Manager manages signal handler registration
   - Signal handler triggers cleanup on Ctrl+C

5. **Download Manager → Proxy Configuration**:
   - One-to-one relationship
   - Download Manager uses proxy configuration for HTTP requests
   - Proxy configuration is read from environment variables

6. **Model Manager → Version Metadata**:
   - One-to-one relationship
   - Model Manager validates version metadata
   - Version metadata is stored in model file metadata

## File System Structure

```
{KB_ROOT}/.knowledge-loom-index/models/
├── all-MiniLM-L6-v2.onnx              # Model file
├── all-MiniLM-L6-v2.json              # Model metadata
├── download-state.json                # Download state
└── .download.lock                    # Download lock file
```

**File Descriptions**:

- `all-MiniLM-L6-v2.onnx`: The actual model file in ONNX format
- `all-MiniLM-L6-v2.json`: Model metadata (checksum, version, download timestamp)
- `download-state.json`: Current download state and progress
- `.download.lock`: File lock to prevent concurrent downloads

## Validation Rules

### Model Download State Validation

1. **Status Validation**:
   - Must be one of: `not_started`, `in_progress`, `completed`, `failed`
   - Default: `not_started`

2. **Progress Validation**:
   - `progress_percentage` must be between 0 and 100, inclusive
   - `bytes_downloaded` must be ≤ `total_bytes`
   - `download_speed` must be non-negative

3. **Error Message Validation**:
   - Required if `status` is `failed`
   - Optional otherwise
   - Must be non-empty string if present

4. **Timestamp Validation**:
   - `last_updated` must be valid ISO 8601 timestamp
   - Must be updated on every state change

### Model File Validation

1. **File Existence Validation**:
   - File must exist at specified path
   - File must be readable

2. **File Size Validation**:
   - File size must match expected size (~120MB)
   - Tolerance: ±5% (to account for minor variations)

3. **Checksum Validation**:
   - SHA-256 checksum must match expected checksum
   - Checksum must be 64-character hexadecimal string

4. **Format Validation**:
   - File must be valid ONNX format
   - File must be loadable by fastembed

5. **Metadata Validation**:
   - `model_name` must be `all-MiniLM-L6-v2`
   - `model_version` must be non-empty string
   - `validated` must be `true` for model to be usable

### Download Progress Information Validation

1. **Progress Validation**:
   - `bytes_downloaded` must be ≤ `total_bytes`
   - `percentage_complete` must be between 0 and 100, inclusive
   - `download_speed` must be non-negative

2. **Time Validation**:
   - `estimated_time_remaining` must be non-negative
   - `elapsed_time` must be non-negative

3. **Consistency Validation**:
   - `percentage_complete` must equal `(bytes_downloaded / total_bytes) * 100`
   - `estimated_time_remaining` must equal `(total_bytes - bytes_downloaded) / download_speed`

## Error Handling

### Validation Errors

1. **Invalid State**:
   - Error: `"Invalid download state: {details}"`
   - Action: Reset state to `not_started` or `failed`

2. **Corrupted Model File**:
   - Error: `"Model validation failed: Checksum mismatch"`
   - Action: Delete corrupted file, trigger re-download

3. **Missing Metadata**:
   - Error: `"Model metadata not found: {path}"`
   - Action: Re-generate metadata from model file

4. **Lock Timeout**:
   - Error: `"Model download is already in progress"`
   - Action: Wait for current download to complete

### File System Errors

1. **Permission Denied**:
   - Error: `"Permission denied: Cannot write to {path}"`
   - Action: Check file permissions, retry with correct permissions

2. **Disk Full**:
   - Error: `"Insufficient disk space: {required_mb}MB required, {available_mb}MB available"`
   - Action: Free up disk space, retry download

3. **Path Not Found**:
   - Error: `"Path not found: {path}"`
   - Action: Create directory structure, retry

## Performance Considerations

### State Persistence

- **Atomic Writes**: Write to temp file, then rename
- **Locking**: File locking to prevent concurrent access
- **Caching**: In-memory cache for frequent reads
- **Batch Updates**: Persist state every 1 second during download

### Validation Performance

- **Streaming Checksum**: Calculate checksum during download
- **Parallel Validation**: Validate size and checksum in parallel
- **Early Exit**: Fail fast on validation errors

### Progress Updates

- **Throttling**: Update progress at most once per second
- **Batching**: Batch progress updates to reduce I/O
- **Efficient Formatting**: Pre-allocate strings for progress display

## Security Considerations

### File System Security

- **Path Validation**: Prevent path traversal attacks
- **Permission Checks**: Verify write permissions before download
- **Atomic Writes**: Use temp file + rename for atomic updates
- **Cleanup**: Delete partial files on failure

### Data Integrity

- **Checksum Validation**: SHA-256 checksum ensures integrity
- **Size Validation**: File size check ensures completeness
- **Format Validation**: ONNX format check ensures correctness

### Error Message Security

- **No Sensitive Data**: Don't expose paths, tokens, or secrets
- **Sanitized Output**: Escape user-provided data in error messages
- **Rate Limiting**: Prevent error message spamming

## Testing Considerations

### Unit Tests

- State validation (all fields, all states)
- State transitions (all valid transitions)
- Model file validation (checksum, size, format)
- Progress information validation (all fields)
- Error handling (all error scenarios)

### Integration Tests

- State persistence and recovery
- Model file validation end-to-end
- Progress display formatting
- Error message generation
- File locking and concurrency

### Edge Cases

- Corrupted state file
- Missing model file
- Invalid checksum
- Wrong file size
- Invalid format
- Concurrent access
- Interrupted download
- Partial download

## References

- [Feature specification](./spec.md)
- [Research document](./research.md)
- [Knowledge Loom constitution](../../.specify/memory/constitution.md)
