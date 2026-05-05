#!/Users/odinkirk/Documents/Claude/Projects/Laila's World/tools/.venv/bin/python3.14
"""
Laila's World Knowledge Base MCP Server

Provides targeted, metadata-rich access to the markdown knowledge base without
database dependencies. Uses in-memory BM25 indexing built from file-system scans.

Environment:
  LORE_ROOT — root directory of markdown files (defaults to project root)
"""

import os
import re
import time
from pathlib import Path
from typing import Optional
from dataclasses import dataclass, asdict, field
import fnmatch
from rank_bm25 import BM25Okapi

from mcp.server.fastmcp import FastMCP

# ─────────────────────────────────────────────────────────────────────────────
# Config & Init

LORE_ROOT = Path(os.getenv("LORE_ROOT", str(Path(__file__).parent.parent)))
LORE_ROOT.mkdir(parents=True, exist_ok=True)


def set_lore_root(new_root: str | Path) -> Path:
    """Reassign LORE_ROOT and rebuild the index. Returns the new root."""
    global LORE_ROOT
    LORE_ROOT = Path(new_root)
    LORE_ROOT.mkdir(parents=True, exist_ok=True)
    kb.rebuild()
    return LORE_ROOT

mcp = FastMCP("laila-lore", instructions=(
    "Access Laila's World knowledge base. All tools return location metadata "
    "(file, line numbers) enabling direct follow-up edits. Use `search` to find "
    "content, `outline` to explore structure, `read_section` to fetch by heading."
))

# ─────────────────────────────────────────────────────────────────────────────
# Data Models

@dataclass
class Chunk:
    """A meaningful text unit (typically a section under a heading)."""
    file: str  # relative path
    heading: str  # breadcrumb, e.g. "Top Heading > Sub Heading"
    line_start: int  # 1-indexed, inclusive
    line_end: int  # 1-indexed, inclusive
    text: str  # full content of this chunk
    tokens: list[str] = field(default_factory=list)

    def __post_init__(self):
        if not self.tokens:
            self.tokens = tokenize(self.text)

# ─────────────────────────────────────────────────────────────────────────────
# Utilities

def tokenize(text: str) -> list[str]:
    """Extract lowercase alphanumeric tokens from text."""
    return re.findall(r'\b[a-z0-9]+\b', text.lower())

def read_file(file_path: Path) -> list[str]:
    """Read file; return lines (no trailing newline)."""
    try:
        return file_path.read_text(encoding='utf-8').splitlines()
    except Exception:
        return []


def _split_content(content: str) -> list[str]:
    """Split content into lines, treating a single trailing \\n as a terminator (not a blank line).
    Double trailing \\n\\n produces a trailing empty string, preserving intentional blank lines."""
    if not content:
        return []
    lines = content.split('\n')
    if lines and lines[-1] == '':
        lines = lines[:-1]
    return lines


def _load_exclusions(root: Path) -> list[str]:
    """Read .loomignore patterns; returns empty list if file absent."""
    ignore_file = root / ".loomignore"
    if not ignore_file.exists():
        return []
    return [
        line.strip()
        for line in ignore_file.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.startswith("#")
    ]


def _is_excluded(rel_posix: str, patterns: list[str]) -> bool:
    """Return True if rel_posix matches any .loomignore pattern.

    Directory patterns (ending in '/') match when any path component matches
    the glob. Other patterns are matched against the full path and the filename.
    """
    parts = rel_posix.split("/")
    for pattern in patterns:
        if pattern.endswith("/"):
            dir_glob = pattern.rstrip("/")
            if any(fnmatch.fnmatch(p, dir_glob) for p in parts[:-1]):
                return True
        else:
            if fnmatch.fnmatch(rel_posix, pattern) or fnmatch.fnmatch(parts[-1], pattern):
                return True
    return False


def find_markdown_files() -> list[Path]:
    """Recursively find all .md files under LORE_ROOT."""
    return sorted(LORE_ROOT.glob("**/*.md"))

