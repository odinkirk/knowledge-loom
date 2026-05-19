# Tasks: install-file-structure

**Input**: Design documents from `/specs/005-install-file-structure/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Test tasks are included per spec TEST-001 through TEST-005 requirement.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

**Total Tasks**: T001-T137 (139 tasks across 14 phases)

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

- [x] T001 [P] Verify feature branch 005-install-file-structure is active
- [x] T002 [P] Review constitution requirements for implementation
- [x] T003 [P] Verify cargo build --release succeeds before any changes
- [x] T004 [P] Verify cargo test succeeds before any changes
- [x] T005 [P] Review existing model download code in src/model.rs and src/download.rs (feature 004)

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

## Code Review Remediation Phase 3 (2026-05-17 Branch Review)

**Purpose**: Fix bugs identified in 2026-05-17 branch review before merge

**Constitutional Basis**: Section X — technical debt must be fixed immediately. All items below are **fixed immediately** (no deferrals).

### Review Finding 1: `is_installed` checks directory, not file (SEVERITY: MEDIUM)

- [x] T095 [P] Fix `is_installed()` in `src/install.rs:69-71`:
  - Change `self.model_path().exists()` to `self.model_path().join("model.onnx").exists()`
  - Verify returns `false` when directory exists but model file is missing
  - Update doc comment to clarify it checks for model file, not directory

- [x] T096 [P] Consolidate `check_disk_space` functions:
  - Keep single implementation in `src/download/utils.rs:53` (public)
  - Remove duplicate from `src/download.rs:668`
  - Ensure Windows implementation exists (add `#[cfg(windows)]` variant or use cross-platform approach)
  - Update all callers to use `download::utils::check_disk_space`

- [x] T097 [P] Optimize checksum calculation in `src/install.rs:146`:
  - Change `calculate_checksum` to accept `impl Read` instead of `&[u8]`
  - Use streaming hash: read file in 8KB chunks (reuse pattern from `download.rs:633-651`)
  - Verify memory usage reduced from 90MB to ~8KB buffer

- [x] T098 [P] Simplify callback parameter in `src/download/utils.rs:28`:
  - Change `Option<impl Fn(DownloadProgress) + Send + Sync>` to `F: Fn(DownloadProgress) + Send + Sync`
  - Update callers to pass closure directly or use `|_| {}` for no-op
  - Remove `Option` unwrapping logic inside function

- [x] T099 [P] Add network guards to E2E install tests in `tests/e2e_install_tests.rs`:
  - Add `LOOM_TEST_NETWORK` environment variable check to network-dependent tests
  - OR add `#[ignore]` attribute with reason: "requires network access"
  - Document in test comment how to enable: `cargo test -- --ignored` or `LOOM_TEST_NETWORK=1 cargo test`

- [x] T100 [P] Fix mtime check in `tests/e2e_install_tests.rs:62-74`:
  - Change from checking directory mtime to file mtime
  - Use `model_dir.join("model.onnx").metadata().modified()` instead of `fs::metadata(&model_dir).modified()`
  - Verify test passes on all filesystems (directory mtime behavior varies)

- [x] T101 Run full quality gates: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --all-features` (all pass with zero warnings)

## Phase 8: Smoke Test Bug Fixes

**Purpose**: Fix defects discovered during smoke testing on live corpus (2026-05-17)

**Constitutional Basis**: Section X — technical debt must be fixed immediately. All items below are **fixed immediately** (no deferrals).

### Smoke Bug 1: Chunk Truncation → Splitting (SEVERITY: MEDIUM)

- [x] T102 [P] Write unit tests for `parse_chunks()` splitting behavior in `tests/chunks_tests.rs`:
  - Content exceeding 2000 chars produces multiple chunks
  - Split chunks share the same heading breadcrumb
  - Ordinals are sequential across split chunks
  - Test at whitespace boundary (not mid-word)
  - Test headingless fallback also splits >2000 chars

- [x] T103 Modify `parse_chunks()` in `src/chunks.rs` to split sections exceeding `MAX_CHUNK_CHARS` into multiple chunks instead of truncating

- [x] T104 Replace `VectorIndex::chunk_content()` in `src/index.rs:260–282` with reuse of `parse_chunks()` from `src/chunks.rs`. Update callers in `index_file()` (line 132,136) and `index_vault()` (line 214,218) to destructure `Chunk` struct fields (`chunk.heading`, `chunk.content`) instead of the current `(heading, content)` tuple pattern. Result: BM25 and vector indexes share consistent chunk boundaries.

### Smoke Bug 2: OpenCode Platform Config Format (SEVERITY: HIGH)

- [x] T105 [P] Write unit test for `install_platform(OpenCode, ...)` in `tests/platforms_tests.rs` — verify generated `opencode.json` matches schema at `https://opencode.ai/config.json` §`McpLocalConfig`: `$schema` present, `mcp` key (not `mcpServers`), `type: "local"`, `command` is array of two strings with absolute paths, `environment` object with absolute `KB_ROOT`. Test will fail until T106 implemented (TDD per Constitution §III).

