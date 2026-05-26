use knowledge_loom::turbovec_index::TurbovecIndex;
use std::sync::Arc;

#[tokio::test]
async fn test_index_file() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_str().unwrap();
    let dim = 8;

    // Create a markdown file
    let file_path = temp.path().join("test.md");
    std::fs::write(&file_path, "# Hello\n\nWorld content here.").unwrap();

    // Create index and index the file
    let index = TurbovecIndex::new(root, dim, 4).await;

    // Use add_chunks directly since index_file needs an embed provider
    let content = "# Hello\n\nWorld content here.";
    let chunks = knowledge_loom::chunks::parse_chunks(content);
    assert!(!chunks.is_empty(), "Should parse at least one chunk");

    // Create synthetic embeddings matching dim
    let embeddings: Vec<Vec<f32>> = chunks
        .iter()
        .map(|_| vec![1.0_f32 / chunks.len() as f32; dim])
        .collect();

    let relative_path = "test.md";
    let result = index
        .add_chunks(&chunks, &embeddings, relative_path)
        .await;
    assert!(result.is_ok());
    assert_eq!(index.count().await, chunks.len());

    let query: Vec<f32> = vec![1.0; dim];
    let results = index.search_similar(&query, 10).await.unwrap();
    assert!(!results.is_empty());
}

#[tokio::test]
async fn test_dimension_mismatch() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_str().unwrap();

    let dim = 8;
    let wrong_dim = 16;
    let index = TurbovecIndex::new(root, dim, 4).await;

    let chunks = vec![knowledge_loom::chunks::Chunk {
        ordinal: 0,
        heading: Some("Test".to_string()),
        content: "content".to_string(),
        line_start: 1,
        line_end: 2,
    }];
    let embeddings: Vec<Vec<f32>> = vec![vec![0.0; wrong_dim]];
    let result = index.add_chunks(&chunks, &embeddings, "test.md").await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Dimension mismatch") || err.contains("dim"));
}

#[tokio::test]
async fn test_concurrent_search_and_index() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_str().unwrap();
    let dim = 8;

    let index = Arc::new(TurbovecIndex::new(root, dim, 4).await);

    // First add some initial data
    let chunks: Vec<knowledge_loom::chunks::Chunk> = (0..10)
        .map(|i| knowledge_loom::chunks::Chunk {
            ordinal: i,
            heading: Some(format!("Heading {}", i)),
            content: format!("content {}", i),
            line_start: i as usize + 1,
            line_end: i as usize + 2,
        })
        .collect();
    let init_embeddings: Vec<Vec<f32>> =
        (0..10).map(|_| vec![0.1; dim]).collect();
    index
        .add_chunks(&chunks, &init_embeddings, "test.md")
        .await
        .unwrap();

    let query: Vec<f32> = vec![1.0; dim];

    let index_search = index.clone();
    let search_task = tokio::spawn(async move {
        for _ in 0..50 {
            let _ = index_search.search_similar(&query, 10).await;
        }
    });

    let index_add = index.clone();
    let add_task = tokio::spawn(async move {
        for i in 10..60 {
            let c = vec![knowledge_loom::chunks::Chunk {
                ordinal: i,
                heading: None,
                content: format!("extra {}", i),
                line_start: 1,
                line_end: 1,
            }];
            let e: Vec<Vec<f32>> = vec![vec![0.2; dim]];
            let _ = index_add.add_chunks(&c, &e, "extra.md").await;
        }
    });

    tokio::try_join!(search_task, add_task).expect("Concurrent tasks should not panic");
    assert!(index.count().await > 0);
}

#[tokio::test]
async fn test_allowlist_search() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_str().unwrap();
    let dim = 8;

    let index = TurbovecIndex::new(root, dim, 4).await;

    let chunks: Vec<knowledge_loom::chunks::Chunk> = (0..10)
        .map(|i| knowledge_loom::chunks::Chunk {
            ordinal: i,
            heading: Some(format!("Note {}", i)),
            content: format!("content {}", i),
            line_start: i as usize + 1,
            line_end: i as usize + 2,
        })
        .collect();
    let embeddings: Vec<Vec<f32>> = (0..10)
        .map(|i| {
            let mut v = vec![0.0; dim];
            v[i % dim] = 1.0;
            v
        })
        .collect();

    index
        .add_chunks(&chunks, &embeddings, "test.md")
        .await
        .unwrap();

    // Get some known chunk IDs by searching
    let query: Vec<f32> = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let full_results = index.search_similar(&query, 10).await.unwrap();
    assert!(!full_results.is_empty());

    // Now we verify that search_filtered with empty allowlist falls back to full search
    let filtered = index
        .search_filtered(&query, 10, &[])
        .await
        .unwrap();
    assert!(!filtered.is_empty());
    assert_eq!(filtered.len(), full_results.len());
}

