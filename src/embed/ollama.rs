use std::hash::Hasher;
use std::sync::Arc;

#[derive(Clone)]
#[allow(dead_code)]
pub struct OllamaEmbedProvider {
    ollama_url: Arc<String>,
}

impl OllamaEmbedProvider {
    pub fn new(ollama_url: String) -> Self {
        Self {
            ollama_url: ollama_url.into(),
        }
    }

    pub fn embed(&self, text: &str) -> Vec<f32> {
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

    #[must_use]
    pub fn dimension(&self) -> usize {
        768
    }
}
