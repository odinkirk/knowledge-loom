use axum::http::StatusCode;
use axum_test::TestServer;
use knowledge_loom::web::make_router;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_get_root_returns_html() {
    let tmp = TempDir::new().unwrap();
    let state = knowledge_loom::web::WebState::new(tmp.path().to_str().unwrap()).await;
    let app = make_router(Arc::new(state));
    let server = TestServer::new(app).unwrap();
    let resp = server.get("/").await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    let body = resp.text();
    assert!(body.contains("<html"), "body: {body}");
    assert!(body.contains("search"), "should mention search: {body}");
}

#[tokio::test]
async fn test_post_api_search_empty_query_returns_empty() {
    let tmp = TempDir::new().unwrap();
    let state = knowledge_loom::web::WebState::new(tmp.path().to_str().unwrap()).await;
    let app = make_router(Arc::new(state));
    let server = TestServer::new(app).unwrap();
    let resp = server
        .post("/api/search")
        .json(&serde_json::json!({ "query": "" }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    let body: serde_json::Value = resp.json();
    assert!(body["results"].is_array(), "body: {body}");
}

#[tokio::test]
async fn test_get_api_files_returns_array() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("note.md"), "# Note\nHello").unwrap();
    let state = knowledge_loom::web::WebState::new(tmp.path().to_str().unwrap()).await;
    let app = make_router(Arc::new(state));
    let server = TestServer::new(app).unwrap();
    let resp = server.get("/api/files").await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    let body: serde_json::Value = resp.json();
    assert!(body.is_array(), "files should be array: {body}");
}
