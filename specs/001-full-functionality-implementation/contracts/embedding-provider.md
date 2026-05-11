# Embedding Provider Interface Contract

**Version**: 1.0.0
**Date**: 2025-05-09
**Purpose**: Define the public interface for embedding providers

## Overview

This contract defines the interface that all embedding providers must implement. It ensures consistency across different embedding backends (local, Ollama, OpenRouter) and enables seamless switching between providers.

## Trait Definition

```rust
#[async_trait]
pub trait EmbedProvider: Send + Sync {
    /// Generate an embedding vector for the given text.
    ///
    /// # Arguments
    /// * `text` - The text to embed
    ///
    /// # Returns
    /// * `Ok(Vec<f32>)` - The embedding vector
    /// * `Err(EmbedError)` - Error if embedding generation fails
    ///
    /// # Errors
    /// * `EmbedError::NetworkTimeout` - Request exceeded timeout
    /// * `EmbedError::HttpError` - HTTP error response
    /// * `EmbedError::InvalidResponse` - Malformed response
    /// * `EmbedError::DimensionMismatch` - Embedding dimension mismatch
    /// * `EmbedError::ModelDownloadFailed` - Model download failed
    /// * `EmbedError::ModelCorrupted` - Model file corrupted
    /// * `EmbedError::IoError` - I/O error
    ///
    /// # Performance
    /// * Local provider: <100ms per document
    /// * Ollama provider: <500ms per document
    /// * OpenRouter provider: <1s per document
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbedError>;

    /// Return the dimension of the embedding vectors.
    ///
    /// # Returns
    /// * `usize` - The embedding dimension
    ///
    /// # Notes
    /// * Local provider: 384 (all-MiniLM-L6-v2)
    /// * Ollama provider: Model-dependent
    /// * OpenRouter provider: Model-dependent
    fn dimension(&self) -> usize;
}
```

## Implementation Requirements

### LocalEmbedProvider

**Initialization**:
```rust
impl LocalEmbedProvider {
    /// Create a new local embedding provider.
    ///
    /// # Arguments
    /// * `models_dir` - Directory for cached models
    ///
    /// # Behavior
    /// * Download model on first use if not present
    /// * Validate model integrity via SHA256 hash
    /// * Cache model in memory for subsequent calls
    ///
    /// # Errors
    /// * Returns error if model download fails
    /// * Returns error if model is corrupted
    pub async fn new(models_dir: &Path) -> Self;
}
```

**Configuration**:
- Model: all-MiniLM-L6-v2
- Dimension: 384
- Cache location: `.knowledge-loom-index/models/`
- Download URL: Hugging Face model hub

### OllamaEmbedProvider

**Initialization**:
```rust
impl OllamaEmbedProvider {
    /// Create a new Ollama embedding provider.
    ///
    /// # Arguments
    /// * `ollama_url` - Ollama instance URL
    ///
    /// # Behavior
    /// * Use default model if not specified
    /// * Configure HTTP client with timeout
    /// * Validate URL format
    ///
    /// # Errors
    /// * Returns error if URL is invalid
    pub async fn new(ollama_url: String) -> Self;
}
```

**Configuration**:
- Environment variable: `OLLAMA_URL`
- Default model: "nomic-embed-text"
- Timeout: 5 seconds
- API endpoint: `/api/embeddings`

### OpenRouterEmbedProvider

**Initialization**:
```rust
impl OpenRouterEmbedProvider {
    /// Create a new OpenRouter embedding provider.
    ///
    /// # Arguments
    /// * `api_key` - OpenRouter API key
    ///
    /// # Behavior
    /// * Use default model if not specified
    /// * Configure HTTP client with timeout
    /// * Set Bearer token authentication
    ///
    /// # Errors
    /// * Returns error if API key is invalid
    pub async fn new(api_key: String) -> Self;
}
```

**Configuration**:
- Environment variable: `OPENROUTER_API_KEY`
- Environment variable: `OPENROUTER_MODEL` (optional)
- Default model: "openai/text-embedding-3-small"
- Timeout: 5 seconds
- API endpoint: `https://openrouter.ai/api/v1/embeddings`

## Error Contract

### EmbedError Enum

```rust
pub enum EmbedError {
    /// Request exceeded timeout
    NetworkTimeout(String),

    /// HTTP error response
    HttpError(u16, String),

    /// Malformed response
    InvalidResponse(String),

    /// Embedding dimension mismatch
    DimensionMismatch { expected: usize, actual: usize },

    /// Model download failed
    ModelDownloadFailed(String),

    /// Model file corrupted
    ModelCorrupted(String),

    /// I/O error
    IoError(std::io::Error),
}
```

### Error Handling Requirements

1. **Never panic** in production code
2. **Use Result<T, E>** for all fallible operations
3. **Provide context** in error messages
4. **Log warnings** with context on fallback
5. **Validate inputs** before processing

## Performance Contract

### Latency Targets

| Provider | Target | Notes |
|----------|--------|-------|
| Local | <100ms | CPU-bound, cached model |
| Ollama | <500ms | Network-bound, includes HTTP overhead |
| OpenRouter | <1s | Network-bound, includes HTTP overhead |

### Memory Usage

| Component | Target | Notes |
|-----------|--------|-------|
| Local model | <100MB | all-MiniLM-L6-v2 |
| Embedding vector | ~1.5KB | 384 dimensions × 4 bytes |
| HTTP client | <5MB | Connection pool |

### Concurrency

- All providers must be thread-safe (`Send + Sync`)
- Support concurrent embedding requests
- Use async/await for non-blocking operations

## Fallback Contract

### Provider Priority Chain

1. **Local provider** (always available)
2. **Ollama provider** (if configured)
3. **OpenRouter provider** (if configured)

### Fallback Behavior

```rust
impl EmbedProviderEnum {
    /// Generate embedding with automatic fallback.
    ///
    /// # Behavior
    /// 1. Try providers in priority order
    /// 2. On failure, try next provider
    /// 3. Log warning with context
    /// 4. Return error if all providers fail
    ///
    /// # Fallback Conditions
    /// * Network timeout (>5s)
    /// * HTTP error (4xx/5xx)
    /// * Invalid response format
    /// * Dimension mismatch
    /// * Model download failure
    /// * Model corruption
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbedError>;
}
```

### Fallback Logging

- Log warning when provider fails
- Include error context in log message
- Log which provider is being tried next
- Log final error if all providers fail

## Testing Contract

### Unit Tests

- Test all error paths
- Test dimension validation
- Test model loading
- Test HTTP client configuration

### Integration Tests

- Test provider switching
- Test fallback behavior
- Test concurrent operations
- Test error recovery

### Performance Tests

- Benchmark all providers
- Measure latency and throughput
- Validate memory usage
- Test with realistic workloads

## Versioning

### Semantic Versioning

- Major version: Breaking changes to trait
- Minor version: New functionality
- Patch version: Bug fixes

### Backward Compatibility

- Maintain trait signature compatibility
- Add new methods with default implementations
- Deprecate old methods before removal
- Document breaking changes in CHANGELOG.md

## Security Contract

### API Key Handling

- Never log API keys
- Validate API key format
- Store in environment variables only
- Use HTTPS for all API calls

### Model Integrity

- Validate SHA256 hash of downloaded models
- Detect corrupted model files
- Re-download corrupted models automatically
- Log warnings for integrity issues

### Network Security

- Use HTTPS for OpenRouter API
- Validate SSL certificates
- Set reasonable timeouts
- Implement retry logic with backoff
