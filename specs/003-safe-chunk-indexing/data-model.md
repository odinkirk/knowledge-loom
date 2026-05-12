# Data Model: Safe Chunk Indexing with Ordinal Metadata

**Feature**: Safe Chunk Indexing with Ordinal Metadata
**Date**: 2025-05-11
**Status**: Complete

## Overview

This document describes the data entities, structures, and relationships for the safe chunk indexing feature with ordinal metadata.

## Core Entities

### Chunk

**Purpose**: Represents a segment of markdown content with metadata

**Location**: `src/chunks.rs`

**Fields**:

| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| `ordinal` | `u64` | Sequential position of chunk within file (1-based) | Must be >= 1, sequential within file |
| `heading` | `Option<String>` | Heading context for the chunk (breadcrumb path) | Optional, empty string if None |
| `content` | `String` | Markdown content of the chunk | Non-empty, max 2000 chars |
| `line_start` | `usize` | Starting line number in source file | Must be >= 1 |
| `line_end` | `usize` | Ending line number in source file | Must be >= line_start |

**Invariants**:
- `ordinal` is unique within a file
- `content` is valid UTF-8 (character boundary-safe)
- `content` length <= 2000 characters
- `line_start` <= `line_end`

**Relationships**:
- Belongs to exactly one File
- Has zero or more heading breadcrumbs

### ChunkDoc

**Purpose**: Chunk representation as stored/retrieved from BM25 index

**Location**: `src/bm25.rs`

**Fields**:

| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| `path` | `String` | Relative file path from kb_root | Non-empty, valid path |
| `heading` | `Option<String>` | Heading context for the chunk | Optional, empty string if None |
| `content` | `String` | Markdown content of the chunk | Non-empty, max 2000 chars |
| `line_start` | `usize` | Starting line number in source file | Must be >= 1 |
| `line_end` | `usize` | Ending line number in source file | Must be >= line_start |
| `chunk_ordinal` | `u64` | Sequential position of chunk within file (1-based) | Must be >= 1, sequential within file |

**Invariants**:
- `chunk_ordinal` is unique within a file
- `content` is valid UTF-8 (character boundary-safe)
- `content` length <= 2000 characters
- `line_start` <= `line_end`

**Relationships**:
- Belongs to exactly one File
- Stored in BM25 index with path as key

### File

**Purpose**: Markdown document that can be split into multiple ordered chunks

**Location**: Conceptual entity (not a struct)

**Attributes**:
- Path: File system path (relative to kb_root)
- Content: Full markdown content
- Chunk count: Number of chunks produced by parsing

**Relationships**:
- Contains one or more Chunks
- Chunks are ordered by ordinal number

## Tantivy Schema Changes

### New Field

**Field Name**: `chunk_ordinal`

**Type**: `u64`

**Options**: `STORED`

**Purpose**: Store ordinal number for retrieval by file and chunk number

**Schema Definition**:
```rust
let chunk_ordinal = schema_builder.add_u64_field("chunk_ordinal", STORED);
```

### Existing Fields (Unchanged)

| Field Name | Type | Options | Purpose |
|-------------|------|---------|---------|
| `heading` | `TEXT` | TEXT | STORED | Heading context for search and retrieval |
| `content` | `TEXT` | TEXT | STORED | Chunk content for full-text search |
| `path` | `STRING` | STRING | STORED | File path for filtering and retrieval |
| `line_start` | `u64` | STORED | Starting line number |
| `line_end` | `u64` | STORED | Ending line number |

## Data Flow

### Chunk Creation Flow

```
File Content
    ↓
chunks.rs::parse_chunks()
    ↓
Vec<Chunk> (with ordinal, heading, content, line_start, line_end)
    ↓
BM25::index_file()
    ↓
Tantivy Index (with chunk_ordinal field)
```

### Chunk Retrieval Flow

```
File Path + Chunk Ordinal
    ↓
BM25::get_chunk_by_ordinal()
    ↓
Tantivy Index Query
    ↓
ChunkDoc (with chunk_ordinal)
```

### Re-indexing Flow

```
Edit Operation (Edits module)
    ↓
File Content Updated
    ↓
BM25::index_file() (re-index)
    ↓
Old Chunks Deleted
    ↓
New Chunks Created (with updated ordinals)
    ↓
Tantivy Index Updated
```

## State Transitions

### Chunk Lifecycle

```
Created (parse_chunks)
    ↓
Indexed (BM25::index_file)
    ↓
Retrieved (BM25::get_chunk_by_ordinal)
    ↓
Deleted (BM25::index_file - delete_term)
    ↓
Re-created (BM25::index_file - re-index)
```

### Ordinal Consistency

```
Initial Indexing: Ordinals 1, 2, 3, ..., N
    ↓
Edit (no chunk count change): Ordinals 1, 2, 3, ..., N (preserved)
    ↓
Edit (chunk split): Ordinals 1, 2, 3, 3a, 3b, 4, ..., N (reassigned)
    ↓
Edit (chunk merged): Ordinals 1, 2, 3, ..., N-1 (reassigned)
    ↓
Re-index: Ordinals 1, 2, 3, ..., M (recalculated)
```

