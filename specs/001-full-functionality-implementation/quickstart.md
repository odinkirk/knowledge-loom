# Quickstart: Full Functionality Implementation

**Feature**: Full Functionality Implementation
**Date**: 2025-05-09
**Purpose**: Quick reference for implementing embedding providers

## Overview

This quickstart guide provides step-by-step instructions for implementing the three embedding providers (local, Ollama, OpenRouter) and integrating them into the Knowledge Loom search engine.

## Prerequisites

- Rust 1.75+ (Async Trait support required)
- Existing Knowledge Loom codebase
- Understanding of async/await patterns in Rust
- Familiarity with HTTP client usage (reqwest)

## Step 1: Implement Local Embedding Provider

### 1.1 Add Dependencies

Update `Cargo.toml`:

```toml
[dependencies]
fastembed = "3.0"
```

### 1.2 Create `src/embed/local.rs`

```rust
use std::path::Path;
use std::sync::Arc;

pub struct LocalEmbedProvider {
    models_dir: Arc<Path>,
    // Add model field when fastembed integration is complete
}

impl LocalEmbedProvider {
    pub async fn new(models_dir: &Path) -> Self {
        Self {
            models_dir: Arc::from(models_dir),
        }
    }

    pub async fn embed(&self, text: &str) -> Vec<f32> {
        // TODO: Implement fastembed integration
        // 1. Load model if not already loaded
        // 2. Generate embedding for text
        // 3. Return embedding vector
        vec![0.0_f32; 384] // Placeholder
    }

    pub fn dimension(&self) -> usize {
        384
    }
}
```

### 1.3 Implement Model Download

```rust
impl LocalEmbedProvider {
    async fn download_model(&self) -> Result<(), EmbedError> {
        // TODO: Implement model download
        // 1. Check if model exists in cache
        // 2. Download from Hugging Face if not present
        // 3. Validate SHA256 hash
        // 4. Cache model locally
        Ok(())
    }
}
```

## Step 2: Implement Ollama Embedding Provider

### 2.1 Create `src/embed/ollama.rs`

```rust
use std::sync::Arc;

pub struct OllamaEmbedProvider {
    ollama_url: Arc<String>,
    model: String,
    client: reqwest::Client,
    timeout: Duration,
}

impl OllamaEmbedProvider {
    pub async fn new(ollama_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            ollama_url: Arc::new(ollama_url),
            model: "nomic-embed-text".to_string(),
            client,
            timeout: Duration::from_secs(5),
        }
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbedError> {
        // TODO: Implement Ollama API integration
        // 1. Build request payload
        // 2. Send POST to /api/embeddings
        // 3. Parse response
        // 4. Validate embedding dimension
        // 5. Return embedding vector
        Ok(vec![0.0_f32; 384]) // Placeholder
    }

    pub fn dimension(&self) -> usize {
        // TODO: Return actual dimension from Ollama model
        384
    }
}
```

### 2.2 Implement HTTP Request

```rust
impl OllamaEmbedProvider {
    async fn send_request(&self, text: &str) -> Result<Vec<f32>, EmbedError> {
        let url = format!("{}/api/embeddings", self.ollama_url);
        let payload = serde_json::json!({
            "model": self.model,
            "prompt": text
        });

        let response = self.client
            .post(&url)
            .json(&payload)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| EmbedError::NetworkTimeout(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(EmbedError::HttpError(status, body));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| EmbedError::InvalidResponse(e.to_string()))?;

        // TODO: Parse embedding from result
        Ok(vec![0.0_f32; 384])
    }
}
```

## Step 3: Implement OpenRouter Embedding Provider

### 3.1 Create `src/embed/openrouter.rs`

