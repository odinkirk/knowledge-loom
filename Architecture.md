# Architecture

This document provides a deep dive into the Knowledge Loom architecture, components, and internal workings.

## High-Level System Architecture

```mermaid
graph TB
    subgraph "User Interfaces"
        A[Claude] --> D[MCP Server]
        B[Codex] --> D
        C[OpenCode] --> D
        D --> E[Web UI :8080]
        F[Cursor] --> D
        G[Windsurf] --> D
        H[Zed] --> D
        I[Continue] --> D
        J[Kiro] --> D
    end
    
    subgraph "Knowledge Loom Core"
        D --> K[Search Engine]
        K --> L[BM25 Index]
        K --> M[Vector Store]
        K --> N[Graph Analytics]
    end
    
    subgraph "Storage Layer"
        L --> O[Tantivy Index]
        M --> P[SQLite + sqlite-vec]
        N --> Q[Petgraph Cache]
    end
    
    subgraph "Data Sources"
        R[Markdown Files] --> D
        S[Wikilinks] --> N
    end
    
    subgraph "Background Services"
        T[Daemon] --> D
        U[File Watcher] --> T
    end
```

## Search Flow (RRF Merging)

```mermaid
sequenceDiagram
    participant User
    participant MCP
    participant SearchEngine
    participant BM25
    participant Vector
    participant Graph
    
    User->>MCP: loom_search("query", top_k=10)
    MCP->>SearchEngine: search(query, top_k)
    
    Note over SearchEngine: Pre-compute shared values<br/>to avoid mutex contention
    SearchEngine->>SearchEngine: embed(query)
    SearchEngine->>SearchEngine: get_cached_pagerank()
    
    par Parallel Search Execution
        SearchEngine->>BM25: search_and_retrieve(query, top_k*2)
        SearchEngine->>Vector: search_similar(embedding, top_k*2)
        SearchEngine->>Graph: search_graph(query, top_k*2)
        SearchEngine->>Graph: search_graph_fused(embedding, pagerank, top_k*2)
    end
    
    BM25-->>SearchEngine: BM25 results with chunks
    Vector-->>SearchEngine: Semantic results
    Graph-->>SearchEngine: Graph neighbor results
    Graph-->>SearchEngine: Graph-fused results
    
    Note over SearchEngine: RRF Merging<br/>k=60 for all engines
    SearchEngine->>SearchEngine: Merge and rank results
    SearchEngine->>SearchEngine: Attach sections from BM25
    
    SearchEngine-->>MCP: Ranked results with sections
    MCP-->>User: Results with line_start/heading
```

## Data Processing Pipeline

```mermaid
graph LR
    A[Markdown Files] --> B[Vault Scanner]
    B --> C[Chunk Parser]
    
    C --> D[BM25 Indexer]
    C --> E[Embedding Provider]
    C --> F[Wikilink Extractor]
    
    D --> G[Tantivy Store]
    E --> H[Vector Store]
    F --> I[Graph Builder]
    
    I --> J[PageRank Calculator]
    I --> K[Community Detector]
    
    G --> L[Unified Search]
    H --> L
    J --> L
    K --> L
    
    L --> M[RRF Merged Results]
```

## Component Interaction

```mermaid
graph TB
    subgraph "MCP Server Layer"
        A[LoomServer] --> B[Tool Handler]
        B --> C[Search Engine]
        B --> D[Edit Manager]
        B --> E[Maintenance Manager]
    end
    
    subgraph "Search Engine Components"
        C --> F[BM25 Index]
        C --> G[Vector Index]
        C --> H[Graph State]
        C --> I[Embed Provider]
    end
    
    subgraph "Storage Backends"
        F --> J[Tantivy Index]
        G --> K[SQLite + sqlite-vec]
        H --> L[Binary Graph Cache]
    end
    
    subgraph "Edit Operations"
        D --> M[Vault State]
        D --> N[File Operations]
    end
    
    subgraph "Maintenance"
        E --> O[Index Health]
        E --> P[Reindexing]
    end
```

## Component Breakdown

### Vault Scanner (`vault.rs`)

The vault scanner is responsible for discovering and reading Markdown files from the knowledge base.

**Key Responsibilities:**
- File discovery with `.loomignore` support
- Markdown file filtering
- Content reading with error handling
- Path resolution and normalization

**Implementation Details:**
- Uses `walkdir` for efficient directory traversal
- Applies ignore patterns similar to `.gitignore`
- Handles file system errors gracefully
- Provides relative paths from KB_ROOT

### BM25 Index (`bm25.rs`)

