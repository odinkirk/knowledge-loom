# Feature Specification: Fix MCP Tool Bugs from Live Smoke Test

**Feature Branch**: `006-fix-mcp-tool`  
**Created**: 2026-05-19  
**Status**: Draft  
**Input**: Live smoke test of MCP tools against unspoken-world corpus (see loom-review.md)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Search Returns Only Matched Chunks, Not Entire Files (Priority: P1)

A developer searches for "subdrop" with `top_k=3`. Currently `search` returns every chunk from every matched file (141 sections, ~115K chars), blowing past token limits. After the fix, `search` returns only the chunks that actually matched the query — one per file for `top_k=3`, each with heading breadcrumb and line numbers for surgical editing.

**Why this priority**: Critical — `search` is the most-used tool and currently unusable for most queries. Every query produces output exceeding Claude's tool-output token limit when matching files with many chunks.

**Independent Test**: Search for a term that matches a known file with multiple chunks. Verify only the matching chunks are returned, not the entire file.

**Acceptance Scenarios**:

1. **Given** a vault with a 70-chunk file containing "subdrop" in chunk 12, **When** `search "subdrop" top_k=3` runs, **Then** chunk 12 is returned, not all 70 chunks.
2. **Given** a vault where a file scores via vector/graph but has no direct BM25 chunk hit, **When** `search` processes it, **Then** at most the top-1 vector chunk is returned, not the entire file.

---

### User Story 2 - Graph Tools Return Valid Edges (Priority: P1)

A developer uses `find_connections`, `find_path_between`, or `search_graph` to navigate wikilinks. Currently all graph tools return empty results because the graph has 0 edges despite the corpus containing wikilinks (`[[wikilink]]` and `[text](path.md)`).

**Why this priority**: P1 — three tools are completely non-functional. Graph-based RRF scoring is dead, reducing search quality.

**Independent Test**: Run `find_connections` on a file known to contain wikilinks. Verify non-empty results.

**Acceptance Scenarios**:

1. **Given** a file with `[[other-note]]` wikilinks, **When** reindex runs, **Then** graph edges are created linking the file to "other-note".
2. **Given** a file with standard Markdown links `[text](other.md)`, **When** reindex runs, **Then** graph edges are created for those links too.
3. **Given** a connected graph after reindex, **When** `find_connections "Story Bible"` runs, **Then** linked files are returned.

---

### User Story 3 - Symlinks Not Indexed Twice (Priority: P2)

A developer has symlinks in their vault (e.g., `tools/Per-Leaf Plot Components.md` → `implementations/...`). Currently both the symlink path and the target path are indexed, producing duplicate search results and doubling payload sizes.

**Why this priority**: P2 — duplicates results and wastes index space. Not blocking but degrades quality significantly on vaults with symlinks.

**Independent Test**: Create a test vault with a symlinked file. Verify only one of the two paths appears in search results and `list_files`.

**Acceptance Scenarios**:

1. **Given** a vault with a symlink `A.md` → `sub/B.md`, **When** reindex runs, **Then** only one canonical path is indexed, not both.
2. **Given** a file reachable via both a direct path and a symlink, **When** `list_files` runs, **Then** the file appears exactly once.

---

### User Story 4 - `.knowledge-loom-ignore` Patterns Match Subdirectories (Priority: P2)

A developer adds `.venv/` to `.knowledge-loom-ignore` to exclude Python virtual environments. Currently patterns only match at the KB_ROOT level, so `tools/.venv/` is NOT excluded, polluting search results with third-party license files.

**Why this priority**: P2 — pollutes search with irrelevant files. Fix is small (pattern matching logic) but impact is visible in every `list_files` and search.

**Independent Test**: Create a vault with `.venv/` in a subdirectory. Verify files under it are excluded from `list_files` and `search`.

**Acceptance Scenarios**:

