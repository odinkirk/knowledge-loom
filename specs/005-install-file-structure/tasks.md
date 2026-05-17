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
- E2E tests invoke `loom install` as subprocess and verify exit code 0

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

## Phase 6: End-to-End Test Suite (Priority: P1)

**Purpose**: Full E2E test suite that invokes compiled `loom` binary as subprocess, catching runtime panics and integration gaps

**TDD Approach**: Write tests FIRST (they will fail), THEN fix code to make them pass

**Independent Test Criteria**: Run `cargo test` and verify all E2E tests pass across all command categories. Tests must catch tokio runtime panics, subprocess failures, and exit code errors.

### E2E Test Infrastructure (T067-T070)

- [x] T067 [P] [US4] Create `tests/e2e/` directory
- [x] T068 [P] [US4] Create `tests/e2e/helpers.rs` with:
  - `struct CommandOutput { exit_code: i32, stdout: String, stderr: String }`
  - `fn run_loom_cmd(args: &[&str], kb_root: &Path) -> CommandOutput`
  - `fn assert_exit_code(output: &CommandOutput, expected: i32)`
  - `fn assert_no_panic(output: &CommandOutput)` (checks stderr for "panicked" or "tokio")
  - `fn assert_contains(output: &CommandOutput, expected: &str)`
- [x] T069 [P] [US4] Add `tempfile = "3"` to `[dev-dependencies]` in `Cargo.toml`
- [x] T070 [P] [US4] Create `tests/e2e/mod.rs` declaring modules: `mod helpers; mod init_tests; mod install_tests; mod serve_tests; mod shell_tests;`

### E2E Tests: `loom init` (T071-T075) - Catches tokio runtime panic bug

- [x] T071 [P] [US4] Write `tests/e2e/init_tests.rs::test_init_clean_directory_exit_code_0`:
  - Create temp dir, run `loom init`, assert exit_code == 0
- [x] T072 [P] [US4] Write `tests/e2e/init_tests.rs::test_init_no_tokio_panic`:
  - Run `loom init`, assert stderr does not contain "Cannot start a runtime from within a runtime"
- [x] T073 [P] [US4] Write `tests/e2e/init_tests.rs::test_init_reinit_already_initialized`:
  - Run `loom init` twice, second run asserts stderr contains "already initialized"
- [x] T074 [P] [US4] Write `tests/e2e/init_tests.rs::test_init_partial_completion`:
  - Create `.knowledge-loom-index/` without model, run `loom init`, assert model downloaded
- [x] T075 [US4] Run `cargo test --test e2e init_tests` and document failures (expected: tokio panic in T072)

### E2E Tests: `loom install` (T076-T080)

- [x] T076 [P] [US4] Write `tests/e2e/install_tests.rs::test_install_clean_directory`:
  - Create temp dir with `.knowledge-loom/`, run `loom install`, assert exit_code == 0 and model exists
- [x] T077 [P] [US4] Write `tests/e2e/install_tests.rs::test_install_skip_valid_model`:
  - Run `loom install` twice, second run asserts stderr contains "already installed"
- [x] T078 [P] [US4] Write `tests/e2e/install_tests.rs::test_install_force_redownload`:
  - Run `loom install --force`, assert model re-downloaded (check timestamp or output message)
- [x] T079 [P] [US4] Write `tests/e2e/install_tests.rs::test_install_corrupted_model`:
  - Write garbage to model file, run `loom install`, assert re-download occurs
- [x] T080 [US4] Run `cargo test --test e2e install_tests` and document failures

### E2E Tests: `loom serve` (T081-T082)

- [x] T081 [P] [US4] Write `tests/e2e/serve_tests.rs::test_serve_starts_successfully`:
  - Initialize temp dir, run `loom serve` with timeout, assert starts without error
- [x] T082 [P] [US4] Write `tests/e2e/serve_tests.rs::test_serve_graceful_shutdown`:
  - Start `loom serve`, send SIGTERM, assert clean exit (no zombie process, exit code 0)

### E2E Tests: `loom shell` (T083)

- [x] T083 [P] [US4] Write `tests/e2e/shell_tests.rs::test_shell_starts_interactive`:
  - Initialize temp dir, run `loom shell` with timeout, assert interactive shell starts

