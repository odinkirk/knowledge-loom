# Feature Specification: install-file-structure

**Feature Branch**: `005-install-file-structure`
**Created**: 2026-05-14
**Status**: Draft
**Input**: User description: "Necessary MCP files can and should be installed at root (e.g. opencode.json, .mcp.json, etc). Anything that is structurally required to be in a certain location should be placed where expected. However, things like fastembed and the like should be in the .knowledge-loom directory."

## Clarifications

### Session 2026-05-14

- Q: Implementation Technology → A: Rust binary via existing `loom` CLI (`loom install [--force]`)
- Q: Install Command Scope → A: Install copies files only; `cargo test --release` is a separate verification step
- Q: Existing Directory Without --force → A: Exits with error and message telling user to use `--force`
- Q: Mid-Copy Failure Handling → A: Leave partial state, exit with error and message recommending user run with `--force` to restart
- Q: Installation Scope → A: Only runtime data (embedding models, fastembed cache) goes to `.knowledge-loom`; source code stays at root; index stays in `.knowledge-loom-index` (separate location)
- Q: Original Intent → A: Complete replacement: `loom install` sets up runtime data environment (fastembed models)
- Q: Runtime Data Contents → A: Embedding model files plus cache and config (fastembed models, cache, related config)
- Q: Model Download vs Index Location → A: Models go to `.knowledge-loom/models/`; index stays in `.knowledge-loom-index/` (separate root-level directory)
- Q: Already Installed Behavior → A: Skip download if exists, verify integrity, report "already installed"
- Q: Installation Summary Output → A: Show file count, total size (MB), and install location

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Install Runtime Data (Priority: P1)

A developer runs `loom install` to set up the runtime data environment. The system downloads fastembed models and supporting files into `.knowledge-loom/models/` with cache and config. MCP configuration files (opencode.json, .mcp.json) remain at repository root. Index data stays in `.knowledge-loom-index/`.

**Why this priority**: Provides the core runtime data setup needed for Knowledge Loom to function with offline embedding models.

**Independent Test**: After running `loom install`, verify that:
- `.knowledge-loom/models/` contains fastembed model files.
- Root-level MCP configs (opencode.json, .mcp.json) are untouched.
- `.knowledge-loom-index/` is unchanged.
- `cargo test --release` passes.

**Acceptance Scenarios**:
1. **Given** a clean repository, **When** `loom install` is executed, **Then** `.knowledge-loom/models/` contains fastembed model files plus cache and config.
2. **Given** an existing `.knowledge-loom/models/` with valid models, **When** `loom install` runs, **Then** it skips download, verifies integrity, and reports "already installed".

---

### User Story 2 - Verify Runtime Data Integrity (Priority: P2)

A CI pipeline runs `loom install` to set up runtime data. The pipeline must fail if model files are corrupted or missing. After install, `cargo test --release` passes.

**Why this priority**: Ensures CI reliability and prevents runtime failures due to missing/corrupted models.

**Independent Test**: CI runs `loom install && cargo test --release`. The job passes only when both steps succeed.

**Acceptance Scenarios**:
1. **Given** a CI runner with no cached models, **When** `loom install` finishes, **Then** it reports "Installation successful" and proceeds to the test step.
2. **Given** a corrupted model file in `.knowledge-loom/models/`, **When** `loom install` runs, **Then** it re-downloads the model and reports success.

---

### User Story 3 - Re‑install or Update Runtime Data (Priority: P3)

After a model version update, a developer wants to re‑install runtime data without manually cleaning `.knowledge-loom/models/`.

**Why this priority**: Improves developer ergonomics and ensures model version consistency.

**Independent Test**: Run `loom install --force`, verify updated model files appear in `.knowledge-loom/models/`, and all tests pass.

**Acceptance Scenarios**:
1. **Given** a new model version available, **When** `loom install --force` is run, **Then** the updated model overwrites the previous version in `.knowledge-loom/models/`.
2. **Given** no `--force` flag, **When** models already exist, **Then** it exits with a message telling user to use `--force`.

---

### User Story 4 - End-to-End Test Coverage (Priority: P1)

Developers run `cargo test` and expect a **full suite** of end-to-end tests to pass, catching issues that integration tests miss (e.g., tokio runtime panics, binary invocation errors, subprocess failures, async runtime conflicts).

**Why this priority**: Integration tests call library functions directly and miss runtime-level bugs. E2E tests exercise the actual compiled binary as users invoke it, catching panics, exit codes, and subprocess behavior.

**Independent Test**: Run `cargo test` and verify all E2E tests pass across all command categories.

**Acceptance Scenarios** - E2E tests must cover:

**Category 1: `loom init` command**
1. **Given** a clean test directory, **When** E2E test runs `loom init` as subprocess, **Then** command completes without panic and returns exit code 0
2. **Given** a tokio runtime context, **When** E2E test invokes `loom init`, **Then** no "Cannot start a runtime from within a runtime" panic occurs
3. **Given** an already-initialized directory, **When** `loom init` runs, **Then** it reports "already initialized" and exits gracefully
4. **Given** a directory with partial initialization, **When** `loom init` runs, **Then** it completes the setup

**Category 2: `loom install` command**
5. **Given** a clean `.knowledge-loom/` directory, **When** `loom install` runs, **Then** model downloads and installs successfully
6. **Given** an existing valid model, **When** `loom install` runs without `--force`, **Then** it reports "already installed" and exits 0
7. **Given** an existing model, **When** `loom install --force` runs, **Then** it re-downloads and overwrites
8. **Given** a corrupted model file, **When** `loom install` runs, **Then** it detects corruption and re-downloads

