# Implementation Plan: Safe Chunk Indexing with Ordinal Metadata

**Branch**: `003-safe-chunk-indexing` | **Date**: 2025-05-11 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/003-safe-chunk-indexing/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

This feature implements character boundary-safe chunk truncation to fix UTF-8 panics, adds ordinal metadata to chunks for precise retrieval, and extracts chunking logic into a dedicated `src/chunks.rs` module. The technical approach involves creating a new chunks module that handles UTF-8-safe truncation and ordinal assignment, updating BM25 to store ordinal metadata, implementing file-specific re-indexing after edits, and ensuring all chunk-consuming modules (Search, Edits, Graph, Vault) properly handle the new metadata structure. On re-indexing failure, the system drops indices and re-ingests the entire corpus (completes in 2-3 seconds), returning "indexing: try again in 2 seconds" error for requests during ingestion.

## Technical Context

**Language/Version**: Rust 1.75+ (Async Trait support required)
**Primary Dependencies**: Tantivy (BM25), Petgraph (graph), SQLite/vec (embeddings), rmcp 1.2 (MCP), tokio (async), anyhow/thiserror (error handling)
**Storage**: SQLite (via rusqlite) with sqlite-vec for vector storage, Tantivy index for BM25
**Testing**: cargo test (built-in), tempfile for file system tests, test-vault/ for corpus-based testing
**Target Platform**: Linux, macOS, Windows (cross-platform CLI tool with optional web UI at :8080)
**Project Type**: Library/Package with CLI binary and MCP server
**Performance Goals**: ~150ms unified search for 10k documents, <50ms BM25 search, <100ms vector search, <10ms chunk truncation, <50ms chunk retrieval, <100ms file re-indexing (1-100KB files with 1-50 chunks), <3s corpus re-ingestion on failure
**Constraints**: <200ms p95 for search operations, memory-efficient indexing, offline-capable, <1% memory overhead for ordinal metadata
**Scale/Scope**: 10k+ documents, modular search engines, MCP protocol compliance, file-specific re-indexing (not corpus-wide)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Principle Compliance

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Rust-First Architecture | ✅ PASS | Uses idiomatic Rust patterns, Result<T, E> error handling, async/await with tokio |
| II. Modular Design | ✅ PASS | Creates dedicated `src/chunks.rs` module with clear boundaries, updates 6 existing modules |
| III. Test-First Development | ✅ PASS | TDD required, 80% minimum coverage, tests written before implementation |
| IV. Integration Testing | ✅ PASS | Integration tests required for cross-module interactions (chunks → BM25, Search, Edits, Graph, Vault) |
| V. Quality Gates | ✅ PASS | All quality gates apply: fmt, clippy, test, coverage, security, CI |
| VI. MCP Protocol Compliance | ✅ PASS | Follows rmcp 1.2, maintains backward compatibility, includes protocol tests |
| VII. Performance Standards | ✅ PASS | Performance targets defined: <10ms truncation, <50ms retrieval, <100ms re-indexing, <3s corpus re-ingestion |
| VIII. Documentation Requirements | ✅ PASS | Doc comments required, inline comments for algorithms, updates to ARCHITECTURE.md and CHANGELOG.md |
| IX. Output Conventions | ✅ PASS | Uses eprintln! for debug output, reserves println! for CLI output |
| X. Code Exploration and Analysis | ✅ PASS | Uses CRG tools for code exploration, Knowledge Loom tools for Markdown operations |

### Gate Status

**✅ ALL GATES PASS** - No constitution violations detected. Proceed to Phase 0 research.

## Project Structure

### Documentation (this feature)

```text
specs/003-safe-chunk-indexing/
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
├── chunks.rs            # NEW: Character boundary-safe chunking with ordinal metadata
├── bm25.rs              # BM25 full-text search engine (Tantivy) - UPDATED
├── graph.rs             # Wikilink graph analytics (Petgraph) - UPDATED
├── search.rs            # RRF-merged search orchestration - UPDATED
├── embed/               # Embedding providers
│   ├── mod.rs
│   ├── local.rs         # Local embedding model
│   └── ollama.rs        # Ollama API integration
├── server.rs            # MCP protocol implementation (rmcp) - UPDATED
├── edits.rs             # Surgical file editing operations - UPDATED
├── daemon.rs            # Background file watching (notify)
├── vault.rs             # Markdown vault scanning (walkdir) - UPDATED
├── web.rs               # Optional web UI (Axum)
├── index.rs             # Index management
├── init.rs              # Initialization utilities
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
├── init_tests.rs        # Initialization tests
├── platforms_tests.rs   # Platform-specific tests
├── shell_tests.rs       # Shell integration tests
├── rename_tests.rs      # File rename tests
├── bm25_lock_tests.rs   # BM25 concurrency tests
├── content_agnostic_tests.rs # Content-agnostic tests
└── chunks_tests.rs      # NEW: Chunking module tests

test-vault/              # Test corpus for corpus-based testing
examples/                # Example usage and integrations
docs/                    # Documentation
scripts/                 # Utility scripts
```

**Structure Decision**: Knowledge Loom uses a modular Rust library structure with focused modules for each search engine (BM25, Vector, Graph) and supporting infrastructure (MCP server, CLI, daemon). All modules are co-located in `src/` with comprehensive test coverage in `tests/`. This feature adds a new `chunks.rs` module to centralize chunking logic and updates 6 existing modules to handle ordinal metadata.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations detected. This section remains empty.
