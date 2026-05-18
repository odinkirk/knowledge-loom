# Implementation Plan: install-file-structure

**Branch**: `005-install-file-structure` | **Date**: 2026-05-14 | **Spec**: [link](./spec.md)
**Input**: Feature specification from `specs/005-install-file-structure/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command.

## Summary

Implement `loom install` command that downloads and installs fastembed model files into `.knowledge-loom/models/` with cache and config. MCP configuration files (opencode.json, .mcp.json) remain at repository root; index data stays in .knowledge-loom-index/. Supports --force for re-download, checksum-based integrity verification, and graceful error handling with clear user messaging. **Includes comprehensive E2E test suite** that invokes compiled `loom` binary as subprocess for all commands (`init`, `install`, `serve`, `shell`), catching runtime-level bugs (tokio panics, exit codes, subprocess failures) that integration tests miss. **All existing tests passing**. Code review identified 6 issues (1 medium, 3 low, 2 info) requiring remediation before merge per Constitution Section X — all resolved. **Smoke test (2026-05-17) identified 3 additional defects: chunk truncation, OpenCode platform config, reindex performance, and parallel indexing opportunity. Phase 8 tasks (T102-T117) pending.**

## Technical Context

**Language/Version**: Rust 1.75+
**Primary Dependencies**: reqwest (HTTP download), sha2 (checksum), tokio (async runtime), serde/serde_json (state persistence), anyhow/thiserror (error handling), tempfile (E2E test isolation)
**Storage**: Model files stored in `.knowledge-loom/models/` on filesystem
**Testing**: cargo test (built-in), tempfile for file system tests, std::process::Command for E2E subprocess invocation
**Target Platform**: Linux, macOS, Windows (cross-platform CLI tool)
**Project Type**: Library/Package with CLI binary
**Performance Goals**: Model download + verification < 30s (100Mbps connection); E2E test suite < 5 minutes
**Constraints**: Offline-capable after installation, no external network dependencies at runtime; E2E tests must catch tokio runtime panics
**Scale/Scope**: Single model download (~120MB), single file per model; 13 E2E test scenarios across 4 command categories

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **III. Test-First Development (NON-NEGOTIABLE)**: All new code will follow TDD cycle. E2E tests written before bug fixes. [PASS]
- **IV. Integration Testing**: E2E tests required for CLI commands (`loom init`, `install`, `serve`, `shell`). [PASS]
- **V. Quality Gates**: Formatting, linting, testing, 80% coverage, security checks. All tests must pass before merge. [PASS]
- **IX. Output Conventions**: Use `eprintln!` for debug/logging, `println!` for CLI output. [PASS]
- **X. Code Exploration**: Use code-review-graph tools for Rust analysis. [PASS]
- **Commit Policy**: Explicit individual consent required for each commit. [PASS]
- **Naming**: snake_case files/vars, PascalCase types, SCREAMING_SNAKE_CASE constants. [PASS]
- **Error Handling**: `Result<T, E>` with `anyhow::Error`/`thiserror::Error`. [PASS]
- **Async Patterns**: tokio for async operations. E2E tests verify async compatibility. [PASS]
- **Documentation**: Doc comments on public APIs, CHANGELOG and ARCHITECTURE updates. [PASS]

## Project Structure

### Documentation (this feature)

```text
specs/005-install-file-structure/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
src/
├── install.rs           # NEW: Runtime data setup (model download, checksum, caching)
├── main.rs              # UPDATED: Add loom install [--force] subcommand
├── model.rs             # EXISTING: Reuse model download logic from feature 004
├── ...                  # All other modules unchanged
```

### Test Structure

```text
tests/
├── e2e/
│   ├── mod.rs              # E2E test module
│   ├── helpers.rs          # Subprocess helpers
│   ├── init_tests.rs       # E2E tests for `loom init`
│   ├── install_tests.rs    # E2E tests for `loom install`
│   ├── serve_tests.rs      # E2E tests for `loom serve`
│   └── shell_tests.rs      # E2E tests for `loom shell`
├── integration.rs          # Existing integration tests
├── install_tests.rs        # Existing unit/integration tests
└── ...                     # Other existing test files
```

## Complexity Tracking

**Original Estimate**: Straightforward model download with standard patterns

**Post-Review Reality**: Medium-severity bugs discovered requiring immediate remediation

**Constitutional Status**:
- ✅ All tests passing (Section V satisfied)
- ✅ All 6 code review findings fixed (T095-T100 complete)
- ✅ Quality gates pass: fmt, clippy, tests (T101 complete)
- ✅ Code review findings documented in plan (proper workflow followed)
- ✅ All findings FIXED before merge (Section X default - no deferrals)

**Next Steps**: Phase 8 (T102-T117) pending: (1) chunk truncation → split, (2) OpenCode platform config format, (3) reindex batch embedding, (4) parallel BM25+vector indexing.

## Technical Debt Remediation Plan

**Identified**: 2026-05-16 | **Severity**: Low-Medium | **Impact**: Reliability & Code Quality

### Issues to Address (NOT Deferred)

1. **Missing retry logic for network failures** (SEVERITY: MEDIUM)
   - Current: Single download attempt, fails on transient network errors
   - Impact: Users must manually run `--force` after any network glitch
   - **Action**: Reuse `DownloadManager` from `download.rs` with exponential backoff
   - **Timeline**: Address in Feature 006 if it involves downloads, or as dedicated refactoring

2. **Code duplication with existing download infrastructure** (SEVERITY: MEDIUM)
   - Current: `install.rs` duplicates download functionality from `download.rs`
   - Impact: Two code paths to maintain, inconsistent error handling
   - **Action**: Refactor to create shared download utilities module
   - **Timeline**: Dedicate 1-2 days for refactoring sprint before Feature 007

## Code Review Findings (Post-Implementation)

### Review 1: 2026-05-16 (Automated Code Review)

**Status**: ✅ All items fixed in commits 50bbefd, 90591a9, 0a683a6

**Fixed Issues**:
1. ✅ Incorrect checksum error message (`src/install.rs:102-106`) - Fixed in 0a683a6
2. ✅ Platform install inconsistency (`src/init.rs:256-262`) - Fixed in 0a683a6
3. ✅ Dead code warnings in test helpers (`tests/e2e_helpers.rs`) - Fixed with `#[allow(dead_code)]` in 0a683a6
4. ✅ Args collection design - Reviewed and approved to skip (Option A)
5. ✅ Test assertion logic (`tests/chunks_tests.rs:203-229`) - Already correct
6. ✅ Missing `run_init_with_binary` function - Fixed in 50bbefd
7. ✅ Duplicate model validation check - Fixed in 50bbefd
8. ✅ Clippy warnings - Fixed in 0a683a6
9. ✅ Short flag validation - Fixed in 50bbefd