def parse_headings_and_chunks(file_path: Path) -> list[Chunk]:
    """
    Parse markdown file into chunks, one per section (heading + content).
    Returns list of Chunk objects with full heading breadcrumbs.
    """
    lines = read_file(file_path)
    rel_path = file_path.relative_to(LORE_ROOT).as_posix()
    chunks = []
    heading_stack = []  # (level, text, line_number)
    i = 0

    while i < len(lines):
        line = lines[i]
        match = re.match(r'^(#{1,6})\s+(.+)$', line)

        if match:
            level = len(match.group(1))
            heading_text = match.group(2).strip()

            # Pop headings of equal or greater level (maintain hierarchy)
            while heading_stack and heading_stack[-1][0] >= level:
                heading_stack.pop()

            heading_stack.append((level, heading_text, i + 1))

            # Build breadcrumb (e.g., "I. Central Premise > Demons")
            breadcrumb = " > ".join([h[1] for h in heading_stack])

            # Collect content until next heading
            content_lines = []
            content_start = i + 1
            j = i + 1
            while j < len(lines) and not re.match(r'^#{1,6}\s+', lines[j]):
                content_lines.append(lines[j])
                j += 1

            content = "\n".join(content_lines).strip()
            if content:  # Only create chunk if there's content
                chunks.append(Chunk(
                    file=rel_path,
                    heading=breadcrumb,
                    line_start=i + 1,  # heading line
                    line_end=j - 1 if j > i + 1 else i + 1,
                    text=content
                ))
            i = j
        else:
            i += 1

    return chunks

# ─────────────────────────────────────────────────────────────────────────────
# Global Index (built on server startup)