1. **Given** `.knowledge-loom-ignore` containing `.venv/`, **When** `list_files` runs on a vault with `tools/.venv/LICENSE.md`, **Then** the LICENSE file is NOT listed.
2. **Given** `.knowledge-loom-ignore` containing `*.log`, **When** `list_files` runs, **Then** `.log` files in subdirectories are excluded.

---

### User Story 5 - `index_status` Reports Accurate Counts (Priority: P2)

A developer checks `index_status` to verify indexing completed. Currently it reports `"documents": 0, "vectors": 0, "edges": 0` despite a fully functional index with 65 tracked files and 3030 chunks.

**Why this priority**: P2 — misleading status erodes trust in the tool. Developers may re-run expensive full reindexes thinking the index is broken.

**Independent Test**: After a full reindex, run `index_status` and verify document count > 0, vector count > 0, edge count > 0.

**Acceptance Scenarios**:

1. **Given** a completed reindex with known file count, **When** `index_status` runs, **Then** `bm25.documents` reflects the number of indexed BM25 documents.
2. **Given** a completed reindex, **When** `index_status` runs, **Then** `embeddings.vectors` reflects the total vector count.
3. **Given** a completed reindex with connected files, **When** `index_status` runs, **Then** `graph.edges` reflects the number of wikilink edges.

---

### User Story 6 - `read_section` Depth Control (Priority: P3)

A developer uses `read_section` to read a top-level heading like "III. The Divine Rough Patch". Currently it returns the entire section tree including all subsections (thousands of words). A `depth` parameter would allow reading only content directly under the named heading.

**Why this priority**: P3 — UX improvement, not a bug. Current behavior is sometimes useful, sometimes too verbose.

**Independent Test**: Call `read_section` with `depth=1` on a heading with subsections. Verify only direct content is returned, not subsections.

**Acceptance Scenarios**:

1. **Given** a heading with 3 subsections, **When** `read_section "Heading" depth=1` runs, **Then** only content directly under "Heading" is returned, stopping at the first subheading.
2. **Given** a heading with 3 subsections, **When** `read_section "Heading" depth=0` or no depth specified, **Then** all subsections are included (backward compatible default).

---

### User Story 7 - Tool Outputs Use Relative Paths (Priority: P3)

A developer chains `list_files` → `read_section`. Currently `list_files` returns absolute paths (`/Users/odinkirk/Documents/.../file.md`) but `read_section` accepts relative paths (`file.md`). The developer must manually strip the prefix.

**Why this priority**: P3 — minor friction in tool chaining. Does not block functionality.

**Independent Test**: Run `list_files` and verify returned paths are relative to KB_ROOT, not absolute.

**Acceptance Scenarios**:

1. **Given** KB_ROOT is `/Users/alice/vault`, **When** `list_files` runs, **Then** paths are returned as `notes/file.md`, not `/Users/alice/vault/notes/file.md`.
2. **Given** a `grep` result matching `notes/file.md`, **When** the result is returned, **Then** the path is relative, immediately usable as input to `read_section`.

---

### Edge Cases

- Empty query: `search ""` should return empty or error, not dump entire index
- File with zero chunks after `parse_chunks`: should not appear in search results
- Symlink cycle: canonicalization should handle circular symlinks gracefully
- `.knowledge-loom-ignore` with blank lines and comments: should be parsed correctly (already done)
- `read_section` on non-existent heading: should return clear error
- `read_section` with `depth=0` on a heading with no subsections: should return all content

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `search` MUST return only chunks that matched the query, not all chunks from matched files
- **FR-002**: Graph indexing MUST extract both `[[wikilink]]` and `[text](path.md)` link formats
- **FR-003**: Vault scanning MUST canonicalize symlinks to avoid duplicate indexing
- **FR-004**: `.knowledge-loom-ignore` patterns MUST match against the relative path from KB_ROOT for subdirectory patterns
- **FR-005**: `index_status` MUST report actual BM25 document count, vector count, and graph edge count
- **FR-006**: `read_section` MUST accept an optional `depth` parameter controlling heading depth
- **FR-007**: `list_files` and `grep` MUST return paths relative to KB_ROOT

