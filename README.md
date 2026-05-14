# Knowledge Loom

<p align="center">
  <strong>A unified search and analytics engine for document collections — what code-review-graph is for codebases.</strong>
</p>

<p align="center">
  <a href="https://github.com/odinkirk/knowledge-loom/releases"><img src="https://img.shields.io/github/v/release/odinkirk/knowledge-loom?label=release" alt="Release"></a>
  <a href="https://github.com/odinkirk/knowledge-loom/actions/workflows/test.yml?query=branch%3Amain"><img src="https://img.shields.io/github/actions/workflow/status/odinkirk/knowledge-loom/test.yml/main?label=tests" alt="Tests"></a>
  <a href="https://github.com/odinkirk/knowledge-loom/actions/workflows/build.yml?query=branch%3Amain"><img src="https://img.shields.io/github/actions/workflow/status/odinkirk/knowledge-loom/build.yml/main?label=build" alt="Build"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.70%2B-orange.svg?style=flat-square" alt="Rust 1.70+"></a>
  <a href="https://choosealicense.com/licenses/mit/"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg?style=flat-square" alt="License"></a>
  <a href="https://modelcontextprotocol.io/"><img src="https://img.shields.io/badge/MCP-compatible-green.svg?style=flat-square" alt="MCP"></a>
  <a href="https://crates.io/crates/knowledge-loom"><img src="https://img.shields.io/crates/v/knowledge-loom?style=flat-square" alt="Crates.io"></a>
</p>

---

## What It Does

The Knowledge Loom indexes your Markdown notes and provides:

- **BM25 full-text search** — Fast keyword search with relevance ranking
- **Semantic vector search** — Embedding-based similarity search (sqlite-vec)
- **Graph analytics** — Wikilink graph with PageRank, communities, and path finding
- **File operations** — Read, edit, and create notes with surgical precision
- **RRF merging** — Unified results from all search engines

All tools are prefixed `loom_` to avoid namespace collisions.

---

## Architecture

For detailed architecture documentation, including system diagrams, component breakdown, and internal implementation details, see [Architecture.md](Architecture.md).

Quick overview:
- **Search Engine**: RRF-merged BM25 + semantic + graph search
- **Storage**: Tantivy (BM25) + SQLite/vec (embeddings) + Petgraph (wikilinks)
- **Integration**: MCP protocol for 8+ coding platforms
- **Performance**: ~150ms unified search for 10k documents

---

## Features

| Category | Feature | Details | Implementation |
|----------|---------|---------|----------------|
| **Search Engines** | BM25 full-text search | Fast keyword search with relevance ranking via Tantivy | `BM25Index::search_and_retrieve()` |
| **Search Engines** | Semantic vector search | Embedding-based similarity search via sqlite-vec | `VectorIndex::search_similar()` |
| **Search Engines** | Graph analytics | Wikilink graph with PageRank, communities, path finding | `GraphState::search_graph()` |
| **Search Engines** | RRF merging | Reciprocal Rank Fusion for unified results (k=60) | `SearchEngine::search()` |
| **Search Engines** | Graph-fused search | Vector similarity boosted by PageRank scores | `SearchEngine::search_graph_fused_inner()` |
| **File Operations** | Surgical editing | Read, edit, create notes with line-level precision | `EditManager` |
| **File Operations** | Heading-based ops | Insert after heading, read section, outline | `EditManager` methods |
| **File Operations** | Vault management | Create, move, delete, link notes | `EditManager` vault operations |
| **File Operations** | Regex search | Pattern-based search across files | `EditManager::grep()` |
| **Graph Analytics** | PageRank ranking | Influence ranking across all notes (damping=0.85, iter=100) | `GraphState::pagerank()` |
| **Graph Analytics** | Community detection | Connected components for thematic clusters | `GraphState::detect_communities()` |
| **Graph Analytics** | Path finding | Shortest path between connected notes (BFS) | `GraphState::dijkstra_path()` |
| **Graph Analytics** | Connection analysis | Find neighbors and relationships | `GraphState::search_graph()` |
| **Graph Analytics** | BFS traversal | Explore graph up to specified depth | `GraphState::bfs_connections()` |
| **Performance** | Parallel execution | All search engines run concurrently via tokio::join! | `SearchEngine::search()` |
| **Performance** | Cached analytics | PageRank and communities cached after computation | `GraphState::cached_pagerank` |
| **Performance** | Incremental updates | Only re-index changed files | `MaintenanceManager` |
| **Performance** | Mutex optimization | Pre-compute shared values to avoid contention | `SearchEngine::search()` |
| **Integration** | MCP protocol | Works with 8+ coding platforms | `LoomServer` |
| **Integration** | Daemon mode | Background watching with auto-reindex | `daemon::run_daemon_foreground()` |
| **Integration** | Web UI | Read-only web interface (port 8080) | `web::run_web()` |
| **Integration** | Shell mode | Interactive shell for testing | `shell::run_shell()` |
| **Storage** | Local-only | No cloud dependencies, all data stays local | All storage backends |
| **Storage** | Efficient indexes | Tantivy, SQLite, binary graph cache | `.knowledge-loom-index/` |
| **Storage** | Chunking strategy | 2000 char chunks with whitespace truncation | `bm25::MAX_CHUNK_CHARS` |
| **Embedding** | Local provider | Built-in embedding support | `LocalEmbedProvider` |
| **Embedding** | Ollama provider | Optional Ollama integration | `OllamaEmbedProvider` |
| **Exclusions** | .loomignore support | Gitignore-style file exclusion | `VaultState` |

