use reqwest::Client;
use std::time::Duration;

/// OpenRouter embedding provider configuration
#[derive(Clone)]
pub struct OpenRouterEmbedProvider {
    /// OpenRouter API key
    api_key: String,
    /// Model to use for embeddings
    model: String,
    /// HTTP client for API requests
    client: Client,
    /// Timeout for API requests
    timeout: Duration,
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
        let timeout = Duration::from_secs(10);
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");

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
    /// # Examples
    ///
    /// ```ignore
    /// let embedding = provider.embed("Hello, world!");
    /// assert!(!embedding.is_empty());
    /// ```
    pub fn embed(&self, text: &str) -> Vec<f32> {
        // TODO: Implement actual OpenRouter API call
        // For now, return a hash-based stub
        eprintln!("OpenRouter embed called with: {}", text);

        // Stub implementation - return hash-based embedding
        let hash = self.hash_text(text);
        let mut embedding = Vec::with_capacity(1536); // OpenAI ada-002 dimension
        for i in 0..1536 {
            embedding.push(((hash >> (i % 64)) & 0xFF) as f32 / 255.0);
        }
        embedding
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
    pub fn dimension(&self) -> usize {
        1536 // OpenAI ada-002 dimension
    }

    /// Hash text to generate consistent stub embeddings
    fn hash_text(&self, text: &str) -> u64 {
        let mut hash: u64 = 5381;
        for byte in text.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
        }
        hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openrouter_provider_creation() {
        let provider = OpenRouterEmbedProvider::new("test-key", "openai/text-embedding-ada-002");
        assert_eq!(provider.dimension(), 1536);
    }

    #[test]
    fn test_openrouter_embedding() {
        let provider = OpenRouterEmbedProvider::new("test-key", "openai/text-embedding-ada-002");
        let embedding = provider.embed("test");
        assert_eq!(embedding.len(), 1536);
    }

    #[test]
    fn test_openrouter_dimension() {
        let provider = OpenRouterEmbedProvider::new("test-key", "openai/text-embedding-ada-002");
        assert_eq!(provider.dimension(), 1536);
    }

    #[test]
    fn test_openrouter_embedding_consistency() {
        let provider = OpenRouterEmbedProvider::new("test-key", "openai/text-embedding-ada-002");
        let embedding1 = provider.embed("test");
        let embedding2 = provider.embed("test");
        assert_eq!(embedding1, embedding2);
    }

    #[test]
    fn test_openrouter_embedding_different_inputs() {
        let provider = OpenRouterEmbedProvider::new("test-key", "openai/text-embedding-ada-002");
        let embedding1 = provider.embed("text one");
        let embedding2 = provider.embed("text two");
        assert_ne!(embedding1, embedding2);
    }

    #[test]
    fn test_openrouter_embedding_empty_string() {
        let provider = OpenRouterEmbedProvider::new("test-key", "openai/text-embedding-ada-002");
        let embedding = provider.embed("");
        assert_eq!(embedding.len(), 1536);
    }

    #[test]
    fn test_openrouter_embedding_long_text() {
        let provider = OpenRouterEmbedProvider::new("test-key", "openai/text-embedding-ada-002");
        let long_text = "a".repeat(10000);
        let embedding = provider.embed(&long_text);
        assert_eq!(embedding.len(), 1536);
    }

    #[test]
    fn test_openrouter_embedding_special_characters() {
        let provider = OpenRouterEmbedProvider::new("test-key", "openai/text-embedding-ada-002");
        let special_text = "Hello 世界 🌍";
        let embedding = provider.embed(special_text);
        assert_eq!(embedding.len(), 1536);
    }

    #[test]
    fn test_openrouter_embedding_performance() {
        let provider = OpenRouterEmbedProvider::new("test-key", "openai/text-embedding-ada-002");
        let text = "performance test";

        let start = std::time::Instant::now();
        let _embedding = provider.embed(text);
        let duration = start.elapsed();

        // Should complete in reasonable time (<1s target)
        assert!(
            duration.as_millis() < 1000,
            "OpenRouter embedding should be <1s"
        );
    }
}
