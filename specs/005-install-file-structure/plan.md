# Implementation Plan: install-file-structure

**Branch**: `005-install-file-structure` | **Date**: 2026-05-14 | **Spec**: [link](./spec.md)
**Input**: Feature specification from `specs/005-install-file-structure/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command.

## Summary

Implement `loom install` command that downloads and installs fastembed model files into `.knowledge-loom/models/` with cache and config. MCP configuration files (opencode.json, .mcp.json) remain at repository root; index data stays in .knowledge-loom-index/. Supports --force for re-download, checksum-based integrity verification, and graceful error handling with clear user messaging.

## Technical Context

**Language/Version**: Rust 1.75+
**Primary Dependencies**: reqwest (HTTP download), sha2 (checksum), tokio (async runtime), serde/serde_json (state persistence), anyhow/thiserror (error handling)
**Storage**: Model files stored in `.knowledge-loom/models/` on filesystem
**Testing**: cargo test (built-in), tempfile for file system tests
**Target Platform**: Linux, macOS, Windows (cross-platform CLI tool)
**Project Type**: Library/Package with CLI binary
**Performance Goals**: Model download + verification < 30s (100Mbps connection)
**Constraints**: Offline-capable after installation, no external network dependencies at runtime
**Scale/Scope**: Single model download (~120MB), single file per model

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **III. Test-First Development (NON-NEGOTIABLE)**: All new code will follow TDD cycle. [PASS]
- **V. Quality Gates**: Formatting, linting, testing, 80% coverage, security checks. [PASS]
- **IX. Output Conventions**: Use `eprintln!` for debug/logging, `println!` for CLI output. [PASS]
- **X. Code Exploration**: Use code-review-graph tools for Rust analysis. [PASS]
- **Commit Policy**: Explicit individual consent required for each commit. [PASS]
- **Naming**: snake_case files/vars, PascalCase types, SCREAMING_SNAKE_CASE constants. [PASS]
- **Error Handling**: `Result<T, E>` with `anyhow::Error`/`thiserror::Error`. [PASS]
- **Async Patterns**: tokio for async operations. [PASS]
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

## Complexity Tracking

No constitution violations expected. Implementation is straightforward model download with standard patterns.

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
