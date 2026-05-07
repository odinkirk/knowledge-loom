use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::server::LoomServer;

pub struct WebState {
    pub server: LoomServer,
}

impl WebState {
    pub async fn new(kb_root: &str) -> Self {
        Self { server: LoomServer::new(kb_root).await }
    }
}

const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Knowledge Loom</title>
<style>
  body { font-family: system-ui, sans-serif; max-width: 800px; margin: 40px auto; padding: 0 20px; }
  input { width: 100%; padding: 8px; font-size: 16px; border: 1px solid #ccc; border-radius: 4px; }
  .result { border-left: 3px solid #0066cc; margin: 12px 0; padding: 8px 12px; }
  .result .path { font-weight: bold; color: #0066cc; }
  .result .heading { color: #555; font-style: italic; }
  .result .coord { font-size: 12px; color: #999; font-family: monospace; }
  .result .snippet { margin-top: 4px; font-size: 14px; }
</style>
</head>
<body>
<h1>Knowledge Loom</h1>
<input id="q" type="search" placeholder="Search your knowledge base..." autofocus>
<div id="results"></div>
<script>
let timer;
document.getElementById('q').addEventListener('input', e => {
  clearTimeout(timer);
  timer = setTimeout(() => search(e.target.value), 300);
});

async function search(query) {
  if (!query.trim()) { document.getElementById('results').innerHTML = ''; return; }
  const resp = await fetch('/api/search', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ query, limit: 20 })
  });
  const data = await resp.json();
  const el = document.getElementById('results');
  if (!data.results || data.results.length === 0) {
    el.innerHTML = '<p>No results.</p>';
    return;
  }
  el.innerHTML = data.results.map(r => `
    <div class="result">
      <div class="path">${r.path}</div>
      ${r.heading ? `<div class="heading">${r.heading}</div>` : ''}
      <div class="coord">line ${r.line_start ?? '?'} &bull; score ${r.score?.toFixed(3) ?? '?'}</div>
      ${r.snippet ? `<div class="snippet">${r.snippet}</div>` : ''}
    </div>
  `).join('');
}
</script>
</body>
</html>"#;

#[derive(Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
}
fn default_limit() -> usize { 20 }

#[derive(Serialize)]
pub struct SearchResponse {
    pub results: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct OutlineQuery {
    pub file: String,
}

pub fn make_router(state: Arc<WebState>) -> Router {
    Router::new()
        .route("/", get(handle_index))
        .route("/api/search", post(handle_search))
        .route("/api/files", get(handle_files))
        .route("/api/outline", get(handle_outline))
        .with_state(state)
}

async fn handle_index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn handle_search(
    State(state): State<Arc<WebState>>,
    Json(req): Json<SearchRequest>,
) -> impl IntoResponse {
    if req.query.trim().is_empty() {
        return Json(serde_json::json!({ "results": [] }));
    }
    let results = state.server.search_engine
        .search(&req.query, req.limit)
        .await;
    let json_results: Vec<serde_json::Value> = results.iter().flat_map(|r| {
        r.sections.iter().map(|s| {
            serde_json::json!({
                "path": r.path,
                "heading": s.heading,
                "line_start": s.line_start,
                "score": s.score,
                "snippet": s.content.chars().take(200).collect::<String>(),
            })
        })
    }).collect();
    Json(serde_json::json!({ "results": json_results }))
}

async fn handle_files(State(state): State<Arc<WebState>>) -> impl IntoResponse {
    let files = state.server.edits.list_files().await;
    Json(serde_json::Value::Array(
        files.iter().map(|f| serde_json::json!({ "path": f })).collect()
    ))
}

async fn handle_outline(
    State(state): State<Arc<WebState>>,
    Query(q): Query<OutlineQuery>,
) -> impl IntoResponse {
    match state.server.edits.get_outline(std::path::Path::new(&q.file)).await {
        Ok(outline) => Json(serde_json::json!({ "headings": outline })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

pub async fn run_web(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let kb_root = std::env::var("KB_ROOT")?;
    let state = Arc::new(WebState::new(&kb_root).await);
    let app = make_router(state);
    let addr = format!("0.0.0.0:{port}");
    eprintln!("knowledge-loom web UI: http://localhost:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
