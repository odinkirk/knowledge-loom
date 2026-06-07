# Research: Documentation Cleanup and CI Modernization

## Decision 1: Rust Toolchain Action Replacement

**Decision**: Replace `actions-rs/toolchain@v1` with `dtolnay/rust-toolchain@v1`.

**Rationale**: `actions-rs` is archived and unmaintained since 2023. `dtolnay/rust-toolchain@v1` is maintained by a core Rust team member and is the most widely adopted replacement across the Rust ecosystem (>1M monthly downloads on GitHub Marketplace). It supports specifying `toolchain: stable` and respects `rust-version` from Cargo.toml automatically when no explicit toolchain is given.

**Alternatives considered**:
- `rust-toolchain.toml` file (via `rustup show`) — adds a file, no caching advantage
- `actions-rs/toolchain@v1` (keep) — already deprecated, generating warnings
- Direct `rustup` commands — works but `dtolnay/rust-toolchain` handles caching and cross-platform edge cases

## Decision 2: Cargo Action Replacement

**Decision**: Replace `actions-rs/cargo@v1` with direct `run: cargo <command>`.

**Rationale**: `actions-rs/cargo@v1` is also archived. There is no direct successor because the action was always a thin wrapper around `cargo`. Direct `run:` invocations eliminate a dependency, reduce attack surface, and are clearer to read.

**Alternatives considered**:
- `clechasseur/rs-cargo@v1` — unnecessary indirection; direct `run:` is simpler
- `houseabsolute/actions-rust-cross@v0` — overkill for standard builds

## Decision 3: Cache Action Upgrade

**Decision**: Upgrade `actions/cache@v3` → `actions/cache@v4`.

**Rationale**: v3 runs on Node.js 16 which is deprecated by GitHub. v4 uses Node.js 20 and has improved performance and reliability. The interface is identical — a drop-in version bump.

**Alternatives considered**:
- `actions/cache@v3` (keep) — generates deprecation warnings, will stop working when GitHub sunsets Node.js 16
- `actions/cache/save` + `actions/cache/restore` (v4 split API) — only needed for advanced scenarios

## Decision 4: Mermaid Node ID Fix

**Decision**: Rename `L1`, `M1`, `N1`, `O1` to `DL1`, `DM1`, `DN1`, `DO1` in the "Shared Utilities" subgraph of the Model Download Flow diagram.

**Rationale**: Mermaid interprets single-letter node IDs as reserved shape names (e.g., `L` = circle, `M` = cylinder, `N` = stadium, `O` = circle). When followed by a digit like `L1[...]`, the parser misinterprets the token. A multi-character prefix prevents the collision. The `D` prefix stands for "download utility" which matches the subgraph context.

**Alternatives considered**:
- `LU1`, `MU1`, `NU1`, `OU1` — also works but less semantically meaningful
- Full names like `checksum`, `validate_checksum`, `check_disk` — more verbose, diverge from existing diagram style
- Remove the subgraph entirely — loses diagram context

## Decision 5: cargo-deny Caching Strategy

**Decision**: Cache `~/.cargo/bin/cargo-deny` using `actions/cache@v4` with a key derived from `Cargo.lock` hash. Install via `cargo install --locked cargo-deny` on cache miss.

**Rationale**: `cargo install cargo-deny` takes ~45s on cold starts. The `EmbarkStudios/cargo-deny-action` exists but adds a dependency and doesn't reduce total time meaningfully. A simple cache-restore + conditional install pattern is lighter and more maintainable.

**Alternatives considered**:
- `EmbarkStudios/cargo-deny-action@v1` — adds another action dependency, similar speed
- `cargo-binstall` — faster install but requires an additional tool
- No caching (current behavior) — wastes CI time on every run

## Decision 6: MSRV Derivation from Cargo.toml

**Decision**: Parse `rust-version` from `Cargo.toml` using `grep` + `sed` or `cargo metadata --format-version 1 | jq '.packages[0].rust_version'` in the CI step.

**Rationale**: Eliminates the hardcoded `1.75.0` in test.yml. When the MSRV changes only Cargo.toml needs updating, removing a silent inconsistency risk. The `cargo metadata` approach is portable across OSes.

**Alternatives considered**:
- Hardcoded version (current) — works but is a drift risk
- `tomlq` from `cargo install toml-cli` — adds a dependency, unnecessary for a single field read
- `yq` + `toml2yaml` — overly complex for this use case

## Decision 7: Build Matrix Simplification

**Decision**: Remove `beta` and `nightly` from the build matrix, keeping only `stable` across `ubuntu-latest`, `macos-latest`, `windows-latest`.

**Rationale**: For a library/CLI tool like Knowledge Loom, beta/nightly builds add marginal value (catching Rust regressions that almost never affect stable users) while doubling CI cost and feedback time (8 jobs → 3 jobs). The test workflow on stable already covers correctness. If the project later needs nightly features, the matrix can be expanded.

**Alternatives considered**:
- Keep full matrix (stable + beta + nightly) — safer but expensive and noisy
- Keep only stable on ubuntu — cheap but misses platform-specific build failures