```rust
use std::sync::Arc;

pub struct OpenRouterEmbedProvider {
    api_key: Arc<String>,
    model: String,
    client: reqwest::Client,
    timeout: Duration,
}

impl OpenRouterEmbedProvider {
    pub async fn new(api_key: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            api_key: Arc::new(api_key),
            model: std::env::var("OPENROUTER_MODEL")
                .unwrap_or_else(|_| "openai/text-embedding-3-small".to_string()),
            client,
            timeout: Duration::from_secs(5),
        }
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbedError> {
        // TODO: Implement OpenRouter API integration
        // 1. Build request payload
        // 2. Send POST to /api/v1/embeddings
        // 3. Parse response
        // 4. Validate embedding dimension
        // 5. Return embedding vector
        Ok(vec![0.0_f32; 384]) // Placeholder
    }

    pub fn dimension(&self) -> usize {
        // TODO: Return actual dimension from OpenRouter model
        384
    }
}
```

### 3.2 Implement HTTP Request

```rust
impl OpenRouterEmbedProvider {
    async fn send_request(&self, text: &str) -> Result<Vec<f32>, EmbedError> {
        let url = "https://openrouter.ai/api/v1/embeddings";
        let payload = serde_json::json!({
            "model": self.model,
            "input": text
        });

        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&payload)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| EmbedError::NetworkTimeout(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(EmbedError::HttpError(status, body));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| EmbedError::InvalidResponse(e.to_string()))?;

        // TODO: Parse embedding from result
        Ok(vec![0.0_f32; 384])
    }
}
```

## Step 4: Update Embed Provider Enum

### 4.1 Modify `src/embed/mod.rs`

```rust
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod local;
pub mod ollama;
pub mod openrouter;

use local::LocalEmbedProvider;
use ollama::OllamaEmbedProvider;
use openrouter::OpenRouterEmbedProvider;

#[async_trait]
pub trait EmbedProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbedError>;
    fn dimension(&self) -> usize;
}

pub struct EmbedProviderEnum {
    pub local: Arc<Mutex<LocalEmbedProvider>>,
    pub ollama: Option<Arc<Mutex<OllamaEmbedProvider>>>,
    pub openrouter: Option<Arc<Mutex<OpenRouterEmbedProvider>>>,
    pub use_ollama: bool,
    pub use_openrouter: bool,
}

impl EmbedProviderEnum {
    pub async fn new(kb_root: &str) -> Self {
        let kb_root_path = PathBuf::from(kb_root);
        let models_dir = kb_root_path.join(".knowledge-loom-index/models");

        let local_provider = LocalEmbedProvider::new(&models_dir).await;
        let ollama_url = std::env::var("OLLAMA_URL").ok();
        let openrouter_api_key = std::env::var("OPENROUTER_API_KEY").ok();

        let use_ollama = ollama_url.is_some();
        let use_openrouter = openrouter_api_key.is_some();

        let ollama_provider = if use_ollama {
            Some(Arc::new(Mutex::new(
                OllamaEmbedProvider::new(ollama_url.unwrap()).await,
            )))
        } else {
            None
        };

        let openrouter_provider = if use_openrouter {
            Some(Arc::new(Mutex::new(
                OpenRouterEmbedProvider::new(openrouter_api_key.unwrap()).await,
            )))
        } else {
            None
        };

        Self {
            local: Arc::new(Mutex::new(local_provider)),
            ollama: ollama_provider,
            openrouter: openrouter_provider,
            use_ollama,
            use_openrouter,
        }
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbedError> {
        // Try local first
        {
            let provider = self.local.lock().await;
            match provider.embed(text).await {
                Ok(embedding) => return Ok(embedding),
                Err(e) => {
                    eprintln!("Local provider failed: {}", e);
                }
            }
        }

        // Try Ollama if configured
        if let Some(ref provider) = self.ollama {
            let provider = provider.lock().await;
            match provider.embed(text).await {
                Ok(embedding) => return Ok(embedding),
                Err(e) => {
                    eprintln!("Ollama provider failed: {}", e);
                }
            }
        }

        // Try OpenRouter if configured
        if let Some(ref provider) = self.openrouter {
            let provider = provider.lock().await;
            match provider.embed(text).await {
                Ok(embedding) => return Ok(embedding),
                Err(e) => {
                    eprintln!("OpenRouter provider failed: {}", e);
                }
            }
        }

        Err(EmbedError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            "All embedding providers failed",
        )))
    }

    pub fn dimension(&self) -> usize {
        // Assuming all providers have same dimension for simplicity
        384
    }
}
```

