use knowledge_loom::{
    BIN_DIR, GITIGNORE_ENTRY_BIN, GITIGNORE_ENTRY_INDEX, IGNORE_FILE, INDEX_DIR, MCP_CONFIG_KEY,
};

#[test]
fn test_constants_have_new_names() {
    assert!(
        INDEX_DIR.contains("knowledge-loom"),
        "INDEX_DIR: {INDEX_DIR}"
    );
    assert!(BIN_DIR.contains("knowledge-loom"), "BIN_DIR: {BIN_DIR}");
    assert!(
        IGNORE_FILE.contains("knowledge-loom"),
        "IGNORE_FILE: {IGNORE_FILE}"
    );
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

#[test]
fn test_init_uses_new_paths() {
    let tmp = tempfile::TempDir::new().unwrap();
    let dir = tmp.path();
    let bin = dir.join("fake_loom");
    std::fs::write(&bin, b"#!/bin/sh\necho loom").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut p = std::fs::metadata(&bin).unwrap().permissions();
        p.set_mode(0o755);
        std::fs::set_permissions(&bin, p).unwrap();
    }
    knowledge_loom::init::run_init_with_binary(&dir.to_path_buf(), &bin).unwrap();

    // Binary in new dir
    assert!(dir.join(".knowledge-loom/bin/loom").exists());
    // Old dir must NOT exist
    assert!(!dir.join(".loom").exists());

    // .gitignore entries
    let gi = std::fs::read_to_string(dir.join(".gitignore")).unwrap();
    assert!(gi.contains(".knowledge-loom/"), "gitignore bin: {gi}");
    assert!(
        gi.contains(".knowledge-loom-index/"),
        "gitignore index: {gi}"
    );
    assert!(!gi.contains(".loom/"), "old entry present: {gi}");

    // MCP config key
    let mcp: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join(".mcp.json")).unwrap()).unwrap();
    assert!(mcp["mcpServers"]["knowledge-loom"]["command"].is_string());
    assert!(
        mcp["mcpServers"].get("loom").is_none(),
        "old key still present"
    );
}