The BM25 index provides fast full-text search using Tantivy.

**Key Responsibilities:**
- Tantivy-based full-text search
- Chunking strategy (2000 char max)
- Heading-aware chunk boundaries
- Relevance ranking with BM25 algorithm

**Chunking Strategy:**
```rust
pub const MAX_CHUNK_CHARS: usize = 2000;

pub fn truncate_at_whitespace(content: &str, max: usize) -> &str {
    if content.len() <= max {
        return content;
    }
    let slice = &content[..max];
    match slice.rfind(|c: char| c.is_whitespace()) {
        Some(pos) if pos > 0 => content[..pos].trim_end(),
        _ => &content[..max],
    }
}
```

**Heading-Aware Chunking:**
- Chunks respect heading boundaries
- Each chunk tracks its heading context
- Line numbers preserved for surgical editing
- Chunks stored with metadata for precise retrieval

### Vector Store (`index.rs`)

The vector store provides semantic similarity search using sqlite-vec.

**Key Responsibilities:**
- sqlite-vec for semantic similarity
- Heading-based chunking
- Cosine distance search
- Embedding upsert and removal

**Storage Schema:**
```sql
CREATE TABLE IF NOT EXISTS embeddings (
    id INTEGER PRIMARY KEY,
    path TEXT NOT NULL,
    heading TEXT,
    content TEXT NOT NULL,
    embedding BLOB NOT NULL,
    UNIQUE(path, heading)
);
```

**Search Algorithm:**
- Uses cosine distance via sqlite-vec
- Returns top-k most similar chunks
- Distance converted to similarity: `similarity = 1.0 - distance`

### Graph Engine (`graph.rs`)

The graph engine provides wikilink analysis and graph algorithms.

**Key Responsibilities:**
- Petgraph for wikilink analysis
- Wikilink extraction with regex
- Basename resolution for wikilink links
- PageRank computation (damping=0.85, 100 iterations)
- Community detection (connected components)
- Path finding (BFS-based)

**Wikilink Extraction:**
```rust
fn extract_wikilinks(&self, content: &str) -> HashSet<String> {
    let re = regex::Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]")
        .expect("hardcoded wikilink regex is valid");
    
    re.captures_iter(content)
        .filter_map(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
        .collect()
}
```

**Basename Resolution:**
- Supports Obsidian-style `[[note]]` links
- Resolves to `subdir/note.md` if unique
- Last-write wins on duplicate basenames
- TODO: Prefer closest-path on duplicate basenames

**PageRank Algorithm:**
```rust
pub async fn pagerank(&self, damping: f64, max_iter: usize) -> HashMap<String, f64> {
    // Initialize all nodes with score 1.0
    let mut scores: HashMap<String, f64> = node_map_lock.keys()
        .map(|name| (name.clone(), 1.0))
        .collect();
    
    // Iterate for max_iter iterations
    for _ in 0..max_iter {
        let mut new_scores: HashMap<String, f64> = /* ... */;
        
        // Distribute scores based on out-degree
        for (name, &node_idx) in node_map_lock.iter() {
            let out_edges: Vec<_> = graph_lock.edges(node_idx).collect();
            let out_degree = out_edges.len() as f64;
            
            if out_degree == 0.0 {
                // Dangling node: distribute proportionally
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
        
        // Add teleportation: (1 - damping) / N
        let teleport = (1.0 - damping) / node_count as f64;
        for score in new_scores.values_mut() {
            *score += teleport;
        }
        
        scores = new_scores;
    }
    
    scores
}
```

### Search Engine (`search.rs`)

The search engine unifies all search backends using RRF merging.

**Key Responsibilities:**
- RRF merging of all search backends
- Parallel execution via tokio::join!
- Mutex optimization with pre-computation
- Result ranking and section attachment
- Graph-fused search with PageRank boosting

**RRF Merging Algorithm:**
```rust
// RRF score calculation with k=60
let rrf = 1.0 / (60.0 + rank as f32 + 1.0);

// Applied to each search engine:
// - BM25: file-level using first occurrence
// - Semantic: file-level using first occurrence  
// - Graph: direct neighbor results
// - Graph-fused: PageRank-boosted semantic results
```

**Parallel Execution:**
```rust
// Pre-compute shared values to avoid mutex contention
let query_vec = {
    let embed = self.embed.lock().await;
    embed.embed(query).await
};
let cached_pagerank = {
    self.graph.lock().await.get_cached_analytics().await.0
};

// Run all searches in parallel
let (bm25_results, semantic_results, graph_results, fused_results) = tokio::join!(
    async { /* BM25 search */ },
    async { /* Vector search */ },
    async { /* Graph search */ },
    async { /* Graph-fused search */ }
);
```

