use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::bm25::BM25Index;
use crate::index::VectorIndex;
use crate::embed::EmbedProviderEnum;
use crate::graph::GraphState;
use crate::brainjar::BrainJarWrapper;

pub struct SectionResult {
    pub heading: Option<String>,
    pub content: String,
    pub line_start: usize,
    pub line_end: usize,
    pub score: f32,
}

pub struct SearchResult {
    pub path: String,
    pub sections: Vec<SectionResult>,
    pub score: f32,
}

pub struct SearchEngine {
    pub bm25: Arc<Mutex<BM25Index>>,
    pub vector: Arc<Mutex<VectorIndex>>,
    pub embed: Arc<Mutex<EmbedProviderEnum>>,
    pub graph: Arc<Mutex<GraphState>>,
    pub brainjar: Arc<Mutex<BrainJarWrapper>>,
}

impl SearchEngine {
    #[allow(dead_code)]
    pub async fn new(kb_root: &str) -> Self {
        let bm25 = Arc::new(Mutex::new(BM25Index::new(kb_root).await));
        let vector = Arc::new(Mutex::new(VectorIndex::new(kb_root).await));
        let embed = Arc::new(Mutex::new(EmbedProviderEnum::new(kb_root).await));
        let graph = Arc::new(Mutex::new(GraphState::new(kb_root).await));
        let brainjar_path = std::env::var("BRAINJAR_PATH").ok();
        let brainjar = Arc::new(Mutex::new(BrainJarWrapper::new(brainjar_path)));

        Self { bm25, vector, embed, graph, brainjar }
    }

    pub fn from_components(
        bm25: Arc<Mutex<BM25Index>>,
        vector: Arc<Mutex<VectorIndex>>,
        embed: Arc<Mutex<EmbedProviderEnum>>,
        graph: Arc<Mutex<GraphState>>,
        brainjar: Arc<Mutex<BrainJarWrapper>>,
    ) -> Self {
        Self { bm25, vector, embed, graph, brainjar }
    }
    
