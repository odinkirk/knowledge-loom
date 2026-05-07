use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};

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
        let graph = if graph_path.exists() {
            match std::fs::read(&graph_path) {
                Ok(data) => {
                    if let Ok(graph_data) = bincode::deserialize::<GraphData>(&data) {
                        // Reconstruct graph from data
                        let mut g = DiGraph::new();
                        let mut node_map = HashMap::new();
                        
                        // Add nodes
                        for node_name in &graph_data.nodes {
                            let idx = g.add_node(node_name.clone());
                            node_map.insert(node_name.clone(), idx);
                        }
                        
                        // Add edges
                        for (source_idx, target_idx, edge_label) in &graph_data.edges {
                            if source_idx < &graph_data.nodes.len() && target_idx < &graph_data.nodes.len() {
                                let source_node = NodeIndex::new(*source_idx);
                                let target_node = NodeIndex::new(*target_idx);
                                g.add_edge(source_node, target_node, edge_label.clone());
                            }
                        }
                        
                        g
                    } else {
                        DiGraph::new()
                    }
                }
                Err(_) => DiGraph::new(),
            }
        } else {
            DiGraph::new()
        };
        
        Self {
            graph: Arc::new(Mutex::new(graph)),
            node_map: Arc::new(Mutex::new(HashMap::new())),
            kb_root: kb_root_path,
            cached_pagerank: Arc::new(Mutex::new(None)),
            cached_communities: Arc::new(Mutex::new(None)),
        }
    }
    
    pub async fn build_graph(&self, vault_state: &crate::vault::VaultState) -> Result<(), std::io::Error> {
        let files = vault_state.scan_files().await;
        let mut graph_lock = self.graph.lock().await;
        let mut node_map_lock = self.node_map.lock().await;
        
        // Clear existing graph
        graph_lock.clear();
        node_map_lock.clear();
        
        // First pass: add all files as nodes
        for file_path in &files {
            let node_name = self.path_to_node_name(file_path);
            let idx = graph_lock.add_node(node_name.clone());
            node_map_lock.insert(node_name, idx);
        }
        
        // Second pass: parse wikilinks and add edges
        for file_path in &files {
            if let Some(content) = vault_state.read_file(file_path).await {
                let source_node = self.path_to_node_name(file_path);
                let wikilinks = self.extract_wikilinks(&content);
                
                for target in wikilinks {
                    if let Some(&target_idx) = node_map_lock.get(&target) {
                        if let Some(&source_idx) = node_map_lock.get(&source_node) {
                            graph_lock.add_edge(source_idx, target_idx, "WIKILINK".to_string());
                        }
                    }
                }
            }
        }
        
        // Save graph
        self.save_graph(&graph_lock).await?;
        
        // Drop locks before computing analytics
        drop(graph_lock);
        drop(node_map_lock);
        
        // Compute and cache analytics
        let pagerank = self.pagerank(0.85, 100).await;
        let communities = self.detect_communities().await;
        
        *self.cached_pagerank.lock().await = Some(pagerank);
        *self.cached_communities.lock().await = Some(communities);
        
        Ok(())
    }
    
    fn path_to_node_name(&self, path: &Path) -> String {
        // Convert file path to a node name (relative to kb_root, without extension)
        let relative = path.strip_prefix(&self.kb_root).unwrap_or(path);
        relative.to_string_lossy().to_string().replace(".md", "")
    }
    
    fn extract_wikilinks(&self, content: &str) -> HashSet<String> {
        let mut links = HashSet::new();
        
        // Simple wikilink regex: [[Target]] or [[Target|Display]]
        let re = regex::Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]").unwrap();
        
        for cap in re.captures_iter(content) {
            if let Some(target) = cap.get(1) {
                links.insert(target.as_str().trim().to_string());
            }
        }
        
        links
    }
    
    async fn save_graph(&self, graph: &DiGraph<String, String>) -> Result<(), std::io::Error> {
        let graph_path = self.kb_root.join(".knowledge-loom-index/graph.bin");
        let _ = std::fs::create_dir_all(graph_path.parent().unwrap());
        
        // Convert graph to serializable format
        let nodes: Vec<String> = graph.node_indices()
            .map(|idx| graph[idx].clone())
            .collect();
        
        let edges: Vec<(usize, usize, String)> = graph.edge_indices()
            .map(|idx| {
                let (source, target) = graph.edge_endpoints(idx).unwrap();
                (source.index(), target.index(), graph[idx].clone())
            })
            .collect();
        
        let graph_data = GraphData { nodes, edges };
        
        let data = bincode::serialize(&graph_data).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(graph_path, data)
    }
    
    pub async fn search_graph(&self, note: &str) -> Vec<String> {
        let graph_lock = self.graph.lock().await;
        let node_map_lock = self.node_map.lock().await;
        
        let mut neighbors = Vec::new();
        
        if let Some(&node_idx) = node_map_lock.get(note) {
            for edge in graph_lock.edges(node_idx) {
                let target_idx = edge.target();
                if let Some(node_name) = graph_lock.node_weight(target_idx) {
                    neighbors.push(node_name.clone());
                }
            }
        }
        
        neighbors
    }
    
    pub async fn pagerank(&self, damping: f64, max_iter: usize) -> HashMap<String, f64> {
        let graph_lock = self.graph.lock().await;
        let node_map_lock = self.node_map.lock().await;
        
        let mut scores: HashMap<String, f64> = node_map_lock.keys()
            .map(|name| (name.clone(), 1.0))
            .collect();
        
        let node_count = node_map_lock.len();
        if node_count == 0 {
            return scores;
        }
        
        for _ in 0..max_iter {
            let mut new_scores = HashMap::new();
            
            for (name, &node_idx) in node_map_lock.iter() {
                let out_edges: Vec<_> = graph_lock.edges(node_idx).collect();
                let out_degree = out_edges.len() as f64;
                
                if out_degree == 0.0 {
                    // Teleport to random node
                    for (other_name, _) in node_map_lock.iter() {
                        *new_scores.entry(other_name.clone()).or_insert(0.0) += 1.0 / node_count as f64 * damping;
                    }
                } else {
                    let share = scores[name] * (1.0 - damping) / out_degree;
                    
                    for edge in out_edges {
                        let target_idx = edge.target();
                        if let Some(target_name) = graph_lock.node_weight(target_idx) {
                            *new_scores.entry(target_name.clone()).or_insert(0.0) += share;
                        }
                    }
                }
            }
            
            // Add teleportation factor
            for (_name, score) in &mut new_scores {
                *score += (1.0 - damping) / node_count as f64;
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
        
        if let (Some(&start_idx), Some(&end_idx)) = (node_map_lock.get(note_a), node_map_lock.get(note_b)) {
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
                    communities.entry(component_id).or_default().push(name.clone());
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
        communities.into_iter()
            .map(|(id, members)| (format!("Community_{}", id), members))
            .collect()
    }
    
    pub async fn get_cached_analytics(&self) -> (HashMap<String, f64>, HashMap<String, Vec<String>>) {
        let pagerank = self.cached_pagerank.lock().await.clone().unwrap_or_default();
        let communities = self.cached_communities.lock().await.clone().unwrap_or_default();
        (pagerank, communities)
    }
}