**Graph-Fused Search:**
```rust
const PAGERANK_WEIGHT: f32 = 0.5;

pub async fn search_graph_fused_inner(
    &self,
    query_vec: &[f32],
    pagerank: &HashMap<String, f64>,
    top_k: usize,
) -> Result<Vec<String>, String> {
    // Vector similarity search
    let similar = vector.search_similar(query_vec, top_k * 2)?;
    
    // Re-rank: similarity * (1 + PAGERANK_WEIGHT * pagerank)
    let mut by_path: HashMap<String, f32> = HashMap::new();
    for (path, _heading, _content, similarity) in similar {
        let pr_key = path.strip_suffix(".md").unwrap_or(&path);
        let pr_boost = pagerank.get(pr_key).copied().unwrap_or(0.0) as f32;
        let score = similarity * (1.0 + Self::PAGERANK_WEIGHT * pr_boost);
        by_path.entry(path).or_insert(0.0);
        if score > *entry {
            *entry = score;
        }
    }
    
    // Sort and return top-k
    let mut ranked: Vec<(String, f32)> = by_path.into_iter().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    ranked.truncate(top_k);
    
    Ok(ranked.into_iter().map(|(path, _)| path).collect())
}
```

### Edit Manager (`edits.rs`)

The edit manager provides surgical file operations.

**Key Responsibilities:**
- Surgical file operations
- Heading-based navigation
- Line-level precision
- Vault-level management

**Key Operations:**
- `replace_lines` - In-place line replacement
- `insert_after_heading` - Insert content under a heading
- `append_to_file` - Append with blank-line separator
- `create_note` - Create new note with title
- `edit_note` - Replace full note content
- `link_notes` - Add wikilink between notes
- `move_note` - Move note to new location
- `delete_note` - Delete note from vault

### Maintenance Manager (`maintenance.rs`)

The maintenance manager handles index health and reindexing.

**Key Responsibilities:**
- Index health monitoring
- Incremental reindexing
- Cache management

**Health Monitoring:**
- Checks index integrity
- Monitors chunk counts
- Tracks last update times
- Reports storage usage

**Incremental Reindexing:**
- Only re-indexes changed files
- Preserves existing valid indexes
- Updates graph connections incrementally
- Maintains cache consistency

## Storage Architecture

### Directory Structure

```
.knowledge-loom/
├── bin/
│   └── loom              # Installed binary
└── loom-shell.sh         # Convenience script

.knowledge-loom-index/
├── bm25/                 # Tantivy index
├── embeddings.db         # SQLite + sqlite-vec
└── graph.bin             # Serialized petgraph
```

### Index Formats

**BM25 Index (Tantivy):**
- Inverted index for fast term lookup
- Stored fields: path, heading, content, line_start, line_end
- BM25 scoring with term frequency and document frequency
- Supports phrase queries and boolean operators

**Vector Store (SQLite + sqlite-vec):**
- Table: embeddings (id, path, heading, content, embedding)
- BLOB field for f32 embedding vectors
- Indexed on path for fast lookup
- sqlite-vec extension for cosine distance

**Graph Cache (Binary):**
- Serialized petgraph DiGraph
- Nodes: note names (without .md extension)
- Edges: wikilink connections with labels
- Bincode serialization for fast load/save

## Performance Characteristics

### Search Latency

| Operation | Latency (10k docs) | Scaling |
|-----------|-------------------|---------|
| BM25 search | ~10ms | Linear |
| Vector search | ~50ms | Linear |
| Graph search | ~100ms (cached) | O(V+E) |
| Graph-fused search | ~60ms | Linear |
| Unified search | ~150ms | Parallel |

### Indexing Performance

| Operation | Speed | Notes |
|-----------|-------|-------|
| Initial build | ~1000 docs/sec | Chunking overhead |
| Incremental update | ~50 docs/sec | Changed files only |
| Graph build | ~2s (10k nodes) | One-time cost |
| PageRank computation | ~500ms (10k nodes) | Cached after first run |

### Memory Usage

| Component | Memory (10k docs) | Scaling |
|-----------|------------------|---------|
| BM25 index | ~50MB | Linear |
| Vector store | ~100MB | Linear |
| Graph cache | ~30MB | Linear |
| Runtime overhead | ~20MB | Constant |
| **Total** | **~200MB** | **Linear** |

### Disk Usage

