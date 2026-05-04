#!/usr/bin/env python3
"""
Markdown Knowledge Base MCP Server

Provides targeted, metadata-rich access to a folder of Markdown files.
Uses in-memory BM25 indexing built from file-system scans at startup.

Environment:
  KB_ROOT        — root directory of Markdown files (required, no default)
  MCP_SERVER_NAME — display name for this server (default: "markdown-kb")
"""

import os
from pathlib import Path
from typing import Optional

from mcp.server.fastmcp import FastMCP
from kb_core import KnowledgeBase

KB_ROOT = Path(os.environ["KB_ROOT"])
SERVER_NAME = os.getenv("MCP_SERVER_NAME", "markdown-kb")

mcp = FastMCP(SERVER_NAME, instructions=(
    "Access a Markdown knowledge base. All tools return location metadata "
    "(file, line numbers) enabling direct follow-up edits. Use `search` to find "
    "content, `outline` to explore structure, `read_section` to fetch by heading."
))

kb = KnowledgeBase(KB_ROOT)

# ─────────────────────────────────────────────────────────────────────────────
# Tools (keep all @mcp.tool() definitions and the if __name__ == "__main__": block unchanged)
# ─────────────────────────────────────────────────────────────────────────────

@mcp.tool()
def search(query: str, top_k: int = 5, file_filter: Optional[str] = None) -> list[dict]:
    """
    Search the knowledge base for a query.
    Args: query, top_k (default 5), file_filter (optional path substring)
    """
    return kb.search(query, top_k=top_k, file_filter=file_filter)


@mcp.tool()
def list_files() -> list[dict]:
    """List all Markdown files in the kb with line counts and sizes."""
    return kb.list_files()


@mcp.tool()
def outline(file_path: str) -> list[dict]:
    """Heading hierarchy with line numbers for a file. Args: file_path (relative)"""
    return kb.outline(file_path)


@mcp.tool()
def grep(pattern: str, file_filter: Optional[str] = None) -> list[dict]:
    """Regex search across all kb files. Args: pattern (regex), file_filter (optional path substring)"""
    return kb.grep(pattern, file_filter=file_filter)


@mcp.tool()
def read_section(file_path: str, heading: str) -> Optional[dict]:
    """
    Content under a heading with location metadata for follow-up edits.
    Args: file_path (relative), heading (case-insensitive substring match)
    """
    return kb.read_section(file_path, heading)


@mcp.tool()
def read_lines(file_path: str, start_line: int, end_line: int) -> Optional[dict]:
    """Exact line range (1-indexed, inclusive). Args: file_path, start_line, end_line"""
    return kb.read_lines(file_path, start_line, end_line)


@mcp.tool()
def replace_lines(file_path: str, start_line: int, end_line: int, new_content: str) -> dict:
    """
    Overwrite a line range in-place; rebuilds kb index after write.
    Args: file_path (relative), start_line, end_line, new_content
    """
    return kb.replace_lines(file_path, start_line, end_line, new_content)


@mcp.tool()
def insert_after_heading(file_path: str, heading: str, content: str) -> dict:
    """
    Insert content immediately after a heading line; rebuilds kb index after write.
    Args: file_path (relative), heading (substring match OK), content
    """
    return kb.insert_after_heading(file_path, heading, content)


@mcp.tool()
def append_to_file(file_path: str, content: str) -> dict:
    """
    Append content to end of file with blank-line separator. Creates file if missing.
    Args: file_path (relative), content
    """
    return kb.append_to_file(file_path, content)


if __name__ == "__main__":
    mcp.run(transport="stdio")