class KnowledgeBase:
    def __init__(self):
        self.chunks = []
        self.bm25 = None
        self.all_lines = {}  # file_path -> list of lines
        self.rebuild_timestamp = time.time()  # Timestamp of last rebuild (cache-busting)
        self.rebuild()

    def rebuild(self):
        """Scan all files, parse chunks, build BM25 index."""
        self.chunks = []
        self.all_lines = {}
        
        # Load exclusions from .loomignore
        self.exclusions = _load_exclusions(LORE_ROOT)

        for file_path in find_markdown_files():
            rel_path = file_path.relative_to(LORE_ROOT).as_posix()
            if self.exclusions and _is_excluded(rel_path, self.exclusions):
                continue
            self.all_lines[rel_path] = read_file(file_path)
            self.chunks.extend(parse_headings_and_chunks(file_path))

        # Build BM25 index
        if self.chunks:
            corpus = [c.tokens for c in self.chunks]
            self.bm25 = BM25Okapi(corpus)

        # Update timestamp to invalidate transport-layer caches
        self.rebuild_timestamp = time.time()

    def search(self, query: str, top_k: int = 5, file_filter: Optional[str] = None) -> list[dict]:
        """BM25 search with optional file filtering."""
        if not self.bm25:
            return []

        query_tokens = tokenize(query)
        scores = self.bm25.get_scores(query_tokens)

        # Build results with scores
        results = []
        for chunk, score in zip(self.chunks, scores):
            if file_filter and file_filter not in chunk.file:
                continue
            if score > 0:
                # Extract snippet around query terms (max 200 chars)
                snippet = chunk.text
                for token in query_tokens:
                    idx = snippet.lower().find(token)
                    if idx >= 0:
                        start = max(0, idx - 50)
                        end = min(len(snippet), idx + 150)
                        snippet = snippet[start:end].strip()
                        if start > 0:
                            snippet = "..." + snippet
                        if end < len(chunk.text):
                            snippet = snippet + "..."
                        break

                results.append({
                    "file": chunk.file,
                    "heading": chunk.heading,
                    "line_start": chunk.line_start,
                    "line_end": chunk.line_end,
                    "snippet": snippet,
                    "score": float(score)
                })

        return sorted(results, key=lambda r: r["score"], reverse=True)[:top_k]

    def list_files(self) -> list[dict]:
        """List all markdown files with metadata."""
        files = []
        for file_path in find_markdown_files():
            rel_path = file_path.relative_to(LORE_ROOT).as_posix()
            lines = self.all_lines.get(rel_path, [])
            try:
                size_kb = file_path.stat().st_size / 1024
            except:
                size_kb = 0
            files.append({
                "file": rel_path,
                "line_count": len(lines),
                "size_kb": round(size_kb, 1)
            })
        return sorted(files, key=lambda f: f["file"])

    def outline(self, file_path: str) -> list[dict]:
        """Extract heading hierarchy with line numbers."""
        rel_path = Path(file_path).as_posix()
        lines = self.all_lines.get(rel_path, [])
        outline = []
        for i, line in enumerate(lines, 1):
            match = re.match(r'^(#{1,6})\s+(.+)$', line)
            if match:
                level = len(match.group(1))
                heading = match.group(2).strip()
                outline.append({
                    "level": level,
                    "heading": heading,
                    "line_number": i
                })
        return outline

    def grep(self, pattern: str, file_filter: Optional[str] = None, limit: int = 50) -> list[dict]:
        """Regex search across all files."""
        results = []
        try:
            regex = re.compile(pattern, re.IGNORECASE)
        except re.error as e:
            return [{"error": f"Invalid regex: {e}"}]

        for file_rel, lines in self.all_lines.items():
            if file_filter and file_filter not in file_rel:
                continue
            for i, line in enumerate(lines, 1):
                if regex.search(line):
                    results.append({
                        "file": file_rel,
                        "line_number": i,
                        "line_text": line[:500]
                    })
                    if len(results) >= limit:
                        return results
        return results

    def add_cache_bust(self, response: dict) -> dict:
        """Add cache-busting metadata to response (invalidates transport-layer caches)."""
        response["_cache_bust"] = self.rebuild_timestamp
        return response

    def read_section(self, file_path: str, heading: str) -> Optional[dict]:
        """
        Read all content under a heading (case-insensitive substring match).
        Returns full section including sub-headings.
        """
        heading_lower = heading.lower()
        for chunk in self.chunks:
            if chunk.file == file_path and heading_lower in chunk.heading.lower():
                result = {
                    "file": chunk.file,
                    "heading": chunk.heading,
                    "heading_line": chunk.line_start,
                    "content_start": chunk.line_start + 1,
                    "content_end": chunk.line_end,
                    "content": chunk.text
                }
                return self.add_cache_bust(result)
        return None

    def read_lines(self, file_path: str, start_line: int, end_line: int) -> Optional[dict]:
        """Read specific line range (1-indexed, inclusive)."""
        rel_path = Path(file_path).as_posix()
        lines = self.all_lines.get(rel_path)
        if not lines:
            return None
        start_line = max(1, start_line)
        end_line = min(len(lines), end_line)
        content = "\n".join(lines[start_line - 1:end_line])
        result = {
            "file": rel_path,
            "start_line": start_line,
            "end_line": end_line,
            "content": content
        }
        return self.add_cache_bust(result)

    def replace_lines(self, file_path: str, start_line: int, end_line: int, new_content: str) -> dict:
        """Overwrite a line range in-place; rebuilds index after write."""
        rel_path = Path(file_path).as_posix()
        full_path = LORE_ROOT / rel_path
        lines = read_file(full_path)
        start_line = max(1, start_line)
        end_line = min(len(lines), end_line)
        new_lines = lines[:start_line - 1] + _split_content(new_content) + lines[end_line:]
        full_path.write_text("\n".join(new_lines) + "\n", encoding='utf-8')
        self.rebuild()
        result = {
            "file": rel_path,
            "replaced_lines": end_line - start_line + 1,
            "new_line_count": len(new_lines)
        }
        return self.add_cache_bust(result)

    def insert_after_heading(self, file_path: str, heading: str, content: str) -> dict:
        """Insert content immediately after a heading line; rebuilds index after write."""
        rel_path = Path(file_path).as_posix()
        full_path = LORE_ROOT / rel_path
        lines = read_file(full_path)

        heading_lower = heading.lower()
        insert_at = None
        for i, line in enumerate(lines):
            if re.match(r'^#{1,6}\s+', line) and heading_lower in line.lower():
                insert_at = i + 1
                break

        if insert_at is None:
            return {"error": f"Heading '{heading}' not found in {rel_path}"}

        new_lines = (
            lines[:insert_at] +
            [""] +
            _split_content(content) +
            lines[insert_at:]
        )
        full_path.write_text("\n".join(new_lines) + "\n", encoding='utf-8')

        self.rebuild()
        return {"file": rel_path, "inserted_at_line": insert_at + 1}

    def append_to_file(self, file_path: str, content: str) -> dict:
        """Append content to end of file with blank-line separator; rebuilds index after write."""
        rel_path = Path(file_path).as_posix()
        full_path = LORE_ROOT / rel_path
        full_path.parent.mkdir(parents=True, exist_ok=True)
        current = read_file(full_path) if full_path.exists() else []
        content_lines = _split_content(content)
        new_lines = current + [""] + content_lines
        full_path.write_text("\n".join(new_lines) + "\n", encoding='utf-8')
        self.rebuild()
        return {"file": rel_path, "appended_at_line": len(current) + 2}

