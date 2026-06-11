use crate::bm25::BM25Index;
use crate::embed::EmbedProviderEnum;
use crate::graph::GraphState;
use crate::turbovec_index::TurbovecIndex;
use crate::vault::VaultState;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
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
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
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
    pub vector_index: Arc<Mutex<TurbovecIndex>>,
    pub graph_state: Arc<Mutex<GraphState>>,
}

impl MaintenanceManager {
    pub fn new(
        kb_root: String,
        vault_state: Arc<Mutex<VaultState>>,
        bm25_index: Arc<Mutex<BM25Index>>,
        embed_provider: Arc<EmbedProviderEnum>,
        vector_index: Arc<Mutex<TurbovecIndex>>,
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

    pub async fn reindex_all(&self, force: bool) -> Result<String, String> {
        let total_start = std::time::Instant::now();
        let mut state = ReindexState::load(&self.kb_root);

        // Incremental path: only if not forced and state has previously tracked files
        if !force && !state.files.is_empty() {
            eprintln!(
                "{} previously tracked files in state; attempting incremental reindex...",
                state.files.len()
            );
            match self.reindex_incremental(&mut state).await {
                Ok(msg) => {
                    eprintln!(
                        "  [perf] TOTAL reindex_all (incremental): {:?}",
                        total_start.elapsed()
                    );
                    return Ok(msg);
                }
                Err(e) => {
                    eprintln!(
                        "  Incremental reindex failed ({}); falling back to full rebuild.",
                        e
                    );
                    state = ReindexState::load(&self.kb_root);
                }
            }
        }

        // Index health check: verify embedding count (non-blocking, advisory)
        {
            let vector = self.vector_index.lock().await;
            let _ = self.verify_embedding_count(&state, &vector).await;
            drop(vector);
        }

        eprintln!("Full rebuild in progress (may take several minutes). Use --force to skip incremental check, or wait for incremental path.");
        eprintln!("  [perf] starting full rebuild...");

        let mut bm25_lock = {
            let bm25_lock = self.bm25_index.lock().await;
            bm25_lock.set_ingesting(true).await;
            bm25_lock
        };

        eprintln!("  [perf] BM25 index_vault starting...");
        let bm25_start = std::time::Instant::now();
        if let Err(e) = bm25_lock.index_vault(&*self.vault_state.lock().await).await {
            eprintln!(
                "  BM25 index_vault failed ({}); wiping corrupt index and retrying...",
                e
            );
            let tantivy_dir = self.kb_root.join(".knowledge-loom-index/tantivy");
            let _ = std::fs::remove_dir_all(&tantivy_dir);
            bm25_lock
                .index_vault(&*self.vault_state.lock().await)
                .await
                .map_err(|e| format!("BM25 retry failed: {}", e))?;
        }
        eprintln!("  [perf] BM25 index_vault: {:?}", bm25_start.elapsed());

        eprintln!("  [perf] Vector index_vault starting...");
        let vector_start = std::time::Instant::now();
        let (successful, _, _) = {
            let vector = self.vector_index.lock().await;
            let vault = self.vault_state.lock().await;
            let result = vector.index_vault(&vault, &self.embed_provider).await;
            match result {
                Ok(r) => r,
                Err(e) => {
                    eprintln!(
                        "  Vector index_vault failed ({}); wiping corrupt index and retrying...",
                        e
                    );
                    // Wipe turbovec files but preserve models dir
                    let tvim = self.kb_root.join(".knowledge-loom-index/turbovec.tvim");
                    let meta = self.kb_root.join(".knowledge-loom-index/turbovec_meta.bin");
                    let config = self
                        .kb_root
                        .join(".knowledge-loom-index/turbovec_config.bin");
                    let _ = std::fs::remove_file(&tvim);
                    let _ = std::fs::remove_file(&meta);
                    let _ = std::fs::remove_file(&config);
                    drop(vector);
                    let vector = self.vector_index.lock().await;
                    vector
                        .index_vault(&vault, &self.embed_provider)
                        .await
                        .map_err(|e| format!("Vector retry failed: {}", e))?
                }
            }
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
                state.update_file(
                    &relative,
                    mtime,
                    crate::chunks::parse_chunks(&content).len(),
                );
            }
        }
        drop(vault_lock);
        let _ = state.save();
        eprintln!("  [perf] TOTAL reindex_all: {:?}", total_start.elapsed());
        Ok("Reindex complete.".to_string())
    }

    async fn verify_embedding_count(
        &self,
        state: &ReindexState,
        vector: &TurbovecIndex,
    ) -> Result<(), String> {
        let total_expected: usize = state.files.values().map(|f| f.chunk_count).sum();
        if total_expected > 0 {
            let actual = vector.count().await;
            if actual > 0 && (actual as f64 / total_expected as f64) < 0.5 {
                eprintln!(
                    "  Index health: embedding count {} is <50% of expected {}; forcing full rebuild.",
                    actual, total_expected
                );
            } else {
                eprintln!(
                    "  Index health ok: {} embeddings (expected {}).",
                    actual, total_expected
                );
            }
        }
        Ok(())
    }

