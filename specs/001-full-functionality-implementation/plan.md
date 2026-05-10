# Implementation Plan: Full Functionality Implementation

**Branch**: `001-full-functionality-implementation` | **Date**: 2025-05-09 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-full-functionality-implementation/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Replace hash-based mock embeddings with real embedding implementations across three providers (local, Ollama, OpenRouter). The local provider uses fastembed with all-MiniLM-L6-v2 (384 dimensions), while external providers offer optional higher-quality embeddings via HTTP APIs. All providers implement a common trait with automatic fallback on failure (timeout >5s, HTTP errors, invalid format). The system defaults to local embeddings with optional priority configuration via environment variables.

## Technical Context

**Language/Version**: Rust 1.75+ (Async Trait support required)
**Primary Dependencies**: Tantivy (BM25), Petgraph (graph), SQLite/vec (embeddings), rmcp 1.2 (MCP), tokio (async), anyhow/thiserror (error handling), fastembed (local embeddings), reqwest (HTTP client)
**Storage**: SQLite (via rusqlite) with sqlite-vec for vector storage, Tantivy index for BM25, local model cache in `.knowledge-loom-index/models/`
**Testing**: cargo test (built-in), tempfile for file system tests, test-vault/ for corpus-based testing, mock HTTP responses for external providers
**Target Platform**: Linux, macOS, Windows (cross-platform CLI tool with optional web UI at :8080)
**Project Type**: Library/Package with CLI binary and MCP server
**Performance Goals**: <100ms local embedding generation, <500ms Ollama embedding, <1s OpenRouter embedding, <150ms unified search for 10k documents
**Constraints**: <200ms p95 for search operations, memory-efficient indexing (<500MB for embedding model), offline-capable with local provider
**Scale/Scope**: 10k+ documents, modular search engines, MCP protocol compliance, automatic fallback on provider failure

**Key Technical Decisions**:
- **Local Provider**: fastembed with all-MiniLM-L6-v2 (384 dimensions, ~80MB model)
- **Ollama Provider**: HTTP API integration with configurable URL via OLLAMA_URL
- **OpenRouter Provider**: HTTP API integration with API key (OPENROUTER_API_KEY) and model selection (OPENROUTER_MODEL)
- **Fallback Strategy**: Local first, then external providers, with optional explicit priority configuration
- **Failure Detection**: Network timeout >5s, HTTP 4xx/5xx errors, invalid response format
- **Dimension Validation**: Reject mismatched dimensions with warning, fallback to local provider
- **Model Recovery**: Auto-retry corrupted/missing local models, use external providers if available

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Compliance Status

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Rust-First Architecture | ✅ PASS | Uses idiomatic Rust patterns, async/await with tokio |
| II. Modular Design | ✅ PASS | Focused modules in `src/embed/` with clear boundaries |
| III. Test-First Development | ⚠️ NEEDS VERIFICATION | Must ensure TDD approach and 80% coverage |
| IV. Integration Testing | ⚠️ NEEDS VERIFICATION | Must add integration tests for provider switching |
| V. Quality Gates | ⚠️ NEEDS VERIFICATION | Must pass fmt, clippy, test, coverage, deny checks |
| VI. MCP Protocol Compliance | ✅ PASS | No MCP protocol changes required |
| VII. Performance Standards | ⚠️ NEEDS VERIFICATION | Must meet <100ms local embedding target |
| VIII. Documentation Requirements | ⚠️ NEEDS VERIFICATION | Must add doc comments and update docs |
| IX. Output Conventions | ✅ PASS | Using eprintln! for debug output, no println! in MCP code |
| X. Code Exploration and Analysis | ✅ PASS | CRG tools used for code analysis |

### Gates

**PRE-PHASE 0 GATES**:
- ✅ No MCP protocol changes required
- ⚠️ Must verify TDD approach during implementation
- ⚠️ Must ensure 80% code coverage
- ⚠️ Must meet performance targets (<100ms local embedding)

**POST-PHASE 1 GATES** (re-evaluated after design):
- ✅ Integration tests for provider switching (defined in quickstart.md)
- ✅ Performance benchmarks for all providers (defined in research.md)
- ✅ Documentation updates (defined in quickstart.md)
- ⚠️ Must implement TDD approach during implementation
- ⚠️ Must verify 80% code coverage after implementation
- ⚠️ Must meet performance targets after implementation

## Project Structure

### Documentation (this feature)

```text
specs/001-full-functionality-implementation/
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
│   ├── mod.rs           # Provider trait and enum
│   ├── local.rs         # Local embedding model (fastembed)
│   ├── ollama.rs        # Ollama API integration
│   └── openrouter.rs    # OpenRouter API integration (NEW)
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

**Structure Decision**: Knowledge Loom uses a modular Rust library structure with focused modules for each search engine (BM25, Vector, Graph) and supporting infrastructure (MCP server, CLI, daemon). All modules are co-located in `src/` with comprehensive test coverage in `tests/`. The new OpenRouter provider follows the existing pattern established by the Ollama provider.

## CRG Tools Usage

**Constitution Requirement**: Use code-review-graph (CRG) tools for all code exploration and analysis tasks.

### CRG Tools for This Feature

During implementation, CRG tools will be used for:

1. **Code Exploration**: Understanding existing embedding provider structure
   - `code-review-graph_query_graph_tool` with `callers_of` pattern to find embed usage
   - `code-review-graph_semantic_search_nodes_tool` to find embedding-related code
   - `code-review-graph_get_impact_radius_tool` to analyze changes impact

2. **Impact Analysis**: Understanding how embed provider changes affect the codebase
   - `code-review-graph_detect_changes_tool` to identify affected functions
   - `code-review-graph_get_affected_flows_tool` to find impacted execution flows
   - `code-review-graph_get_review_context_tool` for comprehensive code review

3. **Code Review**: Ensuring quality and compliance
   - `code-review-graph_get_hub_nodes_tool` to identify architectural hotspots
   - `code-review-graph_get_bridge_nodes_tool` to find critical dependencies
   - `code-review-graph_get_suggested_questions_tool` for review guidance

### CRG Tools Priority

**ALWAYS use CRG tools first** before Grep/Glob/Read for code exploration:
- Use CRG for: understanding code structure, finding dependencies, impact analysis, code reviews
- Use CRG semantic search for finding code entities by name or keyword
- Use CRG graph queries for understanding relationships (callers, callees, imports)
- Use CRG change detection for code reviews and PR analysis
- **EXCEPTION**: Do NOT use CRG for Markdown files - use Knowledge Loom tools instead

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |

**Note**: No constitution violations identified. All gates are passing or require verification during implementation.