**Category 3: `loom serve` command**
9. **Given** an initialized knowledge base, **When** `loom serve` starts, **Then** MCP server starts and accepts connections
10. **Given** a running `loom serve` process, **When** SIGTERM is sent, **Then** server shuts down gracefully

**Category 4: `loom shell` command**
11. **Given** an initialized knowledge base, **When** `loom shell` runs, **Then** interactive shell starts with MCP server running

**Category 5: Regression Prevention**
12. **Given** any code change that breaks binary invocation, **When** developer runs `cargo test`, **Then** E2E tests fail and catch the regression before merge
13. **Given** a tokio runtime bug introduced, **When** E2E tests run, **Then** they detect the panic and fail

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System **MUST** download and install fastembed model files into `.knowledge-loom/models/` with cache and config.
- **FR-002**: System **MUST** verify model file integrity (checksum) after download and re-download on mismatch.
- **FR-003**: System **MUST** create `.knowledge-loom/models/` if it does not exist.
- **FR-004**: System **MUST** provide a `--force` flag that re-downloads models. Without `--force`, skip download if models already exist and report "already installed".
- **FR-005**: System **MUST** abort with a clear error if download fails (e.g., network error, disk full). Error message **MUST** recommend running with `--force` to retry.
- **FR-006**: System **MUST** output a concise summary showing: model version, download size, and target location.
- **FR-007**: System **MUST** exit with status code `0` only when runtime data is installed correctly (test suite verification is a separate step).
- **FR-008**: System **MUST** leave MCP configuration files (`opencode.json`, `.mcp.json`) and `.knowledge-loom-index/` untouched.
- **FR-009**: System **MUST** provide end-to-end tests that invoke the compiled `loom` binary as a subprocess for all user-facing commands (`loom init`, `loom install`, `loom serve`).
- **FR-010**: E2E tests **MUST** catch tokio runtime panics, subprocess failures, and exit code errors that integration tests miss.
- **FR-011**: All tests (unit, integration, E2E) **MUST** pass before merge — zero failures tolerated.
- **FR-012**: No tests **MUST** be removed, bypassed, or marked `#[ignore]` without explicit authorization.

### Key Entities

- **Repository Root**: The directory containing the source code and all configuration files.
- **Runtime Data Directory**: `.knowledge-loom/models/` contains fastembed model files, cache, and config.
- **Index Directory**: `.knowledge-loom-index/` (separate, unchanged by this feature).
- **MCP Config Files**: `opencode.json`, `.mcp.json` (stay at repository root).
- **Model Download URL**: Remote endpoint to fetch fastembed models (configurable).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: After running `loom install`, `.knowledge-loom/models/` contains fastembed model files.
- **SC-002**: Corrupted model files are detected via checksum and re-downloaded automatically.
- **SC-003**: MCP config files (`opencode.json`, `.mcp.json`) remain at root, unmodified.
- **SC-004**: Installation (download + verify) completes within 30 seconds on a standard developer machine (macOS 12+, SSD, 100Mbps connection).
- **SC-005**: Full E2E test suite executes in under 5 minutes on CI infrastructure.
- **SC-006**: All tests (unit, integration, E2E) pass with zero failures before merge.
- **SC-007**: E2E tests catch tokio runtime panics and subprocess failures that integration tests miss.

## Assumptions

- The user has write permission to the repository root.
- The repository is checked out with a clean working tree.
- Internet connectivity available for initial model download.
- Disk space is sufficient (> 500 MB) for model storage.

## Edge Cases

- No internet: Abort with clear error and manual download instructions
- Disk full mid-download: Abort with error, partial files cleaned up
- Corrupted model file: Re-download on next `loom install` without `--force`
- Already installed with valid model: Report "already installed" and exit 0

## Knowledge Loom Specific Requirements

### Performance Requirements

- **PERF-001**: Installation must not exceed **30 seconds** on a typical developer workstation.

### Testing Requirements *(mandatory for all features)*

- **TEST-001**: Unit tests **MUST** achieve at least **80%** coverage for runtime data installation logic.
- **TEST-002**: Integration tests **MUST** verify model download and checksum validation.
- **TEST-003**: Tests **MUST** verify that `--force` re-downloads even if models exist.
- **TEST-004**: Tests **MUST** verify error handling on download failures (network, disk full).
- **TEST-005**: Tests **MUST** confirm MCP config files and `.knowledge-loom-index/` remain untouched.
- **TEST-006**: **End-to-End tests MUST** invoke the compiled `loom` binary as a subprocess for all user-facing commands.
- **TEST-007**: **E2E tests MUST** cover: `loom init` (fresh, re-init, partial), `loom install` (fresh, already installed, --force, corrupted), `loom serve` (start, graceful shutdown), `loom shell` (interactive start).
- **TEST-008**: **E2E tests MUST** catch tokio runtime panics, subprocess failures, and exit code errors.
- **TEST-009**: **All tests MUST pass** before merge — zero failures tolerated.
- **TEST-010**: **No tests MAY be removed**, bypassed, or marked `#[ignore]` without explicit authorization.

## Module Impact *(mandatory for all features)*

- **src/install.rs** (new or updated module): Runtime data setup logic (model download, checksum, caching).
- **src/main.rs**: Extended with `loom install [--force]` CLI subcommand.
- **src/model.rs** (already exists from feature 004): Reuse model download logic, adapt for fastembed model files.

