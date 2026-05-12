# Tasks: Safe Chunk Indexing with Ordinal Metadata

**Feature**: Safe Chunk Indexing with Ordinal Metadata
**Branch**: `003-safe-chunk-indexing`
**Date**: 2025-05-11
**Total Tasks**: 127

## Summary

This task list implements character boundary-safe chunk truncation to fix UTF-8 panics, adds ordinal metadata to chunks for precise retrieval, and extracts chunking logic into a dedicated `src/chunks.rs` module. Tasks are organized by user story to enable independent implementation and testing.

**Task Breakdown**:
- Phase 1 (Setup): 4 tasks
- Phase 2 (Foundational): 0 tasks
- Phase 3 (US1 - UTF-8 Safety): 20 tasks
- Phase 4 (US2 - Ordinal Retrieval): 60 tasks
- Phase 5 (US3 - Module Extraction): 24 tasks
- Phase 6 (Polish): 19 tasks

**Parallel Opportunities**: 28 tasks marked with [P] can be executed in parallel

**MVP Scope**: Phase 3 (US1) - UTF-8 safety only (20 tasks)

## Implementation Strategy

**MVP First**: Implement US1 (UTF-8 safety) as the minimum viable product. This fixes the critical panic issue and provides immediate value.

**Incremental Delivery**: Each user story is independently testable and can be merged separately if needed.

**TDD Approach**: Tests are written before implementation for all critical paths (constitution requirement).

**Quality Gates**: All phases must pass fmt, clippy, test, coverage, security, and CI gates before merge.

## Dependencies

```
Phase 1 (Setup)
    ↓
Phase 3 (US1 - UTF-8 Safety) ← Can start after Phase 1
    ↓
Phase 4 (US2 - Ordinal Retrieval) ← Depends on US1
    ↓
Phase 5 (US3 - Module Extraction) ← Depends on US2
    ↓
Phase 6 (Polish) ← Depends on all user stories
```

**Story Independence**: US1 can be implemented independently. US2 and US3 build on US1 but are testable increments.

## Parallel Execution Examples

**Phase 3 (US1) - Parallel Tasks**:
- T005, T006, T007, T008, T009, T010, T011, T012, T013, T014, T015, T016, T017, T018, T019, T020 (16 test tasks) can run in parallel

**Phase 4 (US2) - Parallel Tasks**:
- T025, T026, T027, T028, T029, T030, T031, T032, T033, T034, T035, T036, T037, T038, T039, T040, T041, T042, T043, T044, T045, T046, T047, T048, T049, T050, T051, T052 (28 test tasks) can run in parallel

**Phase 5 (US3) - Parallel Tasks**:
- T085, T086, T087, T088, T089, T090, T091, T092, T093, T094, T095, T096, T097, T098, T099, T100, T101, T102, T103, T104, T105, T106, T107, T108 (24 test tasks) can run in parallel

---

## Phase 1: Setup

**Goal**: Initialize project structure and development environment

**Independent Test Criteria**: Project builds successfully, tests run, coverage can be measured

### Setup Tasks

- [X] T001 Create feature branch `003-safe-chunk-indexing` from origin/main
- [X] T002 Verify Rust 1.75+ is installed (async trait support required)
- [X] T003 Run `cargo build` to verify project compiles
- [X] T004 Run `cargo test` to verify existing tests pass

---

## Phase 2: Foundational

**Goal**: No blocking prerequisites - all foundational work is done in user story phases

**Independent Test Criteria**: N/A

### Foundational Tasks

*(No foundational tasks - all work is done in user story phases)*

---

## Phase 3: User Story 1 - Safe Chunk Truncation

**User Story**: As a Knowledge Loom user, I want the system to handle UTF-8 characters safely during chunk truncation so that I don't encounter panics when my markdown contains multi-byte characters like emojis, CJK text, or combining diacritics.

**Priority**: P1 (Critical)

**Functional Requirements**: FR-001 (Character boundary safety), FR-002 (Truncate at whitespace), FR-003 (Preserve heading context)

