use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct SectionPreview {
    pub heading: String,
    pub line_start: usize,
    pub line_end: usize,
    pub current: String,
    pub proposed: String,
}

#[allow(dead_code)]
pub struct EditManager {
    pub kb_root: PathBuf,
    pub vault_state: Arc<Mutex<crate::vault::VaultState>>,
    pub bm25_index: Arc<Mutex<crate::bm25::BM25Index>>,
    pub embed_provider: Arc<Mutex<crate::embed::EmbedProviderEnum>>,
    pub vector_index: Arc<Mutex<crate::index::VectorIndex>>,
    pub graph_state: Arc<Mutex<crate::graph::GraphState>>,
}

impl EditManager {
    pub fn new(
        kb_root: String,
        vault_state: Arc<Mutex<crate::vault::VaultState>>,
        bm25_index: Arc<Mutex<crate::bm25::BM25Index>>,
        embed_provider: Arc<Mutex<crate::embed::EmbedProviderEnum>>,
        vector_index: Arc<Mutex<crate::index::VectorIndex>>,
        graph_state: Arc<Mutex<crate::graph::GraphState>>,
    ) -> Self {
        Self {
            kb_root: PathBuf::from(kb_root),
            vault_state,
            bm25_index,
            embed_provider,
            vector_index,
            graph_state,
        }
    }

    pub async fn list_files(&self) -> Vec<String> {
        let vault_lock = self.vault_state.lock().await;
        let files = vault_lock.scan_files().await;
        files
            .into_iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect()
    }