| Component | Disk (10k docs) | Scaling |
|-----------|----------------|---------|
| BM25 index | ~150MB | Linear |
| Vector store | ~300MB | Linear |
| Graph cache | ~50MB | Linear |
| **Total** | **~500MB** | **Linear** |

## Concurrency Model

### Mutex Strategy

The system uses Arc<Mutex<T>> for shared state with careful optimization:

```rust
// Pre-compute shared values to avoid mutex contention
let query_vec = {
    let embed = self.embed.lock().await;
    embed.embed(query).await
};
let cached_pagerank = {
    self.graph.lock().await.get_cached_analytics().await.0
};

// Now all searches can run in parallel without contention
let (bm25_results, semantic_results, graph_results, fused_results) = tokio::join!(
    /* ... */
);
```

### Parallel Execution

All search engines run concurrently using tokio::join!:

- BM25 search runs independently
- Vector search runs independently
- Graph search runs independently
- Graph-fused search runs independently
- Results merged after all complete

### Cache Invalidation

- PageRank cache invalidated on graph updates
- Community cache invalidated on graph updates
- BM25 index updated incrementally
- Vector store updated incrementally
- Graph updated incrementally

## Platform Integration

### MCP Protocol

Knowledge Loom implements the Model Context Protocol for seamless integration with coding agents.

**Server Capabilities:**
- 30+ tools across search, graph, edit, and maintenance
- Stdio transport for agent communication
- Tool descriptions with JSON schemas
- Error handling and validation

**Tool Categories:**
- Search: 3 tools (unified, file-specific, graph)
- Graph: 4 tools (ranking, connections, paths, themes)
- Navigation: 3 tools (list, outline, grep)
- Read: 2 tools (section, lines)
- Edit: 9 tools (replace, insert, append, create, edit, link, move, delete, preview)
- Maintenance: 2 tools (reindex, status)

### Platform-Specific Configuration

Each platform has unique configuration requirements:

**Claude:** `.mcp.json` with `mcpServers` object
**Codex:** `.codex/config.toml` with TOML format
**Cursor:** `.cursor/mcp.json` with `.cursorrules` instructions
**Windsurf:** `~/.codeium/windsurf/mcp_config.json` with `.windsurfrules`
**Zed:** Platform-specific settings with `context_servers` object
**Continue:** `~/.continue/config.json` with array format
**OpenCode:** `opencode.json` with `mcp` object and `AGENTS.md`
**Kiro:** `.kiro/settings/mcp.json` with `AGENTS.md`

## Extension Points

### Custom Embedding Providers

The system supports pluggable embedding providers:

```rust
pub trait EmbedProvider: Send + Sync {
    fn embed(&self, text: &str) -> Vec<f32>;
    fn dimension(&self) -> usize;
}

// Built-in providers:
// - LocalEmbedProvider: Built-in embeddings
// - OllamaEmbedProvider: Ollama integration
```

### Custom Search Engines

The search engine can be extended with additional backends:

1. Implement search function returning `Vec<(String, f32)>`
2. Add to parallel execution in `SearchEngine::search()`
3. Include in RRF merging with appropriate k value
4. Update tool definitions if exposing via MCP

### Custom Graph Algorithms

The graph engine supports custom algorithms:

1. Access graph via `GraphState`
2. Implement algorithm using petgraph API
3. Cache results if expensive
4. Expose via MCP tool if needed

## Security Considerations

### Local-Only Operation

- No network calls by default
- All data stays on local machine
- No cloud dependencies
- No telemetry or analytics

### File System Access

- Respects `.loomignore` patterns
- Only processes Markdown files
- Handles file system errors gracefully
- No arbitrary file execution

### Input Validation

- All MCP inputs validated
- File paths sanitized
- Query length limits enforced
- SQL injection prevented via parameterized queries

## Future Enhancements

### Planned Features

- [ ] LLM-decomposed smart search
- [ ] Advanced community detection (Louvain algorithm)
- [ ] Graph visualization export
- [ ] Real-time indexing with file watching
- [ ] Multi-vault support
- [ ] Advanced query syntax
- [ ] Result caching and memoization
- [ ] Performance profiling tools

### Performance Optimizations

- [ ] Streaming search results
- [ ] Lazy loading of large indexes
- [ ] Query optimization and caching
- [ ] Parallel chunk processing
- [ ] Memory-mapped file access

### Integration Enhancements

- [ ] Additional platform support
- [ ] Webhook notifications
- [ ] REST API for external access
- [ ] Plugin system for extensions
- [ ] Custom tool definitions
