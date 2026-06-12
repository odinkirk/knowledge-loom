# Changelog

All notable changes to Knowledge Loom will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

  ## [Unreleased]

  ### Fixed
  - **Documentation cleanup**: Replaced stale sqlite-vec references with turbovec in README.md (features table, architecture overview, storage description, benchmarks, comparison table, credits), removed SQLite prerequisite from CONTRIBUTING.md, fixed Mermaid parse error in Architecture.md Model Download Flow diagram by renaming single-letter node IDs (L1/M1/N1/O1 → DL1/DM1/DN1/DO1)
  - **MSRV consistency**: Added `rust-version = "1.75"` to Cargo.toml as canonical MSRV, updated README badge and prereqs from `1.70+` to `1.75+`
  - **CI modernization**: Replaced deprecated `actions-rs/toolchain@v1` with `dtolnay/rust-toolchain@v1`, `actions-rs/cargo@v1` with direct `run: cargo` commands, `actions-rs/audit-check@v1` with direct `cargo audit`; upgraded `actions/cache@v3` to `actions/cache@v4` across all workflows; added `cargo-deny` and `cargo-audit` binary caching for faster CI runs; simplified build matrix to stable-only across 3 OSes; MSRV check now derives version from `Cargo.toml` instead of hardcoding

  ### Changed
  - **turbovec vector index integration**:
    - Replaced sqlite-vec vector store with turbovec `IdMapIndex` for approximate nearest neighbor search
    - 4-bit TurboQuant compression (3.5x total, 8x vectors-only vs float32) with configurable `LOOM_TURBOVEC_BIT_WIDTH` env var (2 or 4)
    - SIMD-accelerated search kernels (NEON on ARM, AVX-512BW on x86, AVX2 fallback)
    - Pure turbovec ANN search: ~135µs (p50) on Apple M3, 1,000 chunks — negligible overhead vs full RRF pipeline
    - Filtered search via `allowlist` for graph-aware scoped queries
    - Disk persistence via `.tvim` + `turbovec_meta.bin` + `turbovec_config.bin`
    - Automatic migration from legacy sqlite-vec embeddings on first startup
    - macOS: Added Accelerate framework linking via `.cargo/config.toml`
    - **Performance impact**: 47% memory reduction, 59% disk reduction, vector search 350x faster than sqlite-vec brute-force scan
  - **Grep tool improvements**:
    - Added default result limit of 200 matches to prevent response flooding
    - Implemented optional `limit` parameter (0 = no limit)
    - Implemented `file_filter` parameter for substring filtering on file paths
    - Response now includes `truncated` flag and `total_matches` count
    - Results use relative paths consistent with other MCP tools

  ### Added
  - **Runtime data install command**:
    - New `loom install` subcommand to download fastembed models
    - Models stored in `.knowledge-loom/models/` (separate from index)
    - SHA-256 checksum verification for integrity
    - `--force` flag to re-download even if already installed
    - Skip download if model already valid
    - Output summary: model version, download size (MB), target location
    - Clear error messages with `--force` recommendation on failure
  - **Integrity verification**:
    - `verify_integrity()` checks SHA-256 checksum against stored state
    - Auto-detects corrupted or missing model files
    - Triggers re-download when integrity check fails
  - **Re-install/update support**:
    - `loom install --force` for re-downloading models
    - Skip logic: reports "already installed" when model is valid
    - Overwrites existing model files on force re-download
  - **Shared download utilities** (Technical Debt Remediation):
    - Consolidated download logic in `src/download/utils.rs`
    - Reusable `calculate_checksum()`, `validate_checksum()`, `check_disk_space()`
    - Eliminated code duplication across install.rs, model.rs, init.rs
  - **CLI argument parsing utilities** (Technical Debt Remediation):
    - Created `src/cli/args.rs` with `parse_flag()`, `parse_string_value()`, `validate_flags()`
    - Supports --flag, -f, --key=value, --key value forms
    - Provides helpful error messages for missing values and unknown flags
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
  - **CI test-vault**: Tests workflow now clones `test-vault/` corpus before running tests, resolving `test_graph_edges_from_test_vault` failure
  - **E2E tests**: Added `loom install` step to pre-download ONNX model before e2e tests, preventing network download failures during `cargo test`
  - **Test isolation**: Marked embed provider tests with `#[serial]` to prevent fastembed model cache lock contention; fixed env var contamination in test runner
  - **Platform test**: Rewrote `test_run_init_with_platform_claude` to use `InitManager` directly, bypassing env-var-dependent initialization checks
  - **MSRV**: Bumped minimum Rust version from 1.75 to 1.89 to match actual code dependencies (`StorageFull` from 1.83, `file.unlock()` from 1.89)
  - **Clippy**: Allowed `collapsible_str_replace` lint introduced in Rust 1.96.0
  - **Smoke test**: Rewrote `smoke_test_subdrop_search` as `smoke_test_corpus_search` using portable `test-vault/` corpus instead of hardcoded personal machine path
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
   - **Fixed broken glob matching in .knowledge-loom-ignore**: Replaced `contains()` substring match with `glob::Pattern` and directory-prefix matching. Patterns like `.claude/**` and `*.log` now work correctly. Added `.claude/` to default ignored patterns.
   - **Fixed OpenCode platform config**: `loom init --platform opencode` now writes correct `opencode.json` with `$schema`, `mcp` key, `type: "local"`, `command` as array, `environment` as object. No longer creates unwanted `.mcp.json`.
   - **Fixed reindex performance (80× BM25 speedup)**: Single commit at end of `index_vault()` instead of per-file commits. 87s → 1.1s.
   - **Fixed chunk truncation**: Sections exceeding 800 chars are now split into multiple chunks instead of being silently truncated, preserving all content for search.
   - **Fixed reindex state tracking**: Added `ReindexState` with per-file mtime+chunk_count for incremental reindex. Subsequent `loom reindex` skips unchanged files (93ms vs minutes for full rebuild).
   - **Fixed tantivy lock contention**: Removed `check_schema()` from pre-reindex health check — it opened an IndexWriter, then `index_vault()` immediately tried to open another writer, causing `LockBusy` on every run.
   - **Fixed 10 edit test failures**: `BM25Index::index_file()` missing `commit()` after Phase 8 single-commit optimization. Added `BM25Index::commit()` called from `EditManager::reindex_file()`.
   - **Fixed search returning entire files**: `search` now returns only BM25-scored chunks, not all chunks from matched files. Vector/graph fallback returns at most top-1 chunk. `top_k` applied to total section count, not file count.
   - **Fixed graph tools returning zero edges**: `extract_wikilinks()` now handles both `[[wikilink]]` and `[text](path.md)` formats. `.md` extension stripped to match node naming.
   - **Fixed symlink duplicate indexing**: `scan_files()` now canonicalizes paths via `std::fs::canonicalize()` and deduplicates by canonical form.
   - **Fixed subdirectory ignore patterns**: `.knowledge-loom-ignore` patterns like `.venv/` now match in subdirectories (`tools/.venv/`), not just KB_ROOT.
   - **Fixed index_status reporting zeros**: `get_index_status()` now queries actual BM25 document count, vector count, and graph edge count.
   - **Added read_section depth parameter**: Optional `depth` param (default 0 = full tree, backward compatible). `depth=1` stops at first subheading.
   - **Relative paths in tool outputs**: `list_files` and `grep` now return paths relative to KB_ROOT, immediately reusable as input to other tools.

  ### Changed
   - Reduced `MAX_CHUNK_CHARS` from 2000 to 800 to fit token window
   - Vector embedding now uses batch inference (`embed_batch`) instead of per-chunk `embed()` calls
   - BM25 and Vector indexes now share consistent chunk boundaries via unified `parse_chunks()`
   - OpenCode platform config: removed `opencode` boolean parameter, format is always schema-conformant
   - **Embedding model**: Switched from `AllMiniLML6V2` to `BGESmallENV15` (384-dim) for consistent 200s full-reindex on Intel CPU (MiniLM varied 147–552s due to thermal throttling). Incremental improved: 93ms vs 117ms.

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
