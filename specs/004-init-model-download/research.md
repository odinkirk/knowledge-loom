# Research: Init-Time Model Download

**Feature**: Init-Time Model Download
**Date**: 2026-05-12
**Status**: Complete

## Overview

This research document captures the technical decisions and rationale for implementing init-time model download with structured plain text progress indicators, graceful error handling, and state management.

## Technical Decisions

### 1. Model Download URL

**Decision**: Hardcoded in binary (all-MiniLM-L6-v2 from Qdrant)

**Rationale**:
- Simplicity: Single model version reduces configuration complexity
- Reliability: Hardcoded URL ensures all users use the same validated model
- Security: Prevents users from downloading untrusted models
- Maintenance: Easier to update model version in one place

**Alternatives Considered**:
- Configurable via environment variable: More flexible but adds complexity
- Discovered from remote manifest: Dynamic but requires network and adds failure modes
- User-provided URL: Maximum flexibility but error-prone and insecure

**Implementation Notes**:
- URL: `https://huggingface.co/Qdrant/all-MiniLM-L6-v2-onnx/resolve/main/model.onnx`
- Model name: `all-MiniLM-L6-v2`
- Expected file size: ~120MB
- SHA-256 checksum: [to be determined during implementation]

### 2. Model Storage Location

**Decision**: KB_ROOT/.knowledge-loom-index/models/

**Rationale**:
- Consistency: Follows existing index pattern (KB_ROOT/.knowledge-loom-index/)
- Isolation: Per-knowledge-base storage prevents conflicts
- Discoverability: Easy to find and manage model files
- Cleanup: Can be cleaned up with index cleanup

**Alternatives Considered**:
- ~/.cache/loom/models/: System-wide cache but breaks per-knowledge-base isolation
- KB_ROOT/models/: Top-level directory but less organized
- Temporary directory with symlink: Adds complexity without clear benefit

**Implementation Notes**:
- Directory structure: `{KB_ROOT}/.knowledge-loom-index/models/`
- Model file: `{KB_ROOT}/.knowledge-loom-index/models/all-MiniLM-L6-v2.onnx`
- Metadata file: `{KB_ROOT}/.knowledge-loom-index/models/all-MiniLM-L6-v2.json`
- State file: `{KB_ROOT}/.knowledge-loom-index/models/download-state.json`

### 3. Retry Strategy

**Decision**: Exponential backoff with 3 retries (1s, 2s, 4s delays)

**Rationale**:
- Industry standard: Exponential backoff is the recommended approach for network operations
- Resilience: Balances retry attempts with reasonable completion time
- Server-friendly: Avoids overwhelming the server with immediate retries
- User experience: Provides reasonable wait time without excessive delays

**Alternatives Considered**:
- Fixed 1-second delay with 5 retries: Simpler but less resilient to transient failures
- No automatic retry: Requires manual intervention for all failures
- Immediate retry up to 3 times: Fastest but can overwhelm server

**Implementation Notes**:
- Retry delays: 1s, 2s, 4s (exponential backoff)
- Max retries: 3
- Total max wait time: 7s (1s + 2s + 4s)
- Retry on: Network errors, 5xx server errors, timeout
- No retry on: 4xx client errors, authentication errors

### 4. Checksum Algorithm

**Decision**: SHA-256 checksum

**Rationale**:
- Industry standard: SHA-256 is widely used for file integrity verification
- Strong security: Provides strong collision resistance
- Performance: Efficient for files under 500MB
- Compatibility: Widely supported across platforms and libraries

**Alternatives Considered**:
- MD5 checksum: Faster but weaker security (collision attacks)
- CRC32 checksum: Fastest but minimal security (not cryptographically secure)
- No checksum validation: Fastest but no integrity verification

**Implementation Notes**:
- Algorithm: SHA-256
- Library: `sha2` crate
- Performance target: <5s for 500MB model
- Checksum storage: In model metadata JSON file
- Validation: After download completes, before making model available

### 5. Progress Indicator Format

**Decision**: Structured plain text format (human-readable and AI-parseable)

**Rationale**:
- Dual usability: Works for both humans and AI assistants
- Parseability: Easy to parse with regex or string matching
- Compatibility: Works in all terminal environments (interactive, non-interactive, CI/CD)
- Simplicity: No ANSI escape codes or cursor positioning

