# Knowledge Loom Agent Guide

## Critical Setup
- **Never use git worktrees** - KB_ROOT is hardcoded to main repo root, making worktrees invisible to MCP tools
- **Always set KB_ROOT** - Required for all loom tools (defaults to current directory if unset)
- **Enable graph tools** - Set VAULT_PATH in .mcp.json to use obsidian-brain features (semantic search, graph analytics)
- **Enable smart search** - Set BRAINJAR_PATH in .mcp.json for loom_search_smart

## Essential Commands
- **Install**: `cargo build --release` (or use install.sh for Python version)
- **Test**: `cargo test`
- **Reindex**: `loom_reindex` (after external file changes)
- **Check index health**: `loom_index_status`
- **Restart server**: `/mcp restart` (to pick up external changes)

## Tool Usage Priority
1. **ALWAYS use code-review-graph MCP tools first** before Grep/Glob/Read for code exploration
2. **Use loom_* tools** for all knowledge base operations (search, read, edit, graph analytics)
3. **Prefer executable truth** - trust config/scripts over docs when they conflict

## Development Flow
- **Test corpus**: `git clone https://github.com/ashuotaku/Personal-Wiki test-vault`
- **Environment**: KB_ROOT=test-vault for development testing
- **Git workflow**: Feature branches only (`feature/<description>`), never commit to main

## Verification Shortcuts
- **Single test**: `cargo test test_file::test_function`
- **Index check**: `KB_ROOT=. cargo run --bin loom_index_status`
- **Quick search**: `KB_ROOT=. cargo run --bin loom search "test"`

<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan
<!-- SPECKIT END -->
