# Research: Integrate turbovec

## Decision 1: Index Type — IdMapIndex

**Decision**: Use `turbovec::IdMapIndex` over `turbovec::TurboQuantIndex`.

**Rationale**: The spec requires stable IDs for allowlist filtering (FR-005) and individual vector removal (FR-004). `TurboQuantIndex` uses positional slots that are invalidated by `swap_remove`. `IdMapIndex` provides stable external `u64` IDs with O(1) remove-by-ID, directly mapping to knowledge-loom's chunk identifiers. Framework integrations (LangChain, LlamaIndex) use `IdMapIndex` for the same reason.

**Alternatives considered**:
- `TurboQuantIndex` + manual ID mapping in caller — adds complexity for no benefit; the mapping IS `IdMapIndex`
- Keeping sqlite-vec alongside turbovec — rejected by spec assumption (sqlite-vec removed after migration)

## Decision 2: Chunk ID Scheme

**Decision**: Use `fnv64a` hash of `"{path}\0{heading}\0{chunk_ordinal}"` to generate deterministic uint64 IDs.

**Rationale**: Spec requires deterministic IDs across reindex runs (Assumptions). FNV-1a 64-bit is fast, collision-resistant for vault-scale collections, and produces integers directly usable as turbovec IDs. The separator `\0` prevents ambiguity between e.g. `("a/b", "c")` and `("a", "b/c")`.

**Alternatives considered**:
- Auto-incrementing IDs — non-deterministic across reindex runs, breaks when files are added/removed
- UUID v5 — too wide for uint64
- Using turbovec's positional IDs — breaks with incremental updates

## Decision 3: Quantization — 4-bit default

**Decision**: Default to 4-bit quantization (`bit_width=4`), configurable per FR-002.

**Rationale**: Per turbovec benchmarks, 4-bit TurboQuant at d=1536 achieves recall@1 of ~0.97 (beating FAISS PQ by 0.4-3.4 points) and 8x compression vs float32. At knowledge-loom's typical 384-dim (BGESmallENV15), 4-bit provides a good balance. 2-bit gives 16x compression but incurs a larger recall hit (1.2 points below FAISS on GloVe d=200).

**Alternatives considered**:
- 2-bit default — too aggressive for 384-dim where the asymptotic Beta assumption is looser
- Raw float32 — no memory savings, defeats the purpose
- 8-bit — turbovec only supports 2 and 4

## Decision 4: Migration Strategy

**Decision**: On first startup after upgrade, detect existing `embeddings.db`, read all rows, call `add_with_ids` on the new turbovec index, verify count matches, then delete `embeddings.db`.

**Rationale**: Spec FR-011 mandates automatic one-time migration. Reading from sqlite-vec and writing to turbovec is a straightforward ETL step. The migration is idempotent — if `embeddings.db` is already deleted (or never existed), no-op.

**Alternatives considered**:
- Force full reindex — requires regenerating embeddings, which is slow and may use API credits (Ollama/OpenRouter)
- Run both backends in parallel — adds maintenance burden, spec says sqlite-vec removed

## Decision 5: Concurrency Model

**Decision**: Wrap `IdMapIndex` in `Arc<Mutex<IdMapIndex>>`. Search takes a read lock briefly; mutation takes a write lock. turbovec search is inherently read-only on the index data and can be called concurrently from multiple threads with a shared reference.

**Rationale**: Spec FR-012 requires concurrent search+index without data corruption. The `Arc<Mutex<>>` pattern matches the existing codebase conventions (`BM25Index`, `VectorIndex`, `GraphState`). turbovec's `search` method borrows `&self` so shared read access is safe. Write operations (`add_with_ids`, `remove`) require exclusive access.

**Alternatives considered**:
- `RwLock` — not needed; turbovec search is fast (<1ms for k=10 on 10k vectors) so lock contention is minimal
- Channel-based serialization — overengineered for local single-process tool

## Decision 6: Metadata Storage

**Decision**: Maintain a separate `HashMap<u64, ChunkMetadata>` alongside the `IdMapIndex` for mapping turbovec IDs back to file paths, headings, content, and line numbers.

**Rationale**: `IdMapIndex` stores only `(id, compressed_vector)` — no metadata. The search result from turbovec returns `(scores, ids)`. We need to map IDs back to `(path, heading, content, line_start, line_end)` to produce `SearchResult` structs. A `HashMap<u64, ChunkMetadata>` provides O(1) lookups.

**Alternatives considered**:
- Store metadata in SQLite — adds sqlite-vec dependency we're removing; Tantivy already stores chunk content for BM25
- Store metadata in Tantivy — feasible but couples vector index to BM25 storage; cleaner to keep independent

## Decision 7: Persistence Format

**Decision**: Use turbovec's native `.tvim` format (`IdMapIndex::write`/`load`) stored at `.knowledge-loom-index/turbovec.tvim`. Metadata hashmap serialized alongside as `.knowledge-loom-index/turbovec_meta.bin` via bincode.

**Rationale**: `.tvim` is turbovec's stable format with version byte for forward compatibility. `bincode` is already used for graph serialization. Separating metadata from the vector index avoids coupling and allows metadata to change independently of the vector format.

**Alternatives considered**:
- Custom format — reinvents the wheel; `.tvim` is well-specified
- Metadata inside `.tvim` — the format doesn't support arbitrary metadata

## Decision 8: Rusqlite Retention

**Decision**: Keep `rusqlite` dependency if any other module uses SQLite directly (e.g., init, maintenance state tracking). Verify before removing.

**Rationale**: `Cargo.toml` currently lists `rusqlite` with `bundled` feature. If no other module besides `src/index.rs` uses SQLite, we can remove both `rusqlite` and `sqlite-vec`. The maintenance state tracker (`reindex-state.json`) uses JSON, not SQLite.

**Check performed**: Grep for `rusqlite::` outside `src/index.rs`. If none found, remove the dependency.
