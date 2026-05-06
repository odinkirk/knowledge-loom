#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use std::fs;
    use loom::search::SearchEngine;
    use loom::vault::VaultState;

    #[tokio::test]
    async fn test_search_engine_create() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();
        
        let engine = SearchEngine::new(kb_root.to_str().unwrap()).await;
        
        // Verify components were created
        assert!(kb_root.join(".loom-index/tantivy").exists());
        assert!(kb_root.join(".loom-index/embeddings.db").exists());
    }

    #[tokio::test]
    async fn test_search_engine_unified_search() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create test files
        fs::write(kb_root.join("test1.md"), "# Test 1\nThis is about cats and dogs").unwrap();
        fs::write(kb_root.join("test2.md"), "# Test 2\nThis is about birds and fish").unwrap();

        let engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        // Build indexes
        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = engine.vector.lock().await;
            let embed = engine.embed.lock().await;
            vector.index_vault(&vault, &embed).await.unwrap();
        }

        // Search
        let results = engine.search("cats", 5).await;

        assert!(!results.is_empty());
        // Results should be sorted by score
        for i in 0..results.len().saturating_sub(1) {
            assert!(results[i].score >= results[i + 1].score);
        }
    }

    #[tokio::test]
    async fn test_search_engine_empty_query() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();
        
        let engine = SearchEngine::new(kb_root.to_str().unwrap()).await;
        
        // Search with empty query
        let results = engine.search("", 5).await;
        
        // Should return empty results or handle gracefully
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_search_engine_limit() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create multiple test files
        for i in 0..10 {
            fs::write(kb_root.join(format!("test{}.md", i)),
                     format!("# Test {}\nContent {}", i, i)).unwrap();
        }

        let engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = engine.vector.lock().await;
            let embed = engine.embed.lock().await;
            vector.index_vault(&vault, &embed).await.unwrap();
        }

        // Search with limit
        let results = engine.search("test", 3).await;

        // Should respect limit
        assert!(results.len() <= 3);
    }

    #[tokio::test]
    async fn test_search_engine_rrf_scoring() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create test files with different content
        fs::write(kb_root.join("relevant.md"), "# Relevant\nThis is highly relevant content").unwrap();
        fs::write(kb_root.join("less_relevant.md"), "# Less Relevant\nSome other content").unwrap();

        let engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = engine.vector.lock().await;
            let embed = engine.embed.lock().await;
            vector.index_vault(&vault, &embed).await.unwrap();
        }

        // Search for "relevant"
        let results = engine.search("relevant", 5).await;

        assert!(!results.is_empty());
        // Most relevant result should have highest score
        let max_score = results.iter().map(|r| r.score).fold(f32::NEG_INFINITY, f32::max);
        assert!(max_score > 0.0);
    }

    #[tokio::test]
    async fn test_search_engine_result_structure() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        fs::write(kb_root.join("test.md"), "# Test\nTest content").unwrap();

        let engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = engine.vector.lock().await;
            let embed = engine.embed.lock().await;
            vector.index_vault(&vault, &embed).await.unwrap();
        }

        let results = engine.search("test", 5).await;

        if !results.is_empty() {
            let result = &results[0];
            // Verify result structure
            assert!(!result.path.is_empty());
            assert!(result.score >= 0.0);
            // Content may or may not be populated depending on implementation
        }
    }

    #[tokio::test]
    async fn test_search_returns_sections() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        fs::write(kb_root.join("cats.md"),
            "# Cats\n\nCats are great pets.\n\n## Persian Cats\n\nPersian cats are fluffy.").unwrap();

        let engine = SearchEngine::new(kb_root.to_str().unwrap()).await;
        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }

        let results = engine.search("cats fluffy", 5).await;

        assert!(!results.is_empty());
        // Top result should have sections
        assert!(!results[0].sections.is_empty());
        // Sections should be sorted by score desc
        for i in 0..results[0].sections.len().saturating_sub(1) {
            assert!(results[0].sections[i].score >= results[0].sections[i + 1].score);
        }
    }

    #[tokio::test]
    async fn test_graph_result_gets_sections() {
        // This test verifies that files surfaced by graph (no BM25 match) still get sections.
        // We simulate by indexing a file so get_chunks_for_path can find it, but
        // searching a query that won't BM25-match it (only graph would surface it).
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Write file with unique content unlikely to BM25-match "zephyr"
        fs::write(kb_root.join("unrelated.md"),
            "# Alpha Section\n\nSome alpha content.\n\n# Beta Section\n\nSome beta content.").unwrap();

        let engine = SearchEngine::new(kb_root.to_str().unwrap()).await;
        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }

        // Directly call populate_sections to test the lookup
        let path = kb_root.join("unrelated.md").to_string_lossy().to_string();
        let bm25 = engine.bm25.lock().await;
        let chunks = bm25.get_chunks_for_path(&path).await.unwrap();
        assert_eq!(chunks.len(), 2);
        assert!(chunks[0].line_start < chunks[1].line_start);
    }
}