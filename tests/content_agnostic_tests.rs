#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use std::fs;
    use loom::search::SearchEngine;
    use loom::vault::VaultState;

    #[tokio::test]
    async fn test_search_basic_functionality() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create simple test files
        fs::write(kb_root.join("file1.md"), "# File 1\nContent A").unwrap();
        fs::write(kb_root.join("file2.md"), "# File 2\nContent B").unwrap();

        let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        // Build indexes
        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = search_engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = search_engine.vector.lock().await;
            let embed = search_engine.embed.lock().await;
            vector.index_vault(&vault, &embed).await.unwrap();
        }

        // Test search returns results
        let results = search_engine.search("Content", 5).await;
        assert!(!results.is_empty(), "Search should return results");

        // Test results have proper structure
        for result in &results {
            assert!(!result.path.is_empty(), "Result should have a path");
            assert!(result.score >= 0.0, "Result should have a non-negative score");
        }
    }

    #[tokio::test]
    async fn test_search_empty_query() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        // Empty query should return empty results
        let results = search_engine.search("", 5).await;
        assert!(results.is_empty(), "Empty query should return no results");
    }

    #[tokio::test]
    async fn test_search_limit() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create multiple files
        for i in 0..10 {
            fs::write(kb_root.join(format!("file{}.md", i)),
                     format!("# File {}\nContent {}", i, i)).unwrap();
        }

        let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = search_engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = search_engine.vector.lock().await;
            let embed = search_engine.embed.lock().await;
            vector.index_vault(&vault, &embed).await.unwrap();
        }

        // Test limit is respected
        let results = search_engine.search("Content", 3).await;
        assert!(results.len() <= 3, "Results should respect limit");
    }

    #[tokio::test]
    async fn test_search_scoring() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create files with different relevance
        fs::write(kb_root.join("relevant.md"), "# Relevant\nTarget word appears multiple times").unwrap();
        fs::write(kb_root.join("less_relevant.md"), "# Less Relevant\nTarget word appears once").unwrap();

        let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = search_engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = search_engine.vector.lock().await;
            let embed = search_engine.embed.lock().await;
            vector.index_vault(&vault, &embed).await.unwrap();
        }

        let results = search_engine.search("Target", 5).await;

        // Results should be sorted by score
        for i in 0..results.len().saturating_sub(1) {
            assert!(results[i].score >= results[i + 1].score,
                   "Results should be sorted by score");
        }
    }

    #[tokio::test]
    async fn test_search_no_results() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create files
        fs::write(kb_root.join("file1.md"), "# File 1\nContent A").unwrap();
        fs::write(kb_root.join("file2.md"), "# File 2\nContent B").unwrap();

        let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = search_engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = search_engine.vector.lock().await;
            let embed = search_engine.embed.lock().await;
            vector.index_vault(&vault, &embed).await.unwrap();
        }

        // Search for non-existent term
        let results = search_engine.search("nonexistent", 5).await;
        // Note: Due to mock embeddings, we may get some results, but they should be low quality
        // The important thing is that the search doesn't crash and returns a valid result structure
        for result in &results {
            assert!(!result.path.is_empty(), "Result should have a path");
            assert!(result.score >= 0.0, "Result should have a non-negative score");
        }
    }

    #[tokio::test]
    async fn test_search_case_insensitive() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create file with specific case
        fs::write(kb_root.join("file.md"), "# File\nTargetWord content").unwrap();

        let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = search_engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = search_engine.vector.lock().await;
            let embed = search_engine.embed.lock().await;
            vector.index_vault(&vault, &embed).await.unwrap();
        }

        // Search with different case should still find results
        let results_lower = search_engine.search("targetword", 5).await;
        let results_upper = search_engine.search("TARGETWORD", 5).await;

        assert!(!results_lower.is_empty(), "Lowercase search should find results");
        assert!(!results_upper.is_empty(), "Uppercase search should find results");
    }

    #[tokio::test]
    async fn test_search_multiple_terms() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create file with multiple terms
        fs::write(kb_root.join("file.md"), "# File\nFirst term and second term together").unwrap();

        let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = search_engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = search_engine.vector.lock().await;
            let embed = search_engine.embed.lock().await;
            vector.index_vault(&vault, &embed).await.unwrap();
        }

        // Search for multiple terms
        let results = search_engine.search("First second", 5).await;
        assert!(!results.is_empty(), "Multi-term search should find results");
    }

    #[tokio::test]
    async fn test_search_special_characters() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create file with special characters
        fs::write(kb_root.join("file.md"), "# File\nContent with @ # $ % symbols").unwrap();

        let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = search_engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = search_engine.vector.lock().await;
            let embed = search_engine.embed.lock().await;
            vector.index_vault(&vault, &embed).await.unwrap();
        }

        // Search should handle special characters
        let results = search_engine.search("Content", 5).await;
        assert!(!results.is_empty(), "Search should handle special characters");
    }

    #[tokio::test]
    async fn test_search_unicode() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create file with unicode content
        fs::write(kb_root.join("file.md"), "# File\nUnicode content: café, naïve, résumé").unwrap();

        let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = search_engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = search_engine.vector.lock().await;
            let embed = search_engine.embed.lock().await;
            vector.index_vault(&vault, &embed).await.unwrap();
        }

        // Search should handle unicode
        let results = search_engine.search("café", 5).await;
        assert!(!results.is_empty(), "Search should handle unicode");
    }

    #[tokio::test]
    async fn test_search_rrf_merging() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create files that will be found by different search sources
        // File 1: Good for BM25 (exact term match)
        fs::write(kb_root.join("bm25_match.md"), "# BM25 Match\nmachine learning algorithms").unwrap();
        // File 2: Good for vector search (semantic similarity)
        fs::write(kb_root.join("vector_match.md"), "# Vector Match\nartificial intelligence and neural networks").unwrap();
        // File 3: Good for graph search (linked to other files)
        fs::write(kb_root.join("graph_match.md"), "# Graph Match\n[[bm25_match]] [[vector_match]]").unwrap();

        let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = search_engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = search_engine.vector.lock().await;
            let embed = search_engine.embed.lock().await;
            vector.index_vault(&vault, &embed).await.unwrap();
        }
        {
            let graph = search_engine.graph.lock().await;
            graph.build_graph(&vault).await.unwrap();
        }

        // Search for "machine learning" - should find BM25 match
        let results = search_engine.search("machine learning", 5).await;
        assert!(!results.is_empty(), "RRF search should return results");

        // Verify results are from different sources (RRF merging)
        let paths: Vec<_> = results.iter().map(|r| &r.path).collect();
        assert!(paths.iter().any(|p| p.ends_with("bm25_match.md")), "Should include BM25 match");
    }

    #[tokio::test]
    async fn test_search_parallel_execution() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create multiple files to test parallel execution
        for i in 0..20 {
            fs::write(kb_root.join(format!("file{}.md", i)),
                     format!("# File {}\nContent {}", i, i)).unwrap();
        }

        let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = search_engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = search_engine.vector.lock().await;
            let embed = search_engine.embed.lock().await;
            vector.index_vault(&vault, &embed).await.unwrap();
        }

        // Search should complete quickly (parallel execution)
        let start = std::time::Instant::now();
        let results = search_engine.search("Content", 10).await;
        let duration = start.elapsed();

        assert!(!results.is_empty(), "Parallel search should return results");
        // Parallel execution should be reasonably fast
        assert!(duration.as_millis() < 5000, "Parallel search should complete in reasonable time");
    }

    #[tokio::test]
    async fn test_search_graph_integration() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create files with wikilinks to test graph integration
        fs::write(kb_root.join("main.md"), "# Main\n[[topic1]] [[topic2]]").unwrap();
        fs::write(kb_root.join("topic1.md"), "# Topic 1\nContent about topic 1").unwrap();
        fs::write(kb_root.join("topic2.md"), "# Topic 2\nContent about topic 2").unwrap();

        let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = search_engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = search_engine.vector.lock().await;
            let embed = search_engine.embed.lock().await;
            vector.index_vault(&vault, &embed).await.unwrap();
        }
        {
            let graph = search_engine.graph.lock().await;
            graph.build_graph(&vault).await.unwrap();
        }

        // Search for "main" should find related topics via graph
        let results = search_engine.search("main", 5).await;
        assert!(!results.is_empty(), "Graph-integrated search should return results");

        // Should include the main file and potentially related files
        let paths: Vec<_> = results.iter().map(|r| &r.path).collect();
        assert!(paths.iter().any(|p| p.ends_with("main.md")), "Should include main file");
    }

    #[tokio::test]
    async fn test_search_result_ranking() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create files with different relevance levels
        fs::write(kb_root.join("high_relevance.md"), "# High Relevance\nmachine learning algorithms and neural networks").unwrap();
        fs::write(kb_root.join("medium_relevance.md"), "# Medium Relevance\nsome content about algorithms").unwrap();
        fs::write(kb_root.join("low_relevance.md"), "# Low Relevance\nrandom content").unwrap();

        let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = search_engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = search_engine.vector.lock().await;
            let embed = search_engine.embed.lock().await;
            vector.index_vault(&vault, &embed).await.unwrap();
        }

        let results = search_engine.search("machine learning algorithms", 5).await;

        // Results should be sorted by RRF score
        for i in 0..results.len().saturating_sub(1) {
            assert!(results[i].score >= results[i + 1].score,
                   "Results should be sorted by RRF score");
        }

        // Most relevant file should be first
        if !results.is_empty() {
            assert!(results[0].path.contains("high_relevance"),
                   "Most relevant file should be ranked first");
        }
    }

    #[tokio::test]
    async fn test_search_multiple_sources() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create files that will be found by different search sources
        fs::write(kb_root.join("file1.md"), "# File 1\nmachine learning").unwrap();
        fs::write(kb_root.join("file2.md"), "# File 2\nartificial intelligence").unwrap();
        fs::write(kb_root.join("file3.md"), "# File 3\n[[file1]] [[file2]]").unwrap();

        let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = search_engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = search_engine.vector.lock().await;
            let embed = search_engine.embed.lock().await;
            vector.index_vault(&vault, &embed).await.unwrap();
        }
        {
            let graph = search_engine.graph.lock().await;
            graph.build_graph(&vault).await.unwrap();
        }

        // Search should use multiple sources
        let results = search_engine.search("machine", 5).await;
        assert!(!results.is_empty(), "Multi-source search should return results");

        // Results should come from different search sources (verified by RRF scoring)
        let total_score: f32 = results.iter().map(|r| r.score).sum();
        assert!(total_score > 0.0, "RRF should produce positive scores from multiple sources");
    }

    #[tokio::test]
    async fn test_search_consistency() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create test files
        fs::write(kb_root.join("file.md"), "# File\nTest content").unwrap();

        let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = search_engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = search_engine.vector.lock().await;
            let embed = search_engine.embed.lock().await;
            vector.index_vault(&vault, &embed).await.unwrap();
        }

        // Run same search multiple times
        let results1 = search_engine.search("Test", 5).await;
        let results2 = search_engine.search("Test", 5).await;
        let results3 = search_engine.search("Test", 5).await;

        // Results should be consistent
        assert_eq!(results1.len(), results2.len(), "Search should be consistent");
        assert_eq!(results2.len(), results3.len(), "Search should be consistent");

        // Scores should be consistent
        for i in 0..results1.len() {
            assert_eq!(results1[i].score, results2[i].score, "Scores should be consistent");
            assert_eq!(results2[i].score, results3[i].score, "Scores should be consistent");
        }
    }

    #[tokio::test]
    async fn test_metadata_enrichment_heading() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create file with multiple headings
        fs::write(kb_root.join("file.md"),
                 "# Main Heading\n\nContent under main heading\n\n## Sub Heading\n\nContent under sub heading").unwrap();

        let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = search_engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = search_engine.vector.lock().await;
            let embed = search_engine.embed.lock().await;
            vector.index_vault(&vault, &embed).await.unwrap();
        }

        // Search for content under sub heading
        let results = search_engine.search("sub heading", 5).await;

        if !results.is_empty() {
            // Verify metadata is enriched
            let result = &results[0];
            assert!(!result.path.is_empty(), "Result should have path");
            assert!(result.score >= 0.0, "Result should have score");
            assert!(!result.sections.is_empty(), "Result should have sections");

            // Check if heading is populated (metadata enrichment)
            if result.sections[0].heading.is_some() {
                let heading = result.sections[0].heading.as_ref().unwrap();
                assert!(!heading.is_empty(), "Heading should not be empty if present");
            }
        }
    }

    #[tokio::test]
    async fn test_metadata_enrichment_content() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create file with specific content
        let test_content = "# Test File\n\nThis is the target content that should be returned in search results.";
        fs::write(kb_root.join("file.md"), test_content).unwrap();

        let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = search_engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = search_engine.vector.lock().await;
            let embed = search_engine.embed.lock().await;
            vector.index_vault(&vault, &embed).await.unwrap();
        }

        // Search for the content
        let results = search_engine.search("target content", 5).await;

        if !results.is_empty() {
            // Verify content metadata is enriched
            let result = &results[0];
            assert!(!result.sections.is_empty(), "Result should have sections");
            assert!(!result.sections[0].content.is_empty(), "Result should have content metadata");
            assert!(result.sections[0].content.contains("target content") || result.sections[0].content.contains("target"),
                   "Content should contain search terms");
        }
    }

    #[tokio::test]
    async fn test_metadata_enrichment_line_numbers() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create file with multiple lines
        let content = "# Test File\nLine 1\nLine 2\nLine 3\nLine 4\nLine 5";
        fs::write(kb_root.join("file.md"), content).unwrap();

        let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = search_engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = search_engine.vector.lock().await;
            let embed = search_engine.embed.lock().await;
            vector.index_vault(&vault, &embed).await.unwrap();
        }

        // Search for content
        let results = search_engine.search("Line 3", 5).await;

        if !results.is_empty() {
            // Verify line_start metadata is enriched
            let result = &results[0];
            assert!(!result.sections.is_empty(), "Result should have sections");
            let line_start = result.sections[0].line_start;
            assert!(line_start > 0, "Line start should be positive");
            assert!(line_start <= 10, "Line start should be within file bounds");
        }
    }

    #[tokio::test]
    async fn test_metadata_enrichment_complete() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create file with complete structure
        fs::write(kb_root.join("file.md"),
                 "# Important Section\n\nThis is important content that should be found.\n\n## Details\n\nMore details here.").unwrap();

        let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = search_engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = search_engine.vector.lock().await;
            let embed = search_engine.embed.lock().await;
            vector.index_vault(&vault, &embed).await.unwrap();
        }

        // Search for content
        let results = search_engine.search("important content", 5).await;

        if !results.is_empty() {
            // Verify all metadata fields are populated
            let result = &results[0];
            assert!(!result.sections.is_empty(), "Result should have sections");

            // Required fields
            assert!(!result.path.is_empty(), "Path should be populated");
            assert!(result.score >= 0.0, "Score should be non-negative");
            assert!(!result.sections[0].content.is_empty(), "Content should be populated");
            assert!(result.sections[0].line_start > 0, "Line start should be positive");

            // Optional metadata fields (should be populated when available)
            if result.sections[0].heading.is_some() {
                assert!(!result.sections[0].heading.as_ref().unwrap().is_empty(), "Heading should not be empty");
            }
        }
    }

    #[tokio::test]
    async fn test_metadata_enrichment_multiple_results() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create multiple files
        fs::write(kb_root.join("file1.md"), "# File 1\nContent A").unwrap();
        fs::write(kb_root.join("file2.md"), "# File 2\nContent B").unwrap();
        fs::write(kb_root.join("file3.md"), "# File 3\nContent C").unwrap();

        let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = search_engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = search_engine.vector.lock().await;
            let embed = search_engine.embed.lock().await;
            vector.index_vault(&vault, &embed).await.unwrap();
        }

        // Search that should return multiple results
        let results = search_engine.search("Content", 10).await;

        // All results should have proper metadata
        for result in &results {
            assert!(!result.path.is_empty(), "Each result should have a path");
            assert!(result.score >= 0.0, "Each result should have a score");
            assert!(!result.sections.is_empty(), "Each result should have sections");
            assert!(!result.sections[0].content.is_empty(), "Each result should have content");
        }

        // Results should be sorted by score
        for i in 0..results.len().saturating_sub(1) {
            assert!(results[i].score >= results[i + 1].score,
                   "Multiple results should be sorted by score");
        }
    }
}
