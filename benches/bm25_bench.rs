// Benchmarks for BM25 operations
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tempfile::TempDir;

fn bench_get_chunk_by_ordinal(c: &mut Criterion) {
    let tmp = TempDir::new().unwrap();
    let kb_root = tmp.path().to_str().unwrap().to_string();

    // Create test file
    let test_file = tmp.path().join("test.md");
    let content = "# Section A\n\nContent A.\n\n# Section B\n\nContent B.";
    std::fs::write(&test_file, content).unwrap();

    // Create index and setup in a separate runtime
    let index = std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut index = knowledge_loom::bm25::BM25Index::new(&kb_root).await;
            index.index_file(&test_file, content).await.unwrap();
            {
                let mut writer = index.writer.lock().await;
                writer.commit().unwrap();
            }
            index
        })
    })
    .join()
    .unwrap();

    c.bench_function("get_chunk_by_ordinal", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                index
                    .get_chunk_by_ordinal(black_box("test.md"), black_box(1))
                    .await
            })
        })
    });
}

fn bench_file_reindexing(c: &mut Criterion) {
    let tmp = TempDir::new().unwrap();
    let kb_root = tmp.path();

    // Create test file
    let test_file = kb_root.join("test.md");
    let content = "# Section A\n\nContent A.\n\n# Section B\n\nContent B.";
    std::fs::write(&test_file, content).unwrap();

    c.bench_function("file_reindexing", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut index =
                    knowledge_loom::bm25::BM25Index::new(kb_root.to_str().unwrap()).await;
                index
                    .index_file(black_box(&test_file), black_box(content))
                    .await
            })
        })
    });
}

fn bench_corpus_reingestion(c: &mut Criterion) {
    let tmp = TempDir::new().unwrap();
    let kb_root = tmp.path().to_str().unwrap().to_string();

    // Create test files
    for i in 1..=10 {
        let test_file = tmp.path().join(&format!("test{}.md", i));
        let content = format!("# Section {}\n\nContent {}.", i, i);
        std::fs::write(&test_file, content).unwrap();
    }

    // Create vault state in a separate runtime
    let kb_root_clone = kb_root.clone();
    let vault_state = std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async { knowledge_loom::vault::VaultState::new(&kb_root_clone).await })
    })
    .join()
    .unwrap();

    c.bench_function("corpus_reingestion", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut index = knowledge_loom::bm25::BM25Index::new(&kb_root).await;
                index.index_vault(&vault_state).await
            })
        })
    });
}

criterion_group!(
    benches,
    bench_get_chunk_by_ordinal,
    bench_file_reindexing,
    bench_corpus_reingestion
);
criterion_main!(benches);
