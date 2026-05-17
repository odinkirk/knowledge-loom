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
        self.remove_file_embeddings(path).await?;
        let chunks = self.chunk_content(content);
        let mut successful_count = 0;
        let mut failed_count = 0;

        for (heading, chunk_content) in chunks {
            let embedding = match embed_provider.embed(&chunk_content).await {
                Ok(vec) => vec,
                Err(e) => {
                    eprintln!(
                        "Failed to generate embedding for chunk in {}: {}. Skipping this chunk.",
                        path.display(),
                        e
                    );
                    failed_count += 1;
                    // Skip this chunk if embedding fails
                    continue;
                }
            };
            self.upsert_embedding(path, heading.as_deref(), &chunk_content, &embedding)
                .await?;
            successful_count += 1;
        }

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
        let mut files_with_failures = 0;

        for file_path in files {
            if let Some(content) = vault_state.read_file(&file_path).await {
                self.remove_file_embeddings(&file_path).await?;
                let chunks = self.chunk_content(&content);
                let mut file_successful = 0;
                let mut file_failed = 0;

                for (heading, chunk_content) in chunks {
                    let embedding = match embed_provider.embed(&chunk_content).await {
                        Ok(vec) => vec,
                        Err(e) => {
                            eprintln!("Failed to generate embedding for chunk in {}: {}. Skipping this chunk.", file_path.display(), e);
                            file_failed += 1;
                            // Skip this chunk if embedding fails
                            continue;
                        }
                    };
                    self.upsert_embedding(
                        &file_path,
                        heading.as_deref(),
                        &chunk_content,
                        &embedding,
                    )
                    .await?;
                    file_successful += 1;
                }

                total_successful += file_successful;
                total_failed += file_failed;
                if file_failed > 0 {
                    files_with_failures += 1;
                }
            }
        }

        // Log summary of indexing results
        if total_failed > 0 {
            eprintln!(
                "Vault indexing completed: {} total chunks, {} successful, {} failed across {} files",
                total_successful + total_failed,
                total_successful,
                total_failed,
                files_with_failures
            );
        }

        Ok((total_successful, total_failed, files_with_failures))
    }

    pub fn chunk_content(&self, content: &str) -> Vec<(Option<String>, String)> {
        let mut chunks = Vec::new();
        let mut current_heading: Option<String> = None;
        let mut current_content = String::new();

        for line in content.lines() {
            // Strip any heading level (H1-H6): 1-6 # chars followed by space
            if let Some(stripped) = line.trim_start_matches('#').strip_prefix(" ") {
                if !current_content.trim().is_empty() || current_heading.is_some() {
                    chunks.push((current_heading.take(), current_content.trim().to_string()));
                    current_content.clear();
                }
                current_heading = Some(stripped.to_string());
            } else {
                current_content.push_str(line);
                current_content.push('\n');
            }
        }
        if !current_content.trim().is_empty() || current_heading.is_some() {
            chunks.push((current_heading, current_content.trim().to_string()));
        }
        chunks
    }
}
