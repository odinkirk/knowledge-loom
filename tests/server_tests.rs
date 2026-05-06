use loom::server::LoomServer;
use tempfile::TempDir;

#[tokio::test]
async fn test_list_tools_returns_23_entries() {
    let tmp = TempDir::new().unwrap();
    let kb_root = tmp.path().to_str().unwrap();
    let server = LoomServer::new(kb_root).await;
    let tools = server.tool_list();
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
    assert_eq!(tools.len(), 23, "Expected 23 tools, got {}: {:?}", tools.len(), names);
}

#[tokio::test]
async fn test_list_tools_has_expected_names() {
    let tmp = TempDir::new().unwrap();
    let kb_root = tmp.path().to_str().unwrap();
    let server = LoomServer::new(kb_root).await;
    let tools = server.tool_list();
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
    for expected in &[
        "loom_search", "loom_list_files", "loom_outline",
        "loom_grep", "loom_read_section", "loom_read_lines",
        "loom_replace_lines", "loom_insert_after_heading", "loom_append_to_file",
        "loom_reindex", "loom_index_status",
    ] {
        assert!(names.contains(expected), "Missing tool: {expected}");
    }
}


#[tokio::test]
async fn test_call_loom_list_files() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_str().unwrap();
    std::fs::write(tmp.path().join("note.md"), "# Test\nHello").unwrap();
    let server = LoomServer::new(root).await;
    let result = server.dispatch_tool("loom_list_files", &serde_json::json!({})).await;
    assert!(result.is_ok(), "loom_list_files failed: {:?}", result);
}

#[tokio::test]
async fn test_call_loom_index_status() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_str().unwrap();
    let server = LoomServer::new(root).await;
    let result = server.dispatch_tool("loom_index_status", &serde_json::json!({})).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_call_loom_outline() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_str().unwrap();
    let note = tmp.path().join("note.md");
    std::fs::write(&note, "# Heading One\n\nContent\n\n## Sub\n\nMore\n").unwrap();
    let server = LoomServer::new(root).await;
    let result = server.dispatch_tool("loom_outline", &serde_json::json!({"file": "note.md"})).await;
    assert!(result.is_ok());
    let text = result.unwrap();
    assert!(text.contains("Heading One"));
}

#[tokio::test]
async fn test_call_loom_append_to_file() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_str().unwrap();
    let note = tmp.path().join("note.md");
    std::fs::write(&note, "# Existing\n").unwrap();
    let server = LoomServer::new(root).await;
    let result = server.dispatch_tool("loom_append_to_file", &serde_json::json!({
        "file": "note.md",
        "content": "New line"
    })).await;
    assert!(result.is_ok());
    let content = std::fs::read_to_string(&note).unwrap();
    assert!(content.contains("New line"));
}

#[tokio::test]
async fn test_call_unknown_tool_returns_error() {
    let tmp = TempDir::new().unwrap();
    let server = LoomServer::new(tmp.path().to_str().unwrap()).await;
    let result = server.dispatch_tool("loom_does_not_exist", &serde_json::json!({})).await;
    assert!(result.is_err());
}
