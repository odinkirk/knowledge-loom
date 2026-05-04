#!/usr/bin/env python3
"""
Loom Hub MCP Server

Unified MCP tool surface over kb (BM25), obsidian-brain (semantic/graph),
and brainjar (future LLM-powered search).
All tools prefixed loom_ to avoid namespace collisions.

Environment:
  KB_ROOT                  — required; root path for BM25 kb index
  VAULT_PATH               — required for obsidian-brain tools; path to vault
  OBSIDIAN_BRAIN_EMBEDDING — optional; embedding backend (default: local transformers.js)
  BRAINJAR_PATH            — optional; enables brainjar backend for loom_search_smart
"""

import os
import logging
from collections import defaultdict
from contextlib import asynccontextmanager
from pathlib import Path
from typing import Any, Optional

from mcp import ClientSession
from mcp.client.stdio import stdio_client, StdioServerParameters
from mcp.server.fastmcp import FastMCP
from anyio import move_on_after

from kb_core import KnowledgeBase, _is_excluded

logging.basicConfig(level=logging.INFO, format="[loom] %(message)s")
log = logging.getLogger("loom")

# ── Config ────────────────────────────────────────────────────────────────────

KB_ROOT = Path(os.environ["KB_ROOT"])
VAULT_PATH = os.getenv("VAULT_PATH")
OB_EMBEDDING = os.getenv("OBSIDIAN_BRAIN_EMBEDDING")  # None = use default local model
BRAINJAR_PATH = os.getenv("BRAINJAR_PATH")  # future

# ── obsidian-brain backend ────────────────────────────────────────────────────

class ObsidianBrainBackend:
    """Manages the obsidian-brain subprocess and MCP client session."""

    def __init__(self, vault_path: str, embedding: Optional[str] = None):
        self.vault_path = vault_path
        self.embedding = embedding
        self._session: Optional[ClientSession] = None
        self._exit_stack = None
        self.available = False
        self._starting = False

    async def _connect(self):
        """Establish subprocess and MCP session. Called by start() and on crash recovery."""
        from contextlib import AsyncExitStack
        env = {"VAULT_PATH": self.vault_path}
        if self.embedding:
            env["OBSIDIAN_BRAIN_EMBEDDING"] = self.embedding
        params = StdioServerParameters(
            command="npx",
            args=["-y", "obsidian-brain@latest", "server"],
            env=env,
        )
        if self._exit_stack:
            await self._exit_stack.aclose()
        self._exit_stack = AsyncExitStack()
        read, write = await self._exit_stack.enter_async_context(stdio_client(params))
        self._session = await self._exit_stack.enter_async_context(
            ClientSession(read, write)
        )
        await self._session.initialize()
        self.available = True
        log.info("obsidian-brain backend ready (vault: %s)", self.vault_path)

    async def start(self):
        """Fire connection in background — lifespan returns immediately.
        First-run embedding model download (~34 MB) takes 30–60 s.
        Tools return {"error": "initializing"} until ready."""
        import asyncio

        self._starting = True

        async def _run():
            try:
                await self._connect()
            except Exception as e:
                log.warning("obsidian-brain backend failed to start: %s", e)
                self.available = False
            finally:
                self._starting = False

        asyncio.create_task(_run())

    async def stop(self):
        if self._exit_stack:
            await self._exit_stack.aclose()
        self.available = False

    async def call_tool(self, name: str, arguments: dict[str, Any]) -> Any:
        if self._starting:
            return {"error": "obsidian-brain initializing", "hint": "Backend starting up — retry shortly"}
        if not self.available or self._session is None:
            return {"error": "obsidian-brain unavailable", "reason": "backend not running"}
        try:
            result = await self._session.call_tool(name, arguments)
            if result.content:
                import json
                raw = result.content[0].text
                try:
                    return json.loads(raw)
                except (json.JSONDecodeError, AttributeError):
                    return raw
            return {}
        except Exception as e:
            log.warning("obsidian-brain tool %s failed, attempting restart: %s", name, e)
            self.available = False
            try:
                await self._connect()
                result = await self._session.call_tool(name, arguments)
                if result.content:
                    import json
                    raw = result.content[0].text
                    try:
                        return json.loads(raw)
                    except (json.JSONDecodeError, AttributeError):
                        return raw
                return {}
            except Exception as e2:
                log.error("obsidian-brain restart failed: %s", e2)
                self.available = False
                return {"error": str(e2), "tool": name}


