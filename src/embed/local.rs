use std::hash::Hasher;
use std::path::Path;
use std::sync::Arc;

#[allow(dead_code)]
pub struct LocalEmbedProvider {
    models_dir: Arc<Path>,
}

impl LocalEmbedProvider {
    pub fn new(models_dir: &Path) -> Self {
        Self {
            models_dir: models_dir.to_path_buf().into(),
        }
    }

    pub fn embed(&self, text: &str) -> Vec<f32> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        hasher.write(text.as_bytes());
        let hash = hasher.finish();
        let mut embedding = vec![0.0f32; 384];
        for (idx, byte) in hash.to_le_bytes().iter().enumerate() {
            if idx < embedding.len() {
                embedding[idx] = f32::from(*byte) / 255.0;
            }
        }
        embedding
    }

    #[must_use]
    pub fn dimension(&self) -> usize {
        384
    }
}
