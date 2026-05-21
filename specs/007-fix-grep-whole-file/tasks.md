# Tasks: Fix Grep Whole-File Returns

**Input**: Design documents from `specs/007-fix-grep-whole-file/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), data-model.md, research.md

**Tests**: Included — TDD is mandatory per constitution (Section III: Test-First Development).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Source**: `src/edits.rs`, `src/server.rs`
- **Tests**: `tests/search_tests.rs`, `tests/server_tests.rs`

---

## Phase 1: Setup (Verify Baseline)

**Purpose**: Ensure clean working state before any changes

- [x] T001 Verify Rust toolchain: `rustc --version` (must be 1.75+)
- [x] T002 Run `cargo test --all-features` to verify all existing tests pass
- [x] T003 Run `cargo fmt --all -- --check` and `cargo clippy -- -D warnings` to verify baseline

---

## Phase 2: Foundational — GrepResponse Struct

**Purpose**: Define the response types shared by all user stories. MUST complete before any user story.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T004 [P] Define `GrepMatch` struct with `file`, `line_number`, `line_text` fields and `#[derive(Serialize)]` in `src/edits.rs`
- [x] T005 Define `GrepResponse` struct with `matches: Vec<GrepMatch>`, `truncated: bool`, `total_matches: usize` and `#[derive(Serialize)]` in `src/edits.rs`
- [x] T006 Update existing `grep` function return type from `Vec<(String, usize, String)>` to `GrepResponse` in `src/edits.rs` (wrap current results in the new struct, set `truncated: false`, `total_matches: matches.len()`)
- [x] T007 Update `server.rs` grep handler to serialize the new `GrepResponse` struct correctly

**Checkpoint**: Foundation ready — `loom_grep` still works with the new struct wrapper. Existing tests pass.

---

## Phase 3: User Story 1 - Scoped Grep Results (Priority: P1) 🎯 MVP

**Goal**: Grep results are capped at a default limit of 200, with an optional `limit` parameter to override (0 = no limit). Truncation metadata included in response.

**Independent Test**: Run `loom_grep` with a broad pattern (e.g., `.`) and verify at most 200 results are returned with `truncated: true` and correct `total_matches`.

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T008 [P] [US1] Test: default limit caps results at 200 in `tests/search_tests.rs` — create vault with 3 files × 100 matching lines each, call `grep("note", None, 200)`, assert `matches.len() <= 200`, `truncated == true`, `total_matches == 300`
- [x] T009 [P] [US1] Test: explicit limit=20 returns exactly 20 in `tests/search_tests.rs` — vault with 100 matching lines across 5 files, call `grep(".", None, 20)`, assert `matches.len() == 20`, `truncated == true`
- [x] T010 [P] [US1] Test: limit=0 returns all matches uncapped in `tests/search_tests.rs` — vault with 50 matching lines, call `grep("match", None, 0)`, assert `matches.len() == 50`, `truncated == false`
- [x] T011 [P] [US1] Test: no matches returns empty with correct metadata in `tests/search_tests.rs` — vault with no matching lines, call `grep("noexist", None, 200)`, assert `matches.is_empty()`, `truncated == false`, `total_matches == 0`

### Implementation for User Story 1

- [x] T012 [US1] Add `limit: usize` parameter to `EditManager::grep()` signature in `src/edits.rs`
- [x] T013 [US1] Implement limit enforcement in grep loop: skip push when `matches.len() >= limit` (unless `limit == 0`), continue counting for `total_matches` in `src/edits.rs`
- [x] T014 [US1] Compute `truncated` as `total_matches > matches.len()` in `src/edits.rs`
- [x] T015 [US1] Update MCP tool schema for "grep" in `src/server.rs`: add `limit` parameter with `type: "integer"`, `default: 200`, description
- [x] T016 [US1] Extract `limit` parameter from args in grep handler, pass to `edits.grep()` in `src/server.rs` — default to 200 when absent, handle `limit=0` → uncapped

**Checkpoint**: User Story 1 complete — grep returns capped results with truncation metadata. Tests pass.

---

## Phase 4: User Story 2 - File Filter Narrowing (Priority: P2)

**Goal**: `file_filter` parameter filters files by substring match on path before scanning.

**Independent Test**: Run `loom_grep` with `file_filter="notes"` in a vault with multiple files — verify only matches from files containing "notes" in path are returned.

### Tests for User Story 2

- [x] T017 [P] [US2] Test: file_filter restricts to matching files in `tests/search_tests.rs` — vault with "a/notes.md" (has "TODO"), "b/notes.md" (has "TODO"), "c/other.md" (has "TODO"), call `grep("TODO", Some("notes"), 200)`, assert only 2 results, file paths contain "notes"
- [x] T018 [P] [US2] Test: file_filter with no matches returns empty in `tests/search_tests.rs` — vault with "a.md" (has "hello"), call `grep("hello", Some("nonexistent"), 200)`, assert empty
- [x] T019 [P] [US2] Test: empty file_filter (Some("")) behaves like None in `tests/search_tests.rs` — verify all files searched

### Implementation for User Story 2

- [x] T020 [US2] Add `file_filter: Option<&str>` parameter to `EditManager::grep()` signature in `src/edits.rs`
- [x] T021 [US2] Implement file_filter check in file iteration loop: skip file if `file_filter` present and file path does not contain the filter string in `src/edits.rs`
- [x] T022 [US2] Extract `file_filter` from args in grep handler in `src/server.rs` using `args.get("file_filter").and_then(|v| v.as_str())`

