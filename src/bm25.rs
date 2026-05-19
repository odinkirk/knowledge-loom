use crate::chunks;
use crate::vault::VaultState;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tantivy::{
    query::Occur,
    query::TermQuery,
    schema::Document,
    schema::IndexRecordOption,
    schema::Value,
    schema::{SchemaBuilder, STORED, STRING, TEXT},
    Index, IndexWriter, TantivyDocument, TantivyError, Term,
};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct ChunkDoc {
    pub path: String,
    pub heading: Option<String>,
    pub content: String,
    pub line_start: usize,
    pub line_end: usize,
    pub chunk_ordinal: u64,
}

fn get_text(doc: &TantivyDocument, schema: &tantivy::schema::Schema, field_name: &str) -> String {
    schema
        .get_field(field_name)
        .ok()
        .and_then(|field| {
            doc.iter_fields_and_values()
                .find(|(f, _)| *f == field)
                .and_then(|(_, v)| v.as_str())
        })
        .map(std::string::ToString::to_string)
        .unwrap_or_default()
}

fn get_u64(doc: &TantivyDocument, schema: &tantivy::schema::Schema, field_name: &str) -> u64 {
    schema
        .get_field(field_name)
        .ok()
        .and_then(|field| {
            doc.iter_fields_and_values()
                .find(|(f, _)| *f == field)
                .and_then(|(_, v)| v.as_u64())
        })
        .unwrap_or(1)
}

#[allow(dead_code)]
pub struct BM25Index {
    pub index: Arc<Index>,
    pub writer: Arc<Mutex<IndexWriter>>,
    pub schema: tantivy::schema::Schema,
    pub kb_root: PathBuf,
    pub index_path: PathBuf,
    pub is_ingesting: Arc<Mutex<bool>>,
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
        let _chunk_ordinal = schema_builder.add_u64_field("chunk_ordinal", STORED);
        let schema = schema_builder.build();