    pub async fn get_outline(
        &self,
        file_path: &Path,
    ) -> Result<Vec<(usize, String)>, std::io::Error> {
        let vault_lock = self.vault_state.lock().await;
        if let Some(content) = vault_lock.read_file(file_path).await {
            let mut outline = Vec::new();
            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with('#') {
                    let heading_text = trimmed.trim_start_matches('#').trim();
                    if !heading_text.is_empty() {
                        outline.push((line_num + 1, heading_text.to_string()));
                    }
                }
            }
            Ok(outline)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found",
            ))
        }
    }

    pub async fn grep(&self, pattern: &str) -> Vec<(String, usize, String)> {
        let re = match regex::Regex::new(pattern) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };
        let vault_lock = self.vault_state.lock().await;
        let files = vault_lock.scan_files().await;
        let mut results = Vec::new();

        for file_path in files {
            if let Some(content) = vault_lock.read_file(&file_path).await {
                for (line_num, line) in content.lines().enumerate() {
                    if re.is_match(line) {
                        results.push((
                            file_path.to_string_lossy().to_string(),
                            line_num + 1,
                            line.to_string(),
                        ));
                    }
                }
            }
        }
        results
    }

    pub async fn read_section(
        &self,
        file_path: &Path,
        heading: &str,
    ) -> Result<Option<String>, std::io::Error> {
        let vault_lock = self.vault_state.lock().await;
        if let Some(content) = vault_lock.read_file(file_path).await {
            let lines: Vec<&str> = content.lines().collect();
            let mut in_section = false;
            let mut section_content = Vec::new();
            let mut heading_level = 0;

            for line in lines {
                if line.trim().starts_with('#') {
                    if in_section {
                        // Check if this is a heading of same or higher level
                        let current_level = line.chars().take_while(|c| *c == '#').count();
                        if current_level <= heading_level {
                            break;
                        }
                    }

                    let heading_text = line.trim().trim_start_matches('#').trim();
                    if !in_section && heading_text == heading {
                        in_section = true;
                        heading_level = line.chars().take_while(|c| *c == '#').count();
                        section_content.push(line);
                    }
                } else if in_section {
                    section_content.push(line);
                }
            }

            if in_section {
                Ok(Some(section_content.join("\n")))
            } else {
                Ok(None)
            }
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found",
            ))
        }
    }

    pub async fn apply_edit_preview(
        &self,
        file_path: &Path,
        heading: &str,
        proposed: &str,
    ) -> Result<Option<SectionPreview>, std::io::Error> {
        let vault_lock = self.vault_state.lock().await;
        let content = match vault_lock.read_file(file_path).await {
            Some(c) => c,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "File not found",
                ))
            }
        };

        let lines: Vec<&str> = content.lines().collect();
        let mut in_section = false;
        let mut heading_level = 0usize;
        let mut line_start = 0usize;
        let mut section_lines: Vec<&str> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if line.trim().starts_with('#') {
                if in_section {
                    let cur_level = line.chars().take_while(|c| *c == '#').count();
                    if cur_level <= heading_level {
                        break;
                    }
                }
                let heading_text = line.trim().trim_start_matches('#').trim();
                if !in_section && heading_text == heading {
                    in_section = true;
                    heading_level = line.chars().take_while(|c| *c == '#').count();
                    line_start = i + 1;
                    section_lines.push(line);
                }
            } else if in_section {
                section_lines.push(line);
            }
        }

        if !in_section {
            return Ok(None);
        }

        let line_end = line_start + section_lines.len().saturating_sub(1);
        Ok(Some(SectionPreview {
            heading: heading.to_string(),
            line_start,
            line_end,
            current: section_lines.join("\n"),
            proposed: proposed.to_string(),
        }))
    }

    pub async fn read_lines(
        &self,
        file_path: &Path,
        start: usize,
        end: usize,
    ) -> Result<Option<String>, std::io::Error> {
        let vault_lock = self.vault_state.lock().await;
        if let Some(content) = vault_lock.read_file(file_path).await {
            let lines: Vec<&str> = content.lines().collect();
            let start_idx = start.saturating_sub(1); // Convert to 0-indexed
            let end_idx = end.min(lines.len());

            if start_idx >= lines.len() {
                return Ok(None);
            }

            let selected_lines: Vec<&str> = lines[start_idx..end_idx].to_vec();
            Ok(Some(selected_lines.join("\n")))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found",
            ))
        }
    }

    async fn reindex_file(&self, path: &Path, content: &str) {
        {
            let mut bm25 = self.bm25_index.lock().await;
            let _ = bm25.index_file(path, content).await;
        }
        {
            let vector = self.vector_index.lock().await;
            let embed = self.embed_provider.lock().await;
            let _ = vector.index_file(path, content, &*embed).await;
        }
        {
            let graph = self.graph_state.lock().await;
            let _ = graph.update_file(path, content).await;
        }
    }

    pub async fn replace_lines(
        &self,
        file_path: &Path,
        start: usize,
        end: usize,
        new_content: &str,
    ) -> Result<(), std::io::Error> {
        let vault_lock = self.vault_state.lock().await;
        if let Some(content) = vault_lock.read_file(file_path).await {
            let lines: Vec<&str> = content.lines().collect();
            let start_idx = start.saturating_sub(1); // Convert to 0-indexed
            let end_idx = end.min(lines.len());

            if start_idx >= lines.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Start line out of range",
                ));
            }

            let mut new_lines = Vec::new();
            new_lines.extend_from_slice(&lines[..start_idx]);
            new_lines.push(new_content);
            new_lines.extend_from_slice(&lines[end_idx..]);

            let new_content = new_lines.join("\n");
            vault_lock.write_file(file_path, &new_content).await?;
        } // vault_lock dropped
        self.reindex_file(file_path, &new_content).await;
        Ok(())
    }

    pub async fn insert_after_heading(
        &self,
        file_path: &Path,
        heading: &str,
        content: &str,
    ) -> Result<(), std::io::Error> {
        let new_content;
        {
            let vault_lock = self.vault_state.lock().await;
            if let Some(file_content) = vault_lock.read_file(file_path).await {
                let lines: Vec<&str> = file_content.lines().collect();
                let mut new_lines = Vec::new();
                let mut inserted = false;

                for line in lines {
                    new_lines.push(line);

                    if !inserted && line.trim().starts_with('#') {
                        let heading_text = line.trim_start_matches('#').trim();
                        if heading_text == heading {
                            // Insert content after this heading
                            for content_line in content.lines() {
                                new_lines.push(content_line);
                            }
                            inserted = true;
                        }
                    }
                }

                if !inserted {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Heading not found",
                    ));
                }

                new_content = new_lines.join("\n");
                vault_lock.write_file(file_path, &new_content).await?;
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "File not found",
                ));
            }
        } // vault_lock dropped
        self.reindex_file(file_path, &new_content).await;
        Ok(())
    }

    pub async fn append_to_file(
        &self,
        file_path: &Path,
        content: &str,
    ) -> Result<(), std::io::Error> {
        let new_content;
        {
            let vault_lock = self.vault_state.lock().await;
            let existing_content = vault_lock.read_file(file_path).await.unwrap_or_default();

            let mut content_buf = existing_content;
            if !content_buf.is_empty() && !content_buf.ends_with('\n') {
                content_buf.push('\n');
            }
            content_buf.push_str(content);

            new_content = content_buf;
            vault_lock.write_file(file_path, &new_content).await?;
        } // vault_lock dropped
        self.reindex_file(file_path, &new_content).await;
        Ok(())
    }

    pub async fn create_note(&self, title: &str, content: &str) -> Result<PathBuf, std::io::Error> {
        // Convert title to filename
        let filename = title.replace(' ', "_").replace('/', "_") + ".md";
        let file_path = self.kb_root.join(&filename);

        {
            let vault_lock = self.vault_state.lock().await;
            vault_lock.write_file(&file_path, content).await?;
        } // vault_lock dropped
        self.reindex_file(&file_path, content).await;
        Ok(file_path)
    }

    pub async fn edit_note(
        &self,
        file_path: &Path,
        new_content: &str,
    ) -> Result<(), std::io::Error> {
        {
            let vault_lock = self.vault_state.lock().await;
            vault_lock.write_file(file_path, new_content).await?;
        } // vault_lock dropped
        self.reindex_file(file_path, new_content).await;
        Ok(())
    }

    pub async fn link_notes(&self, from_path: &Path, to_path: &Path) -> Result<(), std::io::Error> {
        let vault_lock = self.vault_state.lock().await;

        // Read the source file
        if let Some(mut content) = vault_lock.read_file(from_path).await {
            // Extract the target note title for the wikilink
            let to_title = to_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled");

            // Append wikilink at the end
            if !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(&format!("[[{}]]", to_title));

            vault_lock.write_file(from_path, &content).await?;

            // Update graph
            // TODO: Update graph state
        }

        Ok(())
    }

    pub async fn move_note(&self, from_path: &Path, to_path: &Path) -> Result<(), std::io::Error> {
        // Check if source exists
        if !from_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Source file not found",
            ));
        }

        // Create parent directory for destination if needed
        if let Some(parent) = to_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Move the file
        tokio::fs::rename(from_path, to_path).await?;

        // Update indexes
        // TODO: Update BM25, vector index, and graph with new path

        Ok(())
    }

    pub async fn delete_note(&self, file_path: &Path) -> Result<(), std::io::Error> {
        // Check if file exists
        if !file_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found",
            ));
        }

        // Delete the file
        tokio::fs::remove_file(file_path).await?;

        // Update indexes
        // TODO: Remove from BM25, vector index, and graph

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::sync::Mutex;

    pub async fn make_edit_manager(
        tmp: &TempDir,
        content: &str,
    ) -> (EditManager, std::path::PathBuf) {
        let root = tmp.path().to_str().unwrap().to_string();
        let vault = Arc::new(Mutex::new(crate::vault::VaultState::new(&root).await));
        let bm25 = Arc::new(Mutex::new(crate::bm25::BM25Index::new(&root).await));
        let embed = Arc::new(Mutex::new(
            crate::embed::EmbedProviderEnum::new(&root).await,
        ));
        let vector = Arc::new(Mutex::new(crate::index::VectorIndex::new(&root).await));
        let graph = Arc::new(Mutex::new(crate::graph::GraphState::new(&root).await));
        let em = EditManager::new(root, vault.clone(), bm25, embed, vector, graph);
        let file_path = tmp.path().join("note.md");
        std::fs::write(&file_path, content).unwrap();
        (em, file_path)
    }

    #[tokio::test]
    async fn test_apply_edit_preview_found() {
        let tmp = TempDir::new().unwrap();
        let content = "# Intro\n\nHello world\n\n# Summary\n\nFin\n";
        let (em, path) = make_edit_manager(&tmp, content).await;
        let preview = em
            .apply_edit_preview(&path, "Intro", "New intro content")
            .await
            .unwrap();
        assert!(preview.is_some());
        let p = preview.unwrap();
        assert_eq!(p.heading, "Intro");
        assert!(p.current.contains("Hello world"));
        assert_eq!(p.proposed, "New intro content");
    }

    #[tokio::test]
    async fn test_apply_edit_preview_not_found() {
        let tmp = TempDir::new().unwrap();
        let content = "# Intro\n\nHello\n";
        let (em, path) = make_edit_manager(&tmp, content).await;
        let preview = em.apply_edit_preview(&path, "Missing", "x").await.unwrap();
        assert!(preview.is_none());
    }
}