**Alternatives Considered**:
- Terminal progress bar with fallback: Best UX but hard for AI to parse
- JSON output only: Machine-readable but not human-friendly
- Plain text percentage only: Simple but less informative
- No progress display: Fastest but confusing

**Implementation Notes**:
- Format: `"Downloading model: {percentage}% ({downloaded_mb}MB/{total_mb}MB) - {speed_mb}MB/s - {remaining}s remaining"`
- Update frequency: At least once per second
- Example: `"Downloading model: 45% (45MB/100MB) - 2.3MB/s - 24s remaining"`
- Completion: `"Model download complete: 100% (100MB/100MB) - 2.5MB/s - 40s total"`
- Error: `"Model download failed: {error message}"`

### 6. State Management

**Decision**: File-based persistence for download state

**Rationale**:
- Simplicity: No database or complex storage required
- Reliability: File system provides atomic writes and durability
- Portability: Works across all platforms without additional dependencies
- Debugging: Easy to inspect and debug state files

**Alternatives Considered**:
- SQLite database: More robust but adds complexity
- In-memory only: Lost on restart, no persistence
- Environment variables: Limited capacity, not persistent

**Implementation Notes**:
- State file: `{KB_ROOT}/.knowledge-loom-index/models/download-state.json`
- States: `not_started`, `in_progress`, `completed`, `failed`
- Fields: `status`, `progress_percentage`, `bytes_downloaded`, `total_bytes`, `download_speed`, `error_message`, `last_updated`
- Atomic writes: Write to temp file, then rename
- Locking: File locking to prevent concurrent access

### 7. Concurrency Control

**Decision**: File locking to prevent concurrent downloads

**Rationale**:
- Simplicity: File locking is straightforward and reliable
- Cross-platform: Works on Linux, macOS, Windows
- Safety: Prevents race conditions and corrupted state
- User experience: Provides clear error message when download is in progress

**Alternatives Considered**:
- In-memory mutex: Only works within single process
- Database locking: Overkill for this use case
- No locking: Allows concurrent downloads but risks corruption

**Implementation Notes**:
- Lock file: `{KB_ROOT}/.knowledge-loom-index/models/.download.lock`
- Lock type: Exclusive file lock
- Timeout: Fail immediately if lock is held
- Error message: `"Model download is already in progress. Please wait for the current download to complete."`

### 8. Error Handling

**Decision**: Clear, actionable error messages with specific guidance

**Rationale**:
- User experience: Users can resolve most errors without support
- Debugging: Specific error messages help diagnose issues
- AI assistance: Structured error messages help AI assistants provide guidance

**Alternatives Considered**:
- Generic error messages: Simpler but less helpful
- No error messages: Confusing and frustrating
- Stack traces: Too technical for most users

**Implementation Notes**:
- Network failure: `"Network error: {details}. Please check your internet connection and try again."`
- Disk full: `"Insufficient disk space: {required_mb}MB required, {available_mb}MB available. Please free up space and try again."`
- Permission denied: `"Permission denied: Cannot write to {path}. Please check file permissions and try again."`
- Checksum mismatch: `"Model validation failed: Checksum mismatch. The downloaded file may be corrupted. Please try downloading again."`
- Timeout: `"Download timeout: The download took too long. Please check your internet connection and try again."`

### 9. HTTP Client

**Decision**: reqwest crate for HTTP operations

**Rationale**:
- Async support: Built-in async/await with tokio
- Features: Supports HTTP/2, TLS, proxies, redirects
- Reliability: Well-maintained and widely used
- Performance: Efficient and performant

**Alternatives Considered**:
- ureq: Simpler but synchronous only
- curl: Powerful but complex API
- hyper: Low-level, more complex

**Implementation Notes**:
- Library: `reqwest` crate
- Features: `rustls-tls` (TLS support), `gzip` (compression)
- Timeout: 30 seconds per request
- User agent: `"Knowledge Loom/{version}"`
- Proxy support: Respect system proxy environment variables
- Range requests: Support for resuming downloads

### 10. Model Validation

**Decision**: Multi-stage validation (checksum, size, format)

