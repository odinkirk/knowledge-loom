use knowledge_loom::{search::SearchEngine, vault::VaultState};
use std::fs;
use tempfile::tempdir;

/// Index vault into ALL engine backends (BM25 + vector + graph)
async fn index_into_engine(engine: &SearchEngine, vault: &VaultState) {
    // BM25 index
    let mut bm25 = engine.bm25.lock().await;
    bm25.index_vault(vault).await.unwrap();
    drop(bm25);

    // Vector index
    let vector = engine.vector.lock().await;
    vector.index_vault(vault, &engine.embed).await.unwrap();
    vector.save().await.unwrap();
    drop(vector);

    // Graph index
    let graph = engine.graph.lock().await;
    graph.build_graph(vault).await.unwrap();
    drop(graph);
}

/// Helper: create a vault, index it, return SearchEngine
async fn setup_search_vault(temp_dir: &tempfile::TempDir) -> SearchEngine {
    let kb_root = temp_dir.path().to_str().unwrap().to_string();

    fs::write(
        temp_dir.path().join("machine_learning.md"),
        "# Machine Learning\n\nNeural networks, gradient descent, backpropagation, and deep learning architectures.\n\n## Transformers\nThe transformer architecture uses self-attention mechanisms for sequence modeling.\n",
    ).unwrap();

    fs::write(
        temp_dir.path().join("cooking.md"),
        "# Italian Cooking\n\nPasta recipes, tomato sauce, olive oil, and fresh basil. Italian cuisine focuses on simple ingredients and bold flavors.\n\n## Pasta Making\n\nHomemade pasta requires flour, eggs, and a rolling pin.\n",
    ).unwrap();

    fs::write(
        temp_dir.path().join("quantum_physics.md"),
        "# Quantum Physics\n\nQuantum mechanics, Schrödinger equation, wave function collapse, and quantum entanglement.\n\n## Wave Functions\nThe wave function describes the quantum state of a particle.\n",
    ).unwrap();

    let engine = SearchEngine::new(&kb_root).await;
    let vault = VaultState::new(&kb_root).await;
    index_into_engine(&engine, &vault).await;

    engine
}

#[tokio::test]
async fn test_turbovec_e2e_search() {
    let temp_dir = tempdir().unwrap();
    let engine = setup_search_vault(&temp_dir).await;

    // Search for ML concepts — should rank machine_learning.md highest
    let results = engine.search("neural networks deep learning", 3).await;
    assert!(!results.is_empty(), "Search should return results");
    assert!(results.len() <= 3, "Should respect top_k limit");

    // The ML article should be top result
    let top_path = &results[0].path;
    assert!(
        top_path.contains("machine_learning"),
        "Expected 'machine_learning.md' as top result for ML query, got: {}",
        top_path
    );

    // Search for cooking — should rank cooking.md highest
    let results = engine.search("pasta italian cuisine", 3).await;
    assert!(!results.is_empty(), "Cooking search should return results");
    let top_path = &results[0].path;
    assert!(
        top_path.contains("cooking"),
        "Expected 'cooking.md' as top result for cooking query, got: {}",
        top_path
    );

    // Search for quantum physics
    let results = engine.search("schrödinger wave function", 3).await;
    assert!(!results.is_empty(), "Physics search should return results");
    let top_path = &results[0].path;
    assert!(
        top_path.contains("quantum_physics"),
        "Expected 'quantum_physics.md' as top result, got: {}",
        top_path
    );

    // Empty query should return empty results
    let results = engine.search("", 5).await;
    assert!(results.is_empty(), "Empty query should return no results");
}

