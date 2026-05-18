use std::path::PathBuf;

use async_trait::async_trait;

pub mod error;
pub mod local;
pub mod ollama;
pub mod openrouter;

pub use error::{EmbedError, Result};
pub use local::LocalEmbedProvider;
pub use ollama::OllamaEmbedProvider;
pub use openrouter::OpenRouterEmbedProvider;

/// Trait for embedding providers that can generate text embeddings
///
/// This trait defines the interface for all embedding providers, including
/// local providers (using fastembed) and external providers (Ollama, OpenRouter).
///
/// # Examples
///
/// ```ignore
/// use knowledge_loom::embed::EmbedProvider;
///
/// let provider = LocalEmbedProvider::new(&models_dir);
/// let embedding = provider.embed("Hello, world!").await?;
/// assert_eq!(embedding.len(), provider.dimension());
/// ```
#[allow(dead_code)]
#[async_trait]
pub trait EmbedProvider: Send + Sync {
    /// Generate an embedding vector for the given text
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
    /// - The model is not loaded
    /// - The text is empty or invalid
    /// - The embedding generation fails
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let embedding = provider.embed("Hello, world!").await?;
    /// assert!(!embedding.is_empty());
    /// ```
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate embeddings for a batch of texts
    ///
    /// Default implementation falls back to single-text `embed()` in a loop.
    /// Providers that support native batching (e.g., fastembed) should override.
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }

    /// Get the dimension of the embedding vectors produced by this provider
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
    fn dimension(&self) -> usize;
}

/// Enum representing all available embedding providers
///
/// This enum allows switching between different embedding providers at runtime,
/// with support for local and external providers.
///
/// # Examples
///
/// ```ignore
/// use knowledge_loom::embed::EmbedProviderEnum;
///
/// let provider = EmbedProviderEnum::new("/path/to/kb");
/// let embedding = provider.embed("Hello, world!");
/// ```
#[derive(Clone)]
pub enum EmbedProviderEnum {
    /// Local embedding provider using fastembed
    Local(LocalEmbedProvider),
    /// Ollama embedding provider
    Ollama(OllamaEmbedProvider),
    /// OpenRouter embedding provider
    OpenRouter(OpenRouterEmbedProvider),
}

impl EmbedProviderEnum {
    /// Create a new embedding provider based on environment configuration
    ///
    /// This method checks the following environment variables in order:
    /// 1. `OLLAMA_URL` - If set, uses Ollama provider
    /// 2. `OPENROUTER_API_KEY` - If set, uses OpenRouter provider
    /// 3. Default - Uses local provider with fastembed
    ///
    /// # Arguments
    ///
    /// * `kb_root` - Path to the knowledge base root directory
    ///
    /// # Returns
    ///
    /// A new EmbedProviderEnum instance
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let provider = EmbedProviderEnum::new("/path/to/kb");
    /// ```
    pub fn new(kb_root: &str) -> Self {
        let kb_root_path = PathBuf::from(kb_root);
        let models_dir = kb_root_path.join(".knowledge-loom-index/models");

        let ollama_url = std::env::var("OLLAMA_URL").ok();
        let openrouter_api_key = std::env::var("OPENROUTER_API_KEY").ok();
        let openrouter_model = std::env::var("OPENROUTER_MODEL")
            .unwrap_or_else(|_| "openai/text-embedding-ada-002".to_string());

        // Provider priority: OpenRouter > Ollama > Local
        if let Some(api_key) = openrouter_api_key {
            eprintln!("Using OpenRouter embedding provider");
            Self::OpenRouter(OpenRouterEmbedProvider::new(api_key, openrouter_model))
        } else if let Some(url) = ollama_url {
            eprintln!("Using Ollama embedding provider");
            Self::Ollama(OllamaEmbedProvider::new(url))
        } else {
            eprintln!("Using local embedding provider");
            Self::Local(LocalEmbedProvider::new(&models_dir))
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
    /// let embedding = provider.embed("Hello, world!").await?;
    /// assert!(!embedding.is_empty());
    /// ```
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        match self {
            Self::Local(p) => p.embed(text).await,
            Self::Ollama(p) => {
                match p.embed(text).await {
                    Ok(embedding) => Ok(embedding),
                    Err(ollama_error) => {
                        eprintln!(
                            "Ollama provider failed: {}, falling back to local",
                            ollama_error
                        );
                        // Fall back to local provider
                        // Note: This uses a hardcoded path since we don't have access to the original kb_root
                        // In a future refactor, we should store kb_root in the enum to avoid this limitation
                        let models_dir = std::path::PathBuf::from(".knowledge-loom-index/models");
                        let local = LocalEmbedProvider::new(&models_dir);
                        match local.embed(text).await {
                            Ok(embedding) => Ok(embedding),
                            Err(local_error) => {
                                // Wrap both errors with context about which providers failed
                                Err(EmbedError::Custom(format!(
                                    "Both Ollama and local providers failed. Ollama error: {}, Local error: {}",
                                    ollama_error, local_error
                                )))
                            }
                        }
                    }
                }
            }
            Self::OpenRouter(p) => {
                match p.embed(text).await {
                    Ok(embedding) => Ok(embedding),
                    Err(openrouter_error) => {
                        eprintln!(
                            "OpenRouter provider failed: {}, falling back to local",
                            openrouter_error
                        );
                        // Fall back to local provider
                        // Note: This uses a hardcoded path since we don't have access to the original kb_root
                        // In a future refactor, we should store kb_root in the enum to avoid this limitation
                        let models_dir = std::path::PathBuf::from(".knowledge-loom-index/models");
                        let local = LocalEmbedProvider::new(&models_dir);
                        match local.embed(text).await {
                            Ok(embedding) => Ok(embedding),
                            Err(local_error) => {
                                // Wrap both errors with context about which providers failed
                                Err(EmbedError::Custom(format!(
                                    "Both OpenRouter and local providers failed. OpenRouter error: {}, Local error: {}",
                                    openrouter_error, local_error
                                )))
                            }
                        }
                    }
                }
            }
        }
    }

    /// Generate embeddings for a batch of texts
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        match self {
            Self::Local(p) => p.embed_batch(texts).await,
            // Ollama and OpenRouter fall back to single-text loop
            _ => {
                let mut results = Vec::with_capacity(texts.len());
                for text in texts {
                    results.push(self.embed(text).await?);
                }
                Ok(results)
            }
        }
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
    /// assert!(dim > 0);
    /// ```
    #[must_use]
    #[allow(dead_code)]
    pub fn dimension(&self) -> usize {
        match self {
            Self::Local(p) => p.dimension(),
            Self::Ollama(p) => p.dimension(),
            Self::OpenRouter(p) => p.dimension(),
        }
    }
}