    async fn reindex_incremental(&self, state: &mut ReindexState) -> Result<String, String> {
        let start = std::time::Instant::now();
        let mut status_lines: Vec<String> = Vec::new();
        status_lines.push("Incremental reindex...".to_string());

        let files = {
            let guard = tokio::time::timeout(Duration::from_secs(10), self.vault_state.lock())
                .await
                .map_err(|_| "timeout acquiring vault lock for incremental reindex".to_string())?;
            guard.scan_files().await
        };

        eprintln!(
            "  {} files scanned, building comparison set...",
            files.len()
        );
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
        let mut removed_count = 0;
        for path in &tracked {
            if !disk_paths.contains(path.as_str()) {
                let full = self.kb_root.join(path);
                {
                    let mut bm25 = self.bm25_index.lock().await;
                    let _ = bm25.remove_document(&full).await;
                }
                {
                    let vector = self.vector_index.lock().await;
                    let _ = vector.remove_file(&full).await;
                }
                state.remove_file(path);
                removed_count += 1;
            }
        }
        if removed_count > 0 {
            status_lines.push(format!(
                "  {} deleted files removed from indexes",
                removed_count
            ));
            eprintln!("  {} deleted files removed from indexes", removed_count);
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

        if changed.is_empty() && removed_count == 0 {
            status_lines.push("No changes detected.".to_string());
            eprintln!("  No changes detected.");
            let _ = state.save();
            status_lines.push(format!(
                "Incremental reindex complete in {:?}",
                start.elapsed()
            ));
            return Ok(status_lines.join("\n"));
        }

        status_lines.push(format!(
            "  {} file(s) changed, {} deleted — reindexing",
            changed.len(),
            removed_count
        ));
        eprintln!(
            "  {} file(s) changed, {} deleted — reindexing",
            changed.len(),
            removed_count
        );

        let mut bm25 = self.bm25_index.lock().await;
        bm25.set_ingesting(true).await;
        for (fp, content) in &changed {
            bm25.index_file(fp, content)
                .await
                .map_err(|e| format!("BM25: {}", e))?;
            {
                self.vector_index
                    .lock()
                    .await
                    .index_file(fp, content, &self.embed_provider)
                    .await
                    .map_err(|e| e.to_string())?;
            }
            let rel = fp
                .strip_prefix(&self.kb_root)
                .unwrap_or(fp)
                .to_string_lossy()
                .to_string();
            let cur_mtime = std::fs::metadata(fp)
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            state.update_file(&rel, cur_mtime, crate::chunks::parse_chunks(content).len());
        }
        bm25.set_ingesting(false).await;
        drop(bm25);

        // Incremental graph update: update_file for each changed file
        if !changed.is_empty() {
            eprintln!("  Updating graph for {} changed files...", changed.len());
            let graph = self.graph_state.lock().await;
            for (fp, content) in &changed {
                let _ = graph.update_file(fp, content).await;
            }
            drop(graph);
        }

        let _ = state.save();
        let elapsed = start.elapsed();
        status_lines.push(format!(
            "  {} files scanned, {} changed, {} deleted",
            files.len(),
            changed.len(),
            removed_count
        ));
        status_lines.push(format!("Incremental reindex complete in {:?}", elapsed));
        eprintln!(
            "  Incremental reindex complete in {:?} (scanned {}, changed {}, deleted {})",
            elapsed,
            files.len(),
            changed.len(),
            removed_count
        );
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
        drop(vault_lock);

        let bm25_lock = self.bm25_index.lock().await;
        let bm25_docs = bm25_lock.doc_count().unwrap_or(0);
        drop(bm25_lock);

        status["bm25"] = serde_json::json!({
            "documents": bm25_docs,
            "index_path": self.kb_root.join(".knowledge-loom-index/tantivy").to_string_lossy().to_string()
        });

        let vector_lock = self.vector_index.lock().await;
        let vector_count = vector_lock.count().await;
        drop(vector_lock);

        status["embeddings"] = serde_json::json!({
            "vectors": vector_count,
            "index_path": self.kb_root.join(".knowledge-loom-index/turbovec.tvim").to_string_lossy().to_string()
        });

        let graph_lock = self.graph_state.lock().await;
        let node_count = graph_lock.node_map.lock().await.len();
        let edge_count = {
            let g = graph_lock.graph.lock().await;
            g.edge_count()
        };
        drop(graph_lock);

        status["graph"] = serde_json::json!({
            "nodes": node_count,
            "edges": edge_count,
            "index_path": self.kb_root.join(".knowledge-loom-index/graph.bin").to_string_lossy().to_string()
        });
        Ok(status)
    }
}
