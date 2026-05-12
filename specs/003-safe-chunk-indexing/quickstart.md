# Quickstart: Safe Chunk Indexing with Ordinal Metadata

**Feature**: Safe Chunk Indexing with Ordinal Metadata
**Date**: 2025-05-11
**Audience**: Developers implementing this feature

## Overview

This quickstart guide provides step-by-step instructions for implementing the safe chunk indexing feature with ordinal metadata. This feature fixes UTF-8 panics during chunk truncation, adds ordinal metadata to chunks, and extracts chunking logic into a dedicated module.

## Prerequisites

### Required Knowledge

- Rust programming language (async/await, error handling)
- UTF-8 encoding and character boundaries
- Tantivy index schema and operations
- Knowledge Loom architecture (modules, data flow)

### Required Tools

- Rust 1.75+ (with cargo)
- Git (for version control)
- Text editor or IDE with Rust support

### Required Access

- Read access to Knowledge Loom codebase
- Write access to create feature branch
- Access to test-vault/ for corpus testing

## Implementation Steps

### Step 1: Create chunks.rs Module

**File**: `src/chunks.rs`

**Purpose**: Extract chunking logic from BM25 module

**Actions**:

1. Create new file `src/chunks.rs`
2. Implement `truncate_at_whitespace()` function with character boundary safety
3. Implement `parse_chunks()` function with ordinal assignment
4. Define `Chunk` struct with ordinal, heading, content, line_start, line_end
5. Add module declaration in `src/lib.rs`

**Code Template**:

```rust
// src/chunks.rs

use std::path::Path;

pub const MAX_CHUNK_CHARS: usize = 2000;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub ordinal: u64,
    pub heading: Option<String>,
    pub content: String,
    pub line_start: usize,
    pub line_end: usize,
}

pub fn truncate_at_whitespace(content: &str, max: usize) -> &str {
    if content.len() <= max {
        return content;
    }
    
    // Find safe character boundary
    let safe_max = content.char_indices()
        .map(|(i, _)| i)
        .take_while(|&i| i <= max)
        .last()
        .unwrap_or(content.len());
    
    let slice = &content[..safe_max];
    match slice.rfind(|c: char| c.is_whitespace()) {
        Some(pos) if pos > 0 => content[..pos].trim_end(),
        _ => slice,
    }
}

pub fn parse_chunks(content: &str) -> Vec<Chunk> {
    // Implementation from spec with ordinal assignment
    // See research.md for details
    todo!()
}
```

**Testing**:

```rust
// tests/chunks_tests.rs

#[test]
fn test_truncate_at_whitespace_multi_byte() {
    let content = "Hello—World"; // em dash is 3 bytes
    let result = truncate_at_whitespace(content, 7);
    assert!(result.is_char_boundary(result.len()));
}

#[test]
fn test_parse_chunks_ordinals() {
    let content = "# Heading\n\nContent";
    let chunks = parse_chunks(content);
    assert_eq!(chunks[0].ordinal, 1);
}
```

### Step 2: Update BM25 Module

**File**: `src/bm25.rs`

**Purpose**: Add ordinal metadata to index schema and ChunkDoc

**Actions**:

1. Add `chunk_ordinal` field to Tantivy schema
2. Add `chunk_ordinal` field to `ChunkDoc` struct
3. Update `index_file()` to include ordinal in documents
4. Implement `get_chunk_by_ordinal()` method
5. Remove old chunking logic (now in chunks.rs)

**Schema Change**:

```rust
// src/bm25.rs

let chunk_ordinal = schema_builder.add_u64_field("chunk_ordinal", STORED);
```

**ChunkDoc Update**:

```rust
// src/bm25.rs

#[derive(Debug, Clone)]
pub struct ChunkDoc {
    pub path: String,
    pub heading: Option<String>,
    pub content: String,
    pub line_start: usize,
    pub line_end: usize,
    pub chunk_ordinal: u64,  // NEW FIELD
}
```

**New Method**:

```rust
// src/bm25.rs

impl BM25Index {
    pub async fn get_chunk_by_ordinal(
        &self,
        file_path: &str,
        chunk_ordinal: u64,
    ) -> Result<ChunkDoc, TantivyError> {
        // Implementation from contract
        // See contracts/chunk-retrieval.md for details
        todo!()
    }
}
```

**Testing**:

```rust
// tests/bm25_tests.rs

#[test]
fn test_get_chunk_by_ordinal() {
    let index = create_test_index();
    let chunk = index.get_chunk_by_ordinal("test.md", 1).await.unwrap();
    assert_eq!(chunk.chunk_ordinal, 1);
}
```

### Step 3: Update Edits Module

**File**: `src/edits.rs`

**Purpose**: Trigger re-indexing after edits

**Actions**:

