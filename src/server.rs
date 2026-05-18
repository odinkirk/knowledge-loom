use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

use rmcp::{
    model::{
        CallToolRequestParams, CallToolResult, Content, ListToolsResult, PaginatedRequestParams,
        ServerCapabilities, ServerInfo, Tool,
    },
    service::RequestContext,
    transport::stdio,
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
};
use std::sync::Arc as StdArc;

use crate::bm25::BM25Index;
use crate::edits::EditManager;
use crate::embed::EmbedProviderEnum;
use crate::graph::GraphState;
use crate::index::VectorIndex;
use crate::maintenance::MaintenanceManager;
use crate::search::SearchEngine;
use crate::vault::VaultState;

#[allow(dead_code)]
pub struct LoomServer {
    pub kb_root: String,
    pub vault: Arc<Mutex<VaultState>>,
    pub bm25: Arc<Mutex<BM25Index>>,
    pub embed: Arc<EmbedProviderEnum>,
    pub vector: Arc<Mutex<VectorIndex>>,
    pub graph: Arc<Mutex<GraphState>>,
    pub edits: Arc<EditManager>,
    pub maintenance: Arc<MaintenanceManager>,
    pub search_engine: SearchEngine,
}

pub struct ToolDef {
    pub name: &'static str,
    pub description: &'static str,
    pub schema: Value,
}

impl LoomServer {
    pub async fn new(kb_root: &str) -> Self {
        let vault = Arc::new(Mutex::new(VaultState::new(kb_root).await));
        let bm25 = Arc::new(Mutex::new(BM25Index::new(kb_root).await));
        let embed = Arc::new(EmbedProviderEnum::new(kb_root));
        let vector = Arc::new(Mutex::new(VectorIndex::new(kb_root).await));
        let graph = Arc::new(Mutex::new(GraphState::new(kb_root).await));

        let search_engine = SearchEngine::from_components(
            bm25.clone(),
            vector.clone(),
            embed.clone(),
            graph.clone(),
        );

        let edits = Arc::new(EditManager::new(
            kb_root.to_string(),
            vault.clone(),
            bm25.clone(),
            embed.clone(),
            vector.clone(),
            graph.clone(),
        ));
        let maintenance = Arc::new(MaintenanceManager::new(
            kb_root.to_string(),
            vault.clone(),
            bm25.clone(),
            embed.clone(),
            vector.clone(),
            graph.clone(),
        ));

        Self {
            kb_root: kb_root.to_string(),
            vault,
            bm25,
            embed,
            vector,
            graph,
            edits,
            maintenance,
            search_engine,
        }
    }

