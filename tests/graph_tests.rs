use std::fs;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex;

use knowledge_loom::graph::GraphState;
use knowledge_loom::vault::VaultState;

#[tokio::test]
async fn test_extract_wikilinks_double_bracket() {
    let temp_dir = TempDir::new().unwrap();
    let kb_root = temp_dir.path().to_str().unwrap();

    let content = "See [[other-note]] for details.\nAlso [[another|with alias]] here.\n";
    let path = temp_dir.path().join("source.md");
    fs::write(&path, content).unwrap();
    // Target files must exist for graph edges to resolve
    fs::write(temp_dir.path().join("other-note.md"), "# Other\n").unwrap();
    fs::write(temp_dir.path().join("another.md"), "# Another\n").unwrap();

    let vault = Arc::new(Mutex::new(VaultState::new(kb_root).await));
    let graph = GraphState::new(kb_root).await;
    graph.build_graph(&*vault.lock().await).await.unwrap();

    let edges = graph.graph.lock().await.edge_count();
    assert!(
        edges > 0,
        "should find wikilinks and create edges, got {} edges",
        edges
    );
}

#[tokio::test]
async fn test_extract_wikilinks_markdown_format() {
    let temp_dir = TempDir::new().unwrap();
    let kb_root = temp_dir.path().to_str().unwrap();

    let content = "See [the other file](other.md) for more.\nAlso check [external](https://example.com) link.\n";
    let path = temp_dir.path().join("source.md");
    fs::write(&path, content).unwrap();
    // Create the target file so it exists for graph node lookup
    let target_path = temp_dir.path().join("other.md");
    fs::write(&target_path, "# Other\n\nContent.\n").unwrap();

    let vault = Arc::new(Mutex::new(VaultState::new(kb_root).await));
    let graph = GraphState::new(kb_root).await;
    graph.build_graph(&*vault.lock().await).await.unwrap();

    let edges = graph.graph.lock().await.edge_count();
    assert!(
        edges > 0,
        "should extract [text](path.md) links, got {} edges",
        edges
    );
}

#[tokio::test]
async fn test_graph_edges_from_test_vault() {
    // Reindex test-vault (has [[wikilinks]]) and verify edge_count > 0
    let kb_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-vault");
    let kb_root_str = kb_root.to_str().unwrap();

    let graph = GraphState::new(kb_root_str).await;
    let vault = Arc::new(Mutex::new(VaultState::new(kb_root_str).await));
    graph.build_graph(&*vault.lock().await).await.unwrap();

    let edges = graph.graph.lock().await.edge_count();
    assert!(
        edges > 0,
        "test-vault with wikilinks should produce edges, got {} edges",
        edges
    );
}