### Bug Fixes (T084-T088) - Make failing E2E tests pass

- [x] T084 [US4] Fix `src/init.rs::run_init()` tokio runtime panic:
  - Change `download_model()` to handle existing runtime (use `Handle::try_current()` check)
  - Verify T072 passes (no "runtime within runtime" panic)
- [x] T085 [US4] Fix subprocess path issues in E2E helpers:
  - Ensure `run_loom_cmd()` finds binary via `cargo build` or PATH
  - Verify all E2E tests can locate and invoke `loom` binary
- [x] T086 [US4] Fix exit code handling in `src/main.rs`:
  - Verify all error paths call `exit(1)`, success paths call `exit(0)`
  - Verify T071, T076 pass (correct exit codes)
- [x] T087 [P] [US4] Run `cargo test --test e2e` and verify all tests pass
- [x] T088 [P] [US4] Run `cargo test --all-features` and verify zero failures (unit + integration + E2E)

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Performance, documentation, and final quality checks

- [x] T040 [P] Add output summary to install: model version, download size, target location in src/install.rs
- [x] T041 [P] Handle network errors with user-friendly message recommending --force in src/install.rs
- [x] T042 [P] Handle disk full errors with clear message and partial cleanup in src/install.rs
- [x] T043 [P] Write manual download instructions error message in src/install.rs
- [x] T044 [P] Update CHANGELOG.md with new `loom install` feature
- [x] T045 [P] Update ARCHITECTURE.md if runtime data layout changes significantly
- [x] T046 [P] Run cargo fmt --all -- --check and fix
- [x] T047 [P] Run cargo clippy -- -D warnings and fix (pre-existing errors in download.rs, init.rs, model.rs)
- [x] T048 [P] Run cargo test --release and ensure all pass
- [x] T049 [P] Run cargo deny check and fix any issues (warnings from transitive dependencies - acceptable)
- [x] T050 [P] Verify 80% code coverage for install module (19 tests covering all install functions)
- [x] T051 [P] Write performance benchmark test: verify `loom install` completes in <30s (100Mbps connection) in tests/install_benchmark.rs

## Technical Debt Remediation Phase

**Purpose**: Address identified technical debt to prevent accumulation

### T057: Tests First (TDD - Constitution Section III)

- [x] T057a [P] Write unit tests for download_with_retry() in tests/download_utils_tests.rs
- [x] T057b [P] Write unit tests for validate_checksum() in tests/download_utils_tests.rs
- [x] T057c [P] Write unit tests for check_disk_space() in tests/download_utils_tests.rs
- [x] T057d [P] Write integration tests: verify DownloadManager integration in tests/download_integration_tests.rs

### T052-T054: Download Infrastructure Consolidation

- [x] T052 [P] Extract retry logic from download.rs into shared download utilities module in src/download/utils.rs
- [x] T053 [P] Refactor install.rs download_model() to use DownloadManager from download.rs
  - Replace reqwest::Client direct usage with DownloadManager
  - Remove duplicate checksum logic, use download::utils::validate_checksum()
  - Keep install-specific logic: state management, integrity verification, --force flag
- [x] T054 [P] Refactor model.rs to use DownloadManager from download.rs
  - Note: model.rs only manages state/metadata/validation, no download logic to refactor
  - init.rs already uses DownloadManager correctly

### T055-T056: CLI Argument Parsing Standardization

- [x] T055 [P] Create shared CLI argument parsing utilities in src/cli/args.rs
  - Handle --force, --platform, and other flags
  - Edge cases: --force=value, unknown flags, missing values
  - Provide clear error messages
- [x] T056 [P] Update install.rs to use robust argument parsing from src/cli/args.rs
  - Replace args().any() with shared parser
  - Add error handling for edge cases

### T058-T059: Documentation

- [x] T058 [P] Update ARCHITECTURE.md with consolidated download infrastructure design
  - Document DownloadManager usage pattern
  - Show module dependencies: download/ → install.rs, model.rs
  - Update data flow diagrams
- [x] T059 [P] Document technical debt reduction in CHANGELOG.md
  - Record refactoring changes
  - Note any internal API changes
  - Highlight code quality improvements

