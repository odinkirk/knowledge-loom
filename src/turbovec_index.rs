use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use turbovec::IdMapIndex;

use futures_util::StreamExt;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum TurbovecError {
    #[error("Dimension mismatch: index has {index_dim}, got {embedding_dim}")]
    DimensionMismatch {
        index_dim: usize,
        embedding_dim: usize,
    },

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChunkMetadata {
    pub path: String,
    pub heading: Option<String>,
    pub content: String,
    pub line_start: usize,
    pub line_end: usize,
    pub chunk_ordinal: u64,
}

fn chunk_id(path: &str, heading: &Option<String>, chunk_ordinal: u64) -> u64 {
    let heading_str = heading.as_deref().unwrap_or("");
    let combined = format!("{}\0{}\0{}", path, heading_str, chunk_ordinal);
    fnv64a(combined.as_bytes())
}

fn fnv64a(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
struct IndexConfig {
    dim: usize,
    bit_width: usize,
    version: u32,
    chunk_count: u64,
}

const INDEX_VERSION: u32 = 1;

pub struct TurbovecIndex {
    pub index: Arc<Mutex<IdMapIndex>>,
    pub metadata: Arc<Mutex<HashMap<u64, ChunkMetadata>>>,
    pub kb_root: PathBuf,
    index_path: PathBuf,
    meta_path: PathBuf,
    config_path: PathBuf,
    bit_width: usize,
    dim: usize,
}

pub fn default_bit_width() -> usize {
    if let Ok(val) = std::env::var("LOOM_TURBOVEC_BIT_WIDTH") {
        match val.trim() {
            "2" => 2,
            "4" => 4,
            other => {
                eprintln!(
                    "LOOM_TURBOVEC_BIT_WIDTH={} is invalid (expected 2 or 4). Using default 4.",
                    other
                );
                4
            }
        }
    } else {
        4
    }
}

impl TurbovecIndex {
    pub async fn new(kb_root: &str, dim: usize, bit_width: usize) -> Self {
        let kb_root_path = PathBuf::from(kb_root);
        let index_dir = kb_root_path.join(".knowledge-loom-index");
        let _ = std::fs::create_dir_all(&index_dir);

        let index_path = index_dir.join("turbovec.tvim");
        let meta_path = index_dir.join("turbovec_meta.bin");
        let config_path = index_dir.join("turbovec_config.bin");

        if index_path.exists() && meta_path.exists() {
            match Self::load_from_disk(&index_path, &meta_path, &config_path, dim, bit_width) {
                Ok((idx, meta)) => {
                    eprintln!(
                        "Loaded turbovec index: {} chunks, {} dim, {}-bit",
                        meta.len(),
                        dim,
                        bit_width
                    );
                    return Self {
                        index: Arc::new(Mutex::new(idx)),
                        metadata: Arc::new(Mutex::new(meta)),
                        kb_root: kb_root_path,
                        index_path,
                        meta_path,
                        config_path,
                        bit_width,
                        dim,
                    };
                }
                Err(e) => {
                    eprintln!(
                        "Failed to load turbovec index ({}): creating fresh index",
                        e
                    );
                }
            }
        }

        // Check for legacy sqlite-vec database to migrate
        {
            let legacy_db = index_dir.join("embeddings.db");
            if legacy_db.exists() {
                let mut idx = IdMapIndex::new(dim, bit_width)
                    .expect("Failed to create IdMapIndex for migration");
                let mut meta = HashMap::new();
                eprintln!("Legacy embeddings.db found — migrating to turbovec...");
                if let Err(e) = Self::migrate_from_sqlite_inner(
                    &kb_root_path, &mut idx, &mut meta,
                ) {
                    eprintln!("Migration failed ({}); starting with fresh index.", e);
                }
                return Self {
                    index: Arc::new(Mutex::new(idx)),
                    metadata: Arc::new(Mutex::new(meta)),
                    kb_root: kb_root_path,
                    index_path,
                    meta_path,
                    config_path,
                    bit_width,
                    dim,
                };
            }
        }

        let index = IdMapIndex::new(dim, bit_width).expect("Failed to create IdMapIndex");

        Self {
            index: Arc::new(Mutex::new(index)),
            metadata: Arc::new(Mutex::new(HashMap::new())),
            kb_root: kb_root_path,
            index_path,
            meta_path,
            config_path,
            bit_width,
            dim,
        }
    }

    fn migrate_from_sqlite_inner(
        kb_root: &Path,
        index: &mut IdMapIndex,
        metadata: &mut HashMap<u64, ChunkMetadata>,
    ) -> Result<usize, TurbovecError> {
        use rusqlite::Connection;
        use sqlite_vec::sqlite3_vec_init;

        let db_path = kb_root.join(".knowledge-loom-index/embeddings.db");
        if !db_path.exists() {
            eprintln!("No legacy embeddings.db found; skipping migration.");
            return Ok(0);
        }

        unsafe {
            rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite3_vec_init as *const (),
            )));
        }

        let conn =
            Connection::open(&db_path).map_err(|e| TurbovecError::Embed(e.to_string()))?;

        let mut stmt = conn
            .prepare("SELECT path, heading, content, embedding FROM embeddings")
            .map_err(|e| TurbovecError::Embed(e.to_string()))?;

        let rows: Vec<(String, String, String, Vec<f32>)> = stmt
            .query_map([], |row| {
                let path: String = row.get(0)?;
                let heading: String = row.get(1)?;
                let content: String = row.get(2)?;
                let blob: Vec<u8> = row.get(3)?;
                let floats: Vec<f32> = bytemuck::cast_slice(&blob).to_vec();
                Ok((path, heading, content, floats))
            })
            .map_err(|e| TurbovecError::Embed(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        if rows.is_empty() {
            eprintln!("Legacy embeddings.db is empty; deleting it.");
            let _ = std::fs::remove_file(&db_path);
            return Ok(0);
        }

        eprintln!(
            "Migrating {} chunks from sqlite-vec to turbovec...",
            rows.len()
        );

        let dim = index.dim();
        let n = rows.len();
        let mut flat_embeddings = Vec::with_capacity(n * dim);
        let mut ids = Vec::with_capacity(n);

        for (i, (path, heading, _content, embedding)) in rows.iter().enumerate() {
            if embedding.len() != dim && dim != 0 {
                eprintln!(
                    "Skipping chunk {} ({}): dimension mismatch ({} vs {})",
                    path,
                    heading,
                    embedding.len(),
                    dim
                );
                continue;
            }
            let heading_opt = if heading.is_empty() {
                None
            } else {
                Some(heading.clone())
            };
            let id = chunk_id(path, &heading_opt, i as u64);
            flat_embeddings.extend_from_slice(embedding);
            ids.push(id);
        }

        if ids.is_empty() {
            eprintln!("No compatible embeddings to migrate.");
            return Ok(0);
        }

        index
            .add_with_ids(&flat_embeddings, &ids)
            .map_err(|e| TurbovecError::Embed(format!("migration add_with_ids failed: {}", e)))?;

        for (i, (path, heading, content, _embedding)) in rows.iter().enumerate() {
            let heading_opt = if heading.is_empty() {
                None
            } else {
                Some(heading.clone())
            };
            let id = chunk_id(path, &heading_opt, i as u64);
            if metadata.contains_key(&id) {
                continue;
            }
            metadata.insert(
                id,
                ChunkMetadata {
                    path: path.clone(),
                    heading: heading_opt,
                    content: content.clone(),
                    line_start: 1,
                    line_end: 2,
                    chunk_ordinal: i as u64,
                },
            );
        }

        let count = metadata.len();
        eprintln!("Migration complete: {} chunks migrated.", count);

        if let Err(e) = std::fs::remove_file(&db_path) {
            eprintln!("Warning: failed to delete old embeddings.db: {}", e);
        } else {
            eprintln!("Deleted legacy embeddings.db.");
        }

        Ok(count)
    }

    fn load_from_disk(
        index_path: &Path,
        meta_path: &Path,
        _config_path: &Path,
        expected_dim: usize,
        _expected_bit_width: usize,
    ) -> Result<(IdMapIndex, HashMap<u64, ChunkMetadata>), TurbovecError> {
        let index = IdMapIndex::load(index_path.to_str().ok_or(TurbovecError::CorruptIndex {
            path: index_path.to_path_buf(),
        })?)
        .map_err(|_| TurbovecError::CorruptIndex {
            path: index_path.to_path_buf(),
        })?;

        if index.dim() != 0 && index.dim() != expected_dim {
            return Err(TurbovecError::DimensionMismatch {
                index_dim: index.dim(),
                embedding_dim: expected_dim,
            });
        }

        let meta_bytes = std::fs::read(meta_path)?;
        let metadata: HashMap<u64, ChunkMetadata> = bincode::deserialize(&meta_bytes)?;

        Ok((index, metadata))
    }

    pub async fn add_chunks(
        &self,
        chunks: &[crate::chunks::Chunk],
        embeddings: &[Vec<f32>],
        relative_path: &str,
    ) -> Result<usize, TurbovecError> {
        if chunks.is_empty() {
            return Ok(0);
        }

        let n = chunks.len();
        let mut ids = Vec::with_capacity(n);
        let mut meta_entries = Vec::with_capacity(n);

        for chunk in chunks.iter() {
            let id = chunk_id(relative_path, &chunk.heading, chunk.ordinal);
            ids.push(id);
            meta_entries.push(ChunkMetadata {
                path: relative_path.to_string(),
                heading: chunk.heading.clone(),
                content: chunk.content.clone(),
                line_start: chunk.line_start,
                line_end: chunk.line_end,
                chunk_ordinal: chunk.ordinal,
            });
        }

        for emb in embeddings.iter() {
            if emb.len() != self.dim {
                return Err(TurbovecError::DimensionMismatch {
                    index_dim: self.dim,
                    embedding_dim: emb.len(),
                });
            }
        }

        // Flatten embeddings into a single &[f32] for turbovec
        let flat_embeddings: Vec<f32> = embeddings.iter().flat_map(|e| e.iter().copied()).collect();

        let mut index_lock = self.index.lock().await;
        index_lock
            .add_with_ids(&flat_embeddings, &ids)
            .map_err(|e| TurbovecError::Embed(format!("add_with_ids failed: {}", e)))?;

        drop(index_lock);

        let mut meta_lock = self.metadata.lock().await;
        for (id, meta) in ids.into_iter().zip(meta_entries) {
            meta_lock.insert(id, meta);
        }

        Ok(n)
    }

    pub async fn search_similar(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<(String, Option<String>, String, f32)>, TurbovecError> {
        if query_embedding.is_empty() {
            return Ok(Vec::new());
        }

        let index_lock = self.index.lock().await;
        let n = index_lock.len();
        if n == 0 {
            return Ok(Vec::new());
        }

        let effective_k = limit.min(n);
        if effective_k == 0 {
            return Ok(Vec::new());
        }

        let (scores, ids) = index_lock.search(query_embedding, effective_k);
        drop(index_lock);

        let meta_lock = self.metadata.lock().await;
        let mut results: Vec<(String, Option<String>, String, f32)> =
            Vec::with_capacity(effective_k);

        for i in 0..ids.len().min(effective_k) {
            let id = ids[i];
            if let Some(meta) = meta_lock.get(&id) {
                let score = if i < scores.len() { scores[i] } else { 0.0 };
                let similarity = (score + 1.0) / 2.0;
                results.push((
                    meta.path.clone(),
                    meta.heading.clone(),
                    meta.content.clone(),
                    similarity,
                ));
            }
        }

        results.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
        Ok(results)
    }

    pub async fn search_filtered(
        &self,
        query_embedding: &[f32],
        limit: usize,
        allowed_ids: &[u64],
    ) -> Result<Vec<(String, Option<String>, String, f32)>, TurbovecError> {
        if query_embedding.is_empty() {
            return Ok(Vec::new());
        }

        if allowed_ids.is_empty() {
            return self.search_similar(query_embedding, limit).await;
        }

        let index_lock = self.index.lock().await;
        if index_lock.is_empty() {
            return Ok(Vec::new());
        }

        let effective_k = limit.min(allowed_ids.len());
        if effective_k == 0 {
            return Ok(Vec::new());
        }

        // Filter out IDs not in metadata to prevent turbovec panicking on
        // unknown allowlist entries
        let meta_check = self.metadata.lock().await;
        let valid_ids: Vec<u64> = allowed_ids
            .iter()
            .filter(|id| meta_check.contains_key(id))
            .copied()
            .collect();
        drop(meta_check);

        if valid_ids.is_empty() {
            return Ok(Vec::new());
        }

        let effective_k = effective_k.min(valid_ids.len());
        let (scores, ids) =
            index_lock.search_with_allowlist(query_embedding, effective_k, Some(&valid_ids));
        drop(index_lock);

        let meta_lock = self.metadata.lock().await;
        let mut results: Vec<(String, Option<String>, String, f32)> =
            Vec::with_capacity(effective_k);

        for i in 0..ids.len().min(effective_k) {
            let id = ids[i];
            if let Some(meta) = meta_lock.get(&id) {
                let score = if i < scores.len() { scores[i] } else { 0.0 };
                let similarity = (score + 1.0) / 2.0;
                results.push((
                    meta.path.clone(),
                    meta.heading.clone(),
                    meta.content.clone(),
                    similarity,
                ));
            }
        }

        results.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
        Ok(results)
    }

    pub async fn remove_file(&self, path: &Path) -> Result<usize, TurbovecError> {
        let relative_path = path
            .strip_prefix(&self.kb_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        // Acquire index lock first to match ordering in save/search/add_chunks,
        // preventing deadlock with concurrent save() calls.
        let mut index_lock = self.index.lock().await;
        let mut meta_lock = self.metadata.lock().await;

        let ids_to_remove: Vec<u64> = meta_lock
            .iter()
            .filter(|(_, meta)| meta.path == relative_path)
            .map(|(&id, _)| id)
            .collect();

        let count = ids_to_remove.len();
        for id in &ids_to_remove {
            meta_lock.remove(id);
            index_lock.remove(*id);
        }

        Ok(count)
    }

    pub async fn count(&self) -> usize {
        self.metadata.lock().await.len()
    }

    /// Get all chunk IDs associated with a note (by path stem or full path).
    pub async fn chunk_ids_for_note(&self, note_name: &str) -> Vec<u64> {
        let meta = self.metadata.lock().await;
        meta.iter()
            .filter(|(_, m)| {
                let stem = m
                    .path
                    .strip_suffix(".md")
                    .unwrap_or(&m.path);
                stem == note_name || m.path == note_name
            })
            .map(|(&id, _)| id)
            .collect()
    }

    pub async fn index_file(
        &self,
        path: &Path,
        content: &str,
        embed_provider: &crate::embed::EmbedProviderEnum,
    ) -> Result<(usize, usize), TurbovecError> {
        let chunks = crate::chunks::parse_chunks(content);
        if chunks.is_empty() {
            return Ok((0, 0));
        }

        let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let embeddings = match embed_provider.embed_batch(&texts).await {
            Ok(vec) => vec,
            Err(e) => {
                eprintln!(
                    "Failed to generate batch embedding for {}: {}. Skipping file.",
                    path.display(),
                    e
                );
                return Ok((0, chunks.len()));
            }
        };

        let relative_path = path
            .strip_prefix(&self.kb_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        let _ = self.remove_file(path).await;

        let successful = self
            .add_chunks(&chunks, &embeddings, &relative_path)
            .await?;
        let failed = chunks.len().saturating_sub(successful);

        if failed > 0 {
            eprintln!(
                "Indexing completed for {}: {} successful, {} failed",
                path.display(),
                successful,
                failed
            );
        }

        Ok((successful, failed))
    }

    pub async fn index_vault(
        &self,
        vault_state: &crate::vault::VaultState,
        embed_provider: &crate::embed::EmbedProviderEnum,
    ) -> Result<(usize, usize, usize), TurbovecError> {
        let files = vault_state.scan_files().await;
        let mut total_successful = 0;
        let mut total_failed = 0;

        // Process files with limited parallelism.
        // Embedding generation is CPU-bound ONNX inference; we use low
        // concurrency to avoid thread oversubscription with ONNX's
        // internal threading. Still faster than fully sequential since
        // ONNX can overlap I/O and compute across files.
        let concurrency = 2; // ONNX Runtime serialises internally; >2 causes hangs

        let results: Vec<_> = futures_util::stream::iter(files)
            .map(|file_path| {
                let vault = vault_state;
                let embed = embed_provider;
                async move {
                    let content = match vault.read_file(&file_path).await {
                        Some(c) => c,
                        None => return (file_path, None),
                    };
                    let chunks = crate::chunks::parse_chunks(&content);
                    if chunks.is_empty() {
                        return (file_path, None);
                    }
                    let texts: Vec<String> =
                        chunks.iter().map(|c| c.content.clone()).collect();
                    match embed.embed_batch(&texts).await {
                        Ok(embeddings) => (file_path, Some((chunks, embeddings))),
                        Err(e) => {
                            eprintln!(
                                "embed_batch failed for {}: {}",
                                file_path.display(),
                                e
                            );
                            (file_path, None)
                        }
                    }
                }
            })
            .buffer_unordered(concurrency)
            .collect()
            .await;

        // Add to turbovec sequentially (fast — no embedding wait)
        for (file_path, data) in results {
            match data {
                Some((chunks, embeddings)) => {
                    let relative_path = file_path
                        .strip_prefix(&self.kb_root)
                        .unwrap_or(&file_path)
                        .to_string_lossy()
                        .to_string();

                    let _ = self.remove_file(&file_path).await;

                    match self
                        .add_chunks(&chunks, &embeddings, &relative_path)
                        .await
                    {
                        Ok(successful) => {
                            total_successful += successful;
                        }
                        Err(e) => {
                            eprintln!(
                                "Failed to add chunks for {}: {}",
                                file_path.display(),
                                e
                            );
                            total_failed += chunks.len();
                        }
                    }
                }
                None => {
                    total_failed += 1;
                }
            }
        }

        if total_successful > 0 {
            let _ = self.save().await;
        }

        Ok((total_successful, total_failed, 0))
    }

    pub async fn save(&self) -> Result<(), TurbovecError> {
        let index_lock = self.index.lock().await;
        let meta_lock = self.metadata.lock().await;

        let index_path_str =
            self.index_path
                .to_str()
                .ok_or_else(|| TurbovecError::CorruptIndex {
                    path: self.index_path.clone(),
                })?;
        index_lock
            .write(index_path_str)
            .map_err(|e| TurbovecError::Io(std::io::Error::other(e)))?;

        let meta_bytes = bincode::serialize(&*meta_lock)?;
        std::fs::write(&self.meta_path, &meta_bytes)?;

        let config = IndexConfig {
            dim: self.dim,
            bit_width: self.bit_width,
            version: INDEX_VERSION,
            chunk_count: meta_lock.len() as u64,
        };
        let config_bytes = bincode::serialize(&config)?;
        std::fs::write(&self.config_path, &config_bytes)?;

        eprintln!("Saved turbovec index: {} chunks", meta_lock.len());
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn migrate_from_sqlite(&self) -> Result<usize, TurbovecError> {
        let mut index = self.index.lock().await;
        let mut metadata = self.metadata.lock().await;
        Self::migrate_from_sqlite_inner(&self.kb_root, &mut index, &mut metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chunk_id_deterministic() {
        let id1 = chunk_id("notes/test.md", &Some("Heading".to_string()), 0);
        let id2 = chunk_id("notes/test.md", &Some("Heading".to_string()), 0);
        assert_eq!(id1, id2);

        let id3 = chunk_id("notes/test.md", &Some("Heading".to_string()), 1);
        assert_ne!(id1, id3);

        let id4 = chunk_id("notes/other.md", &Some("Heading".to_string()), 0);
        assert_ne!(id1, id4);

        let id5 = chunk_id("notes/test.md", &Some("Other".to_string()), 0);
        assert_ne!(id1, id5);
    }

    #[tokio::test]
    async fn test_new_and_count() {
        let temp_dir = tempfile::tempdir().unwrap();
        let index = TurbovecIndex::new(temp_dir.path().to_str().unwrap(), 8, 4).await;
        assert_eq!(index.count().await, 0);
    }

    #[tokio::test]
    async fn test_add_and_search() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dim = 8;
        let index = TurbovecIndex::new(temp_dir.path().to_str().unwrap(), dim, 4).await;

        let chunks = vec![
            crate::chunks::Chunk {
                heading: Some("A".to_string()),
                content: "a".to_string(),
                line_start: 1,
                line_end: 2,
                ordinal: 0,
            },
            crate::chunks::Chunk {
                heading: Some("B".to_string()),
                content: "b".to_string(),
                line_start: 2,
                line_end: 3,
                ordinal: 1,
            },
        ];

        let emb1: Vec<f32> = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let emb2: Vec<f32> = vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let embeddings: Vec<Vec<f32>> = vec![emb1.clone(), emb2.clone()];

        let count = index
            .add_chunks(&chunks, &embeddings, "notes/a.md")
            .await
            .unwrap();
        assert_eq!(count, 2);
        assert_eq!(index.count().await, 2);

        // Search with first vector
        let results = index.search_similar(&emb1, 3).await.unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_remove_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let index = TurbovecIndex::new(temp_dir.path().to_str().unwrap(), 8, 4).await;

        let chunks = vec![
            crate::chunks::Chunk {
                heading: Some("A".to_string()),
                content: "a".to_string(),
                line_start: 1,
                line_end: 2,
                ordinal: 0,
            },
            crate::chunks::Chunk {
                heading: Some("B".to_string()),
                content: "b".to_string(),
                line_start: 2,
                line_end: 3,
                ordinal: 1,
            },
        ];
        let embeddings: Vec<Vec<f32>> = vec![
            vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ];
        index
            .add_chunks(&chunks, &embeddings, "notes/a.md")
            .await
            .unwrap();
        assert_eq!(index.count().await, 2);

        let removed = index.remove_file(Path::new("notes/a.md")).await.unwrap();
        assert_eq!(removed, 2);
        assert_eq!(index.count().await, 0);
    }

    #[tokio::test]
    async fn test_empty_search() {
        let temp_dir = tempfile::tempdir().unwrap();
        let index = TurbovecIndex::new(temp_dir.path().to_str().unwrap(), 8, 4).await;

        let results = index
            .search_similar(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 10)
            .await
            .unwrap();
        assert!(results.is_empty());
    }
}
