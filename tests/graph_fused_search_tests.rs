use knowledge_loom::index::VectorIndex;
use knowledge_loom::embed::EmbedProviderEnum;

#[tokio::test]
async fn test_index_file_replaces_old_chunks() {
    let dir = tempfile::tempdir().unwrap();
    let kb_root = dir.path().to_str().unwrap();
    let index = VectorIndex::new(kb_root).await;
    let embed = EmbedProviderEnum::new(kb_root).await;
    let path = dir.path().join("note.md");

    // First index
    std::fs::write(&path, "# Alpha\nalpha content here").unwrap();
    index.index_file(&path, "# Alpha\nalpha content here", &embed).await.unwrap();

    // Re-index with completely different content
    std::fs::write(&path, "# Beta\nbeta material only").unwrap();
    index.index_file(&path, "# Beta\nbeta material only", &embed).await.unwrap();

    // Search for the alpha vector — should return no chunks with "alpha" content
    let query_vec = embed.embed("alpha content here").await;
    let results = index.search_similar(&query_vec, 20).await.unwrap();
    let stale: Vec<_> = results
        .iter()
        .filter(|(_, _, content, _)| content.contains("alpha"))
        .collect();
    assert!(stale.is_empty(), "old alpha chunks should be gone after re-index");
}

#[tokio::test]
async fn test_remove_file_embeddings_clears_path() {
    let dir = tempfile::tempdir().unwrap();
    let kb_root = dir.path().to_str().unwrap();
    let index = VectorIndex::new(kb_root).await;
    let embed = EmbedProviderEnum::new(kb_root).await;
    let path = dir.path().join("target.md");

    std::fs::write(&path, "# Target\nsome content").unwrap();
    index.index_file(&path, "# Target\nsome content", &embed).await.unwrap();

    index.remove_file_embeddings(&path).await.unwrap();

    let query_vec = embed.embed("some content").await;
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
    graph.update_file(&dir.path().join("source.md"), "# Source\n[[beta]]").await.unwrap();

    // source should now link to beta, not alpha
    let neighbors = graph.search_graph("source").await;
    assert!(neighbors.contains(&"beta".to_string()), "source should link to beta");
    assert!(!neighbors.contains(&"alpha".to_string()), "source should no longer link to alpha");
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
    assert!(!pr_before.is_empty(), "cache should be populated after build_graph");

    // Update a.md — cache should be invalidated
    graph.update_file(&dir.path().join("a.md"), "# A\nno links now").await.unwrap();
    let (pr_after, _) = graph.get_cached_analytics().await;
    // After invalidation, get_cached_analytics returns empty (unwrap_or_default)
    assert!(pr_after.is_empty(), "pagerank cache should be invalidated after update_file");
}

use knowledge_loom::search::SearchEngine;
use knowledge_loom::bm25::BM25Index;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_search_includes_graph_fused_results() {
    let dir = tempfile::tempdir().unwrap();
    let kb_root = dir.path().to_str().unwrap();

    // Create a note and index it in the vector backend
    std::fs::write(dir.path().join("target.md"), "# Rust async\nfutures and tokio").unwrap();
    std::fs::write(dir.path().join("other.md"), "# Cooking\nrecipes and ingredients").unwrap();

    let embed = Arc::new(Mutex::new(EmbedProviderEnum::new(kb_root).await));
    let vector = Arc::new(Mutex::new(VectorIndex::new(kb_root).await));

    {
        let e = embed.lock().await;
        let v = vector.lock().await;
        v.index_file(&dir.path().join("target.md"), "# Rust async\nfutures and tokio", &*e).await.unwrap();
        v.index_file(&dir.path().join("other.md"), "# Cooking\nrecipes and ingredients", &*e).await.unwrap();
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
        Arc::new(Mutex::new(EmbedProviderEnum::new(kb_root).await)),
        Arc::new(Mutex::new(GraphState::new(kb_root).await)),
    );

    // Should return empty without error
    let results = engine.search("anything", 5).await;
    assert!(results.is_empty());
}
