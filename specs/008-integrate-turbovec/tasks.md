# Tasks: Integrate turbovec

**Input**: Design documents from `specs/008-integrate-turbovec/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/, quickstart.md

**Tests**: Test tasks are included per the spec's mandatory testing requirements (TEST-001 through TEST-006) and knowledge-loom's TDD constitution (Section III).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4, US5)
- Include exact file paths in descriptions

## Path Conventions

- **Knowledge Loom**: `src/`, `tests/` at repository root
- **Index storage**: `.knowledge-loom-index/turbovec.tvim`, `.knowledge-loom-index/turbovec_meta.bin`
- **Tests**: `tests/` with `*_tests.rs` naming convention

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add turbovec dependency, remove old deps, verify current state

- [x] T001 Verify Rust toolchain: run `rustc --version` (must be 1.70+ for turbovec) and verify `Cargo.toml` `rust-version` field is compatible (>=1.70; if >1.75, turbovec is still compatible)
- [x] T002 [P] Run `cargo fmt --all -- --check` to verify current formatting
- [x] T003 [P] Run `cargo clippy -- -D warnings` to verify current lint state
- [x] T004 [P] Run `cargo test --all-features` to verify existing tests pass
- [x] T005 Add `turbovec = "0.6"` to `Cargo.toml` dependencies
- [x] T006 [P] Change `rusqlite` to optional in `Cargo.toml`: `rusqlite = { version = "0.31", features = ["bundled"], optional = true }`
- [x] T007 [P] Change `sqlite-vec` to optional in `Cargo.toml`: `sqlite-vec = { version = "0.1", optional = true }`
- [x] T007a [P] Add `[features] migration = ["rusqlite", "sqlite-vec"]` to `Cargo.toml`

**Checkpoint**: Dependencies updated, turbovec added, old deps made optional for migration phase

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core TurbovecIndex module — data structures, error types, constructor, ID scheme, metadata storage. Required by ALL user stories.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T008 Create `src/turbovec_index.rs` with module skeleton
- [x] T009 Implement `TurbovecError` enum in `src/turbovec_index.rs` per contracts/turbovec_index.md (variants: DimensionMismatch, DuplicateId, ChunkNotFound, CorruptIndex, Io, Serialization, Embed)
- [x] T010 Implement `ChunkMetadata` struct in `src/turbovec_index.rs` per data-model.md (fields: path, heading, content, line_start, line_end, chunk_ordinal)
- [x] T011 Implement `chunk_id(path, heading, chunk_ordinal) -> u64` in `src/turbovec_index.rs` using FNV-1a 64-bit hash with \0 separators per research.md decision 2
- [x] T012 Implement `TurbovecIndex::new(kb_root, dim, bit_width)` in `src/turbovec_index.rs` — creates IdMapIndex, initializes empty metadata HashMap
- [x] T013 Implement `TurbovecIndex::add_with_ids(&self, chunks, embeddings)` in `src/turbovec_index.rs` — compute chunk IDs, store vectors in IdMapIndex, store metadata in HashMap
- [x] T014 Implement `TurbovecIndex::search_similar(&self, query_embedding, limit)` in `src/turbovec_index.rs` — call IdMapIndex::search, map IDs to ChunkMetadata, return (path, heading, content, similarity)
- [x] T015 Implement `TurbovecIndex::remove_file(&self, path)` in `src/turbovec_index.rs` — resolve all chunk IDs for a path, remove from both IdMapIndex and metadata HashMap
- [x] T016 Implement `TurbovecIndex::count(&self)` in `src/turbovec_index.rs` — return metadata.len()
- [x] T017 Register `pub mod turbovec_index;` in `src/lib.rs` (keep `pub mod index;` until wiring is complete)

**Checkpoint**: Core TurbovecIndex module exists — add, search, remove all functional. Ready for user story integration.

---

## Phase 3: User Story 1 & 2 — Vector Search + Vault Indexing (Priority: P1) 🎯 MVP

**Goal**: Users can index their vault into turbovec and search with semantic similarity. The entire core loop (index → search) works.

**Independent Test**: Index test-vault of markdown files, run a semantic query ("machine learning"), verify results sorted by similarity score, verify chunk count matches expected.

### Tests for US1 & US2

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T019 [P] [US1] Create `tests/turbovec_index_tests.rs` — `test_new_empty_index`: create index, verify count is 0, search on empty index returns empty
- [x] T020 [P] [US1] Add `test_add_and_search` to `tests/turbovec_index_tests.rs` — add 3 vectors with known cosine similarity, search, verify scores are ordered correctly
- [x] T021 [P] [US1] Add `test_add_and_remove_file` to `tests/turbovec_index_tests.rs` — add vectors for 2 files, remove one file, verify count drops and search excludes removed file
- [x] T022 [P] [US2] Add `test_index_file` to `tests/turbovec_index_tests.rs` — create temp markdown file, call index_file, verify chunks are indexed
- [x] T023 [P] [US2] Add `test_dimension_mismatch` to `tests/turbovec_index_tests.rs` — create index with dim=384, try to add 768-dim vectors, verify TurbovecError::DimensionMismatch
- [x] T023a [P] [US2] Add `test_concurrent_search_and_index` to `tests/turbovec_index_tests.rs` — spawn 2 tokio tasks: one adding vectors in a loop, one searching; verify no panics or data races after 100 iterations

### Implementation for US1 & US2

- [x] T024 [US1] Wire TurbovecIndex into `src/search.rs` SearchEngine — replace `VectorIndex` with `TurbovecIndex` in struct fields, `from_components`, and `SearchEngine::new`
- [x] T025 [US1] Update `search()` method in `src/search.rs` line ~94 — change `vector.search_similar(&query_vec, top_k * 2)` to use TurbovecIndex (return type already matches — (path, heading, content, similarity))
- [x] T026 [US1] Update `search_graph_fused_inner()` in `src/search.rs` lines 257-258 — change `vector.search_similar(query_vec, top_k * 2)` to use TurbovecIndex
- [x] T026a Delete `src/index.rs` (old sqlite-vec VectorIndex) — safe now that all references are replaced
- [x] T026b Update `src/lib.rs` — remove `pub mod index;` now that `index.rs` is deleted
- [x] T027 [US2] Implement `TurbovecIndex::index_file(&self, path, content, embed_provider)` in `src/turbovec_index.rs` per contracts/turbovec_index.md — parse chunks, embed batch, add_with_ids
- [x] T028 [US2] Implement `TurbovecIndex::index_vault(&self, vault_state, embed_provider)` in `src/turbovec_index.rs` per contracts/turbovec_index.md — iterate files, call index_file for each
- [x] T029 [US2] Update `src/server.rs` LoomServer — replace `VectorIndex` field with `TurbovecIndex`, update initialization to pass `dim` and `bit_width` from embed provider
- [x] T030 [US2] Update `src/daemon.rs` — replace `VectorIndex` references with `TurbovecIndex`, ensure file-watch reindex triggers use turbovec
- [x] T031 [US2] Update `src/maintenance.rs` — replace `VectorIndex` references with `TurbovecIndex`, update `loom_index_status` to report turbovec stats
- [x] T032 Run `cargo build` and fix all compilation errors from the VectorIndex → TurbovecIndex transition

**Checkpoint**: US1 + US2 fully functional — user can index a vault, search semantically, add/remove individual files

---

## Phase 4: User Story 3 — Graph-Aware Filtered Search (Priority: P2)

**Goal**: Search results can be scoped to graph-connected notes via turbovec's allowlist mechanism. Enhances `search_graph_fused_inner` with true sub-index scoping rather than post-hoc re-ranking.

**Independent Test**: Index a vault with interlinked notes, search with a context note, verify results are constrained to graph neighbors.

### Tests for US3

- [x] T033 [P] [US3] Add `test_allowlist_search` to `tests/turbovec_index_tests.rs` — add 10 vectors with known IDs, search with allowlist of 3 IDs, verify only allowed IDs returned
- [x] T034 [P] [US3] Add `test_empty_allowlist_fallback` to `tests/turbovec_index_tests.rs` — search with empty allowlist, verify unfiltered results returned
- [x] T035 [P] [US3] Add `test_orphan_note_fallback` to `tests/turbovec_index_tests.rs` — search with context note that has no graph neighbors, verify full-index fallback

### Implementation for US3

- [x] T036 [US3] Implement `TurbovecIndex::search_filtered(&self, query_embedding, limit, allowed_ids)` in `src/turbovec_index.rs` per contracts/turbovec_index.md — call IdMapIndex::search with allowlist, map results
- [x] T037 [US3] Add optional `context_note: Option<&str>` parameter to `search_graph_fused_inner()` in `src/search.rs` per contracts/turbovec_index.md
- [x] T038 [US3] In `search_graph_fused_inner()` — when `context_note` is Some, resolve graph neighbors via `self.graph`, map to chunk IDs, call `search_filtered` with allowlist
- [x] T039 [US3] In `search_graph_fused_inner()` — when `context_note` is None (orphan note or no context), fall back to unfiltered `search_similar`

**Checkpoint**: US3 functional — graph-aware filtered search works, orphan notes fall back gracefully

---

## Phase 5: User Story 4 — Index Persistence Across Restarts (Priority: P2)

**Goal**: The turbovec index survives server restarts via disk persistence. Save/load works correctly.

**Independent Test**: Index a vault, save to disk, restart server, verify search works immediately without reindexing.

### Tests for US4

- [x] T040 [P] [US4] Add `test_persistence_roundtrip` to `tests/turbovec_index_tests.rs` — add 10k vectors, save, time the load call, assert load time < 2s, verify count matches and search returns same results
- [x] T041 [P] [US4] Add `test_corrupt_index_fallback` to `tests/turbovec_index_tests.rs` — try to load from garbage file, verify fresh empty index created without panic

### Implementation for US4

- [x] T042 [US4] Implement `TurbovecIndex::save(&self)` in `src/turbovec_index.rs` — call `index.write("turbovec.tvim")`, serialize metadata to `turbovec_meta.bin` via bincode
- [x] T043 [US4] Implement `TurbovecIndex::load(kb_root, dim, bit_width)` in `src/turbovec_index.rs` — call `IdMapIndex::load("turbovec.tvim")`, deserialize metadata from `turbovec_meta.bin`, rebuild HashMap
- [x] T044 [US4] Implement `TurbovecIndex::new_or_load()` in `src/turbovec_index.rs` — check if `.tvim` file exists; if yes load, if no create fresh
- [x] T045 [US4] Update `TurbovecIndex::index_vault` to call `save()` after full reindex completes
- [x] T046 [US4] Update `TurbovecIndex::index_file` to call `save()` after individual file index/update
- [x] T047 [US4] Update `TurbovecIndex::remove_file` to call `save()` after file removal
- [x] T048 [US4] Verify correctness: after save/load cycle, `metadata.len() == index.len()` (parity check from data-model.md validation rules)

**Checkpoint**: US4 functional — index persists across restarts, corrupt index falls back gracefully

---

## Phase 6: User Story 5 — Memory Efficiency & Quantization Config (Priority: P3)

**Goal**: Users can configure the quantization level (2-bit or 4-bit). Memory usage is significantly reduced vs raw float32.

**Independent Test**: Index 50k chunks at 4-bit, verify memory usage is <= 25% of raw float32 storage.

### Tests for US5

- [x] T049 [P] [US5] Add `test_quantization_config` to `tests/turbovec_index_tests.rs` — create index with bit_width=2, verify it works; create with bit_width=4, verify it works
- [x] T050 [P] [US5] Add `test_memory_estimate` to `tests/turbovec_index_tests.rs` — index N vectors, verify index.len() matches, estimate compressed size (N * dim * bit_width / 8 bytes + metadata overhead)

### Implementation for US5

- [x] T051 [US5] Make bit_width configurable via environment variable `LOOM_TURBOVEC_BIT_WIDTH` in `src/turbovec_index.rs` (default 4, accept 2 or 4)
- [x] T052 [US5] Validate bit_width at construction time — panic/reject on invalid values (not 2 or 4)
- [x] T053 [US5] Update `src/server.rs` LoomServer initialization to pass configured bit_width to TurbovecIndex
- [x] T054 [US5] Document quantization config in CHANGELOG.md (via Phase 7 doc task)

**Checkpoint**: US5 functional — bit width configurable, memory savings visible

---

## Phase 7: Migration (Cross-Cutting)

**Purpose**: One-time automatic migration from sqlite-vec embeddings to turbovec per spec FR-011. This is a cross-cutting concern that touches startup flow.

- [x] T055 Implement `TurbovecIndex::migrate_from_sqlite(kb_root, index, metadata)` in `src/turbovec_index.rs` per contracts/turbovec_index.md — open embeddings.db via rusqlite (compile-time optional), read all rows, compute chunk IDs, call add_with_ids, verify counts, delete .db
- [x] T056 Add migration trigger to `TurbovecIndex::new_or_load()` — if embeddings.db exists and turbovec.tvim does NOT exist, run migrate_from_sqlite before first search
- [x] T057 Add migration logging via `eprintln!` — "Migrating N chunks from sqlite-vec to turbovec..." and "Migration complete: N chunks, deleting embeddings.db"
- [x] T058 [P] Add `test_migration` to `tests/turbovec_index_tests.rs` — create a temporary sqlite-vec db with known embeddings, run migration, verify turbovec index parity (same count, same embeddings produce same search results)
- [x] T058a (SKIPPED: migration feature flag retained as documented design decision per research.md) Remove `rusqlite` and `sqlite-vec` from `Cargo.toml` dependencies entirely (remove optional entries and `[features] migration` section)

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, quality gates, final verification

- [x] T059 [P] Add doc comments (`///`) to all public functions and structs in `src/turbovec_index.rs`
- [x] T060 [P] Update `ARCHITECTURE.md` — replace sqlite-vec references with turbovec, document new TurbovecIndex module
- [x] T061 [P] Update `CHANGELOG.md` — add entry for turbovec integration with migration, quantization, filtered search
- [x] T062 [P] Remove or rewrite `tests/vector_tests.rs` — old sqlite-vec tests; if any test logic is reusable, port to turbovec_index_tests.rs
- [x] T063 Run `cargo fmt --all -- --check` and fix any formatting issues
- [x] T064 Run `cargo clippy -- -D warnings` and fix any linting issues
- [x] T065 Run `cargo test --all-features` and ensure all tests pass
- [x] T066 Run `cargo deny check licenses bans sources` — verify turbovec (MIT) passes audit
- [ ] T067 Run code coverage (requires cargo-tarpaulin — run separately) check — minimum 80% line coverage for new `src/turbovec_index.rs`
- [x] T067a [P] Add `test_recall_at_10` to `tests/turbovec_index_tests.rs` — index >= 1000 vectors, run turbovec search, run exact cosine-scan search, verify fraction of top-10 exact results that appear in turbovec top-10 is >= 0.95
- [x] T067b [P] Add `test_search_latency` to `tests/turbovec_index_tests.rs` — index 10k vectors, time 100 search queries, assert p95 < 100ms (per Constitution VII and plan Performance Goals)
- [x] T068 Run quickstart.md validation — follow the 10-step quickstart guide and verify it works end-to-end
- [x] T069 Verify MCP protocol compliance — run `tests/mcp_protocol_tests.rs`, confirm search tools still return expected shapes
- [x] T070 Verify `cargo build --release` compiles without warnings

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS all user stories
- **US1+US2 (Phase 3)**: Depends on Foundational — core search + indexing loop
- **US3 (Phase 4)**: Depends on US1+US2 (needs working search + index to add allowlist)
- **US4 (Phase 5)**: Depends on US1+US2 (needs working add/search to test save/load)
- **US5 (Phase 6)**: Depends on Foundational (config logic touches construction only)
- **Migration (Phase 7)**: Depends on US4 (needs save/load to verify migration persisted)
- **Polish (Phase 8)**: Depends on all user stories complete

