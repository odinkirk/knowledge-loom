// Performance benchmarks for embedding providers
// These benchmarks measure the performance of different embedding providers

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use knowledge_loom::embed::{LocalEmbedProvider, OllamaEmbedProvider, OpenRouterEmbedProvider};
use std::path::PathBuf;

/// Benchmark local embedding provider
fn bench_local_provider(c: &mut Criterion) {
    let models_dir = PathBuf::from(".knowledge-loom-index/models");
    let provider = LocalEmbedProvider::new(&models_dir);
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("local_provider");

    // Benchmark different text lengths
    for text_len in [10, 50, 100, 500, 1000].iter() {
        let text = "a".repeat(*text_len);
        let text_clone = text.clone();
        group.bench_with_input(BenchmarkId::from_parameter(text_len), text_len, |b, _| {
            b.iter(|| {
                let text = text_clone.clone();
                rt.block_on(async { provider.embed(black_box(&text)).await.unwrap() })
            })
        });
    }

    group.finish();
}

/// Benchmark Ollama embedding provider
fn bench_ollama_provider(c: &mut Criterion) {
    let provider = OllamaEmbedProvider::new("http://localhost:11434".to_string());
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("ollama_provider");

    // Benchmark different text lengths
    for text_len in [10, 50, 100, 500, 1000].iter() {
        let text = "a".repeat(*text_len);
        let text_clone = text.clone();
        group.bench_with_input(BenchmarkId::from_parameter(text_len), text_len, |b, _| {
            b.iter(|| {
                let text = text_clone.clone();
                rt.block_on(async { provider.embed(black_box(&text)).await.unwrap() })
            })
        });
    }

    group.finish();
}

/// Benchmark OpenRouter embedding provider
fn bench_openrouter_provider(c: &mut Criterion) {
    let provider = OpenRouterEmbedProvider::new("test-key", "openai/text-embedding-ada-002");
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("openrouter_provider");

    // Benchmark different text lengths
    for text_len in [10, 50, 100, 500, 1000].iter() {
        let text = "a".repeat(*text_len);
        let text_clone = text.clone();
        group.bench_with_input(BenchmarkId::from_parameter(text_len), text_len, |b, _| {
            b.iter(|| {
                let text = text_clone.clone();
                rt.block_on(async { provider.embed(black_box(&text)).await.unwrap() })
            })
        });
    }

    group.finish();
}

/// Benchmark embedding consistency
fn bench_embedding_consistency(c: &mut Criterion) {
    let models_dir = PathBuf::from(".knowledge-loom-index/models");
    let provider = LocalEmbedProvider::new(&models_dir);
    let text = "This is a test text for embedding consistency benchmarking.";
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("embedding_consistency", |b| {
        b.iter(|| {
            let text = text.clone();
            let embedding1 = rt.block_on(async { provider.embed(black_box(&text)).await.unwrap() });
            let embedding2 = rt.block_on(async { provider.embed(black_box(&text)).await.unwrap() });
            // Verify consistency
            assert_eq!(embedding1, embedding2, "Embeddings should be consistent");
        })
    });
}

/// Benchmark batch embedding
fn bench_batch_embedding(c: &mut Criterion) {
    let models_dir = PathBuf::from(".knowledge-loom-index/models");
    let provider = LocalEmbedProvider::new(&models_dir);
    let rt = tokio::runtime::Runtime::new().unwrap();

    let texts: Vec<String> = (0..100)
        .map(|i| format!("Test text number {}", i))
        .collect();

    c.bench_function("batch_embedding_100", |b| {
        b.iter(|| {
            for text in &texts {
                let text = text.clone();
                black_box(rt.block_on(async { provider.embed(black_box(&text)).await }));
            }
        })
    });
}

criterion_group!(
    benches,
    bench_local_provider,
    bench_ollama_provider,
    bench_openrouter_provider,
    bench_embedding_consistency,
    bench_batch_embedding
);
criterion_main!(benches);
