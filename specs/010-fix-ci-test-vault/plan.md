# Implementation Plan: Fix CI Test Vault Dependency

**Branch**: `010-fix-ci-test-vault` | **Date**: 2026-06-07 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/010-fix-ci-test-vault/spec.md`

## Summary

Address deferred technical debt D01 from sprint 009: the Tests workflow fails because `test_graph_edges_from_test_vault` requires the `test-vault/` corpus which is not present on CI runners. Fix: add a clone step in the Tests workflow to fetch the corpus before running tests.

## Technical Context

**Language/Version**: N/A (CI workflow YAML configuration change only)
**Primary Dependencies**: External git repository (`ashuotaku/Personal-Wiki`) — publicly available, documented in README.md as the canonical test corpus
**Storage**: N/A
**Testing**: CI dogfooding — the Tests workflow itself validates the fix
**Target Platform**: GitHub Actions ubuntu-latest runner
**Project Type**: CI configuration
**Performance Goals**: Clone step <30s (repo is ~2MB, 65 files); no impact on test execution time
**Constraints**: Pinned commit hash for determinism; graceful failure if clone is unavailable
**Scale/Scope**: One step added to test.yml; no source code changes

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Gate | Status | Notes |
|------|--------|-------|
| I. Rust-First Architecture | N/A | No Rust code changes |
| II. Modular Design | N/A | No module changes |
| III. Test-First Development | N/A | Fixing existing test infrastructure; test already exists, just needs CI environment |
| IV. Integration Testing | PASS | This IS an integration testing fix — making the test-vault corpus available for existing integration tests |
| V. Quality Gates | PASS | This fix is required for the "Testing: cargo test must pass" quality gate |
| VI. MCP Protocol Compliance | N/A | No MCP changes |
| VII. Performance Standards | N/A | No runtime impact |
| VIII. Documentation Requirements | N/A | No new public API or architecture changes |
| IX. Output Conventions | N/A | No Rust output affected |
| X. Technical Debt Policy | PASS | This IS the remediation of deferred debt D01 from sprint 009 |
| XI. Code Exploration (CRG) | N/A | No Rust code exploration needed |
| XII. Spec-Kit Workflow | PASS | Following spec → plan → tasks flow |

## Project Structure

### Documentation (this feature)

```text
specs/010-fix-ci-test-vault/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root) — changes

```text
.github/workflows/
└── test.yml             # MODIFIED — add git clone step for test-vault/ before cargo test
```

## Complexity Tracking

> No constitutional violations. No complexity to justify.

## Phase 0: Research

See [research.md](./research.md) for detailed findings.

Key decisions:
1. **Clone method**: Use `actions/checkout@v4` for the test-vault repo as a separate checkout step; more reliable than raw `git clone` and handles auth/retries
2. **Directory location**: Clone to `test-vault/` at repo root (where the test code expects it via `CARGO_MANIFEST_DIR`)
3. **Commit pinning**: Use `ref: <commit-hash>` to pin to a known-good state of the Personal-Wiki repo for deterministic test results
4. **Shallow clone**: `fetch-depth: 1` for speed (~2MB repo, no history needed)

## Phase 1: Design

See [data-model.md](./data-model.md) for entity descriptions.
See [quickstart.md](./quickstart.md) for verification steps.

Interfaces: No code interfaces to contract. A single CI configuration change.