## Validation Rules

### Chunk Validation

1. **Character Boundary Safety**:
   - All truncation must occur at valid UTF-8 character boundaries
   - Use `char_indices()` to find safe boundaries
   - Never slice by byte index alone

2. **Ordinal Uniqueness**:
   - Ordinal numbers must be unique within a file
   - Ordinal numbers must be sequential starting from 1
   - No gaps in ordinal sequence

3. **Content Length**:
   - Chunk content must not exceed 2000 characters
   - Truncation must preserve UTF-8 validity
   - Truncate at last whitespace before boundary if possible

4. **Line Number Consistency**:
   - `line_start` must be >= 1
   - `line_end` must be >= `line_start`
   - Line numbers must match actual source file

### Index Validation

1. **Schema Compatibility**:
   - Index schema must include `chunk_ordinal` field
   - Schema mismatch triggers index recreation
   - All chunks must have ordinal value

2. **Data Integrity**:
   - All chunks for a file must have sequential ordinals
   - No duplicate ordinals within a file
   - Ordinal count must match chunk count

3. **Retrieval Consistency**:
   - Retrieval by ordinal must return correct chunk
   - Out-of-bounds requests must return error
   - Ordinal must match requested ordinal

## Error Conditions

### Chunk Creation Errors

| Error | Condition | Handling |
|-------|-----------|----------|
| Invalid UTF-8 | Content contains invalid UTF-8 sequences | Panic (should not happen with valid markdown) |
| Empty Content | Content is empty or whitespace only | Return empty Vec<Chunk> |
| No Headings | File has no heading structure | Create chunks without heading context |

### Index Operation Errors

| Error | Condition | Handling |
|-------|-----------|----------|
| Schema Mismatch | Index schema doesn't include chunk_ordinal | Recreate index, re-index all files |
| Out of Bounds | Requested ordinal > chunk count | Return error or empty result |
| File Not Found | File path doesn't exist | Return error |
| Lock Failure | Cannot acquire index writer lock | Retry once, then panic |

### Re-indexing Errors

| Error | Condition | Handling |
|-------|-----------|----------|
| Concurrent Edit | Another edit in progress | Wait for writer lock |
| Index Corruption | Index data inconsistent | Recreate index, re-index file |
| Partial Failure | Some chunks fail to index | Log error, continue with successful chunks |

## Performance Considerations

### Memory Usage

- **Chunk struct**: ~2000 bytes (content) + ~100 bytes (metadata) = ~2.1KB per chunk
- **ChunkDoc struct**: Same as Chunk (stored in index)
- **Ordinal overhead**: 8 bytes per chunk (u64)
- **Total overhead**: <1% of index size (as per PERF-004)

### Index Size

- **Base index**: ~50MB for 10k documents
- **Ordinal field**: 50k chunks × 8 bytes = 400KB
- **Total**: ~50.4MB (0.8% increase)

### Query Performance

- **Retrieval by ordinal**: <50ms (PERF-003)
- **Re-indexing typical file**: <100ms (PERF-005)
- **Chunk truncation**: <10ms per chunk (PERF-001)

## Migration Strategy

### Schema Migration

**Scenario**: Existing index without `chunk_ordinal` field

**Approach**: Manual index rebuild

**Steps**:
1. Delete existing index directory
2. Re-scan vault
3. Re-index all files with new schema
4. Verify ordinal metadata present

**Performance**: <2 seconds for typical vaults (10k documents)

**Rollback**: Not applicable (rebuild is destructive)

### Data Migration

**Scenario**: Existing chunks without ordinal metadata

**Approach**: Re-parse all files during rebuild

**Steps**:
1. Read file content
2. Parse chunks with ordinal assignment
3. Index chunks with ordinal metadata
4. Verify ordinal consistency

**Data Loss**: None (rebuild from source files)

## Testing Data Model

### Unit Tests

- **Chunk creation**: Test ordinal assignment, heading extraction, line numbers
- **Chunk validation**: Test character boundary safety, content length
- **Schema compatibility**: Test field addition, schema mismatch handling

### Integration Tests

- **End-to-end flow**: File → Chunks → Index → Retrieval
- **Re-indexing**: Edit → Re-index → Verify ordinals updated
- **Cross-module**: Verify ordinal handling in all consuming modules

### Corpus Tests

- **Multi-byte content**: Test with emojis, CJK, combining diacritics
- **Boundary cases**: Test chunks ending on multi-byte boundaries
- **Large files**: Test files with 100+ chunks

## References

- Rust std::str: https://doc.rust-lang.org/std/primitive.str.html
- Tantivy schema: https://docs.rs/tantivy/latest/tantivy/schema/
- UTF-8 encoding: https://en.wikipedia.org/wiki/UTF-8
