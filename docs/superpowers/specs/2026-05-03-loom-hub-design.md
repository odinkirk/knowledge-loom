# Design Spec: Knowledge Loom Hub MCP Server (obsidian-brain integration)

**Date**: 2026-05-03
**Status**: Approved for implementation

---

## Context

The Knowledge Loom is a modular, use-agnostic toolkit for managing Markdown document collections. The foundation (`kb_mcp.py`) provides BM25 search and surgical line-level edits. `obsidian-brain` (Node.js, SQLite) adds hybrid semantic+BM25 search, graph analytics (PageRank, Louvain), and vault-level editing. The goal is to expose both as a single coherent MCP tool surface — hard-coded in a hub server, not prompt-orchestrated via a skill, to avoid permeable boundaries. The design must also accommodate `brainjar` (Rust, entity graph, `--smart` mode) as a future third backend without interface churn.

---

## Architecture

A single Python MCP server (`loom_mcp.py`) at the project root is the **only entry in `.mcp.json`**. It owns two backend connections:

- **kb backend**: imports `KnowledgeBase` from `kb_mcp.py` in-process. Instantiated at startup with `KB_ROOT` from env.
- **obsidian-brain backend**: spawned as a child process at hub startup via the MCP Python SDK's stdio client. Hub manages its lifecycle (start, health, restart on crash). Activated when `VAULT_PATH` is set.
- **brainjar backend** (future): same subprocess/stdio-client pattern, activated when `BRAINJAR_PATH` is set.

**Environment variables:**

| Var | Required | Purpose |
|---|---|---|
| `KB_ROOT` | Yes | Root path for kb backend |
| `VAULT_PATH` | For obsidian-brain tools | Path to Obsidian vault / markdown folder |
| `OBSIDIAN_BRAIN_EMBEDDING` | No | Embedding backend; defaults to local transformers.js |
| `BRAINJAR_PATH` | Future | Activates brainjar backend |

`test-vault/` is added to `.gitignore`. Users populate it with a Logseq export or any markdown collection for local development.

---

## Tool Surface

All tools prefixed `loom_` to make the hub's namespace unambiguous.

### Search (intent-named, backend-agnostic)

| Tool | Backends | Notes |
|---|---|---|
| `loom_search(query, top_k=10)` | kb + obsidian-brain (+ brainjar when added) | RRF-merged unified search |
| `loom_search_graph(query)` | obsidian-brain graph tools | Entity/relationship traversal |
| `loom_search_smart(query)` | brainjar `--smart` | LLM query decomposition; stub until brainjar wired |

### Graph Analytics (obsidian-brain)

- `loom_rank_notes` — PageRank influence ranking
- `loom_find_connections(note)` — links and relationships for a note
- `loom_find_path_between(note_a, note_b)` — connection routes between notes
- `loom_detect_themes` — Louvain community / thematic clusters

### Navigation (kb backend)

- `loom_list_files` — all markdown files with line counts and sizes
- `loom_outline(file)` — heading hierarchy with line numbers
- `loom_grep(pattern, file_filter?)` — regex search across files

### Targeted Reads (kb backend)

- `loom_read_section(file, heading)` — content under a heading
- `loom_read_lines(file, start, end)` — exact line range

### Surgical Edits (kb backend)

- `loom_replace_lines(file, start, end, content)` — in-place line replacement
- `loom_insert_after_heading(file, heading, content)` — insert under a heading
- `loom_append_to_file(file, content)` — append with blank-line separator

### Vault-Level Edits (obsidian-brain)

- `loom_create_note(path, content)` — new markdown file in vault
- `loom_edit_note(path, instructions)` — modify existing note
- `loom_apply_edit_preview(path, instructions)` — preview before committing
- `loom_link_notes(note_a, note_b)` — create bidirectional link
- `loom_move_note(path, new_path)` — relocate note
- `loom_delete_note(path)` — remove note

### Maintenance

- `loom_reindex` — trigger obsidian-brain vault re-index
- `loom_index_status` — obsidian-brain index health and stats

---

## Data Flow: `loom_search`

```
loom_search(query, top_k=10)
    │
    ├─► kb.search(query)           # in-process BM25 → ranked list
    ├─► obsidian-brain: search()   # MCP subprocess call → ranked list
    └─► [brainjar: search() if BRAINJAR_PATH set]
    │
    ▼
RRF merge: score(d) = Σ 1/(60 + rank_i(d))
Deduplicate by file path (keep highest-ranked chunk per file)
Return top_k unified results
```

**Unified result schema:**
```json
{
  "file": "relative/path/to/note.md",
  "score": 0.87,
  "snippet": "...matched text...",
  "source": ["kb", "obsidian-brain"],
  "line": 42
}
```

RRF uses rank position only — raw scores from different engines are discarded, making the merge robust regardless of whether backends expose numeric scores.

All other tools are straight proxies: graph and vault-edit tools pass through to obsidian-brain unchanged; navigation and surgical-edit tools call the kb backend directly.

---

## Error Handling

**Backend unavailable at startup**: Hub starts regardless. Tools requiring the unavailable backend return `{"error": "obsidian-brain unavailable", "reason": "..."}`. Other backends continue operating normally.

**Partial search failure**: `loom_search` merges results from whatever engines responded. Response annotated with `"engines": ["kb"]` and `"warnings": ["obsidian-brain timeout"]}`. Graceful degradation, never hard failure.

**Misconfiguration**: Missing `KB_ROOT` or `VAULT_PATH` produces a clear error on startup and on first tool call. Hub logs active/inactive backends at startup.

**`loom_search_smart` before brainjar**: Returns `{"error": "not configured", "backend": "brainjar", "hint": "Set BRAINJAR_PATH to enable"}`.

---

## Testing

**Manual verification checklist:**
1. Drop markdown files into `test-vault/`, start hub, confirm startup log shows both backends active
2. Call `loom_search` — verify `"source"` field shows both engines contributing
3. Simulate obsidian-brain crash, call `loom_search` — verify graceful degradation with warning
4. Call `loom_rank_notes` — verify proxy to obsidian-brain works
5. Call `loom_replace_lines` — verify file changes and kb index rebuilds
6. Call `loom_search_smart` — verify clean "not configured" error

**`test_hub.py`**: A runnable smoke test script (no pytest required) that exercises one tool from each category and prints pass/fail. Kept at project root alongside `kb_mcp.py`.

---

## Files to Create / Modify

| File | Action | Notes |
|---|---|---|
| `loom_mcp.py` | Create | The hub server |
| `.mcp.json` | Modify | Replace existing entry with `loom` entry only |
| `.gitignore` | Modify | Add `test-vault/` |
| `test_hub.py` | Create | Smoke test script |
| `CLAUDE.md` | Modify | Update tool reference to `loom_*` tools, document env vars |
| `requirements.txt` | Modify | Add MCP client SDK dependency if needed |

`kb_mcp.py` is not modified — its `KnowledgeBase` class is imported directly by the hub.

---

## Open at Implementation Time

- Verify obsidian-brain's MCP response schema (score field presence/shape) before finalizing RRF input normalization
- Confirm MCP Python SDK supports spawning stdio subprocess as client (likely `mcp.client.stdio`)
- Clarify `loom_search_graph` mapping: obsidian-brain graph tools today, brainjar `--graph` mode when added — decide if they merge or remain distinct tools
- Decide whether `loom_read_note` (obsidian-brain's full-file read) should be exposed alongside `loom_read_section`/`loom_read_lines`, or if kb tools cover that sufficiently