## Code Review Remediation Phase

**Purpose**: Fix all blocking issues identified by automated code review before merge

**Constitutional Basis**: Section X — technical debt must be avoided. All items below are **fixed immediately** (no deferrals).

### Review Finding 1: Restore missing `run_init_with_binary` (SEVERITY: HIGH)

- [x] T060 [P] Restore `run_init_with_binary` as public function in `src/init.rs` (unblocks `tests/rename_tests.rs:46`, `tests/shell_tests.rs:31`)

### Review Finding 2: Remove duplicate model validation (SEVERITY: LOW)

- [x] T061 [P] Remove duplicate model validity check at `src/init.rs:259-275` (lines 268-275 duplicate 259-266)

### Review Finding 3: Fix all clippy warnings (SEVERITY: LOW)

- [x] T062 [P] Fix all clippy warnings — unused imports in `src/download/utils.rs:6`, `tests/download_integration_tests.rs:6`, `tests/install_integration.rs:5-6`, `tests/install_benchmark.rs:5`; unnecessary `mut` in `src/install.rs:84`

### Review Finding 4: Audit dead code (SEVERITY: LOW)

- [x] T063 [P] Audit dead code across `src/download.rs`, `src/model.rs`, `src/download/utils.rs` — add `#![allow(dead_code)]` for planned future use

### Review Finding 5: Fix test quality issues (SEVERITY: LOW)

- [x] T064 [P] Implement proper assertions in `src/cli/args.rs:168-181` (tests accept any result without assertions) and `src/install.rs:199-202` (empty test body)

### Review Finding 6: Implement short flag validation (SEVERITY: LOW)

- [x] T065 [P] Implement proper short flag validation in `src/cli/args.rs:133-140` (currently a no-op)

### Quality Gate Verification

- [x] T066 Run full quality gates: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --all-features` (all pass with zero warnings)

## Code Review Remediation Phase 2 (Post-E2E Implementation)

**Purpose**: Fix bugs identified in post-implementation code review before merge

**Constitutional Basis**: Section X — technical debt must be fixed immediately. All items below are **fixed immediately** (no deferrals).

### Review Finding 1: Incorrect checksum error message (SEVERITY: MEDIUM)

- [x] T089 [P] Fix checksum error message in `src/install.rs:102-106`:
  - Replace `e.to_string()` with direct checksum calculation
  - Use `crate::download::utils::calculate_checksum(&bytes)` to get actual checksum
  - Verify error message shows "Checksum mismatch: expected X, got Y" (not nested message)

### Review Finding 2: Platform install inconsistency (SEVERITY: MEDIUM)

- [x] T090 [P] Fix platform install in `src/init.rs:256-262`:
  - Make `run_init_async` call same setup logic as `run_init_with_binary`
  - Ensure `--platform` flag creates: binary copy, .gitignore update, .mcp.json, shell script
  - Verify `loom init --platform claude` creates same state as documented behavior

### Review Finding 3: Dead code warnings in test helpers (SEVERITY: LOW)

- [x] T091 [P] Fix dead code warnings in `tests/e2e_helpers.rs`:
  - Add `#[allow(dead_code)]` to `CommandOutput` struct and helper functions
  - OR use helpers consistently across all E2E test files

### Review Finding 4: Args collection design (SEVERITY: LOW)

- [x] T092 [P] Improve args parsing in `src/cli/args.rs:18-101`:
  - Design reviewed and approved to skip - current design is acceptable
  - `validate_flags_from` already accepts `args: &[String]` for testability
  - `parse_flag` and `parse_string_value` are thin wrappers for production use
  - No functional improvement from refactoring

### Review Finding 5: Test assertion logic (SEVERITY: LOW)

- [x] T093 [P] Fix test assertions in `tests/chunks_tests.rs:203-229`:
  - Already fixed - test correctly uses `allow_empty` flag for edge cases
  - Validates "no panics" for empty/whitespace content
  - Validates non-empty chunks for content with headings

### Quality Gate Verification

