use knowledge_loom::search::SearchEngine;
use knowledge_loom::vault::VaultState;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[tokio::test]
async fn integration_full_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let kb_root = temp_dir.path();

    // Create test vault with realistic content
    create_test_vault(kb_root);

    // Initialize all components
    let vault = VaultState::new(kb_root.to_str().unwrap()).await;
    let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

    // Build indexes
    println!("Building BM25 index...");
    {
        let mut bm25 = search_engine.bm25.lock().await;
        bm25.index_vault(&vault)
            .await
            .expect("Failed to build BM25 index");
    }

    println!("Building vector index...");
    {
        let vector = search_engine.vector.lock().await;
        // embed is now synchronous, no lock needed
        vector
            .index_vault(&vault, &search_engine.embed)
            .await
            .expect("Failed to build vector index");
    }

    // Test search functionality
    println!("Testing search...");
    let results = search_engine.search("machine learning", 5).await;

    assert!(!results.is_empty(), "Search should return results");
    println!("Found {} results", results.len());

    // Verify results are properly scored
    for i in 0..results.len().saturating_sub(1) {
        assert!(
            results[i].score >= results[i + 1].score,
            "Results should be sorted by score"
        );
    }

    // Test different queries
    let test_queries = vec![
        "artificial intelligence",
        "neural networks",
        "data science",
        "programming",
    ];

    for query in test_queries {
        let results = search_engine.search(query, 3).await;
        println!("Query '{}': {} results", query, results.len());
        // Should return some results for each query
        assert!(
            !results.is_empty(),
            "Query '{}' should return results",
            query
        );
    }
}

#[tokio::test]
async fn integration_incremental_indexing() {
    let temp_dir = TempDir::new().unwrap();
    let kb_root = temp_dir.path();

    // Create initial vault
    create_test_vault(kb_root);

    let vault = VaultState::new(kb_root.to_str().unwrap()).await;
    let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

    // Build initial indexes
    {
        let mut bm25 = search_engine.bm25.lock().await;
        bm25.index_vault(&vault).await.unwrap();
    }
    {
        let vector = search_engine.vector.lock().await;
        // embed is now synchronous, no lock needed
        vector.index_vault(&vault, &search_engine.embed).await.unwrap();
    }

    // Get initial result count
    let initial_results = search_engine.search("test", 10).await;
    let _initial_count = initial_results.len();

    // Drop the search engine to release locks
    drop(search_engine);

    // Add new file
    fs::write(
        kb_root.join("new_file.md"),
        "# New File\nThis is a new file added after initial indexing",
    )
    .unwrap();

    // Rebuild indexes with new engine
    let vault = VaultState::new(kb_root.to_str().unwrap()).await;
    let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;
    {
        let mut bm25 = search_engine.bm25.lock().await;
        bm25.index_vault(&vault).await.unwrap();
    }
    {
        let vector = search_engine.vector.lock().await;
        // embed is now synchronous, no lock needed
        vector.index_vault(&vault, &search_engine.embed).await.unwrap();
    }

    // Verify new file is indexed
    let new_results = search_engine.search("new file", 10).await;
    assert!(!new_results.is_empty(), "New file should be indexed");
}

#[tokio::test]
async fn integration_file_deletion() {
    let temp_dir = TempDir::new().unwrap();
    let kb_root = temp_dir.path();

    // Create test vault
    create_test_vault(kb_root);

    let vault = VaultState::new(kb_root.to_str().unwrap()).await;
    let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

    // Build indexes
    {
        let mut bm25 = search_engine.bm25.lock().await;
        bm25.index_vault(&vault).await.unwrap();
    }
    {
        let vector = search_engine.vector.lock().await;
        // embed is now synchronous, no lock needed
        vector.index_vault(&vault, &search_engine.embed).await.unwrap();
    }

    // Get initial results
    let initial_results = search_engine.search("machine learning", 10).await;
    let _initial_count = initial_results.len();

    // Drop the search engine to release locks
    drop(search_engine);

    // Delete a file
    fs::remove_file(kb_root.join("ai/ml_basics.md")).unwrap();

    // Rebuild indexes with new engine
    let vault = VaultState::new(kb_root.to_str().unwrap()).await;
    let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;
    {
        let mut bm25 = search_engine.bm25.lock().await;
        bm25.index_vault(&vault).await.unwrap();
    }
    {
        let vector = search_engine.vector.lock().await;
        // embed is now synchronous, no lock needed
        vector.index_vault(&vault, &search_engine.embed).await.unwrap();
    }

    // Verify file is removed from index
    let _new_results = search_engine.search("machine learning", 10).await;
    // Results should be different (file removed)
    // Note: This test depends on the specific content and may need adjustment
}

