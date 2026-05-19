use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::fs;
use walkdir::WalkDir;

struct IgnorePattern {
    /// If Some, the path must start with this prefix (directory match).
    dir_prefix: Option<String>,
    /// If Some, compile glob::Pattern for filename/path matching.
    glob_pattern: Option<glob::Pattern>,
    /// Original text for display.
    #[allow(dead_code)]
    raw: String,
}

impl IgnorePattern {
    fn from_string(raw: String) -> Self {
        // Directory pattern: ends with "/" or "/**"
        if raw.ends_with("/**") {
            let prefix = raw[..raw.len() - 3].to_string();
            return Self {
                dir_prefix: Some(prefix),
                glob_pattern: None,
                raw,
            };
        }
        if raw.ends_with('/') {
            let prefix = raw[..raw.len() - 1].to_string();
            return Self {
                dir_prefix: Some(prefix),
                glob_pattern: None,
                raw,
            };
        }
        // Try to compile as glob pattern
        if let Ok(pat) = glob::Pattern::new(&raw) {
            Self {
                dir_prefix: None,
                glob_pattern: Some(pat),
                raw,
            }
        } else {
            // Fallback: substring match (shouldn't happen with valid glob)
            Self {
                dir_prefix: None,
                glob_pattern: None,
                raw,
            }
        }
    }

    fn matches(&self, relative: &str) -> bool {
        if let Some(ref prefix) = self.dir_prefix {
            if relative == prefix || relative.starts_with(&format!("{}/", prefix)) {
                return true;
            }
            // Also check if any path component matches the prefix (subdirectory patterns)
            for component in std::path::Path::new(relative).components() {
                if let Some(s) = component.as_os_str().to_str() {
                    if s == prefix {
                        return true;
                    }
                }
            }
            return false;
        }
        if let Some(ref pat) = self.glob_pattern {
            // Match against the full relative path
            if pat.matches(relative) {
                return true;
            }
            // Also try matching just the filename component
            if let Some(name) = std::path::Path::new(relative)
                .file_name()
                .and_then(|n| n.to_str())
            {
                if pat.matches(name) {
                    return true;
                }
            }
        }
        false
    }
}

pub struct VaultState {
    pub kb_root: PathBuf,
    ignored_patterns: Vec<IgnorePattern>,
}

impl VaultState {
    pub async fn new(kb_root: &str) -> Self {
        let kb_root_path = PathBuf::from(kb_root);
        let mut ignored_patterns: Vec<IgnorePattern> = Vec::new();

        // Load .knowledge-loom-ignore if exists
        let loomignore_path = kb_root_path.join(".knowledge-loom-ignore");
        if loomignore_path.exists() {
            if let Ok(content) = fs::read_to_string(&loomignore_path).await {
                for line in content.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
                        ignored_patterns.push(IgnorePattern::from_string(line.to_string()));
                    }
                }
            }
        }

        // Default ignored patterns
        ignored_patterns.push(IgnorePattern::from_string(".git/**".to_string()));
        ignored_patterns.push(IgnorePattern::from_string("target/**".to_string()));
        ignored_patterns.push(IgnorePattern::from_string(".claude/**".to_string()));

        Self {
            kb_root: kb_root_path,
            ignored_patterns,
        }
    }

    fn relative_path(&self, path: &Path) -> String {
        path.strip_prefix(&self.kb_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
    }

    pub fn should_ignore(&self, path: &Path) -> bool {
        let relative = self.relative_path(path);
        for pattern in &self.ignored_patterns {
            if pattern.matches(&relative) {
                return true;
            }
        }
        false
    }

    pub async fn scan_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let mut seen_canonical: HashSet<PathBuf> = HashSet::new();
        let mut ignored_count = 0;

        for entry in WalkDir::new(&self.kb_root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if self.should_ignore(path) {
                    ignored_count += 1;
                } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    // Canonicalize to dedup symlinks
                    let canonical =
                        std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
                    if seen_canonical.insert(canonical) {
                        files.push(path.to_path_buf());
                    }
                }
            }
        }

        if ignored_count > 0 {
            eprintln!(
                "  ignored {} files via .knowledge-loom-ignore",
                ignored_count
            );
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

    #[allow(dead_code)]
    pub async fn get_file_mod_time(&self, path: &Path) -> Option<std::time::SystemTime> {
        fs::metadata(path)
            .await
            .ok()
            .and_then(|m| m.modified().ok())
    }
}