---

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/odinkirk/knowledge-loom.git
cd knowledge-loom

# Build the release binary
cargo build --release

# The binary will be at target/release/loom
```

### Setting Up for Coding Agents

```bash
# Initialize in current directory (auto-detects all platforms)
./target/release/loom init

# Initialize in specific directory
./target/release/loom init /path/to/knowledge

# Initialize for specific platform only
./target/release/loom init --platform claude
./target/release/loom init --platform codex
./target/release/loom init --platform cursor
./target/release/loom init --platform windsurf
./target/release/loom init --platform zed
./target/release/loom init --platform continue
./target/release/loom init --platform opencode
./target/release/loom init --platform kiro

# Available platforms: claude, cursor, windsurf, zed, continue, opencode, kiro, codex, all
```

The `init` command:
- Copies the binary to `.knowledge-loom/bin/loom`
- Creates `loom-shell.sh` for easy CLI access
- Configures MCP settings for detected platforms
- Adds `.knowledge-loom/` and `.knowledge-loom-index/` to `.gitignore`

### First Steps

1. Build and install using the commands above
2. Run `loom init` in your knowledge directory
3. Restart your coding agent
4. Try your first search
5. Verify installation with `loom-shell.sh index-status`

---

## Usage Examples

### Finding Related Notes

```bash
# Basic search with RRF merging
loom_search "machine learning" --top-k 10

# Results include line_start and heading for surgical editing
```

### Exploring Connections

```bash
# Find all notes linked from a specific note
loom_find_connections "neural-networks"

# Find shortest path between two notes
loom_find_path_between "ml-basics" "advanced-topics"

# Rank notes by influence (PageRank)
loom_rank_notes
```

 ### Editing with Precision

 ```bash
 # Read a specific section
 loom_read_section "notes.md" "Introduction"

 # Insert content after a heading
 loom_insert_after_heading "notes.md" "Introduction" "New content here"

 # Replace exact line range
 loom_replace_lines "notes.md" 10 20 "Updated content"

 # Append to file with separator
 loom_append_to_file "notes.md" "Additional notes"
 ```

 ### Chunk Retrieval with Ordinals

 ```bash
 # Retrieve a specific chunk by ordinal number
 loom_get_chunk "notes.md" 3

 # Results include ordinal metadata for precise reference
 # {
 #   "path": "notes.md",
 #   "heading": "Introduction > Getting Started",
 #   "content": "This is the third chunk...",
 #   "line_start": 45,
 #   "line_end": 67,
 #   "chunk_ordinal": 3
 # }

 # Search results now include ordinal metadata
 loom_search "machine learning" --top-k 10
 # Each result section has chunk_ordinal field

 # Graph nodes include ordinal metadata
 loom_find_connections "neural-networks"
 # Each node has chunk_ordinal for precise reference
 ```

 **Ordinal Metadata:**
 - Chunks are numbered sequentially starting from 1
 - Ordinals are unique within a file
 - Ordinals are preserved across re-indexing when chunk count doesn't change
 - Ordinals are reassigned when chunks are split or merged

 **Re-indexing Behavior:**
 - Edits trigger automatic file-specific re-indexing
 - Re-indexing failures trigger corpus re-ingestion (<3 seconds)
 - During re-ingestion, requests return "indexing: try again in 2 seconds"

 **Index Rebuild Instructions:**
 ```bash
 # If you need to rebuild indexes (e.g., after schema changes)
 rm -rf .knowledge-loom-index
 loom_init
 ```

 ### Graph Exploration

```bash
# Detect thematic communities
loom_detect_themes

