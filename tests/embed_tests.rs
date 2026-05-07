#[cfg(test)]
mod tests {
    
    use tempfile::TempDir;
    use loom::embed::EmbedProviderEnum;

    #[tokio::test]
    async fn test_embed_provider_enum_new() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();
        
        let provider = EmbedProviderEnum::new(kb_root.to_str().unwrap()).await;
        
        assert!(!provider.use_ollama); // Should default to local
        assert_eq!(provider.dimension(), 384);
    }

    #[tokio::test]
    async fn test_embed_local_provider() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();
        
        let provider = EmbedProviderEnum::new(kb_root.to_str().unwrap()).await;
        
        // Test embedding
        let text = "This is a test text for embedding";
        let embedding = provider.embed(text).await;
        
        assert!(!embedding.is_empty());
        assert_eq!(embedding.len(), 384); // all-MiniLM-L6-v2 dimension
        
        // Verify embedding values are reasonable
        for &val in &embedding {
            assert!(val >= -1.0 && val <= 1.0);
        }
    }

    #[tokio::test]
    async fn test_embed_different_texts() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();
        
        let provider = EmbedProviderEnum::new(kb_root.to_str().unwrap()).await;
        
        let text1 = "This is about cats";
        let text2 = "This is about dogs";
        
        let embed1 = provider.embed(text1).await;
        let embed2 = provider.embed(text2).await;
        
        // Embeddings should be different
        let similarity = cosine_similarity(&embed1, &embed2);
        assert!(similarity < 0.999); // Not identical (relaxed threshold for simple embeddings)
    }

    #[tokio::test]
    async fn test_embed_same_text() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();
        
        let provider = EmbedProviderEnum::new(kb_root.to_str().unwrap()).await;
        
        let text = "This is a test";
        let embed1 = provider.embed(text).await;
        let embed2 = provider.embed(text).await;
        
        // Same text should produce same embedding
        let similarity = cosine_similarity(&embed1, &embed2);
        assert!(similarity > 0.999); // Nearly identical
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }

    #[tokio::test]
    async fn test_ollama_provider_new() {
        use loom::embed::ollama::OllamaEmbedProvider;
        
        let url = "http://localhost:11434".to_string();
        let provider = OllamaEmbedProvider::new(url.clone()).await;
        
        // Provider should be created successfully
        // We can't directly access the ollama_url field as it's private,
        // but we can test that the provider works
        assert_eq!(provider.dimension(), 384);
    }

    #[tokio::test]
    async fn test_ollama_provider_embed() {
        use loom::embed::ollama::OllamaEmbedProvider;
        
        let url = "http://localhost:11434".to_string();
        let provider = OllamaEmbedProvider::new(url).await;
        
        let text = "This is a test text for embedding";
        let embedding = provider.embed(text).await;
        
        assert!(!embedding.is_empty());
        assert_eq!(embedding.len(), 384);
        
        // Verify embedding values are reasonable (0.0 to 1.0 range for this implementation)
        for &val in &embedding {
            assert!(val >= 0.0 && val <= 1.0);
        }
    }

    #[tokio::test]
    async fn test_ollama_provider_different_texts() {
        use loom::embed::ollama::OllamaEmbedProvider;
        
        let url = "http://localhost:11434".to_string();
        let provider = OllamaEmbedProvider::new(url).await;
        
        let text1 = "This is about cats";
        let text2 = "This is about dogs";
        
        let embed1 = provider.embed(text1).await;
        let embed2 = provider.embed(text2).await;
        
        // Embeddings should be different
        assert_ne!(embed1, embed2);
    }

    #[tokio::test]
    async fn test_ollama_provider_same_text() {
        use loom::embed::ollama::OllamaEmbedProvider;
        
        let url = "http://localhost:11434".to_string();
        let provider = OllamaEmbedProvider::new(url).await;
        
        let text = "This is a test";
        let embed1 = provider.embed(text).await;
        let embed2 = provider.embed(text).await;
        
        // Same text should produce same embedding
        assert_eq!(embed1, embed2);
    }

    #[tokio::test]
    async fn test_ollama_provider_dimension() {
        use loom::embed::ollama::OllamaEmbedProvider;
        
        let url = "http://localhost:11434".to_string();
        let provider = OllamaEmbedProvider::new(url).await;
        
        assert_eq!(provider.dimension(), 384);
    }

    #[tokio::test]
    async fn test_ollama_provider_empty_text() {
        use loom::embed::ollama::OllamaEmbedProvider;
        
        let url = "http://localhost:11434".to_string();
        let provider = OllamaEmbedProvider::new(url).await;
        
        let text = "";
        let embedding = provider.embed(text).await;
        
        // Empty text should still produce a valid embedding
        assert_eq!(embedding.len(), 384);
        // All values should be 0.0 for empty text
        for &val in &embedding {
            assert_eq!(val, 0.0);
        }
    }

    #[tokio::test]
    async fn test_ollama_provider_long_text() {
        use loom::embed::ollama::OllamaEmbedProvider;
        
        let url = "http://localhost:11434".to_string();
        let provider = OllamaEmbedProvider::new(url).await;
        
        let text = "a".repeat(1000); // Long text
        let embedding = provider.embed(&text).await;
        
        // Long text should still produce a valid embedding
        assert_eq!(embedding.len(), 384);
    }
}