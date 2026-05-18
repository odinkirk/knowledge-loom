use knowledge_loom::embed::LocalEmbedProvider;
use std::path::PathBuf;

fn provider() -> LocalEmbedProvider {
    LocalEmbedProvider::new(&PathBuf::from(".knowledge-loom-index/models"))
}

#[tokio::test]
async fn test_embed_batch_single_text_matches_single_embed() {
    let p = provider();
    let text = "test text for batch consistency".to_string();
    let single = p.embed(&text).await.unwrap();
    let batch = p.embed_batch(&[text.clone()]).await.unwrap();
    assert_eq!(batch.len(), 1);
    assert_eq!(batch[0], single);
}

#[tokio::test]
async fn test_embed_batch_32_texts_produces_32_embeddings() {
    let p = provider();
    let texts: Vec<String> = (0..32).map(|i| format!("batch test chunk {}", i)).collect();
    let results = p.embed_batch(&texts).await.unwrap();
    assert_eq!(results.len(), 32);
    for emb in &results {
        assert_eq!(emb.len(), 384);
    }
}

#[tokio::test]
async fn test_embed_batch_empty_returns_empty() {
    let p = provider();
    let results = p.embed_batch(&[]).await.unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_embed_batch_empty_strings_no_panic() {
    let p = provider();
    let texts: Vec<String> = vec!["".to_string(), "valid text".to_string(), "".to_string()];
    let results = p.embed_batch(&texts).await.unwrap();
    assert_eq!(results.len(), 3);
    // Empty strings should still produce valid embeddings
    for emb in &results {
        assert_eq!(emb.len(), 384);
    }
}

#[tokio::test]
async fn test_embed_batch_ordinal_consistency() {
    // Same text in different positions should get same embedding
    let p = provider();
    let a = "consistent test text alpha".to_string();
    let b = "consistent test text beta".to_string();
    let results = p
        .embed_batch(&[a.clone(), b.clone(), a.clone()])
        .await
        .unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(
        results[0], results[2],
        "same text should produce same embedding"
    );
    assert_ne!(results[0], results[1], "different texts should differ");
}