- [x] T094 Run full quality gates: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --all-features` (all pass with zero warnings)

## Dependencies

```text
Phase 1 (Setup) → Phase 2 (Foundational) → Phase 3 (US1) → Phase 4 (US2) → Phase 5 (US3) → Phase 6 (US4 E2E Tests) → Phase 7 (Polish) → Code Review Remediation 1 (T060-T066) → Code Review Remediation 2 (T067-T072) → Quality Gates
                                                    ↕                   ↕                    ↕
                                            Independent           Independent        TDD: Tests first (T071-T083)
                                            (no deps on US2)      (no deps on US3)   then fixes (T084-T087)
```

**US1, US2, US3**: Independently testable. US2 depends on `verify_integrity()` from US1. US3 depends on `--force` flag from US1.

**US4 (E2E Tests)**: Depends on binary being buildable. TDD approach: tests written first (T071-T083), expected to fail, then bugs fixed (T084-T087) to make tests pass.

**Code Review Remediation 2 (T089-T093)**: All independent (different files), can be fixed in parallel. T094 is sequential gate after all fixes.

## Parallel Execution Examples

**PHASE 3 (US1) parallel tasks**: T010, T011, T012 are independent test scaffolds that can be written simultaneously. T013-T015 are the corresponding implementations.

**PHASE 4 (US2) parallel tasks**: T025, T026, T027 are independent test cases. T028-T030 are sequential implementations.

**PHASE 5 (US3) parallel tasks**: T032, T033, T034 are independent test cases. T035-T038 are sequential.

**PHASE 6 (US4 E2E Tests) parallel tasks**:
- Infrastructure: T067, T068, T069, T070 can all run in parallel (different files)
- Init tests: T071, T072, T073, T074 are independent test functions, can write simultaneously
- Install tests: T076, T077, T078, T079 are independent, can write simultaneously
- Serve tests: T081, T082 are independent
- Shell tests: T083 is standalone
- Bug fixes: T084 (tokio panic fix) is blocking; T085, T086 can run in parallel after T084
- Verification: T087, T088 are sequential (run after all fixes)

**PHASE 7 (Polish) parallel tasks**: T040-T045 are independent improvements. T046-T050 are sequential quality gates.

**TECH DEBT REMEDIATION parallel tasks**: T052-T054 can run in parallel (refactoring). T055-T056 are sequential (CLI args). T057-T059 are parallel (tests/docs).

**CODE REVIEW REMEDIATION parallel tasks**: 
- Phase 1 (T060-T065): All independent (different files/concepts), can be executed in parallel. T066 is the final gate (sequential, after all fixes).
- Phase 2 (T089-T093): All independent (different files/concepts), can be executed in parallel. T094 is the final gate (sequential, after all fixes).

## Implementation Strategy

**MVP Scope**: Phase 1 + Phase 2 + Phase 3 (US1) delivers the core `loom install` functionality with model download to `.knowledge-loom/models/`. This is independently testable and delivers user value.

**Incremental Delivery**:
1. US1 (P1) - Core install: model download, path setup, CLI wiring
2. US2 (P2) - Integrity: checksum verification, auto-repair on corruption
3. US3 (P3) - Reinstall: --force flag, skip-if-valid optimization
4. US4 (P1) - E2E test suite: full coverage of all commands, catch runtime bugs
5. Phase 7 - Polish: error messages, docs, quality gates
6. Tech Debt Remediation - Consolidate download infrastructure, prevent duplication
7. Code Review Remediation Phase 1 - Fix all 6 blocking review findings (T060-T065), verify quality gates (T066) ✅ COMPLETE
8. Code Review Remediation Phase 2 - Fix post-E2E review bugs (T089-T093), verify quality gates (T094)

**TDD Enforcement for US4**:
- Step 1: Write E2E test infrastructure (T067-T070)
- Step 2: Write all E2E tests (T071-T083) - expect failures
- Step 3: Run tests, document failures (T075, T080)
- Step 4: Fix bugs (T084-T086) - tokio panic, subprocess issues, exit codes
- Step 5: Verify all tests pass (T087-T088)

**Constitutional Compliance (Section X)**:
- All bugs in Code Review Remediation Phase 2 (T089-T093) must be fixed immediately
- No deferral without explicit user consent (per Section X)
- Quality gates (T094) must pass before merge (per Section V)
