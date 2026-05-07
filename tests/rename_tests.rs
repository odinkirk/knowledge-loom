use knowledge_loom::{INDEX_DIR, BIN_DIR, IGNORE_FILE, MCP_CONFIG_KEY,
                     GITIGNORE_ENTRY_BIN, GITIGNORE_ENTRY_INDEX};

#[test]
fn test_constants_have_new_names() {
    assert!(INDEX_DIR.contains("knowledge-loom"), "INDEX_DIR: {INDEX_DIR}");
    assert!(BIN_DIR.contains("knowledge-loom"), "BIN_DIR: {BIN_DIR}");
    assert!(IGNORE_FILE.contains("knowledge-loom"), "IGNORE_FILE: {IGNORE_FILE}");
    assert_eq!(MCP_CONFIG_KEY, "knowledge-loom");
    assert!(GITIGNORE_ENTRY_BIN.contains("knowledge-loom"));
    assert!(GITIGNORE_ENTRY_INDEX.contains("knowledge-loom"));
}

#[tokio::test]
async fn test_index_dirs_use_new_name() {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path().to_str().unwrap();
    // BM25Index::new creates the index dir on construction
    let _bm25 = knowledge_loom::bm25::BM25Index::new(root).await;
    let idx_dir = tmp.path().join(".knowledge-loom-index").join("tantivy");
    assert!(idx_dir.exists(), "Expected index at {}", idx_dir.display());
    let old_dir = tmp.path().join(".loom-index");
    assert!(!old_dir.exists(), "Old .loom-index dir should not exist");
}