1. Add re-indexing call after each edit operation
2. Handle re-indexing failures with corpus re-ingestion
3. Return "indexing: try again in 2 seconds" error during ingestion
4. Use existing `index_file()` method

**Re-indexing Integration**:

```rust
// src/edits.rs

use crate::bm25::BM25Index;

pub async fn edit_file(
    index: &BM25Index,
    path: &Path,
    edit: &Edit,
) -> Result<(), Error> {
    // Perform edit
    apply_edit(path, edit)?;
    
    // Re-index file to update ordinals
    let content = std::fs::read_to_string(path)?;
    match index.index_file(path, &content).await {
        Ok(_) => Ok(()),
        Err(e) => {
            // Log failure with sufficient detail
            eprintln!("Re-indexing failed for {}: {}", path.display(), e);
            
            // Trigger corpus re-ingestion
            index.rebuild_corpus().await?;
            
            Err(Error::ReindexingFailed("Corpus re-ingestion triggered".to_string()))
        }
    }
}
```

**Error Handling During Ingestion**:

```rust
// src/bm25.rs

impl BM25Index {
    pub async fn get_chunk_by_ordinal(
        &self,
        file_path: &str,
        chunk_ordinal: u64,
    ) -> Result<ChunkDoc, TantivyError> {
        // Check if corpus re-ingestion is in progress
        if self.is_ingesting() {
            return Err(TantivyError::SystemError(
                "indexing: try again in 2 seconds".to_string()
            ));
        }
        
        // Normal retrieval logic
        // ...
    }
}
```

**Testing**:

```rust
// tests/edits_tests.rs

#[test]
fn test_edit_triggers_reindex() {
    let index = create_test_index();
    let path = create_test_file();
    
    // Edit file
    edit_file(&index, &path, &edit).await.unwrap();
    
    // Verify ordinals updated
    let chunks = index.get_chunks_for_path(&path).await.unwrap();
    assert!(chunks.iter().all(|c| c.chunk_ordinal > 0));
}
```

### Step 4: Update Search Module

**File**: `src/search.rs`

**Purpose**: Include ordinal in search results

**Actions**:

1. Update result structs to include ordinal
2. Pass through ordinal from BM25 results
3. No functional changes (ordinal is additive)

**Result Update**:

```rust
// src/search.rs

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub score: f32,
    pub chunk: ChunkDoc,  // Already includes ordinal
}
```

**Testing**:

```rust
// tests/search_tests.rs

#[test]
fn test_search_includes_ordinal() {
    let results = search("query").await.unwrap();
    assert!(results.iter().all(|r| r.chunk.chunk_ordinal > 0));
}
```

### Step 5: Update Graph Module

**File**: `src/graph.rs`

**Purpose**: Include ordinal in node metadata

**Actions**:

1. Update node metadata to include ordinal
2. Pass through ordinal from chunk data
3. No functional changes (ordinal is additive)

**Node Update**:

```rust
// src/graph.rs

#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub chunk_ordinal: u64,  // NEW FIELD
    // ... other fields
}
```

**Testing**:

```rust
// tests/graph_tests.rs

#[test]
fn test_graph_includes_ordinal() {
    let graph = build_graph().await.unwrap();
    assert!(graph.nodes.iter().all(|n| n.chunk_ordinal > 0));
}
```

### Step 6: Update Vault Module

**File**: `src/vault.rs`

**Purpose**: Use chunks.rs for chunking

**Actions**:

1. Import from chunks.rs instead of BM25
2. Update chunking calls to use new module
3. No functional changes (same API)

**Import Update**:

```rust
// src/vault.rs

use crate::chunks::{parse_chunks, Chunk};
```

**Testing**:

```rust
// tests/vault_tests.rs

#[test]
fn test_vault_uses_chunks_module() {
    let chunks = parse_chunks(&content);
    assert!(chunks.iter().all(|c| c.ordinal > 0));
}
```

### Step 7: Update Server Module

**File**: `src/server.rs`

**Purpose**: Include ordinal in MCP tool responses

**Actions**:

1. Update tool response structs to include ordinal
2. Pass through ordinal from BM25 results
3. No functional changes (ordinal is additive)

**Response Update**:

```rust
// src/server.rs

#[derive(Debug, Clone)]
pub struct ChunkResponse {
    pub path: String,
    pub heading: Option<String>,
    pub content: String,
    pub line_start: usize,
    pub line_end: usize,
    pub chunk_ordinal: u64,  // NEW FIELD
}
```

**Testing**:

```rust
// tests/server_tests.rs

#[test]
fn test_mcp_tool_includes_ordinal() {
    let response = call_tool("get_chunk", params).await.unwrap();
    assert!(response.chunk_ordinal > 0);
}
```

## Testing Strategy

### Unit Tests

**Priority**: P1 (Critical)

**Coverage**: 80% minimum (constitution requirement)

