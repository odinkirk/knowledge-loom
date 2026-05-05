use std::sync::Arc;

pub struct OllamaEmbedProvider {
    ollama_url: Arc<String>,
}

impl OllamaEmbedProvider {
    pub async fn new(ollama_url: String) -> Self {
        Self {
            ollama_url: Arc::new(ollama_url),
        }
    }
    
    pub async fn embed(&self, text: &str) -> Vec<f32> {
        // Simple mock implementation for testing
        // In production, this would call the Ollama API
        let mut embedding = vec![0.0_f32; 384];
        
        // Create a simple hash-based embedding
        let bytes = text.as_bytes();
        for (i, &byte) in bytes.iter().enumerate() {
            let idx = i % 384;
            embedding[idx] = (byte as f32) / 255.0;
        }
        
        embedding
    }
    
    pub fn dimension(&self) -> usize {
        384
    }
}