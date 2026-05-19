# Tasks: Fix MCP Tool Bugs

**Input**: Design documents from `specs/006-fix-mcp-tool/`
**Prerequisites**: plan.md (required), spec.md (required)

**Tests**: Test tasks are included per spec TEST-001 through TEST-005 requirement and Constitution §III (TDD).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Knowledge Loom**: `src/`, `tests/` at repository root
- **Modules**: search.rs, graph.rs, vault.rs, server.rs, edits.rs, bm25.rs, index.rs
- **Tests**: `tests/` with `*_tests.rs` naming convention

## Phase 1: Setup

**Purpose**: Verify clean baseline before making changes

- [x] T001 Verify cargo build --release succeeds on current main
- [x] T002 Verify cargo test --all-features passes (all 61+ lib tests) on current main
- [x] T003 Verify cargo fmt -- --check and cargo clippy -- -D warnings pass on current main

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Read existing code to understand current behavior before modifying

- [x] T004 [P] Read `src/search.rs` lines 106-206: understand current BM25 hit loop, `sections_map` accumulation, and `get_chunks_for_path` fallback for vector/graph paths
- [x] T005 [P] Read `src/graph.rs` `extract_wikilinks()` and `build_graph()`: understand current link parsing regex and edge insertion logic
- [x] T006 [P] Read `src/vault.rs` `scan_files()` and `should_ignore()`: understand WalkDir symlink behavior and IgnorePattern matching logic
- [x] T007 [P] Read `src/server.rs` `get_index_status()` dispatch: understand how BM25 docs, vector count, and graph edges are currently queried
- [x] T008 [P] Read `src/server.rs` `dispatch_tool()` for `list_files`, `grep`, `read_section`: understand current path serialization and section reading logic

## Phase 3: User Story 1 - Search Returns Only Matched Chunks (Priority: P1)

**Goal**: `search` returns only chunks that matched the query, not entire files. `top_k` applies to total section count, not file count.

**Independent Test**: Search for a term matching a known multi-chunk file. Verify only matched chunks returned, not all chunks.

### Tests for User Story 1

- [x] T009 [P] [US1] Write unit test for `search` chunk filtering in `tests/search_tests.rs`: search for term in a file with 10 chunks where only chunk 3 matches. Assert only 1 section returned with that chunk's content.
- [x] T010 [P] [US1] Write unit test for vector-only result inclusion in `tests/search_tests.rs`: file scores via vector but has no BM25 hit. Assert at most 1 chunk returned (not entire file).

### Implementation for User Story 1

- [x] T011 [US1] Fix `search()` in `src/search.rs:106–128`: in the BM25 hit loop, only insert chunks with matching `chunk_ordinal` into `sections_map` — do not accumulate every chunk per file.
- [x] T012 [US1] Fix `search()` in `src/search.rs:180–206`: remove `get_chunks_for_path` fallback for vector/graph paths that returns entire file. For files with no BM25 hit but non-zero vector score, return at most the top-1 vector chunk (the one with the highest vector similarity).
- [x] T013 [US1] Apply `top_k` limit after chunk filtering (not before) in `src/search.rs`. If `top_k=3` and one file has 2 matching chunks, return both (3 total sections across files).

## Phase 4: User Story 2 - Graph Tools Return Valid Edges (Priority: P1)

**Goal**: Graph indexing extracts both `[[wikilink]]` and `[text](path.md)` formats. `find_connections`, `find_path_between`, `search_graph` return non-empty results.

**Independent Test**: Reindex a file with `[[other-note]]`. Verify `find_connections` returns the linked file.

### Tests for User Story 2

- [x] T01- [x] T014  [P] [US2] Write unit test for `extract_wikilinks()` with `[[wikilink]]` syntax in `tests/graph_tests.rs`: verify `[[note]]` and `[[note|alias]]` are extracted.
- [x] T01- [x] T015  [P] [US2] Write unit test for `extract_wikilinks()` with standard Markdown links in `tests/graph_tests.rs`: verify `[text](other.md)` is extracted, ignoring external URLs (`[text](https://...)`). Also rebuild graph from a test vault with links and assert `graph_state.graph.edge_count() > 0`.

