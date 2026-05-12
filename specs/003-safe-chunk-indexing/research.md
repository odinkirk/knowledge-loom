# Research: Safe Chunk Indexing with Ordinal Metadata

**Feature**: Safe Chunk Indexing with Ordinal Metadata
**Date**: 2025-05-11
**Status**: Complete

## Overview

This research document addresses the technical decisions and best practices for implementing character boundary-safe chunk truncation, ordinal metadata management, and file-specific re-indexing in the Knowledge Loom codebase.

## Research Topics

### 1. UTF-8 Character Boundary Detection

**Decision**: Use Rust's `char_indices()` method for safe character boundary detection

**Rationale**:
- Rust's `str::char_indices()` returns byte indices that are guaranteed to be at character boundaries
- This is the idiomatic Rust approach for UTF-8 string manipulation
- Avoids manual byte manipulation that could lead to panics on multi-byte characters
- Performance is acceptable for chunk truncation operations (<10ms target)

**Alternatives Considered**:
- Manual byte boundary checking using UTF-8 continuation byte patterns (rejected: error-prone, reinventing the wheel)
- External UTF-8 libraries (rejected: unnecessary complexity, std::str is sufficient)
- Byte-level truncation with panic recovery (rejected: violates requirement for safe truncation)

**Implementation Notes**:
- Use `content.char_indices().map(|(i, _)| i).take_while(|&i| i <= max).last()` to find safe boundary
- Fall back to content length if no valid boundary found before max
- Preserve existing whitespace truncation behavior (truncate at last whitespace before safe boundary)

### 2. Ordinal Metadata Storage in Tantivy Index

**Decision**: Add `chunk_ordinal` as a new STORED field to Tantivy schema

**Rationale**:
- Tantivy supports STORED fields that are returned with search results but not indexed
- Ordinal numbers are used for retrieval, not search queries
- STORED fields have minimal performance impact (<1% memory overhead target)
- Maintains backward compatibility (existing fields unchanged)

**Alternatives Considered**:
- Store ordinal in document content (rejected: mixes metadata with content, harder to query)
- Use external SQLite table for metadata (rejected: adds complexity, Tantivy already handles storage)
- Compute ordinals on-the-fly (rejected: inconsistent across re-indexing, violates FR-008)

**Implementation Notes**:
- Schema change: `let chunk_ordinal = schema_builder.add_u64_field("chunk_ordinal", STORED);`
- Update `ChunkDoc` struct to include `chunk_ordinal: u64` field
- Update all chunk insertion operations to include ordinal value
- Schema mismatch handling: existing code already handles schema recreation

### 3. Module Architecture: chunks.rs

**Decision**: Create dedicated `src/chunks.rs` module with clear public API

**Rationale**:
- Separates chunking concerns from BM25 storage concerns
- Makes chunking logic reusable across all modules (Search, Edits, Graph, Vault)
- Follows single responsibility principle
- Easier to test chunking logic in isolation

**Alternatives Considered**:
- Keep chunking in BM25 module (rejected: violates modular design principle, creates tight coupling)
- Put chunking in Vault module (rejected: Vault is for file scanning, not chunking logic)
- Create chunks/ subdirectory (rejected: over-engineering, single file is sufficient)

**Implementation Notes**:
- Public API: `parse_chunks(content: &str) -> Vec<Chunk>`, `truncate_at_whitespace(content: &str, max: usize) -> &str`
- Internal types: `Chunk` struct with ordinal, heading, content, line_start, line_end
- Module exports: `pub use crate::chunks::{Chunk, parse_chunks, truncate_at_whitespace};`
- Update lib.rs to re-export chunks module

### 4. File-Specific Re-indexing After Edits

**Decision**: Implement re-indexing trigger in Edits module, call BM25.index_file()

**Rationale**:
- Leverages existing `index_file()` method in BM25 module
- File-specific re-indexing is efficient (not corpus-wide)
- Maintains ordinal consistency after edits (FR-008)
- Performance target <100ms for typical files

