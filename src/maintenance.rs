use crate::bm25::BM25Index;
use crate::embed::EmbedProviderEnum;
use crate::graph::GraphState;
use crate::index::VectorIndex;
use crate::vault::VaultState;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct MaintenanceManager {
    pub kb_root: PathBuf,
    pub vault_state: Arc<Mutex<VaultState>>,
    pub bm25_index: Arc<Mutex<BM25Index>>,
    pub embed_provider: Arc<EmbedProviderEnum>,
    pub vector_index: Arc<Mutex<VectorIndex>>,
    pub graph_state: Arc<Mutex<GraphState>>,
}

impl MaintenanceManager {
    pub fn new(
        kb_root: String,
        vault_state: Arc<Mutex<VaultState>>,
        bm25_index: Arc<Mutex<BM25Index>>,
        embed_provider: Arc<EmbedProviderEnum>,
        vector_index: Arc<Mutex<VectorIndex>>,
        graph_state: Arc<Mutex<GraphState>>,
    ) -> Self {
        Self {
            kb_root: PathBuf::from(kb_root),
            vault_state,
            bm25_index,
            embed_provider,
            vector_index,
            graph_state,
        }
    }

    pub async fn reindex_all(&self) -> Result<String, String> {
        let mut status_lines = Vec::new();

        // Set ingestion state
        {
            let bm25_lock = self.bm25_index.lock().await;
            bm25_lock.set_ingesting(true).await;
        }

        // Scan files once and read all content upfront to avoid vault_lock contention
        let files = {
            let vault_lock = self.vault_state.lock().await;
            vault_lock.scan_files().await
        };
        let mut file_contents: Vec<(std::path::PathBuf, String)> = Vec::new();
        let vault_lock = self.vault_state.lock().await;
        for file_path in &files {
            if let Some(content) = vault_lock.read_file(file_path).await {
                file_contents.push((file_path.clone(), content));
            }
        }
        drop(vault_lock);

        // Run BM25 and vector indexing in parallel
        status_lines.push("Rebuilding indexes...".to_string());
        let bm25_idx = self.bm25_index.clone();
        let vector_idx = self.vector_index.clone();
        let embed = self.embed_provider.clone();
        let file_data = &file_contents;

        let (bm25_result, vector_result) = tokio::join!(
            async {
                let mut bm25_lock = bm25_idx.lock().await;
                let mut indexed = 0;
                for (path, content) in file_data {
                    if let Err(e) = bm25_lock.index_file(path, content).await {
                        return Err(format!("BM25 index failed for {}: {}", path.display(), e));
                    }
                    indexed += 1;
                }
                let mut writer = bm25_lock.writer.lock().await;
                if let Err(e) = writer.commit() {
                    return Err(format!("BM25 commit failed: {}", e));
                }
                Ok(indexed)
            },
            async {
                let vector_lock = vector_idx.lock().await;
                let mut total = 0;
                for (path, content) in file_data {
                    match vector_lock.index_file(path, content, &embed).await {
                        Ok((success, _failed)) => total += success,
                        Err(e) => {
                            eprintln!("Vector index failed for {}: {}", path.display(), e);
                        }
                    }
                }
                Ok(total)
            }
        );

        // Check BM25 result
        match bm25_result {
            Ok(count) => status_lines.push(format!("  BM25 index rebuilt ({} files)", count)),
            Err(e) => {
                let bm25_lock = self.bm25_index.lock().await;
                bm25_lock.set_ingesting(false).await;
                return Err(e);
            }
        }

        // Check vector result
        match vector_result {
            Ok(chunks) => status_lines.push(format!("  Vector index rebuilt ({} chunks)", chunks)),
            Err(e) => {
                let bm25_lock = self.bm25_index.lock().await;
                bm25_lock.set_ingesting(false).await;
                return Err(e);
            }
        }

        // Rebuild graph
        status_lines.push("Rebuilding graph...".to_string());
        let graph_result = self
            .graph_state
            .lock()
            .await
            .build_graph(&*self.vault_state.lock().await)
            .await;
        match graph_result {
            Ok(_) => {
                status_lines.push("  Graph rebuilt".to_string());
            }
            Err(e) => {
                let bm25_lock = self.bm25_index.lock().await;
                bm25_lock.set_ingesting(false).await;
                return Err(format!("Graph rebuild failed: {}", e));
            }
        }

        // Clear ingestion state on success
        {
            let bm25_lock = self.bm25_index.lock().await;
            bm25_lock.set_ingesting(false).await;
        }

        status_lines.push("Reindex complete.".to_string());
        Ok(status_lines.join("\n"))
    }

    pub async fn get_index_status(&self) -> Result<serde_json::Value, String> {
        let mut status = serde_json::json!({});

        // Vault status
        let vault_lock = self.vault_state.lock().await;
        let files = vault_lock.scan_files().await;
        status["vault"] = serde_json::json!({
            "files": files.len(),
            "kb_root": self.kb_root.to_string_lossy().to_string()
        });

        // BM25 status
        // TODO: Get actual document count from tantivy
        status["bm25"] = serde_json::json!({
            "documents": 0, // Placeholder
            "index_path": self.kb_root.join(".knowledge-loom-index/tantivy").to_string_lossy().to_string()
        });

        // Vector index status
        // TODO: Get actual vector count from sqlite
        status["embeddings"] = serde_json::json!({
            "vectors": 0, // Placeholder
            "index_path": self.kb_root.join(".knowledge-loom-index/embeddings.db").to_string_lossy().to_string()
        });

        // Graph status
        let graph_lock = self.graph_state.lock().await;
        let node_count = graph_lock.node_map.lock().await.len();
        // TODO: Get actual edge count
        status["graph"] = serde_json::json!({
            "nodes": node_count,
            "edges": 0, // Placeholder
            "index_path": self.kb_root.join(".knowledge-loom-index/graph.bin").to_string_lossy().to_string()
        });

        Ok(status)
    }
}