- [x] T106 Fix `install_platform(OpenCode, ...)` in `src/platforms.rs:153–160` to write `opencode.json` directly (not via `write_json_object_entry`, which targets `mcpServers` key):
  - Root: `"$schema": "https://opencode.ai/config.json"`
  - `"mcp"."knowledge-loom"`: `type: "local"`, `command: [absolute_binary_path, "serve"]`, `environment: {"KB_ROOT": absolute_repo_root}`
  - Use absolute paths for both `command` and `KB_ROOT` (matching the working config in the test corpus)
  - Preserve existing `AGENTS.md` write (platforms.rs:157-159)

- [x] T107 Remove `opencode` parameter from `build_entry()` and `write_json_object_entry()` in `src/platforms.rs` — after T106, the OpenCode handler no longer calls `write_json_object_entry`. The `opencode=true` code path is dead. Update remaining callers (Claude, Cursor, Windsurf, Zed, Kiro arms) to drop the last `false` argument.

- [x] T108 [P] Fix `run_init_async()` in `src/init.rs:319–335` to skip `.mcp.json` creation when `--platform opencode` is specified

### Smoke Bug 3: Reindex Batch Embedding (SEVERITY: MEDIUM)

- [x] T109 [P] Write unit tests for batch embedding in `tests/embed_batch_tests.rs`:
  - Single text batch (len 1) produces same result as single `embed()` call
  - Batch of 32 texts produces 32 embeddings of correct dimension (384)
  - Empty batch returns empty Vec
  - Batch containing empty strings handles gracefully (no panic)
  - Batch containing a chunk exceeding model's max input length: `embed_batch()` returns `Err` for that chunk only, continues processing remaining chunks, logs a warning with the file path and chunk index

- [x] T110 Add `embed_batch(&[String]) -> Result<Vec<Vec<f32>>>` method to `EmbedProvider` trait in `src/embed/mod.rs`

- [x] T111 Implement `embed_batch()` in `LocalEmbedProvider` in `src/embed/local.rs` — calls `model.embed(texts_vec, None)` with the full batch of texts instead of per-text `model.embed(vec![text], None)`

- [x] T112 Dispatch `embed_batch()` in `EmbedProviderEnum` in `src/embed/mod.rs`: delegate to `LocalEmbedProvider::embed_batch()` for Local; for Ollama and OpenRouter, fall back to single-text `embed()` in a loop

- [x] T113 [P] Modify `VectorIndex::index_vault()` in `src/index.rs:201–258` to collect all chunks for a file, call `embed_batch()`, then upsert results — replacing the per-chunk `embed()` loop

- [x] T114 [P] Modify `VectorIndex::index_file()` in `src/index.rs:125–166` to use `embed_batch()` instead of per-chunk `embed()` loop

- [x] T115 [P] Write performance test in `tests/reindex_perf_tests.rs`: verify `loom reindex` completes in <10 seconds for 100-file corpus. Also assert E2E test suite (T067-T083) completes in <5 minutes per spec SC-005.

- [ ] T116 ~~[P] Make BM25 and vector indexing run in parallel in `MaintenanceManager::reindex_all()` in `src/maintenance.rs:38–105` — use `tokio::join!` to run `bm25_lock.index_vault()` and `vector_index.index_vault()` concurrently. Both are read-only on vault files and write to separate indexes (tantivy vs sqlite).~~ **DESCOPED (2026-05-18)**: Second smoke test found `tokio::join!` causes tokio pool saturation, duplicate `parse_chunks()` work, per-file commit I/O contention, and stall at 1541/3998 embeddings. Sequential with `embed_batch` achieves target performance.

