# Research: Fix Grep Whole-File Returns

**Feature**: 007-fix-grep-whole-file
**Date**: 2026-05-21

## Decision: Default limit of 200 matches

**Rationale**: The spec assumes 200 as a reasonable default for MCP client consumption. This is consistent with common search tool defaults (e.g., Tantivy/Top-K patterns in the codebase use 5-10 for ranked search, but grep is unranked line matching with lower signal-per-result, so a higher cap is appropriate). The limit is overridable via the `limit` parameter, including `limit=0` for uncapped results.

**Alternatives considered**:
- 100: Too restrictive for typical vault grep use cases (e.g., finding all instances of a symbol)
- 500: Could still overwhelm MCP clients for large vaults with common patterns
- No default: Rejected — the core bug is unbounded results

## Decision: Filesystem scan order (no sorting)

**Rationale**: Adding explicit sorting (by file path, line number, or relevance) adds overhead without user-facing benefit for a grep tool. The file_filter already provides narrowing. Users who need sorted results can post-process client-side.

**Alternatives considered**:
- Group by file: Increases memory usage (must buffer all results before sorting)
- Relevance scoring: Inappropriate for regex line matching (no relevance model)

## Decision: Response shape — named struct with metadata

**Rationale**: Adding `truncated` and `total_matches` to the response gives MCP clients actionable awareness when results are capped. This follows the principle of least surprise and enables clients to suggest narrowing queries. The change from tuple array to named struct improves readability in tool output.

**Alternatives considered**:
- Keep tuple format, add metadata as separate fields in JSON: Breaks the flat array contract anyway since we're adding new fields
- Inline metadata in first/last tuple: Non-standard, confusing

## Decision: file_filter uses substring matching

**Rationale**: Aligns with the existing MCP tool schema description ("optional file filter") and the Python reference implementation which also uses substring matching (`file_filter in file_rel`). Glob patterns would require additional dependency or custom implementation, adding complexity for marginal benefit given the `top_k` limit provides sufficient narrowing for the common case.

**Alternatives considered**:
- Glob patterns: More powerful but inconsistent with existing schema, requires glob crate dependency
- Regex on file paths: Overly complex for the "filter" use case

## Decision: Early-exit when limit reached

**Rationale**: When `limit > 0`, the enumeration can stop pushing to `matches` once the limit is reached. However, `total_matches` must still count all matches across all files for the truncation indicator. This means we continue scanning but skip the push.

**Alternatives considered**:
- Stop scanning entirely at limit: Cannot provide accurate `total_matches` count
- Count only pushed results: Makes `truncated` detection impossible
