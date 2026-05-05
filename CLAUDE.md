# Knowledge Loom

A structured knowledge repository with Markdown-based notes and an MCP server for intelligent access.

## Git Workflow

Never commit directly to `main` — use feature branches (`feature/<description>`).

Work directly in the main repo checkout (`/Users/odinkirk/git/knowledge-loom`).
**Do not use git worktrees for this project.** The MCP server's `KB_ROOT` is hardcoded
to the main repo root, so worktrees at a different path are invisible to all MCP tools
(`search`, `outline`, `read_section`, etc.), making them counterproductive.

## Setup

### Install dependencies
```bash
pip install -r requirements.txt
```

### Configure Claude Code

The `.mcp.json` file is already configured to use the `loom` MCP server. After installing dependencies, restart Claude Code and run `/mcp` to verify the server is connected.

## Using the Knowledge Loom

The `loom` MCP server is the single entry point — it unifies the kb (BM25) and
obsidian-brain (semantic/graph) backends behind a `loom_*` tool surface.

### Required environment variables

| Var | Required | Purpose |
|---|---|---|
| `KB_ROOT` | Yes | Root path for BM25 index |
| `VAULT_PATH` | For graph/vault tools | Path to Obsidian vault or markdown folder |
| `BRAINJAR_PATH` | For smart search | Path to brainjar binary — enables `loom_search_smart` |

Set `VAULT_PATH` in the `env` block of `.mcp.json` to enable obsidian-brain tools.
On first run, obsidian-brain downloads a ~34 MB local embedding model.

### Search tools

- **`loom_search(query, top_k=10)`** — RRF-merged BM25 + semantic search; results include
  `line_start`/`heading` metadata for immediate surgical editing
- **`loom_search_graph(note)`** — entity/relationship traversal via obsidian-brain graph
- **`loom_search_smart(query)`** — LLM-decomposed search (brainjar; stub until configured)

### Graph analytics (requires VAULT_PATH)

- **`loom_rank_notes`** — PageRank influence ranking
- **`loom_find_connections(note)`** — links and relationships for a note
- **`loom_find_path_between(note_a, note_b)`** — shortest graph path between notes
- **`loom_detect_themes`** — Louvain thematic cluster detection

### Navigation

- **`loom_list_files`** — all Markdown files with line counts and sizes
- **`loom_outline(file)`** — heading hierarchy with line numbers
- **`loom_grep(pattern, file_filter?)`** — regex search across files

### Targeted reads

- **`loom_read_section(file, heading)`** — content under a heading (substring match)
- **`loom_read_lines(file, start, end)`** — exact line range

### Surgical edits (kb; line-precise)

- **`loom_replace_lines(file, start, end, content)`** — in-place line replacement
- **`loom_insert_after_heading(file, heading, content)`** — insert under a heading
- **`loom_append_to_file(file, content)`** — append with blank-line separator

### Vault-level edits (requires VAULT_PATH)

- **`loom_create_note`**, **`loom_edit_note`**, **`loom_apply_edit_preview`**
- **`loom_link_notes`**, **`loom_move_note`**, **`loom_delete_note`**

### Maintenance

- **`loom_reindex`** — rebuild both kb and obsidian-brain indexes
- **`loom_index_status`** — health and chunk counts for all backends

### Brainjar

Set `BRAINJAR_PATH` in `.mcp.json` env to the path of the brainjar binary.
Enables `loom_search_smart` (LLM-decomposed multi-search via brainjar's `--smart` mode).

## Index Freshness

- The index rebuilds automatically after any write operation
- External edits (outside Claude Code) require restarting the server to pick up (`/mcp restart`)

## Testing

Here is the command to run the automated tests:

```bash
source .venv/bin/activate && .venv/bin/python -m pip install rank_bm25 mcp pytest pytest-asyncio && .venv/bin/python -m pytest -q 2>&1
```

<!-- code-review-graph MCP tools -->
## MCP Tools: code-review-graph

**IMPORTANT: This project has a knowledge graph. ALWAYS use the
code-review-graph MCP tools BEFORE using Grep/Glob/Read to explore
the codebase.** The graph is faster, cheaper (fewer tokens), and gives
you structural context (callers, dependents, test coverage) that file
scanning cannot.

### When to use graph tools FIRST

- **Exploring code**: `semantic_search_nodes` or `query_graph` instead of Grep
- **Understanding impact**: `get_impact_radius` instead of manually tracing imports
- **Code review**: `detect_changes` + `get_review_context` instead of reading entire files
- **Finding relationships**: `query_graph` with callers_of/callees_of/imports_of/tests_for
- **Architecture questions**: `get_architecture_overview` + `list_communities`

Fall back to Grep/Glob/Read **only** when the graph doesn't cover what you need.

### Key Tools

| Tool | Use when |
|------|----------|
| `detect_changes` | Reviewing code changes — gives risk-scored analysis |
| `get_review_context` | Need source snippets for review — token-efficient |
| `get_impact_radius` | Understanding blast radius of a change |
| `get_affected_flows` | Finding which execution paths are impacted |
| `query_graph` | Tracing callers, callees, imports, tests, dependencies |
| `semantic_search_nodes` | Finding functions/classes by name or keyword |
| `get_architecture_overview` | Understanding high-level codebase structure |
| `refactor_tool` | Planning renames, finding dead code |

### Workflow

1. The graph auto-updates on file changes (via hooks).
2. Use `detect_changes` for code review.
3. Use `get_affected_flows` to understand impact.
4. Use `query_graph` pattern="tests_for" to check coverage.
