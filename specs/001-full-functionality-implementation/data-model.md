# Data Model: Full Functionality Implementation

**Feature**: Full Functionality Implementation
**Date**: 2025-05-09
**Purpose**: Define data structures and relationships for embedding providers

## Core Entities

### EmbedProvider (Trait)

**Purpose**: Abstract interface for generating embeddings from text

**Methods**:
- `embed(&self, text: &str) -> Result<Vec<f32>, EmbedError>`: Generate embedding vector for text
- `dimension(&self) -> usize`: Return embedding dimension

**Implementations**:
- `LocalEmbedProvider`: Local model using fastembed
- `OllamaEmbedProvider`: Ollama HTTP API
- `OpenRouterEmbedProvider`: OpenRouter HTTP API

### LocalEmbedProvider

**Purpose**: Generate embeddings using local fastembed model

**Fields**:
- `models_dir: Arc<Path>`: Directory for cached models
- `model: Option<EmbeddingModel>`: Loaded model instance (lazy loaded)

**State Transitions**:
```
Uninitialized → Loading → Ready
                    ↓
                  Failed (fallback to external)
```

**Validation Rules**:
- Model file must exist and be valid ONNX format
- Model dimension must be 384 (all-MiniLM-L6-v2)
- Model integrity validated via SHA256 hash

### OllamaEmbedProvider

**Purpose**: Generate embeddings via Ollama HTTP API

**Fields**:
- `ollama_url: Arc<String>`: Ollama instance URL
- `model: String`: Model name (default: "nomic-embed-text")
- `client: reqwest::Client`: Async HTTP client
- `timeout: Duration`: Request timeout (default: 5s)

**State Transitions**:
```
Ready → Requesting → Success
         ↓
       Failed (fallback to local)
```

**Validation Rules**:
- URL must be valid HTTP/HTTPS endpoint
- Model name must be non-empty
- Response must contain valid embedding array
- Embedding dimension must match expected value

### OpenRouterEmbedProvider

**Purpose**: Generate embeddings via OpenRouter HTTP API

**Fields**:
- `api_key: Arc<String>`: OpenRouter API key
- `model: String`: Model name (default: "openai/text-embedding-3-small")
- `client: reqwest::Client`: Async HTTP client
- `timeout: Duration`: Request timeout (default: 5s)

**State Transitions**:
```
Ready → Requesting → Success
         ↓
       Failed (fallback to local)
```

**Validation Rules**:
- API key must be non-empty
- Model name must be non-empty
- Response must contain valid embedding array
- Embedding dimension must match expected value

### EmbedProviderEnum

**Purpose**: Enum wrapper for all embedding providers with priority chain

**Fields**:
- `local: Arc<Mutex<LocalEmbedProvider>>`: Local provider
- `ollama: Option<Arc<Mutex<OllamaEmbedProvider>>>`: Optional Ollama provider
- `openrouter: Option<Arc<Mutex<OpenRouterEmbedProvider>>>`: Optional OpenRouter provider
- `priority: Vec<ProviderType>`: Provider priority order

**ProviderType Enum**:
- `Local`: Local embeddings
- `Ollama`: Ollama API
- `OpenRouter`: OpenRouter API

**Priority Logic**:
1. Try providers in priority order
2. On failure, try next provider
3. Return error if all providers fail

### EmbedError

**Purpose**: Error types for embedding operations

**Variants**:
- `NetworkTimeout(String)`: Request exceeded timeout
- `HttpError(u16, String)`: HTTP error with status code
- `InvalidResponse(String)`: Malformed response
- `DimensionMismatch { expected: usize, actual: usize }`: Dimension mismatch
- `ModelDownloadFailed(String)`: Model download failed
- `ModelCorrupted(String)`: Model file corrupted
- `IoError(std::io::Error)`: I/O error

**Error Context**:
- All errors include descriptive messages
- Network errors include timeout duration
- HTTP errors include status code
- Dimension errors include expected and actual values

## Data Structures

### Embedding Vector