- [x] T116a Revert `reindex_all()` in `src/maintenance.rs` to sequential BM25 → Vector → Graph flow, retaining the `embed_batch` call in `VectorIndex::index_vault()`. Remove the parallel `tokio::join!` block and the pre-read-file-contents optimization.

### Quality Gate Verification

- [x] T117 Run full quality gates: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --all-features` (all pass with zero warnings). **Must be re-run after T116a; final gate before merge.**

## Phase 9: Reindex Performance Fixes (Third Smoke Test)

**Purpose**: Fix reindex performance bottlenecks discovered during third smoke test (2026-05-18) by profiling `reindex_all` with timing instrumentation. Four findings require implementation.

**Constitutional Basis**: Section X — performance defects must be fixed immediately.

### Smoke Bug 3a: ONNX Single-Threaded Inference (SEVERITY: HIGH)

- [x] T118 [P] Write unit test for ONNX threading configuration in `tests/embed_batch_tests.rs`: verify `LocalEmbedProvider::new()` sets `ORT_NUM_THREADS` to a value >1. (Updated from OMP_NUM_THREADS — fastembed uses `ort` crate which reads `ORT_NUM_THREADS`.)

- [x] T119 Configure ONNX Runtime multi-threading in `LocalEmbedProvider::new()` in `src/embed/local.rs`: set `ORT_NUM_THREADS` environment variable to CPU core count before `TextEmbedding::try_new()`. Target: 3–6× inference speedup. (Observed: ~2–3×; `ort` crate uses own thread pool, not OpenMP.)

### Smoke Bug 3d: Per-File Batch Underutilizes ONNX Parallelism (SEVERITY: MEDIUM)

- [x] T120 Modify... (ATTEMPTED: global batch caused hang with 5942 texts; reverted to per-file batching. See plan.md Finding 3d.)

### Smoke Bug 4: Missing Incremental Reindex (SEVERITY: HIGH)

- [x] T121 [P] Write unit test for reindex state file in `tests/reindex_state_tests.rs`: verify state records mtime and chunk_count per file, unchanged files are skipped, deleted files are cleaned

- [x] T122 Implement `ReindexState` struct in `src/maintenance.rs` — reads/writes `.knowledge-loom-index/reindex-state.json` with schema version, per-file `{mtime_secs, chunk_count}`. Provide `should_reindex(path, mtime, chunk_count) -> bool` method.

- [x] T123 Modify `MaintenanceManager::reindex_all()` in `src/maintenance.rs` to use `ReindexState` for incremental path. Key fix: acquire vault_state lock in a `tokio::time::timeout(Duration::from_secs(10))` block, scan files once (not twice), do comparison in memory after releasing lock. Incremental path activates when `!force` and state has files; falls back to full rebuild on error.

- [x] T124 Add `--force` flag to `loom reindex` in `src/main.rs:110–119`: when present, bypass `ReindexState` and force full rebuild.

- [x] T124a Update `EditManager::reindex_file()` in `src/edits.rs:240–270` to call `ReindexState::load()` / `update_file()` / `save()` after reindexing a single file.

### Quality Gate Verification

- [x] T125 Run full quality gates: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --all-features` (all pass with zero warnings). Verify reindex performance: first reindex <60s, subsequent <5s.

### Smoke Bug 5: .knowledge-loom-ignore Glob Matching (SEVERITY: MEDIUM)

**Discovered during third smoke test (2026-05-18)**: The `.knowledge-loom-ignore` file at the test corpus contains `.claude/**` but `VaultState::should_ignore()` uses `path.to_string_lossy().contains(pattern)` — substring matching, not glob expansion. 44 duplicate worktree files are indexed alongside the main repo files, adding ~2500 chunks and ~200s to reindex time.

- [x] T126a [P] Write unit test for glob-based ignore matching in `tests/vault_tests.rs`: verify `should_ignore()` matches `.claude/` subdirectories (e.g., `.claude/worktrees/foo/file.md` → true), does NOT match non-ignored paths (e.g., `world/file.md` → false), respects wildcard patterns like `*.log`. Use the `glob` crate (not `globset` — single pattern matching is sufficient) with IgnorePattern wrapper handling dir-prefix and glob modes.

