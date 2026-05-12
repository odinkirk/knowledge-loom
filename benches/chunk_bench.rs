// Benchmarks for chunk operations
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use knowledge_loom::chunks;

fn bench_truncate_at_whitespace(c: &mut Criterion) {
    let content = "A".repeat(3000);

    c.bench_function("truncate_at_whitespace", |b| {
        b.iter(|| chunks::truncate_at_whitespace(black_box(&content), black_box(2000)))
    });
}

fn bench_parse_chunks(c: &mut Criterion) {
    let content = "# Heading\n\n".to_string() + &"A".repeat(10000);

    c.bench_function("parse_chunks", |b| {
        b.iter(|| chunks::parse_chunks(black_box(&content)))
    });
}

fn bench_parse_chunks_large(c: &mut Criterion) {
    let mut content = String::new();
    for i in 1..=100 {
        content.push_str(&format!(
            "# Section {}\n\nContent for section {}.\n\n",
            i, i
        ));
    }

    c.bench_function("parse_chunks_large", |b| {
        b.iter(|| chunks::parse_chunks(black_box(&content)))
    });
}

criterion_group!(
    benches,
    bench_truncate_at_whitespace,
    bench_parse_chunks,
    bench_parse_chunks_large
);
criterion_main!(benches);
