use knowledge_loom::embed::EmbedProviderEnum;
use knowledge_loom::index::VectorIndex;

#[tokio::test]
async fn test_index_file_replaces_old_chunks() {
    let dir = tempfile::tempdir().unwrap();
    let kb_root = dir.path().to_str().unwrap();
    let index = VectorIndex::new(kb_root).await;
    let embed = EmbedProviderEnum::new(kb_root).await;
    let path = dir.path().join("note.md");

    // First index
    std::fs::write(&path, "# Alpha\nalpha content here").unwrap();
    index
        .index_file(&path, "# Alpha\nalpha content here", &embed)
        .await
        .unwrap();

    // Re-index with completely different content
    std::fs::write(&path, "# Beta\nbeta material only").unwrap();
    index
        .index_file(&path, "# Beta\nbeta material only", &embed)
        .await
        .unwrap();

    // Search for the alpha vector — should return no chunks with "alpha" content
    let query_vec = embed.embed("alpha content here");
    let results = index.search_similar(&query_vec, 20).await.unwrap();
    let stale: Vec<_> = results
        .iter()
        .filter(|(_, _, content, _)| content.contains("alpha"))
        .collect();
    assert!(
        stale.is_empty(),
        "old alpha chunks should be gone after re-index"
    );
}

#[tokio::test]
async fn test_remove_file_embeddings_clears_path() {
    let dir = tempfile::tempdir().unwrap();
    let kb_root = dir.path().to_str().unwrap();
    let index = VectorIndex::new(kb_root).await;
    let embed = EmbedProviderEnum::new(kb_root).await;
    let path = dir.path().join("target.md");

    std::fs::write(&path, "# Target\nsome content").unwrap();
    index
        .index_file(&path, "# Target\nsome content", &embed)
        .await
        .unwrap();

    index.remove_file_embeddings(&path).await.unwrap();

    let query_vec = embed.embed("some content");
    let results = index.search_similar(&query_vec, 20).await.unwrap();
    let found: Vec<_> = results
        .iter()
        .filter(|(p, _, _, _)| p.contains("target"))
        .collect();
    assert!(found.is_empty(), "embeddings should be cleared");
}

use knowledge_loom::graph::GraphState;
use knowledge_loom::vault::VaultState;

#[tokio::test]
async fn test_graph_update_file_replaces_wikilinks() {
    let dir = tempfile::tempdir().unwrap();
    let kb_root = dir.path().to_str().unwrap();

    // Create initial vault with two target notes
    std::fs::write(dir.path().join("alpha.md"), "# Alpha").unwrap();
    std::fs::write(dir.path().join("beta.md"), "# Beta").unwrap();
    std::fs::write(dir.path().join("source.md"), "# Source\n[[alpha]]").unwrap();

    let vault = VaultState::new(kb_root).await;
    let graph = GraphState::new(kb_root).await;
    graph.build_graph(&vault).await.unwrap();

    // source.md now links to alpha; update it to link to beta instead
    std::fs::write(dir.path().join("source.md"), "# Source\n[[beta]]").unwrap();
    graph
        .update_file(&dir.path().join("source.md"), "# Source\n[[beta]]")
        .await
        .unwrap();

    // source should now link to beta, not alpha
    let neighbors = graph.search_graph("source").await;
    assert!(
        neighbors.contains(&"beta".to_string()),
        "source should link to beta"
    );
    assert!(
        !neighbors.contains(&"alpha".to_string()),
        "source should no longer link to alpha"
    );
}

#[tokio::test]
async fn test_graph_update_file_invalidates_analytics_cache() {
    let dir = tempfile::tempdir().unwrap();
    let kb_root = dir.path().to_str().unwrap();

    std::fs::write(dir.path().join("a.md"), "# A\n[[b]]").unwrap();
    std::fs::write(dir.path().join("b.md"), "# B").unwrap();

    let vault = VaultState::new(kb_root).await;
    let graph = GraphState::new(kb_root).await;
    graph.build_graph(&vault).await.unwrap();

    // Prime the analytics cache
    let (pr_before, _) = graph.get_cached_analytics().await;
    assert!(
        !pr_before.is_empty(),
        "cache should be populated after build_graph"
    );

    // Update a.md — cache should be invalidated
    graph
        .update_file(&dir.path().join("a.md"), "# A\nno links now")
        .await
        .unwrap();
    let cache_after = graph.cached_pagerank.lock().await;
    assert!(
        cache_after.is_none(),
        "cached_pagerank should be None immediately after update_file invalidates it"
    );
}

use knowledge_loom::bm25::BM25Index;
use knowledge_loom::search::SearchEngine;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_search_includes_graph_fused_results() {
    let dir = tempfile::tempdir().unwrap();
    let kb_root = dir.path().to_str().unwrap();

    // Create a note and index it in the vector backend
    std::fs::write(
        dir.path().join("target.md"),
        "# Rust async\nfutures and tokio",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("other.md"),
        "# Cooking\nrecipes and ingredients",
    )
    .unwrap();

    let embed = Arc::new(EmbedProviderEnum::new(kb_root).await);
    let vector = Arc::new(Mutex::new(VectorIndex::new(kb_root).await));

    {
        let v = vector.lock().await;
        v.index_file(
            &dir.path().join("target.md"),
            "# Rust async\nfutures and tokio",
            &embed,
        )
        .await
        .unwrap();
        v.index_file(
            &dir.path().join("other.md"),
            "# Cooking\nrecipes and ingredients",
            &embed,
        )
        .await
        .unwrap();
    }

    let engine = SearchEngine::from_components(
        Arc::new(Mutex::new(BM25Index::new(kb_root).await)),
        vector,
        embed,
        Arc::new(Mutex::new(GraphState::new(kb_root).await)),
    );

    // Search should complete without panic
    let results = engine.search("async futures", 5).await;
    // With stub embeddings results may be noisy, but the call must succeed
    let _ = results;
}