### User Story Dependencies

- **US1+US2 (P1)**: Can start after Foundational — no dependencies on other stories
- **US3 (P2)**: Depends on US1+US2 for working `search_similar` and `index_file`
- **US4 (P2)**: Depends on US1+US2 for working `add_with_ids` and metadata
- **US5 (P3)**: Only depends on Foundational (constructor logic)

### Within Each Phase

- Tests MUST be written and FAIL before implementation
- Core module methods before wiring into search/server/daemon
- Implementation before quality gate verification

### Parallel Opportunities

- Phase 1: T002, T003, T004 can run in parallel
- Phase 2: T009, T010, T011 can run in parallel (different concerns)
- Phase 3 tests: T019-T023 can all run in parallel
- Phase 5 tests: T040, T041 can run in parallel
- Phase 8: T059, T060, T061, T062 can run in parallel

---

## Parallel Example: Phase 3 (US1 & US2)

```bash
# Step 1: Launch all tests in parallel (these MUST fail first):
Task: "T019 Create tests/turbovec_index_tests.rs — test_new_empty_index"
Task: "T020 Add test_add_and_search to tests/turbovec_index_tests.rs"
Task: "T021 Add test_add_and_remove_file to tests/turbovec_index_tests.rs"
Task: "T022 Add test_index_file to tests/turbovec_index_tests.rs"
Task: "T023 Add test_dimension_mismatch to tests/turbovec_index_tests.rs"

# Step 2: After tests written and failing, implement core methods:
Task: "T027 Implement index_file in src/turbovec_index.rs"
Task: "T028 Implement index_vault in src/turbovec_index.rs"

# Step 3: After core methods, wire into SearchEngine:
Task: "T024 Wire TurbovecIndex into src/search.rs"
Task: "T025 Update search() in src/search.rs"
Task: "T026 Update search_graph_fused_inner() in src/search.rs"

# Step 4: Wire into infrastructure:
Task: "T029 Update src/server.rs"
Task: "T030 Update src/daemon.rs"
Task: "T031 Update src/maintenance.rs"

# Step 5: Fix compilation:
Task: "T032 Run cargo build and fix errors"
```