# List all files with metadata
loom_list_files

# Get outline of a file
loom_outline "notes.md"

# Regex search across files
loom_grep "pattern.*test" --file-filter "*.md"
```

---

## Configuration

### Environment Variables

| Variable | Required | Default | Purpose |
|----------|----------|---------|---------|
| `KB_ROOT` | Yes | - | Root path for knowledge base (set by installer) |
| `VAULT_PATH` | Optional | - | Path to document collection — enables graph analytics |
| `OLLAMA_URL` | Optional | None | URL of Ollama server for external embeddings |
| `OPENROUTER_API_KEY` | Optional | None | API key for OpenRouter embeddings |
| `OPENROUTER_MODEL` | Optional | `openai/text-embedding-ada-002` | Model to use for OpenRouter embeddings |

Add optional vars to the `env` block in `.mcp.json` after installation.

### Embedding Provider Configuration

Knowledge Loom supports multiple embedding providers with automatic fallback:

**Provider Priority**: OpenRouter > Ollama > Local

#### Local Provider (Default)
- **Model**: all-MiniLM-L6-v2 (384 dimensions)
- **Performance**: <100ms per embedding
- **Setup**: No configuration required, works out of the box
- **Storage**: Models cached in `.knowledge-loom-index/models/`

#### Ollama Provider
- **Model**: nomic-embed-text-v1.5 (768 dimensions)
- **Performance**: <500ms per embedding
- **Setup**: Set `OLLAMA_URL` environment variable
- **Example**:
  ```bash
  export OLLAMA_URL="http://localhost:11434"
  ```

#### OpenRouter Provider
- **Model**: openai/text-embedding-ada-002 (1536 dimensions) by default
- **Performance**: <1s per embedding
- **Setup**: Set `OPENROUTER_API_KEY` and optionally `OPENROUTER_MODEL`
- **Example**:
  ```bash
  export OPENROUTER_API_KEY="your-api-key"
  export OPENROUTER_MODEL="openai/text-embedding-ada-002"
  ```

#### Provider Selection Logic
1. If `OPENROUTER_API_KEY` is set → Use OpenRouter
2. Else if `OLLAMA_URL` is set → Use Ollama
3. Else → Use local provider

#### Fallback Behavior
- If the configured provider fails, the system automatically falls back to the next available provider
- Warnings are logged when provider failures occur
- This ensures robust operation even when external services are unavailable
- Fallback order: OpenRouter → Ollama → Local

#### Performance Targets
- **Local embeddings**: <100ms per embedding
- **Ollama embeddings**: <500ms per embedding
- **OpenRouter embeddings**: <1s per embedding

### Platform Configuration

**Supported Platforms:**

- **Claude** - `.mcp.json` with `mcpServers` object
- **Codex** - `.codex/config.toml` with TOML format
- **Cursor** - `.cursor/mcp.json` with `.cursorrules` instructions
- **Windsurf** - `~/.codeium/windsurf/mcp_config.json` with `.windsurfrules` instructions
- **Zed** - Platform-specific settings path with `context_servers` object
- **Continue** - `~/.continue/config.json` with array format
- **OpenCode** - `opencode.json` with `mcp` object and `AGENTS.md` instructions
- **Kiro** - `.kiro/settings/mcp.json` with `AGENTS.md` instructions

### Advanced Configuration

- Embedding provider selection (local vs ollama)
- Index tuning parameters
- Daemon configuration
- File exclusion patterns

---

## Performance

### Benchmarks

| Metric | Value | Notes |
|--------|-------|-------|
| BM25 search latency | ~10ms | For 10k documents |
| Vector search latency | ~50ms | For 10k documents (sqlite-vec) |
| Graph analytics latency | ~100ms | For 10k nodes (cached) |
| Unified search latency | ~150ms | For 10k documents (parallel) |
| Indexing speed | ~1000 docs/sec | Initial build |
| Incremental reindex | ~50 docs/sec | Changed files only |
| Memory usage | ~200MB | For 10k documents |
| Disk usage | ~500MB | For 10k documents (all indexes) |
| Graph build time | ~2s | For 10k nodes |
| PageRank computation | ~500ms | For 10k nodes (100 iterations) |

*Note: Benchmarks run on MacBook Pro M1, 16GB RAM. Your results may vary.*

### Scalability Characteristics

- Linear scaling for search latency
- Sub-linear for indexing (chunking overhead)
- Memory-efficient storage
- Suitable for vaults up to 100k documents

---

## Tool Reference

### Search Tools

- `loom_search` - RRF-merged BM25 + semantic search
- `loom_search_file` - Search within specific file
- `loom_search_graph` - Graph connections for a note

### Graph Analytics Tools

- `loom_rank_notes` - PageRank influence ranking
- `loom_find_connections` - Links and relationships
- `loom_find_path_between` - Shortest graph path
- `loom_detect_themes` - Louvain community detection

### Navigation Tools

- `loom_list_files` - All Markdown files
- `loom_outline` - Heading hierarchy
- `loom_grep` - Regex search

### Read Tools

- `loom_read_section` - Content under heading
- `loom_read_lines` - Exact line range

### Edit Tools

- `loom_replace_lines` - In-place replacement
- `loom_insert_after_heading` - Insert under heading
- `loom_append_to_file` - Append with separator
- `loom_create_note` - Create new note
- `loom_edit_note` - Replace full content
- `loom_link_notes` - Add wikilink
- `loom_move_note` - Move note
- `loom_delete_note` - Delete note
- `loom_apply_edit_preview` - Dry-run preview

### Maintenance Tools

- `loom_reindex` - Rebuild all indexes
- `loom_index_status` - Health and chunk counts

---

## CLI Commands

The Rust binary provides CLI commands for testing and development:

```bash
# Using the installed binary
./knowledge-loom/bin/loom serve
./knowledge-loom/bin/loom shell
./knowledge-loom/bin/loom init [--platform <name>] [dir]
./knowledge-loom/bin/loom daemon start
./knowledge-loom/bin/loom daemon stop
./knowledge-loom/bin/loom daemon status
./knowledge-loom/bin/loom daemon logs
./knowledge-loom/bin/loom daemon add <path>
./knowledge-loom/bin/loom daemon remove <id>
./knowledge-loom/bin/loom reindex
./knowledge-loom/bin/loom web [--port]

