use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::vault::VaultState;
use crate::bm25::BM25Index;
use crate::embed::EmbedProviderEnum;
use crate::index::VectorIndex;
use crate::graph::GraphState;

pub struct MaintenanceManager {
    pub kb_root: PathBuf,
    pub vault_state: Arc<Mutex<VaultState>>,
    pub bm25_index: Arc<Mutex<BM25Index>>,
    pub embed_provider: Arc<Mutex<EmbedProviderEnum>>,
    pub vector_index: Arc<Mutex<VectorIndex>>,
    pub graph_state: Arc<Mutex<GraphState>>,
}

impl MaintenanceManager {
    pub fn new(
        kb_root: String,
        vault_state: Arc<Mutex<VaultState>>,
        bm25_index: Arc<Mutex<BM25Index>>,
        embed_provider: Arc<Mutex<EmbedProviderEnum>>,
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
        
        // Rebuild BM25 index
        status_lines.push("Rebuilding BM25 index...".to_string());
        let mut bm25_lock = self.bm25_index.lock().await;
        bm25_lock.index_vault(&*self.vault_state.lock().await).await
            .map_err(|e| e.to_string())?;
        status_lines.push("  BM25 index rebuilt".to_string());
        
        // Rebuild vector index
        status_lines.push("Rebuilding vector index...".to_string());
        let embed_lock = self.embed_provider.lock().await;
        self.vector_index.lock().await
            .index_vault(&*self.vault_state.lock().await, &*embed_lock).await
            .map_err(|e| e.to_string())?;
        status_lines.push("  Vector index rebuilt".to_string());
        
        // Rebuild graph
        status_lines.push("Rebuilding graph...".to_string());
        self.graph_state.lock().await
            .build_graph(&*self.vault_state.lock().await).await
            .map_err(|e| e.to_string())?;
        status_lines.push("  Graph rebuilt".to_string());
        
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