use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod local;
pub mod ollama;

use local::LocalEmbedProvider;
use ollama::OllamaEmbedProvider;

pub trait EmbedProvider: Send + Sync {
    fn embed(&self, text: &str) -> Vec<f32>;
    fn dimension(&self) -> usize;
}

pub struct EmbedProviderEnum {
    pub local: Arc<Mutex<LocalEmbedProvider>>,
    pub ollama: Option<Arc<Mutex<OllamaEmbedProvider>>>,
    pub use_ollama: bool,
}

impl EmbedProviderEnum {
    pub async fn new(kb_root: &str) -> Self {
        let kb_root_path = PathBuf::from(kb_root);
        let models_dir = kb_root_path.join(".loom-index/models");
        
        let local_provider = LocalEmbedProvider::new(&models_dir).await;
        let ollama_url = std::env::var("OLLAMA_URL").ok();
        let use_ollama = ollama_url.is_some();
        
        let ollama_provider = if use_ollama {
            Some(Arc::new(Mutex::new(OllamaEmbedProvider::new(ollama_url.unwrap()).await)))
        } else {
            None
        };
        
        Self {
            local: Arc::new(Mutex::new(local_provider)),
            ollama: ollama_provider,
            use_ollama,
        }
    }
    
    pub async fn embed(&self, text: &str) -> Vec<f32> {
        if self.use_ollama {
            if let Some(ref provider) = self.ollama {
                let provider_lock = provider.lock().await;
                return provider_lock.embed(text).await;
            }
        }
        // Fallback to local
        let provider_lock = self.local.lock().await;
        provider_lock.embed(text).await
    }
    
    pub fn dimension(&self) -> usize {
        // Assuming all models have same dimension for simplicity
        // In practice, we'd need to handle different dimensions
        384 // all-MiniLM-L6-v2 dimension
    }
}