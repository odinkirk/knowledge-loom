# Research: Fix CI Test Vault Dependency

## Decision 1: Clone Method

**Decision**: Use `actions/checkout@v4` with a separate repository path to clone `test-vault/` into the workspace.

**Rationale**: GitHub Actions' `actions/checkout` is purpose-built for git operations in CI, handles authentication, retries, and spurious network failures better than raw `git clone`. Using it for the test-vault repo alongside the primary checkout is a standard pattern.

**Alternatives considered**:
- Raw `git clone` in a `run:` step — simpler but no retry logic, no auth handling for private repos
- GitHub Actions `checkout` with multiple repos in one step — not supported by the action
- Pre-bundling test-vault as a git submodule — adds maintenance burden for a CI-only concern

## Decision 2: Directory Location

**Decision**: Clone to `test-vault/` directly in the workspace root (`${{ github.workspace }}/test-vault`).

**Rationale**: The test code at `tests/graph_tests.rs:60` resolves `test-vault` relative to `CARGO_MANIFEST_DIR`:
```rust
let kb_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-vault");
```
The `actions/checkout@v4` action with `path: test-vault` places the clone at exactly this location.

## Decision 3: Commit Pinning

**Decision**: Pin to a specific commit hash rather than cloning `main`.

**Rationale**: Deterministic test results require a stable corpus. If the source repository changes content or structure, tests could break unpredictably. A pinned hash ensures the same corpus on every CI run. The hash should be updated deliberately when the project wants to refresh test data.

**Alternatives considered**:
- Clone `main` — non-deterministic, tests could break silently
- Clone a tag — requires upstream maintenance of tags
- Bundle test-vault in the repo — bloats the repository, copyright concerns

## Decision 4: Shallow Clone

**Decision**: Use `fetch-depth: 1` for a shallow clone.

**Rationale**: The Personal-Wiki repository is ~2MB. A shallow clone is fast enough for CI (<10s) and avoids unnecessary history. Full history is not needed for test data.

**Alternatives considered**:
- Full clone — slower, unnecessary history

## Decision 5: Failure Handling

**Decision**: Let the checkout step fail naturally if the clone is unavailable.

**Rationale**: The `actions/checkout@v4` action fails with a clear error if the repository is unreachable. This provides the "fail fast with clear error" behavior specified in FR-002 without additional scripting. The error message will identify the failed repository URL, making diagnosis straightforward.

**Alternatives considered**:
- `continue-on-error: true` with conditional test skip — hides infrastructure problems, tests silently not run
- Custom error handling script — unnecessary complexity for a standard git operation