    pub fn tool_list(&self) -> Vec<ToolDef> {
        vec![
            ToolDef {
                name: "search",
                description: "BM25 + semantic RRF search",
                schema: json!({"type":"object","properties":{"query":{"type":"string"},"top_k":{"type":"integer","default":10}},"required":["query"]}),
            },
            ToolDef {
                name: "search_file",
                description: "Search within a specific file",
                schema: json!({"type":"object","properties":{"file":{"type":"string"},"query":{"type":"string"},"top_k":{"type":"integer","default":10}},"required":["file","query"]}),
            },
            ToolDef {
                name: "search_graph",
                description: "Graph entity/relationship traversal",
                schema: json!({"type":"object","properties":{"note":{"type":"string"}},"required":["note"]}),
            },
            ToolDef {
                name: "rank_notes",
                description: "PageRank influence ranking",
                schema: json!({"type":"object","properties":{}}),
            },
            ToolDef {
                name: "find_connections",
                description: "Links and relationships for a note",
                schema: json!({"type":"object","properties":{"note":{"type":"string"}},"required":["note"]}),
            },
            ToolDef {
                name: "find_path_between",
                description: "Shortest graph path between two notes",
                schema: json!({"type":"object","properties":{"note_a":{"type":"string"},"note_b":{"type":"string"}},"required":["note_a","note_b"]}),
            },
            ToolDef {
                name: "detect_themes",
                description: "Louvain community/theme detection",
                schema: json!({"type":"object","properties":{}}),
            },
            ToolDef {
                name: "list_files",
                description: "List all Markdown files",
                schema: json!({"type":"object","properties":{}}),
            },
            ToolDef {
                name: "outline",
                description: "Heading hierarchy with line numbers",
                schema: json!({"type":"object","properties":{"file":{"type":"string"}},"required":["file"]}),
            },
            ToolDef {
                name: "grep",
                description: "Regex search, optional file filter",
                schema: json!({"type":"object","properties":{"pattern":{"type":"string"},"file_filter":{"type":"string"}},"required":["pattern"]}),
            },
            ToolDef {
                name: "read_section",
                description: "Read content under a heading",
                schema: json!({"type":"object","properties":{"file":{"type":"string"},"heading":{"type":"string"}},"required":["file","heading"]}),
            },
            ToolDef {
                name: "read_lines",
                description: "Read exact line range",
                schema: json!({"type":"object","properties":{"file":{"type":"string"},"start":{"type":"integer"},"end":{"type":"integer"}},"required":["file","start","end"]}),
            },
            ToolDef {
                name: "replace_lines",
                description: "In-place line range replacement",
                schema: json!({"type":"object","properties":{"file":{"type":"string"},"start":{"type":"integer"},"end":{"type":"integer"},"content":{"type":"string"}},"required":["file","start","end","content"]}),
            },
            ToolDef {
                name: "insert_after_heading",
                description: "Insert content after a heading",
                schema: json!({"type":"object","properties":{"file":{"type":"string"},"heading":{"type":"string"},"content":{"type":"string"}},"required":["file","heading","content"]}),
            },
            ToolDef {
                name: "append_to_file",
                description: "Append to file with blank-line separator",
                schema: json!({"type":"object","properties":{"file":{"type":"string"},"content":{"type":"string"}},"required":["file","content"]}),
            },
            ToolDef {
                name: "create_note",
                description: "Create note from title and content",
                schema: json!({"type":"object","properties":{"title":{"type":"string"},"content":{"type":"string"}},"required":["title","content"]}),
            },
            ToolDef {
                name: "edit_note",
                description: "Replace full note content",
                schema: json!({"type":"object","properties":{"file":{"type":"string"},"content":{"type":"string"}},"required":["file","content"]}),
            },
            ToolDef {
                name: "apply_edit_preview",
                description: "Dry-run section replacement preview",
                schema: json!({"type":"object","properties":{"file":{"type":"string"},"heading":{"type":"string"},"proposed":{"type":"string"}},"required":["file","heading","proposed"]}),
            },
            ToolDef {
                name: "link_notes",
                description: "Append wikilink from one note to another",
                schema: json!({"type":"object","properties":{"from":{"type":"string"},"to":{"type":"string"}},"required":["from","to"]}),
            },
            ToolDef {
                name: "move_note",
                description: "Move note to new path",
                schema: json!({"type":"object","properties":{"from":{"type":"string"},"to":{"type":"string"}},"required":["from","to"]}),
            },
            ToolDef {
                name: "delete_note",
                description: "Delete a note",
                schema: json!({"type":"object","properties":{"file":{"type":"string"}},"required":["file"]}),
            },
            ToolDef {
                name: "reindex",
                description: "Rebuild all search indexes",
                schema: json!({"type":"object","properties":{}}),
            },
            ToolDef {
                name: "index_status",
                description: "Health and chunk counts for all backends",
                schema: json!({"type":"object","properties":{}}),
            },
        ]
    }

    pub async fn dispatch_tool(&self, name: &str, args: &Value) -> Result<String, String> {
        let kb = &self.kb_root;

        // Helper to get string arg
        let str_arg = |key: &str| -> Result<String, String> {
            args.get(key)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| format!("missing required argument: {key}"))
        };
        let int_arg = |key: &str| -> Result<usize, String> {
            args.get(key)
                .and_then(|v| v.as_u64())
                .map(|n| n as usize)
                .ok_or_else(|| format!("missing required argument: {key}"))
        };