# ── Singleton backends ────────────────────────────────────────────────────────

kb = KnowledgeBase(KB_ROOT)
ob = ObsidianBrainBackend(VAULT_PATH, OB_EMBEDDING) if VAULT_PATH else None


# ── Search utilities ──────────────────────────────────────────────────────────

def rrf_merge(
    result_lists: list[tuple[str, list[dict]]],
    top_k: int = 10,
    k: int = 60,
) -> list[dict]:
    """
    Reciprocal Rank Fusion over multiple ranked result lists.

    result_lists: [(engine_name, [result_dict, ...]), ...]
    Each result_dict must have a "file" key used as the dedup handle.
    Returns up to top_k results sorted by RRF score descending.
    """
    scores: dict[str, float] = defaultdict(float)
    best: dict[str, dict] = {}  # file → best result dict (from highest-ranked engine)

    for engine_name, results in result_lists:
        for rank, result in enumerate(results, start=1):
            file_key = result.get("file", "")
            scores[file_key] += 1.0 / (k + rank)
            # Accumulate sources across engines; prefer result dict from rank-1 engine
            if file_key not in best:
                best[file_key] = {**result, "source": [engine_name]}
            else:
                merged_sources = list(set(best[file_key].get("source", [])) | {engine_name})
                if rank == 1:
                    best[file_key] = {**result, "source": merged_sources}
                else:
                    best[file_key]["source"] = merged_sources

    ranked = sorted(scores.items(), key=lambda x: x[1], reverse=True)[:top_k]
    merged = []
    for file_key, score in ranked:
        entry = best[file_key].copy()
        entry["score"] = round(score, 6)
        merged.append(entry)
    return merged


def enrich_with_kb(results: list[dict]) -> list[dict]:
    """
    For results missing line_start/heading (e.g. from obsidian-brain),
    look up the file in the kb index and attach the first matching chunk's
    metadata. Enables surgical edits on any search result without a follow-up call.
    """
    enriched = []
    for r in results:
        if r.get("line_start") is None:
            chunk = kb.find_chunk_for_file(r.get("file", ""))
            if chunk:
                r = {
                    **r,
                    "line_start": chunk.line_start,
                    "line_end": chunk.line_end,
                    "heading": chunk.heading,
                }
        enriched.append(r)
    return enriched


def _filter_excluded(items: list[dict]) -> list[dict]:
    """Drop items whose path matches .loomignore exclusions.
    Checks 'file', 'path', and 'note' keys — covers obsidian-brain and brainjar response shapes."""
    if not kb.exclusions:
        return items
    out = []
    for item in items:
        path = item.get("file") or item.get("path") or item.get("note") or ""
        if not path or not _is_excluded(str(path), kb.exclusions):
            out.append(item)
    return out


# ── Startup / shutdown ────────────────────────────────────────────────────────

@asynccontextmanager
async def lifespan(server):
    log.info("kb backend ready (%d chunks, root: %s)", len(kb.chunks), KB_ROOT)
    if ob:
        await ob.start()  # fires in background, returns immediately
    else:
        log.info("obsidian-brain disabled (VAULT_PATH not set)")
    if BRAINJAR_PATH:
        log.info("brainjar configured but not yet wired (BRAINJAR_PATH=%s)", BRAINJAR_PATH)
    else:
        log.info("brainjar disabled (BRAINJAR_PATH not set)")
    yield
    if ob:
        await ob.stop()


mcp = FastMCP(
    "loom",
    instructions=(
        "Unified knowledge loom tool surface. Use loom_search for hybrid BM25+semantic search. "
        "Use loom_rank_notes / loom_find_connections for graph analytics. "
        "Use loom_read_section / loom_read_lines for targeted reads. "
        "Use loom_replace_lines / loom_insert_after_heading / loom_append_to_file for surgical edits."
    ),
    lifespan=lifespan,
)