**Test Files**:
- `tests/chunks_tests.rs` - Chunking logic tests
- `tests/bm25_tests.rs` - BM25 index tests
- `tests/edits_tests.rs` - Edit and re-index tests

**Test Cases**:
- Character boundary safety (multi-byte characters)
- Ordinal assignment and consistency
- Schema compatibility
- Error handling (out of bounds, file not found)

### Integration Tests

**Priority**: P2 (High)

**Test Files**:
- `tests/integration.rs` - Cross-module integration tests

**Test Cases**:
- End-to-end: Index → Retrieve → Edit → Re-index
- Cross-module: Ordinal handling in Search, Graph, Vault
- Concurrent: Multiple simultaneous operations

### Corpus Tests

**Priority**: P3 (Medium)

**Test Directory**: `test-vault/`

**Test Files**:
- Multi-byte content (emojis, CJK, combining diacritics)
- Boundary cases (chunks ending on multi-byte boundaries)
- Large files (100+ chunks)

## Performance Validation

### Benchmarks

**Tools**: `cargo bench` or custom timing

**Targets**:
- Chunk truncation: <10ms per chunk (PERF-001)
- Chunk retrieval: <50ms (PERF-003)
- File re-indexing: <100ms (PERF-005)

**Validation**:

```rust
#[bench]
fn bench_chunk_truncation(b: &mut Bencher) {
    let content = generate_test_content(2000);
    b.iter(|| {
        truncate_at_whitespace(content, 2000)
    });
}
```

### Memory Profiling

**Tools**: `valgrind`, `heaptrack`, or Rust's built-in profiling

**Target**: <1% memory overhead (PERF-004)

**Validation**:
- Measure index size before and after
- Calculate overhead percentage
- Verify under 1% threshold

## Common Issues

### Issue 1: Schema Mismatch

**Symptom**: Index recreation on every run

**Cause**: Old index without `chunk_ordinal` field

**Solution**:
1. Delete `.knowledge-loom-index/` directory
2. Re-run indexing
3. Verify schema includes `chunk_ordinal`

### Issue 2: UTF-8 Panic

**Symptom**: Thread panic during chunking

**Cause**: Byte slicing without character boundary check

**Solution**:
1. Use `char_indices()` for safe boundaries
2. Never slice by byte index alone
3. See `truncate_at_whitespace()` implementation

### Issue 3: Ordinal Inconsistency

**Symptom**: Ordinals not sequential or missing

**Cause**: Re-indexing not triggered after edits

**Solution**:
1. Verify Edits module calls `index_file()` after edits
2. Check for error handling in re-indexing
3. Verify ordinal assignment in `parse_chunks()`

### Issue 4: Performance Regression

**Symptom**: Operations slower than targets

**Cause**: Inefficient character boundary detection

**Solution**:
1. Profile with `cargo flamegraph`
2. Optimize `char_indices()` iteration
3. Cache results if applicable

### Issue 5: Indexing in Progress Error

**Symptom**: Requests fail with "indexing: try again in 2 seconds" error

**Cause**: Corpus re-ingestion underway after re-indexing failure

**Solution**:
1. Wait 2 seconds and retry request
2. Implement exponential backoff retry logic in client
3. Check logs for re-indexing failure details

## Verification Checklist

### Pre-Commit

- [ ] All unit tests pass (`cargo test`)
- [ ] All integration tests pass (`cargo test --test integration`)
- [ ] Code coverage >= 80% (`cargo tarpaulin`)
- [ ] Formatting passes (`cargo fmt --all -- --check`)
- [ ] Linting passes (`cargo clippy -- -D warnings`)
- [ ] Security passes (`cargo deny check`)

### Pre-Merge

- [ ] All quality gates pass
- [ ] Performance targets met
- [ ] Documentation updated (ARCHITECTURE.md, CHANGELOG.md)
- [ ] README updated with new capabilities
- [ ] Manual testing completed

### Post-Merge

- [ ] Index rebuild documented in README
- [ ] Migration guide updated (if breaking changes)
- [ ] CI/CD pipeline passes
- [ ] No regressions in existing functionality

## Next Steps

After completing this feature:

1. **Index Rebuild**: Users need to rebuild indexes to get ordinal metadata
2. **Documentation**: Update README with chunk retrieval examples
3. **Monitoring**: Track performance metrics in production
4. **Feedback**: Gather user feedback on new capabilities

## References

- **Spec**: [spec.md](./spec.md)
- **Plan**: [plan.md](./plan.md)
- **Research**: [research.md](./research.md)
- **Data Model**: [data-model.md](./data-model.md)
- **Contracts**: [contracts/](./contracts/)
- **Constitution**: [.specify/memory/constitution.md](../../.specify/memory/constitution.md)

## Support

For questions or issues:

1. Check this quickstart guide
2. Review research.md for technical decisions
3. Consult contracts for interface specifications
4. Refer to constitution for coding standards