---

## Implementation Strategy

### MVP First (US1 + US2 Only)

1. Complete Phase 1: Setup — dependencies updated, existing tests pass
2. Complete Phase 2: Foundational — TurbovecIndex core module exists
3. Complete Phase 3: US1 + US2 — search and indexing work end-to-end
4. **STOP and VALIDATE**: Index test-vault, search semantically, verify results
5. Run quality gates: `cargo fmt`, `cargo clippy`, `cargo test`
6. Deploy/demo if ready (MVP delivers working search with memory savings)

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. US1 + US2 → Core search + indexing → **MVP!**
3. US3 → Graph-aware filtered search → Deploy/Demo
4. US4 → Index persistence → Deploy/Demo
5. US5 → Configurable quantization → Deploy/Demo
6. Migration + Polish → Release-ready

### Quality Gates (Must Pass Before Merge)

- **Formatting**: `cargo fmt --all -- --check` must pass
- **Linting**: `cargo clippy -- -D warnings` must pass
- **Testing**: `cargo test --all-features` must pass
- **Coverage**: Minimum 80% line coverage
- **Security**: `cargo deny check licenses bans sources` must pass
- **MCP**: Protocol tests must pass

---

## Notes

- [P] tasks = different files, no dependencies, can run in parallel
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing (TDD per Constitution)
- **EXPLICIT CONSENT REQUIRED**: Each git commit requires individual user consent (per Constitution)
- Run `cargo fmt` before committing
- Run `cargo clippy` before committing
- Run `cargo test` before committing
- Minimum 80% code coverage required for merge
- Use `test-vault/` for corpus-based testing when applicable
- Use `tempfile` for file system tests
- Stop at any checkpoint to validate story independently
- The migration task (T055) requires rusqlite as a compile-time optional for reading old dbs — add `rusqlite = { version = "0.31", optional = true }` with a `migration` feature flag, or read sqlite directly via the `sqlite` crate