        // Open or create index; recreate if the stored schema doesn't match
        // (e.g. stale index from a previous version with different field options)
        let index = match Index::open_in_dir(&index_path) {
            Ok(idx) if idx.schema() == schema => idx,
            Ok(_) | Err(TantivyError::LockFailure(_, _)) => {
                // Attempt a second open before touching the lock file.
                // If the second attempt succeeds, the lock is already released — no deletion needed.
                // If it also fails, the lock file is stale (no live writer) — safe to remove.
                match Index::open_in_dir(&index_path) {
                    Ok(idx) if idx.schema() == schema => idx,
                    Ok(_) => {
                        // Schema mismatch — wipe and recreate
                        let _ = std::fs::remove_dir_all(&index_path);
                        let _ = std::fs::create_dir_all(&index_path);
                        Index::create_in_dir(&index_path, schema.clone())
                            .unwrap_or_else(|e| panic!("Failed to create tantivy index: {e}"))
                    }
                    Err(TantivyError::LockFailure(_, _)) => {
                        // Lock is stale — remove and retry
                        let lock_file = index_path.join(".tantivy-writer.lock");
                        let _ = std::fs::remove_file(&lock_file);
                        Index::open_in_dir(&index_path).unwrap_or_else(|e| {
                            panic!("Failed to recover tantivy index after stale lock removal: {e}")
                        })
                    }
                    Err(_) => {
                        // Other error — wipe and recreate
                        let _ = std::fs::remove_dir_all(&index_path);
                        let _ = std::fs::create_dir_all(&index_path);
                        Index::create_in_dir(&index_path, schema.clone())
                            .unwrap_or_else(|e| panic!("Failed to create tantivy index: {e}"))
                    }
                }
            }
            Err(_) => {
                // Other error — wipe and recreate
                let _ = std::fs::remove_dir_all(&index_path);
                let _ = std::fs::create_dir_all(&index_path);
                Index::create_in_dir(&index_path, schema.clone())
                    .unwrap_or_else(|e| panic!("Failed to create tantivy index: {e}"))
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
            is_ingesting: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn set_ingesting(&self, ingesting: bool) {
        let mut is_ingesting = self.is_ingesting.lock().await;
        *is_ingesting = ingesting;
    }

    pub async fn is_ingesting(&self) -> bool {
        let is_ingesting = self.is_ingesting.lock().await;
        *is_ingesting
    }

    pub async fn commit(&self) -> Result<(), TantivyError> {
        let mut writer_lock = self.writer.lock().await;
        writer_lock.commit().map(|_| ())
    }

    pub fn doc_count(&self) -> Result<u64, TantivyError> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        Ok(searcher.num_docs())
    }

    pub async fn index_file(
        &mut self,
        path: &Path,
        content: &str,
    ) -> Result<(), tantivy::TantivyError> {
        let path_str = path
            .strip_prefix(&self.kb_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        let path_field = self.schema.get_field("path").unwrap();
        let path_term = Term::from_field_text(path_field, &path_str);

        let chunks = chunks::parse_chunks(content);
        let writer_lock = self.writer.lock().await;
        writer_lock.delete_term(path_term);
        for chunk in chunks {
            let mut doc = TantivyDocument::new();
            doc.add_text(
                self.schema.get_field("heading").unwrap(),
                chunk.heading.as_deref().unwrap_or(""),
            );
            doc.add_text(self.schema.get_field("content").unwrap(), &chunk.content);
            doc.add_text(self.schema.get_field("path").unwrap(), &path_str);
            doc.add_u64(
                self.schema.get_field("line_start").unwrap(),
                chunk.line_start as u64,
            );
            doc.add_u64(
                self.schema.get_field("line_end").unwrap(),
                chunk.line_end as u64,
            );
            doc.add_u64(
                self.schema.get_field("chunk_ordinal").unwrap(),
                chunk.ordinal,
            );
            writer_lock.add_document(doc)?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn add_document(
        &mut self,
        path: &Path,
        title: &str,
        content: &str,
    ) -> Result<(), tantivy::TantivyError> {
        let path_str = path
            .strip_prefix(&self.kb_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        let mut doc = TantivyDocument::new();
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
            path_obj
                .strip_prefix(&self.kb_root)
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
        let top_docs = searcher.search(
            &term_query,
            &tantivy::collector::TopDocs::with_limit(1000).order_by_score(),
        )?;

        let mut chunks = Vec::new();
        for (_, doc_address) in top_docs {
            let doc = searcher.doc(doc_address)?;
            let heading_raw = get_text(&doc, &self.schema, "heading");
            let heading = if heading_raw.is_empty() {
                None
            } else {
                Some(heading_raw)
            };
            chunks.push(ChunkDoc {
                path: path.to_string(),
                heading,
                content: get_text(&doc, &self.schema, "content"),
                #[allow(clippy::cast_possible_truncation)]
                line_start: get_u64(&doc, &self.schema, "line_start") as usize,
                #[allow(clippy::cast_possible_truncation)]
                line_end: get_u64(&doc, &self.schema, "line_end") as usize,
                chunk_ordinal: get_u64(&doc, &self.schema, "chunk_ordinal"),
            });
        }
        chunks.sort_by_key(|c| c.line_start);
        Ok(chunks)
    }

    #[allow(dead_code)]
    /// Retrieves a chunk by its ordinal position within a file.
    ///
    /// This method allows precise chunk retrieval using 1-based ordinal numbers.
    /// Returns an error if ingestion is in progress, ordinal is out of bounds,
    /// or the chunk is not found.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path (relative to kb_root)
    /// * `ordinal` - The 1-based ordinal number of the chunk to retrieve
    ///
    /// # Returns
    ///
    /// A `ChunkDoc` containing the chunk metadata and content
    ///
    /// # Errors
    ///
    /// Returns `TantivyError::InvalidArgument` if:
    /// - Ingestion is in progress
    /// - Ordinal is < 1
    /// - Ordinal exceeds chunk count
    /// - Chunk with ordinal not found
    pub async fn get_chunk_by_ordinal(
        &self,
        path: &str,
        ordinal: u64,
    ) -> Result<ChunkDoc, TantivyError> {
        // Check if ingestion is in progress
        if self.is_ingesting().await {
            return Err(TantivyError::InvalidArgument(
                "indexing: try again in 2 seconds".to_string(),
            ));
        }

        // Validate ordinal >= 1
        if ordinal < 1 {
            return Err(TantivyError::InvalidArgument(
                "Ordinal must be >= 1".to_string(),
            ));
        }

        // Get all chunks for the path
        let chunks = self.get_chunks_for_path(path).await?;

        // Validate ordinal <= chunk count
        if ordinal as usize > chunks.len() {
            return Err(TantivyError::InvalidArgument(format!(
                "Ordinal {} exceeds chunk count {}",
                ordinal,
                chunks.len()
            )));
        }

        // Find chunk with matching ordinal
        chunks
            .into_iter()
            .find(|c| c.chunk_ordinal == ordinal)
            .ok_or_else(|| {
                TantivyError::InvalidArgument(format!("Chunk with ordinal {} not found", ordinal))
            })
    }

    #[allow(dead_code)]
    pub async fn remove_document(&mut self, path: &Path) -> Result<(), TantivyError> {
        let path_str = path
            .strip_prefix(&self.kb_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        let path_term = Term::from_field_text(self.schema.get_field("path").unwrap(), &path_str);

        let mut writer_lock = self.writer.lock().await;
        writer_lock.delete_term(path_term);
        writer_lock.commit()?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<(f32, tantivy::DocAddress)>, TantivyError> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        let mut query_parser = tantivy::query::QueryParser::for_index(
            &self.index,
            vec![
                self.schema.get_field("heading").unwrap(),
                self.schema.get_field("content").unwrap(),
            ],
        );
        query_parser.set_conjunction_by_default();

        let query = query_parser.parse_query(query)?;
        let top_docs = searcher.search(
            &query,
            &tantivy::collector::TopDocs::with_limit(limit).order_by_score(),
        )?;

        Ok(top_docs)
    }

    pub async fn search_and_retrieve(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<(f32, ChunkDoc)>, TantivyError> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        let query_parser = tantivy::query::QueryParser::for_index(
            &self.index,
            vec![
                self.schema.get_field("heading").unwrap(),
                self.schema.get_field("content").unwrap(),
            ],
        );

        let query = query_parser.parse_query(query)?;
        let top_docs = searcher.search(
            &query,
            &tantivy::collector::TopDocs::with_limit(limit).order_by_score(),
        )?;

        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let doc = searcher.doc(doc_address)?;
            let heading_raw = get_text(&doc, &self.schema, "heading");
            let heading = if heading_raw.is_empty() {
                None
            } else {
                Some(heading_raw)
            };
            results.push((
                score,
                ChunkDoc {
                    path: get_text(&doc, &self.schema, "path"),
                    heading,
                    content: get_text(&doc, &self.schema, "content"),
                    #[allow(clippy::cast_possible_truncation)]
                    line_start: get_u64(&doc, &self.schema, "line_start") as usize,
                    #[allow(clippy::cast_possible_truncation)]
                    line_end: get_u64(&doc, &self.schema, "line_end") as usize,
                    chunk_ordinal: get_u64(&doc, &self.schema, "chunk_ordinal"),
                },
            ));
        }
        Ok(results)
    }

    #[allow(dead_code)]
    pub async fn search_file(
        &self,
        file_path: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<(f32, ChunkDoc)>, TantivyError> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();

        // Normalize file_path to relative path from kb_root
        let path_obj = Path::new(file_path);
        let relative_path = if path_obj.is_absolute() {
            path_obj
                .strip_prefix(&self.kb_root)
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

        let query_parser = tantivy::query::QueryParser::for_index(
            &self.index,
            vec![
                self.schema.get_field("heading").unwrap(),
                self.schema.get_field("content").unwrap(),
            ],
        );

        let text_query = query_parser.parse_query(query)?;

        // Combine: text_query AND path_query
        let boolean_query = tantivy::query::BooleanQuery::new(vec![
            (Occur::Must, text_query),
            (Occur::Must, path_query),
        ]);

        let top_docs = searcher.search(
            &boolean_query,
            &tantivy::collector::TopDocs::with_limit(limit).order_by_score(),
        )?;

        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let doc = searcher.doc(doc_address)?;
            let heading_raw = get_text(&doc, &self.schema, "heading");
            let heading = if heading_raw.is_empty() {
                None
            } else {
                Some(heading_raw)
            };
            let content = get_text(&doc, &self.schema, "content");
            // Only include results where content contains the query as a substring
            if content.to_lowercase().contains(&query.to_lowercase()) {
                results.push((
                    score,
                    ChunkDoc {
                        path: get_text(&doc, &self.schema, "path"),
                        heading,
                        content,
                        line_start: get_u64(&doc, &self.schema, "line_start") as usize,
                        line_end: get_u64(&doc, &self.schema, "line_end") as usize,
                        chunk_ordinal: get_u64(&doc, &self.schema, "chunk_ordinal"),
                    },
                ));
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

#[allow(dead_code)]
#[must_use]
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