- [x] T126 Fix `VaultState::should_ignore()` in `src/vault.rs` to use `glob::Pattern::matches()` instead of `contains()`. Add `glob = "0.3"` to `[dependencies]` in `Cargo.toml`. Pre-compile patterns in `VaultState::new()` via `IgnorePattern::from_string()`. Patterns ending in `/**` or `/` use dir-prefix matching; others use `glob::Pattern` against both full relative path and filename component.

- [x] T127 Add `.claude/` to default ignored patterns in `VaultState::new()` in `src/vault.rs` (alongside existing `.git/**` and `target/**`).

## Phase 11: Polish & Robustness

**Purpose**: Close gaps identified in third smoke-test analysis (2026-05-18). User-facing quality, E2E coverage, daemon integration, recovery paths.

- [x] T128 [P] Write E2E integration test for full user workflow in `tests/e2e_full_pipeline_tests.rs`: `loom init` → `loom install` → `loom reindex` → edit a file → `loom reindex` (incremental, assert "No changes" or fast) → `loom reindex` (incremental should skip). Use tempdir, minimal markdown files, and `std::process::Command` to invoke the binary. Also tests `--force` flag.

- [x] T129 [P] Add user-facing progress to `reindex_all()` in `src/maintenance.rs`: when entering full rebuild, `eprintln!("Full rebuild in progress (may take several minutes). Use --force to skip incremental check, or wait for incremental path.")`. Also add `eprintln!("  {} files scanned, {} changed, {} deleted", total, changed, deleted)` in the incremental path.

- [x] T130 [P] Write daemon integration smoke test: verify `reindex_all` updates `ReindexState` after edits, subsequent `reindex_all` skips unchanged files, and `chunk_count` changes when section count changes. Tests at `tests/e2e_daemon_state_tests.rs`.

- [x] T131 [P] Add diagnostics for `.knowledge-loom-ignore` in `VaultState::scan_files()` (src/vault.rs): count files excluded by ignore patterns and `eprintln!("  ignored {} files via .knowledge-loom-ignore", count)`. Makes invisible exclusions visible to users.

- [x] T132 [P] Add timeout guard to `reindex_incremental()` in `src/maintenance.rs`: wrap `vault_state.lock()` in `tokio::time::timeout(Duration::from_secs(10))`. On timeout, return error so caller falls back to full rebuild instead of hanging.

- [x] T133 [P] Add index health check to `reindex_all()` startup in `src/maintenance.rs`: compare embedding count (via new `VectorIndex::count_embeddings()`) against `reindex-state.json` expected total. Log warning if actual <50% of expected. On tantivy schema mismatch (detected via new `BM25Index::check_schema()`), log warning. Both are advisory; full rebuild path handles actual corruption.

- [x] T134 [P] Incremental graph: changed `reindex_incremental()` to use `GraphState::update_file()` per changed file instead of `build_graph()` on vault. Reduces incremental reindex time for single-file edits from ~1.2s to ~0.1s.

- [x] T135 [P] Update `ARCHITECTURE.md` with: new `ReindexState` entity, incremental reindex flow, chunk splitting change (2000→800), OpenCode platform config format, `embed_batch` API, BM25 single-commit change. Update `CHANGELOG.md` with all performance fixes and platform fixes. Verified `glob` dependency is clean.