kb = KnowledgeBase()

# ─────────────────────────────────────────────────────────────────────────────
# MCP Tools

@mcp.tool()
def search(query: str, top_k: int = 5, file_filter: Optional[str] = None) -> list[dict]:
    """
    Search the knowledge base with BM25.
    Returns ranked results with location metadata.

    Args:
      query: search terms
      top_k: max results (default 5)
      file_filter: scope to files containing this substring
    """
    results = kb.search(query, top_k=top_k, file_filter=file_filter)
    for result in results:
        result["_cache_bust"] = kb.rebuild_timestamp
    return results

@mcp.tool()
def list_files() -> list[dict]:
    """List all markdown files with line counts and sizes."""
    return kb.list_files()

@mcp.tool()
def outline(file_path: str) -> list[dict]:
    """
    Return heading hierarchy of a file.

    Args:
      file_path: relative path (e.g., "world/Story Bible v4.md")
    """
    results = kb.outline(file_path)
    for result in results:
        result["_cache_bust"] = kb.rebuild_timestamp
    return results

@mcp.tool()
def grep(pattern: str, file_filter: Optional[str] = None) -> list[dict]:
    """
    Regex search across all files.

    Args:
      pattern: regex pattern
      file_filter: scope to files containing this substring
    """
    return kb.grep(pattern, file_filter=file_filter)

@mcp.tool()
def read_section(file_path: str, heading: str) -> Optional[dict]:
    """
    Read all content under a heading (case-insensitive substring match).
    Returns section with location metadata enabling direct follow-up edits.

    Args:
      file_path: relative path
      heading: heading name (substring match OK)
    """
    return kb.read_section(file_path, heading)

@mcp.tool()
def read_lines(file_path: str, start_line: int, end_line: int) -> Optional[dict]:
    """
    Read specific line range.

    Args:
      file_path: relative path
      start_line: 1-indexed, inclusive
      end_line: 1-indexed, inclusive
    """
    return kb.read_lines(file_path, start_line, end_line)

@mcp.tool()
def replace_lines(file_path: str, start_line: int, end_line: int, new_content: str) -> dict:
    """
    Replace lines in a file. Primary edit tool for targeted updates.

    Args:
      file_path: relative path
      start_line: 1-indexed, inclusive
      end_line: 1-indexed, inclusive
      new_content: replacement text (can be multi-line)
    """
    return kb.replace_lines(file_path, start_line, end_line, new_content)

@mcp.tool()
def insert_after_heading(file_path: str, heading: str, content: str) -> dict:
    """
    Insert content after a heading.

    Args:
      file_path: relative path
      heading: heading name (substring match OK)
      content: text to insert
    """
    return kb.insert_after_heading(file_path, heading, content)

@mcp.tool()
def append_to_file(file_path: str, content: str) -> dict:
    """
    Append content to end of file with a blank line separator.

    Args:
      file_path: relative path (creates file if missing)
      content: text to append
    """
    return kb.append_to_file(file_path, content)

@mcp.tool()
def reassign_root(new_root: str) -> dict:
    """
    Reassign the knowledge base root directory and rebuild the index.
    Use this when working in a git worktree — point LORE_ROOT at the
    worktree path so all reads and writes target the worktree copy.

    Args:
      new_root: absolute path to the new root directory
    """
    new = set_lore_root(new_root)
    return {"lore_root": str(new), "files_indexed": len(kb.all_lines), "_cache_bust": kb.rebuild_timestamp}

@mcp.tool()
def cache_status() -> dict:
    """
    Diagnostic tool: report current cache-bust timestamp and index state.
    Use to verify MCP server freshness after worktree switches.
    """
    return {
        "cache_bust_timestamp": kb.rebuild_timestamp,
        "files_indexed": len(kb.all_lines),
        "chunks_indexed": len(kb.chunks),
        "lore_root": str(LORE_ROOT)
    }

# ─────────────────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    mcp.run(transport="stdio")
