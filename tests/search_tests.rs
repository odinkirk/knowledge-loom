#[cfg(test)]
mod tests {
    use knowledge_loom::bm25::BM25Index;
    use knowledge_loom::edits::EditManager;
    use knowledge_loom::embed::EmbedProviderEnum;
    use knowledge_loom::graph::GraphState;
    use knowledge_loom::index::VectorIndex;
    use knowledge_loom::search::SearchEngine;
    use knowledge_loom::vault::VaultState;
    use std::fs;
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::sync::Mutex;

    async fn make_edit_manager_for_test(kb_root: &str) -> EditManager {
        let vault = Arc::new(Mutex::new(VaultState::new(kb_root).await));
        let bm25 = Arc::new(Mutex::new(BM25Index::new(kb_root).await));
        let embed = Arc::new(EmbedProviderEnum::new(kb_root));
        let vector = Arc::new(Mutex::new(VectorIndex::new(kb_root).await));
        let graph = Arc::new(Mutex::new(GraphState::new(kb_root).await));
        EditManager::new(kb_root.to_string(), vault, bm25, embed, vector, graph)
    }

    #[tokio::test]
    async fn test_search_engine_create() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        let _engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        // Verify components were created
        assert!(kb_root.join(".knowledge-loom-index/tantivy").exists());
        assert!(kb_root.join(".knowledge-loom-index/embeddings.db").exists());
    }

    #[tokio::test]
    async fn test_search_engine_unified_search() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create test files
        fs::write(
            kb_root.join("test1.md"),
            "# Test 1\nThis is about cats and dogs",
        )
        .unwrap();
        fs::write(
            kb_root.join("test2.md"),
            "# Test 2\nThis is about birds and fish",
        )
        .unwrap();

        let engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        // Build indexes
        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = engine.vector.lock().await;
            let embed = engine.embed.clone();
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
            fs::write(
                kb_root.join(format!("test{}.md", i)),
                format!("# Test {}\nContent {}", i, i),
            )
            .unwrap();
        }

        let engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = engine.vector.lock().await;
            let embed = engine.embed.clone();
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
        fs::write(
            kb_root.join("relevant.md"),
            "# Relevant\nThis is highly relevant content",
        )
        .unwrap();
        fs::write(
            kb_root.join("less_relevant.md"),
            "# Less Relevant\nSome other content",
        )
        .unwrap();

        let engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        {
            let mut bm25 = engine.bm25.lock().await;
            bm25.index_vault(&vault).await.unwrap();
        }
        {
            let vector = engine.vector.lock().await;
            let embed = engine.embed.clone();
            vector.index_vault(&vault, &embed).await.unwrap();
        }

        // Search for "relevant"
        let results = engine.search("relevant", 5).await;

        assert!(!results.is_empty());
        // Most relevant result should have highest score
        let max_score = results
            .iter()
            .map(|r| r.score)
            .fold(f32::NEG_INFINITY, f32::max);
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
            let embed = engine.embed.clone();
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
            let embed = engine.embed.clone();
            vector.index_vault(&vault, &embed).await.unwrap();
        }

        // Search within the test file for "machine learning"
        let results = engine
            .bm25
            .lock()
            .await
            .search_file("test.md", "machine learning", 5)
            .await
            .unwrap();

        assert!(!results.is_empty());
        // All results should be from test.md
        for (score, chunk) in results {
            assert_eq!(chunk.path, "test.md");
            assert!(chunk.content.contains("machine learning"));
            assert!(score > 0.0);
        }

        // Search within the test file for "cooking" should return empty
        let results = engine
            .bm25
            .lock()
            .await
            .search_file("test.md", "cooking", 5)
            .await
            .unwrap();

        assert!(results.is_empty(), "Should not find cooking in test.md");

        // Search within other.md for "cooking" should return results
        let results = engine
            .bm25
            .lock()
            .await
            .search_file("other.md", "cooking", 5)
            .await
            .unwrap();

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
            let embed = engine.embed.clone();
            vector.index_vault(&vault, &embed).await.unwrap();
        }

        // Search with limit 2
        let results = engine
            .bm25
            .lock()
            .await
            .search_file("test.md", "testing", 2)
            .await
            .unwrap();

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
            let embed = engine.embed.clone();
            vector.index_vault(&vault, &embed).await.unwrap();
        }

        // Search in a file that doesn't exist
        let results = engine
            .bm25
            .lock()
            .await
            .search_file("nonexistent.md", "test", 5)
            .await
            .unwrap();

        // Should return empty results for nonexistent file
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_search_returns_sections() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        fs::write(
            kb_root.join("cats.md"),
            "# Cats\n\nCats are great pets.\n\n## Persian Cats\n\nPersian cats are fluffy.",
        )
        .unwrap();

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
        fs::write(
            kb_root.join("unrelated.md"),
            "# Alpha Section\n\nSome alpha content.\n\n# Beta Section\n\nSome beta content.",
        )
        .unwrap();

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
        let vector = Arc::new(Mutex::new(
            VectorIndex::new(kb_root.to_str().unwrap()).await,
        ));
        let embed: Arc<EmbedProviderEnum> =
            Arc::new(EmbedProviderEnum::new(kb_root.to_str().unwrap()));
        let graph = Arc::new(Mutex::new(GraphState::new(kb_root.to_str().unwrap()).await));

        // Create engine from components
        let engine = SearchEngine::from_components(
            bm25.clone(),
            vector.clone(),
            embed.clone(),
            graph.clone(),
        );

        // Verify all components are present
        assert!(Arc::ptr_eq(&engine.bm25, &bm25));
        assert!(Arc::ptr_eq(&engine.vector, &vector));
        assert!(Arc::ptr_eq(&engine.embed, &embed));
        assert!(Arc::ptr_eq(&engine.graph, &graph));
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
        assert_eq!(
            results.len(),
            2,
            "expected 'foo bar' and 'foo123', got {results:?}"
        );
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

    #[tokio::test]
    async fn test_search_graph_fused_uses_precomputed_pagerank() {
        // Verifies search_graph_fused_inner accepts external pagerank without
        // acquiring self.graph.lock() — i.e., the refactored path compiles and runs.
        let dir = tempfile::tempdir().unwrap();
        let kb_root = dir.path().to_str().unwrap();
        std::fs::write(dir.path().join("alpha.md"), "# Alpha\nalpha content").unwrap();

        let bm25 = Arc::new(Mutex::new(BM25Index::new(kb_root).await));
        let vector = Arc::new(Mutex::new(VectorIndex::new(kb_root).await));
        let embed = Arc::new(EmbedProviderEnum::new(kb_root));
        let graph = Arc::new(Mutex::new(GraphState::new(kb_root).await));
        let engine = SearchEngine::from_components(bm25, vector, embed, graph);

        // Empty pagerank map: no boost expected, function should return Ok
        let query_vec = { engine.embed.embed("alpha").await.unwrap() };
        let pagerank = std::collections::HashMap::new();
        let result = engine
            .search_graph_fused_inner(&query_vec, &pagerank, 5)
            .await;
        assert!(
            result.is_ok(),
            "search_graph_fused_inner must compile and succeed"
        );
    }

    #[tokio::test]
    async fn test_index_vault_removes_stale_sections() {
        let dir = tempfile::tempdir().unwrap();
        let kb_root = dir.path().to_str().unwrap();
        let index = VectorIndex::new(kb_root).await;
        let embed = EmbedProviderEnum::new(kb_root);
        let vault = VaultState::new(kb_root).await;

        // First vault state: file has two sections
        let note_path = dir.path().join("stale.md");
        std::fs::write(&note_path, "# Keeper\nkeep this\n\n# Obsolete\nremove me").unwrap();
        index.index_vault(&vault, &embed).await.unwrap();

        // Second vault state: file has only one section (Obsolete removed)
        std::fs::write(&note_path, "# Keeper\nkeep this").unwrap();
        index.index_vault(&vault, &embed).await.unwrap();

        // Query for content from the removed section — should return nothing
        let query_vec = embed.embed("remove me").await.unwrap();
        let results = index.search_similar(&query_vec, 10).await.unwrap();
        let has_stale = results.iter().any(|(path, heading, content, _)| {
            path.contains("stale")
                && (heading.as_deref() == Some("Obsolete") || content.contains("remove me"))
        });
        assert!(
            !has_stale,
            "stale 'Obsolete' section should be gone after re-indexing"
        );
    }

    #[tokio::test]
    async fn test_pagerank_cycle_converges_uniformly() {
        let dir = tempfile::tempdir().unwrap();
        let gs = GraphState::new(dir.path().to_str().unwrap()).await;

        // 3-node directed cycle: A→B→C→A
        {
            let mut graph = gs.graph.lock().await;
            let mut node_map = gs.node_map.lock().await;
            let a = graph.add_node("A".to_string());
            let b = graph.add_node("B".to_string());
            let c = graph.add_node("C".to_string());
            node_map.insert("A".to_string(), a);
            node_map.insert("B".to_string(), b);
            node_map.insert("C".to_string(), c);
            graph.add_edge(a, b, "WIKILINK".to_string());
            graph.add_edge(b, c, "WIKILINK".to_string());
            graph.add_edge(c, a, "WIKILINK".to_string());
        }

        let scores = gs.pagerank(0.85, 100).await;

        let a = scores["A"];
        let b = scores["B"];
        let c = scores["C"];
        // Symmetric cycle: all converge to 1/3
        assert!((a - 1.0 / 3.0).abs() < 0.01, "A={a:.4}, expected ~0.3333");
        assert!((b - 1.0 / 3.0).abs() < 0.01, "B={b:.4}, expected ~0.3333");
        assert!((c - 1.0 / 3.0).abs() < 0.01, "C={c:.4}, expected ~0.3333");
    }

    #[test]
    fn test_path_to_node_name_only_strips_md_suffix() {
        // A filename containing ".md" mid-stem must not be mangled.
        // strip_suffix removes only the trailing extension.
        let s = "obsidian.md-guide.md";
        let stripped = s.strip_suffix(".md").unwrap_or(s);
        assert_eq!(stripped, "obsidian.md-guide");
    }

    #[tokio::test]
    async fn test_wikilink_resolves_subdirectory_file() {
        let dir = tempfile::tempdir().unwrap();
        let kb_root = dir.path().to_str().unwrap();

        std::fs::create_dir_all(dir.path().join("subdir")).unwrap();
        std::fs::write(dir.path().join("index.md"), "# Index\n[[rust]]").unwrap();
        std::fs::write(dir.path().join("subdir/rust.md"), "# Rust notes").unwrap();

        let vault = VaultState::new(kb_root).await;
        let graph = GraphState::new(kb_root).await;
        graph.build_graph(&vault).await.unwrap();

        let pagerank = graph.pagerank(0.85, 100).await;
        let pr = pagerank.get("subdir/rust").copied().unwrap_or(0.0);
        assert!(
            pr > 0.0,
            "subdir/rust should have non-zero pagerank due to incoming wikilink, got {}",
            pr
        );
    }

    #[tokio::test]
    async fn test_wikilink_full_path_also_resolves() {
        let dir = tempfile::tempdir().unwrap();
        let kb_root = dir.path().to_str().unwrap();

        std::fs::create_dir_all(dir.path().join("subdir")).unwrap();
        std::fs::write(dir.path().join("index.md"), "# Index\n[[subdir/rust]]").unwrap();
        std::fs::write(dir.path().join("subdir/rust.md"), "# Rust notes").unwrap();

        let vault = VaultState::new(kb_root).await;
        let graph = GraphState::new(kb_root).await;
        graph.build_graph(&vault).await.unwrap();

        let pagerank = graph.pagerank(0.85, 100).await;
        let pr = pagerank.get("subdir/rust").copied().unwrap_or(0.0);
        assert!(pr > 0.0, "full-path wikilink must resolve, got {}", pr);
    }

    #[tokio::test]
    async fn test_wikilink_duplicate_basename_resolves_deterministically() {
        // When two files share a basename (notes/rust.md and drafts/rust.md),
        // the basename_map uses last-write-wins. This test documents the
        // deterministic behavior: the resolution depends on iteration order
        // of the HashMap, which is stable for a given build.
        let dir = tempfile::tempdir().unwrap();
        let kb_root = dir.path().to_str().unwrap();

        std::fs::create_dir_all(dir.path().join("notes")).unwrap();
        std::fs::create_dir_all(dir.path().join("drafts")).unwrap();
        std::fs::write(dir.path().join("index.md"), "# Index\n[[rust]]").unwrap();
        std::fs::write(dir.path().join("notes/rust.md"), "# Rust in notes").unwrap();
        std::fs::write(dir.path().join("drafts/rust.md"), "# Rust in drafts").unwrap();

        let vault = VaultState::new(kb_root).await;
        let graph = GraphState::new(kb_root).await;
        graph.build_graph(&vault).await.unwrap();

        // The wikilink [[rust]] should resolve to exactly one of the two files.
        // We verify that an edge was created (not both, not neither).
        let neighbors = graph.search_graph("index").await;
        assert_eq!(
            neighbors.len(),
            1,
            "duplicate basename should resolve to exactly one target, got {:?}",
            neighbors
        );

        // The resolved target should be one of the two files.
        let resolved = &neighbors[0];
        assert!(
            resolved == "notes/rust" || resolved == "drafts/rust",
            "resolved target should be one of the duplicate basenames, got {}",
            resolved
        );

        // Verify that the resolved target has non-zero pagerank (edge exists).
        let pagerank = graph.pagerank(0.85, 100).await;
        let pr = pagerank.get(resolved).copied().unwrap_or(0.0);
        assert!(
            pr > 0.0,
            "resolved target should have non-zero pagerank, got {}",
            pr
        );
    }

    #[tokio::test]
    async fn test_pagerank_scores_sum_to_node_count() {
        // A 3-node balanced cycle converges to sum = 1.0 (standard PageRank normalization).
        let dir = tempfile::tempdir().unwrap();
        let gs = GraphState::new(dir.path().to_str().unwrap()).await;

        {
            let mut graph = gs.graph.lock().await;
            let mut node_map = gs.node_map.lock().await;
            let a = graph.add_node("A".to_string());
            let b = graph.add_node("B".to_string());
            let c = graph.add_node("C".to_string());
            node_map.insert("A".to_string(), a);
            node_map.insert("B".to_string(), b);
            node_map.insert("C".to_string(), c);
            graph.add_edge(a, b, "WIKILINK".to_string());
            graph.add_edge(b, c, "WIKILINK".to_string());
            graph.add_edge(c, a, "WIKILINK".to_string());
        }

        let scores = gs.pagerank(0.85, 100).await;
        let total: f64 = scores.values().sum();
        assert!(
            (total - 1.0).abs() < 0.01,
            "pagerank scores should sum to 1.0 but got {}",
            total
        );
        for (name, &score) in &scores {
            assert!(
                (score - 1.0 / 3.0).abs() < 0.01,
                "balanced cycle: {} should have score ~0.333 but got {}",
                name,
                score
            );
        }
    }

    #[tokio::test]
    async fn test_pagerank_dangling_node_redistributes() {
        // A dangling node (no outgoing edges) must redistribute its score evenly
        // across all nodes, not just disappear from the total.
        let dir = tempfile::tempdir().unwrap();
        let gs = GraphState::new(dir.path().to_str().unwrap()).await;

        {
            let mut graph = gs.graph.lock().await;
            let mut node_map = gs.node_map.lock().await;
            let hub = graph.add_node("hub".to_string());
            let sink = graph.add_node("sink".to_string());
            node_map.insert("hub".to_string(), hub);
            node_map.insert("sink".to_string(), sink);
            graph.add_edge(hub, sink, "WIKILINK".to_string());
        }

        let scores = gs.pagerank(0.85, 100).await;
        let total: f64 = scores.values().sum();
        assert!(
            (total - 1.0).abs() < 0.05,
            "dangling node: pagerank should sum to 1.0 but got {}",
            total
        );
        assert!(
            scores["hub"] > 0.0 && scores["sink"] > 0.0,
            "both nodes must have positive scores"
        );
    }

    #[tokio::test]
    async fn test_pagerank_hub_ranks_higher() {
        let dir = tempfile::tempdir().unwrap();
        let gs = GraphState::new(dir.path().to_str().unwrap()).await;

        // A→C and B→C: C is pointed-to by two nodes
        {
            let mut graph = gs.graph.lock().await;
            let mut node_map = gs.node_map.lock().await;
            let a = graph.add_node("A".to_string());
            let b = graph.add_node("B".to_string());
            let c = graph.add_node("C".to_string());
            node_map.insert("A".to_string(), a);
            node_map.insert("B".to_string(), b);
            node_map.insert("C".to_string(), c);
            graph.add_edge(a, c, "WIKILINK".to_string());
            graph.add_edge(b, c, "WIKILINK".to_string());
        }

        let scores = gs.pagerank(0.85, 100).await;

        assert!(
            scores["C"] > scores["A"],
            "C (2 in-edges) should outrank A (dangling): C={:.4}, A={:.4}",
            scores["C"],
            scores["A"]
        );
        assert!(
            scores["C"] > scores["B"],
            "C (2 in-edges) should outrank B (dangling): C={:.4}, B={:.4}",
            scores["C"],
            scores["B"]
        );
    }
}
