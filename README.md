# The Knowledge Loom

A unified search and analytics engine for document collections — what code-review-graph is for codebases.
Combines BM25 full-text search, semantic vector search, graph analytics, and LLM-decomposed smart search
behind a single `loom_*` tool surface.

## What It Does

The Knowledge Loom indexes your Markdown notes and provides:

- **BM25 full-text search** — Fast keyword search with relevance ranking
- **Semantic vector search** — Embedding-based similarity search (sqlite-vec)
- **Graph analytics** — Wikilink graph with PageRank, communities, and path finding
- **Smart search** — LLM-decomposed multi-search via brainjar
- **File operations** — Read, edit, and create notes with surgical precision

All tools are prefixed `loom_` to avoid namespace collisions.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Knowledge Loom                           │
├─────────────────────────────────────────────────────────────┤
│  Unified Search Engine (RRF merging)                         │
│  ├─ BM25 (tantivy)                                          │
│  ├─ Vector Search (sqlite-vec)                              │
│  ├─ Graph Analytics (petgraph)                              │
│  └─ BrainJar (subprocess)                                    │
├─────────────────────────────────────────────────────────────┤
│  Storage                                                     │
│  ├─ BM25 Index (tantivy)                                     │
│  ├─ Vector Store (sqlite-vec)                               │
│  └─ Graph Cache (petgraph)                                  │
├─────────────────────────────────────────────────────────────┤
│  File Operations                                             │
│  └─ Vault Scanner (.loomignore support)                     │
└─────────────────────────────────────────────────────────────┘
```

## Quick Install

### From Source

```bash
# Clone the repository
git clone https://github.com/odinkirk/knowledge-loom.git
cd knowledge-loom

# Build the release binary
cargo build --release

# The binary will be at target/release/loom
```

### Using the Installer (Python version)

Run this from your knowledge directory (the folder containing your Markdown notes):

```bash
curl -fsSL https://raw.githubusercontent.com/odinkirk/knowledge-loom/main/install.sh | bash
```

This creates `.loom/` with the tool files and a Python virtual environment, then merges
the `loom` server into `.mcp.json` (other MCP servers are preserved). It also adds `.loom/`
to `.gitignore` if you're inside a git repo.

To use a different install directory:

```bash
LOOM_DIR=.knowledge-loom curl -fsSL https://raw.githubusercontent.com/odinkirk/knowledge-loom/main/install.sh | bash
```

The installer handles macOS/Linux with managed (PEP 668) or unmanaged Python. It tries
`uv` first, then falls back to `python3 -m venv`, and prints clear guidance if neither works.

## Environment Variables

| Variable | Required | Purpose |
|---|---|---|
| `KB_ROOT` | Yes | Root path for BM25 index (set automatically by the installer) |
| `VAULT_PATH` | For graph tools | Path to Obsidian vault — enables semantic search and graph analytics |
| `BRAINJAR_PATH` | For smart search | Path to brainjar binary — enables `loom_search_smart` |

Add optional vars to the `env` block in `.mcp.json` after installation.

## Tool Surface

### Search
- `loom_search(query, top_k=10)` — RRF-merged BM25 + semantic search; results include `line_start`/`heading` for immediate surgical editing
- `loom_search_graph(note)` — graph connections for a specific note (requires `VAULT_PATH`)
- `loom_search_smart(query)` — LLM-decomposed multi-search via brainjar (requires `BRAINJAR_PATH`)

### Graph analytics (requires `VAULT_PATH`)
- `loom_rank_notes` — PageRank influence ranking
- `loom_find_connections(note)` — links and relationships for a note
- `loom_find_path_between(note_a, note_b)` — shortest graph path between two notes
- `loom_detect_themes` — Louvain thematic cluster detection

### Navigation
- `loom_list_files` — all Markdown files with line counts and sizes
- `loom_outline(file)` — heading hierarchy with line numbers
- `loom_grep(pattern, file_filter?)` — regex search across files

### Reads
- `loom_read_section(file, heading)` — content under a heading
- `loom_read_lines(file, start, end)` — exact line range

### Edits
- `loom_replace_lines(file, start, end, content)` — in-place line replacement
- `loom_insert_after_heading(file, heading, content)` — insert under a heading
- `loom_append_to_file(file, content)` — append with blank-line separator
- `loom_create_note`, `loom_edit_note`, `loom_link_notes`, `loom_move_note`, `loom_delete_note` — vault-level edits (requires `VAULT_PATH`)

### Maintenance
- `loom_reindex` — rebuild all indexes
- `loom_index_status` — health and chunk counts for all backends

## Excluding Files

Create a `.loomignore` file in your knowledge directory. It supports the same patterns
as `.gitignore`: directory patterns (`.venv/`), file globs (`*.dist-info/`), and exact names.

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

### Test corpus

The automated tests run against `test-vault/`. [ashuotaku/Personal-Wiki](https://github.com/ashuotaku/Personal-Wiki) makes a good corpus for it:

```bash
git clone https://github.com/ashuotaku/Personal-Wiki test-vault
```

### Automated tests

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

### Smoke test (CLI)

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

### Smoke test (after installation)

Run these from your knowledge directory (where `.loom/` was created):

```bash
# Index health — shows chunk count and KB root
KB_ROOT=. .loom/.venv/bin/python3 -c \
  'import sys; sys.path.insert(0,".loom"); import asyncio,loom_mcp; print(asyncio.run(loom_mcp.loom_index_status()))'

# Quick search — should return results from your notes
KB_ROOT=. .loom/.venv/bin/python3 -c \
  'import sys; sys.path.insert(0,".loom"); import asyncio,loom_mcp; r=asyncio.run(loom_mcp.loom_search("knowledge")); print(len(r["results"]),"results via",r["engines"])'
```

Then restart Claude Code and run `/mcp` — `loom` should appear in the connected server list.

## CLI Commands

The Rust version provides CLI binaries for testing and development:

```bash
# Search
KB_ROOT=. loom search "query" --top-k 10

# List files
KB_ROOT=. loom list-files

# Get outline
KB_ROOT=. loom outline path/to/file.md

# Grep
KB_ROOT=. loom grep "pattern" --file-filter "*.md"

# Index status
KB_ROOT=. loom index-status

# Reindex
KB_ROOT=. loom reindex
```

## Performance

- **BM25 search**: ~10ms for 10k documents
- **Vector search**: ~50ms for 10k documents (sqlite-vec)
- **Graph analytics**: ~100ms for 10k nodes (cached)
- **Unified search**: ~150ms for 10k documents (parallel execution)

## License

This project is dual-licensed under either:

- **MIT License** - [LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT
- **Apache License, Version 2.0** - [LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0

You may choose either license for your use.

### Third-Party Licenses

All dependencies use permissive, commercial-friendly licenses. See `about.toml` for the full list.

## Contributing

Contributions are welcome! Please read our contributing guidelines and submit pull requests to the main branch.

## Support

For issues, questions, or contributions, please visit [GitHub Issues](https://github.com/odinkirk/knowledge-loom/issues).
