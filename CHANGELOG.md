# Changelog

All notable changes to Knowledge Loom will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

 ## [Unreleased]

 ### Added
 - **Safe chunk indexing with ordinal metadata**:
   - UTF-8-safe chunk truncation using `char_indices()` for character boundary detection
   - Ordinal metadata (1-based sequential numbering) for precise chunk retrieval
   - New `chunks.rs` module with centralized chunking logic
   - `get_chunk_by_ordinal()` API for retrieving chunks by file path and ordinal number
   - File-specific re-indexing after edits (not corpus-wide)
   - Corpus re-ingestion on re-indexing failure (<3 seconds for typical vaults)
   - Ingestion state tracking with "indexing: try again in 2 seconds" error during re-ingestion
   - Concurrent edit serialization with request queuing
 - Comprehensive README with architecture diagrams and feature documentation
 - GitHub Actions workflows for testing, building, and releasing
 - Multi-platform binary support (Linux, macOS, Windows)
 - Security vulnerability scanning with cargo-audit
 - License compliance checking with cargo-deny
 - MSRV set to Rust 1.75 for modern async trait support
 - Standardized code formatting with rustfmt.toml
 - Enhanced Clippy linting configuration
 - **Multiple embedding providers** with automatic fallback support:
   - LocalEmbedProvider: Built-in fastembed integration (all-MiniLM-L6-v2, 384 dimensions)
   - OllamaEmbedProvider: Ollama HTTP API integration (nomic-embed-text-v1.5, 768 dimensions)
   - OpenRouterEmbedProvider: OpenRouter HTTP API integration (openai/text-embedding-embedding-ada-002, 1536 dimensions)
 - **Provider priority chain**: OpenRouter > Ollama > Local with automatic fallback
 - **Environment configuration**: OLLAMA_URL, OPENROUTER_API_KEY, OPENROUTER_MODEL
 - **Performance targets**: <100ms local, <500ms Ollama, <1s OpenRouter
 - **Comprehensive error handling**: Network errors, timeouts, dimension mismatches
 - **Logging with eprintln!**: Debug output uses stderr to avoid MCP server pollution

 ### Changed
 - Updated installation process to use `.knowledge-loom` directory structure
 - Improved platform detection and configuration for 8+ coding platforms
 - Enhanced search engine with RRF merging and graph-fused search
 - Improved documentation with Architecture.md and CONTRIBUTING.md
 - **BM25 schema**: Added `chunk_ordinal` STORED field for ordinal metadata
 - **ChunkDoc structure**: Added `chunk_ordinal` field to all chunk results
 - **SearchResult structure**: Ordinal metadata included in search results
 - **Graph Node structure**: Ordinal metadata included in node metadata
 - **MCP tool responses**: Ordinal metadata included in all chunk-related responses

 ### Fixed
 - **UTF-8 panic during chunk truncation**: Fixed by using `char_indices()` for character boundary detection
 - **Inability to retrieve chunks by position**: Added `get_chunk_by_ordinal()` API
 - **Duplicate chunking code**: Extracted chunking logic into dedicated `chunks.rs` module
 - Removed legacy Python installer references
 - Fixed path inconsistencies between old `.loom` and new `.knowledge-loom` structure
 - Removed unimplemented `search_smart` tool from MCP interface

## [0.1.0] - Initial Release

### Added
- BM25 full-text search with Tantivy
- Semantic vector search with sqlite-vec
- Graph analytics with PageRank and community detection
- Surgical file editing with line-level precision
- MCP protocol support for 8+ coding platforms
- Daemon mode for background watching
- Web UI for read-only access
- Multi-platform binary distribution
