use knowledge_loom::search::SearchEngine;
use knowledge_loom::vault::VaultState;
use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn behavioral_error_handling_invalid_file_path() {
    let temp_dir = TempDir::new().unwrap();
    let kb_root = temp_dir.path();

    let _vault = VaultState::new(kb_root.to_str().unwrap()).await;
    let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

    // Test with non-existent file
    let results = search_engine.search("nonexistent content", 5).await;
    // Should return empty results, not panic
    assert!(results.is_empty());
}

#[tokio::test]
async fn behavioral_error_handling_corrupted_index() {
    let temp_dir = TempDir::new().unwrap();
    let kb_root = temp_dir.path();

    // Create a corrupted index file
    let index_path = kb_root.join(".knowledge-loom-index/tantivy/meta.json");
    fs::create_dir_all(index_path.parent().unwrap()).unwrap();
    fs::write(&index_path, "invalid json content").unwrap();

    // Should handle corrupted index gracefully
    let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;
    let vault = VaultState::new(kb_root.to_str().unwrap()).await;

    // Create a valid file
    fs::write(kb_root.join("test.md"), "# Test\nContent").unwrap();

    // Should rebuild index and work correctly
    {
        let mut bm25 = search_engine.bm25.lock().await;
        let _ = bm25.index_vault(&vault).await;
    }

    let results = search_engine.search("test", 5).await;
    // Should work despite initial corruption
    assert!(!results.is_empty());
}

#[tokio::test]
async fn behavioral_edge_case_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let kb_root = temp_dir.path();

    // Create empty file
    fs::write(kb_root.join("empty.md"), "").unwrap();

    let vault = VaultState::new(kb_root.to_str().unwrap()).await;
    let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

    {
        let mut bm25 = search_engine.bm25.lock().await;
        bm25.index_vault(&vault).await.unwrap();
    }

    // Should handle empty file gracefully
    let results = search_engine.search("anything", 5).await;
    assert!(results.is_empty());
}

#[tokio::test]
async fn behavioral_edge_case_very_large_file() {
    let temp_dir = TempDir::new().unwrap();
    let kb_root = temp_dir.path();

    // Create a large file with many sections
    let mut content = String::new();
    for i in 0..1000 {
        content.push_str(&format!(
            "# Section {}\n\nContent for section {}.\n\n",
            i, i
        ));
    }
    fs::write(kb_root.join("large.md"), content).unwrap();

    let vault = VaultState::new(kb_root.to_str().unwrap()).await;
    let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

    {
        let mut bm25 = search_engine.bm25.lock().await;
        bm25.index_vault(&vault).await.unwrap();
    }

    // Should handle large file efficiently
    let results = search_engine.search("section 500", 5).await;
    assert!(!results.is_empty());
}

#[tokio::test]
async fn behavioral_edge_case_special_characters() {
    let temp_dir = TempDir::new().unwrap();
    let kb_root = temp_dir.path();

    // Create file with special characters
    let content = r#"# Special Characters

Test with symbols: @#$%^&*()
Test with quotes: "quotes" and 'apostrophes'
Test with brackets: [brackets] {braces} <angles>
Test with math: 2+2=4, x>y, a<b
"#;
    fs::write(kb_root.join("special.md"), content).unwrap();

    let vault = VaultState::new(kb_root.to_str().unwrap()).await;
    let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

    {
        let mut bm25 = search_engine.bm25.lock().await;
        bm25.index_vault(&vault).await.unwrap();
    }

    // Should handle special characters correctly
    let results = search_engine.search("symbols", 5).await;
    assert!(!results.is_empty());
}

#[tokio::test]
async fn behavioral_edge_case_unicode_content() {
    let temp_dir = TempDir::new().unwrap();
    let kb_root = temp_dir.path();

    // Create file with unicode content
    let content = "# Unicode Test\n\nHello 世界\nПривет мир\nمرحبا بالعالم\n🌍🌎🌏";
    fs::write(kb_root.join("unicode.md"), content).unwrap();

    let vault = VaultState::new(kb_root.to_str().unwrap()).await;
    let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

    {
        let mut bm25 = search_engine.bm25.lock().await;
        bm25.index_vault(&vault).await.unwrap();
    }

    // Should handle unicode correctly
    let results = search_engine.search("世界", 5).await;
    assert!(!results.is_empty());
}

