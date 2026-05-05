#[cfg(test)]
mod tests {
    use super::*;
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
}