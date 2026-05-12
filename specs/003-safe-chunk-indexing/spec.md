# Feature Specification: Safe Chunk Indexing with Ordinal Metadata

**Feature Branch**: `003-safe-chunk-indexing`
**Created**: 2025-05-11
**Status**: Draft
**Input**: User description: "truncation at whitespace for chunks must use character boundry safe indexing and chunking to avoid panic. Additionally, the coordinate metadata should show the ordinal number of the chunk (if it's the 3rd chunk, that should be in the metadata as it may be advantageous to retrieve chunk 2 or 4. Consequently, retrieveal by file and chunk should be available."

## Clarifications

### Session 2025-05-11

- Q: Should README be updated with new chunk retrieval capabilities? → A: Yes, README must be updated with new chunk retrieval capabilities
- Q: Which modules need updates to handle ordinal metadata? → A: Update all modules that use chunks (Search, Edits, Graph, Vault) to handle ordinal metadata
- Q: Should chunking logic be extracted from BM25 into a dedicated module? → A: Yes, create a dedicated `src/chunks.rs` module for chunking logic
- Q: Should re-indexing after edits be included in this feature? → A: Add re-indexing after edits to this feature's scope, with file-specific re-indexing (not corpus-wide)
- Q: What constitutes "typical files" for performance targets? → A: 1-100KB files with 1-50 chunks (covers 95% of typical markdown documents)
- Q: Which specific operations require performance testing? → A: Chunk truncation, chunk retrieval, file re-indexing (core operations with explicit targets)
- Q: How should re-indexing failures be handled? → A: Drop indices and re-ingest entire corpus (2-3 seconds), return "indexing: try again in 2 seconds" error for requests during ingestion

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Safe Chunk Truncation (Priority: P1)

When the system processes markdown files containing multi-byte UTF-8 characters (such as em dashes, emojis, or non-Latin scripts), the chunk truncation operation must complete successfully without panicking. Users should be able to index any valid UTF-8 content regardless of character encoding complexity.

**Why this priority**: This is a critical bug fix - the current implementation panics on common multi-byte characters, making the system unusable for many real-world documents. This blocks all other functionality.

**Independent Test**: Can be fully tested by indexing a markdown file containing multi-byte characters at chunk boundaries and verifying the operation completes without errors.

**Acceptance Scenarios**:

1. **Given** a markdown file containing multi-byte UTF-8 characters (e.g., em dashes, emojis, CJK characters), **When** the file is indexed, **Then** the operation completes successfully without panicking
2. **Given** content where a multi-byte character spans the truncation boundary, **When** truncation occurs, **Then** the result is a valid UTF-8 string ending at a character boundary
3. **Given** a file with 10,000 characters including multi-byte characters, **When** chunked at 2000-character intervals, **Then** all chunks are valid UTF-8 strings

---

### User Story 2 - Ordinal Chunk Metadata (Priority: P2)

When chunks are created from markdown files, each chunk should include its ordinal position (1st, 2nd, 3rd, etc.) in the metadata. This allows users and systems to understand the sequence and context of chunks within a document.

**Why this priority**: Ordinal metadata enables advanced use cases like retrieving adjacent chunks (chunk 2 or 4 when viewing chunk 3), understanding document structure, and implementing context-aware search results. This enhances the value of the chunking system without breaking existing functionality.

**Independent Test**: Can be fully tested by indexing a multi-chunk file and verifying that each chunk's metadata includes the correct ordinal number.

**Acceptance Scenarios**:

1. **Given** a markdown file that produces 5 chunks, **When** the file is indexed, **Then** each chunk includes ordinal metadata (1, 2, 3, 4, 5)
2. **Given** a file with a single chunk, **When** indexed, **Then** the chunk has ordinal metadata of 1
3. **Given** a file with 100 chunks, **When** indexed, **Then** all chunks have sequential ordinal metadata from 1 to 100

---

### User Story 3 - Retrieval by File and Chunk Number (Priority: P3)

Users should be able to retrieve a specific chunk from a file by specifying the file path and chunk ordinal number. This enables precise content access for editing, analysis, or display purposes.

**Why this priority**: This capability enables surgical editing workflows and context-aware operations where users need to work with specific chunks within a document. It builds on the ordinal metadata from User Story 2.

