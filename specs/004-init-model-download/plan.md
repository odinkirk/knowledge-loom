# Implementation Plan: Init-Time Model Download

**Branch**: `004-init-model-download` | **Date**: 2026-05-12` | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/004-init-model-download/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Implement init-time model download with structured plain text progress indicators, graceful error handling, and state management. The system will download the all-MiniLM-L6-v2 model during `loom init` with exponential backoff retry (1s, 2s, 4s delays), SHA-256 checksum validation, and file locking to prevent concurrent downloads. Progress is displayed in structured plain text format (e.g., "Downloading model: 45% (45MB/100MB) - 2.3MB/s - 24s remaining") that is both human-readable and AI-parseable. Model files are stored in KB_ROOT/.knowledge-loom-index/models/ following the existing index pattern.

**Recent Clarifications**:
- Output conventions: Use `println!` ONLY in CLI context for user-facing progress indicators, use `eprintln!` for all debug/logging output
- Interrupted downloads: Support resuming from last byte downloaded using HTTP Range requests
- Proxy configurations: Respect system proxy environment variables (HTTP_PROXY, HTTPS_PROXY, NO_PROXY)
- Model version mismatch: Detect by comparing model metadata version with expected version
- Ctrl+C signal handling: Catch SIGINT signal, clean up partial files, preserve download state for resume

## Technical Context

**Language/Version**: Rust 1.75+ (Async Trait support required)
**Primary Dependencies**: Tantivy (BM25), Petgraph (graph), SQLite/vec (embeddings), rmcp 1.2 (MCP), tokio (async), anyhow/thiserror (error handling), fastembed (model download), reqwest (HTTP client), sha2 (SHA-256 checksum), signal-hook (signal handling)
**Storage**: SQLite (via rusqlite) with sqlite-vec for vector storage, Tantivy index for BM25, KB_ROOT/.knowledge-loom-index/models/ for model files
**Testing**: cargo test (built-in), tempfile for file system tests, test-vault/ for corpus-based testing
**Target Platform**: Linux, macOS, Windows (cross-platform CLI tool with optional web UI at :8080)
**Project Type**: Library/Package with CLI binary and MCP server
**Performance Goals**: ~150ms unified search for 10k documents, <50ms BM25 search, <100ms vector search, <5s SHA-256 validation for 500MB model, <10ms download state checks, <1s HTTP Range request resume, <500ms Ctrl+C cleanup
**Constraints**: <200ms p95 for search operations, memory-efficient indexing, offline-capable, structured plain text progress format, println! ONLY in CLI context
**Scale/Scope**: 10k+ documents, modular search engines, MCP protocol compliance, single model (all-MiniLM-L6-v2)

**Technical Decisions**:
- Model download URL: Hardcoded in binary (all-MiniLM-L6-v2 from Qdrant)
- Model storage: KB_ROOT/.knowledge-loom-index/models/ (follows existing index pattern)
- Retry strategy: Exponential backoff with 3 retries (1s, 2s, 4s delays)
- Checksum algorithm: SHA-256 checksum (industry standard, strong security)
- Progress format: Structured plain text format (human-readable and AI-parseable)
- State management: File-based persistence for download state
- Concurrency control: File locking to prevent concurrent downloads
- Output conventions: `println!` ONLY in CLI context, `eprintln!` for all debug/logging output
- Interrupted downloads: HTTP Range requests for resume capability
- Proxy support: Respect system proxy environment variables (HTTP_PROXY, HTTPS_PROXY, NO_PROXY)
- Version detection: Compare model metadata version with expected version
- Signal handling: Catch SIGINT, clean up partial files, preserve state

**NEEDS CLARIFICATION**:
- None (all technical decisions resolved during spec clarification)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Rust-First Architecture ✅
- ✅ Uses idiomatic Rust patterns (Result<T, E>, Option<T>)
- ✅ Modular design with focused modules (init.rs, model.rs, download.rs)
- ✅ Leverages ownership system for memory safety
- ✅ Uses async/await with tokio for concurrent operations

### II. Modular Design ✅
- ✅ New modules: src/init.rs, src/model.rs, src/download.rs
- ✅ Affected modules: Embed, Server, Daemon
- ✅ Clear module boundaries and minimal cross-module dependencies
- ✅ Follows existing module structure

### III. Test-First Development ✅
- ✅ TDD approach: Write tests first → Get approval → Tests fail → Then implement
- ✅ Red-Green-Refactor cycle enforced
- ✅ All tests must pass before committing
- ✅ 80% code coverage minimum required

### IV. Integration Testing ✅
- ✅ Integration tests for model download during init
- ✅ Tests for error conditions (network failure, disk full, permission denied)
- ✅ Tests for concurrent download prevention
- ✅ Tests for download state persistence and recovery
- ✅ Tests for HTTP Range request support
- ✅ Tests for proxy configuration support
- ✅ Tests for model version mismatch detection
- ✅ Tests for Ctrl+C signal handling

### V. Quality Gates ✅
- ✅ Formatting: `cargo fmt --all -- --check` must pass
- ✅ Linting: `cargo clippy -- -D warnings` must pass
- ✅ Testing: `cargo test --all-features` must pass
- ✅ Coverage: Minimum 80% line coverage
- ✅ Security: `cargo deny check licenses bans sources` must pass
- ✅ CI: All GitHub Actions workflows must pass

### VI. MCP Protocol Compliance ✅
- ✅ No MCP protocol changes (feature does not involve MCP server changes)
- ✅ Maintains backward compatibility with existing clients

### VII. Performance Standards ✅
- ✅ Target: <5s SHA-256 validation for 500MB model
- ✅ Target: <10ms download state checks
- ✅ Target: <1s progress update visibility
- ✅ Target: <5min init completion on 10 Mbps connection
- ✅ Target: <1s HTTP Range request resume
- ✅ Target: <500ms Ctrl+C cleanup
- ✅ Avoids blocking operations in async contexts
- ✅ Uses appropriate data structures

### VIII. Documentation Requirements ✅
- ✅ Public functions and structs must have doc comments (`///`)
- ✅ Complex algorithms must have inline comments
- ✅ Architecture changes must update `ARCHITECTURE.md`
- ✅ New features must update `CHANGELOG.md`
- ✅ Manual download instructions must be documented in `README.md`
- ✅ Output conventions must be documented in code comments
- ✅ Edge case handling must be documented

