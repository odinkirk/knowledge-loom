# Feature Specification: Fix CI Test Vault Dependency

**Feature Branch**: `010-fix-ci-test-vault`  
**Created**: 2026-06-07  
**Status**: Draft  
**Input**: User description: "Let's start a new sprint to address the failing CI test."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - CI Tests Workflow Passes (Priority: P1)

A maintainer pushes changes or opens a pull request. The Tests workflow completes without failures — all tests pass including those that depend on the `test-vault/` corpus. Previously, `test_graph_edges_from_test_vault` failed because the test corpus was not present on CI runners.

**Why this priority**: This is the deferred technical debt (D01) from the previous feature. Without it, the Tests workflow reports a failure, masking real test regressions and violating the constitution's quality gate requirement that all tests must pass.

**Independent Test**: Push to a branch and observe the Tests workflow. The step `cargo test --all-features` completes with zero failures, including `test_graph_edges_from_test_vault`.

**Acceptance Scenarios**:

1. **Given** a PR is opened, **When** the test workflow runs on ubuntu-latest, **Then** `test_graph_edges_from_test_vault` passes and the overall test suite reports zero failures.
2. **Given** the Tests workflow runs, **When** any test that requires `test-vault/` corpus data executes, **Then** the test corpus is available and the test runs against real data.

---

### Edge Cases

- What if the `test-vault/` clone fails due to network issues on the CI runner? (The clone step should be the first test setup step; if it fails, the workflow fails fast with a clear error rather than a confusing test failure.)
- What if multiple test files depend on `test-vault/` beyond `test_graph_edges_from_test_vault`? (The corpus should be available for all tests; a single clone step serves all dependents.)
- What if the test-vault repository changes and breaks tests? (We pin to a known-good commit hash rather than cloning the default branch.)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The Tests workflow MUST make the `test-vault/` corpus available before running `cargo test`, so that `test_graph_edges_from_test_vault` and any other tests depending on it can execute successfully.
- **FR-002**: The corpus acquisition step MUST fail gracefully with a clear error message if the external clone is unavailable (e.g., network failure), rather than producing an obscure test failure.
- **FR-003**: The corpus MUST be cloned from a pinned commit hash to ensure deterministic test behavior across CI runs.

### Key Entities

- **Test Vault Corpus**: A directory of markdown files with wikilinks (`test-vault/`) sourced from `https://github.com/ashuotaku/Personal-Wiki`. Used by multiple test files for corpus-based integration testing. Currently absent from CI runners.
- **CI Test Step**: The `cargo test --all-features` invocation in `.github/workflows/test.yml` that must pass with zero failures per the constitution's quality gate.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The Tests workflow `cargo test --all-features` step completes with exit code 0 and zero test failures.
- **SC-002**: `test_graph_edges_from_test_vault` passes on every CI run with edge count > 0.
- **SC-003**: The corpus clone step completes in under 30 seconds (the `Personal-Wiki` repo is small — 65 markdown files).

## Assumptions

- The `ashuotaku/Personal-Wiki` repository is publicly available and will remain so. It is the documented test corpus in README.md.
- The clone step will use `actions/checkout` or `git clone` with `--depth 1` for efficiency.
- The corpus clone adds minimal CI time (the repo is ~2MB).
- No changes to the test code itself are required — the test is correct; only the corpus availability needs to be fixed.
- The pinned commit hash will be fixed at the time of implementation and should be a stable, known-good state of the test corpus.
