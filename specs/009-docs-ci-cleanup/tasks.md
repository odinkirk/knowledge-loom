# Tasks: Documentation Cleanup and CI Modernization

**Input**: Design documents from `/specs/009-docs-ci-cleanup/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Not required — this feature is pure documentation and CI YAML changes with no Rust source code modifications. Validation is manual visual inspection and CI dogfooding per quickstart.md.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Documentation**: `README.md`, `CONTRIBUTING.md`, `Architecture.md`, `CHANGELOG.md` at repo root
- **Configuration**: `Cargo.toml` at repo root
- **CI Workflows**: `.github/workflows/test.yml`, `.github/workflows/build.yml`, `.github/workflows/release.yml`

---

## Phase 1: Setup (Baseline Verification)

**Purpose**: Confirm the project is in a clean state before making changes

- [x] T001 Verify existing tests pass: `cargo test --all-features`
- [x] T002 [P] Verify existing formatting: `cargo fmt --all -- --check`
- [x] T003 [P] Verify existing linting: `cargo clippy -- -D warnings`

**Checkpoint**: All quality gates pass on current code — any failures here are pre-existing and must be addressed before proceeding.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add the canonical MSRV field to Cargo.toml — both US1 (badge) and US3 (CI MSRV check) depend on this

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 Add `rust-version = "1.75"` to the `[package]` section of `Cargo.toml` (after line 5, before `[[bin]]`)

**Checkpoint**: `grep rust-version Cargo.toml` returns `rust-version = "1.75"`. Foundation ready — user story implementation can now begin.

---

## Phase 3: User Story 1 - Accurate Project Documentation (Priority: P1) 🎯 MVP

**Goal**: All active documentation (README.md, CONTRIBUTING.md) accurately describes turbovec as the vector backend and shows consistent Rust version requirements

**Independent Test**: `grep -n "sqlite-vec\|SQLite/vec" README.md CONTRIBUTING.md` returns zero stale references. `grep "1.75" README.md` shows consistent version.

### Implementation for User Story 1

- [x] T005 [US1] Update all sqlite-vec → turbovec references in `README.md`: architecture section (line 24, 39-40), features table (line 50), storage description (line 72), benchmarks (line 325), comparison table (line 513), credits (line 645)
- [x] T006 [P] [US1] Update Rust version badge in `README.md` line 11 from `1.70%2B` to `1.75%2B` and any inline version text to `1.75+`
- [x] T007 [P] [US1] Remove SQLite prerequisite from `CONTRIBUTING.md` line 10 (`- **SQLite**: Local data storage.`) and remove its surrounding context about SQLite as a dependency
- [x] T008 [US1] Verify US1: `grep -n "sqlite-vec\|SQLite/vec" README.md CONTRIBUTING.md` — only matches in historical context (Credits section mentioning migration) should remain

**Checkpoint**: README.md and CONTRIBUTING.md accurately describe turbovec as the active backend and show `1.75+` Rust version consistently.

---

## Phase 4: User Story 2 - Error-Free Mermaid Diagrams (Priority: P2)

**Goal**: Architecture.md's Model Download Flow diagram renders without parse errors in any Mermaid-compatible renderer

**Independent Test**: Render Architecture.md in a Mermaid viewer — Model Download Flow diagram renders without parse error

### Implementation for User Story 2

- [x] T009 [US2] Rename Mermaid node IDs in `Architecture.md` lines 315-320 ("Shared Utilities" subgraph): `L1`→`DL1`, `M1`→`DM1`, `N1`→`DN1`, `O1`→`DO1`
- [x] T010 [US2] Verify US2: render Architecture.md in a Mermaid-compatible viewer — all diagrams including Model Download Flow render without errors, no regressions in other diagrams

**Checkpoint**: Architecture.md diagrams render clean. Both US1 and US2 can be verified independently.

---

## Phase 5: User Story 3 - Reliable CI Pipeline (Priority: P3)

**Goal**: All three GitHub Actions workflows use maintained action versions with zero deprecation warnings; cargo-deny uses caching for faster repeat runs

**Independent Test**: Push to branch and observe CI logs — zero deprecation warnings; cargo-deny step <10s on cache hit

### Implementation for User Story 3

- [x] T011 [P] [US3] Modernize `.github/workflows/test.yml`:
  - Replace `actions-rs/toolchain@v1` with `dtolnay/rust-toolchain@v1` (toolchain: stable)
  - Replace `actions-rs/audit-check@v1` with direct `run: cargo install cargo-audit && cargo audit`
  - Add `actions/cache@v4` step for `cargo-deny` binary (key: `${{ runner.os }}-cargo-deny-${{ hashFiles('Cargo.lock') }}`, restore `~/.cargo/bin/cargo-deny`, install on cache miss)
  - Replace manual `cargo install cargo-deny` / `cargo install cargo-audit` with conditional install (only if binary not cached)
  - Replace hardcoded MSRV `1.75.0` with parsed value from `Cargo.toml` via `grep rust-version Cargo.toml | sed 's/.*"\(.*\)".*/\1/'` or `cargo metadata`

- [x] T012 [P] [US3] Modernize `.github/workflows/build.yml`:
  - Replace `actions-rs/toolchain@v1` with `dtolnay/rust-toolchain@v1`
  - Replace `actions-rs/cargo@v1` with direct `run: cargo build` / `run: cargo build --release`
  - Upgrade `actions/cache@v3` → `actions/cache@v4` (all 3 cache steps)
  - Reduce matrix to `rust: [stable]` only, remove `beta` and `nightly` from the rust list
  - Remove the `exclude` block for windows+nightly
  - Keep `os: [ubuntu-latest, macos-latest, windows-latest]`

- [x] T013 [P] [US3] Modernize `.github/workflows/release.yml`:
  - Replace `actions-rs/toolchain@v1` with `dtolnay/rust-toolchain@v1`
  - Replace `actions-rs/cargo@v1` with direct `run: cargo test` / `run: cargo build --release` / `run: cargo publish`
  - Upgrade `actions/cache@v3` → `actions/cache@v4` (all 3 cache steps and `softprops/action-gh-release@v1` is already current — no change needed)

- [x] T014 [US3] Verify US3: validate YAML syntax (`yamllint .github/workflows/*.yml`), validate action references (`actionlint .github/workflows/*.yml`), confirm no `actions-rs` references remain (`grep -rn actions-rs .github/workflows/`)

**Checkpoint**: All three workflows use maintained actions. Ready to dogfood on CI push.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final verification, changelog update, and CI dogfooding

- [x] T015 [P] Run quickstart.md verification checklist: grep checks for stale references, YAML validation, MSRV consistency
- [x] T016 Update `CHANGELOG.md` under `## [Unreleased]` → `### Fixed` with summary of documentation corrections (sqlite-vec→turbovec references, MSRV consistency, Mermaid diagram fix) and CI modernization (deprecated action replacements, cargo-deny caching, build matrix simplification)
- [ ] T017 Push to remote and verify CI dogfooding: all three workflows pass with zero deprecation warnings in Actions UI; cargo-deny step <30s on second run (cached)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational (needs `rust-version` in Cargo.toml)
- **User Story 2 (Phase 4)**: Depends on Foundational — can proceed independently of US1
- **User Story 3 (Phase 5)**: Depends on Foundational (needs `rust-version` in Cargo.toml)
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational — no dependencies on US2 or US3
- **User Story 2 (P2)**: Can start after Foundational — completely independent (different file)
- **User Story 3 (P3)**: Can start after Foundational — completely independent (different files)

### Within Each User Story

- All tasks within a story are sequential (verify step at end)
- US3 tasks T011, T012, T013 are in different files → all marked [P], can run in parallel

### Parallel Opportunities

- Phase 1: T002 and T003 can run in parallel (different cargo commands)
- Phase 3: T006 and T007 can run in parallel (different files: README.md, CONTRIBUTING.md)
- Phase 5: T011, T012, T013 can all run in parallel (different workflow files)
- Phase 6: T015 can run in parallel with T016
- Once Foundational completes, US1, US2, and US3 can proceed in parallel

---

## Parallel Example: User Story 3

```bash
# All three workflow files can be modified simultaneously:
Task: "Modernize .github/workflows/test.yml"
Task: "Modernize .github/workflows/build.yml"  
Task: "Modernize .github/workflows/release.yml"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup — verify baseline quality gates pass
2. Complete Phase 2: Foundational — add `rust-version` to Cargo.toml
3. Complete Phase 3: User Story 1 — README.md and CONTRIBUTING.md cleanup
4. **STOP and VALIDATE**: Run quickstart.md grep checks; confirm no stale references
5. Commit and push — documentation alone is a meaningful improvement

### Incremental Delivery

1. Setup + Foundational → MSRV canonicalized
2. Add US1 (docs) → verify → commit (MVP — accurate documentation!)
3. Add US2 (mermaid fix) → verify → commit (clean architecture diagrams)
4. Add US3 (CI) → verify → commit → push → CI dogfoods itself
5. Polish → CHANGELOG update → final push

### Quality Gates (Must Pass Before Merge)

- **Formatting**: `cargo fmt --all -- --check` (no code changes expected)
- **Linting**: `cargo clippy -- -D warnings` (no code changes expected)
- **Testing**: `cargo test --all-features` (no code changes expected)
- **CI**: All three GitHub Actions workflows must pass with zero deprecation warnings — this IS the feature's validation

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story is independently completable and testable
- No Rust source code changes — `cargo test/fmt/clippy` should remain green throughout
- **EXPLICIT CONSENT REQUIRED**: Each git commit requires individual user consent per constitution
- Use `grep` and `yamllint` / `actionlint` for validation (not `cargo` for this feature)
- The CI workflows being modified ARE the quality gates — they validate themselves