# ── Unified search ────────────────────────────────────────────────────────────

@mcp.tool()
async def loom_search(query: str, top_k: int = 10, file_filter: Optional[str] = None) -> dict:
    """
    Unified hybrid search across all configured backends. Results are RRF-merged
    and enriched with kb line/heading metadata for immediate surgical editing.
    Args: query, top_k (default 10), file_filter (optional path substring — applied to kb only)
    """
    result_lists = []
    warnings = []

    # kb (in-process BM25, always available)
    kb_results = kb.search(query, top_k=top_k, file_filter=file_filter)
    result_lists.append(("kb", kb_results))

    # obsidian-brain (semantic + BM25 hybrid)
    if ob and ob.available:
        ob_raw = await ob.call_tool("search", {"query": query, "limit": top_k})
        if isinstance(ob_raw, list):
            # Normalise to unified schema: obsidian-brain returns {path, score, chunk}
            ob_results = _filter_excluded([
                {
                    "file": r.get("path") or r.get("file", ""),
                    "snippet": r.get("chunk") or r.get("snippet", ""),
                    "score": r.get("score", 0.0),
                }
                for r in ob_raw
            ])
            result_lists.append(("obsidian-brain", ob_results))
        elif isinstance(ob_raw, dict) and "error" in ob_raw:
            warnings.append(f"obsidian-brain: {ob_raw['error']}")
    elif ob and not ob.available:
        warnings.append("obsidian-brain unavailable")

    merged = rrf_merge(result_lists, top_k=top_k)
    enriched = enrich_with_kb(merged)

    engines = [name for name, _ in result_lists]
    response = {"results": enriched, "engines": engines}
    if warnings:
        response["warnings"] = warnings
    return response


@mcp.tool()
async def loom_search_graph(note: str) -> dict:
    """
    Connections and relationships for a specific note via obsidian-brain's graph.
    Use as starting point for graph traversal — not a free-text search.
    Args: note (note path or title)
    """
    if not ob or not ob.available:
        return {"error": "obsidian-brain unavailable", "hint": "Set VAULT_PATH to enable"}
    raw = await ob.call_tool("find_connections", {"note": note})
    results = _filter_excluded(raw) if isinstance(raw, list) else raw
    return {"results": results, "engine": "obsidian-brain"}


@mcp.tool()
async def loom_search_smart(query: str) -> dict:
    """
    LLM-decomposed multi-search (brainjar --smart mode).
    Decomposes query into 2-5 targeted sub-searches before retrieval.
    Args: query
    """
    return {
        "error": "not configured",
        "backend": "brainjar",
        "hint": "Set BRAINJAR_PATH to enable loom_search_smart",
    }


# ── Graph analytics ───────────────────────────────────────────────────────────

@mcp.tool()
async def loom_rank_notes(limit: int = 20) -> dict:
    """PageRank-based ranking of most influential notes in the vault. Args: limit (default 20)"""
    if not ob or not ob.available:
        return {"error": "obsidian-brain unavailable", "hint": "Set VAULT_PATH to enable"}
    raw = await ob.call_tool("rank_notes", {"limit": limit})
    if isinstance(raw, list):
        return {"results": _filter_excluded(raw)}
    return raw


@mcp.tool()
async def loom_find_connections(note: str) -> dict:
    """Links and relationships for a specific note. Args: note (path or title)"""
    if not ob or not ob.available:
        return {"error": "obsidian-brain unavailable", "hint": "Set VAULT_PATH to enable"}
    raw = await ob.call_tool("find_connections", {"note": note})
    if isinstance(raw, list):
        return {"results": _filter_excluded(raw)}
    return raw


@mcp.tool()
async def loom_find_path_between(note_a: str, note_b: str) -> dict:
    """Shortest connection route between two notes in the graph. Args: note_a, note_b"""
    if not ob or not ob.available:
        return {"error": "obsidian-brain unavailable", "hint": "Set VAULT_PATH to enable"}
    return await ob.call_tool("find_path_between", {"note_a": note_a, "note_b": note_b})