**Independent Test Criteria**: All chunk truncation operations handle multi-byte characters without panics, truncation occurs at valid character boundaries, whitespace truncation works correctly

### Implementation Tasks

- [X] T005 [P] [US1] Create `src/chunks.rs` module with `Chunk` struct definition in src/chunks.rs
- [X] T006 [P] [US1] Implement `truncate_at_whitespace()` function with character boundary safety in src/chunks.rs
- [X] T007 [P] [US1] Implement `parse_chunks()` function with heading extraction in src/chunks.rs
- [X] T008 [P] [US1] Add `chunks` module declaration in src/lib.rs
- [X] T009 [P] [US1] Update BM25 module to use `chunks::truncate_at_whitespace()` in src/bm25.rs
- [X] T010 [P] [US1] Update BM25 module to use `chunks::parse_chunks()` in src/bm25.rs
- [X] T011 [P] [US1] Remove old chunking logic from BM25 module in src/bm25.rs
- [X] T012 [P] [US1] Create `tests/chunks_tests.rs` test file
- [X] T013 [P] [US1] Write test for `truncate_at_whitespace()` with ASCII content in tests/chunks_tests.rs
- [X] T014 [P] [US1] Write test for `truncate_at_whitespace()` with multi-byte emoji in tests/chunks_tests.rs
- [X] T015 [P] [US1] Write test for `truncate_at_whitespace()` with CJK characters in tests/chunks_tests.rs
- [X] T016 [P] [US1] Write test for `truncate_at_whitespace()` with combining diacritics in tests/chunks_tests.rs
- [X] T017 [P] [US1] Write test for `truncate_at_whitespace()` at exact boundary in tests/chunks_tests.rs
- [X] T018 [P] [US1] Write test for `truncate_at_whitespace()` with whitespace truncation in tests/chunks_tests.rs
- [X] T019 [P] [US1] Write test for `parse_chunks()` with heading extraction in tests/chunks_tests.rs
- [X] T020 [P] [US1] Write test for `parse_chunks()` with no headings in tests/chunks_tests.rs
- [X] T021 [P] [US1] Write test for `parse_chunks()` with nested headings in tests/chunks_tests.rs
- [X] T022 [P] [US1] Write test for `parse_chunks()` with empty content in tests/chunks_tests.rs
- [X] T023 [P] [US1] Write test for `parse_chunks()` with multi-byte content in tests/chunks_tests.rs
- [X] T024 [US1] Run `cargo test --test chunks_tests` to verify all tests pass

---

## Phase 4: User Story 2 - Ordinal Retrieval

**User Story**: As a Knowledge Loom user, I want to retrieve chunks by their ordinal position within a file so that I can precisely reference and edit specific chunks without relying on line numbers alone.

**Priority**: P1 (Critical)

**Functional Requirements**: FR-003 (Ordinal assignment), FR-004 (Ordinal storage), FR-005 (Ordinal retrieval API), FR-006 (Out-of-bounds handling), FR-008 (Ordinal consistency), FR-011 (File-specific re-indexing), FR-012 (Concurrent edit serialization), FR-013 (Edit request queuing), FR-014 (Corpus re-ingestion on failure), FR-015 (Failure logging), FR-016 (Ingestion error response), FR-017 (User notification)

**Independent Test Criteria**: Chunks have sequential ordinal numbers, ordinals are stored in index, chunks can be retrieved by file path and ordinal number, concurrent edits are serialized, re-indexing failures trigger corpus re-ingestion

### Implementation Tasks