**Type**: `Vec<f32>`

**Purpose**: Numerical representation of text for semantic similarity

**Constraints**:
- Length must match provider dimension
- Values must be finite (no NaN or infinity)
- Normalized for cosine similarity (optional)

### Model Metadata

**Type**: Struct

**Purpose**: Information about cached models

**Fields**:
- `name: String`: Model name (e.g., "all-MiniLM-L6-v2")
- `version: String`: Model version
- `dimension: usize`: Embedding dimension
- `sha256: String`: File integrity hash
- `downloaded_at: chrono::DateTime<chrono::Utc>`: Download timestamp
- `size_bytes: u64`: File size

### Provider Configuration

**Type**: Struct

**Purpose**: Configuration for embedding providers

**Fields**:
- `local_enabled: bool`: Enable local provider
- `ollama_url: Option<String>`: Ollama instance URL
- `ollama_model: Option<String>`: Ollama model name
- `openrouter_api_key: Option<String>`: OpenRouter API key
- `openrouter_model: Option<String>`: OpenRouter model name
- `priority: Vec<ProviderType>`: Provider priority order
- `timeout_seconds: u64`: Request timeout in seconds

## Relationships

### Provider Hierarchy

```
EmbedProvider (trait)
├── LocalEmbedProvider
├── OllamaEmbedProvider
└── OpenRouterEmbedProvider

EmbedProviderEnum
├── LocalEmbedProvider (always present)
├── OllamaEmbedProvider (optional)
└── OpenRouterEmbedProvider (optional)
```

### Error Flow

```
EmbedProviderEnum::embed()
  ↓
Try provider 1 (e.g., Local)
  ↓ Success → Return embedding
  ↓ Failure
Try provider 2 (e.g., Ollama)
  ↓ Success → Return embedding
  ↓ Failure
Try provider 3 (e.g., OpenRouter)
  ↓ Success → Return embedding
  ↓ Failure
Return error
```

### Model Lifecycle

```
Model Download
  ↓
Model Cache (.knowledge-loom-index/models/)
  ↓
Model Load (LocalEmbedProvider)
  ↓
Embedding Generation
  ↓
Embedding Cache (optional)
```

## Validation Rules

### Input Validation

- Text must be non-empty string
- Text length must be reasonable (<10,000 characters)
- Provider configuration must be valid

### Output Validation

- Embedding vector must have correct dimension
- Embedding values must be finite
- Embedding must be normalized (if required)

### State Validation

- Provider must be initialized before use
- Model must be loaded before embedding
- HTTP client must be configured

## Performance Considerations

### Memory Usage

- Local model: ~80MB (all-MiniLM-L6-v2)
- Embedding vector: 384 * 4 bytes = ~1.5KB per document
- HTTP client: ~1MB connection pool

### Latency Targets

- Local embedding: <100ms per document
- Ollama embedding: <500ms per document
- OpenRouter embedding: <1s per document
- Fallback: <2s total (including retries)

### Caching Strategy

- Model cache: Persistent on disk
- Embedding cache: Optional in-memory LRU cache
- HTTP connections: Connection pooling

## Security Considerations

### API Keys

- OpenRouter API key stored in environment variable
- Never log API keys
- Validate API key format before use

### Model Integrity

- Validate SHA256 hash of downloaded models
- Detect corrupted model files
- Re-download corrupted models automatically

### Network Security

- Use HTTPS for OpenRouter API
- Validate SSL certificates
- Set reasonable timeouts to prevent hanging

## Testing Considerations

### Unit Tests

- Provider implementations
- Error handling
- Dimension validation
- Model loading

### Integration Tests

- Provider switching
- Fallback behavior
- HTTP client mocking
- Model download simulation

### Performance Tests

- Benchmark all providers
- Measure latency and throughput
- Validate memory usage
- Test concurrent operations

### Edge Cases

- Empty text input
- Very long text input
- Network timeouts
- HTTP errors
- Invalid responses
- Corrupted model files
- Missing model files
- Dimension mismatches
