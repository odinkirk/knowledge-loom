fn main() {
    let models_dir = std::path::Path::new(
        "/Users/odinkirk/Documents/Claude/Projects/unspoken-world/.knowledge-loom-index/models",
    );
    let provider = knowledge_loom::embed::LocalEmbedProvider::new(models_dir);

    // Warm up
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _ = rt.block_on(provider.embed("warmup"));

    // Test 1: single-text embed (32 times - old per-chunk approach)
    let text = "test chunk with some content for performance measurement".repeat(10);
    let start = std::time::Instant::now();
    for _ in 0..32 {
        let _ = rt.block_on(provider.embed(&text));
    }
    println!("32x single embed: {:?}", start.elapsed());

    // Test 2: batch embed 32 texts (new approach)
    let texts: Vec<String> = (0..32).map(|_| text.clone()).collect();
    let start = std::time::Instant::now();
    let _ = rt.block_on(provider.embed_batch(&texts));
    println!("1x batch(32): {:?}", start.elapsed());

    // Test 3: batch embed 100 texts
    let texts: Vec<String> = (0..100).map(|i| format!("chunk {}", i)).collect();
    let start = std::time::Instant::now();
    let r = rt.block_on(provider.embed_batch(&texts));
    println!(
        "1x batch(100): {:?} -> {} embeddings",
        start.elapsed(),
        r.unwrap().len()
    );
}