**Checkpoint**: User Story 2 complete — `file_filter` correctly narrows results. Tests pass.

---

## Phase 5: User Story 3 - Consistent Result Formatting (Priority: P3)

**Goal**: All returned file paths are relative to KB_ROOT, immediately reusable as input to other MCP tools.

**Independent Test**: Run `loom_grep` and verify file paths are relative (not absolute) and match the format accepted by `read_section`, `read_lines`, etc.

### Tests for User Story 3

- [x] T023 [P] [US3] Test: grep returns relative file paths in `tests/search_tests.rs` — vault at `/tmp/vault/` with file `/tmp/vault/sub/note.md`, call `grep`, assert file path is `sub/note.md` not `/tmp/vault/sub/note.md`
- [x] T024 [P] [US3] Test: grep paths match list_files output format in `tests/search_tests.rs` — call both `list_files()` and `grep()`, verify path formats are identical (both strip KB_ROOT prefix)

### Implementation for User Story 3

- [x] T025 [US3] Verify and normalize relative path stripping in `EditManager::grep()` in `src/edits.rs` — ensure `strip_prefix(&self.kb_root)` is applied consistently, handle edge case where `unwrap_or` falls back to full path (should never happen in practice since vault files are always under KB_ROOT)

**Checkpoint**: User Story 3 complete — all paths relative and consistent. Tests pass.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, quality gates, and integration tests

- [x] T026 [P] Add doc comments (`///`) to `GrepMatch` and `GrepResponse` structs and the updated `grep()` function in `src/edits.rs`
- [x] T027 [P] Update grep description in `src/server.rs` tool definition to reflect new parameters
- [x] T028 Add MCP dispatch test for grep with limit and file_filter in `tests/server_tests.rs`
- [x] T029 Update `CHANGELOG.md` with grep improvements
- [x] T030 [P] Run `cargo fmt --all -- --check` to verify formatting
- [x] T031 [P] Run `cargo clippy -- -D warnings` to verify linting
- [x] T032 Run `cargo test --all-features` to verify all tests pass
- [x] T033 Verify existing tests (`test_grep_regex_anchored_start`, `test_grep_regex_digit_sequence`, `test_grep_invalid_regex_returns_empty`, `test_dispatch_tool_grep_with_pattern`) still pass with the updated function signature

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS all user stories
- **User Stories (Phase 3-5)**: All depend on Foundational phase completion
  - US1 → US2 → US3 in priority order (each builds on previous grep signature changes)
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: After Foundational — adds `limit` parameter to grep signature. No dependency on US2 or US3.
- **User Story 2 (P2)**: After US1 — adds `file_filter` parameter to already-modified grep signature.
- **User Story 3 (P3)**: After US2 — verifies and normalizes path formatting in the now-fully-modified grep function.

> **Note**: US2 and US3 are sequential because each adds to the same function signature (`grep`). If desired, they could be combined into a single phase due to their tight coupling to the same code.

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD)
- Implementation follows: signature change → loop logic → server handler → MCP schema

### Parallel Opportunities

- T004 and T005 (struct definitions) can run in parallel
- All test tasks within a phase marked [P] can run in parallel
- Polish tasks T026, T027, T029, T030, T031, T033 can all run in parallel

---

## Parallel Example: User Story 1 Tests

```bash
# Write all US1 tests in parallel:
Task: "Test default limit caps at 200 in tests/search_tests.rs" (T008)
Task: "Test explicit limit=20 returns exactly 20 in tests/search_tests.rs" (T009)
Task: "Test limit=0 returns all matches in tests/search_tests.rs" (T010)
Task: "Test no matches returns empty with metadata in tests/search_tests.rs" (T011)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (verify toolchain, tests pass)
2. Complete Phase 2: Foundational (GrepResponse struct)
3. Complete Phase 3: User Story 1 (limit + truncation)
4. **STOP and VALIDATE**: Test US1 independently with `cargo test`
5. Run quality gates: `cargo fmt`, `cargo clippy`, `cargo test`
6. Grep no longer returns whole files — MVP ready!

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add US1 (limit) → Test → Quality gates → Grep is usable (MVP!)
3. Add US2 (file_filter) → Test → Quality gates → Grep is scoped
4. Add US3 (relative paths) → Test → Quality gates → Grep is consistent
5. Polish → Final quality gates → Ready for merge

### Quality Gates (Must Pass Before Merge)

- **Formatting**: `cargo fmt --all -- --check` must pass
- **Linting**: `cargo clippy -- -D warnings` must pass
- **Testing**: `cargo test --all-features` must pass
- **Coverage**: Minimum 80% line coverage
- **Security**: `cargo deny check licenses bans sources` must pass
- **Backward Compat**: Existing grep tests must continue to pass

---

## Notes

- [P] tasks = different files or independent test cases, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- **TDD enforced**: Tests written first, verified failing, then implementation
- **EXPLICIT CONSENT REQUIRED**: Each git commit requires individual user consent per constitution
- US3 (relative paths) is already partially implemented in the current grep code — verify and normalize
- Existing grep tests at `tests/search_tests.rs:439-472` and `tests/server_tests.rs:181-191` must continue to pass
- Use `tempfile` for file system tests (existing pattern in test files)
