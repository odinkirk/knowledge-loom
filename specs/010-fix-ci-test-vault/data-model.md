# Data Model: Fix CI Test Vault Dependency

This feature adds one CI configuration entry. The "entities" describe the components involved.

## Entity: Test Vault Corpus

External git repository containing markdown files with wikilinks, used as test data.

| Attribute | Description |
|-----------|-------------|
| `source` | `https://github.com/ashuotaku/Personal-Wiki` |
| `pinned_commit` | Fixed commit hash for deterministic tests |
| `target_path` | `test-vault/` in workspace root |
| `clone_depth` | 1 (shallow clone, no history) |
| `approximate_size` | ~2MB, 65 markdown files |

## Entity: CI Clone Step

A new step in `.github/workflows/test.yml` that executes before `cargo test`.

| Attribute | Description |
|-----------|-------------|
| `action` | `actions/checkout@v4` |
| `repository` | `ashuotaku/Personal-Wiki` |
| `path` | `test-vault` |
| `ref` | Pinned commit hash |
| `fetch-depth` | `1` |
| `placement` | Before the `Run tests` step, after the `Install Rust toolchain` step |

### Test Dependencies

Tests that depend on `test-vault/` being present:

| Test | File | Line |
|------|------|------|
| `test_graph_edges_from_test_vault` | `tests/graph_tests.rs` | 58 |

The clone step serves all current and future tests that reference `test-vault/`.
