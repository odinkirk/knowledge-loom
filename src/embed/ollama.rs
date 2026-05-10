use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::error::{EmbedError, Result};

/// Ollama embedding provider configuration
#[derive(Clone)]
pub struct OllamaEmbedProvider {
    ollama_url: Arc<String>,
    client: Client,
    model: String,
}

/// Ollama API request structure
#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
}

/// Ollama API response structure
#[derive(Deserialize)]
struct OllamaResponse {
    embedding: Vec<f32>,
}

impl OllamaEmbedProvider {
    /// Create a new Ollama embedding provider
    ///
    /// # Arguments
    ///
    /// * `ollama_url` - URL of the Ollama server (e.g., "http://localhost:11434")
    ///
    /// # Returns
    ///
    /// A new OllamaEmbedProvider instance
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let provider = OllamaEmbedProvider::new("http://localhost:11434".to_string());
    /// ```
    pub fn new(ollama_url: String) -> Self {
        eprintln!("Initializing Ollama embedding provider...");

        let client = Client::new();
        let model = "nomic-embed-text".to_string(); // Default model

        eprintln!("Ollama embedding provider initialized successfully");

        Self {
            ollama_url: ollama_url.into(),
            client,
            model,
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
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let embedding = provider.embed("Hello, world!")?;
    /// assert!(!embedding.is_empty());
    /// ```
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        eprintln!("Ollama embed called with: {}", text);

        // Make HTTP API call to Ollama
        let url = format!("{}/api/embeddings", self.ollama_url);
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: text.to_string(),
        };

        let response = self.client.post(&url).json(&request).send().await.map_err(|e| {
            EmbedError::NetworkError(format!("Failed to send request to Ollama: {}", e))
        })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = format!("Ollama API returned error: {}", response.status());
            return Err(EmbedError::HttpError { status, message });
        }

        let ollama_response: OllamaResponse = response.json().await.map_err(|e| {
            EmbedError::InvalidResponseFormat(format!("Failed to parse Ollama response: {}", e))
        })?;

        Ok(ollama_response.embedding)
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
    /// assert_eq!(dim, 768); // for nomic-embed-text-v1.5
    /// ```
    #[must_use]
    pub fn dimension(&self) -> usize {
        768 // nomic-embed-text-v1.5 dimension
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Check if Ollama service is available
    async fn is_ollama_available() -> bool {
        let client = Client::new();
        let url = "http://localhost:11434/api/tags";
        client.get(url).send().await.is_ok()
    }

    #[test]
    fn test_ollama_provider_creation() {
        let provider = OllamaEmbedProvider::new("http://localhost:11434".to_string());
        assert_eq!(provider.dimension(), 768);
    }

    #[tokio::test]
    async fn test_ollama_embedding() {
        if !is_ollama_available().await {
            eprintln!("Skipping test: Ollama service not available");
            return;
        }

        let provider = OllamaEmbedProvider::new("http://localhost:11434".to_string());
        let embedding = provider.embed("test").await.unwrap();
        assert!(!embedding.is_empty(), "Embedding should not be empty");
        assert_eq!(embedding.len(), 768);
    }

    #[tokio::test]
    async fn test_ollama_embedding_consistency() {
        if !is_ollama_available().await {
            eprintln!("Skipping test: Ollama service not available");
            return;
        }

        let provider = OllamaEmbedProvider::new("http://localhost:11434".to_string());
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
    async fn test_ollama_embedding_different_inputs() {
        if !is_ollama_available().await {
            eprintln!("Skipping test: Ollama service not available");
            return;
        }

        let provider = OllamaEmbedProvider::new("http://localhost:11434".to_string());
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
    async fn test_ollama_embedding_empty_string() {
        if !is_ollama_available().await {
            eprintln!("Skipping test: Ollama service not available");
            return;
        }

        let provider = OllamaEmbedProvider::new("http://localhost:11434".to_string());
        let embedding = provider.embed("").await.unwrap();
        assert!(
            !embedding.is_empty(),
            "Empty string embedding should not be empty"
        );
        assert_eq!(embedding.len(), 768);
    }

    #[tokio::test]
    async fn test_ollama_embedding_long_text() {
        if !is_ollama_available().await {
            eprintln!("Skipping test: Ollama service not available");
            return;
        }

        let provider = OllamaEmbedProvider::new("http://localhost:11434".to_string());
        let long_text = "a".repeat(10000);
        let embedding = provider.embed(&long_text).await.unwrap();
        assert!(
            !embedding.is_empty(),
            "Long text embedding should not be empty"
        );
        assert_eq!(embedding.len(), 768);
    }

    #[tokio::test]
    async fn test_ollama_embedding_special_characters() {
        if !is_ollama_available().await {
            eprintln!("Skipping test: Ollama service not available");
            return;
        }

        let provider = OllamaEmbedProvider::new("http://localhost:11434".to_string());
        let special_text = "Hello 世界 🌍";
        let embedding = provider.embed(special_text).await.unwrap();
        assert!(
            !embedding.is_empty(),
            "Special characters embedding should not be empty"
        );
        assert_eq!(embedding.len(), 768);
    }

    #[tokio::test]
    async fn test_ollama_embedding_performance() {
        if !is_ollama_available().await {
            eprintln!("Skipping test: Ollama service not available");
            return;
        }

        let provider = OllamaEmbedProvider::new("http://localhost:11434".to_string());
        let text = "performance test";

        // Warm up
        let _ = provider.embed("warm up").await.unwrap();

        let start = std::time::Instant::now();
        let _embedding = provider.embed(text).await.unwrap();
        let duration = start.elapsed();

        // Should complete in reasonable time (<500ms target)
        assert!(
            duration.as_millis() < 500,
            "Ollama embedding should be <500ms, took {}ms",
            duration.as_millis()
        );
    }
}
