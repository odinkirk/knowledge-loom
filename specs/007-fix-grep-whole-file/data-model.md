# Data Model: Fix Grep Whole-File Returns

**Feature**: 007-fix-grep-whole-file
**Date**: 2026-05-21

## Entities

### GrepMatch

A single line match result from a grep operation.

| Field | Type | Description |
|-------|------|-------------|
| `file` | `String` | File path relative to the knowledge base root |
| `line_number` | `usize` | 1-indexed line number within the file |
| `line_text` | `String` | The full text of the matching line |

**Serialization**: JSON object `{"file": "...", "line_number": N, "line_text": "..."}`

### GrepResponse

The complete response from a grep operation, wrapping matches with truncation metadata.

| Field | Type | Description |
|-------|------|-------------|
| `matches` | `Vec<GrepMatch>` | Matching lines, up to the configured limit |
| `truncated` | `bool` | `true` if more matches exist than were returned |
| `total_matches` | `usize` | Total count of all matching lines across all searched files |

**Serialization**: JSON object `{"matches": [...], "truncated": bool, "total_matches": N}`

**Invariants**:
- `matches.len() <= limit` (when limit > 0)
- `truncated == true` iff `total_matches > matches.len()`
- When `truncated == false`, `total_matches == matches.len()`
- When no matches exist: `matches: []`, `truncated: false`, `total_matches: 0`

### GrepQuery (input parameters, not a persisted entity)

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `pattern` | `String` | Yes | — | Regex pattern to match against each line |
| `file_filter` | `Option<String>` | No | `None` | Substring filter on file paths; only files whose paths contain this string are searched |
| `limit` | `usize` | No | `200` | Maximum matches to return; `0` means no limit |

## Error Handling

| Condition | Behavior |
|-----------|----------|
| Invalid regex pattern | Return `GrepResponse` with empty matches, no error |
| `file_filter` matches no files | Return `GrepResponse` with empty matches |
| File read error (binary, permissions) | Skip file silently (existing behavior, unchanged) |
| Empty pattern string | Return empty matches (matches no lines) |

## State Transitions

No stateful entities. GrepResponse is a pure computation result.
