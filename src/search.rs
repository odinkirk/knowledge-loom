use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::bm25::{BM25Index, ChunkDoc};
use crate::index::VectorIndex;
use crate::embed::EmbedProviderEnum;
use crate::graph::GraphState;
use crate::brainjar::BrainJarWrapper;

pub struct SearchResult {
    pub path: String,
    pub heading: Option<String>,
    pub content: String,
    pub score: f32,
    pub line_start: usize,
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
        // Run all searches in parallel
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
        
        // Apply RRF across all result sources
        let mut rrf_scores: HashMap<String, f32> = HashMap::new();
        let mut results_map: HashMap<String, SearchResult> = HashMap::new();
        
        // BM25 contribution (k=60)
        for (rank, (_score, chunk)) in bm25_results.iter().enumerate() {
            let path = chunk.path.clone();
            let content = chunk.content.clone();
            let rrf_score = 1.0 / (60.0 + rank as f32 + 1.0);

            *rrf_scores.entry(path.clone()).or_insert(0.0) += rrf_score;

            if !results_map.contains_key(&path) {
                results_map.insert(path.clone(), SearchResult {
                    path: path.clone(),
                    heading: chunk.heading.clone(),
                    content,
                    score: 0.0,
                    line_start: chunk.line_start,
                });
            }
        }

        // Semantic contribution (k=60)
        for (rank, (path, heading, content, _similarity)) in semantic_results.iter().enumerate() {
            let rrf_score = 1.0 / (60.0 + rank as f32 + 1.0);

            *rrf_scores.entry(path.clone()).or_insert(0.0) += rrf_score;

            if !results_map.contains_key(path) {
                results_map.insert(path.clone(), SearchResult {
                    path: path.clone(),
                    heading: heading.clone(),
                    content: content.clone(),
                    score: 0.0,
                    line_start: 1, // Default to first line for semantic results
                });
            }
        }

        // Graph contribution (k=60)
        for (rank, path) in graph_results.iter().enumerate() {
            let rrf_score = 1.0 / (60.0 + rank as f32 + 1.0);

            *rrf_scores.entry(path.clone()).or_insert(0.0) += rrf_score;

            if !results_map.contains_key(path) {
                results_map.insert(path.clone(), SearchResult {
                    path: path.clone(),
                    heading: None,
                    content: String::new(),
                    score: 0.0,
                    line_start: 1, // Default to first line for graph results
                });
            }
        }

        // BrainJar contribution (k=60)
        for (rank, path) in brainjar_results.iter().enumerate() {
            let rrf_score = 1.0 / (60.0 + rank as f32 + 1.0);

            *rrf_scores.entry(path.clone()).or_insert(0.0) += rrf_score;

            if !results_map.contains_key(path) {
                results_map.insert(path.clone(), SearchResult {
                    path: path.clone(),
                    heading: None,
                    content: String::new(),
                    score: 0.0,
                    line_start: 1, // Default to first line for BrainJar results
                });
            }
        }

        // Sort by RRF score and take top_k
        let mut results: Vec<SearchResult> = results_map.into_values()
            .map(|mut result| {
                result.score = *rrf_scores.get(&result.path).unwrap_or(&0.0);
                result
            })
            .collect();
        
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
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