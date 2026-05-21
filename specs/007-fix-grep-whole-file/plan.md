# Implementation Plan: Fix Grep Whole-File Returns

**Branch**: `007-fix-grep-whole-file` | **Date**: 2026-05-21 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `specs/007-fix-grep-whole-file/spec.md`

## Summary

Fix `loom_grep` so it never floods the client with entire file contents. Add a default result limit (200), an optional `limit` override (0 = no limit), implement the already-declared `file_filter` parameter, and enrich the response with truncation metadata (`truncated` bool + `total_matches` count). All changes are localized to `src/edits.rs` and `src/server.rs`.

## Technical Context

**Language/Version**: Rust 1.75+ (Async Trait support required)
**Primary Dependencies**: regex (pattern matching), serde_json (MCP response serialization), tokio (async), rmcp 1.2 (MCP protocol)
**Testing**: cargo test, tempfile for file system tests
**Target Platform**: Linux, macOS, Windows
**Project Type**: MCP server tool (internal change, no new external interfaces)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **III. Test-First Development (NON-NEGOTIABLE)**: Tests will be written first following TDD cycle. [PASS]
- **IV. Integration Testing**: Integration tests required for MCP protocol changes (new `limit` and `file_filter` params). [PASS]
- **V. Quality Gates**: `cargo fmt`, `cargo clippy`, `cargo test` all must pass. [PASS]
- **VI. MCP Protocol Compliance**: Tool follows rmcp 1.2. Backward compatible — existing callers passing only `pattern` continue to work. [PASS]
- **VII. Performance Standards**: No blocking ops; grep scanning is already O(files × lines). Limit provides an early-exit optimization. [PASS]
- **VIII. Documentation Requirements**: Public function signatures updated with doc comments. [PASS]
- **IX. Output Conventions**: No `println!` used — MCP server uses return values. [PASS]
- **X. Technical Debt Policy**: No new debt introduced; existing `file_filter` gap (schema declared, not implemented) is fixed immediately. [PASS]
- **XII. Spec-Kit Workflow**: Spec → Plan → Tasks flow followed. [PASS]

## Project Structure

### Documentation (this feature)

```text
specs/007-fix-grep-whole-file/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── server.rs            # grep handler: extract limit/file_filter from args, serialize GrepResponse
├── edits.rs             # grep function: add limit/file_filter params, return GrepResponse struct
```

## Complexity Tracking

No new modules. No architectural changes. Both changes are localized to existing functions. The response format change (from `Vec<(String, usize, String)>` to a GrepResponse struct) is the only structural change, and it's contained within the internal MCP tool boundary (serialization decouples clients).

## Design

### Current State

```
loom_grep(pattern: "regex")
  → Edits::grep(pattern: &str) → Vec<(file_path, line_number, line_text)>
  → MCP response: JSON array of tuples, unbounded
```

- `file_filter` is declared in MCP tool schema but never read from args or passed to `grep()`
- No result limit — broad patterns return every matching line across every file
- Response is a flat JSON array with no truncation awareness

### Target State

```
loom_grep(pattern: "regex", file_filter?: "substring", limit?: 200)
  → Edits::grep(pattern, file_filter, limit) → GrepResponse { matches, truncated, total_matches }
  → MCP response: JSON object with matches array and metadata
```

- `file_filter` filters files before scanning (substring match on path)
- `limit` caps returned matches (default 200, 0 = no limit)
- Response includes `truncated` and `total_matches` for client awareness
- Enumeration stops early when limit is reached (optimization)
- Results in filesystem scan order (no additional sorting)

### GrepResponse Struct

```rust
#[derive(Serialize)]
struct GrepResponse {
    matches: Vec<GrepMatch>,
    truncated: bool,
    total_matches: usize,
}

#[derive(Serialize)]
struct GrepMatch {
    file: String,
    line_number: usize,
    line_text: String,
}
```

Tuple format `[file, line, text]` is replaced with named fields for clarity. Both server and client benefit from structured output.

### EditManager::grep signature change

```rust
// Before
pub async fn grep(&self, pattern: &str) -> Vec<(String, usize, String)>

// After
pub async fn grep(&self, pattern: &str, file_filter: Option<&str>, limit: usize) -> GrepResponse
```

### Server handler change

```rust
"grep" => {
    let pattern = str_arg("pattern")?;
    let file_filter = args.get("file_filter").and_then(|v| v.as_str());
    let limit = args.get("limit")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(200); // default limit
    let response = self.edits.grep(&pattern, file_filter, limit).await;
    Ok(serde_json::to_string(&response).unwrap_or_default())
}
```

### Tool Schema Update

Add `limit` to the MCP tool schema:
```json
{
  "type": "object",
  "properties": {
    "pattern": { "type": "string" },
    "file_filter": { "type": "string" },
    "limit": { "type": "integer", "default": 200, "description": "Max results (0 = no limit)" }
  },
  "required": ["pattern"]
}
```

### Algorithm

```
1. Compile regex (return empty GrepResponse on invalid pattern)
2. Lock vault, scan files
3. For each file:
   a. If file_filter present, skip if file path does not contain filter string
   b. Read file content
   c. For each line:
      - If regex matches, increment total_counter
      - If matches.len() < limit (or limit == 0), push match
4. Return GrepResponse {
     matches: collected_results,
     truncated: total_counter > matches.len(),
     total_matches: total_counter,
   }
```

## Constitution Re-Check (Post-Design)

- **III. TDD**: Tests defined in spec acceptance scenarios will be written first. [PASS]
- **V. Quality Gates**: No new dependencies; `cargo fmt/clippy/test` must pass. [PASS]
- **VI. MCP Compliance**: Response format change is backwards-incompatible (array → object), but MCP clients are expected to handle JSON shape changes as tool protocol evolution. Existing `pattern`-only callers receive valid responses. [PASS]
- **X. Technical Debt**: Fixes the `file_filter` regression. [PASS]