- [x] T015a [P] [US2] Write integration test for graph edges using test-vault corpus in `tests/graph_tests.rs`: reindex `test-vault/` (7 files with `[[wikilinks]]`), then assert `graph_state.graph.edge_count() > 0` and `find_connections` returns non-empty results for a known linked file. Verifies the fix works on real wikilinks, not just synthetic test data.

### Implementation for User Story 2

- [x] T01- [x] T016  [US2] Fix `extract_wikilinks()` in `src/graph.rs` to handle both `[[wikilink]]` (with optional `|alias`) and standard `[text](path.md)` Markdown links. Ignore external HTTP links.
- [x] T01- [x] T017  [US2] Verify `build_graph()` in `src/graph.rs` uses the updated `extract_wikilinks()` during reindex. Edge insertion should already work once links are extracted.

## Phase 5: User Story 3 - Symlinks Not Indexed Twice (Priority: P2)

**Goal**: Vault scanning canonicalizes paths to avoid indexing the same file twice when reached via symlink.

**Independent Test**: Create a test vault with a symlinked file. Verify only one canonical path in search results.

### Tests for User Story 3

- [x] T018 [P] [US3] Write unit test for symlink dedup in `tests/vault_tests.rs`: create a tempdir with `dir/B.md`, create symlink `A.md → dir/B.md`. Run `scan_files()`. Assert only one path returned.

### Implementation for User Story 3

- [x] T019 [US3] Fix `scan_files()` in `src/vault.rs:49–67`: after collecting files, canonicalize each path with `std::fs::canonicalize()`. Skip paths whose canonical form has already been seen. Store canonical paths in a `HashSet<PathBuf>` before returning.

## Phase 6: User Story 4 - .knowledge-loom-ignore Subdirectory Patterns (Priority: P2)

**Goal**: `.knowledge-loom-ignore` patterns like `.venv/` match in subdirectories (`tools/.venv/`), not just KB_ROOT.

**Independent Test**: Create a test vault with `.venv/` in `.knowledge-loom-ignore` and `tools/.venv/LICENSE.md`. Verify file is excluded.

### Tests for User Story 4

- [x] T020 [P] [US4] Write unit test for subdirectory ignore in `tests/vault_tests.rs`: create `.knowledge-loom-ignore` with `.venv/`, create `tools/.venv/LICENSE.md`. Assert file is excluded from `scan_files()`.

### Implementation for User Story 4

- [x] T021 [US4] Fix `IgnorePattern::matches()` in `src/vault.rs`: for directory-prefix patterns, also check if any path component matches. If the relative path is `tools/.venv/LICENSE.md` and pattern is `.venv/`, match because a path component (`.venv`) equals the prefix.

## Phase 7: User Story 5 - index_status Reports Accurate Counts (Priority: P2)

**Goal**: `index_status` returns actual BM25 document count, vector count, and graph edge count, not hardcoded zeros.

**Independent Test**: After full reindex, run `index_status`. Verify `documents > 0`, `vectors > 0`, `edges > 0`.

### Tests for User Story 5

- [x] T022 [P] [US5] Write integration test for `index_status` counts in `tests/server_tests.rs`: initialize a test vault with 2 linked files, reindex, call `get_index_status`. Assert `documents >= 2`, `vectors > 0`, `edges >= 1`.

### Implementation for User Story 5

- [x] T023 [US5] Fix `get_index_status()` in `src/maintenance.rs:305–329`: replace hardcoded `0` for `documents` with `bm25_index.reader().searcher().num_docs()` (must refresh reader first).
- [x] T024 [P] [US5] Fix `get_index_status()` vector count: replace `0` with `VectorIndex::count_embeddings()` result (method already exists at src/index.rs:127–130).
- [x] T025 [P] [US5] Fix `get_index_status()` graph status in `src/maintenance.rs:305–329`: replace `0` for `edges` with actual edge count from `graph_state.graph.edge_count()`. Node count is already correct (uses `node_map.lock().await.len()`).

## Phase 8: User Story 6 - read_section Depth Control (Priority: P3)

**Goal**: `read_section` accepts optional `depth` parameter. `depth=1` stops at first subheading. Default `depth=0` (or absent) preserves backward-compatible behavior (full section tree).

**Independent Test**: Call `read_section` with `depth=1` on a heading with subsections. Verify only direct content returned.

### Tests for User Story 6

