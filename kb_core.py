#!/usr/bin/env python3
"""
Core knowledge base implementation - importable without side effects.
Provides BM25 search and chunk parsing for Markdown files.
"""
import fnmatch
import re
from pathlib import Path
from typing import Optional
from dataclasses import dataclass, field
from rank_bm25 import BM25Okapi


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


def tokenize(text: str) -> list[str]:
    """Extract lowercase alphanumeric tokens from text."""
    return re.findall(r'\b[a-z0-9]+\b', text.lower())


def read_file(file_path: Path) -> list[str]:
    """Read file lines, returning empty list on error."""
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


def parse_chunks(file_path: Path, root: Path) -> list[Chunk]:
    """Parse a Markdown file into chunks based on heading structure.
    Files with no headings get a single whole-file chunk so they appear in search and enrichment."""
    lines = read_file(file_path)
    rel_path = file_path.relative_to(root).as_posix()
    chunks = []
    heading_stack = []  # (level, text, line_number)
    i = 0

    while i < len(lines):
        line = lines[i]
        match = re.match(r'^(#{1,6})\s+(.+)$', line)

        if match:
            level = len(match.group(1))
            heading_text = match.group(2).strip()

            while heading_stack and heading_stack[-1][0] >= level:
                heading_stack.pop()
            heading_stack.append((level, heading_text, i + 1))

            breadcrumb = " > ".join(h[1] for h in heading_stack)

            j = i + 1
            content_lines = []
            while j < len(lines) and not re.match(r'^#{1,6}\s+', lines[j]):
                content_lines.append(lines[j])
                j += 1

            content = "\n".join(content_lines).strip()
            if content:
                chunks.append(Chunk(
                    file=rel_path,
                    heading=breadcrumb,
                    line_start=i + 1,
                    line_end=j - 1 if j > i + 1 else i + 1,
                    text=content
                ))
            i = j
        else:
            i += 1

    if not chunks and lines:
        full_text = "\n".join(lines).strip()
        if full_text:
            chunks.append(Chunk(
                file=rel_path,
                heading="(document)",
                line_start=1,
                line_end=len(lines),
                text=full_text,
            ))

    return chunks


