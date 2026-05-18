use crate::bm25::BM25Index;
use crate::embed::EmbedProviderEnum;
use crate::graph::GraphState;
use crate::index::VectorIndex;
use crate::vault::VaultState;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Per-file state for incremental reindex tracking.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
struct FileState {
    mtime_secs: u64,
    chunk_count: usize,
}

/// Tracks which files have been indexed for incremental reindex.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ReindexState {
    schema_version: u32,
    files: HashMap<String, FileState>,
    #[serde(skip)]
    state_path: PathBuf,
}

impl ReindexState {
    const SCHEMA_VERSION: u32 = 1;

    pub fn load(kb_root: &Path) -> Self {
        let state_path = kb_root.join(".knowledge-loom-index/reindex-state.json");
        if state_path.exists() {
            if let Ok(data) = std::fs::read_to_string(&state_path) {
                if let Ok(mut state) = serde_json::from_str::<ReindexState>(&data) {
                    if state.schema_version == Self::SCHEMA_VERSION {
                        state.state_path = state_path;
                        return state;
                    }
                }
            }
        }
        Self {
            schema_version: Self::SCHEMA_VERSION,
            files: HashMap::new(),
            state_path,
        }
    }

    pub fn should_reindex(
        &self,
        path: &str,
        current_mtime: u64,
        current_chunk_count: usize,
    ) -> bool {
        match self.files.get(path) {
            Some(stored) => {
                stored.mtime_secs != current_mtime || stored.chunk_count != current_chunk_count
            }
            None => true,
        }
    }

    pub fn update_file(&mut self, path: &str, mtime_secs: u64, chunk_count: usize) {
        self.files.insert(
            path.to_string(),
            FileState {
                mtime_secs,
                chunk_count,
            },
        );
    }

    pub fn remove_file(&mut self, path: &str) {
        self.files.remove(path);
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(parent) = self.state_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(&self.state_path, json)
    }

