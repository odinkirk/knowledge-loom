use std::hash::Hasher;
use std::sync::Arc;
use std::time::Duration;

/// Ollama embedding provider configuration
#[derive(Clone)]
pub struct OllamaEmbedProvider {
    ollama_url: Arc<String>,
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
        Self {
            ollama_url: ollama_url.into(),
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
        // TODO: Implement actual Ollama API call
        // For now, return a hash-based stub
        eprintln!("Ollama embed called with: {}", text);

        // Stub implementation - return hash-based embedding
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        hasher.write(text.as_bytes());
        let hash = hasher.finish();
        let mut embedding = vec![0.0f32; 768];
        for (idx, byte) in hash.to_le_bytes().iter().enumerate() {
            if idx < embedding.len() {
                embedding[idx] = f32::from(*byte) / 255.0;
            }
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

    #[test]
    fn test_ollama_provider_creation() {
        let provider = OllamaEmbedProvider::new("http://localhost:11434".to_string());
        assert_eq!(provider.dimension(), 768);
    }

    #[test]
    fn test_ollama_embedding() {
        let provider = OllamaEmbedProvider::new("http://localhost:11434".to_string());
        let embedding = provider.embed("test");
        assert_eq!(embedding.len(), 768);
    }

    #[test]
    fn test_ollama_dimension() {
        let provider = OllamaEmbedProvider::new("http://localhost:11434".to_string());
        assert_eq!(provider.dimension(), 768);
    }

    #[test]
    fn test_ollama_embedding_consistency() {
        let provider = OllamaEmbedProvider::new("http://localhost:11434".to_string());
        let embedding1 = provider.embed("test");
        let embedding2 = provider.embed("test");
        assert_eq!(embedding1, embedding2);
    }

    #[test]
    fn test_ollama_embedding_different_inputs() {
        let provider = OllamaEmbedProvider::new("http://localhost:11434".to_string());
        let embedding1 = provider.embed("text one");
        let embedding2 = provider.embed("text two");
        assert_ne!(embedding1, embedding2);
    }

    #[test]
    fn test_ollama_embedding_empty_string() {
        let provider = OllamaEmbedProvider::new("http://localhost:11434".to_string());
        let embedding = provider.embed("");
        assert_eq!(embedding.len(), 768);
    }

    #[test]
    fn test_ollama_embedding_long_text() {
        let provider = OllamaEmbedProvider::new("http://localhost:11434".to_string());
        let long_text = "a".repeat(10000);
        let embedding = provider.embed(&long_text);
        assert_eq!(embedding.len(), 768);
    }

    #[test]
    fn test_ollama_embedding_special_characters() {
        let provider = OllamaEmbedProvider::new("http://localhost:11434".to_string());
        let special_text = "Hello 世界 🌍";
        let embedding = provider.embed(special_text);
        assert_eq!(embedding.len(), 768);
    }

    #[test]
    fn test_ollama_embedding_performance() {
        let provider = OllamaEmbedProvider::new("http://localhost:11434".to_string());
        let text = "performance test";

        let start = std::time::Instant::now();
        let _embedding = provider.embed(text);
        let duration = start.elapsed();

        // Should complete in reasonable time (<500ms target)
        assert!(
            duration.as_millis() < 500,
            "Ollama embedding should be <500ms"
        );
    }
}