class KnowledgeBase:
    """In-memory BM25 index over a directory of Markdown files."""

    def __init__(self, root: Path):
        self.root = root
        self.chunks: list[Chunk] = []
        self.bm25 = None
        self.all_lines: dict[str, list[str]] = {}
        self.exclusions: list[str] = []
        self.rebuild()

    def rebuild(self):
        """Rebuild the index from disk."""
        self.chunks = []
        self.all_lines = {}
        self.exclusions = _load_exclusions(self.root)
        for fp in sorted(self.root.glob("**/*.md")):
            rel = fp.relative_to(self.root).as_posix()
            if self.exclusions and _is_excluded(rel, self.exclusions):
                continue
            self.all_lines[rel] = read_file(fp)
            self.chunks.extend(parse_chunks(fp, self.root))
        if self.chunks:
            self.bm25 = BM25Okapi([c.tokens for c in self.chunks])

    def search(self, query: str, top_k: int = 5, file_filter: Optional[str] = None) -> list[dict]:
        """BM25 search with snippet extraction and location metadata."""
        if not self.bm25:
            return []
        query_tokens = tokenize(query)
        scores = self.bm25.get_scores(query_tokens)
        results = []
        for chunk, score in zip(self.chunks, scores):
            if file_filter and file_filter not in chunk.file:
                continue
            if score > 0:
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

    def find_chunk_for_file(self, file: str) -> Optional[Chunk]:
        """Return the first chunk for a file — used to enrich external results."""
        for chunk in self.chunks:
            if chunk.file == file or chunk.file.endswith(file):
                return chunk
        return None

    def list_files(self) -> list[dict]:
        """List indexed Markdown files with line counts and sizes (exclusions applied)."""
        files = []
        for rel in sorted(self.all_lines):
            fp = self.root / rel
            size_kb = round(fp.stat().st_size / 1024, 1) if fp.exists() else 0.0
            files.append({"file": rel, "line_count": len(self.all_lines[rel]), "size_kb": size_kb})
        return files

    def outline(self, file_path: str) -> list[dict]:
        """Extract heading hierarchy with line numbers."""
        lines = self.all_lines.get(Path(file_path).as_posix(), [])
        result = []
        for i, line in enumerate(lines, 1):
            m = re.match(r'^(#{1,6})\s+(.+)$', line)
            if m:
                result.append({
                    "level": len(m.group(1)),
                    "heading": m.group(2).strip(),
                    "line_number": i
                })
        return result

    def grep(self, pattern: str, file_filter: Optional[str] = None, limit: int = 50) -> list[dict]:
        """Regex search across all files."""
        try:
            regex = re.compile(pattern, re.IGNORECASE)
        except re.error as e:
            return [{"error": f"Invalid regex: {e}"}]
        results = []
        for rel, lines in self.all_lines.items():
            if file_filter and file_filter not in rel:
                continue
            for i, line in enumerate(lines, 1):
                if regex.search(line):
                    results.append({
                        "file": rel,
                        "line_number": i,
                        "line_text": line[:500]
                    })
                    if len(results) >= limit:
                        return results
        return results

    def read_section(self, file_path: str, heading: str) -> Optional[dict]:
        """Read content under a heading (substring match)."""
        heading_lower = heading.lower()
        for chunk in self.chunks:
            if chunk.file == file_path and heading_lower in chunk.heading.lower():
                return {
                    "file": chunk.file,
                    "heading": chunk.heading,
                    "heading_line": chunk.line_start,
                    "content_start": chunk.line_start + 1,
                    "content_end": chunk.line_end,
                    "content": chunk.text
                }
        return None

    def read_lines(self, file_path: str, start_line: int, end_line: int) -> Optional[dict]:
        """Read exact line range (1-indexed, inclusive)."""
        rel = Path(file_path).as_posix()
        lines = self.all_lines.get(rel)
        if not lines:
            return None
        start_line = max(1, start_line)
        end_line = min(len(lines), end_line)
        return {
            "file": rel,
            "start_line": start_line,
            "end_line": end_line,
            "content": "\n".join(lines[start_line - 1:end_line])
        }

    def replace_lines(self, file_path: str, start_line: int, end_line: int, new_content: str) -> dict:
        """Overwrite a line range in-place; rebuilds index after write."""
        rel = Path(file_path).as_posix()
        full_path = self.root / rel
        lines = read_file(full_path)
        start_line = max(1, start_line)
        end_line = min(len(lines), end_line)
        new_lines = lines[:start_line - 1] + _split_content(new_content) + lines[end_line:]
        full_path.write_text("\n".join(new_lines) + "\n", encoding='utf-8')
        self.rebuild()
        return {
            "file": rel,
            "replaced_lines": end_line - start_line + 1,
            "new_line_count": len(new_lines)
        }

    def insert_after_heading(self, file_path: str, heading: str, content: str) -> dict:
        """Insert content immediately after a heading line; rebuilds index after write."""
        rel = Path(file_path).as_posix()
        full_path = self.root / rel
        lines = read_file(full_path)
        heading_lower = heading.lower()
        insert_at = None
        for i, line in enumerate(lines):
            if re.match(r'^#{1,6}\s+', line) and heading_lower in line.lower():
                insert_at = i + 1
                break
        if insert_at is None:
            return {"error": f"Heading '{heading}' not found in {rel}"}
        new_lines = lines[:insert_at] + [""] + _split_content(content) + lines[insert_at:]
        full_path.write_text("\n".join(new_lines) + "\n", encoding='utf-8')
        self.rebuild()
        return {"file": rel, "inserted_at_line": insert_at + 1}

    def append_to_file(self, file_path: str, content: str) -> dict:
        """Append content to end of file with blank-line separator; rebuilds index after write."""
        rel = Path(file_path).as_posix()
        full_path = self.root / rel
        full_path.parent.mkdir(parents=True, exist_ok=True)
        current = read_file(full_path) if full_path.exists() else []
        content_lines = _split_content(content)
        new_lines = current + [""] + content_lines
        full_path.write_text("\n".join(new_lines) + "\n", encoding='utf-8')
        self.rebuild()
        return {"file": rel, "appended_at_line": len(current) + 2}