**Rationale**:
- Security: SHA-256 checksum ensures file integrity
- Completeness: File size check ensures complete download
- Correctness: Format validation ensures model is usable
- User experience: Early detection of corrupted or incomplete downloads

**Alternatives Considered**:
- Checksum only: Less comprehensive validation
- Size only: Doesn't detect corruption
- No validation: Risk of corrupted or incomplete models

**Implementation Notes**:
- Stage 1: File size check (expected size vs actual size)
- Stage 2: SHA-256 checksum validation
- Stage 3: Format validation (ONNX format check)
- Performance target: <5s for 500MB model
- Failure handling: Delete corrupted file, provide clear error message

### 11. Output Conventions

**Decision**: Use `println!` ONLY in CLI context for user-facing progress indicators, use `eprintln!` for all debug/logging output

**Rationale**:
- MCP server stability: `println!` dirties stdio which causes the MCP server to panic
- User experience: Progress indicators need to be visible in CLI context
- Debug safety: All debug output must use `eprintln!` to avoid stdio pollution
- Context separation: Clear distinction between user-facing and debug output

**Alternatives Considered**:
- Always use `println!`: Would break MCP server stability
- Always use `eprintln!`: Progress indicators would be invisible to users
- Logging framework: Overkill for simple CLI tool

**Implementation Notes**:
- CLI context: Use `println!` for progress indicators
- MCP server context: Use `eprintln!` for all output
- Debug output: Always use `eprintln!`
- Progress format: Structured plain text (human-readable and AI-parseable)
- Error messages: Use `eprintln!` for debug, `println!` for user-facing errors

### 12. Interrupted Downloads (HTTP Range Requests)

**Decision**: Support resuming from last byte downloaded using HTTP Range requests

**Rationale**:
- User experience: Users can resume interrupted downloads without starting over
- Bandwidth efficiency: Only download remaining bytes
- Reliability: Handles network interruptions gracefully
- Industry standard: HTTP Range requests are widely supported

**Alternatives Considered**:
- Always restart download: Wastes bandwidth and time
- No resume support: Poor user experience for large files
- Custom resume protocol: Complex and non-standard

**Implementation Notes**:
- HTTP Range header: `Range: bytes={start}-`
- Server support: Verify server supports Range requests
- State persistence: Track bytes downloaded in download state
- Resume logic: Check file size, request remaining bytes
- Fallback: If Range not supported, restart download

### 13. Proxy Configuration Support

**Decision**: Respect system proxy environment variables (HTTP_PROXY, HTTPS_PROXY, NO_PROXY)

**Rationale**:
- Corporate environments: Many organizations require proxy configuration
- User convenience: Automatic proxy detection without manual configuration
- Industry standard: Standard proxy environment variables are widely used
- Flexibility: Supports bypass rules via NO_PROXY

**Alternatives Considered**:
- No proxy support: Fails in corporate environments
- Manual proxy configuration: Adds complexity and user burden
- Custom proxy format: Non-standard and confusing

**Implementation Notes**:
- Environment variables: HTTP_PROXY, HTTPS_PROXY, NO_PROXY
- Proxy types: HTTP, HTTPS, SOCKS
- Bypass rules: NO_PROXY for local addresses
- reqwest support: Built-in proxy support via environment variables
- Fallback: Direct connection if proxy not configured

### 14. Model Version Mismatch Detection

**Decision**: Detect version mismatch by comparing model metadata version with expected version

**Rationale**:
- Safety: Prevents using incompatible model versions
- User experience: Clear error message when version mismatch detected
- Maintenance: Easy to update expected version in one place
- Reliability: Metadata-based detection is reliable

**Alternatives Considered**:
- No version checking: Risk of using incompatible models
- File size only: Less reliable for version detection
- User manual verification: Error-prone and user burden

**Implementation Notes**:
- Expected version: Hardcoded in binary (all-MiniLM-L6-v2)
- Metadata version: Stored in model metadata JSON file
- Comparison: Compare metadata version with expected version
- Error handling: Prompt user for re-download on mismatch
- Validation: Check version before using model

### 15. Ctrl+C Signal Handling

**Decision**: Catch SIGINT signal, clean up partial files, preserve download state for resume

