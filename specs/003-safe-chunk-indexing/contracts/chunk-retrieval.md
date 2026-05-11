# Contract: Chunk Retrieval by File and Ordinal

**Feature**: Safe Chunk Indexing with Ordinal Metadata
**Date**: 2025-05-11
**Type**: Internal API Contract
**Status**: Complete

## Overview

This contract defines the interface for retrieving chunks by file path and ordinal number. This capability is exposed through the MCP server tools and used internally by the Edits module for re-indexing operations.

## Interface Definition

### Function Signature

```rust
pub async fn get_chunk_by_ordinal(
    &self,
    file_path: &str,
    chunk_ordinal: u64,
) -> Result<ChunkDoc, TantivyError>
```

### Location

**Module**: `src/bm25.rs`
**Struct**: `BM25Index`
**Method**: `get_chunk_by_ordinal`

## Parameters

### Input Parameters

| Parameter | Type | Description | Validation |
|-----------|------|-------------|------------|
| `file_path` | `&str` | Relative file path from kb_root | Non-empty, valid path format |
| `chunk_ordinal` | `u64` | Ordinal number of chunk to retrieve (1-based) | Must be >= 1 |

### Return Value

**Success**: `Ok(ChunkDoc)`

**Error**: `Err(TantivyError)`

| Error Condition | Error Type | Description |
|-----------------|------------|-------------|
| File not found | `TantivyError` | No chunks found for given file path |
| Ordinal out of bounds | `TantivyError` | Requested ordinal exceeds chunk count |
| Index error | `TantivyError` | Index corruption or access failure |
| Indexing in progress | `TantivyError` | Corpus re-ingestion underway, return "indexing: try again in 2 seconds" |

## Data Structures

### ChunkDoc

```rust
pub struct ChunkDoc {
    pub path: String,
    pub heading: Option<String>,
    pub content: String,
    pub line_start: usize,
    pub line_end: usize,
    pub chunk_ordinal: u64,
}
```

**Field Descriptions**:

| Field | Type | Description |
|-------|------|-------------|
| `path` | `String` | Relative file path from kb_root |
| `heading` | `Option<String>` | Heading context (breadcrumb path) |
| `content` | `String` | Markdown content (max 2000 chars) |
| `line_start` | `usize` | Starting line number in source file |
| `line_end` | `usize` | Ending line number in source file |
| `chunk_ordinal` | `u64` | Sequential position (1-based) |

## Behavior Specification

### Success Cases

**Case 1: Valid File and Ordinal**

**Input**:
- `file_path`: "docs/example.md"
- `chunk_ordinal`: 3

**Preconditions**:
- File exists in vault
- File has at least 3 chunks
- Index is up-to-date

**Expected Output**:
```rust
Ok(ChunkDoc {
    path: "docs/example.md".to_string(),
    heading: Some("Introduction > Getting Started".to_string()),
    content: "This is the third chunk...".to_string(),
    line_start: 45,
    line_end: 67,
    chunk_ordinal: 3,
})
```

**Postconditions**:
- Returned chunk has `chunk_ordinal == 3`
- Content is valid UTF-8
- Line numbers match source file

**Case 2: First Chunk**

**Input**:
- `file_path`: "docs/example.md"
- `chunk_ordinal`: 1

**Expected Output**:
```rust
Ok(ChunkDoc {
    path: "docs/example.md".to_string(),
    heading: Some("Introduction".to_string()),
    content: "First chunk content...".to_string(),
    line_start: 1,
    line_end: 22,
    chunk_ordinal: 1,
})
```

**Case 3: Last Chunk**

**Input**:
- `file_path`: "docs/example.md"
- `chunk_ordinal`: 5 (file has exactly 5 chunks)

**Expected Output**:
```rust
Ok(ChunkDoc {
    path: "docs/example.md".to_string(),
    heading: Some("Conclusion".to_string()),
    content: "Final chunk content...".to_string(),
    line_start: 89,
    line_end: 112,
    chunk_ordinal: 5,
})
```

### Error Cases

**Case 1: File Not Found**

**Input**:
- `file_path`: "docs/nonexistent.md"
- `chunk_ordinal`: 1

**Expected Output**:
```rust
Err(TantivyError::SystemError("No chunks found for file".to_string()))
```

**Behavior**:
- Returns error immediately
- No index modification
- Error message indicates file not found

**Case 2: Ordinal Out of Bounds (Low)**

**Input**:
- `file_path`: "docs/example.md"
- `chunk_ordinal`: 0

**Expected Output**:
```rust
Err(TantivyError::SystemError("Ordinal must be >= 1".to_string()))
```

**Behavior**:
- Validates ordinal before querying index
- Returns error immediately
- No index modification

**Case 3: Ordinal Out of Bounds (High)**

**Input**:
- `file_path`: "docs/example.md"
- `chunk_ordinal`: 10 (file has only 5 chunks)

**Expected Output**:
```rust
Err(TantivyError::SystemError("Ordinal 10 exceeds chunk count 5".to_string()))
```

**Behavior**:
- Queries index for file chunks
- Determines chunk count
- Returns error if ordinal > chunk count
- No index modification