**Independent Test**: Can be fully tested by retrieving chunk 3 from a known file and verifying the returned content matches the expected chunk.

**Acceptance Scenarios**:

1. **Given** a file with 5 chunks, **When** requesting chunk 3, **Then** the system returns the correct third chunk with its metadata
2. **Given** a file with 10 chunks, **When** requesting chunk 1 and chunk 10, **Then** the system returns the first and last chunks respectively
3. **Given** a request for chunk 0 or chunk 11 (out of bounds), **When** the request is made, **Then** the system returns an appropriate error or empty result

---

### Edge Cases

- What happens when content is empty or contains only whitespace?
- How does the system handle files where all characters are multi-byte (e.g., CJK text)?
- What happens when a file produces only a single chunk?
- How does the system handle very long files that produce hundreds of chunks?
- What happens when a chunk boundary falls exactly on a multi-byte character?
- How does the system handle retrieval requests for non-existent chunk numbers?
- What happens when a file path is invalid or the file doesn't exist during retrieval?
- What happens when an edit causes a chunk to exceed the maximum size and needs to be split?
- How does the system handle concurrent edits to the same file during re-indexing? (Addressed by FR-012, FR-013)
- What happens when re-indexing fails after an edit (partial update scenario)? (Addressed by FR-014, FR-015, FR-016, FR-017)
- How does the system handle requests during whole-corpus re-ingestion? (Addressed by FR-016)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST use character boundary-safe indexing when truncating chunks to avoid UTF-8 panics
- **FR-002**: System MUST preserve valid UTF-8 encoding in all truncated chunk content
- **FR-003**: System MUST assign ordinal numbers to chunks sequentially starting from 1
- **FR-004**: System MUST include ordinal chunk number in chunk metadata
- **FR-005**: System MUST support retrieval of chunks by file path and ordinal number
- **FR-006**: System MUST handle out-of-bounds chunk requests gracefully
- **FR-007**: System MUST maintain existing chunk metadata (heading, line_start, line_end) alongside ordinal number
- **FR-008**: System MUST ensure ordinal numbers are consistent across re-indexing of the same file
- **FR-009**: All modules that use chunks (Search, Edits, Graph, Vault) MUST properly handle ordinal metadata
- **FR-010**: Chunking logic MUST be extracted into a dedicated `src/chunks.rs` module for reuse across the system
- **FR-011**: System MUST re-index the specific file after edits (not corpus-wide) to maintain accurate ordinal metadata when chunks are modified, split, or merged
- **FR-012**: System MUST serialize concurrent edits to the same file during re-indexing
- **FR-013**: System MUST queue edit requests during active re-indexing and process them sequentially
- **FR-014**: System MUST drop indices and re-ingest entire corpus on re-indexing failure (completes in 2-3 seconds)
- **FR-015**: System MUST log re-indexing failures with sufficient detail for debugging
- **FR-016**: System MUST return "indexing: try again in 2 seconds" error for requests during whole-corpus ingestion
- **FR-017**: System MUST notify user of re-indexing failure via error response

### Key Entities

- **Chunk**: A segment of markdown content with metadata including ordinal position, heading context, line numbers, and file path
- **File**: A markdown document that can be split into multiple ordered chunks
- **Chunk Metadata**: Information associated with each chunk including ordinal number, heading, line_start, line_end, and path
- **Chunks Module**: A dedicated `src/chunks.rs` module providing character boundary-safe chunking logic and ordinal metadata management for reuse across the system

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: System can index markdown files containing multi-byte UTF-8 characters without panicking
- **SC-002**: All truncated chunks are valid UTF-8 strings (100% of test cases)
- **SC-003**: Chunk ordinal numbers are accurate and sequential (verified across 100+ chunk files)
- **SC-004**: Retrieval by file and chunk number returns the correct chunk 100% of the time for valid requests
- **SC-005**: Out-of-bounds chunk requests return appropriate errors 100% of the time
- **SC-006**: File-specific re-indexing after edits completes in <100ms for typical files (1-100KB files with 1-50 chunks)
- **SC-007**: Whole-corpus re-ingestion on re-indexing failure completes in <3 seconds

## Assumptions