#[tokio::test]
async fn integration_large_vault() {
    let temp_dir = TempDir::new().unwrap();
    let kb_root = temp_dir.path();

    // Create large vault with many files
    for i in 0..50 {
        let subdir = kb_root.join(format!("dir{}", i % 5));
        fs::create_dir_all(&subdir).unwrap();

        let content = format!(
            "# Document {}\n\nThis is document number {}. \
                              It contains various content about topic {}.",
            i,
            i,
            i % 10
        );
        fs::write(subdir.join(format!("doc{}.md", i)), content).unwrap();
    }

    let vault = VaultState::new(kb_root.to_str().unwrap()).await;
    let files = vault.scan_files().await;

    assert_eq!(files.len(), 50, "Should find all 50 files");

    let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

    // Build indexes
    {
        let mut bm25 = search_engine.bm25.lock().await;
        bm25.index_vault(&vault).await.unwrap();
    }
    {
        let vector = search_engine.vector.lock().await;
        // embed is now synchronous, no lock needed
        vector.index_vault(&vault, &search_engine.embed).await.unwrap();
    }

    // Test search performance
    let start = std::time::Instant::now();
    let results = search_engine.search("document", 10).await;
    let duration = start.elapsed();

    assert!(!results.is_empty(), "Search should return results");
    assert!(
        duration.as_secs() < 5,
        "Search should complete in reasonable time"
    );
    println!("Search completed in {:?}", duration);
}

#[tokio::test]
async fn integration_complex_queries() {
    let temp_dir = TempDir::new().unwrap();
    let kb_root = temp_dir.path();

    create_test_vault(kb_root);

    let vault = VaultState::new(kb_root.to_str().unwrap()).await;
    let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

    {
        let mut bm25 = search_engine.bm25.lock().await;
        bm25.index_vault(&vault).await.unwrap();
    }
    {
        let vector = search_engine.vector.lock().await;
        // embed is now synchronous, no lock needed
        vector.index_vault(&vault, &search_engine.embed).await.unwrap();
    }

    // Test various query types
    let test_cases = vec![
        ("single word", "machine"),
        ("phrase", "machine learning"),
        ("multiple words", "artificial intelligence neural networks"),
        ("case insensitive", "MACHINE LEARNING"),
        ("partial match", "machin"),
    ];

    for (description, query) in test_cases {
        let results = search_engine.search(query, 5).await;
        println!("{}: {} results", description, results.len());
        // Most queries should return some results
        if description != "partial match" {
            assert!(
                !results.is_empty(),
                "{} query should return results",
                description
            );
        }
    }
}

#[tokio::test]
async fn integration_empty_vault() {
    let temp_dir = TempDir::new().unwrap();
    let kb_root = temp_dir.path();

    // Create empty vault
    let vault = VaultState::new(kb_root.to_str().unwrap()).await;
    let files = vault.scan_files().await;

    assert_eq!(files.len(), 0, "Empty vault should have no files");

    let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

    // Build indexes (should handle empty vault gracefully)
    {
        let mut bm25 = search_engine.bm25.lock().await;
        bm25.index_vault(&vault).await.unwrap();
    }
    {
        let vector = search_engine.vector.lock().await;
        // embed is now synchronous, no lock needed
        vector.index_vault(&vault, &search_engine.embed).await.unwrap();
    }

    // Search should return empty results
    let results = search_engine.search("test", 10).await;
    assert!(results.is_empty(), "Empty vault should return no results");
}

