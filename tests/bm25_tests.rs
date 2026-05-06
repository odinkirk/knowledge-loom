#[cfg(test)]
mod tests {
    
    use tempfile::TempDir;
    
    use loom::bm25::{BM25Index, extract_title, parse_chunks, Chunk, ChunkDoc};

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

        let test_path = kb_root.join("test.md");
        index.index_file(&test_path, "# Test Document\n\nThis is test content about testing").await
            .unwrap();

        {
            let mut writer = index.writer.lock().await;
            writer.commit().unwrap();
        }

        let results = index.search("test", 10).await.unwrap();
        assert!(!results.is_empty());
        assert!(results[0].0 > 0.0);
    }

    #[tokio::test]
    async fn test_bm25_remove_document() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        let mut index = BM25Index::new(kb_root.to_str().unwrap()).await;

        let test_path = kb_root.join("test.md");
        index.index_file(&test_path, "# Test\n\nTest content").await.unwrap();

        {
            let mut writer = index.writer.lock().await;
            writer.commit().unwrap();
        }

        let results_before = index.search("test", 10).await.unwrap();
        assert!(!results_before.is_empty());

        index.remove_document(&test_path).await.unwrap();

        let results_after = index.search("test", 10).await.unwrap();
        assert!(results_after.is_empty());
    }

    #[tokio::test]
    async fn test_bm25_search_and_retrieve() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        let mut index = BM25Index::new(kb_root.to_str().unwrap()).await;

        let test_path = kb_root.join("test.md");
        let content = "# Test Title\n\nThis is test content";
        index.index_file(&test_path, content).await.unwrap();

        let mut writer = index.writer.lock().await;
        writer.commit().unwrap();

        let results = index.search_and_retrieve("test", 10).await.unwrap();
        assert!(!results.is_empty());
        let (_score, chunk) = &results[0];
        assert!(chunk.heading.as_deref().unwrap_or("").contains("Test Title"));
        assert!(chunk.content.contains("test content"));
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

    #[test]
    fn test_parse_chunks_two_sections() {
        let content = "# Introduction\n\nSome intro text here.\n\n## Background\n\nBackground details.";
        let chunks = parse_chunks(content);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].heading, Some("Introduction".to_string()));
        assert_eq!(chunks[0].line_start, 1);
        assert!(chunks[0].content.contains("intro text"));
        assert_eq!(chunks[1].heading, Some("Introduction > Background".to_string()));
        assert!(chunks[1].content.contains("Background details"));
    }

    #[test]
    fn test_parse_chunks_headingless_fallback() {
        let content = "Just some plain text\nwith no headings at all.";
        let chunks = parse_chunks(content);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].heading, None);
        assert_eq!(chunks[0].line_start, 1);
        assert!(chunks[0].content.contains("plain text"));
    }

    #[test]
    fn test_parse_chunks_empty_section_skipped() {
        let content = "# Heading With No Content\n\n# Second Heading\n\nActual content here.";
        let chunks = parse_chunks(content);
        // Empty section should be skipped; only second heading with content
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].heading, Some("Second Heading".to_string()));
    }

    #[test]
    fn test_parse_chunks_breadcrumb() {
        let content = "# Top\n\nTop content.\n\n## Sub\n\nSub content.\n\n### DeepSub\n\nDeep content.";
        let chunks = parse_chunks(content);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].heading, Some("Top".to_string()));
        assert_eq!(chunks[1].heading, Some("Top > Sub".to_string()));
        assert_eq!(chunks[2].heading, Some("Top > Sub > DeepSub".to_string()));
    }

    #[tokio::test]
    async fn test_index_file_returns_chunks() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        let mut index = BM25Index::new(kb_root.to_str().unwrap()).await;

        let path = kb_root.join("note.md");
        let content = "# Alpha\n\nAlpha content here.\n\n## Beta\n\nBeta content here.";
        index.index_file(&path, content).await.unwrap();

        {
            let mut writer = index.writer.lock().await;
            writer.commit().unwrap();
        }

        let results = index.search_and_retrieve("alpha", 10).await.unwrap();
        assert!(!results.is_empty());
        let (_, chunk) = &results[0];
        assert!(chunk.heading.as_deref().unwrap_or("").contains("Alpha"));
        assert!(chunk.content.contains("Alpha content"));
        assert!(chunk.line_start > 0);
    }

    #[tokio::test]
    async fn test_get_chunks_for_path() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        let mut index = BM25Index::new(kb_root.to_str().unwrap()).await;

        let path = kb_root.join("note.md");
        let content = "# Section A\n\nContent A.\n\n# Section B\n\nContent B.";
        index.index_file(&path, content).await.unwrap();

        {
            let mut writer = index.writer.lock().await;
            writer.commit().unwrap();
        }

        let chunks = index.get_chunks_for_path(path.to_str().unwrap()).await.unwrap();
        assert_eq!(chunks.len(), 2);
    }

    #[tokio::test]
    async fn test_index_file_replaces_on_reindex() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        let mut index = BM25Index::new(kb_root.to_str().unwrap()).await;
        let path = kb_root.join("note.md");

        index.index_file(&path, "# Old\n\nOld content.").await.unwrap();
        {
            let mut writer = index.writer.lock().await;
            writer.commit().unwrap();
        }

        index.index_file(&path, "# New\n\nNew content.").await.unwrap();
        {
            let mut writer = index.writer.lock().await;
            writer.commit().unwrap();
        }

        let chunks = index.get_chunks_for_path(path.to_str().unwrap()).await.unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].content.contains("New content"));
    }
}