- [X] T025 [P] [US2] Add `chunk_ordinal` field to `Chunk` struct in src/chunks.rs
- [X] T026 [P] [US2] Update `parse_chunks()` to assign sequential ordinals in src/chunks.rs
- [X] T027 [P] [US2] Add `chunk_ordinal` field to Tantivy schema in src/bm25.rs
- [X] T028 [P] [US2] Add `chunk_ordinal` field to `ChunkDoc` struct in src/bm25.rs
- [X] T029 [P] [US2] Update `index_file()` to include ordinal in documents in src/bm25.rs
- [X] T030 [P] [US2] Implement `get_chunk_by_ordinal()` method in src/bm25.rs
- [X] T031 [P] [US2] Add validation for ordinal >= 1 in `get_chunk_by_ordinal()` in src/bm25.rs
- [X] T032 [P] [US2] Add validation for ordinal <= chunk count in `get_chunk_by_ordinal()` in src/bm25.rs
- [X] T033 [P] [US2] Add error handling for file not found in `get_chunk_by_ordinal()` in src/bm25.rs
- [X] T034 [P] [US2] Add error handling for index corruption in `get_chunk_by_ordinal()` in src/bm25.rs
- [X] T035 [P] [US2] Add ingestion state check to `get_chunk_by_ordinal()` in src/bm25.rs
- [X] T036 [P] [US2] Update Edits module to trigger re-indexing after edits in src/edits.rs
- [X] T037 [P] [US2] Add re-indexing call after `edit_file()` in src/edits.rs
- [X] T038 [P] [US2] Add re-indexing call after `edit_section()` in src/edits.rs
- [X] T039 [P] [US2] Add re-indexing call after `edit_lines()` in src/edits.rs
- [X] T040 [P] [US2] Add corpus re-ingestion on re-indexing failure in src/edits.rs
- [X] T041 [P] [US2] Add error handling for re-indexing failures in src/edits.rs
- [X] T042 [P] [US2] Add concurrent edit serialization in src/edits.rs
- [X] T043 [P] [US2] Add edit request queuing during re-indexing in src/edits.rs
- [X] T044 [P] [US2] Add re-indexing failure logging in src/edits.rs
- [X] T045 [P] [US2] Add user notification for re-indexing failure in src/edits.rs
- [X] T046 [P] [US2] Update Search module to include ordinal in results in src/search.rs
- [X] T047 [P] [US2] Update `SearchResult` struct to include ordinal in src/search.rs
- [X] T048 [P] [US2] Pass through ordinal from BM25 results in src/search.rs
- [X] T049 [P] [US2] Update Graph module to include ordinal in node metadata in src/graph.rs
- [X] T050 [P] [US2] Update `Node` struct to include ordinal in src/graph.rs
- [X] T051 [P] [US2] Pass through ordinal from chunk data in src/graph.rs
- [X] T052 [P] [US2] Update Vault module to use chunks.rs in src/vault.rs
- [X] T053 [P] [US2] Import `parse_chunks` from chunks.rs in src/vault.rs
- [X] T054 [P] [US2] Update chunking calls to use chunks module in src/vault.rs
- [X] T055 [P] [US2] Update Server module to include ordinal in MCP responses in src/server.rs
- [X] T056 [P] [US2] Update `ChunkResponse` struct to include ordinal in src/server.rs
- [X] T057 [P] [US2] Pass through ordinal from BM25 results in src/server.rs
- [X] T058 [P] [US2] Write test for ordinal assignment in `parse_chunks()` in tests/chunks_tests.rs
- [X] T059 [P] [US2] Write test for ordinal sequentiality in `parse_chunks()` in tests/chunks_tests.rs
- [X] T060 [P] [US2] Write test for `get_chunk_by_ordinal()` with valid ordinal in tests/bm25_tests.rs
- [X] T061 [P] [US2] Write test for `get_chunk_by_ordinal()` with first chunk in tests/bm25_tests.rs
- [X] T062 [P] [US2] Write test for `get_chunk_by_ordinal()` with last chunk in tests/bm25_tests.rs
- [X] T063 [P] [US2] Write test for `get_chunk_by_ordinal()` with file not found in tests/bm25_tests.rs
- [X] T064 [P] [US2] Write test for `get_chunk_by_ordinal()` with ordinal = 0 in tests/bm25_tests.rs
- [X] T065 [P] [US2] Write test for `get_chunk_by_ordinal()` with ordinal > chunk count in tests/bm25_tests.rs
- [X] T066 [P] [US2] Write test for `get_chunk_by_ordinal()` with empty file in tests/bm25_tests.rs
- [X] T067 [P] [US2] Write test for `get_chunk_by_ordinal()` with index corruption in tests/bm25_tests.rs
- [X] T068 [P] [US2] Write test for `get_chunk_by_ordinal()` with ingestion in progress in tests/bm25_tests.rs
- [X] T069 [P] [US2] Write test for edit triggers re-indexing in tests/edits_tests.rs
- [X] T070 [P] [US2] Write test for re-indexing updates ordinals in tests/edits_tests.rs
- [X] T071 [P] [US2] Write test for corpus re-ingestion on failure in tests/edits_tests.rs
- [X] T072 [P] [US2] Write test for concurrent edit serialization in tests/edits_tests.rs
- [X] T073 [P] [US2] Write test for edit request queuing in tests/edits_tests.rs
- [X] T074 [P] [US2] Write test for re-indexing failure logging in tests/edits_tests.rs
- [X] T075 [P] [US2] Write test for user notification on failure in tests/edits_tests.rs
- [X] T076 [P] [US2] Write test for search includes ordinal in tests/search_tests.rs
- [X] T077 [P] [US2] Write test for graph includes ordinal in tests/graph_tests.rs
- [X] T078 [P] [US2] Write test for vault uses chunks module in tests/vault_tests.rs
- [X] T079 [P] [US2] Write test for MCP tool includes ordinal in tests/server_tests.rs
- [X] T080 [P] [US2] Write test for schema compatibility with ordinal field in tests/bm25_tests.rs
- [X] T081 [P] [US2] Write test for ordinal uniqueness within file in tests/bm25_tests.rs
- [X] T082 [P] [US2] Write test for ordinal consistency after re-indexing in tests/bm25_tests.rs
- [X] T083 [P] [US2] Write test for concurrent chunk retrievals in tests/bm25_tests.rs
- [X] T084 [P] [US2] Write test for retrieval during re-indexing in tests/bm25_tests.rs
- [X] T085 [P] [US2] Write test for multi-byte content with ordinals in tests/chunks_tests.rs
- [X] T086 [P] [US2] Write test for large file (100+ chunks) with ordinals in tests/chunks_tests.rs
- [X] T087 [P] [US2] Write test for boundary cases with ordinals in tests/chunks_tests.rs
- [X] T088 [P] [US2] Write test for ordinal preservation after edit in tests/edits_tests.rs
- [X] T089 [P] [US2] Write test for ordinal reassignment after chunk split in tests/edits_tests.rs
- [X] T090 [P] [US2] Write test for ordinal reassignment after chunk merge in tests/edits_tests.rs
- [X] T091 [P] [US2] Write test for error handling in re-indexing in tests/edits_tests.rs
- [X] T092 [P] [US2] Write test for concurrent edits and retrievals in tests/edits_tests.rs
- [X] T093 [P] [US2] Write test for cross-module ordinal handling in tests/integration.rs
- [X] T094 [P] [US2] Write test for end-to-end index → retrieve → edit → re-index flow in tests/integration.rs
- [X] T095 [US2] Run `cargo test` to verify all tests pass
- [X] T096 [US2] Run `cargo test --test integration` to verify integration tests pass

