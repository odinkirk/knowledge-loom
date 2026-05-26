# Quickstart: Integrate turbovec

## Prerequisites

- Rust 1.70+
- Working knowledge-loom dev environment (`cargo build` succeeds)
- `test-vault/` cloned for corpus testing

## 1. Add turbovec dependency

```toml
# Cargo.toml
[dependencies]
turbovec = "0.6"
```

## 2. Remove sqlite-vec dependencies

Remove from `Cargo.toml`:
- `rusqlite = { version = "0.31", features = ["bundled"] }`
- `sqlite-vec = "0.1"`

## 3. Create the new module

```bash
touch src/turbovec_index.rs
```

Implement `TurbovecIndex` per [contracts/turbovec_index.md](./contracts/turbovec_index.md).

## 4. Update module declarations

```rust
// src/lib.rs
pub mod turbovec_index;  // replaces pub mod index;
```

## 5. Update SearchEngine

Replace `VectorIndex` → `TurbovecIndex` in `src/search.rs`:
- Constructor types
- `search()` method — swap `self.vector` call
- `search_graph_fused_inner()` — add allowlist support

## 6. Update server/daemon/maintenance

Replace `VectorIndex` references with `TurbovecIndex` in:
- `src/server.rs` — `LoomServer` fields
- `src/daemon.rs` — reindex triggers
- `src/maintenance.rs` — index health checks + migration

## 7. Add migration logic

On startup, check for `.knowledge-loom-index/embeddings.db`:
- If exists: read all rows, compute chunk IDs, call `add_with_ids`, verify count, delete `.db`
- If absent: normal startup

## 8. Write tests

```bash
touch tests/turbovec_index_tests.rs
```

Test coverage:
- `test_new_empty_index` — create, verify count=0
- `test_add_and_search` — add vectors, search, verify similarity scores
- `test_add_and_remove` — add, remove by ID, verify count
- `test_allowlist_search` — add vectors, search with allowlist, verify only allowed results
- `test_persistence` — add vectors, save, reload, verify search still works
- `test_migration` — create sqlite-vec db, run migration, verify turbovec parity
- `test_concurrent_search_and_index` — spawn search + add tasks, verify no panics
- `test_dimension_mismatch` — add wrong-dim vectors, verify error
- `test_corrupt_index` — load from garbage file, verify fresh index fallback

## 9. Remove old files

```bash
rm src/index.rs
rm tests/vector_tests.rs  # or rewrite for turbovec
```

## 10. Verify quality gates

```bash
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo test --all-features
cargo deny check licenses bans sources
```

## Key reference

- turbovec docs: https://github.com/RyanCodrai/turbovec/blob/main/docs/api.md
- turbovec crate: https://crates.io/crates/turbovec
