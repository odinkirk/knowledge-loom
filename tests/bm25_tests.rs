#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    use loom::bm25::{BM25Index, extract_title};

    #[tokio::test]
    async fn test_bm25_create_index() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();
        
        let index = BM25Index::new(kb_root.to_str().unwrap()).await;
        
        assert!(index.index_path.exists());
    }

    #[tokio::test]
    async fn test_bm25_add_and_search() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();
        
        let mut index = BM25Index::new(kb_root.to_str().unwrap()).await;
        
        // Add documents
        let test_path = kb_root.join("test.md");
        index.add_document(&test_path, "Test Document", "This is test content about testing").await
            .unwrap();
        
        // Commit to make searchable
        let mut writer = index.writer.lock().await;
        writer.commit().unwrap();
        
        // Search
        let results = index.search("test", 10).await.unwrap();
        
        assert!(!results.is_empty());
        assert!(results[0].0 > 0.0); // Score should be positive
    }

    #[tokio::test]
    async fn test_bm25_remove_document() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();
        
        let mut index = BM25Index::new(kb_root.to_str().unwrap()).await;
        
        // Add document
        let test_path = kb_root.join("test.md");
        index.add_document(&test_path, "Test Document", "Test content").await
            .unwrap();
        
        // Commit to make it searchable
        {
            let mut writer = index.writer.lock().await;
            writer.commit().unwrap();
        }
        
        // Verify it exists
        let results_before = index.search("test", 10).await.unwrap();
        assert!(!results_before.is_empty());
        
        // Remove document
        index.remove_document(&test_path).await.unwrap();
        
        // Verify it's gone
        let results_after = index.search("test", 10).await.unwrap();
        assert!(results_after.is_empty());
    }

    #[tokio::test]
    async fn test_bm25_search_and_retrieve() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();
        
        let mut index = BM25Index::new(kb_root.to_str().unwrap()).await;
        
        // Add document
        let test_path = kb_root.join("test.md");
        let title = "Test Title";
        let content = "This is test content";
        index.add_document(&test_path, title, content).await.unwrap();
        
        let mut writer = index.writer.lock().await;
        writer.commit().unwrap();
        
        // Search and retrieve
        let results = index.search_and_retrieve("test", 10).await.unwrap();
        
        assert!(!results.is_empty());
        let (_score, doc) = &results[0];
        
        // Verify document content
        let mut found_title = false;
        let mut found_content = false;
        
        if let Some(title_field) = index.schema.get_field("title") {
            for value in doc.get_all(title_field) {
                if let tantivy::schema::Value::Str(s) = value {
                    if s == title {
                        found_title = true;
                    }
                }
            }
        }
        
        if let Some(content_field) = index.schema.get_field("content") {
            for value in doc.get_all(content_field) {
                if let tantivy::schema::Value::Str(s) = value {
                    if s == content {
                        found_content = true;
                    }
                }
            }
        }
        
        assert!(found_title);
        assert!(found_content);
    }

    #[test]
    fn test_extract_title() {
        let content = "# Main Title\n\nSome content\n\n## Subtitle\n\nMore content";
        let title = extract_title(content);
        
        assert_eq!(title, Some("Main Title".to_string()));
    }

    #[test]
    fn test_extract_title_no_heading() {
        let content = "Just some content without headings";
        let title = extract_title(content);
        
        assert_eq!(title, None);
    }

    #[test]
    fn test_extract_title_empty_heading() {
        let content = "# \n\nContent after empty heading";
        let title = extract_title(content);
        
        assert_eq!(title, None);
    }
}