- [x] T026 [P] [US6] Write integration test for `read_section` depth in `tests/server_tests.rs`: create a file with heading "A" containing content and subsections "B", "C". Call `read_section "A" depth=1`. Assert content under "A" is returned but "B" and "C" content is excluded.

### Implementation for User Story 6

- [x] T027 [US6] Add `depth` parameter to `read_section` tool schema in `src/server.rs`: `"depth": {"type": "integer", "description": "Max heading depth (0 = full tree, 1 = stop at first subheading)", "default": 0}` in the JSON schema definition.
- [x] T028 [US6] Implement depth filtering in `read_section` handler in `src/edits.rs` (or `src/server.rs`): when reading section under a heading, if `depth > 0`, stop collecting content when encountering a heading at or above the same level as `depth`.

## Phase 9: User Story 7 - Relative Paths in Tool Outputs (Priority: P3)

**Goal**: `list_files` and `grep` return paths relative to KB_ROOT, immediately reusable as input to other tools.

**Independent Test**: Run `list_files`. Verify paths do not contain the KB_ROOT absolute prefix.

### Tests for User Story 7

- [x] T029 [P] [US7] Write integration test for relative paths in `tests/server_tests.rs`: with KB_ROOT set to a temp dir, run `list_files`. Assert all returned paths are relative (no absolute prefix).
- [x] T030 [P] [US7] Write integration test for relative paths in `tests/server_tests.rs`: run `grep` on a known file. Assert returned paths are relative.

### Implementation for User Story 7

- [x] T031 [US7] Fix path serialization in `src/server.rs` for `list_files` and `grep`: strip `KB_ROOT` prefix from returned paths. Paths already stored relative in BM25/vector indexes, so this may only need output formatting changes.

## Phase 10: Polish & Quality Gates

**Purpose**: Documentation, cleanup, and final verification

- [x] T032 [P] Update `ARCHITECTURE.md` with: search chunk filtering logic, graph link extraction (both formats), symlink canonicalization, subdirectory ignore pattern fix.
- [x] T033 [P] Update `CHANGELOG.md` with all 7 fixes (2 critical, 3 medium, 2 UX).
- [x] T034 Run full quality gates: `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `cargo test --all-features` (all pass, zero warnings, zero failures). Verifies SC-005, SC-006.
- [x] T035 Verify against success criteria: SC-001 (search ≤3 sections), SC-002 (0 .venv paths), SC-003 (non-zero counts), SC-004 (non-empty connections), SC-005 (all existing tests pass), SC-006 (no clippy warnings).

## Dependencies

```
Phase 1 (Setup) → Phase 2 (Foundational) → Phases 3-9 (US1-US7) → Phase 10 (Polish)
                                                    ↕
                                            All independent (different files)
```

- **US1 (search.rs)** and **US2 (graph.rs)**: Independent — different files, no shared state
- **US3 (vault.rs)** and **US4 (vault.rs)**: Same file but different functions (`scan_files` vs `IgnorePattern::matches`). Sequential within vault.rs.
- **US5 (server.rs, bm25.rs, index.rs, maintenance.rs)**: Independent from US1-US4, US6-US7
- **US6 (server.rs, edits.rs)** and **US7 (server.rs)**: Same file but different tool handlers. Sequential within server.rs.
- **All P3 stories (US5, US6, US7)**: Independent of each other

## Parallel Execution Examples

**Phase 2**: T004-T008 all [P] — read different files in parallel

**US1 Tests**: T009-T010 [P] — different test functions, same file

**US2 Tests**: T014-T015 [P] — different test functions, same file

**US5 Implementation**: T023-T025 [P] after T022 — different functions in different files

**US7 Tests**: T029-T030 [P] — different test functions, same file

## Implementation Strategy

**MVP Scope (P1)**: Phase 1 + Phase 2 + US1 (search fix) + US2 (graph fix) + US3 (symlink fix). These three deliver the critical + high bugs. US1 alone is usable independently.

**Incremental Delivery**:
1. US1 (P1) — Search chunk filtering → test → validate
2. US2 (P1) — Graph link extraction → test → validate
3. US3 (P2) — Symlink dedup → test → validate
4. US4 (P2) — Subdirectory ignore → test → validate
5. US5 (P2) — index_status counts → test → validate
6. US6 (P3) — read_section depth → test → validate
7. US7 (P3) — Relative paths → test → validate
8. Phase 10 — Polish, docs, quality gates → merge
