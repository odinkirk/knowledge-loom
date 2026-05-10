use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::path::Path;
use std::sync::Arc;

use super::error::{EmbedError, Result};

/// Local embedding provider using fastembed
///
/// This provider uses the fastembed library to generate embeddings locally
/// using the all-MiniLM-L6-v2 model (384 dimensions).
///
/// # Examples
///
/// ```ignore
/// use knowledge_loom::embed::LocalEmbedProvider;
///
/// let models_dir = PathBuf::from(".knowledge-loom-index/models");
/// let provider = LocalEmbedProvider::new(&models_dir);
/// let embedding = provider.embed("Hello, world!").unwrap();
/// assert_eq!(embedding.len(), 384);
/// ```
#[derive(Clone)]
pub struct LocalEmbedProvider {
    model: Arc<TextEmbedding>,
    #[allow(dead_code)]
    models_dir: Arc<Path>,
}

impl LocalEmbedProvider {
    /// Create a new local embedding provider
    ///
    /// # Arguments
    ///
    /// * `models_dir` - Directory to store downloaded models
    ///
    /// # Returns
    ///
    /// A new LocalEmbedProvider instance
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let models_dir = PathBuf::from(".knowledge-loom-index/models");
    /// let provider = LocalEmbedProvider::new(&models_dir);
    /// ```
    pub fn new(models_dir: &Path) -> Self {
        eprintln!("Initializing local embedding provider...");

        // Create models directory if it doesn't exist
        std::fs::create_dir_all(models_dir).unwrap_or_else(|e| {
            eprintln!("Failed to create models directory: {}", e);
        });

        // Initialize fastembed model
        let init_options =
            InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(false);

        let model =
            TextEmbedding::try_new(init_options).expect("Failed to initialize fastembed model");

        eprintln!("Local embedding provider initialized successfully");

        Self {
            model: Arc::new(model),
            models_dir: models_dir.to_path_buf().into(),
        }
    }

    /// Generate an embedding for the given text
    ///
    /// # Arguments
    ///
    /// * `text` - The text to embed
    ///
    /// # Returns
    ///
    /// A vector of floats representing the embedding
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The model is not loaded
    /// - The embedding generation fails
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let embedding = provider.embed("Hello, world!").await?;
    /// assert!(!embedding.is_empty());
    /// ```
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Use fastembed to generate real embeddings
        // fastembed is synchronous, so we use spawn_blocking to avoid blocking the async runtime
        let text = text.to_string();
        let model = self.model.clone();
        
        let embeddings = tokio::task::spawn_blocking(move || {
            model.embed(vec![text], None).map_err(|e| {
                EmbedError::EmbeddingError(format!("Failed to generate embedding: {}", e))
            })
        })
        .await
        .map_err(|e| {
            EmbedError::EmbeddingError(format!("Task join error: {}", e))
        })??;

        // Return the first (and only) embedding
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| EmbedError::EmbeddingError("No embedding generated".to_string()))
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
    /// assert_eq!(dim, 384); // for all-MiniLM-L6-v2
    /// ```
    #[must_use]
    pub fn dimension(&self) -> usize {
        // Get the actual dimension from the model
        // For all-MiniLM-L6-v2, this should be 384
        self.model
            .embed(vec!["test"], None)
            .expect("Failed to get model dimension")
            .into_iter()
            .next()
            .map(|v| v.len())
            .unwrap_or(384)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_local_provider_creation() {
        let models_dir = PathBuf::from(".knowledge-loom-index/models");
        let provider = LocalEmbedProvider::new(&models_dir);
        assert_eq!(provider.dimension(), 384);
    }

    #[tokio::test]
    async fn test_local_embedding() {
        let models_dir = PathBuf::from(".knowledge-loom-index/models");
        let provider = LocalEmbedProvider::new(&models_dir);
        let embedding = provider.embed("test text").await.unwrap();
        assert_eq!(embedding.len(), 384);
        assert!(embedding.iter().any(|&x| x != 0.0));
    }

    #[test]
    fn test_local_dimension() {
        let models_dir = PathBuf::from(".knowledge-loom-index/models");
        let provider = LocalEmbedProvider::new(&models_dir);
        assert_eq!(provider.dimension(), 384);
    }

    #[tokio::test]
    async fn test_local_embedding_consistency() {
        let models_dir = PathBuf::from(".knowledge-loom-index/models");
        let provider = LocalEmbedProvider::new(&models_dir);
        let text = "consistent test";
        let embedding1 = provider.embed(text).await.unwrap();
        let embedding2 = provider.embed(text).await.unwrap();
        assert_eq!(embedding1, embedding2);
    }

    #[tokio::test]
    async fn test_local_embedding_different_inputs() {
        let models_dir = PathBuf::from(".knowledge-loom-index/models");
        let provider = LocalEmbedProvider::new(&models_dir);
        let embedding1 = provider.embed("text one").await.unwrap();
        let embedding2 = provider.embed("text two").await.unwrap();
        assert_ne!(embedding1, embedding2);
    }

    #[tokio::test]
    async fn test_local_embedding_empty_string() {
        let models_dir = PathBuf::from(".knowledge-loom-index/models");
        let provider = LocalEmbedProvider::new(&models_dir);
        let embedding = provider.embed("").await.unwrap();
        assert_eq!(embedding.len(), 384);
    }

    #[tokio::test]
    async fn test_local_embedding_long_text() {
        let models_dir = PathBuf::from(".knowledge-loom-index/models");
        let provider = LocalEmbedProvider::new(&models_dir);
        let long_text = "a".repeat(10000);
        let embedding = provider.embed(&long_text).await.unwrap();
        assert_eq!(embedding.len(), 384);
    }

    #[tokio::test]
    async fn test_local_embedding_special_characters() {
        let models_dir = PathBuf::from(".knowledge-loom-index/models");
        let provider = LocalEmbedProvider::new(&models_dir);
        let special_text = "Hello 世界 🌍";
        let embedding = provider.embed(special_text).await.unwrap();
        assert_eq!(embedding.len(), 384);
    }

    #[tokio::test]
    async fn test_local_embedding_performance() {
        let models_dir = PathBuf::from(".knowledge-loom-index/models");
        let provider = LocalEmbedProvider::new(&models_dir);
        let text = "performance test";

        // Warm up the model with a dummy call
        let _ = provider.embed("warm up").await.unwrap();

        let start = std::time::Instant::now();
        let _embedding = provider.embed(text).await.unwrap();
        let duration = start.elapsed();

        // Should complete in reasonable time (<100ms target)
        assert!(
            duration.as_millis() < 100,
            "Local embedding should be <100ms, took {}ms",
            duration.as_millis()
        );
    }
}
