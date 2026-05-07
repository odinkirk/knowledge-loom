use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tantivy::{
    schema::{SchemaBuilder, TEXT, STRING, STORED, Document},
    collector::TopDocs,
    query::TermQuery,
    query::Occur,
    schema::IndexRecordOption,
    Index, IndexWriter, Term, TantivyError,
};
use crate::vault::VaultState;

pub const MAX_CHUNK_CHARS: usize = 2000;

pub fn truncate_at_whitespace(content: &str, max: usize) -> &str {
    if content.len() <= max {
        return content;
    }
    let slice = &content[..max];
    match slice.rfind(|c: char| c.is_whitespace()) {
        Some(pos) if pos > 0 => content[..pos].trim_end(),
        _ => &content[..max],
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub heading: Option<String>,
    pub content: String,
    pub line_start: usize,
    pub line_end: usize,
}

pub fn parse_chunks(content: &str) -> Vec<Chunk> {
    let lines: Vec<&str> = content.lines().collect();
    let mut chunks = Vec::new();
    // heading_stack: (level, text)
    let mut heading_stack: Vec<(usize, String)> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim_start();
        let level = trimmed.chars().take_while(|&c| c == '#').count();

        if level > 0 && level <= 6 && trimmed.len() > level {
            let after = &trimmed[level..];
            if after.starts_with(' ') || after.starts_with('\t') {
                let heading_text = after.trim().to_string();
                if !heading_text.is_empty() {
                    // Pop same-or-deeper headings
                    while heading_stack.last().map_or(false, |(l, _)| *l >= level) {
                        heading_stack.pop();
                    }
                    heading_stack.push((level, heading_text));

                    let breadcrumb = heading_stack.iter()
                        .map(|(_, t)| t.as_str())
                        .collect::<Vec<_>>()
                        .join(" > ");

                    let section_start = i + 1; // 1-indexed heading line

                    // Collect content until next heading
                    let mut j = i + 1;
                    while j < lines.len() {
                        let next = lines[j].trim_start();
                        let next_level = next.chars().take_while(|&c| c == '#').count();
                        if next_level > 0 && next_level <= 6 && next.len() > next_level {
                            let next_after = &next[next_level..];
                            if next_after.starts_with(' ') || next_after.starts_with('\t') {
                                break;
                            }
                        }
                        j += 1;
                    }

                    let section_content = lines[i + 1..j].join("\n");
                    let section_content = truncate_at_whitespace(section_content.trim(), MAX_CHUNK_CHARS).to_string();
                    let section_end = if j > i + 1 { j } else { i + 1 };

                    if !section_content.is_empty() {
                        chunks.push(Chunk {
                            heading: Some(breadcrumb),
                            content: section_content,
                            line_start: section_start,
                            line_end: section_end,
                        });
                    }

                    i = j;
                    continue;
                }
            }
        }
        i += 1;
    }

    // Headingless fallback
    if chunks.is_empty() {
        let full = truncate_at_whitespace(content.trim(), MAX_CHUNK_CHARS).to_string();
        if !full.is_empty() {
            chunks.push(Chunk {
                heading: None,
                content: full,
                line_start: 1,
                line_end: lines.len(),
            });
        }
    }

    chunks
}

#[derive(Debug, Clone)]
pub struct ChunkDoc {
    pub path: String,
    pub heading: Option<String>,
    pub content: String,
    pub line_start: usize,
    pub line_end: usize,
}

