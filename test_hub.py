#!/usr/bin/env python3
"""
Smoke tests for the loom hub. Run directly: python test_hub.py
Tests each tool category without requiring pytest or a live MCP connection.
"""

import asyncio
import os
import sys
from pathlib import Path

VAULT = Path("test-vault")
VAULT.mkdir(exist_ok=True)
(VAULT / "test-note.md").write_text(
    "# Test Note\n\nThis is a test.\n\n## Details\n\nMore content here.\n"
)

os.environ.setdefault("KB_ROOT", str(VAULT))

import loom_mcp

PASS = "\033[32mPASS\033[0m"
FAIL = "\033[31mFAIL\033[0m"
results = []


def check(name: str, condition: bool, detail: str = ""):
    status = PASS if condition else FAIL
    print(f"  {status}  {name}" + (f": {detail}" if detail else ""))
    results.append((name, condition))


async def run():
    print("\n── kb backend ──────────────────────────────────")

    files = loom_mcp.loom_list_files()
    check("loom_list_files", any(f["file"] == "test-note.md" for f in files))

    outline = loom_mcp.loom_outline("test-note.md")
    check("loom_outline", len(outline) >= 2, f"{len(outline)} headings")

    section = loom_mcp.loom_read_section("test-note.md", "Details")
    check("loom_read_section", section is not None and "More content" in section["content"])

    lines = loom_mcp.loom_read_lines("test-note.md", 1, 3)
    check("loom_read_lines", lines is not None and "Test Note" in lines["content"])

    grepped = loom_mcp.loom_grep("content")
    check("loom_grep", len(grepped) > 0)

    print("\n── unified search ──────────────────────────────")

    search_result = await loom_mcp.loom_search("test content")
    check("loom_search returns results", len(search_result["results"]) > 0)
    check("loom_search engines list", "kb" in search_result["engines"])
    check(
        "loom_search result has line_start",
        all(r.get("line_start") is not None for r in search_result["results"]),
    )

    filtered = await loom_mcp.loom_search("test", file_filter="test-note")
    check(
        "loom_search file_filter",
        all("test-note" in r["file"] for r in filtered["results"]),
    )

    print("\n── search stubs ────────────────────────────────")

    smart = await loom_mcp.loom_search_smart("test")
    check("loom_search_smart stub error", smart["error"] == "not configured")
    check("loom_search_smart stub backend", smart["backend"] == "brainjar")

    print("\n── obsidian-brain tools (unavailable) ──────────")

    rank = await loom_mcp.loom_rank_notes()
    check("loom_rank_notes returns error when unavailable", "error" in rank)

    graph = await loom_mcp.loom_search_graph("test")
    check("loom_search_graph returns error when unavailable", "error" in graph)

    connections = await loom_mcp.loom_find_connections("test")
    check("loom_find_connections returns error when unavailable", "error" in connections)

    path = await loom_mcp.loom_find_path_between("a.md", "b.md")
    check("loom_find_path_between returns error when unavailable", "error" in path)

    themes = await loom_mcp.loom_detect_themes()
    check("loom_detect_themes returns error when unavailable", "error" in themes)

    print("\n── surgical edits ──────────────────────────────")

    (VAULT / "edit-smoke.md").write_text("# Edit Test\n\nOriginal content.\n")
    loom_mcp.kb.rebuild()

    appended = loom_mcp.loom_append_to_file("edit-smoke.md", "## Appended\n\nAppended line.")
    check("loom_append_to_file", "appended_at_line" in appended)

    read_back = loom_mcp.loom_read_lines("edit-smoke.md", appended["appended_at_line"], appended["appended_at_line"] + 2)
    check("loom_read_lines after append", read_back is not None and "Appended" in read_back["content"])

    replaced = loom_mcp.loom_replace_lines("edit-smoke.md", 3, 3, "Replaced content.")
    check("loom_replace_lines", "replaced_lines" in replaced)

    inserted = loom_mcp.loom_insert_after_heading("edit-smoke.md", "Edit Test", "Inserted line.")
    check("loom_insert_after_heading", "inserted_at_line" in inserted)

    print("\n── vault-level edits (kb path) ─────────────────")

    (VAULT / "vault-smoke.md").write_text("# Vault Test\n\nSection content.\n\n## Sub\n\nSub content.\n")
    loom_mcp.kb.rebuild()

    preview = loom_mcp.loom_apply_edit_preview("vault-smoke.md", "Vault Test", "New content.")
    check("loom_apply_edit_preview returns current", "current" in preview and "proposed" in preview)
    check("loom_apply_edit_preview heading", preview.get("heading") is not None)

    edited = loom_mcp.loom_edit_note("vault-smoke.md", "Sub", "Replaced sub content.")
    check("loom_edit_note", "heading" in edited and "preview" in edited)

    create_result = await loom_mcp.loom_create_note("new-note.md", "# New\n\nContent.")
    check("loom_create_note unavailable", "error" in create_result)

    print("\n── maintenance ─────────────────────────────────")

    status = await loom_mcp.loom_index_status()
    check("loom_index_status has kb", "kb" in status)
    check("loom_index_status kb chunks > 0", status["kb"]["chunks"] > 0)
    check("loom_index_status ob unavailable", not status["obsidian-brain"]["available"])

    reindex = await loom_mcp.loom_reindex()
    check("loom_reindex rebuilds kb", reindex["kb"] == "rebuilt")

    # cleanup
    for f in ["edit-smoke.md", "vault-smoke.md", "test-note.md"]:
        p = VAULT / f
        if p.exists():
            p.unlink()
    loom_mcp.kb.rebuild()

    print(f"\n{'─' * 50}")
    passed = sum(1 for _, ok in results if ok)
    total = len(results)
    failed = [(name, ok) for name, ok in results if not ok]
    print(f"  {passed}/{total} checks passed")
    if failed:
        print("\nFailed checks:")
        for name, _ in failed:
            print(f"  - {name}")
        sys.exit(1)


asyncio.run(run())