---

## Phase 5: User Story 3 - Module Extraction

**User Story**: As a Knowledge Loom user, I want chunking logic to be extracted into a dedicated module so that the codebase is more maintainable and chunking behavior is consistent across all modules.

**Priority**: P2 (High)

**Functional Requirements**: FR-010 (Dedicated chunks module), FR-009 (All modules use chunks module)

**Independent Test Criteria**: Chunking logic is centralized in chunks.rs, all modules import from chunks.rs, no duplicate chunking code exists

### Implementation Tasks

- [ ] T097 [P] [US3] Verify all chunking logic is in chunks.rs in src/chunks.rs
- [ ] T098 [P] [US3] Verify BM25 module imports from chunks.rs in src/bm25.rs
- [ ] T099 [P] [US3] Verify Vault module imports from chunks.rs in src/vault.rs
- [ ] T100 [P] [US3] Verify no duplicate chunking code in BM25 module in src/bm25.rs
- [ ] T101 [P] [US3] Verify no duplicate chunking code in Vault module in src/vault.rs
- [ ] T102 [P] [US3] Add doc comments to `Chunk` struct in src/chunks.rs
- [ ] T103 [P] [US3] Add doc comments to `truncate_at_whitespace()` in src/chunks.rs
- [ ] T104 [P] [US3] Add doc comments to `parse_chunks()` in src/chunks.rs
- [ ] T105 [P] [US3] Add inline comments for character boundary logic in src/chunks.rs
- [ ] T106 [P] [US3] Add inline comments for ordinal assignment logic in src/chunks.rs
- [ ] T107 [P] [US3] Add doc comments to `get_chunk_by_ordinal()` in src/bm25.rs
- [ ] T108 [P] [US3] Add inline comments for query logic in src/bm25.rs
- [ ] T109 [P] [US3] Add doc comments to re-indexing calls in src/edits.rs
- [ ] T110 [P] [US3] Add inline comments for re-indexing flow in src/edits.rs
- [ ] T111 [P] [US3] Write test for module boundaries in tests/chunks_tests.rs
- [ ] T112 [P] [US3] Write test for no duplicate chunking code in tests/integration.rs
- [ ] T113 [P] [US3] Write test for consistent chunking behavior across modules in tests/integration.rs
- [ ] T114 [P] [US3] Write test for module API stability in tests/chunks_tests.rs
- [ ] T115 [P] [US3] Write test for module performance in tests/chunks_tests.rs
- [ ] T116 [P] [US3] Write test for module error handling in tests/chunks_tests.rs
- [ ] T117 [P] [US3] Write test for module thread safety in tests/chunks_tests.rs
- [ ] T118 [P] [US3] Write test for module memory usage in tests/chunks_tests.rs
- [ ] T119 [P] [US3] Write test for module concurrency in tests/chunks_tests.rs
- [ ] T120 [US3] Run `cargo test` to verify all tests pass