### IX. Output Conventions ✅
- ✅ Uses `eprintln!` instead of `println!` for debug output
- ✅ Uses `println!` only for user-facing CLI output (progress indicators) in CLI context
- ✅ All debug/logging output uses `eprintln!` or proper logging frameworks
- ✅ Non-negotiable for MCP server stability
- ✅ Clarified: `println!` ONLY in CLI context, never in MCP server context

### X. Code Exploration and Analysis ✅
- ✅ Uses code-review-graph (CRG) tools for code exploration
- ✅ Uses CRG tools first before Grep/Glob/Read for code exploration
- ✅ Uses CRG for understanding code structure, finding dependencies, impact analysis
- ✅ Uses CRG semantic search for finding code entities by name or keyword
- ✅ Uses CRG graph queries for understanding relationships
- ✅ Uses CRG change detection for code reviews and PR analysis
- ✅ Uses Knowledge Loom tools (`loom_*`) for Markdown operations

**GATE STATUS**: ✅ PASSED - No constitution violations

## Project Structure

### Documentation (this feature)

```text
specs/004-init-model-download/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── bm25.rs              # BM25 full-text search engine (Tantivy)
├── graph.rs             # Wikilink graph analytics (Petgraph)
├── search.rs            # RRF-merged search orchestration
├── embed/               # Embedding providers
│   ├── mod.rs
│   ├── local.rs         # Local embedding model
│   └── ollama.rs        # Ollama API integration
├── server.rs            # MCP protocol implementation (rmcp)
├── edits.rs             # Surgical file editing operations
├── daemon.rs            # Background file watching (notify)
├── vault.rs             # Markdown vault scanning (walkdir)
├── web.rs               # Optional web UI (Axum)
├── index.rs             # Index management
├── init.rs              # Initialization utilities (NEW)
├── model.rs             # Model download and management (NEW)
├── download.rs          # Download progress and error handling (NEW)
├── maintenance.rs       # Maintenance operations
├── platforms.rs         # Platform-specific code
├── shell.rs             # Shell integration
├── lib.rs               # Library exports
└── main.rs              # CLI entry point

tests/
├── integration.rs       # Integration tests
├── behavioral_tests.rs  # Behavioral/end-to-end tests
├── mcp_protocol_tests.rs # MCP protocol compliance tests
├── bm25_tests.rs        # BM25 engine tests
├── vector_tests.rs      # Vector search tests
├── search_tests.rs      # Search orchestration tests
├── graph_fused_search_tests.rs # Graph search tests
├── embed_tests.rs       # Embedding provider tests
├── server_tests.rs      # MCP server tests
├── daemon_tests.rs      # Daemon tests
├── vault_tests.rs       # Vault scanning tests
├── init_tests.rs        # Initialization tests (NEW)
├── model_tests.rs       # Model download tests (NEW)
├── download_tests.rs    # Download progress tests (NEW)
├── platforms_tests.rs   # Platform-specific tests
├── shell_tests.rs       # Shell integration tests
├── rename_tests.rs      # File rename tests
├── bm25_lock_tests.rs   # BM25 concurrency tests
└── content_agnostic_tests.rs # Content-agnostic tests

test-vault/              # Test corpus for corpus-based testing
examples/                # Example usage and integrations
docs/                    # Documentation
scripts/                 # Utility scripts
```

**Structure Decision**: Knowledge Loom uses a modular Rust library structure with focused modules for each search engine (BM25, Vector, Graph) and supporting infrastructure (MCP server, CLI, daemon). All modules are co-located in `src/` with comprehensive test coverage in `tests/`. This feature adds three new modules (init.rs, model.rs, download.rs) following the existing modular pattern.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |

**No constitution violations - complexity tracking not required**
