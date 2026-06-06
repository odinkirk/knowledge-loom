# Data Model: Documentation Cleanup and CI Modernization

This feature operates on static files, not a database. The "entities" below describe the files modified and their key attributes relevant to the changes.

## Entity: Project Documentation File

Root-level markdown files that serve as public-facing project documentation.

| Attribute | Description |
|-----------|-------------|
| `path` | File path relative to repo root |
| `target_references` | Text patterns to replace or remove |
| `preserved_references` | Text patterns that must NOT be modified (historical context) |

### Files and Their Changes

**README.md**:
- Replace `sqlite-vec` / `SQLite/vec` → `turbovec` in features table, architecture overview, storage description, performance benchmarks, comparison table
- Update Rust version badge from `1.70+` → `1.75+`
- Update Credits section: mark sqlite-vec as legacy (replaced by turbovec), not removed
- Keep `sqlite-vec` references in context that documents the migration history

**CONTRIBUTING.md**:
- Remove line 10: `- **SQLite**: Local data storage.`
- Remove line 10's context about SQLite being a prerequisite

**Architecture.md**:
- Lines 315-320: Rename Mermaid node IDs `L1`→`DL1`, `M1`→`DM1`, `N1`→`DN1`, `O1`→`DO1` in the "Shared Utilities" subgraph

**Cargo.toml**:
- Add `rust-version = "1.75"` to `[package]` section

## Entity: CI Workflow Definition

GitHub Actions YAML files defining automated quality gates.

| Attribute | Description |
|-----------|-------------|
| `path` | `.github/workflows/<name>.yml` |
| `deprecated_actions` | Actions needing replacement |
| `caching_changes` | Cache steps to add or upgrade |

### Files and Their Changes

**test.yml**:
- Line 22: Replace `actions-rs/toolchain@v1` → `dtolnay/rust-toolchain@v1` with `toolchain: stable`
- Line 38: Replace `actions-rs/audit-check@v1` → direct `run: cargo install cargo-audit && cargo audit`
- Lines 44-46: Replace `cargo install cargo-deny` + `cargo deny check` with cache-restore pattern
- Line 47-49: Replace hardcoded `1.75.0` → parsed `rust-version` from Cargo.toml

**build.yml**:
- Line 29: Replace `actions-rs/toolchain@v1` → `dtolnay/rust-toolchain@v1`
- Lines 54,60: Replace `actions-rs/cargo@v1` → direct `run: cargo build`
- Lines 36,42,48: Upgrade `actions/cache@v3` → `actions/cache@v4`
- Lines 18-22: Reduce matrix to `os: [ubuntu-latest, macos-latest, windows-latest]`, `rust: [stable]`
- Remove exclude block for windows+nightly

**release.yml**:
- Line 21: Replace `actions-rs/toolchain@v1` → `dtolnay/rust-toolchain@v1`
- Lines 46,53,60: Replace `actions-rs/cargo@v1` → direct `run: cargo <command>`
- Lines 28,34,40: Upgrade `actions/cache@v3` → `actions/cache@v4`

## Entity: Mermaid Diagram Node

Individual nodes within the Model Download Flow mermaid diagram in Architecture.md.

| Attribute | Description |
|-----------|-------------|
| `old_id` | Current node identifier (e.g., `L1`) |
| `new_id` | Replacement identifier (e.g., `DL1`) |
| `label` | Display text (unchanged) |
| `subgraph` | Containing subgraph ("Shared Utilities") |

### Rename Map

| Old ID | New ID | Label |
|--------|--------|-------|
| `L1` | `DL1` | `download/utils.rs` |
| `M1` | `DM1` | `calculate_checksum()` |
| `N1` | `DN1` | `validate_checksum()` |
| `O1` | `DO1` | `check_disk_space()` |
