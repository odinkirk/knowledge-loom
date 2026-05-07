#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::TempDir;
    use knowledge_loom::search::SearchEngine;
    use knowledge_loom::vault::VaultState;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use knowledge_loom::bm25::BM25Index;
    use knowledge_loom::index::VectorIndex;
    use knowledge_loom::embed::EmbedProviderEnum;
    use knowledge_loom::graph::GraphState;
    use knowledge_loom::brainjar::BrainJarWrapper;
    use knowledge_loom::edits::make_edit_manager_for_test;

    #[tokio::test]
    async fn test_search_engine_create() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();
        
        let _engine = SearchEngine::new(kb_root.to_str().unwrap()).await;
        
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
    async fn test_search_file_basic_functionality() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create a test file with distinct content
        let test_content = "# Introduction\n\nThis is about machine learning and AI.\n\n## Deep Learning\n\nDeep learning is a subset of ML.";
        fs::write(kb_root.join("test.md"), test_content).unwrap();

        // Create another file to ensure we're searching within the correct file
        let other_content = "# Other Topic\n\nThis is about cooking and recipes.";
        fs::write(kb_root.join("other.md"), other_content).unwrap();

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

        // Search within the test file for "machine learning"
        let results = engine.bm25.lock().await.search_file(
            "test.md", 
            "machine learning", 
            5
        ).await.unwrap();

        assert!(!results.is_empty());
        // All results should be from test.md
        for (score, chunk) in results {
            assert_eq!(chunk.path, "test.md");
            assert!(chunk.content.contains("machine learning"));
            assert!(score > 0.0);
        }

        // Search within the test file for "cooking" should return empty
        let results = engine.bm25.lock().await.search_file(
            "test.md", 
            "cooking", 
            5
        ).await.unwrap();

        assert!(results.is_empty(), "Should not find cooking in test.md");

        // Search within other.md for "cooking" should return results
        let results = engine.bm25.lock().await.search_file(
            "other.md", 
            "cooking", 
            5
        ).await.unwrap();

        assert!(!results.is_empty());
        for (score, chunk) in results {
            assert_eq!(chunk.path, "other.md");
            assert!(chunk.content.contains("cooking"));
            assert!(score > 0.0);
        }
    }

    #[tokio::test]
    async fn test_search_file_limit() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create a test file with multiple mentions of the same term
        let test_content = "# Test\n\nThis is a test about testing.\n\nMore testing content here.\n\nEven more testing information.";
        fs::write(kb_root.join("test.md"), test_content).unwrap();

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

        // Search with limit 2
        let results = engine.bm25.lock().await.search_file(
            "test.md", 
            "testing", 
            2
        ).await.unwrap();

        // Should respect limit
        assert!(results.len() <= 2);
    }

    #[tokio::test]
    async fn test_search_file_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

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

        // Search in a file that doesn't exist
        let results = engine.bm25.lock().await.search_file(
            "nonexistent.md", 
            "test", 
            5
        ).await.unwrap();

        // Should return empty results for nonexistent file
        assert!(results.is_empty());
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

    #[tokio::test]
    async fn test_search_engine_from_components() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();
        
        // Create mock components
        let bm25 = Arc::new(Mutex::new(BM25Index::new(kb_root.to_str().unwrap()).await));
        let vector = Arc::new(Mutex::new(VectorIndex::new(kb_root.to_str().unwrap()).await));
        let embed = Arc::new(Mutex::new(EmbedProviderEnum::new(kb_root.to_str().unwrap()).await));
        let graph = Arc::new(Mutex::new(GraphState::new(kb_root.to_str().unwrap()).await));
        let brainjar = Arc::new(Mutex::new(BrainJarWrapper::new(None)));
        
        // Create engine from components
        let engine = SearchEngine::from_components(bm25.clone(), vector.clone(), embed.clone(), graph.clone(), brainjar.clone());
        
        // Verify all components are present
        assert!(Arc::ptr_eq(&engine.bm25, &bm25));
        assert!(Arc::ptr_eq(&engine.vector, &vector));
        assert!(Arc::ptr_eq(&engine.embed, &embed));
        assert!(Arc::ptr_eq(&engine.graph, &graph));
        assert!(Arc::ptr_eq(&engine.brainjar, &brainjar));
    }

    #[tokio::test]
    async fn test_search_engine_search_with_empty_query_returns_empty() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();
        let engine = SearchEngine::new(kb_root.to_str().unwrap()).await;
        
        // Search with empty string
        let results = engine.search("", 5).await;
        assert!(results.is_empty());
        
        // Search with only whitespace
        let results = engine.search("   ", 5).await;
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_grep_regex_anchored_start() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("note.md"), "foo bar\nbaz foo\nfoo123").unwrap();
        let em = make_edit_manager_for_test(tmp.path().to_str().unwrap()).await;

        // Only lines starting with "foo"
        let results = em.grep("^foo").await;
        assert_eq!(results.len(), 2, "expected 'foo bar' and 'foo123', got {results:?}");
    }

    #[tokio::test]
    async fn test_grep_regex_digit_sequence() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("note.md"), "foo bar\nbaz qux\nfoo123").unwrap();
        let em = make_edit_manager_for_test(tmp.path().to_str().unwrap()).await;

        let results = em.grep(r"\d+").await;
        assert_eq!(results.len(), 1, "expected only 'foo123', got {results:?}");
    }

    #[tokio::test]
    async fn test_grep_invalid_regex_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("note.md"), "anything").unwrap();
        let em = make_edit_manager_for_test(tmp.path().to_str().unwrap()).await;

        // Invalid regex — should not panic, should return empty
        let results = em.grep("[invalid").await;
        assert!(results.is_empty(), "invalid regex should return empty vec");
    }
}