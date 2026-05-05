use std::path::{Path, PathBuf};
use std::sync::Arc;
use tantivy::{collector::TopDocs, Document, Index, IndexWriter, TantivyError, Term};
use tantivy::schema::{SchemaBuilder, TEXT, STORED, STRING};
use tokio::sync::Mutex;

use crate::vault::VaultState;

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
        let index_path = kb_root_path.join(".loom-index/tantivy");
        
        // Create directory if it doesn't exist
        let _ = std::fs::create_dir_all(&index_path);
        
        // Define schema using correct tantivy 0.19 API
        let mut schema_builder = SchemaBuilder::new();
        let title = schema_builder.add_text_field("title", TEXT | STORED);
        let content = schema_builder.add_text_field("content", TEXT | STORED);
        let path = schema_builder.add_text_field("path", STRING | STORED);
        let schema = schema_builder.build();
        
        // Open or create index
        let index = match Index::open_in_dir(&index_path) {
            Ok(idx) => idx,
            Err(_) => Index::create_in_dir(&index_path, schema.clone()).unwrap_or_else(|e| {
                panic!("Failed to create tantivy index: {}", e)
            }),
        };
        
        // Initialize writer
        let writer = index.writer(50_000_000).unwrap();
        
        Self {
            index: Arc::new(index),
            writer: Arc::new(Mutex::new(writer)),
            schema,
            kb_root: kb_root_path,
            index_path,
        }
    }
    
    pub async fn add_document(&mut self, path: &Path, title: &str, content: &str) -> Result<(), TantivyError> {
        let mut doc = Document::new();
        doc.add_text(self.schema.get_field("title").unwrap(), title);
        doc.add_text(self.schema.get_field("content").unwrap(), content);
        doc.add_text(self.schema.get_field("path").unwrap(), 
                    path.to_string_lossy().as_ref());
        
        let mut writer_lock = self.writer.lock().await;
        writer_lock.add_document(doc)?;
        // Don't commit here - commit at the end of batch operations
        Ok(())
    }
    
    pub async fn remove_document(&mut self, path: &Path) -> Result<(), TantivyError> {
        let path_term = Term::from_field_text(
            self.schema.get_field("path").unwrap(),
            path.to_string_lossy().as_ref()
        );
        
        let mut writer_lock = self.writer.lock().await;
        writer_lock.delete_term(path_term);
        writer_lock.commit()?;
        Ok(())
    }
    
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<(f32, tantivy::DocAddress)>, TantivyError> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        let query_parser = tantivy::query::QueryParser::for_index(&self.index, vec![
            self.schema.get_field("title").unwrap(),
            self.schema.get_field("content").unwrap()
        ]);
        
        let query = query_parser.parse_query(query)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;
        
        Ok(top_docs)
    }
    
    pub async fn search_and_retrieve(&self, query: &str, limit: usize) -> Result<Vec<(f32, tantivy::schema::Document)>, TantivyError> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        let query_parser = tantivy::query::QueryParser::for_index(&self.index, vec![
            self.schema.get_field("title").unwrap(),
            self.schema.get_field("content").unwrap()
        ]);
        
        let query = query_parser.parse_query(query)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;
        
        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address)?;
            results.push((score, retrieved_doc));
        }
        
        Ok(results)
    }
    
    pub async fn index_vault(&mut self, vault_state: &VaultState) -> Result<(), TantivyError> {
        let files = vault_state.scan_files().await;
        let mut indexed_count = 0;
        
        for file_path in files {
            // Check if file has been modified since last index
            if let Some(mod_time) = vault_state.get_file_mod_time(&file_path).await {
                // For now, we'll just index everything
                // In a real implementation, we'd check against stored timestamps
                
                if let Some(content) = vault_state.read_file(&file_path).await {
                    // Extract title from first heading or filename
                    let title = extract_title(&content)
                        .unwrap_or_else(|| file_path.file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string());
                    
                    self.add_document(&file_path, &title, &content).await?;
                    indexed_count += 1;
                }
            }
        }
        
        // Commit once at the end
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