# Implementation Plan: Documentation Cleanup and CI Modernization

**Branch**: `009-docs-ci-cleanup` | **Date**: 2026-06-06 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/009-docs-ci-cleanup/spec.md`

## Summary

Clean up stale sqlite-vec references in root-level documentation (README.md, CONTRIBUTING.md) left over from the turbovec migration in spec 008, fix a Mermaid parse error in Architecture.md's Model Download Flow diagram, and modernize all three GitHub Actions workflows by replacing deprecated action versions (`actions-rs/*`, `actions/cache@v3`) and adding `cargo-deny` caching. No Rust source code changes.

## Technical Context

**Language/Version**: Rust 1.75 (canonical MSRV; no source code changes in this feature)
**Primary Dependencies**: N/A (no code dependencies affected)
**Storage**: N/A
**Testing**: Manual visual inspection (docs), CI dogfooding (workflows validated in their own pipeline runs)
**Target Platform**: GitHub Actions runners (ubuntu-latest, macos-latest, windows-latest) + any Mermaid-compatible renderer
**Project Type**: Documentation + CI configuration
**Performance Goals**: CI `cargo-deny` step <30s (cached), zero deprecation warnings in Actions logs
**Constraints**: No Rust source code changes; historical doc references preserved (FR-011)
**Scale/Scope**: 3 markdown files, 1 Cargo.toml edit, 3 workflow YAML files

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Gate | Status | Notes |
|------|--------|-------|
| I. Rust-First Architecture | N/A | No Rust code changes |
| II. Modular Design | N/A | No module changes |
| III. Test-First Development | N/A | No code to test; CI dogfooding serves as validation |
| IV. Integration Testing | N/A | No integration surfaces changed |
| V. Quality Gates | PASS | CI workflows being modernized ARE the quality gates; they must pass as part of this feature |
| VI. MCP Protocol Compliance | N/A | No MCP changes |
| VII. Performance Standards | N/A | No runtime performance impact |
| VIII. Documentation Requirements | PASS | Primary deliverable — documentation corrections and Architecture.md fix |
| IX. Output Conventions | N/A | No Rust output affected |
| X. Technical Debt Policy | PASS | This feature IS technical debt remediation — no deferral needed |
| XI. Code Exploration (CRG) | N/A | No Rust code exploration needed |
| XII. Spec-Kit Workflow | PASS | Following spec → plan → tasks flow |

## Project Structure

### Documentation (this feature)

```text
specs/009-docs-ci-cleanup/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (minimal — no code interfaces)
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root) — changes

```text
Cargo.toml               # MODIFIED — add rust-version = "1.75"

README.md                # MODIFIED — sqlite-vec → turbovec references, MSRV badge
CONTRIBUTING.md          # MODIFIED — remove SQLite prerequisite
Architecture.md          # MODIFIED — fix Mermaid node IDs in Model Download Flow

.github/workflows/
├── test.yml             # MODIFIED — dtolnay action, cargo-deny caching, MSRV from Cargo.toml
├── build.yml            # MODIFIED — dtolnay action, direct cargo, cache@v4, stable-only matrix
└── release.yml          # MODIFIED — dtolnay action, direct cargo, cache@v4
```

## Complexity Tracking

> No constitutional violations. No complexity to justify.

## Phase 0: Research

See [research.md](./research.md) for detailed findings.

Key decisions:
1. **Rust toolchain action**: `dtolnay/rust-toolchain@v1` is the de facto standard replacement for the deprecated `actions-rs/toolchain@v1`
2. **Cargo invocation**: Direct `run: cargo <command>` replaces `actions-rs/cargo@v1` — no third-party wrapper needed
3. **Cache upgrade**: `actions/cache@v4` is a drop-in replacement for v3, running on Node.js 20
4. **Mermaid node ID fix**: Single-letter IDs `L1`, `M1`, `N1`, `O1` collide with Mermaid reserved shape names; rename to `DL1`, `DM1`, `DN1`, `DO1`
5. **cargo-deny caching**: Cache `~/.cargo/bin/cargo-deny` with a key based on lockfile hash; install if cache miss
6. **MSRV derivation**: Parse `rust-version` from Cargo.toml using `cargo metadata` or `tomlq` in CI step

## Phase 1: Design

See [data-model.md](./data-model.md) for entity descriptions.
See [quickstart.md](./quickstart.md) for verification steps.

Interfaces: No code interfaces to contract. Changes are purely markdown text edits and YAML configuration updates with no new public APIs, CLI commands, or MCP tools.