- Existing chunk structure and metadata (heading, line_start, line_end, path) will be preserved
- Ordinal numbering starts at 1 for the first chunk in each file
- Chunk ordinal numbers are reset per file (not global across all files)
- Retrieval by file and chunk uses the existing file path structure
- The maximum chunk size (2000 characters) remains unchanged
- Chunk truncation at whitespace behavior is preserved (truncates at last whitespace before boundary)

## Knowledge Loom Specific Requirements

### MCP Protocol Requirements *(if feature involves MCP server)*

- **MCP-001**: Tool MUST follow rmcp 1.2 specification
- **MCP-002**: Tool MUST maintain backward compatibility with existing clients
- **MCP-003**: Tool MUST include protocol tests in `tests/mcp_protocol_tests.rs`
- **MCP-004**: Tool MUST document tool signatures and return types
- **MCP-005**: Tool MUST handle errors gracefully and return appropriate error codes

### Search Engine Requirements *(if feature involves search)*

- **SEARCH-001**: Search MUST use RRF merging for multiple engines (if applicable)
- **SEARCH-002**: Search MUST return results with line_start/heading metadata for surgical editing
- **SEARCH-003**: Search MUST support top_k parameter for result limiting
- **SEARCH-004**: Search MUST handle empty queries gracefully
- **SEARCH-005**: Search MUST target <150ms for 10k documents (performance requirement)

### Graph Analytics Requirements *(if feature involves graph operations)*

- **GRAPH-001**: Graph operations MUST use Petgraph for graph data structures
- **GRAPH-002**: Graph MUST support PageRank ranking
- **GRAPH-003**: Graph MUST support community detection
- **GRAPH-004**: Graph MUST support path finding between nodes
- **GRAPH-005**: Graph MUST handle disconnected graphs gracefully

### Performance Requirements *(if feature is performance-critical)*

- **PERF-001**: Chunk truncation with character boundary safety MUST complete in <10ms per chunk
- **PERF-002**: Indexing operations with ordinal metadata MUST NOT increase indexing time by more than 5%
- **PERF-003**: Retrieval by file and chunk number MUST complete in <50ms
- **PERF-004**: Memory overhead for ordinal metadata MUST be <1% of total index size
- **PERF-005**: File-specific re-indexing after edits MUST complete in <100ms for typical files (1-100KB files with 1-50 chunks)
- **PERF-006**: Whole-corpus re-ingestion on re-indexing failure MUST complete in <3 seconds

### Testing Requirements *(mandatory for all features)*

- **TEST-001**: Unit tests MUST achieve 80% minimum code coverage
- **TEST-002**: Integration tests MUST be added for cross-module interactions
- **TEST-003**: Tests MUST use `test-vault/` for corpus-based testing (if applicable)
- **TEST-004**: Tests MUST be deterministic (no flaky tests)
- **TEST-005**: Error paths MUST be tested alongside success paths
- **TEST-006**: Performance tests MUST be added for critical paths: chunk truncation, chunk retrieval, file re-indexing

### Module Impact *(mandatory for all features)*

**Affected Modules** (select all that apply):
- [x] BM25 (`src/bm25.rs`)
- [x] Graph (`src/graph.rs`)
- [x] Search (`src/search.rs`)
- [ ] Embed (`src/embed/`)
- [x] Server (`src/server.rs`)
- [x] Edits (`src/edits.rs`)
- [ ] Daemon (`src/daemon.rs`)
- [x] Vault (`src/vault.rs`)
- [ ] Web (`src/web.rs`)
- [x] Other: Chunks (`src/chunks.rs` - new module)

**New Modules Required** (if any):
- [x] Yes - Create dedicated `src/chunks.rs` module for chunking logic with character boundary-safe truncation and ordinal metadata
- [ ] No

### Documentation Requirements *(mandatory for all features)*

- **DOC-001**: Public functions MUST have doc comments (`///`)
- **DOC-002**: Complex algorithms MUST have inline comments
- **DOC-003**: Architecture changes MUST update `ARCHITECTURE.md`
- **DOC-004**: New features MUST update `CHANGELOG.md`
- **DOC-005**: Breaking changes MUST update migration guide (if applicable)
- **DOC-006**: README MUST be updated with new chunk retrieval capabilities and ordinal metadata documentation
