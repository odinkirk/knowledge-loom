# Feature Specification: Documentation Cleanup and CI Modernization

**Feature Branch**: `009-docs-ci-cleanup`  
**Created**: 2026-06-06  
**Status**: Draft  
**Input**: User description: "All documentation shipped with spec 008 (turbovec integration) still references sqlite-vec as the vector backend in README.md and CONTRIBUTING.md, and the Model Download Flow mermaid diagram in Architecture.md fails to parse due to reserved single-letter node IDs. Additionally, all three GitHub Actions workflows (test, build, release) use deprecated action versions (actions-rs/toolchain@v1, actions-rs/cargo@v1, actions-rs/audit-check@v1, actions/cache@v3) and lack cargo-deny caching, causing CI flakiness and deprecation warnings."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Accurate Project Documentation (Priority: P1)

A developer new to the project reads README.md and CONTRIBUTING.md to understand the architecture. The documentation accurately describes the current vector backend (turbovec) rather than the replaced sqlite-vec, providing correct setup prerequisites and performance expectations.

**Why this priority**: Stale documentation is the most visible and impactful issue — it misleads every new contributor and user about fundamental project architecture and setup requirements.

**Independent Test**: Read README.md and CONTRIBUTING.md and verify zero references to "sqlite-vec" or "SQLite/vec" as the active vector backend, with all references correctly describing turbovec. Verify the Rust version requirement is consistent across all docs.

**Acceptance Scenarios**:

1. **Given** a contributor reads README.md, **When** they review the architecture section and features table, **Then** all vector search references describe turbovec, not sqlite-vec.
2. **Given** a contributor reads CONTRIBUTING.md prerequisites, **When** they check the dependency list, **Then** SQLite is not listed as a prerequisite for local data storage.
3. **Given** a user reads any project markdown file under the root, **When** they search for "sqlite-vec" or "SQLite/vec" as the active backend, **Then** no such references exist outside of historical spec documents and changelog entries documenting the migration.

---

### User Story 2 - Error-Free Mermaid Diagrams (Priority: P2)

A developer views Architecture.md in a Mermaid-compatible renderer (GitHub, IDE, or documentation site). The Model Download Flow diagram renders correctly without parse errors, allowing the developer to understand the model download pipeline visually.

**Why this priority**: A broken diagram in Architecture.md degrades the documentation experience and suggests the project is unmaintained or careless. The fix is straightforward (rename node IDs).

**Independent Test**: Render Architecture.md in a Mermaid viewer and verify all diagrams, including the Model Download Flow, render without errors.

**Acceptance Scenarios**:

1. **Given** Architecture.md is opened in a Mermaid-compatible renderer, **When** the Model Download Flow diagram is processed, **Then** it renders without any parse error.
2. **Given** the fix is applied, **When** any other Mermaid diagram in Architecture.md is rendered, **Then** no regressions are introduced — all existing diagrams continue to render correctly.

---

### User Story 3 - Reliable CI Pipeline (Priority: P3)

A maintainer pushes changes or opens a pull request. The CI pipeline (tests, builds across Linux, macOS, and Windows on stable Rust, plus release) runs successfully without deprecation warnings from GitHub Actions. The `cargo-deny` license check runs quickly using caching rather than re-installing every run.

**Why this priority**: While important for project health, CI pipeline modernization doesn't affect end-user documentation quality. Deprecated actions still function today but may break without notice and generate noise in CI logs.

**Independent Test**: Push to a branch and observe that all three workflows (test, build, release) complete with zero deprecation warnings in the Actions logs. Verify that `cargo-deny` uses a cached installation on repeat runs.

**Acceptance Scenarios**:

1. **Given** a PR is opened, **When** the test workflow runs, **Then** no deprecation warnings appear for `actions-rs/toolchain`, `actions-rs/cargo`, `actions-rs/audit-check`, or `actions/cache` actions.
2. **Given** the build workflow runs, **When** it executes on ubuntu-latest, macos-latest, and windows-latest with stable Rust, **Then** all three OS targets compile successfully in debug and release modes.
3. **Given** a tag starting with `v` is pushed, **When** the release workflow runs, **Then** tests pass, the release binary builds, and the publish step executes without deprecated action warnings.
4. **Given** the test workflow runs, **When** the license check step executes, **Then** `cargo-deny` uses a cached binary on repeat runs, completing in under 10 seconds.

---

### Edge Cases

- What happens when a Mermaid diagram uses node IDs that are reserved words in one Mermaid version but not another? (Fix: use multi-character IDs consistently.)
- How does the README handle mentioning sqlite-vec in historical context (Changelog, Credits section)? (Keep historical references but ensure they are clearly marked as prior/legacy.)
- What if `cargo-deny` is already installed on the CI runner? (The cache action should still provide a faster path; installation should be a fallback.)
- What happens if the `GITHUB_TOKEN` secret is unavailable for the security audit step? (The audit step should fail gracefully with a clear error rather than a cryptic permissions error.)

## Clarifications

### Session 2026-06-06