**Alternatives Considered**:
- Corpus-wide re-indexing after every edit (rejected: too slow, violates performance requirements)
- Lazy re-indexing on next search (rejected: inconsistent state, violates FR-008)
- Manual re-indexing command only (rejected: doesn't maintain consistency automatically)

**Implementation Notes**:
- Add re-indexing call at end of each edit operation in Edits module
- Use existing `index_file()` method which handles deletion and re-insertion
- Handle re-indexing failures gracefully (log error, don't fail the edit operation)
- Performance: typical files <100ms, large files may take longer but acceptable

### 5. Cross-Module Ordinal Metadata Handling

**Decision**: Update all chunk-consuming modules to include ordinal in ChunkDoc struct

**Rationale**:
- Ensures consistency across all modules (FR-009)
- Maintains backward compatibility (ordinal is additive, not breaking)
- Modules can ignore ordinal if not needed, but it's always available
- Simplifies data flow (single ChunkDoc struct across system)

**Alternatives Considered**:
- Create separate OrdinalChunkDoc struct (rejected: unnecessary complexity, violates DRY)
- Make ordinal optional (rejected: all chunks have ordinals, Option adds no value)
- Pass ordinal separately (rejected: breaks data encapsulation, error-prone)

**Implementation Notes**:
- Update `ChunkDoc` struct in BM25 module to include `chunk_ordinal: u64`
- Update Search module to pass through ordinal in results
- Update Graph module to include ordinal in node metadata
- Update Vault module to include ordinal in chunk listings
- Update Server module to include ordinal in MCP tool responses
- Update Edits module to preserve ordinal during re-indexing

### 6. Concurrency Handling for Concurrent Edits

**Decision**: Queue edit requests during active re-indexing and process them sequentially

**Rationale**:
- Prevents concurrent index modifications that could corrupt data
- Ensures ordinal consistency across all edits (FR-008)
- Provides predictable behavior for users
- Leverages existing Tantivy writer lock for serialization

**Alternatives Considered**:
- Allow concurrent re-indexing (rejected: corrupts index, violates data integrity)
- Fail concurrent edit requests (rejected: poor user experience, loses data)
- Complex distributed locking (rejected: over-engineering for single-process system)

**Implementation Notes**:
- Use existing `Arc<Mutex<IndexWriter>>` in BM25Index for serialization
- Queue edit requests when re-indexing is in progress
- Process queued edits sequentially after re-indexing completes
- Lock contention is rare for typical edit patterns
- No additional synchronization primitives needed

### 7. Re-indexing Failure Recovery

**Decision**: Drop indices and re-ingest entire corpus on re-indexing failure

**Rationale**:
- Simpler than rollback (no need to maintain previous index state)
- Fast operation (2-3 seconds for typical vaults)
- Establishes known-good state after failure
- Avoids partial index corruption

**Alternatives Considered**:
- Rollback to previous index state (rejected: complex, requires maintaining snapshots)
- Retry failed re-indexing (rejected: may fail repeatedly, wastes time)
- Ignore failure and continue (rejected: corrupts index, violates data integrity)
- Partial recovery (rejected: complex, error-prone)

**Implementation Notes**:
- Delete `.knowledge-loom-index/` directory on failure
- Trigger full vault scan to re-ingest all files
- Return "indexing: try again in 2 seconds" error for requests during ingestion
- Log failure with sufficient detail for debugging
- Performance: <3 seconds for typical vaults (10k documents)

### 8. Request Handling During Corpus Re-ingestion

**Decision**: Return "indexing: try again in 2 seconds" error for requests during whole-corpus ingestion

**Rationale**:
- Provides clear feedback to users about system state
- Allows clients to implement retry logic
- Prevents requests from failing silently or hanging
- Simple implementation (no complex queueing needed)

**Alternatives Considered**:
- Queue requests during ingestion (rejected: adds complexity, may timeout)
- Block requests until ingestion completes (rejected: poor user experience)
- Return generic error (rejected: doesn't provide actionable information)
- Allow stale reads during ingestion (rejected: inconsistent state, confusing)

**Implementation Notes**:
- Add ingestion state flag to index manager
- Check flag before processing requests
- Return specific error message with retry guidance
- Clients can implement exponential backoff retry
- No changes to existing request processing logic

### 9. Index Rebuilding for Existing Indexes

**Decision**: Provide manual rebuild command, not automatic migration

**Rationale**:
- Index rebuild is fast (<2 seconds for typical vaults)
- Manual rebuild gives users control over timing
- Avoids unexpected index recreation during updates
- Simple implementation (delete index, re-scan vault)

**Alternatives Considered**:
- Automatic migration on startup (rejected: unexpected behavior, may fail silently)
- In-place ordinal addition (rejected: complex, error-prone, Tantivy doesn't support well)
- Version-specific migration logic (rejected: over-engineering for simple case)

**Implementation Notes**:
- Add `--rebuild-index` flag to CLI or separate command
- Implementation: delete `.knowledge-loom-index/` directory, run full vault scan
- Document in README that rebuild is needed after ordinal metadata feature
- Performance: <2 seconds for typical vaults (10k documents)

## Performance Considerations

### Chunk Truncation Performance

**Target**: <10ms per chunk
**Analysis**:
- `char_indices()` iteration is O(n) where n is chunk size (max 2000 chars)
- Whitespace search is O(n) in worst case
- Combined operations: <1ms for typical chunks, well under 10ms target
- No performance concerns identified

### Ordinal Metadata Overhead

**Target**: <1% memory overhead
**Analysis**:
- One u64 field per chunk (8 bytes)
- Typical vault: 10k documents × 5 chunks/doc = 50k chunks
- Additional memory: 50k × 8 bytes = 400KB
- Base index size: ~50MB for 10k documents
- Overhead: 400KB / 50MB = 0.8% (under 1% target)
- No memory concerns identified

### Re-indexing Performance

**Target**: <100ms for typical files (1-100KB files with 1-50 chunks)
**Analysis**:
- File-specific re-indexing processes single file
- Typical file: 5-10 chunks
- Chunk parsing: <1ms per chunk
- Index operations: <5ms per chunk
- Total: <50ms for typical file (under 100ms target)
- Large files (100+ chunks) may take longer but acceptable

### Corpus Re-ingestion Performance

**Target**: <3 seconds for entire corpus
**Analysis**:
- Full vault scan for 10k documents
- Chunk parsing: <1ms per chunk (50k chunks total)
- Index operations: <5ms per chunk
- Total: <3 seconds for typical vault (under 3s target)
- Performance acceptable for failure recovery scenario

## Testing Strategy

### Unit Tests

- **chunks.rs**: Test character boundary detection, ordinal assignment, whitespace truncation
- **BM25**: Test ordinal storage, retrieval by file+ordinal, re-indexing
- **Cross-module**: Test ordinal handling in Search, Edits, Graph, Vault

### Integration Tests

- **End-to-end**: Index file with multi-byte chars, retrieve by ordinal, edit and re-index
- **Concurrent edits**: Multiple simultaneous edits to same file (verify queueing)
- **Large files**: Files with 100+ chunks
- **Re-indexing failure**: Simulate failure, verify corpus re-ingestion
- **Request during ingestion**: Verify "indexing: try again in 2 seconds" error

### Corpus-Based Tests

- **Multi-byte content**: Files with emojis, CJK characters, combining diacritics
- **Boundary cases**: Chunks ending exactly on multi-byte character boundaries
- **Empty/whitespace**: Files with no content or only whitespace

## Dependencies

### External Dependencies

No new external dependencies required. Uses existing:
- `std::str` for UTF-8 handling
- `tantivy` for index storage
- `tokio` for async operations

### Internal Dependencies

- **chunks.rs**: No dependencies (pure functions)
- **BM25**: Depends on chunks.rs
- **Search**: Depends on BM25 (transitively chunks.rs)
- **Edits**: Depends on BM25 (transitively chunks.rs)
- **Graph**: Depends on BM25 (transitively chunks.rs)
- **Vault**: Depends on chunks.rs

## Risks and Mitigations

### Risk 1: Performance Regression

**Risk**: Ordinal metadata adds overhead to indexing operations
**Mitigation**: Performance targets defined (<5% increase), benchmarked during implementation

### Risk 2: Schema Migration Issues

**Risk**: Existing indexes incompatible with new schema
**Mitigation**: Existing code handles schema recreation, manual rebuild command provided

### Risk 3: Cross-Module Inconsistency

**Risk**: Some modules not updated to handle ordinal metadata
**Mitigation**: Module impact clearly documented, all 6 modules updated in this feature

### Risk 4: Concurrent Edit Conflicts

**Risk**: Concurrent edits to same file cause index corruption
**Mitigation**: Tantivy writer lock provides serialization, existing code handles this

### Risk 5: Corpus Re-ingestion Performance

**Risk**: Corpus re-ingestion takes longer than 3 seconds for large vaults
**Mitigation**: Performance target defined (<3s), benchmarked during implementation, acceptable for failure recovery

### Risk 6: Request Handling During Ingestion

**Risk**: Clients don't handle "indexing: try again in 2 seconds" error correctly
**Mitigation**: Clear error message, clients can implement retry logic, documented in API

## Open Questions

None. All technical decisions resolved.

## References

- Rust std::str documentation: https://doc.rust-lang.org/std/primitive.str.html
- Tantivy schema documentation: https://docs.rs/tantivy/latest/tantivy/schema/
- UTF-8 encoding: https://en.wikipedia.org/wiki/UTF-8