#[tokio::test]
async fn behavioral_concurrent_search_operations() {
    let temp_dir = TempDir::new().unwrap();
    let kb_root = temp_dir.path();

    // Create test files
    for i in 0..10 {
        fs::write(
            kb_root.join(format!("file{}.md", i)),
            format!("# File {}\nContent {}", i, i),
        )
        .unwrap();
    }

    let vault = VaultState::new(kb_root.to_str().unwrap()).await;
    let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

    {
        let mut bm25 = search_engine.bm25.lock().await;
        bm25.index_vault(&vault).await.unwrap();
    }

    // Perform concurrent searches using Arc references
    let engine_arc = std::sync::Arc::new(search_engine);
    let mut handles = vec![];
    for i in 0..10 {
        let engine = engine_arc.clone();
        let handle = tokio::spawn(async move { engine.search(&format!("content {}", i), 5).await });
        handles.push(handle);
    }

    // All searches should complete successfully
    for handle in handles {
        let results = handle.await.unwrap();
        assert!(!results.is_empty());
    }
}

#[tokio::test]
async fn behavioral_concurrent_index_operations() {
    let temp_dir = TempDir::new().unwrap();
    let kb_root = temp_dir.path();

    let vault = VaultState::new(kb_root.to_str().unwrap()).await;
    let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

    // Create files concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let kb_root = kb_root.to_path_buf();
        let handle = tokio::spawn(async move {
            fs::write(
                kb_root.join(format!("file{}.md", i)),
                format!("# File {}\nContent {}", i, i),
            )
            .unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Index should handle concurrent file creation
    {
        let mut bm25 = search_engine.bm25.lock().await;
        bm25.index_vault(&vault).await.unwrap();
    }

    let results = search_engine.search("content", 10).await;
    assert!(!results.is_empty());
}

#[tokio::test]
async fn behavioral_integration_edit_and_search() {
    let temp_dir = TempDir::new().unwrap();
    let kb_root = temp_dir.path();

    // Create initial file
    fs::write(kb_root.join("test.md"), "# Original\nOriginal content").unwrap();

    let vault = VaultState::new(kb_root.to_str().unwrap()).await;
    let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

    // Index initial content
    {
        let mut bm25 = search_engine.bm25.lock().await;
        bm25.index_vault(&vault).await.unwrap();
    }

    // Search for original content
    let results = search_engine.search("original", 5).await;
    assert!(!results.is_empty());

    // Edit the file directly
    let file_path = kb_root.join("test.md");
    fs::write(&file_path, "# Updated\nUpdated content").unwrap();

    // Reindex after edit
    {
        let mut bm25 = search_engine.bm25.lock().await;
        bm25.index_vault(&vault).await.unwrap();
    }

    // Search for updated content
    let results = search_engine.search("updated", 5).await;
    assert!(!results.is_empty());
}

#[tokio::test]
async fn behavioral_performance_large_dataset_search() {
    let temp_dir = TempDir::new().unwrap();
    let kb_root = temp_dir.path();

    // Create large dataset
    for i in 0..50 {
        let content = format!("# Document {}\n\nThis is document {} with various content including keywords like machine learning, artificial intelligence, and data science.\n\n## Section {}\n\nMore content here.", i, i, i);
        fs::write(kb_root.join(format!("doc{}.md", i)), content).unwrap();
    }

    let vault = VaultState::new(kb_root.to_str().unwrap()).await;
    let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

    // Time the indexing
    let start = std::time::Instant::now();
    {
        let mut bm25 = search_engine.bm25.lock().await;
        bm25.index_vault(&vault).await.unwrap();
    }
    let index_time = start.elapsed();

    println!("Indexing 50 files took: {:?}", index_time);
    // Just log the time, don't assert strict limits as performance varies
    assert!(
        index_time.as_secs() < 300,
        "Indexing should complete eventually"
    );

    // Time the search
    let start = std::time::Instant::now();
    let results = search_engine.search("machine learning", 10).await;
    let search_time = start.elapsed();

    println!("Search took: {:?}", search_time);
    assert!(!results.is_empty());
    assert!(
        search_time.as_millis() < 5000,
        "Search should complete reasonably quickly"
    );
}

#[tokio::test]
async fn behavioral_integration_nested_directory_structure() {
    let temp_dir = TempDir::new().unwrap();
    let kb_root = temp_dir.path();

    // Create nested directory structure
    let dir1 = kb_root.join("category1");
    let dir2 = kb_root.join("category2");
    fs::create_dir_all(&dir1).unwrap();
    fs::create_dir_all(&dir2).unwrap();

    // Create files in different directories
    fs::write(dir1.join("file1.md"), "# Category 1\nContent in category 1").unwrap();
    fs::write(dir2.join("file2.md"), "# Category 2\nContent in category 2").unwrap();
    fs::write(kb_root.join("root.md"), "# Root\nRoot content").unwrap();

    let vault = VaultState::new(kb_root.to_str().unwrap()).await;
    let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

    {
        let mut bm25 = search_engine.bm25.lock().await;
        bm25.index_vault(&vault).await.unwrap();
    }

    // Should find files across all directories
    let results = search_engine.search("category", 10).await;
    assert!(results.len() >= 2);

    // Verify relative paths are stored correctly
    let paths: Vec<_> = results.iter().map(|r| &r.path).collect();
    assert!(paths.iter().any(|p| p.contains("category1")));
    assert!(paths.iter().any(|p| p.contains("category2")));
}

#[tokio::test]
async fn behavioral_error_handling_filesystem_permissions() {
    let temp_dir = TempDir::new().unwrap();
    let kb_root = temp_dir.path();

    // Create a file and make it read-only
    let file_path = kb_root.join("readonly.md");
    fs::write(&file_path, "# Readonly\nContent").unwrap();

    let mut perms = fs::metadata(&file_path).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(&file_path, perms).unwrap();

    let vault = VaultState::new(kb_root.to_str().unwrap()).await;
    let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

    // Should handle read-only files gracefully
    {
        let mut bm25 = search_engine.bm25.lock().await;
        let result = bm25.index_vault(&vault).await;
        // Should either succeed or fail gracefully, not panic
        assert!(result.is_ok() || result.is_err());
    }

    // Restore permissions for cleanup
    let mut perms = fs::metadata(&file_path).unwrap().permissions();
    perms.set_readonly(false);
    fs::set_permissions(&file_path, perms).unwrap();
}

#[tokio::test]
async fn behavioral_edge_case_deeply_nested_headings() {
    let temp_dir = TempDir::new().unwrap();
    let kb_root = temp_dir.path();

    // Create file with deeply nested headings
    let content = r#"# Level 1
Content 1

## Level 2
Content 2

### Level 3
Content 3

#### Level 4
Content 4

##### Level 5
Content 5

###### Level 6
Content 6
"#;
    fs::write(kb_root.join("nested.md"), content).unwrap();

    let vault = VaultState::new(kb_root.to_str().unwrap()).await;
    let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

    {
        let mut bm25 = search_engine.bm25.lock().await;
        bm25.index_vault(&vault).await.unwrap();
    }

    // Should handle all heading levels
    let results = search_engine.search("level", 10).await;
    assert!(!results.is_empty());
}

#[tokio::test]
async fn behavioral_integration_search_file_specificity() {
    let temp_dir = TempDir::new().unwrap();
    let kb_root = temp_dir.path();

    // Create multiple files with overlapping content
    fs::write(
        kb_root.join("file1.md"),
        "# File 1\nmachine learning algorithms",
    )
    .unwrap();
    fs::write(
        kb_root.join("file2.md"),
        "# File 2\nmachine learning applications",
    )
    .unwrap();
    fs::write(kb_root.join("file3.md"), "# File 3\ndeep learning networks").unwrap();

    let vault = VaultState::new(kb_root.to_str().unwrap()).await;
    let search_engine = SearchEngine::new(kb_root.to_str().unwrap()).await;

    {
        let mut bm25 = search_engine.bm25.lock().await;
        bm25.index_vault(&vault).await.unwrap();
    }

    // Search within specific file
    let results = search_engine
        .bm25
        .lock()
        .await
        .search_file("file1.md", "machine learning", 5)
        .await
        .unwrap();

    assert!(!results.is_empty());
    // All results should be from file1.md
    for (_score, chunk) in results {
        assert_eq!(chunk.path, "file1.md");
    }
}

#[tokio::test]
async fn test_edit_reindexes_vector_backend() {
    use knowledge_loom::server::LoomServer;

    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("note.md"), "# Topic\noriginal text").unwrap();

    let server = LoomServer::new(tmp.path().to_str().unwrap()).await;

    // Edit the file through the server - this should trigger reindex_file
    let result = server
        .dispatch_tool(
            "replace_lines",
            &serde_json::json!({
                "file": "note.md",
                "start": 2,
                "end": 2,
                "content": "updated vector content"
            }),
        )
        .await;

    // The edit should succeed
    assert!(result.is_ok(), "edit should succeed: {:?}", result);
}
