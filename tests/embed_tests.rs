// Integration tests for embedding providers
// Tests for local, Ollama, and OpenRouter embedding providers

use knowledge_loom::embed::{LocalEmbedProvider, OllamaEmbedProvider, OpenRouterEmbedProvider};
use std::path::PathBuf;

#[cfg(test)]
mod local_tests {
    use super::*;

    #[test]
    fn test_local_provider_creation() {
        let models_dir = PathBuf::from(".knowledge-loom-index/models");
        let provider = LocalEmbedProvider::new(&models_dir);
        // Provider should be created successfully
        assert_eq!(provider.dimension(), 384);
    }

    #[test]
    fn test_local_embedding() {
        let models_dir = PathBuf::from(".knowledge-loom-index/models");
        let provider = LocalEmbedProvider::new(&models_dir);
        let embedding = provider.embed("test text");
        // Embedding should have correct dimension
        assert_eq!(embedding.len(), 384);
        // Embedding should be non-zero
        assert!(embedding.iter().any(|&x| x != 0.0));
    }

    #[test]
    fn test_local_dimension() {
        let models_dir = PathBuf::from(".knowledge-loom-index/models");
        let provider = LocalEmbedProvider::new(&models_dir);
        assert_eq!(provider.dimension(), 384);
    }

    #[test]
    fn test_local_embedding_consistency() {
        let models_dir = PathBuf::from(".knowledge-loom-index/models");
        let provider = LocalEmbedProvider::new(&models_dir);
        let text = "consistent test";
        let embedding1 = provider.embed(text);
        let embedding2 = provider.embed(text);
        // Embeddings should be consistent for the same input
        assert_eq!(embedding1, embedding2);
    }

    #[test]
    fn test_local_embedding_different_inputs() {
        let models_dir = PathBuf::from(".knowledge-loom-index/models");
        let provider = LocalEmbedProvider::new(&models_dir);
        let embedding1 = provider.embed("text one");
        let embedding2 = provider.embed("text two");
        // Embeddings should be different for different inputs
        assert_ne!(embedding1, embedding2);
    }

    #[test]
    fn test_local_embedding_empty_string() {
        let models_dir = PathBuf::from(".knowledge-loom-index/models");
        let provider = LocalEmbedProvider::new(&models_dir);
        let embedding = provider.embed("");
        // Should handle empty string gracefully
        assert_eq!(embedding.len(), 384);
    }

    #[test]
    fn test_local_embedding_long_text() {
        let models_dir = PathBuf::from(".knowledge-loom-index/models");
        let provider = LocalEmbedProvider::new(&models_dir);
        let long_text = "a".repeat(10000);
        let embedding = provider.embed(&long_text);
        // Should handle long text
        assert_eq!(embedding.len(), 384);
    }

    #[test]
    fn test_local_embedding_special_characters() {
        let models_dir = PathBuf::from(".knowledge-loom-index/models");
        let provider = LocalEmbedProvider::new(&models_dir);
        let special_text = "Hello 世界 🌍";
        let embedding = provider.embed(special_text);
        // Should handle unicode and special characters
        assert_eq!(embedding.len(), 384);
    }

    #[test]
    fn test_local_embedding_performance() {
        let models_dir = PathBuf::from(".knowledge-loom-index/models");
        let provider = LocalEmbedProvider::new(&models_dir);
        let text = "performance test";

        let start = std::time::Instant::now();
        let _embedding = provider.embed(text);
        let duration = start.elapsed();

        // Should complete in reasonable time (<100ms target)
        assert!(
            duration.as_millis() < 100,
            "Local embedding should be <100ms"
        );
    }
}

#[cfg(test)]
mod ollama_tests {
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
}

#[cfg(test)]
mod openrouter_tests {
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
}

#[cfg(test)]
mod provider_enum_tests {
    use super::*;
    use knowledge_loom::embed::EmbedProviderEnum;
    use std::path::PathBuf;

    #[test]
    fn test_provider_enum_local() {
        let models_dir = PathBuf::from(".knowledge-loom-index/models");
        let provider = EmbedProviderEnum::Local(LocalEmbedProvider::new(&models_dir));
        assert_eq!(provider.dimension(), 384);

        let embedding = provider.embed("test");
        assert_eq!(embedding.len(), 384);
    }

    #[test]
    fn test_provider_enum_ollama() {
        let provider = EmbedProviderEnum::Ollama(OllamaEmbedProvider::new(
            "http://localhost:11434".to_string(),
        ));
        assert_eq!(provider.dimension(), 768);

        let embedding = provider.embed("test");
        assert_eq!(embedding.len(), 768);
    }

    #[test]
    fn test_provider_enum_openrouter() {
        let provider = EmbedProviderEnum::OpenRouter(OpenRouterEmbedProvider::new(
            "test-key",
            "openai/text-embedding-ada-002",
        ));
        assert_eq!(provider.dimension(), 1536);

        let embedding = provider.embed("test");
        assert_eq!(embedding.len(), 1536);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use knowledge_loom::embed::EmbedProviderEnum;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_end_to_end_embedding() {
        let models_dir = PathBuf::from(".knowledge-loom-index/models");
        let provider = LocalEmbedProvider::new(&models_dir);

        // Test embedding generation
        let embedding = provider.embed("end to end test");
        assert_eq!(embedding.len(), 384);
        assert!(embedding.iter().any(|&x| x != 0.0));

        // Test dimension
        assert_eq!(provider.dimension(), 384);
    }

    #[test]
    fn test_embedding_consistency() {
        let models_dir = PathBuf::from(".knowledge-loom-index/models");
        let provider = LocalEmbedProvider::new(&models_dir);

        let text = "consistency test";
        let embeddings: Vec<Vec<f32>> = (0..10).map(|_| provider.embed(text)).collect();

        // All embeddings should be identical
        for embedding in &embeddings[1..] {
            assert_eq!(embeddings[0], *embedding);
        }
    }

    #[test]
    fn test_embedding_dimensions() {
        let models_dir = PathBuf::from(".knowledge-loom-index/models");

        // Test all providers return correct dimensions
        let local = LocalEmbedProvider::new(&models_dir);
        assert_eq!(local.dimension(), 384);

        let ollama = OllamaEmbedProvider::new("http://localhost:11434".to_string());
        assert_eq!(ollama.dimension(), 768);

        let openrouter = OpenRouterEmbedProvider::new("test-key", "openai/text-embedding-ada-002");
        assert_eq!(openrouter.dimension(), 1536);
    }
}