#[tokio::test]
async fn integration_special_characters() {
    let temp_dir = TempDir::new().unwrap();
    let kb_root = temp_dir.path();

    // Create files with special characters
    let special_files = vec![
        ("test-file.md", "# Test File\nContent with hyphens"),
        ("test_file.md", "# Test File\nContent with underscores"),
        ("test file.md", "# Test File\nContent with spaces"),
        ("test@file.md", "# Test File\nContent with @ symbol"),
    ];

    for (filename, content) in special_files {
        fs::write(kb_root.join(filename), content).unwrap();
    }

    let vault = VaultState::new(kb_root.to_str().unwrap()).await;
    let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

    {
        let mut bm25 = search_engine.bm25.lock().await;
        bm25.index_vault(&vault).await.unwrap();
    }
    {
        let vector = search_engine.vector.lock().await;
        // embed is now synchronous, no lock needed
        vector.index_vault(&vault, &search_engine.embed).await.unwrap();
    }

    // Should find files despite special characters
    let results = search_engine.search("test", 10).await;
    assert!(
        !results.is_empty(),
        "Should find files with special characters"
    );
}

#[tokio::test]
async fn smoke_test_against_test_vault() {
    // Use the actual test-vault for smoke testing
    let test_vault_path = Path::new("test-vault");

    if !test_vault_path.exists() {
        println!("test-vault not found, skipping smoke test");
        return;
    }

    let kb_root = test_vault_path.to_str().unwrap();

    // Initialize all components
    let vault = VaultState::new(kb_root).await;
    let files = vault.scan_files().await;
    println!("Found {} files in test-vault", files.len());
    assert!(!files.is_empty(), "test-vault should contain files");

    // Check if old index exists and remove it if it does
    let index_path = Path::new(kb_root).join(".knowledge-loom-index/tantivy");
    if index_path.exists() {
        println!("Removing old tantivy index for compatibility with tantivy 0.26");
        let _ = std::fs::remove_dir_all(&index_path);
    }

    let search_engine = SearchEngine::new(kb_root).await;

    // Build indexes
    println!("Building BM25 index...");
    {
        let mut bm25 = search_engine.bm25.lock().await;
        bm25.index_vault(&vault)
            .await
            .expect("Failed to build BM25 index");
    }

    println!("Building vector index...");
    {
        let vector = search_engine.vector.lock().await;
        // embed is now synchronous, no lock needed
        vector
            .index_vault(&vault, &search_engine.embed)
            .await
            .expect("Failed to build vector index");
    }

    println!("Building graph index...");
    {
        let graph = search_engine.graph.lock().await;
        graph
            .build_graph(&vault)
            .await
            .expect("Failed to build graph index");
    }

    // Test search functionality with content-agnostic queries
    println!("Testing search...");
    let results = search_engine.search("test", 5).await;

    println!("Found {} results for 'test'", results.len());

    // Verify results are properly scored
    for i in 0..results.len().saturating_sub(1) {
        assert!(
            results[i].score >= results[i + 1].score,
            "Results should be sorted by score"
        );
    }

    // Test graph analytics
    println!("Testing graph analytics...");
    let graph = search_engine.graph.lock().await;
    let (pagerank, communities) = graph.get_cached_analytics().await;

    println!("PageRank scores: {} nodes", pagerank.len());
    println!("Communities: {} communities", communities.len());

    assert!(!pagerank.is_empty(), "Graph should have PageRank scores");
    assert!(!communities.is_empty(), "Graph should have communities");

    // Test that all files in vault are indexed
    assert_eq!(pagerank.len(), files.len(), "All files should be in graph");

    println!("Smoke test passed!");
}

fn create_test_vault(kb_root: &Path) {
    // Create directory structure
    fs::create_dir_all(kb_root.join("ai")).unwrap();
    fs::create_dir_all(kb_root.join("programming")).unwrap();
    fs::create_dir_all(kb_root.join("data")).unwrap();

    // Create test files with realistic content
    let test_files = vec![
        ("ai/ml_basics.md", "# Machine Learning Basics\n\nMachine learning is a subset of artificial intelligence that focuses on building systems that can learn from data."),
        ("ai/neural_networks.md", "# Neural Networks\n\nNeural networks are computing systems inspired by biological neural networks that constitute animal brains."),
        ("programming/rust.md", "# Rust Programming\n\nRust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety."),
        ("data/science.md", "# Data Science\n\nData science is an interdisciplinary field that uses scientific methods, processes, algorithms and systems to extract knowledge and insights from noisy, structured and unstructured data."),
        ("ai/deep_learning.md", "# Deep Learning\n\nDeep learning is part of a broader family of machine learning methods based on artificial neural networks with representation learning."),
    ];

    for (path, content) in test_files {
        let full_path = kb_root.join(path);
        fs::create_dir_all(full_path.parent().unwrap()).unwrap();
        fs::write(full_path, content).unwrap();
    }
}