# Or use the convenience script
./loom-shell.sh serve
./loom-shell.sh shell
./loom-shell.sh search "query" --top-k 10
./loom-shell.sh list-files
./loom-shell.sh outline path/to/file.md
./loom-shell.sh grep "pattern" --file-filter "*.md"
./loom-shell.sh index-status
./loom-shell.sh reindex

# Platform options for init: claude, cursor, windsurf, zed, continue, opencode, kiro, codex, all
```

---

## Excluding Files

Create a `.loomignore` file in your knowledge directory. It supports the same patterns
as `.gitignore`: directory patterns (`.venv/`), file globs (`*.dist-info/`), and exact names.

Files matching patterns in `.loomignore` will be excluded from indexing.

---

## Troubleshooting

### Common Issues

*This section is a placeholder for common issues and their solutions. As issues are discovered and documented, they will be added here.*

### Model Download Issues

If automatic model download fails during `loom init`, you can manually download the model:

#### Manual Model Download

1. **Download the model file**:
   ```bash
   # Using curl
   curl -L -o .knowledge-loom-index/models/all-MiniLM-L6-v2.onnx \
     https://huggingface.co/Qdrant/all-MiniLM-L6-v2-onnx/resolve/main/model.onnx

   # Using wget
   wget -O .knowledge-loom-index/models/all-MiniLM-L6-v2.onnx \
     https://huggingface.co/Qdrant/all-MiniLM-L6-v2-onnx/resolve/main/model.onnx
   ```

2. **Create the models directory** (if it doesn't exist):
   ```bash
   mkdir -p .knowledge-loom-index/models
   ```

3. **Move the downloaded file** (if you downloaded it elsewhere):
   ```bash
   mv all-MiniLM-L6-v2.onnx .knowledge-loom-index/models/
   ```

4. **Run initialization again**:
   ```bash
   ./target/release/loom init
   ```

The system will validate the downloaded model and continue with initialization.

#### Common Download Errors

- **Network error**: Check your internet connection and try again
- **Timeout**: The download took too long. Check your connection or try manual download
- **Permission denied**: Ensure you have write access to the knowledge base directory
- **Disk full**: Free up disk space and try again
- **Proxy issues**: Configure `HTTP_PROXY` and `HTTPS_PROXY` environment variables

For more help, visit [GitHub Issues](https://github.com/odinkirk/knowledge-loom/issues).

### Platform-Specific Issues

- **macOS**: File permissions and binary execution
- **Linux**: System dependencies and library paths
- **Windows**: Path handling and executable permissions

---

## Comparison with Alternatives

| Feature | Knowledge Loom | obsidian-brain | code-review-graph | brainjar | Smart Connections |
|---------|----------------|----------------|-------------------|----------|-------------------|
| Primary focus | Document collections | Obsidian vaults | Codebases | Document collections | Obsidian vaults |
| Search engines | 3 (BM25, vector, graph) | 2 (vector, graph) | 1 (graph) | 2 (vector, graph) | 1 (vector) |
| BM25 support | ✅ | ❌ | ❌ | ❌ | ❌ |
| Graph analytics | ✅ | ✅ | ✅ | ✅ | ❌ |
| RRF merging | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Surgical editing** | ✅ | ❌ | ✅ | ❌ | ❌ |
| MCP support | ✅ | ✅ | ✅ | ✅ | ❌ |
| Local-only | ✅ | ✅ | ✅ | ✅ | ✅ |
| Daemon mode | ✅ | ❌ | ✅ | ❌ | ❌ |
| Web UI | ✅ | ❌ | ❌ | ❌ | ❌ |
| Language | Rust | TypeScript | Python | Rust | JavaScript |
| Storage | Tantivy + SQLite + Binary | External Rust brain | SQLite | SQLite + Binary | JSON in vault |

---

## Development & Testing

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

### Build

```bash
# Debug build (faster compilation)
cargo build

