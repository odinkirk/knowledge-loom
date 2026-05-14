# Changelog

All notable changes to Knowledge Loom will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

  ## [Unreleased]

  ### Added
  - **Init-time model download with structured plain text progress indicators**:
    - Automatic model download during `loom init` with progress display
    - Structured plain text format: "Downloading model: {percentage}% ({downloaded_mb}MB/{total_mb}MB) - {speed_mb}MB/s - {remaining}s remaining"
    - Progress updates at least once per second
    - Completion message with total time and average speed
  - **Model validation with SHA-256 checksum**:
    - Multi-stage validation (checksum, size, format)
    - SHA-256 checksum calculation for file integrity verification
    - Automatic deletion of corrupted files
    - Re-download on validation failure
  - **Download state management and persistence**:
    - File-based JSON state persistence at `{KB_ROOT}/.knowledge-loom-index/models/download-state.json`
    - State includes: status, progress, error message, timestamp
    - State recovery for resume capability
    - Atomic writes with temp file + rename
  - **File locking to prevent concurrent downloads**:
    - Exclusive file lock at `{KB_ROOT}/.knowledge-loom-index/models/.download.lock`
    - Clear error message when download is in progress
    - Automatic lock release on completion or error
  - **Graceful error handling with actionable messages**:
    - Network errors: "Network error: {details}. Please check your internet connection and try again."
    - Disk full: "Insufficient disk space: {required_mb}MB required, {available_mb}MB available. Please free up space and try again."
    - Permission denied: "Permission denied: Cannot write to {path}. Please check file permissions and try again."
    - Checksum mismatch: "Model validation failed: Checksum mismatch. The downloaded file may be corrupted. Please try downloading again."
    - Timeout: "Download timeout: The download took too long. Please check your internet connection and try again."
  - **HTTP Range request support for resuming interrupted downloads**:
    - Automatic resume from last byte downloaded
    - HTTP Range header: `Range: bytes={start}-`
    - Server support detection (206 Partial Content)
    - Fallback to full download if Range not supported
  - **Ctrl+C signal handling with cleanup and state preservation**:
    - Catch SIGINT signal (Ctrl+C)
    - Clean up partial files
    - Preserve download state for resume
    - Exit gracefully within 500ms
  - **Proxy configuration support**:
    - Respect HTTP_PROXY environment variable
    - Respect HTTPS_PROXY environment variable
    - Respect NO_PROXY environment variable
    - Automatic proxy detection and application
  - **Model version mismatch detection**:
    - Compare model metadata version with expected version
    - Prompt user for re-download on mismatch
    - Block model usage if version mismatch detected
  - **Manual download instructions as fallback**:
    - Step-by-step manual download instructions
    - Clear error messages with manual download guidance
    - Instructions included in README.md troubleshooting section
    - Helper function for formatting errors with instructions
  - **Safe chunk indexing with ordinal metadata**:
    - UTF-8-safe chunk truncation using `char_indices()` for character boundary detection
    - Ordinal metadata (1-based sequential numbering) for precise chunk retrieval
    - New `chunks.rs` module with centralized chunking logic
    - `get_chunk_by_ordinal()` API for retrieving chunks by file path and ordinal number
    - File-specific re-indexing after edits (not corpus-wide)
    - Corpus re-ingestion on re-indexing failure (<3 seconds for typical vaults)
    - Ingestion state tracking with "indexing: try again in 2 seconds" error during re-ingestion
    - Concurrent edit serialization with request queuing
  - Comprehensive README with architecture diagrams and feature documentation
  - GitHub Actions workflows for testing, building, and releasing
  - Multi-platform binary support (Linux, macOS, Windows)
  - Security vulnerability scanning with cargo-audit
  - License compliance checking with cargo-deny
  - MSRV set to Rust 1.75 for modern async trait support
  - Standardized code formatting with rustfmt.toml
  - Enhanced Clippy linting configuration
  - **Multiple embedding providers** with automatic fallback support:
    - LocalEmbedProvider: Built-in fastembed integration (all-MiniLM-L6-v2, 384 dimensions)
    - OllamaEmbedProvider: Ollama HTTP API integration (nomic-embed-text-v1.5, 768 dimensions)
    - OpenRouterEmbedProvider: OpenRouter HTTP API integration (openai/text-embedding-embedding-ada-002, 1536 dimensions)
  - **Provider priority chain**: OpenRouter > Ollama > Local with automatic fallback
  - **Environment configuration**: OLLAMA_URL, OPENROUTER_API_KEY, OPENROUTER_MODEL
  - **Performance targets**: <100ms local, <500ms Ollama, <1s OpenRouter
  - **Comprehensive error handling**: Network errors, timeouts, dimension mismatches
  - **Logging with eprintln!**: Debug output uses stderr to avoid MCP server pollution

  ### Changed
  - **`loom init` now downloads model during initialization**:
    - Model files stored in `{KB_ROOT}/.knowledge-loom-index/models/`
    - Automatic download with progress indicators
    - Validation before making model available
    - Manual download fallback for errors
  - Updated installation process to use `.knowledge-loom` directory structure
  - Improved platform detection and configuration for 8+ coding platforms
  - Enhanced search engine with RRF merging and graph-fused search
  - Improved documentation with Architecture.md and CONTRIBUTING.md
  - **BM25 schema**: Added `chunk_ordinal` STORED field for ordinal metadata
  - **ChunkDoc structure**: Added `chunk_ordinal` field to all chunk results
  - **SearchResult structure**: Ordinal metadata included in search results
  - **Graph Node structure**: Ordinal metadata included in node metadata
  - **MCP tool responses**: Ordinal metadata included in all chunk-related responses

   ### Fixed
  - **Fixed confusing hang during first-time indexing**: Model download now shows clear progress indicators
  - **Fixed model download error handling**: Enhanced error messages with specific guidance and manual download instructions
  - **UTF-8 panic during chunk truncation**: Fixed by using `char_indices()` for character boundary detection
  - **Errors swallowed silently in reindex_file()**: Fixed by returning `Result` and propagating errors to callers
  - **Race condition in ingestion state**: Fixed by keeping lock held between setting ingestion state and starting rebuild
  - **Inconsistent index state**: Fixed by implementing atomic semantics for multi-index updates
  - **Clippy warnings**: Fixed by using `std::io::Error::other()` instead of `std::io::Error::new(std::io::ErrorKind::Other, ...)`
  - Removed legacy Python installer references
  - Fixed path inconsistencies between old `.loom` and new `.knowledge-loom` structure
  - Removed unimplemented `search_smart` tool from MCP interface

## [0.1.0] - Initial Release

### Added
- BM25 full-text search with Tantivy
- Semantic vector search with sqlite-vec
- Graph analytics with PageRank and community detection
- Surgical file editing with line-level precision
- MCP protocol support for 8+ coding platforms
- Daemon mode for background watching
- Web UI for read-only access
- Multi-platform binary distribution
