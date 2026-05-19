# Implementation Plan: Fix MCP Tool Bugs from Live Smoke Test

**Branch**: `006-fix-mcp-tool` | **Date**: 2026-05-19 | **Spec**: [spec.md](./spec.md)
**Input**: Live smoke test findings from loom-review.md against unspoken-world corpus

## Summary

Fix 5 bugs and 2 UX issues discovered during live smoke testing of all 24 MCP tools against a real-world knowledge base. Bugs range from critical (search returns entire files instead of matched chunks, graph tools entirely non-functional) to medium (index_status reports zeros, symlink duplicates, subdirectory ignore failures). Two UX improvements: read_section depth parameter and relative path outputs.

## Technical Context

**Language/Version**: Rust 1.75+
**Primary Dependencies**: Tantivy (BM25), Petgraph (graph), SQLite/vec (embeddings), rmcp 1.2 (MCP), tokio (async), walkdir (vault scanning), glob (ignore patterns)
**Testing**: cargo test (built-in), tempfile for file system tests
**Target Platform**: Linux, macOS, Windows
**Project Type**: MCP server + CLI binary

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **III. Test-First Development (NON-NEGOTIABLE)**: All fixes will follow TDD cycle. [PASS]
- **IV. Integration Testing**: Tests required for search result filtering, graph link extraction, symlink dedup. [PASS]
- **V. Quality Gates**: fmt, clippy, test all must pass. [PASS]
- **VI. MCP Protocol Compliance**: All modified tools follow rmcp 1.2. Backward compatible. [PASS]
- **VII. Performance Standards**: <150ms search target preserved. [PASS]
- **VIII. Documentation Requirements**: ARCHITECTURE.md and CHANGELOG.md updated. [PASS]
- **IX. Output Conventions**: eprintln! for server, println! only for CLI. [PASS]
- **X. Technical Debt Policy**: All 7 items fixed immediately (no deferrals). [PASS]

## Project Structure

### Documentation (this feature)

```text
specs/006-fix-mcp-tool/
├── plan.md              # This file
├── spec.md              # Feature specification
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
src/
├── search.rs            # Bug 1: chunk filtering fix
├── graph.rs             # Bug 5: link extraction fix (both [[wikilink]] and [text](path.md))
├── vault.rs             # Bug 2: symlink dedup, Bug 4: subdirectory ignore patterns
├── server.rs            # Bug 3: index_status counts, UX 2: relative paths
├── edits.rs             # UX 1: read_section depth parameter
├── bm25.rs              # Bug 3: document count query
├── index.rs             # Bug 3: vector count query
```

## Complexity Tracking

No new modules. All fixes are localized to existing files. No architectural changes.

## Bug Fix Plan

### Bug 1 — Search Chunk Filtering (`src/search.rs`)

**Root cause**: Lines 180-206 fetch ALL chunks for any result whose sections array is empty. The vector/graph paths call `get_chunks_for_path` which returns every chunk. Additionally, BM25 hits accumulate every chunk regardless of score.

**Fix**: For each file in the final result set, only include chunks that appeared in the BM25 hit list (non-zero score). If a file scores via graph/vector but has no BM25 chunk hit, return at most the top-1 vector chunk, not the entire file.

### Bug 2 — Symlink Dedup (`src/vault.rs`)

**Root cause**: `WalkDir::follow_links(true)` resolves symlinks but indexes both the symlink path and the target path independently. The vault scanner should track canonical paths.

**Fix**: During vault scanning, resolve each path to its canonical form (`std::fs::canonicalize`). Skip any path whose canonical form has already been seen.

### Bug 3 — index_status Reports Zeros (`src/server.rs`, `src/bm25.rs`, `src/index.rs`)

**Root cause**: `get_index_status()` returns hardcoded zeros for `documents`, `vectors`, and `edges`. The Tantivy reader has `num_docs()` and the SQLite index can `SELECT COUNT(*)`.

**Fix**: Query actual counts from Tantivy reader (for BM25 docs), SQLite (for vector count), and graph (for edges/nodes).

### Bug 4 — Subdirectory Ignore Patterns (`src/vault.rs`)

**Root cause**: The `IgnorePattern` matching matches against the full relative path, but `.venv/` matches only `tools/.venv/` not `**/.venv/`. Patterns should match any path component suffix.

**Fix**: In `IgnorePattern::matches()`, also check if any path component matches the pattern. For directory-prefix patterns, check if the relative path starts with the prefix OR contains `/{prefix}/`.

### Bug 5 — Graph Link Extraction (`src/graph.rs`)

**Root cause**: Link extraction likely handles only one format. The Story Bible uses both `[[wikilink]]` and `[text](path.md)`.

**Fix**: Verify and fix `extract_wikilinks()` to handle both formats. Add regression tests with both link styles.

### UX 1 — read_section Depth (`src/edits.rs`, `src/server.rs`)

**Fix**: Add `depth` parameter to `read_section` (default 0 = full tree, backward compatible). Depth 1 stops at the first subheading.

### UX 2 — Relative Paths (`src/server.rs`)

**Fix**: In `dispatch_tool` for `list_files` and `grep`, strip KB_ROOT prefix from path outputs so they're immediately reusable as input to other tools.