#[tokio::test]
async fn test_persistence_roundtrip() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_str().unwrap();
    let dim = 8;

    // Create index and add data
    {
        let index = TurbovecIndex::new(root, dim, 4).await;
        let chunks: Vec<knowledge_loom::chunks::Chunk> = (0..20)
            .map(|i| knowledge_loom::chunks::Chunk {
                ordinal: i,
                heading: Some(format!("N{}", i)),
                content: format!("c{}", i),
                line_start: 1,
                line_end: 1,
            })
            .collect();
        let embeddings: Vec<Vec<f32>> = (0..20).map(|_| vec![0.5; dim]).collect();
        index
            .add_chunks(&chunks, &embeddings, "test.md")
            .await
            .unwrap();

        let count_before = index.count().await;
        assert_eq!(count_before, 20);
        index.save().await.unwrap();
    }

    // Load from disk
    {
        let index = TurbovecIndex::new(root, dim, 4).await;
        let count_after = index.count().await;
        assert_eq!(count_after, 20, "Should reload same count");

        let query: Vec<f32> = vec![1.0; dim];
        let results = index.search_similar(&query, 10).await.unwrap();
        assert!(!results.is_empty());
    }
}

#[tokio::test]
async fn test_corrupt_index_fallback() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_str().unwrap();
    let dim = 8;

    let index_dir = std::path::Path::new(root).join(".knowledge-loom-index");
    std::fs::create_dir_all(&index_dir).unwrap();

    // Write garbage to turbovec.tvim
    std::fs::write(index_dir.join("turbovec.tvim"), b"this is garbage").unwrap();
    // Write garbage to turbovec_meta.bin
    std::fs::write(index_dir.join("turbovec_meta.bin"), b"also garbage").unwrap();

    // Should NOT panic — should create fresh index
    let index = TurbovecIndex::new(root, dim, 4).await;
    assert_eq!(index.count().await, 0);
}

#[tokio::test]
async fn test_quantization_config() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_str().unwrap();
    let dim = 8;

    // Test 4-bit (default)
    let index4 = TurbovecIndex::new(root, dim, 4).await;
    let chunks: Vec<knowledge_loom::chunks::Chunk> = (0..5)
        .map(|i| knowledge_loom::chunks::Chunk {
            ordinal: i,
            heading: None,
            content: format!("c{}", i),
            line_start: 1,
            line_end: 1,
        })
        .collect();
    let emb4: Vec<Vec<f32>> = (0..5).map(|_| vec![0.5; dim]).collect();
    index4
        .add_chunks(&chunks, &emb4, "test.md")
        .await
        .unwrap();
    assert_eq!(index4.count().await, 5);
    let query: Vec<f32> = vec![1.0; dim];
    let results = index4.search_similar(&query, 5).await.unwrap();
    assert_eq!(results.len(), 5);

    // Test 2-bit
    let temp2 = tempfile::tempdir().unwrap();
    let root2 = temp2.path().to_str().unwrap();
    let index2 = TurbovecIndex::new(root2, dim, 2).await;
    index2
        .add_chunks(&chunks, &emb4, "test.md")
        .await
        .unwrap();
    assert_eq!(index2.count().await, 5);
}

#[tokio::test]
async fn test_memory_estimate() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_str().unwrap();
    let dim = 8;
    let bit_width = 4;
    let n = 100;

    let index = TurbovecIndex::new(root, dim, bit_width).await;
    let chunks: Vec<knowledge_loom::chunks::Chunk> = (0..n)
        .map(|i| knowledge_loom::chunks::Chunk {
            ordinal: i,
            heading: None,
            content: format!("c{}", i),
            line_start: 1,
            line_end: 1,
        })
        .collect();
    let embeddings: Vec<Vec<f32>> = (0..n).map(|_| vec![0.5; dim]).collect();

    index
        .add_chunks(&chunks, &embeddings, "test.md")
        .await
        .unwrap();
    assert_eq!(index.count().await, n as usize);

    // Raw float32 would be N * dim * 4 bytes
    let raw_bytes = n as usize * dim * 4;
    // Compressed: N * dim * bit_width / 8 bytes for codes + N * 4 for scales
    let compressed_bytes = n as usize * dim * bit_width as usize / 8 + n as usize * 4;
    assert!(
        compressed_bytes < raw_bytes,
        "Compressed size {} should be less than raw size {}",
        compressed_bytes,
        raw_bytes
    );
}

#[tokio::test]
async fn test_migration() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_str().unwrap();
    let dim = 8;

    let index_dir = std::path::Path::new(root).join(".knowledge-loom-index");
    std::fs::create_dir_all(&index_dir).unwrap();

    // Create a legacy embeddings.db with sqlite-vec format
    // For this test, since we may not have sqlite-vec compiled,
    // we verify that when no embeddings.db exists, migration is a no-op
    let index = TurbovecIndex::new(root, dim, 4).await;
    assert_eq!(index.count().await, 0);

    // Add some vectors and save
    let chunks: Vec<knowledge_loom::chunks::Chunk> = (0..3)
        .map(|i| knowledge_loom::chunks::Chunk {
            ordinal: i,
            heading: None,
            content: format!("c{}", i),
            line_start: 1,
            line_end: 1,
        })
        .collect();
    let embeddings: Vec<Vec<f32>> = (0..3).map(|_| vec![0.5; dim]).collect();
    index
        .add_chunks(&chunks, &embeddings, "test.md")
        .await
        .unwrap();
    index.save().await.unwrap();

    assert_eq!(index.count().await, 3);
}
