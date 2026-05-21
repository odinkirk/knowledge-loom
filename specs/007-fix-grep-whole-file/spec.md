# Feature Specification: Fix Grep Whole-File Returns

**Feature Branch**: `007-fix-grep-whole-file`  
**Created**: 2026-05-21  
**Status**: Draft  
**Input**: User description: "The grep must not return the whole file."

## Clarifications

### Session 2026-05-21

- Q: When `limit=0` is explicitly passed, should it mean "no limit" or "return empty"? → A: 0 means no limit (return all matches, uncapped).
- Q: When results are capped by a limit, what ordering should be used? → A: Filesystem scan order — matches returned in the order they are encountered during vault traversal (no explicit sorting).
- Q: Should grep responses include a truncation indicator when results exceed the limit? → A: Yes — include both `truncated: true` and `total_matches: N` (full count of all matches across all files).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Scoped Grep Results (Priority: P1)

A user runs `loom_grep` with a pattern that matches many lines across many files (e.g., a common word like "the"). Instead of receiving an overwhelming dump of every matching line across every file — effectively returning entire file contents — the user receives a manageable, capped set of results limited to the most relevant matches.

**Why this priority**: This is the core bug. Without a result limit, `loom_grep` is practically unusable for broad patterns because it floods the client with potentially thousands of lines, making it impossible to find useful information.

**Independent Test**: Run `loom_grep` with a pattern that matches many lines (e.g., ".") in a vault with multiple files. Verify the tool returns at most the configured limit of results instead of every matching line in every file.

**Acceptance Scenarios**:

1. **Given** a vault with 3 files each containing 100 lines matching "note", **When** the user runs `loom_grep` with pattern "note" and no explicit limit, **Then** the response contains at most 200 matches, `truncated` is true, and `total_matches` reports 300.
2. **Given** a vault with 10 files, **When** the user runs `loom_grep` with a pattern matching all lines (e.g., ".") and limit=20, **Then** exactly 20 results are returned (or fewer if fewer matches exist).
3. **Given** a vault where no lines match the pattern, **When** the user runs `loom_grep`, **Then** an empty result set is returned — not an error or the whole file.

---

### User Story 2 - File Filter Narrowing (Priority: P2)

A user wants to search within a specific file or subset of files using `loom_grep`. They provide a `file_filter` parameter and receive results only from files whose paths contain the filter string.

**Why this priority**: The `file_filter` parameter is already declared in the MCP tool schema but is not implemented — it is silently ignored. This is a broken promise in the API contract. Fixing it enables targeted searches that further prevent whole-file flooding.

**Independent Test**: Run `loom_grep` with pattern "error" and file_filter="server.log" in a vault containing multiple log files. Verify only matches from "server.log" are returned.

**Acceptance Scenarios**:

1. **Given** a vault with files "a/notes.md", "b/notes.md", and "c/other.md", **When** the user runs `loom_grep` with pattern "TODO" and file_filter="notes", **Then** only matches from "a/notes.md" and "b/notes.md" are returned.
2. **Given** a vault with files "a.md" and "b.md", **When** the user runs `loom_grep` with file_filter="nonexistent", **Then** an empty result set is returned.
3. **Given** an empty file_filter or no file_filter, **When** the user runs `loom_grep`, **Then** all files are searched (backward compatible — no change from current behavior when file_filter is absent).

---

### User Story 3 - Consistent Result Formatting (Priority: P3)

A user receives grep results that use relative paths (stripped of the knowledge base root prefix) so the output is immediately reusable as input to other MCP tools without manual path trimming.

**Why this priority**: The plan for the prior spec (006-fix-mcp-tool, UX-2) already identifies relative path output as a needed improvement. This completes the grep usability picture alongside the limit and file_filter fixes.

**Independent Test**: Run `loom_grep` and verify all returned file paths are relative to the knowledge base root and match the format accepted by other MCP tools (`read_section`, `read_lines`, etc.).

**Acceptance Scenarios**:

1. **Given** a vault at `/home/user/vault/` with file `/home/user/vault/notes.md`, **When** the user runs `loom_grep`, **Then** the returned file path is `notes.md` (not `/home/user/vault/notes.md`).
2. **Given** a vault at `/home/user/vault/` with file `/home/user/vault/sub/deep/file.md`, **When** the user runs `loom_grep`, **Then** the returned file path is `sub/deep/file.md`.

---

### Edge Cases