- Q: Cargo.toml has no `rust-version` field, yet the spec references it as MSRV source of truth. What is the canonical MSRV? → A: `1.75.0` — add `rust-version = "1.75"` to Cargo.toml, align README badge and CONTRIBUTING.md to match.
- Q: Should the build.yml matrix keep beta and nightly Rust channels, or drop to stable-only? → A: Drop beta and nightly — keep only stable across all 3 OSes (ubuntu, macos, windows).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: README.md MUST replace all references to sqlite-vec as the active vector backend with turbovec, including the features table, architecture overview, storage description, performance benchmarks table, and comparison table.
- **FR-002**: Cargo.toml MUST include `rust-version = "1.75"` in its `[package]` section as the canonical MSRV. README.md MUST display `1.75+` consistently across the badge and any text references.
- **FR-003**: CONTRIBUTING.md MUST remove "SQLite: Local data storage" from the prerequisites section, reflecting the post-turbovec architecture where SQLite is no longer a required local dependency.
- **FR-004**: Architecture.md's Model Download Flow mermaid diagram MUST render without parse errors by renaming single-letter node IDs (L1, M1, N1, O1) in the "Shared Utilities" subgraph to multi-character IDs that do not collide with Mermaid reserved shape names.
- **FR-005**: test.yml workflow MUST replace deprecated `actions-rs/toolchain@v1` with `dtolnay/rust-toolchain@v1` for Rust toolchain installation.
- **FR-006**: test.yml workflow MUST replace deprecated `actions-rs/audit-check@v1` with direct `cargo audit` invocation (via `cargo install cargo-audit` with caching) for security auditing.
- **FR-007**: test.yml workflow MUST use cached `cargo-deny` installation (restore binary from cache keyed on lockfile hash; install via `cargo install --locked cargo-deny` only on cache miss) instead of reinstalling on every CI run.
- **FR-008**: build.yml workflow MUST replace deprecated `actions-rs/toolchain@v1` with `dtolnay/rust-toolchain@v1` and deprecated `actions-rs/cargo@v1` with direct `run: cargo` commands. The build matrix MUST be reduced to stable Rust only across the three OS targets (ubuntu, macos, windows), removing beta and nightly channels.
- **FR-009**: build.yml and release.yml workflows MUST upgrade `actions/cache@v3` to `actions/cache@v4`.
- **FR-010**: release.yml workflow MUST replace deprecated `actions-rs/toolchain@v1` with `dtolnay/rust-toolchain@v1` and deprecated `actions-rs/cargo@v1` with direct `run: cargo` commands.
- **FR-011**: Historical references to sqlite-vec in CHANGELOG.md and spec documents under `specs/` MUST be preserved — they document the migration history accurately. Only active/current documentation should be updated.
- **FR-012**: MSRV check in test.yml MUST derive the Rust version from `Cargo.toml`'s `rust-version` field rather than hardcoding a version string. The value resolved must be `1.75.0`.

### Key Entities

- **Project Documentation**: Root-level markdown files (README.md, CONTRIBUTING.md, Architecture.md) that serve as the public-facing documentation for users and contributors. These must reflect the current architecture, not the historical one.
- **CI Workflow Definition**: GitHub Actions YAML files (test.yml, build.yml, release.yml) that define the automated quality gates. These must use maintained, non-deprecated action versions.
- **Mermaid Diagram**: A visual diagram embedded in Architecture.md using the Mermaid.js syntax. Its node IDs must not collide with Mermaid's reserved single-letter shape identifiers.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Zero references to "sqlite-vec" or "SQLite/vec" as the active vector backend remain in README.md and CONTRIBUTING.md after the cleanup.
- **SC-002**: The Rust version badge and all Rust version references in README.md match the single source of truth in `Cargo.toml`'s `rust-version` field.
- **SC-003**: All Mermaid diagrams in Architecture.md render without parse errors when viewed in GitHub's markdown renderer or any Mermaid-compatible viewer.
- **SC-004**: All three GitHub Actions workflows pass with zero deprecation warnings in the Actions UI.
- **SC-005**: The `cargo-deny` license check step in test.yml completes in under 30 seconds on repeat CI runs (cached).
- **SC-006**: The MSRV check in test.yml derives its Rust version from `Cargo.toml` rather than a hardcoded string.

## Assumptions

- The `dtolnay/rust-toolchain@v1` action is the recommended replacement for the deprecated `actions-rs/toolchain@v1` and is widely adopted in the Rust ecosystem.
- Direct `run: cargo <command>` invocations are the preferred replacement for `actions-rs/cargo@v1`, eliminating the need for a third-party action wrapper around cargo commands.
- The `actions/cache@v4` is a drop-in upgrade from v3 with the same interface but improved performance and Node.js 20 runtime.
- `cargo-deny` should be installed via `cargo install --locked cargo-deny` with the binary path cached, rather than via a dedicated GitHub Action, to keep the dependency footprint minimal.
- Mermaid node IDs `L1`, `M1`, `N1`, `O1` collide with reserved shape identifiers because Mermaid interprets the leading single letter as a shape type. Renaming to multi-letter IDs (e.g., `LU1`, `MU1`, `NU1`, `OU1`) resolves the parse error.
- Rust version `1.75.0` is the canonical MSRV. A `rust-version = "1.75"` field will be added to `Cargo.toml` as the single source of truth, replacing the hardcoded `1.75.0` in test.yml and resolving the inconsistency between README (`1.70+`) and CONTRIBUTING.md (`1.75+`).
- Existing spec documents under `specs/001` through `specs/008` are historical snapshots that accurately capture the state at the time they were written and should not be modified.
- No new code modules are required — this feature is purely documentation and CI configuration changes.
