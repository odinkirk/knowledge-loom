use rusqlite::{ffi::sqlite3_auto_extension, params, Connection, Result as SqliteResult};
use sqlite_vec::sqlite3_vec_init;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

#[allow(dead_code)]
pub struct VectorIndex {
    pub conn: Arc<Mutex<Connection>>,
    pub kb_root: PathBuf,
}

impl VectorIndex {
    pub async fn new(kb_root: &str) -> Self {
        let kb_root_path = PathBuf::from(kb_root);
        let index_path = kb_root_path.join(".knowledge-loom-index/embeddings.db");

        // Create directory if it doesn't exist
        let _ = std::fs::create_dir_all(index_path.parent().unwrap());

        // Initialize sqlite-vec extension globally
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
        }

        // Open or create database
        let conn = Connection::open(&index_path).expect("Failed to open database");

        // Create tables if they don't exist
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS embeddings (
                id INTEGER PRIMARY KEY,
                path TEXT NOT NULL,
                heading TEXT,
                content TEXT NOT NULL,
                embedding BLOB NOT NULL,
                UNIQUE(path, heading)
            );
            CREATE INDEX IF NOT EXISTS idx_embeddings_path ON embeddings(path);
            ",
        )
        .expect("Failed to create tables");

        Self {
            conn: Arc::new(Mutex::new(conn)),
            kb_root: kb_root_path,
        }
    }

    #[allow(dead_code)]
    pub async fn upsert_embedding(
        &self,
        path: &Path,
        heading: Option<&str>,
        content: &str,
        embedding: &[f32],
    ) -> SqliteResult<()> {
        let conn_lock = self.conn.lock().await;

        // Convert to relative path from kb_root
        let relative_path = path
            .strip_prefix(&self.kb_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        // Convert embedding to blob
        let embedding_blob = bytemuck::cast_slice(embedding);

        conn_lock.execute(
            "
            INSERT OR REPLACE INTO embeddings (path, heading, content, embedding)
            VALUES (?1, ?2, ?3, ?4)
            ",
            params![
                relative_path,
                heading.map(|s| s.to_string()).unwrap_or_default(),
                content,
                embedding_blob,
            ],
        )?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn remove_embedding(&self, path: &Path, heading: Option<&str>) -> SqliteResult<()> {
        let conn_lock = self.conn.lock().await;

        // Convert to relative path from kb_root
        let relative_path = path
            .strip_prefix(&self.kb_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        conn_lock.execute(
            "
            DELETE FROM embeddings
            WHERE path = ?1 AND (heading = ?2 OR (?2 IS NULL AND heading IS NULL))
            ",
            params![
                relative_path,
                heading.map(|s| s.to_string()).unwrap_or_default(),
            ],
        )?;

        Ok(())
    }

    pub async fn remove_file_embeddings(&self, path: &Path) -> SqliteResult<()> {
        let conn_lock = self.conn.lock().await;
        let relative_path = path
            .strip_prefix(&self.kb_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        conn_lock.execute(
            "DELETE FROM embeddings WHERE path = ?1",
            params![relative_path],
        )?;
        Ok(())
    }

    pub async fn index_file(
        &self,
        path: &Path,
        content: &str,
        embed_provider: &crate::embed::EmbedProviderEnum,
    ) -> Result<(usize, usize), rusqlite::Error> {
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

        let mut successful_count: usize = 0;
        let conn_lock = self.conn.lock().await;

        // Delete old rows and insert new ones in a single transaction
        let relative_path = path
            .strip_prefix(&self.kb_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        conn_lock.execute(
            "DELETE FROM embeddings WHERE path = ?1",
            params![relative_path],
        )?;

        conn_lock.execute_batch("BEGIN TRANSACTION")?;
        for (chunk, embedding) in chunks.iter().zip(embeddings.iter()) {
            let heading = chunk
                .heading
                .as_deref()
                .map(|s| s.to_string())
                .unwrap_or_default();
            let embedding_blob = bytemuck::cast_slice(embedding);
            conn_lock.execute(
                "INSERT OR REPLACE INTO embeddings (path, heading, content, embedding) VALUES (?1, ?2, ?3, ?4)",
                params![relative_path, heading, &chunk.content, embedding_blob],
            )?;
            successful_count += 1;
        }
        conn_lock.execute_batch("COMMIT")?;
        drop(conn_lock);

        let failed_count = chunks.len().saturating_sub(successful_count);

        // Log summary of indexing results
        if failed_count > 0 {
            eprintln!(
                "Indexing completed for {}: {} successful, {} failed",
                path.display(),
                successful_count,
                failed_count
            );
        }

        Ok((successful_count, failed_count))
    }

    pub async fn search_similar(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<(String, Option<String>, String, f32)>, rusqlite::Error> {
        let query_blob = bytemuck::cast_slice(query_embedding);
        let conn_lock = self.conn.lock().await;

        let mut stmt = conn_lock.prepare(
            "
            SELECT path, heading, content, COALESCE(vec_distance_cosine(embedding, ?1), 1.0) as distance
            FROM embeddings
            ORDER BY distance
            LIMIT ?2
            ",
        )?;

        let mut rows = stmt.query(params![query_blob, limit as i32])?;

        let mut results = Vec::new();
        while let Some(row) = rows.next()? {
            let path: String = row.get(0)?;
            let heading: Option<String> = row.get(1)?;
            let content: String = row.get(2)?;
            let distance: f32 = row.get(3)?;
            // Convert distance to similarity (1 - distance for cosine)
            let similarity = 1.0 - distance;
            results.push((path, heading, content, similarity));
        }

        Ok(results)
    }

    pub async fn index_vault(
        &self,
        vault_state: &crate::vault::VaultState,
        embed_provider: &crate::embed::EmbedProviderEnum,
    ) -> Result<(usize, usize, usize), rusqlite::Error> {
        let files = vault_state.scan_files().await;
        let mut total_successful = 0;
        let mut total_failed = 0;

        for file_path in files {
            let content = match vault_state.read_file(&file_path).await {
                Some(c) => c,
                None => continue,
            };
            let chunks = crate::chunks::parse_chunks(&content);
            if chunks.is_empty() {
                continue;
            }

            // Per-file batch embed
            let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
            let embeddings = match embed_provider.embed_batch(&texts).await {
                Ok(vec) => vec,
                Err(e) => {
                    eprintln!("embed_batch failed for {}: {}", file_path.display(), e);
                    total_failed += chunks.len();
                    continue;
                }
            };

            // SQLite per-file transaction
            let conn_lock = self.conn.lock().await;
            let relative_path = file_path
                .strip_prefix(&self.kb_root)
                .unwrap_or(&file_path)
                .to_string_lossy()
                .to_string();
            conn_lock.execute(
                "DELETE FROM embeddings WHERE path = ?1",
                params![relative_path],
            )?;
            conn_lock.execute_batch("BEGIN TRANSACTION")?;
            for (chunk, embedding) in chunks.iter().zip(embeddings.iter()) {
                let heading = chunk
                    .heading
                    .as_deref()
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                let embedding_blob = bytemuck::cast_slice(embedding);
                conn_lock.execute(
                    "INSERT OR REPLACE INTO embeddings (path, heading, content, embedding) VALUES (?1, ?2, ?3, ?4)",
                    params![relative_path, heading, &chunk.content, embedding_blob],
                )?;
            }
            conn_lock.execute_batch("COMMIT")?;
            drop(conn_lock);
            total_successful += chunks.len();
        }

        Ok((total_successful, total_failed, 0))
    }

    #[allow(dead_code)]
    pub fn chunk_content(&self, _content: &str) -> Vec<(Option<String>, String)> {
        // Replaced by crate::chunks::parse_chunks — kept for API compatibility
        crate::chunks::parse_chunks(_content)
            .into_iter()
            .map(|c| (c.heading, c.content))
            .collect()
    }
}