---

### Review 2: 2026-05-17 (Branch Review - Current)

**Reviewed**: 2026-05-17 | **Severity**: Low-Medium | **Status**: Requires remediation before merge

#### Medium Severity Bugs

1. **`is_installed` checks directory existence, not file existence** (`src/install.rs:69-71`)
   - **Issue**: `model_path().exists()` checks `.knowledge-loom/models/` directory, not `model.onnx` file
   - **Root Cause**: `model_path()` returns directory path, not file path
   - **Impact**: False positive if directory exists but model file is missing; `is_installed()` returns `true` while `verify_integrity()` returns `Ok(false)`
   - **Fix**: Check for actual model file existence: `self.model_path().join("model.onnx").exists()`
   - **Timeline**: Immediate fix required (Section X default - misleading API)

#### Low Severity Issues

2. **Two `check_disk_space` functions with different behavior** (`src/download.rs:668`, `src/download/utils.rs:53`)
   - **Issue**: Different buffer logic (10% headroom vs none), different parent dir handling, platform support differs
   - **Impact**: `download/utils.rs` version has no Windows implementation (compile error risk)
   - **Fix**: Consolidate into single function in `download/utils.rs`, remove duplicate from `download.rs`
   - **Timeline**: Fix immediately (code quality, potential portability bug)

3. **`verify_integrity` loads full model into memory** (`src/install.rs:146`)
   - **Issue**: Reads entire 90MB model file into memory for checksum calculation
   - **Impact**: 90MB heap allocation; existing `download.rs:633-651` has streaming version
   - **Fix**: Make `calculate_checksum` accept `impl Read` or reuse streaming approach
   - **Timeline**: Fix immediately (performance improvement, straightforward)