        match name {
            "search" => {
                let query = str_arg("query")?;
                let top_k = args.get("top_k").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
                let results = self.search_engine.search(&query, top_k).await;
                // Convert to serializable format
                let serializable: Vec<serde_json::Value> = results
                    .into_iter()
                    .map(|r| {
                        let sections: Vec<serde_json::Value> = r
                            .sections
                            .into_iter()
                            .map(|s| {
                                serde_json::json!({
                                    "heading": s.heading,
                                    "content": s.content,
                                    "line_start": s.line_start,
                                    "line_end": s.line_end,
                                    "chunk_ordinal": s.chunk_ordinal,
                                    "score": s.score,
                                })
                            })
                            .collect();
                        serde_json::json!({
                            "path": r.path,
                            "score": r.score,
                            "sections": sections,
                        })
                    })
                    .collect();
                Ok(serde_json::to_string(&serializable).unwrap_or_default())
            }
            "search_file" => {
                let file = str_arg("file")?;
                let query = str_arg("query")?;
                let top_k = args.get("top_k").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
                // Search with larger top_k to ensure we get results from this file
                let results = self.search_engine.search(&query, 100).await;
                // Filter by exact file path (supports both "note.md" and "note" formats)
                let file_normalized = file.trim_end_matches(".md");
                let filtered: Vec<_> = results
                    .into_iter()
                    .filter(|r| {
                        let path_normalized = r.path.trim_end_matches(".md");
                        path_normalized.ends_with(file_normalized)
                    })
                    .collect();
                let limited: Vec<_> = filtered.into_iter().take(top_k).collect();
                let serializable: Vec<serde_json::Value> = limited
                    .into_iter()
                    .map(|r| {
                        let sections: Vec<serde_json::Value> = r
                            .sections
                            .into_iter()
                            .map(|s| {
                                serde_json::json!({
                                    "heading": s.heading,
                                    "content": s.content,
                                    "line_start": s.line_start,
                                    "line_end": s.line_end,
                                    "chunk_ordinal": s.chunk_ordinal,
                                    "score": s.score,
                                })
                            })
                            .collect();
                        serde_json::json!({
                            "path": r.path,
                            "score": r.score,
                            "sections": sections,
                        })
                    })
                    .collect();
                Ok(serde_json::to_string(&serializable).unwrap_or_default())
            }
            "list_files" => {
                let files = self.edits.list_files().await;
                Ok(serde_json::to_string(&files).unwrap_or_default())
            }
            "outline" => {
                let file = str_arg("file")?;
                let path = std::path::Path::new(kb).join(&file);
                match self.edits.get_outline(&path).await {
                    Ok(outline) => Ok(serde_json::to_string(&outline).unwrap_or_default()),
                    Err(e) => Err(e.to_string()),
                }
            }
            "grep" => {
                let pattern = str_arg("pattern")?;
                let results = self.edits.grep(&pattern).await;
                Ok(serde_json::to_string(&results).unwrap_or_default())
            }
            "read_section" => {
                let file = str_arg("file")?;
                let heading = str_arg("heading")?;
                let path = std::path::Path::new(kb).join(&file);
                match self.edits.read_section(&path, &heading).await {
                    Ok(Some(content)) => Ok(content),
                    Ok(None) => Ok("Section not found".to_string()),
                    Err(e) => Err(e.to_string()),
                }
            }
            "read_lines" => {
                let file = str_arg("file")?;
                let start = int_arg("start")?;
                let end = int_arg("end")?;
                let path = std::path::Path::new(kb).join(&file);
                match self.edits.read_lines(&path, start, end).await {
                    Ok(Some(content)) => Ok(content),
                    Ok(None) => Ok(String::new()),
                    Err(e) => Err(e.to_string()),
                }
            }
            "replace_lines" => {
                let file = str_arg("file")?;
                let start = int_arg("start")?;
                let end = int_arg("end")?;
                let content = str_arg("content")?;
                let path = std::path::Path::new(kb).join(&file);
                self.edits
                    .replace_lines(&path, start, end, &content)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok("ok".to_string())
            }
            "insert_after_heading" => {
                let file = str_arg("file")?;
                let heading = str_arg("heading")?;
                let content = str_arg("content")?;
                let path = std::path::Path::new(kb).join(&file);
                self.edits
                    .insert_after_heading(&path, &heading, &content)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok("ok".to_string())
            }
            "append_to_file" => {
                let file = str_arg("file")?;
                let content = str_arg("content")?;
                let path = std::path::Path::new(kb).join(&file);
                self.edits
                    .append_to_file(&path, &content)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok("ok".to_string())
            }
            "create_note" => {
                let title = str_arg("title")?;
                let content = str_arg("content")?;
                let path = self
                    .edits
                    .create_note(&title, &content)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(path.to_string_lossy().to_string())
            }
            "edit_note" => {
                let file = str_arg("file")?;
                let content = str_arg("content")?;
                let path = std::path::Path::new(kb).join(&file);
                self.edits
                    .edit_note(&path, &content)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok("ok".to_string())
            }
            "apply_edit_preview" => {
                let file = str_arg("file")?;
                let heading = str_arg("heading")?;
                let proposed = str_arg("proposed")?;
                let path = std::path::Path::new(kb).join(&file);
                match self
                    .edits
                    .apply_edit_preview(&path, &heading, &proposed)
                    .await
                {
                    Ok(Some(preview)) => Ok(serde_json::to_string(&serde_json::json!({
                        "heading": preview.heading,
                        "line_start": preview.line_start,
                        "line_end": preview.line_end,
                        "current": preview.current,
                        "proposed": preview.proposed,
                    }))
                    .unwrap()),
                    Ok(None) => Ok("Section not found".to_string()),
                    Err(e) => Err(e.to_string()),
                }
            }
            "link_notes" => {
                let from = str_arg("from")?;
                let to = str_arg("to")?;
                let from_path = std::path::Path::new(kb).join(&from);
                let to_path = std::path::Path::new(kb).join(&to);
                self.edits
                    .link_notes(&from_path, &to_path)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok("ok".to_string())
            }
            "move_note" => {
                let from = str_arg("from")?;
                let to = str_arg("to")?;
                let from_path = std::path::Path::new(kb).join(&from);
                let to_path = std::path::Path::new(kb).join(&to);
                self.edits
                    .move_note(&from_path, &to_path)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok("ok".to_string())
            }
            "delete_note" => {
                let file = str_arg("file")?;
                let path = std::path::Path::new(kb).join(&file);
                self.edits
                    .delete_note(&path)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok("ok".to_string())
            }
            "rank_notes" => {
                let graph = self.graph.lock().await;
                let (pagerank, _) = graph.get_cached_analytics().await;
                let mut ranked: Vec<_> = pagerank.iter().collect();
                ranked.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
                Ok(serde_json::to_string(&ranked).unwrap_or_default())
            }
            "detect_themes" => {
                let graph = self.graph.lock().await;
                let (_, communities) = graph.get_cached_analytics().await;
                Ok(serde_json::to_string(&communities).unwrap_or_default())
            }
            "find_connections" => {
                let note = str_arg("note")?;
                let graph = self.graph.lock().await;
                let connections = graph.bfs_connections(&note, 3).await;
                Ok(serde_json::to_string(&connections).unwrap_or_default())
            }
            "find_path_between" => {
                let note_a = str_arg("note_a")?;
                let note_b = str_arg("note_b")?;
                let graph = self.graph.lock().await;
                let path = graph.dijkstra_path(&note_a, &note_b).await;
                Ok(serde_json::to_string(&path).unwrap_or_default())
            }
            "search_graph" => {
                let note = str_arg("note")?;
                let graph = self.graph.lock().await;
                let results = graph.search_graph(&note).await;
                Ok(serde_json::to_string(&results).unwrap_or_default())
            }
            "search_smart" => {
                let _query = str_arg("query")?;
                // LLM-decomposed search is out of scope for this implementation
                // Requires LLM API integration (separate plan)
                Ok(serde_json::json!({"error": "search_smart requires LLM API integration (not yet implemented)"}).to_string())
            }
            "reindex" => self.maintenance.reindex_all(false).await,
            "index_status" => match self.maintenance.get_index_status().await {
                Ok(status) => Ok(serde_json::to_string(&status).unwrap_or_default()),
                Err(e) => Err(e),
            },
            other => Err(format!("unknown tool: {other}")),
        }
    }
}