- What happens when the pattern is an empty string? Should return empty results (not match every line).
- What happens when limit is set to 0? Treated as "no limit" — returns all matching results uncapped.
- What happens when the vault contains binary files or non-text files? Should gracefully skip or report errors for unreadable files.
- What happens when the regex pattern is invalid? Should return an empty result set (not crash or return error to client).
- What happens when file_filter matches no files? Should return an empty result set.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST default to a maximum result limit (e.g., 200 matches) when no explicit limit is provided, preventing unbounded result flooding.
- **FR-002**: System MUST accept an optional `limit` parameter that overrides the default result cap; a value of 0 means "no limit" (return all matching results).
- **FR-003**: System MUST accept an optional `file_filter` parameter to restrict search to files whose paths contain the filter string.
- **FR-004**: System MUST return only matching lines (file path, line number, line text) — never full file content.
- **FR-005**: System MUST return results as a JSON object with a `matches` array of `[file_path, line_number, line_text]` tuples, a `truncated` boolean indicating whether results were capped by the limit, and a `total_matches` integer reflecting the full count of all matching lines across all files.
- **FR-006**: System MUST use paths relative to the knowledge base root in all result entries.
- **FR-007**: System MUST maintain backward compatibility — existing callers passing only `pattern` must continue to work identically (except for the new default result cap).
- **FR-008**: System MUST return an empty result set (not an error) when the regex pattern is invalid.
- **FR-009**: System MUST return an empty result set when no matches are found.

### Key Entities

- **GrepResult**: A single match entry containing a relative file path, a 1-indexed line number, and the matched line text.
- **GrepResponse**: The full response containing a `matches` array of GrepResult tuples, a `truncated` boolean (true when results exceed the limit), and `total_matches` (the total number of matching lines across all files).
- **GrepQuery**: Input parameters — a required regex pattern string, an optional file_filter string, and an optional limit integer.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Grep results never exceed the configured limit (default or user-specified) for any input pattern; when truncated, `truncated` is true and `total_matches` reflects the full count.
- **SC-002**: Grep results include only lines that match the regex pattern — no full-file content is ever returned.
- **SC-003**: All returned file paths are relative to the knowledge base root and directly usable as input to other MCP tools.
- **SC-004**: File_filter correctly narrows results to only matching file paths; when no files match the filter, zero results are returned.
- **SC-005**: Invalid regex patterns return an empty result set without server errors or crashes.

## Assumptions

- The default result limit of 200 matches is reasonable for MCP client consumption. This can be adjusted later based on user feedback.
- Grep results are returned in filesystem scan order (the order files and lines are encountered during vault traversal). No explicit sorting is applied.
- The `file_filter` uses simple substring matching against file paths (not glob patterns), consistent with the existing MCP tool schema description.
- Relative path behavior applies existing path-stripping logic already used elsewhere in the codebase (`strip_prefix` on `kb_root`).
- The existing `Edits::grep` function signature will be extended rather than replaced, preserving backward compatibility at the Rust API level.

## Knowledge Loom Specific Requirements

### MCP Protocol Requirements *(if feature involves MCP server)*

- **MCP-001**: Tool MUST follow rmcp 1.2 specification
- **MCP-002**: Tool MUST maintain backward compatibility with existing clients
- **MCP-003**: Tool MUST include protocol tests in `tests/mcp_protocol_tests.rs`
- **MCP-004**: Tool MUST document tool signatures and return types

### Testing Requirements *(mandatory for all features)*

- **TEST-001**: Unit tests MUST achieve 80% minimum code coverage
- **TEST-002**: Integration tests MUST be added for cross-module interactions
- **TEST-003**: Tests MUST use `test-vault/` for corpus-based testing (if applicable)
- **TEST-004**: Tests MUST be deterministic (no flaky tests)
- **TEST-005**: Error paths MUST be tested alongside success paths

### Module Impact *(mandatory for all features)*

**Affected Modules** (select all that apply):
- [ ] BM25 (`src/bm25.rs`)
- [ ] Graph (`src/graph.rs`)
- [ ] Search (`src/search.rs`)
- [ ] Embed (`src/embed/`)
- [x] Server (`src/server.rs`)
- [x] Edits (`src/edits.rs`)
- [ ] Daemon (`src/daemon.rs`)
- [ ] Vault (`src/vault.rs`)
- [ ] Web (`src/web.rs`)
- [ ] Other: [specify]

**New Modules Required** (if any):
- [ ] Yes - [describe new module]
- [x] No

### Documentation Requirements *(mandatory for all features)*

- **DOC-001**: Public functions MUST have doc comments (`///`)
- **DOC-002**: Complex algorithms MUST have inline comments
- **DOC-003**: Architecture changes MUST update `ARCHITECTURE.md`
- **DOC-004**: New features MUST update `CHANGELOG.md`
- **DOC-005**: Breaking changes MUST update migration guide (if applicable)