4. **`download_with_retry` uses `Option<impl Fn>` parameter** (`src/download/utils.rs:28`)
   - **Issue**: `Option<impl Fn(...)>` makes callback awkward to use; callers must write `Some(|p| ...)` even when they want callback
   - **Impact**: Awkward API, unnecessary `Option` wrapping
   - **Fix**: Make callback a generic `F: Fn(...)` parameter or branch at call site
   - **Timeline**: Fix immediately (API design, low effort)

#### Info Severity Issues

5. **Network-dependent E2E tests without guards** (`tests/e2e_install_tests.rs`)
   - **Issue**: Tests attempt real HTTP downloads from Hugging Face without network guards
   - **Impact**: Will fail in CI without network access
   - **Fix**: Add `#[ignore]` or `LOOM_TEST_NETWORK` env guard to network-dependent tests
   - **Timeline**: Fix immediately (CI reliability)

6. **Directory mtime check (should check file mtime)** (`tests/e2e_install_tests.rs:62-74`)
   - **Issue**: Checks `modified()` on directory, not model file; directory mtime behavior is filesystem-dependent
   - **Impact**: Test may fail on some filesystems where replacing files doesn't update directory mtime
   - **Fix**: Check `model_dir.join("model.onnx").metadata().modified()` instead
   - **Timeline**: Fix immediately (test reliability)

### Remediation Approach