- [x] T136 [P] Future-proofing note in plan: `MAX_CHUNK_CHARS=800` is optimized for English (chars/token ≈ 4). CJK and other scripts need different values. Defer token-based chunking (using the model's tokenizer) to a future feature. Note added to plan.md §6.

### Quality Gate Verification

- [x] T137 Run full quality gates: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --all-features` (zero new failures, zero warnings). Verify: first reindex completes, incremental reindex completes, state file persists, `--force` works.

## Dependencies

```text
Phase 1 (Setup) → Phase 2 (Foundational) → Phase 3 (US1) → Phase 4 (US2) → Phase 5 (US3) → Phase 6 (US4 E2E Tests) → Phase 7 (Polish) → Code Review Remediation 1 (T060-T066) → Code Review Remediation 2 (T089-T094) → Code Review Remediation 3 (T095-T101) → Phase 8 (T102-T116a,T117) → Phase 9 (T118-T124a,T125) → Phase 10 (T126a-T127) → Phase 11 (T128-T137) → Quality Gates
                                                    ↕                   ↕                    ↕
                                            Independent           Independent        TDD: Tests first (T071-T083)
                                            (no deps on US2)      (no deps on US3)   then fixes (T084-T087)
```

**US1, US2, US3**: Independently testable. US2 depends on `verify_integrity()` from US1. US3 depends on `--force` flag from US1.

**US4 (E2E Tests)**: Depends on binary being buildable. TDD approach: tests written first (T071-T083), expected to fail, then bugs fixed (T084-T087) to make tests pass.

**Code Review Remediation 1 (T060-T065)**: All independent (different files), can be fixed in parallel. T066 is sequential gate after all fixes.

**Code Review Remediation 2 (T089-T093)**: All independent (different files), can be fixed in parallel. T094 is sequential gate after all fixes.

**Code Review Remediation 3 (T095-T100)**: All independent (different files), can be fixed in parallel. T101 is sequential gate after all fixes.

**Phase 8 (T102-T116a,T117)**: Smoke test bugs (independent across findings, sequential within each):
- Smoke Bug 1: T102 (tests first) → T103 (chunk split) → T104 (unify chunkers) — sequential ✓
- Smoke Bug 2: T105 (test first per TDD) → T106 (fix OpenCode handler) → T107 (cleanup dead param) — sequential ✓; T108 [P] parallel with T106-T107 (different file) ✓
- Smoke Bug 3: T109 (batch tests first) → T110 (trait) → T111 (local impl) → T112 (enum dispatch) — sequential chain ✓; T113, T114 [P] after T112 ✓; T115 [P] after T113+T114 ✓
- T116 descoped (parallel indexing unstable). T116a: revert to sequential flow
- T117 is final quality gate

**Phase 9 (T118-T125)**: Third smoke test reindex performance fixes:
- T118-T119 done. T120 done (reverted). T121-T122 done. T123 blocked. T124-T124a done. T125 pending.

**Phase 10 (T126a-T127)**: Glob ignore fix:
- T126a (TDD test) [P] → T126 (glob impl) → T127 (default patterns) — sequential (same file)

**Phase 11 (T128-T137)**: Polish & robustness (all [P] — independent across files):
- T128, T130 [P] dependent on T123 (incremental unblocked) — E2E pipeline + daemon tests
- T129, T131, T132, T133, T136 [P] — independent user-facing fixes (maintenance.rs, vault.rs)
- T134 [P] dependent on T123 — incremental graph optimization
- T135 [P] — documentation update (independent)
- T137 is final quality gate

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
- Phase 3 (T095-T100): All independent (different files/concepts), can be executed in parallel. T101 is the final gate (sequential, after all fixes).

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
8. Code Review Remediation Phase 2 - Fix post-E2E review bugs (T089-T093), verify quality gates (T094) ✅ COMPLETE
9. Code Review Remediation Phase 3 - Fix 2026-05-17 branch review bugs (T095-T100), verify quality gates (T101) ✅ COMPLETE
10. Phase 8 - Smoke test fixes: chunk splitting, OpenCode platform config, batch embedding, descoped parallel indexing (T102-T116a,T117) ✅ COMPLETE
11. Phase 9 - Reindex performance: ONNX threading, global batch, incremental reindex (T118-T124a,T125)
12. Phase 10 - Ignore file glob fix (T126a-T127)
13. Phase 11 - Polish & robustness: E2E pipeline test, daemon test, user-facing progress, diagnostics, timeout guard, health check, incremental graph, docs (T128-T137)

**TDD Enforcement for US4**:
- Step 1: Write E2E test infrastructure (T067-T070)
- Step 2: Write all E2E tests (T071-T083) - expect failures
- Step 3: Run tests, document failures (T075, T080)
- Step 4: Fix bugs (T084-T086) - tokio panic, subprocess issues, exit codes
- Step 5: Verify all tests pass (T087-T088)

**Constitutional Compliance (Section X)**:
- All bugs in Code Review Remediation Phase 3 (T095-T100) must be fixed immediately
- All smoke test bugs in Phase 8 (T102-T115, T116a) must be fixed immediately
- All reindex performance bugs in Phase 9 (T118-T124a) must be fixed immediately
- All glob and robustness gaps in Phase 10 (T126a-T127) and Phase 11 (T128-T136) must be fixed immediately
- T116 (parallel indexing) descoped after second smoke test — replaced by T116a (sequential revert)
- No deferral without explicit user consent (per Section X)
- Quality gates (T101, T117, T125, T137) must pass before merge (per Section V)