    /// Combined search using RRF (Reciprocal Rank Fusion)
    /// Runs BM25, vector, graph, and BrainJar searches in parallel
    pub async fn search(&self, query: &str, top_k: usize) -> Vec<SearchResult> {
        use std::collections::{HashMap, HashSet};

        if query.is_empty() {
            return Vec::new();
        }

        let (bm25_results, semantic_results, graph_results, brainjar_results) = tokio::join!(
            async {
                let bm25 = self.bm25.lock().await;
                bm25.search_and_retrieve(query, top_k * 2).await
            },
            async {
                let embed = self.embed.lock().await;
                let query_embedding = embed.embed(query).await;
                let vector = self.vector.lock().await;
                vector.search_similar(&query_embedding, top_k * 2).await
            },
            self.search_graph(query, top_k * 2),
            self.search_brainjar(query, top_k * 2)
        );

        let bm25_results = bm25_results.unwrap_or_default();
        let semantic_results = semantic_results.unwrap_or_default();
        let graph_results = graph_results.unwrap_or_default();
        let brainjar_results = brainjar_results.unwrap_or_default();

        let mut rrf_scores: HashMap<String, f32> = HashMap::new();
        // path -> Vec<SectionResult> collected from BM25 chunks
        let mut sections_map: HashMap<String, Vec<SectionResult>> = HashMap::new();

        // BM25: use first-occurrence rank per file for RRF; collect all chunks as sections
        let mut bm25_file_seen: HashSet<String> = HashSet::new();
        for (rank, (score, chunk)) in bm25_results.iter().enumerate() {
            if !bm25_file_seen.contains(&chunk.path) {
                bm25_file_seen.insert(chunk.path.clone());
                let rrf = 1.0 / (60.0 + rank as f32 + 1.0);
                *rrf_scores.entry(chunk.path.clone()).or_insert(0.0) += rrf;
            }
            sections_map.entry(chunk.path.clone()).or_default().push(SectionResult {
                heading: chunk.heading.clone(),
                content: chunk.content.clone(),
                line_start: chunk.line_start,
                line_end: chunk.line_end,
                score: *score,
            });
        }

        // Semantic (k=60, file-level using first occurrence)
        let mut sem_file_seen: HashSet<String> = HashSet::new();
        for (rank, (path, _heading, _content, _sim)) in semantic_results.iter().enumerate() {
            if !sem_file_seen.contains(path) {
                sem_file_seen.insert(path.clone());
                let rrf = 1.0 / (60.0 + rank as f32 + 1.0);
                *rrf_scores.entry(path.clone()).or_insert(0.0) += rrf;
            }
        }

        // Graph (k=60)
        for (rank, path) in graph_results.iter().enumerate() {
            let rrf = 1.0 / (60.0 + rank as f32 + 1.0);
            *rrf_scores.entry(path.clone()).or_insert(0.0) += rrf;
        }

        // BrainJar (k=60)
        for (rank, path) in brainjar_results.iter().enumerate() {
            let rrf = 1.0 / (60.0 + rank as f32 + 1.0);
            *rrf_scores.entry(path.clone()).or_insert(0.0) += rrf;
        }

        // Sort sections within each file by score desc
        for sections in sections_map.values_mut() {
            sections.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        }

        // Build final results
        let mut results: Vec<SearchResult> = rrf_scores.iter()
            .map(|(path, &rrf)| {
                let sections = sections_map.remove(path).unwrap_or_default();
                SearchResult { path: path.clone(), sections, score: rrf }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Populate sections for files without BM25 chunks (graph/brainjar-only results)
        let paths_needing_sections: Vec<String> = results.iter()
            .filter(|r| r.sections.is_empty())
            .map(|r| r.path.clone())
            .collect();

        if !paths_needing_sections.is_empty() {
            let bm25 = self.bm25.lock().await;
            for path in paths_needing_sections {
                if let Ok(chunks) = bm25.get_chunks_for_path(&path).await {
                    let sections: Vec<SectionResult> = chunks.into_iter().map(|c| SectionResult {
                        heading: c.heading,
                        content: c.content,
                        line_start: c.line_start,
                        line_end: c.line_end,
                        score: 0.0,
                    }).collect();
                    // Find the result and set sections
                    if let Some(r) = results.iter_mut().find(|r| r.path == path) {
                        r.sections = sections;
                    }
                }
            }
        }

        results.truncate(top_k);
        results
    }

    async fn search_graph(&self, query: &str, top_k: usize) -> Result<Vec<String>, String> {
        // Search graph for nodes matching the query
        let graph = self.graph.lock().await;
        let neighbors = graph.search_graph(query).await;

        // If no direct neighbors, try BFS to find related nodes
        let mut results = if neighbors.is_empty() {
            let bfs_results = graph.bfs_connections(query, 2).await;
            bfs_results.into_iter().map(|(name, _depth)| name).collect()
        } else {
            neighbors
        };

        // Limit results
        results.truncate(top_k);
        Ok(results)
    }

    async fn search_brainjar(&self, query: &str, top_k: usize) -> Result<Vec<String>, String> {
        let brainjar = self.brainjar.lock().await;
        if !brainjar.is_available().await {
            return Ok(Vec::new());
        }

        // Get cached graph analytics
        let graph = self.graph.lock().await;
        let (pagerank, communities) = graph.get_cached_analytics().await;
        
        // Call BrainJar search tool with graph context
        let args = serde_json::json!({
            "query": query,
            "top_k": top_k,
            "graph_context": {
                "pagerank_scores": pagerank,
                "communities": communities
            }
        });
        
        match brainjar.call_tool("search", args).await {
            Ok(result) => {
                // Parse results from BrainJar response
                if let Some(results) = result.get("results").and_then(|r| r.as_array()) {
                    Ok(results.iter()
                        .filter_map(|r| r.get("path").and_then(|p| p.as_str()))
                        .map(|s| s.to_string())
                        .collect())
                } else {
                    Ok(Vec::new())
                }
            }
            Err(e) => {
                // Log error but don't fail the entire search
                eprintln!("BrainJar search error: {}", e);
                Ok(Vec::new())
            }
        }
    }
}