#[tokio::test]
async fn test_turbovec_e2e_persistence() {
    let temp_dir = tempdir().unwrap();
    let kb_root = temp_dir.path().to_str().unwrap().to_string();

    // Phase 1: Index and save
    {
        fs::write(
            temp_dir.path().join("note.md"),
            "# Test Note\n\nThis is a test document about distributed systems.\n",
        )
        .unwrap();

        let engine = SearchEngine::new(&kb_root).await;
        let vault = VaultState::new(&kb_root).await;
        index_into_engine(&engine, &vault).await;

        // Verify search works
        let results = engine.search("distributed systems", 3).await;
        assert!(
            !results.is_empty(),
            "Search should find results before restart"
        );
    }

    // Phase 2: Verify index files exist on disk
    let tvim = temp_dir.path().join(".knowledge-loom-index/turbovec.tvim");
    let meta = temp_dir
        .path()
        .join(".knowledge-loom-index/turbovec_meta.bin");
    assert!(tvim.exists(), "turbovec.tvim should persist to disk");
    assert!(meta.exists(), "turbovec_meta.bin should persist to disk");

    // Phase 3: Create a new SearchEngine (simulating restart) and verify search still works
    {
        let engine = SearchEngine::new(&kb_root).await;
        let results = engine.search("distributed systems", 3).await;
        assert!(
            !results.is_empty(),
            "Search should find results after simulated restart"
        );
    }
}

#[tokio::test]
async fn test_turbovec_e2e_individual_file_operations() {
    let temp_dir = tempdir().unwrap();
    let kb_root = temp_dir.path().to_str().unwrap().to_string();

    let engine = SearchEngine::new(&kb_root).await;

    fs::write(
        temp_dir.path().join("hello.md"),
        "# Greetings\n\nHello world content for testing.\n",
    )
    .unwrap();

    let count_before = {
        let vector = engine.vector.lock().await;
        vector.count().await
    };

    {
        let vector = engine.vector.lock().await;
        vector
            .index_file(
                &temp_dir.path().join("hello.md"),
                "# Greetings\n\nHello world content for testing.\n",
                &*engine.embed,
            )
            .await
            .unwrap();
    }

    let count_after = {
        let vector = engine.vector.lock().await;
        vector.count().await
    };
    assert!(count_after > count_before, "Should index new file");

    // Search directly against turbovec vector index
    let query_embedding = engine.embed.embed("hello world greetings").await.unwrap();
    let results = {
        let vector = engine.vector.lock().await;
        vector.search_similar(&query_embedding, 5).await.unwrap()
    };
    assert!(!results.is_empty(), "Vector search should return results");

    // Remove the file
    let removed = {
        let vector = engine.vector.lock().await;
        vector
            .remove_file(&temp_dir.path().join("hello.md"))
            .await
            .unwrap()
    };
    assert!(removed > 0, "Should remove indexed chunks");

    let count_final = {
        let vector = engine.vector.lock().await;
        vector.count().await
    };
    assert_eq!(
        count_final, count_before,
        "Count should return to original after removal"
    );
}

#[tokio::test]
async fn test_turbovec_e2e_index_status() {
    let temp_dir = tempdir().unwrap();
    let kb_root = temp_dir.path().to_str().unwrap().to_string();

    fs::write(temp_dir.path().join("note.md"), "# Test\n\nContent.\n").unwrap();

    // Init + index via CLI
    use std::process::Command;
    let binary = env!("CARGO_BIN_EXE_loom");

    let output = Command::new(binary)
        .args(["init"])
        .env("KB_ROOT", &kb_root)
        .output()
        .unwrap();
    assert!(output.status.success(), "init failed: {:?}", output);

    let output = Command::new(binary)
        .args(["reindex"])
        .env("KB_ROOT", &kb_root)
        .output()
        .unwrap();
    assert!(output.status.success(), "reindex failed: {:?}", output);

    // Verify turbovec files on disk after reindex
    assert!(
        temp_dir
            .path()
            .join(".knowledge-loom-index/turbovec.tvim")
            .exists(),
        "turbovec.tvim should exist after reindex"
    );
    assert!(
        temp_dir
            .path()
            .join(".knowledge-loom-index/turbovec_meta.bin")
            .exists(),
        "turbovec_meta.bin should exist after reindex"
    );

    // Verify old sqlite-vec embeddings.db does NOT exist
    assert!(
        !temp_dir
            .path()
            .join(".knowledge-loom-index/embeddings.db")
            .exists(),
        "Legacy embeddings.db should not exist"
    );
}