**Constitutional Compliance** (Section X):
- **Medium severity (#1)**: Fix immediately (Section X default - no consent needed for fixing)
- **Low severity (#2-#4)**: Fix immediately (all are straightforward, <1 hour each)
- **Info severity (#5-#6)**: Fix immediately (test reliability, CI blockers)

**Required Action**:
1. Fix all 6 issues before merge (Section V - all tests must pass, Section X - avoid technical debt)
2. No deferrals requested (all items are quick fixes)
3. Re-run full test suite after fixes

**Explicit Consent**: Not required for fixing (Section X default is to fix). If user wants to defer any items, explicit consent must be given for EACH item individually.

3. **Argument parsing could be more robust** (SEVERITY: LOW)
   - Current: Simple `args().any()` check for `--force` flag
   - Impact: Doesn't handle edge cases like `--force=value` or provide detailed error messages
   - **Action**: Create shared CLI argument parsing utilities
   - **Timeline**: Address during standardization pass (same sprint as #2)

4. **Checksum field usage** (SEVERITY: LOW)
   - Status: ✅ RESOLVED - Added checksum display to success output in commit 5254fee

### Remediation Commitments

**Immediate** (before Feature 006):
- Document download infrastructure duplication in Feature 006 planning
- Estimate refactoring effort for shared utilities module

**Short-term** (before Feature 007):
- Dedicate sprint to refactoring: consolidate `download.rs`, `model.rs`, `install.rs`
- Create shared download utilities with retry logic, progress tracking, error handling
- Standardize CLI argument parsing across all subcommands

**Tracking**:
- Add technical debt remediation tasks to Feature 006/007 task lists
- Review technical debt status at start of each new feature planning
- Measure reduction in code duplication metrics after refactoring

## Refactoring Technical Specification (T053-T059)

**Added**: 2026-05-16 | **Phase**: Technical Debt Remediation | **Estimate**: 1-2 days

### Technical Approach

**Goal**: Consolidate download logic to eliminate duplication and ensure consistent error handling across all download operations.

**Architecture**:

```
src/
├── download/
│   ├── mod.rs           # DownloadManager, DownloadError, DownloadProgress
│   └── utils.rs         # NEW: Shared utilities (download_with_retry, checksum, disk space)
├── install.rs           # REFACTORED: Use DownloadManager from download/
├── model.rs             # REFACTORED: Use DownloadManager from download/
└── cli/
    └── args.rs          # NEW: Shared CLI argument parsing utilities
```

**Refactoring Steps**:

1. **T057: Write tests first** (Constitution Section III - TDD)
   - Unit tests for `download/utils.rs`: retry logic, checksum validation, disk space checking
   - Integration tests: verify DownloadManager works with various network conditions
   - Test file: `tests/download_utils_tests.rs`

2. **T053: Refactor install.rs**
   - Replace direct `reqwest::Client` usage with `DownloadManager`
   - Remove duplicate checksum logic → use `download::utils::validate_checksum()`
   - Keep install-specific logic: state management, integrity verification, `--force` flag
   - Expected changes: ~40% code reduction in download logic

3. **T054: Refactor model.rs**
   - Replace direct download logic with `DownloadManager`
   - Reuse progress tracking from download module
   - Keep model-specific logic: validation, metadata management
   - Expected changes: ~50% code reduction in download logic

4. **T055-T056: CLI argument parsing utilities**
   - Create `src/cli/args.rs` with reusable argument parsing
   - Handle edge cases: `--force=value`, unknown flags, missing values
   - Update `install.rs` and `init.rs` to use shared utilities

5. **T058: Update ARCHITECTURE.md**
   - Document consolidated download infrastructure
   - Show data flow: DownloadManager → utils → callers
   - Update module dependency diagram

6. **T059: Document in CHANGELOG.md**
   - Record technical debt reduction
   - Note breaking changes (if any) to internal APIs

### Testing Strategy

**Unit Tests** (T057):
- `test_download_with_retry_success()` - Verify retry logic works
- `test_download_with_retry_failure()` - Verify max retries exceeded
- `test_validate_checksum_match()` / `test_validate_checksum_mismatch()`
- `test_check_disk_space_sufficient()` / `test_check_disk_space_insufficient()`

**Integration Tests** (T057):
- `test_install_uses_download_manager()` - Verify install.rs uses shared logic
- `test_model_download_uses_download_manager()` - Verify model.rs uses shared logic

**Regression Tests**:
- All existing install tests must still pass
- All existing model tests must still pass
- Verify no behavior changes (only code reorganization)

### Risk Assessment

**Low Risk**:
- Checksum validation logic (pure function, well-tested)
- Disk space checking (isolated function)

**Medium Risk**:
- DownloadManager integration (need to verify progress callbacks work correctly)
- CLI argument parsing (must maintain backward compatibility)

**Mitigation**:
- TDD approach (tests before refactoring)
- Comprehensive regression testing
- Small, incremental commits with verification at each step

### Success Criteria

**Functional**:
- All existing tests pass (19 install tests + model tests)
- No behavior changes observable to users
- Download retry works for install.rs (new functionality)

**Code Quality**:
- Zero code duplication for download logic (DRY principle)
- Single source of truth for checksum validation
- Consistent error handling across all download operations
- Clippy passes with zero warnings

**Metrics**:
- Lines of code reduced by ~30% in download-related modules
- Code coverage maintained at >80%
- Cyclomatic complexity reduced in install.rs and model.rs

### Dependencies

**Blocks**: Feature 006 (if it involves downloads)
**Blocked By**: None (can proceed immediately)

### Constitution Compliance

- ✅ **Section III (TDD)**: Tests written before refactoring (T057 before T053-T054)
- ✅ **Section V (Quality Gates)**: All quality gates must pass after refactoring
- ✅ **Section X (Technical Debt)**: All review findings will be FIXED (not deferred)

## Code Review Findings & Remediation

**Identified**: 2026-05-16 | **Severity**: High (Blocking) | **Source**: Automated Code Review

**Constitutional Status**: All items below **will be fixed** before merge per Section X (Technical Debt must be avoided). No items are deferred. Explicit consent for deferral: NOT REQUESTED (all items will be fixed).

### Blocking Issues (Must Fix Before Merge)

**1. Missing `run_init_with_binary` Function**  
**Severity**: HIGH - Compilation Failure  
**Files Affected**: `tests/rename_tests.rs:46`, `tests/shell_tests.rs:31`  
**Root Cause**: Function removed during refactoring but tests still call it

**Fix Plan**: Restore `run_init_with_binary` as public function in `src/init.rs`

---

**2. Duplicate Model Validation Check**  
**Severity**: LOW - Code Quality  
**File**: `src/init.rs:259-275`  
**Issue**: Lines 259-266 and 268-275 perform identical model validity checks

**Fix Plan**: Remove duplicate check at lines 268-275

---

### Quality Gate Failures (Will Fix Before Merge)

**3. Clippy Warnings** (6+ instances)
- `src/download/utils.rs:6` - Unused `PathBuf` import
- `src/install.rs:84` - Unnecessary `mut` on `manager`
- `tests/download_integration_tests.rs:6` - Unused `PathBuf` import
- `tests/install_integration.rs:5-6` - Unused `PathBuf`, `Command` imports
- `tests/install_benchmark.rs:5` - Unused `PathBuf` import

**Fix Plan**: Remove unused imports and `mut` qualifiers

---

**4. Dead Code** (Multiple files)
- `src/download.rs`: `BUFFER_SIZE`, `format_download_error`, `acquire_lock`, `release_lock`
- `src/model.rs`: `DownloadStatus`, `DownloadState`, multiple `ModelManager` methods
- `src/download/utils.rs`: `download_with_retry`, `check_disk_space`

**Fix Plan**: Add `#[allow(dead_code)]` for planned future use, remove truly obsolete code

---

**5. Test Quality Issues**
- `src/cli/args.rs:168-181`: Tests accept any result without assertions
- `src/install.rs:199-202`: Empty test body with comment only

**Fix Plan**: Implement proper assertions

---

**6. Incomplete Validation**
- `src/cli/args.rs:133-140`: Short flag validation is a no-op

**Fix Plan**: Implement proper short flag validation (fix now, before merge)

---

### Fix Commitments (Not Deferrals)

**All items above will be fixed before merge**. This section tracks progress, not deferrals.

**Fix Sequence**:
1. Restore `run_init_with_binary` function (unblock tests)
2. Remove duplicate model validation check
3. Fix all clippy warnings
4. Fix test quality issues
5. Audit dead code (add `#[allow(dead_code)]` or remove)
6. Implement proper short flag validation

**Testing Requirements**:
- All 26 tests must pass (7 integration + 8 utils + 11 install)
- Pre-existing tests must pass (rename_tests, shell_tests)
- Zero clippy warnings (`cargo clippy -- -D warnings`)

### Metrics

**Current State**:
- Tests: 26/26 passing (new tests) + 2 failing (pre-existing due to missing function)
- Clippy: 6+ warnings (will fix)
- Dead Code: 10+ unused functions/constants (will audit)

**Target State** (Before Merge):
- Tests: All passing
- Clippy: Zero warnings
- Dead Code: Documented with `#[allow(dead_code)]` or removed

## End-to-End Test Suite Technical Specification (US4)

**Added**: 2026-05-17 | **Phase**: E2E Test Implementation | **Estimate**: 1 day

### Technical Approach

**Goal**: Create comprehensive E2E test suite that invokes compiled `loom` binary as subprocess, catching runtime-level bugs (tokio panics, exit codes, subprocess failures) that integration tests miss.

**Architecture**:

```
tests/
├── e2e/
│   ├── mod.rs              # E2E test module
│   ├── helpers.rs          # Subprocess helpers (run_loom_cmd, assert_exit_code)
│   ├── init_tests.rs       # E2E tests for `loom init`
│   ├── install_tests.rs    # E2E tests for `loom install`
│   ├── serve_tests.rs      # E2E tests for `loom serve`
│   └── shell_tests.rs      # E2E tests for `loom shell`
├── integration.rs          # Existing integration tests (unchanged)
└── ...                     # Other existing test files
```

**Key Design Decisions**:

1. **Subprocess Invocation**: Use `std::process::Command` to invoke compiled binary
2. **Test Isolation**: Use `tempfile::tempdir()` for isolated test directories
3. **Timeout Handling**: Set reasonable timeouts (30s) for subprocess calls
4. **Panic Detection**: Capture stderr output, detect tokio runtime panics
5. **Exit Code Validation**: Assert expected exit codes (0 for success, non-zero for failures)

### Test Categories

**Category 1: `loom init` E2E Tests** (T071-T075)
- Fresh initialization: clean directory → exit 0, no panic
- Tokio runtime context: invoke from async test → no "runtime within runtime" panic
- Re-initialization: already initialized → "already initialized" message, exit 0
- Partial initialization: missing model → completes setup

**Category 2: `loom install` E2E Tests** (T076-T080)
- Fresh install: clean `.knowledge-loom/` → downloads model, exit 0
- Skip valid: existing valid model → "already installed", exit 0
- Force re-download: `--force` flag → overwrites existing
- Corrupted model: detects corruption → re-downloads

**Category 3: `loom serve` E2E Tests** (T081-T082)
- Server start: initialized KB → MCP server accepts connections
- Graceful shutdown: SIGTERM → clean exit, no zombie processes

**Category 4: `loom shell` E2E Test** (T083)
- Interactive shell: starts with MCP server running

### Bug Fix Phase (T084-T087)

**After E2E tests written (expecting failures)**:

1. **T084: Fix tokio runtime panic in `loom init`**
   - Root cause: `run_init()` called from `#[tokio::main]` but `download_model()` tries `block_on`
   - Fix: Make `run_init()` async-compatible (use `download_model_async()` or handle existing runtime)
   - Test: T072 passes (no panic in tokio context)

2. **T085: Fix subprocess invocation bugs**
   - Debug any path/binary location issues exposed by E2E tests
   - Fix environment variable handling (KB_ROOT)

3. **T086: Fix exit code handling**
   - Ensure all error paths return appropriate exit codes
   - Verify success paths return 0

4. **T087: Verify all E2E tests pass**
   - Run `cargo test --test e2e_*` → all pass
   - Run full `cargo test --all-features` → zero failures

### Testing Strategy

**TDD Enforcement**:
1. Write E2E tests FIRST (T071-T083) → expect failures
2. Run tests → document failures (tokio panic, etc.)
3. Fix bugs (T084-T086) → make tests pass
4. Verify (T087) → all tests pass

**Test Helpers** (`tests/e2e/helpers.rs`):
- `run_loom_cmd(args: &[&str], temp_dir: &Path) -> CommandOutput`
- `assert_exit_code(output: CommandOutput, expected: i32)`
- `assert_no_panic(output: CommandOutput)`
- `assert_contains(output: CommandOutput, expected_substring: &str)`

**CommandOutput struct**:
```rust
struct CommandOutput {
    exit_code: i32,
    stdout: String,
    stderr: String,
    panicked: bool,  // detected via stderr patterns
}
```

### Risk Assessment

**Low Risk**:
- Subprocess invocation (standard `std::process::Command`)
- Exit code assertion (simple integer comparison)

**Medium Risk**:
- Tokio runtime panic fix (may require async refactoring)
- Test isolation (tempfile cleanup on failures)

**Mitigation**:
- TDD approach (tests first, then fix)
- Small incremental commits
- Verify each test category independently

### Success Criteria

**Functional**:
- All E2E tests pass (13 scenarios across 4 categories)
- Tokio runtime panic bug fixed
- No false positives (tests don't fail on valid code)

**Code Quality**:
- E2E tests follow project conventions
- Helper utilities are reusable and well-documented
- Test output is clear on failure (helps debugging)

**Metrics**:
- E2E test suite executes in < 5 minutes
- 100% of user-facing commands covered (`init`, `install`, `serve`, `shell`)
- Zero tokio runtime panics in E2E tests

### Dependencies

**Blocks**: Merge of feature 005 (E2E tests must pass before merge per spec)
**Blocked By**: None (can proceed immediately after spec approval)

### Constitution Compliance

- ✅ **Section III (TDD)**: Tests written before bug fixes (T071-T083 before T084-T086)
- ✅ **Section V (Quality Gates)**: All tests must pass before merge
- ✅ **Section X (Technical Debt)**: Runtime bugs fixed immediately (not deferred)

## Smoke Test Findings (2026-05-17)

**Context**: Full smoke test on live corpus (91 markdown files, unspoken-world project). Three classes of defects discovered.

---

### Finding 1: Chunk Truncation Defect (SEVERITY: MEDIUM)

**Observed**: The Story Bible (1403 lines, 131 heading sections) indexed as 133 embedding rows — correct count. But each section is truncated to 2000 characters instead of being split into multiple sequential chunks.

**Root Cause**: `parse_chunks()` in `src/chunks.rs:88–169` creates one chunk per heading section. When section content exceeds `MAX_CHUNK_CHARS` (2000), `truncate_at_whitespace()` (line 140) silently discards everything beyond the limit. Content past 2000 chars is permanently absent from both BM25 and vector search.

**Impact**: Long narrative sections lose most of their content from the search index. Which portion survives depends on lexicographic truncation — the first 2000 chars of each section are included, the rest is lost.

**Additional Issue**: `VectorIndex::chunk_content()` in `src/index.rs:260–282` uses a completely different chunking strategy (per-heading splits with no size cap or ordinal assignment), meaning BM25 and vector search operate over different chunk boundaries, degrading RRF fusion quality.

**Expected Behavior**:
1. Sections longer than `MAX_CHUNK_CHARS` are split into multiple sequential chunks, each ≤ 2000 chars, at whitespace boundaries
2. All chunks from the same heading share the same heading breadcrumb context
3. Chunk ordinals remain file-local and sequential across all chunks including splits
4. The headingless fallback also splits long content instead of truncating
5. `VectorIndex::chunk_content()` is replaced by reuse of `parse_chunks()` so both indexes share consistent chunk boundaries

---

### Finding 2: OpenCode Platform Config Defect (SEVERITY: HIGH)

**Observed**: `loom init --platform opencode` produces a wrong `opencode.json` and an unwanted `.mcp.json`.

**Root Cause — Bug A**: `run_init_async()` in `src/init.rs:319–335` unconditionally creates `.mcp.json` before calling `install_platform()`. For OpenCode platform, this produces both `.mcp.json` (unwanted) and `opencode.json` (broken).

**Root Cause — Bug B**: The OpenCode handler in `src/platforms.rs:153–160` calls `write_json_object_entry(&path, "mcpServers", ...)` with `opencode=true`. The `build_entry()` function (platforms.rs:200–213) with `opencode=true` sets `env` to `[]` (a hack for an older format) and produces `type: "stdio"`.

**Research — Correct Format**: Verified against the authoritative OpenCode config schema at `https://opencode.ai/config.json` (§`McpLocalConfig`) and the working local `opencode.json` from the test corpus.

| Aspect | Current Code Output | Required Format |
|---|---|---|
| Schema | missing `$schema` | `"$schema": "https://opencode.ai/config.json"` |
| MCP key | writes `mcpServers` | must write `mcp` (primary key per schema) |
| `type` field | `"stdio"` | `"local"` |
| `command` field | single string | array of strings: `[binary_path, "serve"]` |
| `environment` field | `[]` (empty array) | object: `{"KB_ROOT": "/path/to/kb"}` |
| `.mcp.json` | created unconditionally | must NOT be created for OpenCode |

Note: The `mcpServers` key in the existing corpus file is NOT defined in the OpenCode config schema and appears to be legacy/alien. The only MCP key defined by the schema is `mcp`.

**Expected Behavior**:
1. When `--platform opencode` is specified, `run_init_async()` skips `.mcp.json` creation
2. `install_platform(OpenCode, ...)` writes `opencode.json` with correct `$schema`, `mcp` key, `type: "local"`, `command` as array, and `environment` as object
3. `build_entry()` removal of `opencode` boolean parameter — format is always the schema-conformant `mcp` key format

---

### Finding 3: Reindex Performance Defect (SEVERITY: MEDIUM)

**Observed**: `loom reindex` for 91 markdown files (3998 chunks) takes multiple minutes. Users expect seconds for this corpus size.

**Root Cause**: `VectorIndex::index_vault()` in `src/index.rs:201–258` calls `embed_provider.embed()` once per chunk — 3998 individual ONNX model inferences. The local embed provider at `src/embed/local.rs:187` passes a single-element Vec to fastembed: `model.embed(vec![text], None)`. fastembed supports batch inference but the code never uses it.

**Performance Analysis**:
- Per-chunk embedding: ~80–150ms × 3998 chunks ≈ 5–10 minutes for embedding alone
- Plus BM25 indexing (sequential per-file), graph building, and SQLite writes add more
- With batch size 32: 125 inference calls instead of 3998 → ~30× speedup
- Total reindex time should be <10 seconds for a 91-file corpus with batch embedding

**Expected Behavior**:
1. Chunks are batched (e.g., 32 at a time) before calling fastembed for inference
2. BM25 and vector indexing can proceed in parallel (currently serial)
3. `loom reindex` completes in under 10 seconds for 100-file corpora
