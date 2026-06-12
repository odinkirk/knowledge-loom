# Tasks: Fix CI Test Vault Dependency

**Input**: Design documents from `/specs/010-fix-ci-test-vault/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md

**Tests**: Not required — the existing test is correct; only the CI environment needs to provide the corpus.

**Organization**: Single user story, single file change.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Gather prerequisites for the fix

- [x] T001 Find the current HEAD commit hash of the test-vault corpus: `git ls-remote https://github.com/ashuotaku/Personal-Wiki.git HEAD`
- [x] T002 [P] Verify the test passes locally with the corpus present: `git clone --depth 1 https://github.com/ashuotaku/Personal-Wiki.git test-vault && cargo test test_graph_edges_from_test_vault -- --nocapture && rm -rf test-vault`

---

## Phase 2: User Story 1 - CI Tests Workflow Passes (Priority: P1) 🎯 MVP

**Goal**: The Tests workflow completes with zero failures by making `test-vault/` available on the CI runner before `cargo test` runs.

**Independent Test**: Push to the branch and observe the Tests workflow — `cargo test --all-features` completes with exit code 0, `test_graph_edges_from_test_vault` passes.

### Implementation for User Story 1

- [x] T003 [US1] Add `actions/checkout@v4` step for `ashuotaku/Personal-Wiki` to `.github/workflows/test.yml` before the `Run tests` step, with `path: test-vault`, `ref: <commit-hash>`, and `fetch-depth: 1`
- [x] T004 [US1] Verify US1: push to remote and observe Tests workflow — `test_graph_edges_from_test_vault` passes, zero test failures overall

---

## Phase 3: Polish

**Purpose**: Final documentation update

- [x] T005 Update `CHANGELOG.md` under `## [Unreleased]` → `### Fixed` with note that the CI Tests workflow now clones `test-vault/` corpus before running tests, resolving the `test_graph_edges_from_test_vault` failure

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **User Story 1 (Phase 2)**: Depends on Setup (needs commit hash from T001, verified test from T002)
- **Polish (Phase 3)**: Depends on US1 completion and verification

### Within User Story 1

- T003 (clone step) before T004 (CI verification)

### Parallel Opportunities

- T001 and T002 can run in parallel

---

## Implementation Strategy

### MVP (User Story 1)

1. Complete Phase 1: Get commit hash and verify test locally
2. Complete T003: Add checkout step to test.yml
3. Push and verify T004: CI Tests workflow passes
4. Update CHANGELOG

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- No Rust source code changes — `cargo test/fmt/clippy` should remain green
- **EXPLICIT CONSENT REQUIRED**: Each git commit requires individual user consent per constitution
- The commit hash should be pinned for deterministic test behavior
