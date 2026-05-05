use std::path::{Path, PathBuf};
use std::sync::Arc;
use rusqlite::{ffi::sqlite3_auto_extension, params, Connection, Result};
use tokio::sync::Mutex;
use sqlite_vec::sqlite3_vec_init;

pub struct VectorIndex {
    pub conn: Arc<Mutex<Connection>>,
    pub kb_root: PathBuf,
}

impl VectorIndex {
    pub async fn new(kb_root: &str) -> Self {
        let kb_root_path = PathBuf::from(kb_root);
        let index_path = kb_root_path.join(".loom-index/embeddings.db");
        
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
        ).expect("Failed to create tables");
        
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
    ) -> Result<()> {
        let mut conn_lock = self.conn.lock().await;
        
        // Convert embedding to blob
        let embedding_blob = bytemuck::cast_slice(embedding);
        
        conn_lock.execute(
            "
            INSERT OR REPLACE INTO embeddings (path, heading, content, embedding)
            VALUES (?1, ?2, ?3, ?4)
            ",
            params![
                path.to_string_lossy().as_ref(),
                heading.map(|s| s.to_string()).unwrap_or_default(),
                content,
                embedding_blob,
            ],
        )?;
        
        Ok(())
    }
    
    pub async fn remove_embedding(&self, path: &Path, heading: Option<&str>) -> Result<()> {
        let mut conn_lock = self.conn.lock().await;
        
        conn_lock.execute(
            "
            DELETE FROM embeddings 
            WHERE path = ?1 AND (heading = ?2 OR (?2 IS NULL AND heading IS NULL))
            ",
            params![
                path.to_string_lossy().as_ref(),
                heading.map(|s| s.to_string()).unwrap_or_default(),
            ],
        )?;
        
        Ok(())
    }
    
    pub async fn search_similar(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<(String, Option<String>, String, f32)>, rusqlite::Error> {
        let query_blob = bytemuck::cast_slice(query_embedding);
        let mut conn_lock = self.conn.lock().await;
        
        let mut stmt = conn_lock.prepare(
            "
            SELECT path, heading, content, vec_distance_cosine(embedding, ?1) as distance
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
    ) -> Result<(), rusqlite::Error> {
        let files = vault_state.scan_files().await;
        
        for file_path in files {
            // For simplicity, we'll reindex all files
            // In a real implementation, we'd check timestamps
            
            if let Some(content) = vault_state.read_file(&file_path).await {
                // Chunk the content by headings or fixed size
                let chunks = self.chunk_content(&content);
                
                for (heading, chunk_content) in chunks {
                    // Get embedding
                    let embedding = embed_provider.embed(&chunk_content).await;
                    
                    // Store in database
                    self.upsert_embedding(&file_path, heading.as_deref(), &chunk_content, &embedding)
                        .await?;
                }
            }
        }
        
        Ok(())
    }
    
    pub fn chunk_content(&self, content: &str) -> Vec<(Option<String>, String)> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        let mut current_heading = None;
        let mut current_chunk = Vec::new();
        let mut current_size = 0;
        
        for line in lines {
            // Check if line is a heading
            if line.trim().starts_with('#') {
                // Save previous chunk if exists
                if !current_chunk.is_empty() {
                    chunks.push((
                        current_heading.clone(),
                        current_chunk.join("\n")
                    ));
                    current_chunk = Vec::new();
                    current_size = 0;
                }
                
                // Extract heading text (remove # markers)
                let heading_text = line.trim_start_matches('#').trim();
                if !heading_text.is_empty() {
                    current_heading = Some(heading_text.to_string());
                } else {
                    current_heading = None;
                }
            }
            
            current_chunk.push(line);
            current_size += line.len() + 1; // +1 for newline
            
            // If chunk gets too big, save it and start a new one
            if current_size > 1000 { // ~1000 chars per chunk
                if !current_chunk.is_empty() {
                    chunks.push((
                        current_heading.clone(),
                        current_chunk.join("\n")
                    ));
                    current_chunk = Vec::new();
                    current_size = 0;
                }
            }
        }
        
        // Don't forget the last chunk
        if !current_chunk.is_empty() {
            chunks.push((
                current_heading.clone(),
                current_chunk.join("\n")
            ));
        }
        
        chunks
    }
}