@mcp.tool()
async def loom_detect_themes() -> dict:
    """Louvain community detection — identifies thematic clusters in the vault."""
    if not ob or not ob.available:
        return {"error": "obsidian-brain unavailable", "hint": "Set VAULT_PATH to enable"}
    return await ob.call_tool("detect_themes", {})


# ── Navigation ────────────────────────────────────────────────────────────────

@mcp.tool()
def loom_list_files() -> list[dict]:
    """List all Markdown files in the kb with line counts and sizes."""
    return kb.list_files()


@mcp.tool()
def loom_outline(file_path: str) -> list[dict]:
    """Heading hierarchy with line numbers for a file. Args: file_path (relative)"""
    return kb.outline(file_path)


@mcp.tool()
def loom_grep(pattern: str, file_filter: Optional[str] = None) -> list[dict]:
    """Regex search across all kb files. Args: pattern (regex), file_filter (optional path substring)"""
    return kb.grep(pattern, file_filter=file_filter)


# ── Targeted reads ────────────────────────────────────────────────────────────

@mcp.tool()
def loom_read_section(file_path: str, heading: str) -> Optional[dict]:
    """
    Content under a heading with location metadata for follow-up edits.
    Args: file_path (relative), heading (case-insensitive substring match)
    """
    return kb.read_section(file_path, heading)


@mcp.tool()
def loom_read_lines(file_path: str, start_line: int, end_line: int) -> Optional[dict]:
    """Exact line range (1-indexed, inclusive). Args: file_path, start_line, end_line"""
    return kb.read_lines(file_path, start_line, end_line)


# ── Surgical edits ────────────────────────────────────────────────────────────

@mcp.tool()
def loom_replace_lines(file_path: str, start_line: int, end_line: int, new_content: str) -> dict:
    """
    Overwrite a line range in-place; rebuilds kb index after write.
    Args: file_path (relative), start_line, end_line, new_content
    """
    return kb.replace_lines(file_path, start_line, end_line, new_content)


@mcp.tool()
def loom_insert_after_heading(file_path: str, heading: str, content: str) -> dict:
    """
    Insert content immediately after a heading line; rebuilds kb index after write.
    Args: file_path (relative), heading (substring match OK), content
    """
    return kb.insert_after_heading(file_path, heading, content)


@mcp.tool()
def loom_append_to_file(file_path: str, content: str) -> dict:
    """
    Append content to end of file with blank-line separator. Creates file if missing.
    Args: file_path (relative), content
    """
    return kb.append_to_file(file_path, content)


# ── Vault-level edits ─────────────────────────────────────────────────────────

def _ob_required() -> dict:
    return {"error": "obsidian-brain unavailable", "hint": "Set VAULT_PATH to enable"}


@mcp.tool()
async def loom_create_note(path: str, content: str) -> dict:
    """Create a new markdown note in the vault. Args: path (relative), content"""
    if not ob or not ob.available:
        return _ob_required()
    return await ob.call_tool("create_note", {"path": path, "content": content})


@mcp.tool()
def loom_edit_note(path: str, heading: str, new_content: str) -> dict:
    """
    Replace the content of a section identified by heading using kb line coordinates.
    No round-trip needed — heading and line_start come directly from loom_search results.
    Args: path (relative), heading (substring match), new_content
    """
    section = kb.read_section(path, heading)
    if section is None:
        return {"error": f"heading '{heading}' not found in {path}"}
    result = kb.replace_lines(path, section["content_start"], section["content_end"], new_content)
    return {**result, "heading": section["heading"], "preview": new_content[:120]}


@mcp.tool()
def loom_apply_edit_preview(path: str, heading: str, new_content: str) -> dict:
    """
    Preview a section replacement without committing. Returns current and proposed content.
    Call loom_edit_note with the same args to apply.
    Args: path (relative), heading (substring match), new_content
    """
    section = kb.read_section(path, heading)
    if section is None:
        return {"error": f"heading '{heading}' not found in {path}"}
    return {
        "file": path,
        "heading": section["heading"],
        "lines": f"{section['content_start']}–{section['content_end']}",
        "current": section["content"],
        "proposed": new_content,
    }


