use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::fs;
use walkdir::WalkDir;

pub struct VaultState {
    pub kb_root: PathBuf,
    pub ignored_patterns: HashSet<String>,
}

impl VaultState {
    pub async fn new(kb_root: &str) -> Self {
        let kb_root_path = PathBuf::from(kb_root);
        let mut ignored_patterns = HashSet::new();
        
        // Load .loomignore if exists
        let loomignore_path = kb_root_path.join(".loomignore");
        if loomignore_path.exists() {
            if let Ok(content) = fs::read_to_string(&loomignore_path).await {
                for line in content.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
                        ignored_patterns.insert(line.to_string());
                    }
                }
            }
        }
        
        // Also respect standard gitignore patterns
        ignored_patterns.insert(".git/**".to_string());
        ignored_patterns.insert("target/**".to_string());
        
        Self {
            kb_root: kb_root_path,
            ignored_patterns,
        }
    }
    
    pub fn should_ignore(&self, path: &Path) -> bool {
        // Check if path matches any ignored patterns
        for pattern in &self.ignored_patterns {
            if path.to_string_lossy().contains(pattern) {
                return true;
            }
        }
        false
    }
    
    pub async fn scan_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        
        for entry in WalkDir::new(&self.kb_root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && !self.should_ignore(path) {
                // Only include markdown files for now
                if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    files.push(path.to_path_buf());
                }
            }
        }
        
        files
    }
    
    pub async fn read_file(&self, path: &Path) -> Option<String> {
        fs::read_to_string(path).await.ok()
    }
    
    pub async fn write_file(&self, path: &Path, content: &str) -> Result<(), std::io::Error> {
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(path, content).await
    }
    
    pub async fn get_file_mod_time(&self, path: &Path) -> Option<std::time::SystemTime> {
        fs::metadata(path).await.ok().and_then(|m| m.modified().ok())
    }
}