fn get_text(doc: &Document, schema: &tantivy::schema::Schema, field_name: &str) -> String {
    schema.get_field(field_name)
        .and_then(|f| doc.get_first(f))
        .and_then(|v| if let tantivy::schema::Value::Str(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_default()
}

fn get_u64(doc: &Document, schema: &tantivy::schema::Schema, field_name: &str) -> u64 {
    schema.get_field(field_name)
        .and_then(|f| doc.get_first(f))
        .and_then(|v| if let tantivy::schema::Value::U64(n) = v { Some(*n) } else { None })
        .unwrap_or(1)
}

#[allow(dead_code)]
pub struct BM25Index {
    pub index: Arc<Index>,
    pub writer: Arc<Mutex<IndexWriter>>,
    pub schema: tantivy::schema::Schema,
    pub kb_root: PathBuf,
    pub index_path: PathBuf,
}

impl BM25Index {
    pub async fn new(kb_root: &str) -> Self {
        let kb_root_path = PathBuf::from(kb_root);
        let index_path = kb_root_path.join(".knowledge-loom-index/tantivy");
        
        // Create directory if it doesn't exist
        let _ = std::fs::create_dir_all(&index_path);
        
        // Define schema using correct tantivy 0.19 API
        let mut schema_builder = SchemaBuilder::new();
        let _heading = schema_builder.add_text_field("heading", TEXT | STORED);
        let _content = schema_builder.add_text_field("content", TEXT | STORED);
        let _path = schema_builder.add_text_field("path", STRING | STORED);
        let _line_start = schema_builder.add_u64_field("line_start", STORED);
        let _line_end = schema_builder.add_u64_field("line_end", STORED);
        let schema = schema_builder.build();
        
        // Open or create index; recreate if the stored schema doesn't match
        // (e.g. stale index from a previous version with different field options)
        let index = match Index::open_in_dir(&index_path) {
            Ok(idx) if idx.schema() == schema => idx,
            Ok(_) | Err(_) => {
                // Schema mismatch or missing — wipe and recreate
                let _ = std::fs::remove_dir_all(&index_path);
                let _ = std::fs::create_dir_all(&index_path);
                Index::create_in_dir(&index_path, schema.clone()).unwrap_or_else(|e| {
                    panic!("Failed to create tantivy index: {}", e)
                })
            }
        };
        // Always derive schema from the actual index so field IDs are correct
        let schema = index.schema();
        
        // Initialize writer — on LockBusy, the previous server may have left a stale
        // lock file. Delete it and retry once (safe: we verified no live process holds it).
        let writer = match index.writer(50_000_000) {
            Ok(w) => w,
            Err(TantivyError::LockFailure(_, _)) => {
                let _ = std::fs::remove_file(index_path.join(".tantivy-writer.lock"));
                index.writer(50_000_000).unwrap_or_else(|e| {
                    panic!("Failed to acquire index writer after lock reset: {e}")
                })
            }
            Err(e) => panic!("Failed to open index writer: {e}"),
        };
        
        Self {
            index: Arc::new(index),
            writer: Arc::new(Mutex::new(writer)),
            schema,
            kb_root: kb_root_path,
            index_path,
        }
    }
    
    pub async fn index_file(&mut self, path: &Path, content: &str) -> Result<(), TantivyError> {
        let path_str = path.strip_prefix(&self.kb_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        let path_field = self.schema.get_field("path").unwrap();
        let path_term = Term::from_field_text(path_field, &path_str);

        let chunks = parse_chunks(content);
        let mut writer_lock = self.writer.lock().await;
        writer_lock.delete_term(path_term);
        for chunk in chunks {
            let mut doc = Document::new();
            doc.add_text(self.schema.get_field("heading").unwrap(),
                chunk.heading.as_deref().unwrap_or(""));
            doc.add_text(self.schema.get_field("content").unwrap(), &chunk.content);
            doc.add_text(self.schema.get_field("path").unwrap(), &path_str);
            doc.add_u64(self.schema.get_field("line_start").unwrap(), chunk.line_start as u64);
            doc.add_u64(self.schema.get_field("line_end").unwrap(), chunk.line_end as u64);
            writer_lock.add_document(doc)?;
        }
        writer_lock.commit()?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn add_document(&mut self, path: &Path, title: &str, content: &str) -> Result<(), TantivyError> {
        let path_str = path.strip_prefix(&self.kb_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        let mut doc = Document::new();
        doc.add_text(self.schema.get_field("heading").unwrap(), title);
        doc.add_text(self.schema.get_field("content").unwrap(), content);
        doc.add_text(self.schema.get_field("path").unwrap(), &path_str);

        let writer_lock = self.writer.lock().await;
        writer_lock.add_document(doc)?;
        // Don't commit here - commit at the end of batch operations
        Ok(())
    }

    pub async fn get_chunks_for_path(&self, path: &str) -> Result<Vec<ChunkDoc>, TantivyError> {
        // Normalize path to relative path from kb_root
        let path_obj = Path::new(path);
        let relative_path = if path_obj.is_absolute() {
            path_obj.strip_prefix(&self.kb_root)
                .unwrap_or(path_obj)
                .to_string_lossy()
                .to_string()
        } else {
            path.to_string()
        };

        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        let path_field = self.schema.get_field("path").unwrap();
        let path_term = Term::from_field_text(path_field, &relative_path);
        let term_query = TermQuery::new(path_term, IndexRecordOption::Basic);
        let top_docs = searcher.search(&term_query, &TopDocs::with_limit(1000))?;

        let mut chunks = Vec::new();
        for (_, doc_address) in top_docs {
            let doc = searcher.doc(doc_address)?;
            let heading_raw = get_text(&doc, &self.schema, "heading");
            let heading = if heading_raw.is_empty() { None } else { Some(heading_raw) };
            chunks.push(ChunkDoc {
                path: path.to_string(),
                heading,
                content: get_text(&doc, &self.schema, "content"),
                line_start: get_u64(&doc, &self.schema, "line_start") as usize,
                line_end: get_u64(&doc, &self.schema, "line_end") as usize,
            });
        }
        chunks.sort_by_key(|c| c.line_start);
        Ok(chunks)
    }

    #[allow(dead_code)]
    pub async fn remove_document(&mut self, path: &Path) -> Result<(), TantivyError> {
        let path_str = path.strip_prefix(&self.kb_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        let path_term = Term::from_field_text(
            self.schema.get_field("path").unwrap(),
            &path_str,
        );

        let mut writer_lock = self.writer.lock().await;
        writer_lock.delete_term(path_term);
        writer_lock.commit()?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<(f32, tantivy::DocAddress)>, TantivyError> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        let mut query_parser = tantivy::query::QueryParser::for_index(&self.index, vec![
            self.schema.get_field("heading").unwrap(),
            self.schema.get_field("content").unwrap(),
        ]);
        query_parser.set_conjunction_by_default();

        let query = query_parser.parse_query(query)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        Ok(top_docs)
    }
    
    pub async fn search_and_retrieve(&self, query: &str, limit: usize) -> Result<Vec<(f32, ChunkDoc)>, TantivyError> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        let query_parser = tantivy::query::QueryParser::for_index(&self.index, vec![
            self.schema.get_field("heading").unwrap(),
            self.schema.get_field("content").unwrap(),
        ]);

        let query = query_parser.parse_query(query)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let doc = searcher.doc(doc_address)?;
            let heading_raw = get_text(&doc, &self.schema, "heading");
            let heading = if heading_raw.is_empty() { None } else { Some(heading_raw) };
            results.push((score, ChunkDoc {
                path: get_text(&doc, &self.schema, "path"),
                heading,
                content: get_text(&doc, &self.schema, "content"),
                line_start: get_u64(&doc, &self.schema, "line_start") as usize,
                line_end: get_u64(&doc, &self.schema, "line_end") as usize,
            }));
        }
        Ok(results)
    }

    pub async fn search_file(&self, file_path: &str, query: &str, limit: usize) -> Result<Vec<(f32, ChunkDoc)>, TantivyError> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();

        // Normalize file_path to relative path from kb_root
        let path_obj = Path::new(file_path);
        let relative_path = if path_obj.is_absolute() {
            path_obj.strip_prefix(&self.kb_root)
                .unwrap_or(path_obj)
                .to_string_lossy()
                .to_string()
        } else {
            file_path.to_string()
        };

        // Create a boolean query that combines text search with path filter
        let path_field = self.schema.get_field("path").unwrap();
        let path_term = Term::from_field_text(path_field, &relative_path);
        let path_query = Box::new(TermQuery::new(path_term, IndexRecordOption::Basic));

        let query_parser = tantivy::query::QueryParser::for_index(&self.index, vec![
            self.schema.get_field("heading").unwrap(),
            self.schema.get_field("content").unwrap(),
        ]);

        let text_query = query_parser.parse_query(query)?;

        // Combine: text_query AND path_query
        let boolean_query = tantivy::query::BooleanQuery::new(vec![
            (Occur::Must, text_query),
            (Occur::Must, path_query),
        ]);

        let top_docs = searcher.search(&boolean_query, &TopDocs::with_limit(limit))?;

        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let doc = searcher.doc(doc_address)?;
            let heading_raw = get_text(&doc, &self.schema, "heading");
            let heading = if heading_raw.is_empty() { None } else { Some(heading_raw) };
            let content = get_text(&doc, &self.schema, "content");
            // Only include results where content contains the query as a substring
            if content.to_lowercase().contains(&query.to_lowercase()) {
                results.push((score, ChunkDoc {
                    path: get_text(&doc, &self.schema, "path"),
                    heading,
                    content,
                    line_start: get_u64(&doc, &self.schema, "line_start") as usize,
                    line_end: get_u64(&doc, &self.schema, "line_end") as usize,
                }));
            }
        }
        Ok(results)
    }

    pub async fn index_vault(&mut self, vault_state: &VaultState) -> Result<(), TantivyError> {
        let files = vault_state.scan_files().await;
        let mut indexed_count = 0;

        for file_path in files {
            if let Some(content) = vault_state.read_file(&file_path).await {
                self.index_file(&file_path, &content).await?;
                indexed_count += 1;
            }
        }

        if indexed_count > 0 {
            let mut writer_lock = self.writer.lock().await;
            writer_lock.commit()?;
        }

        Ok(())
    }
}

pub fn extract_title(content: &str) -> Option<String> {
    // Look for first markdown heading
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            // Remove heading markers and trim
            let title = trimmed.trim_start_matches('#').trim();
            if !title.is_empty() {
                return Some(title.to_string());
            }
        }
    }
    None
}