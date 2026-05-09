# Changelog

All notable changes to Knowledge Loom will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive README with architecture diagrams and feature documentation
- GitHub Actions workflows for testing, building, and releasing
- Multi-platform binary support (Linux, macOS, Windows)
- Security vulnerability scanning with cargo-audit
- License compliance checking with cargo-deny
- MSRV set to Rust 1.75 for modern async trait support
- Standardized code formatting with rustfmt.toml
- Enhanced Clippy linting configuration

### Changed
- Updated installation process to use `.knowledge-loom` directory structure
- Improved platform detection and configuration for 8+ coding platforms
- Enhanced search engine with RRF merging and graph-fused search
- Improved documentation with Architecture.md and CONTRIBUTING.md

### Fixed
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
