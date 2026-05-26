# Implementation Plan: Integrate turbovec

**Branch**: `008-integrate-turbovec` | **Date**: 2026-05-25 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/008-integrate-turbovec/spec.md`

## Summary

Replace the sqlite-vec based vector index (`src/index.rs`) with turbovec's `IdMapIndex`, providing faster ANN search with 4-bit quantization (8x memory compression), SIMD-accelerated kernels, and native allowlist-based filtered search for graph-aware queries. Existing embeddings are auto-migrated on first startup.

## Technical Context

**Language/Version**: Rust 1.70+ (turbovec requires 1.70, knowledge-loom currently targets 1.75+)
**Primary Dependencies**: Tantivy 0.26 (BM25), Petgraph 0.6 (graph), rmcp 1.2 (MCP), tokio 1.0 (async), fastembed 4 (local embeddings), reqwest 0.12 (Ollama/OpenRouter)
**New Dependency**: turbovec 0.6.0 (MIT license) — vector index with IdMapIndex, 4-bit quantization, SIMD kernels
**Removed Dependency**: sqlite-vec 0.1, rusqlite (only if no other SQLite uses remain)
**Storage**: Tantivy index for BM25, turbovec `.tvim` file for vector index, bincode `graph.bin` for graph
**Testing**: cargo test (built-in), tempfile for file system tests, test-vault/ for corpus-based testing
**Target Platform**: Linux (x86-64, ARM), macOS (ARM/Apple Silicon, x86-64), Windows (x86-64)
**Project Type**: Library/Package with CLI binary and MCP server
**Performance Goals**: Vector search <100ms for 10k chunks, index memory >=50% smaller than raw float32, recall@10 >= 0.95
**Constraints**: >=50% memory reduction, recall@10 >= 0.95, concurrent search+index safe
**Scale/Scope**: 10k-100k indexed chunks, modular search engines, MCP protocol compliance

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Gate | Status | Notes |
|------|--------|-------|
| I. Rust-First Architecture | PASS | turbovec is a Rust crate; idiomatic patterns maintained |
| II. Modular Design | PASS | New `src/turbovec_index.rs` module replaces `src/index.rs`; clear boundary, minimal cross-deps |
| III. Test-First Development | PASS | Will write tests per TDD cycle; 80% coverage required |
| IV. Integration Testing | PASS | Integration tests for search, indexing, migration, concurrency |
| V. Quality Gates | PASS | `cargo fmt`, `cargo clippy`, `cargo test`, coverage, `cargo deny` all required |
| VI. MCP Protocol Compliance | PASS | MCP server tools unchanged; search contract preserved |
| VII. Performance Standards | PASS | Vector search <100ms target maintained; turbovec SIMD improves perf |
| VIII. Documentation Requirements | PASS | Doc comments, ARCHITECTURE.md, CHANGELOG.md |
| IX. Output Conventions | PASS | `eprintln!` for logging; `println!` only for CLI output |
| X. Technical Debt Policy | PASS | No intentional debt; sqlite-vec removed after migration |
| XI. Code Exploration (CRG) | PASS | CRG tools used for impact analysis |
| XII. Spec-Kit Workflow | PASS | Following spec → plan → tasks flow |
| Turbovec license (MIT) | PASS | Compatible with project's license policy |
| turbovec BLAS deps (accelerate/openblas) | PASS | System-native BLAS via pkg-config; no new audit issues |

## Project Structure

### Documentation (this feature)

```text
specs/008-integrate-turbovec/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root) — changes

```text
src/
├── turbovec_index.rs    # NEW — replaces src/index.rs, wraps turbovec::IdMapIndex
├── index.rs             # REMOVED — sqlite-vec vector store
├── search.rs            # MODIFIED — swap VectorIndex for TurbovecIndex in SearchEngine
├── server.rs            # MODIFIED — update LoomServer initialization
├── daemon.rs            # MODIFIED — update reindex triggers
├── maintenance.rs       # MODIFIED — add migration step, update index health
├── lib.rs               # MODIFIED — add turbovec_index module, remove index module
└── main.rs              # POSSIBLY MODIFIED — if EmbedProviderEnum needs dim for Vec construction

tests/
├── turbovec_index_tests.rs  # NEW — unit tests for TurbovecIndex
├── vector_tests.rs          # REMOVED/MODIFIED — replace sqlite-vec tests
├── search_tests.rs          # MODIFIED — update for TurbovecIndex
└── integration.rs           # MODIFIED — migration, persistence, concurrency tests

Cargo.toml               # MODIFIED — add turbovec, remove sqlite-vec/rusqlite if unused
```

## Complexity Tracking

> No constitutional violations. No complexity to justify.

## Phase 0: Research

See [research.md](./research.md) for detailed findings.

Key decisions:
1. **Use `IdMapIndex`** — stable external IDs needed for allowlist filtering and remove-by-ID
2. **ID scheme**: `fnv64a(path || heading || chunk_ordinal)` produces deterministic uint64 IDs
3. **4-bit quantization default** — balances recall and memory per spec FR-002
4. **Migration**: Read sqlite-vec DB → add_with_ids to turbovec → delete .db
5. **Concurrency**: turbovec is Send + Sync; lock via `Arc<Mutex<IdMapIndex>>` for write safety, read-only search can relax
6. **Metadata storage**: File path and heading stored in a separate `HashMap<u64, ChunkMeta>` alongside the index (turbovec doesn't store metadata)

## Phase 1: Design

See [data-model.md](./data-model.md) for entity design.
See [contracts/](./contracts/) for interface contracts.
See [quickstart.md](./quickstart.md) for developer getting-started.
