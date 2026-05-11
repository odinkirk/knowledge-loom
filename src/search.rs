use crate::bm25::BM25Index;
use crate::embed::EmbedProviderEnum;
use crate::graph::GraphState;
use crate::index::VectorIndex;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

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
    pub embed: Arc<EmbedProviderEnum>,
    pub graph: Arc<Mutex<GraphState>>,
}

impl SearchEngine {
    #[allow(dead_code)]
    pub async fn new(kb_root: &str) -> Self {
        let bm25 = Arc::new(Mutex::new(BM25Index::new(kb_root).await));
        let vector = Arc::new(Mutex::new(VectorIndex::new(kb_root).await));
        let embed = Arc::new(EmbedProviderEnum::new(kb_root));
        let graph = Arc::new(Mutex::new(GraphState::new(kb_root).await));
        Self {
            bm25,
            vector,
            embed,
            graph,
        }
    }

    pub fn from_components(
        bm25: Arc<Mutex<BM25Index>>,
        vector: Arc<Mutex<VectorIndex>>,
        embed: Arc<EmbedProviderEnum>,
        graph: Arc<Mutex<GraphState>>,
    ) -> Self {
        Self {
            bm25,
            vector,
            embed,
            graph,
        }
    }

    /// Combined search using RRF (Reciprocal Rank Fusion)
    /// Runs BM25, vector, graph, and graph-fused searches in parallel
    pub async fn search(&self, query: &str, top_k: usize) -> Vec<SearchResult> {
        use std::collections::HashSet;

        if query.is_empty() {
            return Vec::new();
        }

        // Pre-hoist shared computations before the join to avoid mutex contention:
        // both the semantic branch and graph-fused branch would otherwise race on
        // self.embed and self.graph locks, serializing what should be parallel work.
        let query_vec = {
            match self.embed.embed(query).await {
                Ok(vec) => vec,
                Err(e) => {
                    eprintln!("Failed to generate embedding for query: {}. Using empty vector as fallback.", e);
                    // Return empty vector as fallback - this will result in poor search results
                    // but prevents the entire search from failing
                    Vec::new()
                }
            }
        };
        let cached_pagerank = { self.graph.lock().await.get_cached_analytics().await.0 };

        let (bm25_results, semantic_results, graph_results, fused_results) = tokio::join!(
            async {
                let bm25 = self.bm25.lock().await;
                bm25.search_and_retrieve(query, top_k * 2).await
            },
            async {
                let vector = self.vector.lock().await;
                vector.search_similar(&query_vec, top_k * 2).await
            },
            self.search_graph(query, top_k * 2),
            self.search_graph_fused_inner(&query_vec, &cached_pagerank, top_k * 2)
        );

        let bm25_results = bm25_results.unwrap_or_default();
        let semantic_results = semantic_results.unwrap_or_default();
        let graph_results = graph_results.unwrap_or_default();
        let fused_results = fused_results.unwrap_or_default();

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
            sections_map
                .entry(chunk.path.clone())
                .or_default()
                .push(SectionResult {
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

        // Graph-fused (k=60)
        for (rank, path) in fused_results.iter().enumerate() {
            let rrf = 1.0 / (60.0 + rank as f32 + 1.0);
            *rrf_scores.entry(path.clone()).or_insert(0.0) += rrf;
        }

        // Sort sections within each file by score desc
        for sections in sections_map.values_mut() {
            sections.sort_by(|a, b| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        // Build final results
        let mut results: Vec<SearchResult> = rrf_scores
            .iter()
            .map(|(path, &rrf)| {
                let sections = sections_map.remove(path).unwrap_or_default();
                SearchResult {
                    path: path.clone(),
                    sections,
                    score: rrf,
                }
            })
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let paths_needing_sections: Vec<String> = results
            .iter()
            .filter(|r| r.sections.is_empty())
            .map(|r| r.path.clone())
            .collect();

        if !paths_needing_sections.is_empty() {
            let bm25 = self.bm25.lock().await;
            for path in paths_needing_sections {
                if let Ok(chunks) = bm25.get_chunks_for_path(&path).await {
                    let sections: Vec<SectionResult> = chunks
                        .into_iter()
                        .map(|c| SectionResult {
                            heading: c.heading,
                            content: c.content,
                            line_start: c.line_start,
                            line_end: c.line_end,
                            score: 0.0,
                        })
                        .collect();
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

    const PAGERANK_WEIGHT: f32 = 0.5;

    pub async fn search_graph_fused_inner(
        &self,
        query_vec: &[f32],
        pagerank: &HashMap<String, f64>,
        top_k: usize,
    ) -> Result<Vec<String>, String> {
        // Vector similarity search (vector lock only — no embed or graph lock)
        let similar = {
            let vector = self.vector.lock().await;
            vector
                .search_similar(query_vec, top_k * 2)
                .await
                .map_err(|e| e.to_string())?
        };

        if similar.is_empty() {
            return Ok(Vec::new());
        }

        // Re-rank: similarity * (1 + PAGERANK_WEIGHT * pagerank)
        // VectorIndex paths keep .md; PageRank keys strip it — align before lookup.
        let mut by_path: HashMap<String, f32> = HashMap::new();
        for (path, _heading, _content, similarity) in similar {
            let pr_key = path.strip_suffix(".md").unwrap_or(&path);
            let pr_boost = pagerank.get(pr_key).copied().unwrap_or(0.0) as f32;
            let score = similarity * (1.0 + Self::PAGERANK_WEIGHT * pr_boost);
            let entry = by_path.entry(path).or_insert(0.0);
            if score > *entry {
                *entry = score;
            }
        }

        let mut ranked: Vec<(String, f32)> = by_path.into_iter().collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked.truncate(top_k);

        Ok(ranked.into_iter().map(|(path, _)| path).collect())
    }
}
