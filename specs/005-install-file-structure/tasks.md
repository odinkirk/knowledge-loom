# Tasks: install-file-structure

**Input**: Design documents from `/specs/005-install-file-structure/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Test tasks are included per spec TEST-001 through TEST-005 requirement.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Knowledge Loom**: `src/`, `tests/` at repository root
- **New module**: `src/install.rs` (runtime data setup)
- **Updated module**: `src/main.rs` (CLI subcommand), `src/model.rs` (reuse model logic)
- **Tests**: `tests/` with `*_tests.rs` naming
- **Storage**: `.knowledge-loom/models/` (runtime data), `.knowledge-loom-index/` (index, unchanged)

## Phase 1: Setup

**Purpose**: Project initialization and basic structure

- [ ] T001 [P] Verify feature branch 005-install-file-structure is active
- [ ] T002 [P] Review constitution requirements for implementation
- [ ] T003 [P] Verify cargo build --release succeeds before any changes
- [ ] T004 [P] Verify cargo test succeeds before any changes
- [ ] T005 [P] Review existing model download code in src/model.rs and src/download.rs (feature 004)

## Phase 2: Foundational (Blocking prerequisites for all user stories)

**Purpose**: Tasks that block all user stories

- [x] T006 [P] Add download state constants (MODEL_DIR, STATE_FILE, MODEL_URL) to src/install.rs
- [x] T007 [P] Define InstallError enum in src/install.rs with variants: DownloadFailed, ChecksumMismatch, AlreadyInstalled, NetworkError, DiskFull
- [x] T008 [P] Create run_install() function signature in src/main.rs as entry point for `loom install`
- [x] T009 [P] Write InstallManager struct skeleton in src/install.rs with new(), kb_root() methods

## Phase 3: User Story 1 - Install Runtime Data (Priority: P1)

**Story Goal**: A developer runs `loom install` to download fastembed models into `.knowledge-loom/models/` with cache and config. MCP configs stay at root, index stays in `.knowledge-loom-index/`.

**Independent Test Criteria**: After running `loom install`:
- `.knowledge-loom/models/` contains fastembed model files
- Root-level MCP configs are untouched
- `.knowledge-loom-index/` is unchanged
- `cargo test --release` passes

- [x] T010 [P] [US1] Write unit test for InstallManager::new() in tests/install_tests.rs
- [x] T011 [P] [US1] Write unit test for InstallManager::model_path() returning `.knowledge-loom/models/` in tests/install_tests.rs
- [x] T012 [P] [US1] Write unit test for InstallManager::is_installed() returning false initially in tests/install_tests.rs
- [x] T013 [US1] Implement InstallManager::new() in src/install.rs
- [x] T014 [US1] Implement InstallManager::model_path() returning `.knowledge-loom/models/` in src/install.rs
- [x] T015 [US1] Implement InstallManager::is_installed() checking `.knowledge-loom/models/` in src/install.rs
- [x] T016 [P] [US1] Write unit test for InstallManager::download_model() downloading mock file in tests/install_tests.rs
- [x] T017 [P] [US1] Write unit test for checksum validation after download in tests/install_tests.rs
- [x] T018 [US1] Implement InstallManager::download_model() using reqwest to download fastembed in src/install.rs
- [x] T019 [US1] Implement checksum verification after download in src/install.rs
- [x] T020 [US1] Add `loom install` subcommand to src/main.rs CLI argument parsing
- [x] T021 [US1] Wire InstallManager::download_model() to `loom install` handler in src/main.rs
- [x] T022 [US1] Verify MCP config files (opencode.json, .mcp.json) are untouched after install
- [x] T023 [US1] Verify `.knowledge-loom-index/` is unchanged after install
- [x] T024 [US1] Run cargo test --release and fix any failures

## Phase 4: User Story 2 - Verify Runtime Data Integrity (Priority: P2)

**Story Goal**: CI pipeline runs `loom install && cargo test --release`. Pipeline fails if model files are corrupted or missing. After install, test suite passes.

**Independent Test Criteria**: CI runs `loom install && cargo test --release`. Job passes only when both succeed.

- [x] T025 [P] [US2] Write unit test for integrity check: valid checksum passes in tests/install_tests.rs
- [x] T026 [P] [US2] Write unit test for integrity check: corrupted file triggers re-download in tests/install_tests.rs
- [x] T027 [P] [US2] Write unit test for integrity check: missing file triggers download in tests/install_tests.rs
- [x] T028 [US2] Implement InstallManager::verify_integrity() checking checksum in src/install.rs
- [x] T029 [US2] Implement InstallManager::validate_or_download() - verify then download if needed in src/install.rs
- [x] T030 [US2] Add integrity verification step to `loom install` flow in src/main.rs
- [x] T031 [US2] Write integration test: install + cargo test --release passes in tests/install_integration.rs

## Phase 5: User Story 3 - Re-install or Update (Priority: P3)

**Story Goal**: After model version update, developer runs `loom install --force` to re-download. Without --force, skip if already valid.

**Independent Test Criteria**: Run `loom install --force`, verify updated model files appear, all tests pass.

- [x] T032 [P] [US3] Write unit test for --force flag triggering re-download in tests/install_tests.rs
- [x] T033 [P] [US3] Write unit test for skip-download when model already valid in tests/install_tests.rs
- [x] T034 [P] [US3] Write unit test for error message when --force not provided and model exists in tests/install_tests.rs
- [x] T035 [US3] Implement --force flag handling in InstallManager in src/install.rs
- [x] T036 [US3] Add --force argument to `loom install` CLI parser in src/main.rs
- [x] T037 [US3] Implement skip logic: if model valid and no --force, report "already installed" and exit 0 in src/install.rs
- [x] T038 [US3] Implement re-download logic: if --force, re-download and overwrite model in src/install.rs
- [x] T039 [US3] Write integration test: force re-download + verify test suite in tests/install_integration.rs

## Final Phase: Polish & Cross-Cutting Concerns

**Purpose**: Performance, documentation, and final quality checks

- [ ] T040 [P] Add output summary to install: model version, download size, target location in src/install.rs
- [ ] T041 [P] Handle network errors with user-friendly message recommending --force in src/install.rs
- [ ] T042 [P] Handle disk full errors with clear message and partial cleanup in src/install.rs
- [ ] T043 [P] Write manual download instructions error message in src/install.rs
- [ ] T044 [P] Update CHANGELOG.md with new `loom install` feature
- [ ] T045 [P] Update ARCHITECTURE.md if runtime data layout changes significantly
- [ ] T046 [P] Run cargo fmt --all -- --check and fix
- [ ] T047 [P] Run cargo clippy -- -D warnings and fix
- [ ] T048 [P] Run cargo test --release and ensure all pass
- [ ] T049 [P] Run cargo deny check and fix any issues
- [ ] T050 [P] Verify 80% code coverage for install module
- [ ] T051 [P] Write performance benchmark test: verify `loom install` completes in <30s (100Mbps connection) in tests/install_benchmark.rs

## Dependencies

```
Phase 1 (Setup) → Phase 2 (Foundational) → Phase 3 (US1) → Phase 4 (US2) → Phase 5 (US3) → Final Phase
                                                  ↕                   ↕
                                          Independent           Independent
                                          (no deps on US2)      (no deps on US3)
```

US1, US2, and US3 are designed to be independently testable. US2 depends on the `verify_integrity()` method built in US1, and US3 depends on the `--force` flag added alongside US1.

## Parallel Execution Examples

**PHASE 3 (US1) parallel tasks**: T010, T011, T012 are independent test scaffolds that can be written simultaneously. T013-T015 are the corresponding implementations.

**PHASE 4 (US2) parallel tasks**: T025, T026, T027 are independent test cases. T028-T030 are sequential implementations.

**PHASE 5 (US3) parallel tasks**: T032, T033, T034 are independent test cases. T035-T038 are sequential.

**FINAL PHASE parallel tasks**: T040-T045 are independent improvements. T046-T050 are sequential quality gates.

## Implementation Strategy

**MVP Scope**: Phase 1 + Phase 2 + Phase 3 (US1) delivers the core `loom install` functionality with model download to `.knowledge-loom/models/`. This is independently testable and delivers user value.

**Incremental Delivery**:
1. US1 (P1) - Core install: model download, path setup, CLI wiring
2. US2 (P2) - Integrity: checksum verification, auto-repair on corruption
3. US3 (P3) - Reinstall: --force flag, skip-if-valid optimization
4. Final - Polish: error messages, docs, quality gates