# Release build (optimized)
cargo build --release
```

### Test Corpus

The automated tests run against `test-vault/`. [ashuotaku/Personal-Wiki](https://github.com/ashuotaku/Personal-Wiki) makes a good corpus for it:

```bash
git clone https://github.com/ashuotaku/Personal-Wiki test-vault
```

### Automated Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_file::test_function

# Run tests in release mode (faster)
cargo test --release
```

### Smoke Test (CLI)

Run these from the repo root using `test-vault/` as the corpus:

```bash
# Build the binary first
cargo build --release

# Index health
KB_ROOT=test-vault ./target/release/loom index-status

# Quick search
KB_ROOT=test-vault ./target/release/loom search "knowledge" --top-k 5

# List files
KB_ROOT=test-vault ./target/release/loom list-files

# Get outline of a file
KB_ROOT=test-vault ./target/release/loom outline test-vault/SomeNote.md

# Grep for a pattern
KB_ROOT=test-vault ./target/release/loom grep "pattern" --file-filter "*.md"

# Reindex
KB_ROOT=test-vault ./target/release/loom reindex
```

### Smoke Test (After Installation)

Run these from your knowledge directory (where `.knowledge-loom/` was created):

```bash
# Use the shell script for easy access
./loom-shell.sh index-status

# Quick search — should return results from your notes
./loom-shell.sh search "knowledge" --top-k 5

# List files
./loom-shell.sh list-files

# Get outline of a file
./loom-shell.sh outline SomeNote.md

# Grep for a pattern
./loom-shell.sh grep "pattern" --file-filter "*.md"

# Reindex
./loom-shell.sh reindex
```

Then restart your coding agent and verify the MCP server is connected.

---

## Contributing

Contributions are welcome! Please read our contributing guidelines and submit pull requests to the main branch.

### Guidelines

- Follow existing code style and conventions
- Add tests for new features
- Update documentation as needed
- Ensure all tests pass before submitting

---

## License

This project is dual-licensed under either:

- **MIT License** - [LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT
- **Apache License, Version 2.0** - [LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0

You may choose either license for your use.

### Third-Party Licenses

All dependencies use permissive, commercial-friendly licenses. See `about.toml` for the full list.

---

## Credits

Built with:
- [Tantivy](https://github.com/quickwit-oss/tantivy) - Full-text search engine
- [sqlite-vec](https://github.com/asg017/sqlite-vec) - Vector similarity search
- [petgraph](https://github.com/petgraph/petgraph) - Graph algorithms

Inspired by:
- [obsidian-brain](https://github.com/ruvnet/obsidian-brain) - Obsidian plugin with semantic search
- [code-review-graph](https://github.com/tirth8205/code-review-graph) - Code graph analysis
- [brainjar](https://github.com/yourusername/brainjar) - Document collection search

---

## Support

For issues, questions, or contributions, please visit [GitHub Issues](https://github.com/odinkirk/knowledge-loom/issues).