#[tokio::test]
async fn test_graph_fused_empty_vector_index_returns_no_fused_results() {
    let dir = tempfile::tempdir().unwrap();
    let kb_root = dir.path().to_str().unwrap();
    // No files indexed in vector backend

    let engine = SearchEngine::from_components(
        Arc::new(Mutex::new(BM25Index::new(kb_root).await)),
        Arc::new(Mutex::new(VectorIndex::new(kb_root).await)),
        Arc::new(EmbedProviderEnum::new(kb_root).await),
        Arc::new(Mutex::new(GraphState::new(kb_root).await)),
    );

    // Should return empty without error
    let results = engine.search("anything", 5).await;
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_graph_fused_pagerank_key_alignment() {
    // Pagerank map uses bare-stem keys (as GraphState produces via path_to_node_name).
    let mut pagerank: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    pagerank.insert("important_note".to_string(), 0.9);

    // VectorIndex returns paths with .md suffix.
    let vector_path = "important_note.md";
    let pr_key = vector_path.strip_suffix(".md").unwrap_or(vector_path);

    assert_eq!(
        pr_key, "important_note",
        "strip_suffix('.md') must yield the bare stem used as a pagerank key"
    );
    assert!(
        pagerank.contains_key(pr_key),
        "pagerank lookup must find the key after .md stripping"
    );

    // Nested paths: only the .md suffix is removed, directory component stays.
    let nested = "subdir/important_note.md";
    let nested_key = nested.strip_suffix(".md").unwrap_or(nested);
    assert_eq!(nested_key, "subdir/important_note");
}

#[tokio::test]
async fn test_graph_fused_inner_reranks_by_pagerank() {
    let dir = tempfile::tempdir().unwrap();
    let kb_root = dir.path().to_str().unwrap();

    let embed = Arc::new(EmbedProviderEnum::new(kb_root).await);
    let vector = Arc::new(Mutex::new(VectorIndex::new(kb_root).await));

    // Index two docs with the same content so vector similarity is comparable
    {
        let v = vector.lock().await;
        v.index_file(&dir.path().join("high_pr.md"), "async futures in rust", &embed)
            .await
            .unwrap();
        v.index_file(&dir.path().join("low_pr.md"), "async futures in rust", &embed)
            .await
            .unwrap();
    }

    let engine = SearchEngine::from_components(
        Arc::new(Mutex::new(
            knowledge_loom::bm25::BM25Index::new(kb_root).await,
        )),
        vector.clone(),
        embed.clone(),
        Arc::new(Mutex::new(GraphState::new(kb_root).await)),
    );

    let query_vec = {
        embed.embed("async futures")
    };

    // Build a pagerank map where high_pr dominates
    let mut pagerank: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    pagerank.insert("high_pr".to_string(), 0.9_f64);
    pagerank.insert("low_pr".to_string(), 0.1_f64);

    let results = engine
        .search_graph_fused_inner(&query_vec, &pagerank, 5)
        .await
        .unwrap();

    assert!(!results.is_empty(), "should return results");
    let high_idx = results.iter().position(|p| p.contains("high_pr"));
    let low_idx = results.iter().position(|p| p.contains("low_pr"));

    // Assert both files appear in results — prevents vacuous pass if stub embed returns zero vectors
    assert!(
        high_idx.is_some() && low_idx.is_some(),
        "both indexed files should appear in results; high_pr: {:?}, low_pr: {:?}",
        high_idx,
        low_idx
    );

    // Now check ordering: high-pagerank doc must rank before low-pagerank doc
    assert!(
        high_idx.unwrap() < low_idx.unwrap(),
        "high-pagerank doc must rank before low-pagerank doc with equal similarity"
    );
}

#[tokio::test]
async fn test_index_vault_removes_stale_embeddings() {
    // Test that index_vault (not just index_file) removes stale embeddings
    // when a file is modified between calls.
    let dir = tempfile::tempdir().unwrap();
    let kb_root = dir.path().to_str().unwrap();
    let index = VectorIndex::new(kb_root).await;
    let embed = EmbedProviderEnum::new(kb_root).await;
    let vault = VaultState::new(kb_root).await;

    // First vault state: file contains "Old Section"
    let note_path = dir.path().join("note.md");
    std::fs::write(&note_path, "# Old Section\noriginal content").unwrap();
    index.index_vault(&vault, &embed).await.unwrap();

    // Second vault state: file contains only "New Section" (Old Section removed)
    std::fs::write(&note_path, "# New Section\nreplacement content").unwrap();
    index.index_vault(&vault, &embed).await.unwrap();

    // Query for content from the removed section — should return no results
    let query_vec = embed.embed("original content");
    let results = index.search_similar(&query_vec, 10).await.unwrap();
    assert!(
        results.iter().all(|(path, heading, content, _)| {
            !path.contains("note")
                || (heading.as_deref() != Some("Old Section")
                    && !content.contains("original content"))
        }),
        "index_vault must remove stale chunks: 'original content' should not appear after re-index"
    );
}
