use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::error::{EmbedError, Result};

/// OpenRouter embedding provider configuration
#[derive(Clone)]
pub struct OpenRouterEmbedProvider {
    /// OpenRouter API key
    api_key: String,
    /// Model to use for embeddings
    model: String,
    /// HTTP client for API requests
    client: Client,
    #[allow(dead_code)]
    /// Timeout for API requests
    timeout: Duration,
}

/// OpenRouter API request structure
#[derive(Serialize)]
struct OpenRouterRequest {
    model: String,
    input: String,
}

/// OpenRouter API response structure
#[derive(Deserialize)]
struct OpenRouterResponse {
    data: Vec<OpenRouterEmbedding>,
}

/// OpenRouter embedding data structure
#[derive(Deserialize)]
struct OpenRouterEmbedding {
    embedding: Vec<f32>,
}

impl OpenRouterEmbedProvider {
    /// Create a new OpenRouter embedding provider
    ///
    /// # Arguments
    ///
    /// * `api_key` - OpenRouter API key
    /// * `model` - Model to use for embeddings (default: "openai/text-embedding-ada-002")
    ///
    /// # Returns
    ///
    /// A new OpenRouterEmbedProvider instance
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let provider = OpenRouterEmbedProvider::new(
    ///     "your-api-key",
    ///     "openai/text-embedding-ada-002"
    /// );
    /// ```
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        // Make timeout configurable via environment variable
        let timeout_secs = std::env::var("OPENROUTER_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10); // Default to 10 seconds
        let timeout = Duration::from_secs(timeout_secs);

        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");

        eprintln!(
            "OpenRouter embedding provider initialized successfully with {}s timeout",
            timeout_secs
        );

        Self {
            api_key: api_key.into(),
            model: model.into(),
            client,
            timeout,
        }
    }

    /// Generate an embedding for the given text
    ///
    /// # Arguments
    ///
    /// * `text` - The text to generate an embedding for
    ///
    /// # Returns
    ///
    /// A vector of f32 values representing the text embedding
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Network request fails
    /// - HTTP response indicates failure
    /// - Response format is invalid
    /// - Authentication fails
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let embedding = provider.embed("Hello, world!").await?;
    /// assert!(!embedding.is_empty());
    /// ```
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        eprintln!("OpenRouter embed called with: {}", text);

        // Make HTTP API call to OpenRouter
        let url = "https://openrouter.ai/api/v1/embeddings";
        let request = OpenRouterRequest {
            model: self.model.clone(),
            input: text.to_string(),
        };

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    EmbedError::Timeout {
                        timeout_secs: self.timeout.as_secs(),
                    }
                } else if e.is_connect() {
                    EmbedError::NetworkError(format!("Failed to connect to OpenRouter: {}", e))
                } else {
                    EmbedError::NetworkError(format!("Failed to send request to OpenRouter: {}", e))
                }
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = format!("OpenRouter API returned error: {}", response.status());

            // Check for authentication errors
            if status == 401 || status == 403 {
                return Err(EmbedError::AuthenticationError(message));
            }

