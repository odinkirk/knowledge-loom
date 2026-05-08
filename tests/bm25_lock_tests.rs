use std::fs;
use tempfile::TempDir;
use knowledge_loom::bm25::BM25Index;

#[tokio::test]
async fn test_bm25_stale_lock_recovery() {
    let temp_dir = TempDir::new().unwrap();
    let kb_root = temp_dir.path().to_str().unwrap();

    // Create initial index
    let mut index1 = BM25Index::new(kb_root).await;

    // Add a document
    let test_path = temp_dir.path().join("test.md");
    index1.index_file(&test_path, "# Test\n\nTest content").await.unwrap();

    // Commit to make searchable
    {
        let mut writer = index1.writer.lock().await;
        writer.commit().unwrap();
    }

    // Simulate stale lock by creating a lock file
    let lock_path = temp_dir.path().join(".knowledge-loom-index").join("tantivy").join(".tantivy-writer.lock");
    fs::write(&lock_path, "stale lock").unwrap();

    // Create a new index instance - should recover from stale lock
    let mut index2 = BM25Index::new(kb_root).await;

    // Verify we can write to the recovered index
    let test_path2 = temp_dir.path().join("test2.md");
    index2.index_file(&test_path2, "# Test2\n\nTest content 2").await.unwrap();

    // Commit to make searchable
    {
        let mut writer = index2.writer.lock().await;
        writer.commit().unwrap();
    }

    // Verify we can search
    let results = index2.search("Test", 10).await.unwrap();
    assert!(!results.is_empty(), "Should find results after lock recovery");
}

#[tokio::test]
async fn test_bm25_new_does_not_delete_live_lock() {
    // If a valid index exists with a released writer, opening a second
    // BM25Index should succeed without touching any lock file.
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("a.md"), "# A\ncontent").unwrap();

    // First index — creates and releases the writer
    let mut idx = BM25Index::new(tmp.path().to_str().unwrap()).await;
    idx.index_file(tmp.path().join("a.md").as_path(), "# A\ncontent")
        .await
        .expect("first index");
    drop(idx); // writer lock released

    // Second index — should open cleanly without lock deletion
    let idx2 = BM25Index::new(tmp.path().to_str().unwrap()).await;
    // Verify it can search (proves the index is intact)
    let results = idx2.search("content", 5).await.expect("search failed");
    assert!(!results.is_empty(), "index was corrupted by lock handling");
}
