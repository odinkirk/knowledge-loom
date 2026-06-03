# Data Model: Integrate turbovec

## Entities

### TurbovecIndex

Replaces `VectorIndex` (`src/index.rs`). Wraps `turbovec::IdMapIndex` plus metadata storage.

| Field | Type | Description |
|-------|------|-------------|
| `index` | `Arc<Mutex<IdMapIndex>>` | The turbovec vector index (compressed vectors + ID map) |
| `metadata` | `Arc<Mutex<HashMap<u64, ChunkMetadata>>>` | Maps turbovec ID → chunk metadata for search result assembly |
| `kb_root` | `PathBuf` | Root of the knowledge vault |
| `index_path` | `PathBuf` | File path for `.tvim` persistence |
| `meta_path` | `PathBuf` | File path for metadata persistence (`turbovec_meta.bin`) |
| `bit_width` | `u8` | Quantization level (2 or 4, default 4) |
| `dim` | `usize` | Embedding dimension (locked on first add) |

### ChunkMetadata

Stored in `HashMap<u64, ChunkMetadata>` alongside the turbovec index.

| Field | Type | Description |
|-------|------|-------------|
| `path` | `String` | Relative file path from kb_root (includes `.md` suffix) |
| `heading` | `Option<String>` | Section heading if chunk is under a heading |
| `content` | `String` | Chunk text content (same as stored in BM25/Tantivy) |
| `line_start` | `usize` | 1-indexed line number where chunk starts |
| `line_end` | `usize` | 1-indexed line number where chunk ends |
| `chunk_ordinal` | `u64` | Position of chunk within the file (0-based) |

### ID Mapping

Chunk identifiers are derived from the triple `(path, heading, chunk_ordinal)`:

```
id = fnv64a(path || "\0" || heading.unwrap_or("") || "\0" || chunk_ordinal.to_string())
```

This produces deterministic `u64` values across reindex runs for the same chunk. The separator `\0` prevents ambiguity between adjacent fields.

### Index Metadata (on-disk)

Persisted separately from the `.tvim` file to track configuration:

| Field | Type | Description |
|-------|------|-------------|
| `dim` | `usize` | Embedding dimension validated on load |
| `bit_width` | `u8` | Quantization level (2 or 4) |
| `version` | `u32` | Schema version for forward compatibility |
| `chunk_count` | `u64` | Number of indexed chunks (for health check parity) |

Stored as bincode-serialized struct at `.knowledge-loom-index/turbovec_config.bin`.

## State Transitions

### Index Lifecycle

```
[Empty] --add_with_ids--> [Active] --write--> [Persisted]
   ^                          |                    |
   |                          | remove(id)         | load
   +---load (corrupt)---------+                    |
                                                   v
                                              [Active]
```

### Migration Lifecycle

```
[Detect embeddings.db exists]
    |
    v
[Read sqlite-vec rows] --extract (path, heading, content, embedding_blob)-->
    |
    v
[Compute chunk IDs via fnv64a] --id, vector, metadata-->
    |
    v
[idMapIndex.add_with_ids] --count matches?-->
    |
    |-- YES --> [Delete embeddings.db] --> [Save .tvim + metadata] --> [Done]
    |-- NO  --> [Report error, keep old .db, log warning]
```

## Validation Rules

| Rule | Enforcement |
|------|-------------|
| Embedding dimension must match index `dim` after first add | `add_with_ids` returns error on mismatch; caller skips file per FR-010 |
| Chunk ID must be unique (no collisions) | `add_with_ids` rejects duplicate IDs; if collision detected, log error and skip |
| Index file version byte must match expected | On `load`, turbovec validates format; on corruption → fresh index per FR-008 |
| Metadata count must equal index `len()` | Validated on load; mismatch → log warning, rebuild metadata from BM25 |