    pub fn tracked_paths(&self) -> Vec<&str> {
        self.files.keys().map(|k| k.as_str()).collect()
    }
}

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

    pub async fn reindex_all(&self, _force: bool) -> Result<String, String> {
        let total_start = std::time::Instant::now();
        let mut state = ReindexState::load(&self.kb_root);

        // Full rebuild always runs for now; incremental state is saved for next run.
        // TODO: Incremental path (Finding 4) — state file saved, comparison logic written,
        // but hangs inside reindex_incremental. Debug separately.
        eprintln!("  [perf] starting full rebuild...");

        let mut bm25_lock = {
            let bm25_lock = self.bm25_index.lock().await;
            bm25_lock.set_ingesting(true).await;
            bm25_lock
        };

        eprintln!("  [perf] BM25 index_vault starting...");
        let bm25_start = std::time::Instant::now();
        bm25_lock
            .index_vault(&*self.vault_state.lock().await)
            .await
            .map_err(|e| format!("BM25: {}", e))?;
        eprintln!("  [perf] BM25 index_vault: {:?}", bm25_start.elapsed());

        eprintln!("  [perf] Vector index_vault starting...");
        let vector_start = std::time::Instant::now();
        let (successful, _, _) = {
            let vector = self.vector_index.lock().await;
            let vault = self.vault_state.lock().await;
            vector
                .index_vault(&*vault, &self.embed_provider)
                .await
                .map_err(|e| format!("Vector: {}", e))?
        };
        eprintln!(
            "  [perf] Vector index_vault: {:?} ({} chunks)",
            vector_start.elapsed(),
            successful
        );

        eprintln!("  [perf] Graph build starting...");
        let graph_start = std::time::Instant::now();
        self.graph_state
            .lock()
            .await
            .build_graph(&*self.vault_state.lock().await)
            .await
            .map_err(|e| format!("Graph: {}", e))?;
        eprintln!("  [perf] Graph build: {:?}", graph_start.elapsed());

        bm25_lock.set_ingesting(false).await;

        eprintln!("  [perf] Saving state...");
        let vault_lock = self.vault_state.lock().await;
        let files = vault_lock.scan_files().await;
        for file_path in &files {
            if let Some(content) = vault_lock.read_file(file_path).await {
                let relative = file_path
                    .strip_prefix(&self.kb_root)
                    .unwrap_or(file_path)
                    .to_string_lossy()
                    .to_string();
                let mtime = vault_lock
                    .get_file_mod_time(file_path)
                    .await
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                state.update_file(&relative, mtime, content.lines().count());
            }
        }
        drop(vault_lock);
        let _ = state.save();
        eprintln!("  [perf] TOTAL reindex_all: {:?}", total_start.elapsed());
        Ok("Reindex complete.".to_string())
    }

    #[allow(dead_code)]
    async fn reindex_incremental(&self, state: &mut ReindexState) -> Result<String, String> {
        let start = std::time::Instant::now();
        let mut status_lines = Vec::new();
        status_lines.push("Incremental reindex...".to_string());
        let _files = self.vault_state.lock().await.scan_files().await;

        let files = self.vault_state.lock().await.scan_files().await;
        let disk_paths: std::collections::HashSet<String> = files
            .iter()
            .map(|p| {
                p.strip_prefix(&self.kb_root)
                    .unwrap_or(p)
                    .to_string_lossy()
                    .to_string()
            })
            .collect();

        // Clean deleted files
        let tracked: Vec<String> = state
            .tracked_paths()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        for path in &tracked {
            if !disk_paths.contains(path.as_str()) {
                let full = self.kb_root.join(path);
                let mut bm25 = self.bm25_index.lock().await;
                let _ = bm25.remove_document(&full).await;
                drop(bm25);
                let vector = self.vector_index.lock().await;
                let _ = vector.remove_file_embeddings(&full).await;
                drop(vector);
                state.remove_file(path);
            }
        }

        // Find changed files
        let mut changed: Vec<(std::path::PathBuf, String)> = Vec::new();
        let vault = self.vault_state.lock().await;
        for fp in &files {
            let rel = fp
                .strip_prefix(&self.kb_root)
                .unwrap_or(fp)
                .to_string_lossy()
                .to_string();
            let mtime = vault
                .get_file_mod_time(fp)
                .await
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let content = match vault.read_file(fp).await {
                Some(c) => c,
                None => continue,
            };
            let cc = crate::chunks::parse_chunks(&content).len();
            if state.should_reindex(&rel, mtime, cc) {
                changed.push((fp.clone(), content));
            }
        }
        drop(vault);

        if changed.is_empty() {
            status_lines.push("No changes detected.".to_string());
        } else {
            status_lines.push(format!("Reindexing {} changed files", changed.len()));
            let mut bm25 = self.bm25_index.lock().await;
            bm25.set_ingesting(true).await;
            for (fp, content) in &changed {
                bm25.index_file(fp, content)
                    .await
                    .map_err(|e| format!("BM25: {}", e))?;
                self.vector_index
                    .lock()
                    .await
                    .index_file(fp, content, &self.embed_provider)
                    .await
                    .map_err(|e| e.to_string())?;
                let rel = fp
                    .strip_prefix(&self.kb_root)
                    .unwrap_or(fp)
                    .to_string_lossy()
                    .to_string();
                state.update_file(
                    &rel,
                    std::fs::metadata(fp)
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
                    crate::chunks::parse_chunks(content).len(),
                );
            }
            bm25.set_ingesting(false).await;
            // Rebuild graph
            self.graph_state
                .lock()
                .await
                .build_graph(&*self.vault_state.lock().await)
                .await
                .map_err(|e| format!("Graph: {}", e))?;
        }
        let _ = state.save();
        status_lines
            .push(format!("Incremental reindex complete in {:?}", start.elapsed()).to_string());
        Ok(status_lines.join("\n"))
    }

    pub async fn get_index_status(&self) -> Result<serde_json::Value, String> {
        let mut status = serde_json::json!({});
        let vault_lock = self.vault_state.lock().await;
        let files = vault_lock.scan_files().await;
        status["vault"] = serde_json::json!({
            "files": files.len(),
            "kb_root": self.kb_root.to_string_lossy().to_string()
        });
        status["bm25"] = serde_json::json!({
            "documents": 0,
            "index_path": self.kb_root.join(".knowledge-loom-index/tantivy").to_string_lossy().to_string()
        });
        status["embeddings"] = serde_json::json!({
            "vectors": 0,
            "index_path": self.kb_root.join(".knowledge-loom-index/embeddings.db").to_string_lossy().to_string()
        });
        let graph_lock = self.graph_state.lock().await;
        let node_count = graph_lock.node_map.lock().await.len();
        status["graph"] = serde_json::json!({
            "nodes": node_count,
            "edges": 0,
            "index_path": self.kb_root.join(".knowledge-loom-index/graph.bin").to_string_lossy().to_string()
        });
        Ok(status)
    }
}
