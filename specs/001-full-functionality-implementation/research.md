# Research: Full Functionality Implementation

**Feature**: Full Functionality Implementation
**Date**: 2025-05-09
**Purpose**: Resolve technical unknowns and establish best practices for embedding provider implementation

## Research Tasks

### 1. Local Embedding Provider (fastembed)

**Decision**: Use fastembed crate with all-MiniLM-L6-v2 model

**Rationale**:
- fastembed is a Rust-native embedding library with minimal dependencies
- all-MiniLM-L6-v2 is well-suited for semantic search (384 dimensions, ~80MB model)
- Provides CPU-only inference, suitable for offline use
- Active maintenance and good performance characteristics

**Alternatives Considered**:
- candle-ml: More flexible but larger dependency footprint
- ort (ONNX Runtime): Requires external runtime installation
- Custom ONNX integration: Too complex for this use case

**Implementation Notes**:
- Model will be cached in `.knowledge-loom-index/models/`
- Download on first use with progress indication
- Validate model integrity on load
- Support model version pinning for reproducibility

### 2. Ollama API Integration

**Decision**: Use reqwest HTTP client with timeout configuration

**Rationale**:
- Ollama provides REST API for embeddings at `/api/embeddings`
- Standard HTTP POST with JSON request/response
- reqwest is already a project dependency
- Async HTTP client fits well with tokio runtime

**API Details**:
- Endpoint: `http://<OLLAMA_URL>/api/embeddings`
- Request: `{"model": "<model>", "prompt": "<text>"}`
- Response: `{"embedding": [float_array]}`
- Timeout: 5 seconds (configurable)

**Alternatives Considered**:
- ollama-rs: Less mature, limited feature set
- Custom HTTP implementation: Re-inventing the wheel

### 3. OpenRouter API Integration

**Decision**: Use reqwest HTTP client with API key authentication

**Rationale**:
- OpenRouter provides OpenAI-compatible API for embeddings
- Standard HTTP POST with Bearer token authentication
- Consistent with Ollama integration pattern
- reqwest already available in project

**API Details**:
- Endpoint: `https://openrouter.ai/api/v1/embeddings`
- Headers: `Authorization: Bearer <OPENROUTER_API_KEY>`
- Request: `{"model": "<OPENROUTER_MODEL>", "input": "<text>"}`
- Response: `{"data": [{"embedding": [float_array]}]}`
- Timeout: 5 seconds (configurable)
- Default model: `openai/text-embedding-3-small` if OPENROUTER_MODEL not set

**Alternatives Considered**:
- openai-rs: OpenRouter-specific but less flexible
- Custom HTTP implementation: Unnecessary complexity

### 4. HTTP Client Best Practices

**Decision**: Use reqwest with async/await, proper timeout handling, and connection pooling

**Rationale**:
- reqwest provides async HTTP client with tokio integration
- Built-in connection pooling and keep-alive
- Configurable timeouts and retry logic
- Type-safe JSON serialization/deserialization

**Best Practices**:
- Set connection timeout: 5 seconds
- Set read timeout: 10 seconds
- Use connection pooling for multiple requests
- Implement exponential backoff for retries
- Log HTTP errors with context

**Alternatives Considered**:
- surf: Less mature ecosystem
- hyper: Too low-level for this use case

### 5. Error Handling and Fallback Strategies

**Decision**: Use Result<T, E> with thiserror for custom error types, implement provider priority chain

**Rationale**:
- Rust's Result type provides explicit error handling
- thiserror allows for custom error types with context
- Provider priority chain enables graceful degradation
- Logging provides visibility into fallback behavior

**Error Types**:
- `EmbedError::NetworkTimeout`: Request exceeded timeout
- `EmbedError::HttpError(status)`: HTTP error response
- `EmbedError::InvalidResponse`: Malformed response
- `EmbedError::DimensionMismatch`: Embedding dimension mismatch
- `EmbedError::ModelDownloadFailed`: Model download failed
- `EmbedError::ModelCorrupted`: Model file corrupted

**Fallback Strategy**:
1. Try configured provider (local by default)
2. On failure, try next provider in priority chain
3. Log warning with context
4. Return error if all providers fail

**Alternatives Considered**:
- Panic on error: Not idiomatic Rust
- Silent fallback: No visibility into issues

### 6. Dimension Validation

**Decision**: Validate embedding dimensions at provider initialization and on each embedding

