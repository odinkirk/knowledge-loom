# Knowledge Loom

A structured knowledge repository with Markdown-based notes and an MCP server for intelligent access.

## Git Workflow

Worktrees are stored in `~/git/worktrees/` (global, shared across all projects).
Never commit directly to `main` — always use a feature worktree.

## Setup

### Install dependencies
```bash
pip install -r requirements.txt
```

### Configure Claude Code

The `.mcp.json` file is already configured to use the `knowledge-loom-repo` MCP server. After installing dependencies, restart Claude Code and run `/mcp` to verify the server is connected.

## Using the Knowledge Base

Use the `knowledge-loom-repo` MCP server for intelligent search and targeted reads:

- **`search(query)`** — BM25 full-text search across all notes with ranking
- **`list_files()`** — See all notes with line counts and sizes
- **`outline(file)`** — Browse a note's heading structure before reading
- **`read_section(file, heading)`** — Fetch content under a specific heading
- **`read_lines(file, start, end)`** — Read a precise line range
- **`grep(pattern)`** — Regex search across all files
- **`replace_lines(file, start, end, content)`** — Targeted in-place edits
- **`append_to_file(file, content)`** — Add content to a file

All tools return file paths and line numbers, enabling direct follow-up edits without re-reading.

## Index Freshness

- The index rebuilds automatically after any write operation
- External edits (outside Claude Code) require restarting the server to pick up (`/mcp restart`)