## Step 5: Add Error Types

### 5.1 Create `src/embed/error.rs`

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EmbedError {
    #[error("Network timeout: {0}")]
    NetworkTimeout(String),

    #[error("HTTP error {0}: {1}")]
    HttpError(u16, String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("Model download failed: {0}")]
    ModelDownloadFailed(String),

    #[error("Model corrupted: {0}")]
    ModelCorrupted(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}
```

### 5.2 Update `src/embed/mod.rs`

```rust
mod error;

pub use error::EmbedError;
```

## Step 6: Write Tests

### 6.1 Create `tests/embed_tests.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_provider_dimension() {
        let provider = LocalEmbedProvider::new(Path::new("/tmp/models")).await;
        assert_eq!(provider.dimension(), 384);
    }

    #[tokio::test]
    async fn test_ollama_provider_dimension() {
        let provider = OllamaEmbedProvider::new("http://localhost:11434".to_string()).await;
        assert_eq!(provider.dimension(), 384);
    }

    #[tokio::test]
    async fn test_openrouter_provider_dimension() {
        let provider = OpenRouterEmbedProvider::new("test-key".to_string()).await;
        assert_eq!(provider.dimension(), 384);
    }

    #[tokio::test]
    async fn test_provider_enum_fallback() {
        // TODO: Test fallback behavior
    }
}
```

## Step 7: Update Documentation

### 7.1 Update `README.md`

Add section on embedding providers:

```markdown
## Embedding Providers

Knowledge Loom supports multiple embedding providers:

### Local Provider (Default)
- Uses fastembed with all-MiniLM-L6-v2
- 384 dimensions
- Offline-capable
- <100ms per document

### Ollama Provider
- Set `OLLAMA_URL` environment variable
- Uses Ollama HTTP API
- Configurable model
- <500ms per document

### OpenRouter Provider
- Set `OPENROUTER_API_KEY` environment variable
- Set `OPENROUTER_MODEL` for model selection
- Uses OpenRouter HTTP API
- <1s per document

### Fallback Behavior
The system automatically falls back to the next available provider if one fails:
1. Local (always available)
2. Ollama (if configured)
3. OpenRouter (if configured)
```

### 7.2 Update `ARCHITECTURE.md`

Add section on embedding architecture:

```markdown
## Embedding Architecture

### Provider Interface
All embedding providers implement the `EmbedProvider` trait:
- `embed(&self, text: &str) -> Result<Vec<f32>, EmbedError>`
- `dimension(&self) -> usize`

### Provider Implementations
- `LocalEmbedProvider`: fastembed integration
- `OllamaEmbedProvider`: HTTP API integration
- `OpenRouterEmbedProvider`: HTTP API integration

### Fallback Strategy
Provider priority chain with automatic fallback on failure.
```

## Step 8: Verify Implementation

### 8.1 Run Tests

```bash
cargo test --lib embed
```

### 8.2 Check Code Coverage

```bash
cargo tarpaulin --out Html --output-dir ./tarpaulin-report
```

### 8.3 Run Linter

```bash
cargo clippy -- -D warnings
```

### 8.4 Format Code

```bash
cargo fmt --all -- --check
```

## Common Issues

### Issue: Model download fails

**Solution**: Check internet connection and disk space. Verify Hugging Face is accessible.

### Issue: Ollama connection timeout

**Solution**: Verify OLLAMA_URL is correct and Ollama is running. Check firewall settings.

### Issue: OpenRouter API key invalid

**Solution**: Verify OPENROUTER_API_KEY is set correctly. Check API key permissions.

### Issue: Dimension mismatch

**Solution**: Ensure all providers use models with compatible dimensions. Check model configuration.

## Next Steps

1. Complete TODO items in code
2. Add comprehensive error handling
3. Implement model download and caching
4. Add performance benchmarks
5. Update documentation
6. Verify 80% code coverage
7. Run integration tests
8. Test with real data