            return Err(EmbedError::HttpError { status, message });
        }

        let openrouter_response: OpenRouterResponse = response.json().await.map_err(|e| {
            EmbedError::InvalidResponseFormat(format!("Failed to parse OpenRouter response: {}", e))
        })?;

        // Return the first (and only) embedding
        openrouter_response
            .data
            .into_iter()
            .next()
            .map(|e| e.embedding)
            .ok_or_else(|| {
                EmbedError::InvalidResponseFormat("No embedding data in response".to_string())
            })
    }

    /// Get the dimension of the embedding vectors
    ///
    /// # Returns
    ///
    /// The dimension (length) of the embedding vectors
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let dim = provider.dimension();
    /// assert_eq!(dim, 1536); // for openai/text-embedding-ada-002
    /// ```
    #[must_use]
    #[allow(dead_code)]
    pub fn dimension(&self) -> usize {
        1536 // OpenAI ada-002 dimension
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Check if OpenRouter API key is configured
    fn is_openrouter_configured() -> bool {
        std::env::var("OPENROUTER_API_KEY").is_ok()
    }

    #[test]
    fn test_openrouter_provider_creation() {
        let provider = OpenRouterEmbedProvider::new("test-key", "openai/text-embedding-ada-002");
        assert_eq!(provider.dimension(), 1536);
    }

    #[tokio::test]
    async fn test_openrouter_embedding() {
        if !is_openrouter_configured() {
            eprintln!("Skipping test: OPENROUTER_API_KEY not configured");
            return;
        }

        let api_key = std::env::var("OPENROUTER_API_KEY").unwrap();
        let provider = OpenRouterEmbedProvider::new(&api_key, "openai/text-embedding-ada-002");
        let embedding = provider.embed("test").await.unwrap();
        assert!(!embedding.is_empty(), "Embedding should not be empty");
        assert_eq!(embedding.len(), 1536);
    }

    #[tokio::test]
    async fn test_openrouter_embedding_consistency() {
        if !is_openrouter_configured() {
            eprintln!("Skipping test: OPENROUTER_API_KEY not configured");
            return;
        }

        let api_key = std::env::var("OPENROUTER_API_KEY").unwrap();
        let provider = OpenRouterEmbedProvider::new(&api_key, "openai/text-embedding-ada-002");
        let text = "consistency test";
        let embedding1 = provider.embed(text).await.unwrap();
        let embedding2 = provider.embed(text).await.unwrap();

        assert!(
            !embedding1.is_empty(),
            "First embedding should not be empty"
        );
        assert!(
            !embedding2.is_empty(),
            "Second embedding should not be empty"
        );
        assert_eq!(embedding1, embedding2, "Embeddings should be consistent");
    }

    #[tokio::test]
    async fn test_openrouter_embedding_different_inputs() {
        if !is_openrouter_configured() {
            eprintln!("Skipping test: OPENROUTER_API_KEY not configured");
            return;
        }

        let api_key = std::env::var("OPENROUTER_API_KEY").unwrap();
        let provider = OpenRouterEmbedProvider::new(&api_key, "openai/text-embedding-ada-002");
        let embedding1 = provider.embed("text one").await.unwrap();
        let embedding2 = provider.embed("text two").await.unwrap();

        assert!(
            !embedding1.is_empty(),
            "First embedding should not be empty"
        );
        assert!(
            !embedding2.is_empty(),
            "Second embedding should not be empty"
        );
        assert_ne!(
            embedding1, embedding2,
            "Different inputs should produce different embeddings"
        );
    }

    #[tokio::test]
    async fn test_openrouter_embedding_empty_string() {
        if !is_openrouter_configured() {
            eprintln!("Skipping test: OPENROUTER_API_KEY not configured");
            return;
        }

        let api_key = std::env::var("OPENROUTER_API_KEY").unwrap();
        let provider = OpenRouterEmbedProvider::new(&api_key, "openai/text-embedding-ada-002");
        let embedding = provider.embed("").await.unwrap();
        assert!(
            !embedding.is_empty(),
            "Empty string embedding should not be empty"
        );
        assert_eq!(embedding.len(), 1536);
    }

    #[tokio::test]
    async fn test_openrouter_embedding_long_text() {
        if !is_openrouter_configured() {
            eprintln!("Skipping test: OPENROUTER_API_KEY not configured");
            return;
        }

        let api_key = std::env::var("OPENROUTER_API_KEY").unwrap();
        let provider = OpenRouterEmbedProvider::new(&api_key, "openai/text-embedding-ada-002");
        let long_text = "a".repeat(10000);
        let embedding = provider.embed(&long_text).await.unwrap();
        assert!(
            !embedding.is_empty(),
            "Long text embedding should not be empty"
        );
        assert_eq!(embedding.len(), 1536);
    }

    #[tokio::test]
    async fn test_openrouter_embedding_special_characters() {
        if !is_openrouter_configured() {
            eprintln!("Skipping test: OPENROUTER_API_KEY not configured");
            return;
        }

        let api_key = std::env::var("OPENROUTER_API_KEY").unwrap();
        let provider = OpenRouterEmbedProvider::new(&api_key, "openai/text-embedding-ada-002");
        let special_text = "Hello 世界 🌍";
        let embedding = provider.embed(special_text).await.unwrap();
        assert!(
            !embedding.is_empty(),
            "Special characters embedding should not be empty"
        );
        assert_eq!(embedding.len(), 1536);
    }

    #[tokio::test]
    async fn test_openrouter_embedding_performance() {
        if !is_openrouter_configured() {
            eprintln!("Skipping test: OPENROUTER_API_KEY not configured");
            return;
        }

        let api_key = std::env::var("OPENROUTER_API_KEY").unwrap();
        let provider = OpenRouterEmbedProvider::new(&api_key, "openai/text-embedding-ada-002");
        let text = "performance test";

        // Warm up
        let _ = provider.embed("warm up").await.unwrap();

        let start = std::time::Instant::now();
        let _embedding = provider.embed(text).await.unwrap();
        let duration = start.elapsed();

        // Should complete in reasonable time (<1s target)
        assert!(
            duration.as_secs() < 1,
            "OpenRouter embedding should be <1s, took {}s",
            duration.as_secs()
        );
    }
}