### Key Entities

- **SearchResult**: section matches with path, heading, content, line_start, chunk_ordinal — must not include unmatched chunks
- **GraphEdge**: wikilink connections between files, extracted during reindex from both `[[link]]` and `[text](path.md)` formats
- **IgnorePattern**: glob/dir-prefix match against relative path from KB_ROOT (already implemented, needs subdir fix)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `search "subdrop" top_k=3` returns ≤3 sections total on the test corpus
- **SC-002**: `list_files` on test corpus shows 0 `.venv` paths
- **SC-003**: `index_status` shows non-zero document, vector, and edge counts matching reindex-state.json totals
- **SC-004**: `find_connections "Story Bible"` returns non-empty link list
- **SC-005**: All existing tests pass (61 lib + integration tests)
- **SC-006**: No new clippy warnings

## Assumptions

- The test corpus (unspoken-world) uses both `[[wikilink]]` and `[text](path.md)` link formats
- Current BM25 Tantivy index contains correct per-chunk documents (verified: `search_file` works)
- Graph link extraction was implemented but only handles one link format, not both
- `.knowledge-loom-ignore` glob patterns work at the file level but not recursively through subdirectories

## Knowledge Loom Specific Requirements

### MCP Protocol Requirements

- **MCP-001**: All modified tools MUST follow rmcp 1.2 specification
- **MCP-002**: All modified tools MUST maintain backward compatibility with existing clients
- **MCP-003**: Modified tools MUST have protocol tests in `tests/mcp_protocol_tests.rs`
- **MCP-004**: New `depth` parameter on `read_section` MUST be optional (backward compatible)

### Search Engine Requirements

- **SEARCH-001**: `search` MUST still use RRF merging for BM25 + vector + graph scores
- **SEARCH-002**: `search` MUST preserve heading breadcrumb and line_start metadata
- **SEARCH-003**: `search` MUST respect `top_k` parameter applied to the final chunk count, not file count

### Graph Analytics Requirements

- **GRAPH-001**: Link extraction MUST handle both `[[wikilink]]` and `[text](path.md)` formats
- **GRAPH-002**: Graph rebuild MUST preserve existing basename resolution for wikilinks

### Performance Requirements

- **PERF-001**: `search` must still complete <150ms for 10k documents
- **PERF-002**: Canonical path dedup must not slow down `scan_files()` by more than 10%
- **PERF-003**: `index_status` queries must complete <100ms

### Testing Requirements

- **TEST-001**: Unit tests for search chunk filtering
- **TEST-002**: Unit tests for graph link extraction (both formats)
- **TEST-003**: Integration test for symlink dedup
- **TEST-004**: Integration test for subdirectory ignore patterns
- **TEST-005**: All existing tests must continue to pass

### Module Impact

**Affected Modules**:
- [x] BM25 (`src/bm25.rs`) — index_status doc count
- [x] Graph (`src/graph.rs`) — link extraction fix
- [x] Search (`src/search.rs`) — chunk filtering fix
- [ ] Embed (`src/embed/`)
- [x] Server (`src/server.rs`) — tool implementations, path serialization, index_status queries
- [x] Edits (`src/edits.rs`) — read_section depth parameter
- [ ] Daemon (`src/daemon.rs`)
- [x] Vault (`src/vault.rs`) — symlink dedup, subdirectory ignore patterns
- [ ] Web (`src/web.rs`)
- [ ] Other:

**New Modules Required**:
- [ ] Yes
- [x] No

### Documentation Requirements

- **DOC-001**: All public function signatures updated with doc comments
- **DOC-003**: `ARCHITECTURE.md` updated with search result filtering and graph link extraction details
- **DOC-004**: `CHANGELOG.md` updated with all 7 fixes