@mcp.tool()
def loom_link_notes(
    note_a: str,
    note_b: str,
    heading_a: Optional[str] = None,
    heading_b: Optional[str] = None,
) -> dict:
    """
    Append an inline wiki-link at the end of a section in each note (bidirectional).
    Appends to the first section if no heading specified. Links are woven into prose, not
    a separate links section.
    Args: note_a, note_b (relative paths), heading_a, heading_b (optional, substring match)
    """
    def _append_link(target_path: str, link_path: str, heading: Optional[str]) -> dict:
        stem = Path(link_path).stem
        link_text = f"See also: [[{stem}]]"
        if heading:
            section = kb.read_section(target_path, heading)
            if section is None:
                return {"error": f"heading '{heading}' not found in {target_path}"}
            new_content = section["content"].rstrip() + "\n\n" + link_text
            return kb.replace_lines(target_path, section["content_start"], section["content_end"], new_content)
        return kb.append_to_file(target_path, link_text)

    return {"note_a": _append_link(note_a, note_b, heading_a), "note_b": _append_link(note_b, note_a, heading_b)}


@mcp.tool()
async def loom_move_note(path: str, new_path: str) -> dict:
    """
    Relocate a note. Proxies to obsidian-brain (updates backlinks) when available;
    falls back to filesystem rename with a dangling-link warning.
    Args: path, new_path (both relative to KB_ROOT)
    """
    if ob and ob.available:
        result = await ob.call_tool("move_note", {"path": path, "new_path": new_path})
        kb.rebuild()
        return result
    src = kb.root / path
    dst = kb.root / new_path
    if not src.exists():
        return {"error": f"file not found: {path}"}
    dst.parent.mkdir(parents=True, exist_ok=True)
    src.rename(dst)
    kb.rebuild()
    return {
        "moved": True, "from": path, "to": new_path,
        "warning": "obsidian-brain unavailable — backlinks in other notes NOT updated",
    }


@mcp.tool()
async def loom_delete_note(path: str) -> dict:
    """
    Remove a note. Proxies to obsidian-brain (removes backlinks) when available;
    falls back to filesystem delete with a dangling-link warning.
    Args: path (relative to KB_ROOT)
    """
    if ob and ob.available:
        result = await ob.call_tool("delete_note", {"path": path})
        kb.rebuild()
        return result
    target = kb.root / path
    if not target.exists():
        return {"error": f"file not found: {path}"}
    target.unlink()
    kb.rebuild()
    return {
        "deleted": True, "path": path,
        "warning": "obsidian-brain unavailable — backlinks in other notes NOT updated",
    }


# ── Maintenance ───────────────────────────────────────────────────────────────

@mcp.tool()
async def loom_reindex() -> dict:
    """Trigger obsidian-brain vault re-index and rebuild kb index."""
    kb.rebuild()
    log.info("kb index rebuilt (%d chunks)", len(kb.chunks))
    if not ob or not ob.available:
        return {"kb": "rebuilt", "obsidian-brain": "unavailable"}
    ob_result = await ob.call_tool("reindex", {})
    return {"kb": "rebuilt", "obsidian-brain": ob_result}


@mcp.tool()
async def loom_index_status() -> dict:
    """Index health and statistics for all active backends."""
    status = {
        "kb": {
            "chunks": len(kb.chunks),
            "files": len(kb.all_lines),
            "root": str(KB_ROOT),
        }
    }
    if ob and ob.available:
        ob_status = await ob.call_tool("index_status", {})
        status["obsidian-brain"] = ob_status
    else:
        status["obsidian-brain"] = {"available": False, "vault_path": VAULT_PATH}
    if BRAINJAR_PATH:
        status["brainjar"] = {"available": False, "note": "not yet wired"}
    return status


if __name__ == "__main__":
    mcp.run(transport="stdio")