**Case 4: Index Corruption**

**Input**:
- `file_path`: "docs/example.md"
- `chunk_ordinal`: 1

**Expected Output**:
```rust
Err(TantivyError::IndexError("Index corrupted".to_string()))
```

**Behavior**:
- Returns TantivyError from index operation
- No index modification
- Error indicates index needs rebuild

**Case 5: Indexing in Progress**

**Input**:
- `file_path`: "docs/example.md"
- `chunk_ordinal`: 1

**Expected Output**:
```rust
Err(TantivyError::SystemError("indexing: try again in 2 seconds".to_string()))
```

**Behavior**:
- Checks if corpus re-ingestion is in progress
- Returns specific error message with retry guidance
- No index modification
- Client can implement retry logic

## Performance Requirements

### Latency

**Target**: <50ms for typical retrieval (PERF-003)

**Measurement**:
- Start timer before index query
- End timer after ChunkDoc construction
- Include all validation and error handling

**Acceptance Criteria**:
- 95th percentile < 50ms
- 99th percentile < 100ms
- Mean < 25ms

### Throughput

**Target**: Support concurrent retrievals

**Measurement**:
- Multiple simultaneous requests to different files
- No blocking on shared resources

**Acceptance Criteria**:
- 10 concurrent requests complete in <500ms total
- No deadlocks or race conditions

## Concurrency Model

### Thread Safety

**Read Operations**:
- Multiple concurrent reads allowed
- No modification of index during read
- Uses Tantivy reader (thread-safe)

**Write Operations**:
- Exclusive access required
- Writer lock serializes modifications
- Reads wait for write completion

### Lock Ordering

1. Acquire reader lock (for index access)
2. Query index for file chunks
3. Release reader lock
4. Construct ChunkDoc
5. Return result

**Deadlock Prevention**:
- Never hold reader lock while acquiring writer lock
- Always release locks in reverse acquisition order
- Timeout on lock acquisition (use existing Tantivy behavior)

## Error Handling

### Error Propagation

**Strategy**: Propagate TantivyError directly

**Rationale**:
- TantivyError already covers all error cases
- Caller can handle specific error types
- Consistent with existing BM25 API

**Error Types**:

| Error Type | When Returned | Caller Action |
|-------------|---------------|---------------|
| `SystemError` | File not found, ordinal invalid | Log error, return to user |
| `IndexError` | Index corruption | Trigger index rebuild |
| `IOError` | File system error | Log error, retry or fail |
| `LockError` | Lock acquisition failed | Retry with backoff |

### Error Recovery

**Recoverable Errors**:
- Lock failures: Retry with exponential backoff
- Temporary IO errors: Retry up to 3 times

**Non-Recoverable Errors**:
- Index corruption: Require manual rebuild
- File not found: Return error to caller
- Invalid ordinal: Return error to caller

## Security Considerations

### Input Validation

**File Path**:
- Validate path format (no directory traversal)
- Normalize path separators
- Restrict to kb_root directory

**Ordinal**:
- Validate ordinal >= 1
- Validate ordinal <= reasonable maximum (e.g., 1,000,000)
- Prevent integer overflow

### Access Control

**Read Access**:
- No authentication required (public read)
- All files in kb_root accessible

**Write Access**:
- Not applicable (read-only operation)

### Data Privacy

**Content Exposure**:
- Chunk content may contain sensitive information
- No additional encryption (stored in plain text)
- Caller responsible for access control

## Testing Requirements

### Unit Tests

**Test Cases**:
1. Valid file and ordinal → returns correct chunk
2. First chunk (ordinal = 1) → returns first chunk
3. Last chunk (ordinal = N) → returns last chunk
4. File not found → returns error
5. Ordinal = 0 → returns error
6. Ordinal > chunk count → returns error
7. Empty file (no chunks) → returns error
8. Index corruption → returns error

### Integration Tests

**Test Cases**:
1. Retrieve chunk after indexing → returns correct chunk
2. Retrieve chunk after re-indexing → returns updated chunk
3. Concurrent retrievals → no race conditions
4. Retrieval during re-indexing → waits for completion

### Performance Tests

**Test Cases**:
1. Single retrieval latency < 50ms
2. 10 concurrent retrievals < 500ms total
3. Large file (100+ chunks) retrieval < 50ms

## Versioning

### Current Version

**Version**: 1.0

**Changes**:
- Initial version with ordinal metadata support
- Added `chunk_ordinal` field to ChunkDoc
- Added `get_chunk_by_ordinal` method

### Backward Compatibility

**Compatible Changes**:
- Adding new fields to ChunkDoc (ordinal)
- Adding new retrieval methods

**Breaking Changes**:
- Changing ChunkDoc field types
- Removing fields from ChunkDoc
- Changing method signatures

**Migration Strategy**:
- No breaking changes in this version
- Future breaking changes require major version bump

## References

- Tantivy Error Types: https://docs.rs/tantivy/latest/tantivy/enum.TantivyError.html
- Rust Error Handling: https://doc.rust-lang.org/book/ch09-00-recovery.html
- Performance Requirements: spec.md §PERF-003
