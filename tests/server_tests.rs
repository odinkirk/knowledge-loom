use knowledge_loom::server::LoomServer;
use tempfile::TempDir;
use rmcp::handler::server::ServerHandler;

#[tokio::test]
async fn test_list_tools_returns_23_entries() {
    let tmp = TempDir::new().unwrap();
    let kb_root = tmp.path().to_str().unwrap();
    let server = LoomServer::new(kb_root).await;
    let tools = server.tool_list();
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
    assert_eq!(tools.len(), 24, "Expected 24 tools, got {}: {:?}", tools.len(), names);
}

#[tokio::test]
async fn test_list_tools_has_expected_names() {
    let tmp = TempDir::new().unwrap();
    let kb_root = tmp.path().to_str().unwrap();
    let server = LoomServer::new(kb_root).await;
    let tools = server.tool_list();
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
    for expected in &[
        "search", "list_files", "outline",
        "grep", "read_section", "read_lines",
        "replace_lines", "insert_after_heading", "append_to_file",
        "reindex", "index_status", "search_file", "search_graph",
        "search_smart", "rank_notes", "find_connections", "find_path_between",
        "detect_themes", "create_note", "edit_note", "apply_edit_preview",
        "link_notes", "move_note", "delete_note"
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
    let result = server.dispatch_tool("list_files", &serde_json::json!({})).await;
    assert!(result.is_ok(), "list_files failed: {:?}", result);
}

#[tokio::test]
async fn test_call_loom_index_status() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_str().unwrap();
    let server = LoomServer::new(root).await;
    let result = server.dispatch_tool("index_status", &serde_json::json!({})).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_call_loom_outline() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_str().unwrap();
    let note = tmp.path().join("note.md");
    std::fs::write(&note, "# Heading One\n\nContent\n\n## Sub\n\nMore\n").unwrap();
    let server = LoomServer::new(root).await;
    let result = server.dispatch_tool("outline", &serde_json::json!({"file": "note.md"})).await;
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
    let result = server.dispatch_tool("append_to_file", &serde_json::json!({
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

#[tokio::test]
async fn test_dispatch_tool_missing_required_argument() {
    let tmp = TempDir::new().unwrap();
    let server = LoomServer::new(tmp.path().to_str().unwrap()).await;
    let result = server.dispatch_tool("search", &serde_json::json!({})).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("missing required argument"));
}

#[tokio::test]
async fn test_dispatch_tool_search_with_valid_args() {
    let tmp = TempDir::new().unwrap();
    let server = LoomServer::new(tmp.path().to_str().unwrap()).await;
    let result = server.dispatch_tool("search", &serde_json::json!({"query": "test"})).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_tool_search_file_with_valid_args() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_str().unwrap();
    let note = tmp.path().join("note.md");
    std::fs::write(&note, "# Test\nHello world").unwrap();
    let server = LoomServer::new(root).await;
    let result = server.dispatch_tool("search_file", &serde_json::json!({"file": "note.md", "query": "hello"})).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_tool_grep_with_pattern() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_str().unwrap();
    let note = tmp.path().join("note.md");
    std::fs::write(&note, "# Test\nHello world").unwrap();
    let server = LoomServer::new(root).await;
    let result = server.dispatch_tool("grep", &serde_json::json!({"pattern": "Hello"})).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_tool_read_section() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_str().unwrap();
    let note = tmp.path().join("note.md");
    std::fs::write(&note, "# Test\nContent here").unwrap();
    let server = LoomServer::new(root).await;
    let result = server.dispatch_tool("read_section", &serde_json::json!({"file": "note.md", "heading": "Test"})).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_tool_read_lines() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_str().unwrap();
    let note = tmp.path().join("note.md");
    std::fs::write(&note, "# Test\nLine 1\nLine 2").unwrap();
    let server = LoomServer::new(root).await;
    let result = server.dispatch_tool("read_lines", &serde_json::json!({"file": "note.md", "start": 1, "end": 2})).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_tool_replace_lines() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_str().unwrap();
    let note = tmp.path().join("note.md");
    std::fs::write(&note, "# Test\nLine 1\nLine 2").unwrap();
    let server = LoomServer::new(root).await;
    let result = server.dispatch_tool("replace_lines", &serde_json::json!({"file": "note.md", "start": 2, "end": 2, "content": "New line"})).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_tool_insert_after_heading() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_str().unwrap();
    let note = tmp.path().join("note.md");
    std::fs::write(&note, "# Test\nContent").unwrap();
    let server = LoomServer::new(root).await;
    let result = server.dispatch_tool("insert_after_heading", &serde_json::json!({"file": "note.md", "heading": "Test", "content": "New content"})).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_tool_create_note() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_str().unwrap();
    let server = LoomServer::new(root).await;
    let result = server.dispatch_tool("create_note", &serde_json::json!({"title": "New Note", "content": "Content"})).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_tool_edit_note() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_str().unwrap();
    let note = tmp.path().join("note.md");
    std::fs::write(&note, "# Test\nOld content").unwrap();
    let server = LoomServer::new(root).await;
    let result = server.dispatch_tool("edit_note", &serde_json::json!({"file": "note.md", "content": "# Test\nNew content"})).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_tool_apply_edit_preview() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_str().unwrap();
    let note = tmp.path().join("note.md");
    std::fs::write(&note, "# Test\nOld content").unwrap();
    let server = LoomServer::new(root).await;
    let result = server.dispatch_tool("apply_edit_preview", &serde_json::json!({"file": "note.md", "heading": "Test", "proposed": "New content"})).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_tool_link_notes() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_str().unwrap();
    let note1 = tmp.path().join("note1.md");
    let note2 = tmp.path().join("note2.md");
    std::fs::write(&note1, "# Note 1\nContent").unwrap();
    std::fs::write(&note2, "# Note 2\nContent").unwrap();
    let server = LoomServer::new(root).await;
    let result = server.dispatch_tool("link_notes", &serde_json::json!({"from": "note1.md", "to": "note2.md"})).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_tool_move_note() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_str().unwrap();
    let note = tmp.path().join("note.md");
    std::fs::write(&note, "# Test\nContent").unwrap();
    let server = LoomServer::new(root).await;
    let result = server.dispatch_tool("move_note", &serde_json::json!({"from": "note.md", "to": "moved.md"})).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_tool_delete_note() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_str().unwrap();
    let note = tmp.path().join("note.md");
    std::fs::write(&note, "# Test\nContent").unwrap();
    let server = LoomServer::new(root).await;
    let result = server.dispatch_tool("delete_note", &serde_json::json!({"file": "note.md"})).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_tool_rank_notes() {
    let tmp = TempDir::new().unwrap();
    let server = LoomServer::new(tmp.path().to_str().unwrap()).await;
    let result = server.dispatch_tool("rank_notes", &serde_json::json!({})).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_tool_detect_themes() {
    let tmp = TempDir::new().unwrap();
    let server = LoomServer::new(tmp.path().to_str().unwrap()).await;
    let result = server.dispatch_tool("detect_themes", &serde_json::json!({})).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_tool_find_connections() {
    let tmp = TempDir::new().unwrap();
    let server = LoomServer::new(tmp.path().to_str().unwrap()).await;
    let result = server.dispatch_tool("find_connections", &serde_json::json!({"note": "test"})).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_tool_find_path_between() {
    let tmp = TempDir::new().unwrap();
    let server = LoomServer::new(tmp.path().to_str().unwrap()).await;
    let result = server.dispatch_tool("find_path_between", &serde_json::json!({"note_a": "test1", "note_b": "test2"})).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_tool_search_graph() {
    let tmp = TempDir::new().unwrap();
    let server = LoomServer::new(tmp.path().to_str().unwrap()).await;
    let result = server.dispatch_tool("search_graph", &serde_json::json!({"note": "test"})).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_tool_search_smart_without_brainjar() {
    let tmp = TempDir::new().unwrap();
    let server = LoomServer::new(tmp.path().to_str().unwrap()).await;
    let result = server.dispatch_tool("search_smart", &serde_json::json!({"query": "test"})).await;
    assert!(result.is_ok());
    // Should return error about BRAINJAR_PATH not configured
    assert!(result.unwrap().contains("BRAINJAR_PATH not configured"));
}

#[tokio::test]
async fn test_dispatch_tool_reindex() {
    let tmp = TempDir::new().unwrap();
    let server = LoomServer::new(tmp.path().to_str().unwrap()).await;
    let result = server.dispatch_tool("reindex", &serde_json::json!({})).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_server_get_info() {
    let tmp = TempDir::new().unwrap();
    let server = LoomServer::new(tmp.path().to_str().unwrap()).await;
    let info = server.get_info();
    assert!(info.instructions.is_some());
    assert!(info.capabilities.tools.is_some());
}

#[tokio::test]
async fn test_server_tool_list_count() {
    let tmp = TempDir::new().unwrap();
    let server = LoomServer::new(tmp.path().to_str().unwrap()).await;
    let tools = server.tool_list();
    assert_eq!(tools.len(), 24);
}

#[tokio::test]
async fn test_dispatch_tool_with_optional_params() {
    let tmp = TempDir::new().unwrap();
    let server = LoomServer::new(tmp.path().to_str().unwrap()).await;
    let result = server.dispatch_tool("search", &serde_json::json!({"query": "test", "top_k": 5, "max_sections": 2, "max_section_chars": 300})).await;
    assert!(result.is_ok());
}