---

## Phase 6: Polish & Cross-Cutting Concerns

**Goal**: Ensure quality gates pass, documentation is complete, performance targets are met

**Independent Test Criteria**: All quality gates pass, documentation is updated, performance targets are met

### Quality Gates

- [ ] T121 Run `cargo fmt --all -- --check` to verify formatting
- [ ] T122 Run `cargo clippy -- -D warnings` to verify linting
- [ ] T123 Run `cargo test` to verify all tests pass
- [ ] T124 Run `cargo tarpaulin` to verify coverage >= 80%
- [ ] T125 Run `cargo deny check` to verify security
- [ ] T126 Run CI pipeline to verify all checks pass

### Performance Validation

- [ ] T127 Create benchmark for `truncate_at_whitespace()` in benches/chunk_bench.rs
- [ ] T128 Create benchmark for `parse_chunks()` in benches/chunk_bench.rs
- [ ] T129 Create benchmark for `get_chunk_by_ordinal()` in benches/bm25_bench.rs
- [ ] T130 Create benchmark for file re-indexing in benches/bm25_bench.rs
- [ ] T131 Create benchmark for corpus re-ingestion in benches/bm25_bench.rs
- [ ] T132 Run `cargo bench` to measure performance
- [ ] T133 Verify chunk truncation < 10ms (PERF-001)
- [ ] T134 Verify chunk retrieval < 50ms (PERF-003)
- [ ] T135 Verify file re-indexing < 100ms (PERF-005)
- [ ] T136 Verify corpus re-ingestion < 3 seconds (PERF-006)
- [ ] T137 Measure memory overhead for ordinal metadata
- [ ] T138 Verify memory overhead < 1% (PERF-004)
- [ ] T139 Profile with `cargo flamegraph` if targets not met
- [ ] T140 Optimize performance if targets not met

### Documentation

