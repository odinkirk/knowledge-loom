use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

static WIKILINK_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
static MDLINK_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

#[derive(Serialize, Deserialize, Debug)]
struct GraphData {
    nodes: Vec<String>,
    edges: Vec<(usize, usize, String)>,
}

pub struct GraphState {
    pub graph: Arc<Mutex<DiGraph<String, String>>>,
    pub node_map: Arc<Mutex<HashMap<String, NodeIndex>>>,
    pub kb_root: PathBuf,
    pub cached_pagerank: Arc<Mutex<Option<HashMap<String, f64>>>>,
    pub cached_communities: Arc<Mutex<Option<HashMap<String, Vec<String>>>>>,
}

impl GraphState {
    pub async fn new(kb_root: &str) -> Self {
        let kb_root_path = PathBuf::from(kb_root);
        let graph_path = kb_root_path.join(".knowledge-loom-index/graph.bin");

        // Try to load existing graph
        let (graph, node_map) = if graph_path.exists() {
            match std::fs::read(&graph_path) {
                Ok(data) => {
                    if let Ok(graph_data) = bincode::deserialize::<GraphData>(&data) {
                        // Reconstruct graph from data
                        let mut g = DiGraph::new();
                        let mut nm = HashMap::new();

                        // Add nodes
                        for node_name in &graph_data.nodes {
                            let idx = g.add_node(node_name.clone());
                            nm.insert(node_name.clone(), idx);
                        }

                        // Add edges
                        for (source_idx, target_idx, edge_label) in &graph_data.edges {
                            if source_idx < &graph_data.nodes.len()
                                && target_idx < &graph_data.nodes.len()
                            {
                                let source_node = NodeIndex::new(*source_idx);
                                let target_node = NodeIndex::new(*target_idx);
                                g.add_edge(source_node, target_node, edge_label.clone());
                            }
                        }

                        (g, nm)
                    } else {
                        (DiGraph::new(), HashMap::new())
                    }
                }
                Err(_) => (DiGraph::new(), HashMap::new()),
            }
        } else {
            (DiGraph::new(), HashMap::new())
        };

        Self {
            graph: Arc::new(Mutex::new(graph)),
            node_map: Arc::new(Mutex::new(node_map)),
            kb_root: kb_root_path,
            cached_pagerank: Arc::new(Mutex::new(None)),
            cached_communities: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn build_graph(
        &self,
        vault_state: &crate::vault::VaultState,
    ) -> Result<(), std::io::Error> {
        let files = vault_state.scan_files().await;
        let mut graph_lock = self.graph.lock().await;
        let mut node_map_lock = self.node_map.lock().await;

        graph_lock.clear();
        node_map_lock.clear();

        // First pass: create one node per file
        for file_path in &files {
            let file_name = self.path_to_node_name(file_path);
            let idx = graph_lock.add_node(file_name.clone());
            node_map_lock.insert(file_name, idx);
        }

        // Build basename → node_name index for Obsidian-style wikilink resolution.
        // Wikilinks like [[note]] must resolve to "subdir/note" if that is the only
        // node whose stem is "note". Last-write wins on duplicate basenames.
        // TODO: prefer closest-path on duplicate basenames
        let basename_map: HashMap<String, String> = node_map_lock
            .keys()
            .map(|name| {
                let bare = name.rsplit('/').next().unwrap_or(name).to_string();
                (bare, name.clone())
            })
            .collect();

        // Second pass: parse wikilinks from all chunks and add edges at file level
        for file_path in &files {
            if let Some(content) = vault_state.read_file(file_path).await {
                let source_node = self.path_to_node_name(file_path);
                let chunks = self.chunk_content(&content);
                for (_, chunk_content) in chunks {
                    let wikilinks = self.extract_wikilinks(&chunk_content);
                    for target in wikilinks {
                        let resolved = node_map_lock.get(&target).copied().or_else(|| {
                            basename_map
                                .get(&target)
                                .and_then(|full| node_map_lock.get(full))
                                .copied()
                        });
                        if let Some(target_idx) = resolved {
                            if let Some(&source_idx) = node_map_lock.get(&source_node) {
                                graph_lock.add_edge(source_idx, target_idx, "WIKILINK".to_string());
                            }
                        }
                    }
                }
            }
        }

        self.save_graph(&graph_lock).await?;
        drop(graph_lock);
        drop(node_map_lock);

        let pagerank = self.pagerank(0.85, 100).await;
        let communities = self.detect_communities().await;

        *self.cached_pagerank.lock().await = Some(pagerank);
        *self.cached_communities.lock().await = Some(communities);

        Ok(())
    }

    pub async fn update_file(
        &self,
        path: &std::path::Path,
        content: &str,
    ) -> Result<(), std::io::Error> {
        let file_name = self.path_to_node_name(path);
        let mut graph_lock = self.graph.lock().await;
        let node_map_lock = self.node_map.lock().await;

        if let Some(&source_idx) = node_map_lock.get(&file_name) {
            // Remove all outgoing edges from this node
            let outgoing: Vec<_> = graph_lock.edges(source_idx).map(|e| e.id()).collect();
            for edge_id in outgoing {
                graph_lock.remove_edge(edge_id);
            }

            // Build basename index for wikilink resolution (same two-step resolve as build_graph)
            let basename_map: HashMap<String, String> = node_map_lock
                .keys()
                .map(|name| {
                    let bare = name.rsplit('/').next().unwrap_or(name).to_string();
                    (bare, name.clone())
                })
                .collect();

            // Re-add edges from updated content (all chunks)
            let chunks = self.chunk_content(content);
            for (_, chunk_content) in chunks {
                let wikilinks = self.extract_wikilinks(&chunk_content);
                for target in wikilinks {
                    let resolved = node_map_lock.get(&target).copied().or_else(|| {
                        basename_map
                            .get(&target)
                            .and_then(|full| node_map_lock.get(full))
                            .copied()
                    });
                    if let Some(target_idx) = resolved {
                        graph_lock.add_edge(source_idx, target_idx, "WIKILINK".to_string());
                    }
                }
            }
        }

        self.save_graph(&graph_lock).await?;
        drop(graph_lock);
        drop(node_map_lock);

        *self.cached_pagerank.lock().await = None;
        *self.cached_communities.lock().await = None;

        Ok(())
    }

    fn path_to_node_name(&self, path: &Path) -> String {
        // Convert file path to a node name (relative to kb_root, without extension)
        let relative = path.strip_prefix(&self.kb_root).unwrap_or(path);
        let s = relative.to_string_lossy();
        s.strip_suffix(".md").unwrap_or(&s).to_string()
    }

    fn extract_wikilinks(&self, content: &str) -> HashSet<String> {
        let mut links = HashSet::new();

        // [[wikilink]] (with optional |alias)
        let re_double = WIKILINK_RE.get_or_init(|| {
            regex::Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]")
                .expect("hardcoded wikilink regex is valid")
        });
        for cap in re_double.captures_iter(content) {
            if let Some(m) = cap.get(1) {
                links.insert(m.as_str().trim().to_string());
            }
        }

        // Standard Markdown: [text](path.md)
        // Ignore URLs (http://, https://)
        let re_md = MDLINK_RE.get_or_init(|| {
            regex::Regex::new(r"\[[^\]]*\]\(([^)\s]+\.md)\)")
                .expect("hardcoded markdown link regex is valid")
        });
        for cap in re_md.captures_iter(content) {
            if let Some(m) = cap.get(1) {
                let target = m.as_str().trim();
                if target.starts_with("http://") || target.starts_with("https://") {
                    continue;
                }
                // Strip .md extension to match node naming convention
                let stripped = target.strip_suffix(".md").unwrap_or(target);
                links.insert(stripped.to_string());
            }
        }

        links
    }

    fn chunk_content(&self, content: &str) -> Vec<(Option<String>, String)> {
        let mut chunks = Vec::new();
        let mut current_heading: Option<String> = None;
        let mut current_content = String::new();

        for line in content.lines() {
            // Strip any heading level (H1-H6): 1-6 # chars followed by space
            if let Some(stripped) = line.strip_prefix("# ").or_else(|| {
                // Check for ##, ###, etc.
                let trimmed = line.trim_start_matches('#');
                if trimmed.starts_with(' ') && trimmed.len() < line.len() {
                    Some(trimmed.trim_start())
                } else {
                    None
                }
            }) {
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

    async fn save_graph(&self, graph: &DiGraph<String, String>) -> Result<(), std::io::Error> {
        let graph_path = self.kb_root.join(".knowledge-loom-index/graph.bin");
        let _ = std::fs::create_dir_all(graph_path.parent().unwrap());

        // Convert graph to serializable format
        let nodes: Vec<String> = graph.node_indices().map(|idx| graph[idx].clone()).collect();

        let edges: Vec<(usize, usize, String)> = graph
            .edge_indices()
            .filter_map(|idx| {
                graph
                    .edge_endpoints(idx)
                    .map(|(source, target)| (source.index(), target.index(), graph[idx].clone()))
            })
            .collect();

        let graph_data = GraphData { nodes, edges };

        let data = bincode::serialize(&graph_data).map_err(std::io::Error::other)?;
        std::fs::write(graph_path, data)
    }

    pub async fn search_graph(&self, note: &str) -> Vec<String> {
        let graph_lock = self.graph.lock().await;
        let node_map_lock = self.node_map.lock().await;

        let mut neighbors = Vec::new();

        if let Some(&node_idx) = node_map_lock.get(note) {
            for edge in graph_lock.edges(node_idx) {
                if let Some(node_name) = graph_lock.node_weight(edge.target()) {
                    neighbors.push(node_name.clone());
                }
            }
        }

        neighbors
    }

    pub async fn pagerank(&self, damping: f64, max_iter: usize) -> HashMap<String, f64> {
        let graph_lock = self.graph.lock().await;
        let node_map_lock = self.node_map.lock().await;

        let mut scores: HashMap<String, f64> = node_map_lock
            .keys()
            .map(|name| (name.clone(), 1.0))
            .collect();

        let node_count = node_map_lock.len();
        if node_count == 0 {
            return scores;
        }

        for _ in 0..max_iter {
            let mut new_scores: HashMap<String, f64> = node_map_lock
                .keys()
                .map(|name| (name.clone(), 0.0))
                .collect();

            for (name, &node_idx) in node_map_lock.iter() {
                let out_edges: Vec<_> = graph_lock.edges(node_idx).collect();
                let out_degree = out_edges.len() as f64;

                if out_degree == 0.0 {
                    // Dangling node: distribute proportionally to its score
                    let share = scores[name] * damping / node_count as f64;
                    for other_name in node_map_lock.keys() {
                        *new_scores.get_mut(other_name).unwrap() += share;
                    }
                } else {
                    let share = scores[name] * damping / out_degree;
                    for edge in out_edges {
                        if let Some(target_name) = graph_lock.node_weight(edge.target()) {
                            *new_scores.get_mut(target_name).unwrap() += share;
                        }
                    }
                }
            }

            // Teleportation: (1 - damping) / N added to every node
            let teleport = (1.0 - damping) / node_count as f64;
            for score in new_scores.values_mut() {
                *score += teleport;
            }

            scores = new_scores;
        }

        scores
    }

    pub async fn bfs_connections(&self, note: &str, max_depth: usize) -> Vec<(String, usize)> {
        let graph_lock = self.graph.lock().await;
        let node_map_lock = self.node_map.lock().await;

        let mut result = Vec::new();

        if let Some(&start_idx) = node_map_lock.get(note) {
            let mut visited = HashSet::new();
            let mut queue = std::collections::VecDeque::new();
            queue.push_back((start_idx, 0));
            visited.insert(start_idx);

            while let Some((node_idx, depth)) = queue.pop_front() {
                if depth > 0 {
                    if let Some(node_name) = graph_lock.node_weight(node_idx) {
                        result.push((node_name.clone(), depth));
                    }
                }

                if depth < max_depth {
                    for edge in graph_lock.edges(node_idx) {
                        let neighbor_idx = edge.target();
                        if visited.insert(neighbor_idx) {
                            queue.push_back((neighbor_idx, depth + 1));
                        }
                    }
                }
            }
        }

        result
    }

    pub async fn dijkstra_path(&self, note_a: &str, note_b: &str) -> Vec<String> {
        let graph_lock = self.graph.lock().await;
        let node_map_lock = self.node_map.lock().await;

        if let (Some(&start_idx), Some(&end_idx)) =
            (node_map_lock.get(note_a), node_map_lock.get(note_b))
        {
            // For simplicity, we'll use BFS since we don't have weighted edges
            let mut visited = HashSet::new();
            let mut queue = std::collections::VecDeque::new();
            let mut parent: HashMap<NodeIndex, NodeIndex> = HashMap::new();

            queue.push_back(start_idx);
            visited.insert(start_idx);

            while let Some(node_idx) = queue.pop_front() {
                if node_idx == end_idx {
                    // Reconstruct path
                    let mut path = Vec::new();
                    let mut current = end_idx;
                    while let Some(&p) = parent.get(&current) {
                        if let Some(name) = graph_lock.node_weight(current) {
                            path.push(name.clone());
                        }
                        current = p;
                    }
                    if let Some(name) = graph_lock.node_weight(start_idx) {
                        path.push(name.clone());
                    }
                    path.reverse();
                    return path;
                }

                for edge in graph_lock.edges(node_idx) {
                    let neighbor_idx = edge.target();
                    if visited.insert(neighbor_idx) {
                        parent.insert(neighbor_idx, node_idx);
                        queue.push_back(neighbor_idx);
                    }
                }
            }
        }

        Vec::new()
    }

    pub async fn detect_communities(&self) -> HashMap<String, Vec<String>> {
        let graph_lock = self.graph.lock().await;
        let node_map_lock = self.node_map.lock().await;

        // Simple community detection: group by connected components
        // In a real implementation, we'd use the Louvain algorithm
        let mut communities: HashMap<usize, Vec<String>> = HashMap::new();
        let mut visited = HashSet::new();
        let mut component_id = 0;

        for &node_idx in node_map_lock.values() {
            if visited.contains(&node_idx) {
                continue;
            }

            // BFS to find all nodes in this component
            let mut queue = std::collections::VecDeque::new();
            queue.push_back(node_idx);
            visited.insert(node_idx);

            while let Some(current) = queue.pop_front() {
                if let Some(name) = graph_lock.node_weight(current) {
                    communities
                        .entry(component_id)
                        .or_default()
                        .push(name.clone());
                }

                for edge in graph_lock.edges(current) {
                    let neighbor = edge.target();
                    if visited.insert(neighbor) {
                        queue.push_back(neighbor);
                    }
                }

                // Also check incoming edges
                for edge in graph_lock.edges_directed(current, petgraph::Direction::Incoming) {
                    let neighbor = edge.source();
                    if visited.insert(neighbor) {
                        queue.push_back(neighbor);
                    }
                }
            }

            component_id += 1;
        }

        // Convert to format: community_name -> member_names
        communities
            .into_iter()
            .map(|(id, members)| (format!("Community_{}", id), members))
            .collect()
    }

    pub async fn get_cached_analytics(
        &self,
    ) -> (HashMap<String, f64>, HashMap<String, Vec<String>>) {
        let pagerank = self
            .cached_pagerank
            .lock()
            .await
            .clone()
            .unwrap_or_default();
        let communities = self
            .cached_communities
            .lock()
            .await
            .clone()
            .unwrap_or_default();
        (pagerank, communities)
    }
}