impl ServerHandler for LoomServer {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.instructions = Some("Knowledge Loom MCP server — 24 tools for search, navigation, editing, and graph analytics.".into());
        info
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let tools = self
            .tool_list()
            .into_iter()
            .map(|t| {
                Tool::new(
                    t.name,
                    t.description,
                    StdArc::new(serde_json::from_value(t.schema).unwrap_or_default()),
                )
            })
            .collect();
        Ok(ListToolsResult {
            meta: None,
            next_cursor: None,
            tools,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let args = request
            .arguments
            .map(Value::Object)
            .unwrap_or(Value::Object(Default::default()));

        match self.dispatch_tool(&request.name, &args).await {
            Ok(text) => Ok(CallToolResult::success(vec![Content::text(text)])),
            Err(msg) => Ok(CallToolResult::error(vec![Content::text(msg)])),
        }
    }
}

pub async fn run_server() {
    let kb_root = std::env::var("KB_ROOT").expect("KB_ROOT environment variable must be set");

    let server = LoomServer::new(&kb_root).await;

    let running = server
        .serve(stdio())
        .await
        .expect("MCP server initialization error");

    // Wait for the service to complete (this keeps the server running)
    let quit_reason = running.waiting().await.expect("MCP server runtime error");
    eprintln!("MCP server stopped: {:?}", quit_reason);
}
