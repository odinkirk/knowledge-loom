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
