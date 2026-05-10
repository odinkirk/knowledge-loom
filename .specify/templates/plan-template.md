# Implementation Plan: [FEATURE]

**Branch**: `[###-feature-name]` | **Date**: [DATE] | **Spec**: [link]
**Input**: Feature specification from `/specs/[###-feature-name]/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

[Extract from feature spec: primary requirement + technical approach from research]

## Technical Context

**Language/Version**: Rust 1.75+ (Async Trait support required)
**Primary Dependencies**: Tantivy (BM25), Petgraph (graph), SQLite/vec (embeddings), rmcp 1.2 (MCP), tokio (async), anyhow/thiserror (error handling)
**Storage**: SQLite (via rusqlite) with sqlite-vec for vector storage, Tantivy index for BM25
**Testing**: cargo test (built-in), tempfile for file system tests, test-vault/ for corpus-based testing
**Target Platform**: Linux, macOS, Windows (cross-platform CLI tool with optional web UI at :8080)
**Project Type**: Library/Package with CLI binary and MCP server
**Performance Goals**: ~150ms unified search for 10k documents, <50ms BM25 search, <100ms vector search
**Constraints**: <200ms p95 for search operations, memory-efficient indexing, offline-capable
**Scale/Scope**: 10k+ documents, modular search engines, MCP protocol compliance

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

[Gates determined based on constitution file]

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
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
└── content_agnostic_tests.rs # Content-agnostic tests

test-vault/              # Test corpus for corpus-based testing
examples/                # Example usage and integrations
docs/                    # Documentation
scripts/                 # Utility scripts
```

**Structure Decision**: Knowledge Loom uses a modular Rust library structure with focused modules for each search engine (BM25, Vector, Graph) and supporting infrastructure (MCP server, CLI, daemon). All modules are co-located in `src/` with comprehensive test coverage in `tests/`.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
