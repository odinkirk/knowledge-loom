use std::path::PathBuf;

pub mod local;
pub mod ollama;

use local::LocalEmbedProvider;
use ollama::OllamaEmbedProvider;

#[allow(dead_code)]
pub trait EmbedProvider: Send + Sync {
    fn embed(&self, text: &str) -> Vec<f32>;
    fn dimension(&self) -> usize;
}

pub enum EmbedProviderEnum {
    Local(LocalEmbedProvider),
    Ollama(OllamaEmbedProvider),
}

impl EmbedProviderEnum {
    pub async fn new(kb_root: &str) -> Self {
        let kb_root_path = PathBuf::from(kb_root);
        let models_dir = kb_root_path.join(".knowledge-loom-index/models");

        let ollama_url = std::env::var("OLLAMA_URL").ok();
        
        if let Some(url) = ollama_url {
            Self::Ollama(OllamaEmbedProvider::new(url))
        } else {
            Self::Local(LocalEmbedProvider::new(&models_dir))
        }
    }

    pub fn embed(&self, text: &str) -> Vec<f32> {
        match self {
            Self::Local(p) => p.embed(text),
            Self::Ollama(p) => p.embed(text),
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn dimension(&self) -> usize {
        match self {
            Self::Local(p) => p.dimension(),
            Self::Ollama(p) => p.dimension(),
        }
    }
}