**Rationale**:
- User experience: Graceful shutdown instead of abrupt termination
- Data integrity: Clean up partial files to prevent corruption
- Resume capability: Preserve download state for later resume
- Industry standard: Standard signal handling pattern

**Alternatives Considered**:
- No signal handling: Abruptive and confusing for users
- Ignore signal: Poor user experience and confusing behavior
- Immediate exit: No cleanup or state preservation

**Implementation Notes**:
- Signal: SIGINT (Ctrl+C)
- Cleanup: Delete partial files, preserve download state
- State preservation: Save current progress to download state file
- Resume capability: HTTP Range requests for resume
- Performance target: <500ms cleanup time
- Library: `signal-hook` crate for cross-platform signal handling

## Dependencies

### New Dependencies

- `reqwest`: HTTP client for model download
- `sha2`: SHA-256 checksum calculation
- `tokio`: Async runtime (already used)
- `anyhow`: Error handling (already used)
- `thiserror`: Error types (already used)
- `fastembed`: Model loading (already used)
- `signal-hook`: Cross-platform signal handling for Ctrl+C

### Existing Dependencies

- `tokio`: Async runtime
- `anyhow`: Error handling
- `thiserror`: Error types
- `fastembed`: Model loading and management

## Performance Considerations

### Download Performance

- Target: <5min init completion on 10 Mbps connection
- Model size: ~120MB
- Expected download time: ~96s on 10 Mbps connection
- Buffer size: 8KB chunks for streaming download
- Progress updates: At least once per second

### Validation Performance

- Target: <5s SHA-256 validation for 500MB model
- Actual model size: ~120MB
- Expected validation time: ~1.2s for 120MB model
- Streaming validation: Calculate checksum during download

### State Check Performance

- Target: <10ms download state checks
- Implementation: File-based JSON with atomic reads
- Caching: In-memory cache for frequent checks
- Locking: Non-blocking lock checks

### HTTP Range Request Performance

- Target: <1s HTTP Range request resume
- Implementation: Check file size, request remaining bytes
- Fallback: Restart download if Range not supported
- Performance: Minimal overhead for resume capability

### Signal Handling Performance

- Target: <500ms Ctrl+C cleanup
- Implementation: Signal handler with cleanup logic
- State preservation: Save download state before exit
- Performance: Fast cleanup to prevent data loss

## Security Considerations

### Model Download Security

- HTTPS only: Enforce TLS for model download
- Certificate validation: Verify server certificate
- Checksum validation: SHA-256 checksum ensures integrity
- URL validation: Hardcoded URL prevents malicious downloads

### File System Security

- Permission checks: Verify write permissions before download
- Path validation: Prevent path traversal attacks
- Atomic writes: Use temp file + rename for atomic updates
- Cleanup: Delete partial files on failure

### Error Message Security

- No sensitive data: Don't expose paths, tokens, or secrets
- Sanitized output: Escape user-provided data in error messages
- Rate limiting: Prevent error message spamming

## Testing Strategy

### Unit Tests

- Model download logic (success, failure, retry)
- Checksum calculation and validation
- State management (persistence, recovery)
- Progress indicator formatting
- Error message generation
- File locking and concurrency control
- HTTP Range request support
- Proxy configuration handling
- Model version mismatch detection
- Ctrl+C signal handling and cleanup
- Output conventions (println! vs eprintln!)

### Integration Tests

- Model download during init
- Error conditions (network failure, disk full, permission denied)
- Concurrent download prevention
- Download state persistence and recovery
- Model validation (checksum, size, format)
- Manual download instructions

### Performance Tests

- SHA-256 validation performance
- Download state check performance
- Progress update frequency
- Download speed measurement

### Edge Cases

- Interrupted downloads (Ctrl+C)
- Partial downloads (resume)
- Wrong model version
- Concurrent init commands
- Download URL changes
- Proxy configurations
- Multiple knowledge bases
- Non-interactive environments

## Open Questions

None - all technical decisions have been resolved.

## References

- [reqwest documentation](https://docs.rs/reqwest/)
- [sha2 crate](https://docs.rs/sha2/)
- [fastembed documentation](https://docs.rs/fastembed/)
- [Knowledge Loom constitution](../../.specify/memory/constitution.md)
- [Feature specification](./spec.md)