**Rationale**:
- Early detection of dimension mismatches
- Prevents runtime errors in vector operations
- Clear error messages for debugging
- Consistent with type safety principles

**Validation Approach**:
- Check dimension on provider initialization
- Validate each embedding result
- Log warning on mismatch
- Reject mismatched embeddings
- Fallback to next provider

**Expected Dimensions**:
- Local (all-MiniLM-L6-v2): 384
- Ollama: Model-dependent (validate at runtime)
- OpenRouter: Model-dependent (validate at runtime)

**Alternatives Considered**:
- Truncate/pad embeddings: Loses information
- Allow mismatched dimensions: Runtime errors

### 7. Model Download and Caching

**Decision**: Download model on first use, cache locally, validate integrity

**Rationale**:
- Avoids bundling large model files in binary
- Enables offline use after initial download
- Reduces startup time for subsequent runs
- Allows model updates without binary rebuild

**Implementation Details**:
- Download URL: Hugging Face model hub
- Cache location: `.knowledge-loom-index/models/`
- File naming: `all-MiniLM-L6-v2.onnx`
- Integrity check: SHA256 hash validation
- Progress indication: Log download progress
- Retry logic: 3 attempts with exponential backoff

**Alternatives Considered**:
- Bundle model in binary: Increases binary size significantly
- Download on every startup: Unnecessary overhead

### 8. Performance Optimization

**Decision**: Use async/await, batch embeddings where possible, cache results

**Rationale**:
- Async operations prevent blocking
- Batching reduces HTTP overhead
- Caching avoids redundant computation
- Meets performance targets (<100ms local, <500ms Ollama, <1s OpenRouter)

**Optimization Techniques**:
- Use tokio for async operations
- Implement embedding cache (LRU eviction)
- Batch multiple embeddings in single request (if supported)
- Pre-allocate vectors for known dimensions
- Use efficient serialization (bincode for cache)

**Performance Targets**:
- Local embedding: <100ms per document
- Ollama embedding: <500ms per document
- OpenRouter embedding: <1s per document
- Model download: <5 minutes on typical connection

**Alternatives Considered**:
- Synchronous operations: Blocks event loop
- No caching: Redundant computation

## Consolidated Findings

### Technology Stack

| Component | Technology | Rationale |
|------------|-----------|-----------|
| Local Embeddings | fastembed + all-MiniLM-L6-v2 | Rust-native, CPU-only, 384 dimensions |
| HTTP Client | reqwest | Async, tokio integration, connection pooling |
| Error Handling | thiserror | Custom error types with context |
| Serialization | serde + bincode | Type-safe, efficient for caching |
| Async Runtime | tokio | Project standard, async/await support |

### Integration Patterns

**Provider Trait**:
```rust
#[async_trait]
pub trait EmbedProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbedError>;
    fn dimension(&self) -> usize;
}
```

**Provider Priority Chain**:
1. Local (default)
2. Ollama (if configured)
3. OpenRouter (if configured)
4. Error if all fail

**Error Handling**:
- Use Result<T, E> for fallible operations
- Log warnings with context
- Implement graceful fallback
- Never panic in production code

### Performance Considerations

- **Local Provider**: CPU-bound, ~100ms per document
- **Ollama Provider**: Network-bound, ~500ms per document
- **OpenRouter Provider**: Network-bound, ~1s per document
- **Caching**: Reduces redundant computation
- **Async Operations**: Prevents blocking

### Testing Strategy

- **Unit Tests**: Provider implementations, error handling
- **Integration Tests**: Provider switching, fallback behavior
- **Performance Tests**: Benchmark all providers
- **Mock Tests**: HTTP responses, model download
- **Corpus Tests**: test-vault/ for semantic validation

## Open Questions Resolved

1. ✅ How to integrate fastembed for local embeddings?
2. ✅ What is the Ollama API integration pattern?
3. ✅ What is the OpenRouter API integration pattern?
4. ✅ What are HTTP client best practices?
5. ✅ How to handle errors and implement fallback?
6. ✅ How to validate embedding dimensions?
7. ✅ How to download and cache models?
8. ✅ How to optimize embedding performance?

## Next Steps

1. Implement local embedding provider with fastembed
2. Implement Ollama provider with HTTP client
3. Implement OpenRouter provider with HTTP client
4. Add provider priority chain and fallback logic
5. Implement dimension validation
6. Add comprehensive error handling
7. Write tests for all providers
8. Add performance benchmarks
9. Update documentation
10. Verify 80% code coverage
