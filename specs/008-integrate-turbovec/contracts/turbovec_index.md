# Interface Contracts

## TurbovecIndex Public API

This module (`src/turbovec_index.rs`) replaces `src/index.rs`. All callers in `search.rs`, `server.rs`, `daemon.rs`, and `maintenance.rs` use this contract.

### Constructor

```rust
impl TurbovecIndex {
    /// Create or load a turbovec index.
    /// If .knowledge-loom-index/turbovec.tvim exists, loads it.
    /// Otherwise creates an empty index with the given dimension and bit_width.
    pub async fn new(kb_root: &str, dim: usize, bit_width: u8) -> Self;
}
```

### Search

```rust
impl TurbovecIndex {
    /// Search for chunks similar to the query embedding.
    /// Returns (path, heading, content, similarity) sorted by similarity descending.
    /// similarity = Dot product score from turbovec (already L2-normalized).
    pub async fn search_similar(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<(String, Option<String>, String, f32)>, TurbovecError>;

    /// Search with an allowlist of chunk IDs.
    /// Only vectors whose ID is in `allowed_ids` participate.
    /// If allowed_ids is empty, falls back to unfiltered search.
    pub async fn search_filtered(
        &self,
        query_embedding: &[f32],
        limit: usize,
        allowed_ids: &[u64],
    ) -> Result<Vec<(String, Option<String>, String, f32)>, TurbovecError>;
}
```

### Index Mutation

```rust
impl TurbovecIndex {
    /// Add or update a single file's chunks.
    /// Deletes old entries for this path, then inserts new embeddings.
    /// Returns (successful_count, total_chunks).
    pub async fn index_file(
        &self,
        path: &Path,
        content: &str,
        embed_provider: &EmbedProviderEnum,
    ) -> Result<(usize, usize), TurbovecError>;

    /// Remove all chunks for a file.
    pub async fn remove_file(&self, path: &Path) -> Result<usize, TurbovecError>;

    /// Index all files in the vault (full reindex).
    pub async fn index_vault(
        &self,
        vault_state: &VaultState,
        embed_provider: &EmbedProviderEnum,
    ) -> Result<(usize, usize, usize), TurbovecError>;

    /// Count indexed chunks.
    pub async fn count(&self) -> usize;
}
```

### Persistence

```rust
impl TurbovecIndex {
    /// Persist index and metadata to disk.
    pub async fn save(&self) -> Result<(), TurbovecError>;

    /// Load index and metadata from disk.
    pub async fn load(&self) -> Result<(), TurbovecError>;
}
```

### Migration

```rust
impl TurbovecIndex {
    /// Migrate from legacy sqlite-vec embeddings.db to turbovec.
    /// Returns the number of chunks migrated.
    /// On success, deletes embeddings.db.
    pub fn migrate_from_sqlite(
        kb_root: &Path,
        index: &mut IdMapIndex,
        metadata: &mut HashMap<u64, ChunkMetadata>,
    ) -> Result<usize, TurbovecError>;
}
```

## Error Type

```rust
#[derive(Debug, thiserror::Error)]
pub enum TurbovecError {
    #[error("Dimension mismatch: index has {index_dim}, got {embedding_dim}")]
    DimensionMismatch { index_dim: usize, embedding_dim: usize },

    #[error("Duplicate chunk ID: {id}")]
    DuplicateId { id: u64 },

    #[error("Chunk not found: {id}")]
    ChunkNotFound { id: u64 },

    #[error("Index file corrupt or unreadable: {path}")]
    CorruptIndex { path: PathBuf },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("Embedding error: {0}")]
    Embed(String),
}
```

## SearchEngine Integration

The `SearchEngine` struct in `src/search.rs` changes one field:

```diff
- pub vector: Arc<Mutex<VectorIndex>>,
+ pub vector: Arc<Mutex<TurbovecIndex>>,
```

The `search_similar` call site at `search.rs:94` changes return type but the consumer (RRF merging at `search.rs:130-138`) is unchanged — it iterates `(path, heading, content, similarity)` tuples, which match the existing pattern.

### Graph-Fused Search Contract

`search_graph_fused_inner` currently calls `vector.search_similar(query_vec, top_k * 2)`. With turbovec, it additionally calls `search_filtered` when a graph context note is provided:

```rust
pub async fn search_graph_fused_inner(
    &self,
    query_vec: &[f32],
    pagerank: &HashMap<String, f64>,
    top_k: usize,
    context_note: Option<&str>,  // NEW: optional graph context
) -> Result<Vec<String>, String>;
```

When `context_note` is `Some`, graph neighbors are resolved to chunk IDs and passed as `allowed_ids` to `search_filtered`.