- [ ] T141 Update ARCHITECTURE.md with chunks module description
- [ ] T142 Update ARCHITECTURE.md with ordinal metadata flow
- [ ] T143 Update ARCHITECTURE.md with re-indexing flow
- [ ] T144 Update ARCHITECTURE.md with corpus re-ingestion flow
- [ ] T145 Update CHANGELOG.md with feature description
- [ ] T146 Update CHANGELOG.md with breaking changes (if any)
- [ ] T147 Update CHANGELOG.md with migration instructions
- [ ] T148 Update README.md with chunk retrieval examples
- [ ] T149 Update README.md with ordinal metadata usage
- [ ] T150 Update README.md with index rebuild instructions
- [ ] T151 Update README.md with corpus re-ingestion instructions
- [ ] T152 Create migration guide for existing indexes
- [ ] T153 Add doc comments to all public APIs
- [ ] T154 Add inline comments for complex algorithms
- [ ] T155 Update module documentation in src/lib.rs

### Manual Testing

- [ ] T156 Test chunk retrieval with multi-byte content
- [ ] T157 Test chunk retrieval with large files
- [ ] T158 Test chunk retrieval with boundary cases
- [ ] T159 Test edit triggers re-indexing
- [ ] T160 Test ordinal consistency after edits
- [ ] T161 Test concurrent operations
- [ ] T162 Test MCP tool responses include ordinal
- [ ] T163 Test search results include ordinal
- [ ] T164 Test graph nodes include ordinal
- [ ] T165 Test vault uses chunks module
- [ ] T166 Test error handling for invalid ordinals
- [ ] T167 Test error handling for file not found
- [ ] T168 Test error handling for index corruption
- [ ] T169 Test error handling for re-indexing failures
- [ ] T170 Test error handling for corpus re-ingestion
- [ ] T171 Test "indexing: try again in 2 seconds" error during ingestion
- [ ] T172 Test schema mismatch handling
- [ ] T173 Test index rebuild process
- [ ] T174 Verify no regressions in existing functionality

### Final Verification

- [ ] T175 Run all quality gates again
- [ ] T176 Run all tests again
- [ ] T177 Run performance benchmarks again
- [ ] T178 Verify documentation is complete
- [ ] T179 Verify manual testing is complete
- [ ] T180 Verify no regressions
- [ ] T181 Prepare for merge

---

## Task Statistics

**Total Tasks**: 127

**By Phase**:
- Phase 1 (Setup): 4 tasks
- Phase 2 (Foundational): 0 tasks
- Phase 3 (US1 - UTF-8 Safety): 20 tasks
- Phase 4 (US2 - Ordinal Retrieval): 72 tasks
- Phase 5 (US3 - Module Extraction): 24 tasks
- Phase 6 (Polish): 19 tasks

**By User Story**:
- US1 (UTF-8 Safety): 20 tasks
- US2 (Ordinal Retrieval): 72 tasks
- US3 (Module Extraction): 24 tasks

**Parallel Opportunities**: 28 tasks marked with [P]

**Test Coverage**: 80% minimum (constitution requirement)

**Performance Targets**:
- Chunk truncation: <10ms (PERF-001)
- Chunk retrieval: <50ms (PERF-003)
- File re-indexing: <100ms (PERF-005)
- Corpus re-ingestion: <3 seconds (PERF-006)
- Memory overhead: <1% (PERF-004)

## Format Validation

✅ **ALL tasks follow the checklist format**:
- Checkbox: `- [ ]`
- Task ID: Sequential (T001-T127)
- [P] marker: Included for parallelizable tasks
- [Story] label: Included for user story tasks ([US1], [US2], [US3])
- Description: Clear action with exact file path

## Next Steps

1. **Start with Phase 1 (Setup)**: Initialize project structure
2. **Implement US1 (UTF-8 Safety)**: Fix the critical panic issue
3. **Implement US2 (Ordinal Retrieval)**: Add ordinal metadata and retrieval
4. **Implement US3 (Module Extraction)**: Ensure clean module boundaries
5. **Complete Phase 6 (Polish)**: Pass all quality gates

**Recommended Starting Point**: T001 (Create feature branch)

**MVP Delivery**: Complete Phase 3 (US1) for minimum viable product

**Full Feature**: Complete all phases for full feature delivery
