use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::collections::HashMap;
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
    cache: Arc<tokio::sync::Mutex<EmbeddingCache>>,
    /// The dimension of the embedding vectors (384 for all-MiniLM-L6-v2)
    #[allow(dead_code)]
    dimension: usize,
}

/// Simple LRU cache for embeddings
struct EmbeddingCache {
    entries: HashMap<u64, Vec<f32>>,
    access_order: Vec<u64>,
    max_size: usize,
}

impl EmbeddingCache {
    fn new(max_size: usize) -> Self {
        Self {
            entries: HashMap::new(),
            access_order: Vec::new(),
            max_size,
        }
    }

    fn get(&mut self, key: u64) -> Option<Vec<f32>> {
        if let Some(pos) = self.access_order.iter().position(|&k| k == key) {
            // Move to end (most recently used)
            let key = self.access_order.remove(pos);
            self.access_order.push(key);
            // Return a cloned value to avoid race conditions
            self.entries.get(&key).cloned()
        } else {
            None
        }
    }

    fn put(&mut self, key: u64, value: Vec<f32>) {
        // Evict if at capacity
        if self.entries.len() >= self.max_size && !self.entries.contains_key(&key) {
            if let Some(old_key) = self.access_order.first() {
                self.entries.remove(old_key);
                self.access_order.remove(0);
            }
        }

        // Update or insert
        if self.entries.contains_key(&key) {
            if let Some(pos) = self.access_order.iter().position(|&k| k == key) {
                self.access_order.remove(pos);
            }
        }

        self.entries.insert(key, value);
        self.access_order.push(key);
    }

    #[allow(dead_code)]
    fn len(&self) -> usize {
        self.entries.len()
    }
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
            eprintln!("Failed to create models directory: {e}");
        });

        // Configure ONNX Runtime for multi-threaded execution
        if std::env::var("ORT_NUM_THREADS").is_err() {
            let n_threads = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4);
            std::env::set_var("ORT_NUM_THREADS", n_threads.to_string());
        }
        // Initialize fastembed model
        let init_options =
            InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(false);

        let model =
            TextEmbedding::try_new(init_options).expect("Failed to initialize fastembed model");

        eprintln!("Local embedding provider initialized successfully");

        // Initialize cache with default size of 1000 embeddings
        let cache_size = std::env::var("LOOM_EMBED_CACHE_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1000);

        eprintln!("Embedding cache initialized with size: {cache_size}");

        // The dimension is a constant for all-MiniLM-L6-v2
        let dimension = 384;

        Self {
            model: Arc::new(model),
            models_dir: models_dir.to_path_buf().into(),
            cache: Arc::new(tokio::sync::Mutex::new(EmbeddingCache::new(cache_size))),
            dimension,
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
        // Compute cache key from text hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let cache_key = hasher.finish();

        // Check cache
        {
            let mut cache = self.cache.lock().await;
            if let Some(cached_embedding) = cache.get(cache_key) {
                // Cache hit - no logging in production
                return Ok(cached_embedding);
            }
            // Cache miss - no logging in production
        }

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
        .map_err(|e| EmbedError::EmbeddingError(format!("Task join error: {}", e)))??;

        // Get the embedding
        let embedding = embeddings
            .into_iter()
            .next()
            .ok_or_else(|| EmbedError::EmbeddingError("No embedding generated".to_string()))?;

        // Store in cache
        {
            let mut cache = self.cache.lock().await;
            cache.put(cache_key, embedding.clone());
            // No logging in production
        }

        Ok(embedding)
    }

    /// Generate embeddings for a batch of texts using native fastembed batch inference
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::with_capacity(texts.len());
        let mut uncached_texts: Vec<String> = Vec::new();
        let mut uncached_indices: Vec<usize> = Vec::new();

        // Check cache for each text
        {
            let mut cache = self.cache.lock().await;
            for (i, text) in texts.iter().enumerate() {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                text.hash(&mut hasher);
                let key = hasher.finish();

                if let Some(cached) = cache.get(key) {
                    // Pad results with None placeholders, we'll fill in later
                    while results.len() <= i {
                        results.push(None);
                    }
                    results[i] = Some(cached.clone());
                } else {
                    uncached_texts.push(text.clone());
                    uncached_indices.push(i);
                }
            }
        }

        // Batch-embed uncached texts
        if !uncached_texts.is_empty() {
            let model = self.model.clone();
            let embeddings = tokio::task::spawn_blocking(move || {
                model.embed(uncached_texts, None).map_err(|e| {
                    EmbedError::EmbeddingError(format!("Failed to generate batch embedding: {}", e))
                })
            })
            .await
            .map_err(|e| EmbedError::EmbeddingError(format!("Task join error: {}", e)))??;

            // Store in cache and map results
            let mut cache = self.cache.lock().await;
            for (idx, embedding) in uncached_indices.iter().zip(embeddings.into_iter()) {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                texts[*idx].hash(&mut hasher);
                let key = hasher.finish();

                cache.put(key, embedding.clone());

                while results.len() <= *idx {
                    results.push(None);
                }
                results[*idx] = Some(embedding);
            }
        }

        // Unwrap all results
        results
            .into_iter()
            .map(|r| {
                r.ok_or_else(|| EmbedError::EmbeddingError("Missing embedding result".to_string()))
            })
            .collect()
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
    #[allow(dead_code)]
    pub fn dimension(&self) -> usize {
        self.dimension
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

        // Should complete in reasonable time (<150ms target)
        assert!(
            duration.as_millis() < 150,
            "Local embedding should be <150ms, took {}ms",
            duration.as_millis()
        );
    }
}
