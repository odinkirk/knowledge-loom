use std::path::Path;
use std::sync::Arc;

pub struct LocalEmbedProvider {
    models_dir: Arc<Path>,
}

impl LocalEmbedProvider {
    pub async fn new(models_dir: &Path) -> Self {
        Self {
            models_dir: Arc::from(models_dir),
        }
    }
    
    pub async fn embed(&self, text: &str) -> Vec<f32> {
        // Simple mock embedding for testing
        // In production, this would use fastembed or another embedding model
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