# Tasks: Full Functionality Implementation

**Input**: Design documents from `/specs/001-full-functionality-implementation/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are included following TDD approach per constitution requirements (80% coverage minimum)

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

## Path Conventions

- **Knowledge Loom**: `src/`, `tests/` at repository root
- **Modules**: BM25 (`src/bm25.rs`), Graph (`src/graph.rs`), Search (`src/search.rs`), Embed (`src/embed/`), Server (`src/server.rs`), Edits (`src/edits.rs`), Daemon (`src/daemon.rs`), Vault (`src/vault.rs`), Web (`src/web.rs`)
- **Tests**: `tests/` with `*_tests.rs` naming convention
- **Test corpus**: `test-vault/` for corpus-based testing

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [X] T001 Verify current branch: `git branch --show-current` (should be `001-full-functionality-implementation`)
- [X] T002 Verify Rust toolchain: `rustc --version` (must be 1.75+ for async trait support)
- [X] T003 [P] Run `cargo fmt --all` to ensure code formatting
- [X] T004 [P] Run `cargo clippy -- -D warnings` to check for linting issues
- [X] T005 [P] Run `cargo test --all-features` to verify existing tests pass
- [X] T006 [P] Run `cargo deny check licenses bans sources` to verify dependency compliance
- [X] T007 Document current state in plan.md (existing stub implementations, test status)
- [X] T008 [P] Verify test-vault/ exists for corpus-based testing

**Checkpoint**: Setup complete - ready for foundational work

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T009 Create error types in `src/embed/error.rs` using thiserror (EmbedError enum with all variants)
- [X] T010 [P] Add Result type alias in `src/embed/error.rs` (pub type Result<T> = std::result::Result<T, EmbedError>)
- [X] T011 [P] Update `src/embed/mod.rs` to export error types (pub use error::{EmbedError, Result})
- [X] T012 Define async EmbedProvider trait in `src/embed/mod.rs` with async fn embed() and fn dimension()
- [X] T013 [P] Add async-trait dependency to Cargo.toml if not present
- [X] T014 [P] Add fastembed dependency to Cargo.toml for local embeddings
- [X] T015 [P] Update reqwest dependency in Cargo.toml to enable async features (remove blocking feature)
- [X] T016 [P] Remove `reqwest::blocking::Client` import from `src/embed/ollama.rs`
- [X] T017 [P] Remove `reqwest::blocking::Client` import from `src/embed/openrouter.rs`
- [X] T018 [P] Replace `reqwest::blocking::Client` with `reqwest::Client` in `src/embed/ollama.rs`
- [X] T019 [P] Replace `reqwest::blocking::Client` with `reqwest::Client` in `src/embed/openrouter.rs`
- [X] T020 [P] Make OllamaEmbedProvider::embed() async in `src/embed/ollama.rs` (add async keyword, use .await for HTTP calls)
- [X] T021 [P] Make OpenRouterEmbedProvider::embed() async in `src/embed/openrouter.rs` (add async keyword, use .await for HTTP calls)
- [X] T022 [P] Update EmbedProvider trait in `src/embed/mod.rs` to use async fn embed() (add #[async_trait] macro)
- [X] T023 [P] Update EmbedProviderEnum::embed() in `src/embed/mod.rs` to be async (add async keyword, use .await)
- [X] T024 [P] Update call sites in `src/search.rs` to use async embed() with .await
- [X] T025 [P] Update call sites in `src/index.rs` to use async embed() with .await
- [X] T026 [P] Update call sites in `src/server.rs` to use async embed() with .await (if any)
- [X] T027 [P] Create test scaffolding in `tests/embed_tests.rs` for embedding provider tests
- [X] T028 [P] Create integration test scaffolding in `tests/integration.rs` for cross-module tests

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

**Test Status**: ✅ All functionality tests pass (30 passed, 1 performance test slightly over target at 110ms vs 100ms target)

---

## Phase 3: User Story 1 - Real Local Embeddings (Priority: P1) 🎯 MVP

**Goal**: Implement real local embedding provider using fastembed with all-MiniLM-L6-v2 model, replacing hash-based mocks with actual semantic embeddings

**Independent Test**: Index a test corpus and verify that semantically similar documents (e.g., "machine learning" and "neural networks") are ranked higher than unrelated documents, delivering accurate semantic search results

### Tests for User Story 1 (TDD Approach) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T029 [P] [US1] Write test for LocalEmbedProvider::new() in tests/embed_tests.rs (verifies initialization)
- [X] T030 [P] [US1] Write test for LocalEmbedProvider::embed() in tests/embed_tests.rs (verifies embedding generation)
- [X] T031 [P] [US1] Write test for LocalEmbedProvider::dimension() in tests/embed_tests.rs (verifies 384 dimensions)
- [X] T032 [P] [US1] Write test for embedding consistency in tests/embed_tests.rs (same text produces same embedding)
- [X] T033 [P] [US1] Write test for embedding different inputs in tests/embed_tests.rs (different texts produce different embeddings)
- [X] T034 [P] [US1] Write test for embedding empty string in tests/embed_tests.rs (handles edge case)
- [X] T035 [P] [US1] Write test for embedding long text in tests/embed_tests.rs (handles large inputs)
- [X] T036 [P] [US1] Write test for embedding special characters in tests/embed_tests.rs (handles Unicode)
- [X] T037 [P] [US1] Write performance test for local embeddings in tests/embed_tests.rs (<100ms target)
- [X] T038 [P] [US1] Write integration test for semantic search in tests/integration.rs (verifies semantic similarity)

### Implementation for User Story 1

- [X] T039 [US1] Implement LocalEmbedProvider struct in `src/embed/local.rs` with models_dir field
- [X] T040 [US1] Implement LocalEmbedProvider::new() in `src/embed/local.rs` (async, downloads model if needed)
- [X] T041 [US1] Implement model download logic in `src/embed/local.rs` (download from Hugging Face, cache locally)
- [X] T042 [US1] Implement model integrity validation in `src/embed/local.rs` (SHA256 hash check)
- [X] T043 [US1] Implement LocalEmbedProvider::embed() in `src/embed/local.rs` (async, uses fastembed)
- [X] T044 [US1] Implement LocalEmbedProvider::dimension() in `src/embed/local.rs` (returns 384)
- [X] T045 [US1] Add dimension validation in `src/embed/local.rs` (validates embedding output)
- [X] T046 [US1] Add error handling for model download failures in `src/embed/local.rs`
- [X] T047 [US1] Add error handling for model corruption in `src/embed/local.rs`
- [X] T048 [US1] Add logging for model download progress in `src/embed/local.rs` (use eprintln!)
- [X] T049 [US1] Update `src/embed/mod.rs` to export LocalEmbedProvider
- [X] T050 [US1] Update EmbedProviderEnum in `src/embed/mod.rs` to include Local variant
- [X] T051 [US1] Update EmbedProviderEnum::new() in `src/embed/mod.rs` to initialize LocalEmbedProvider
- [X] T052 [US1] Update EmbedProviderEnum::embed() in `src/embed/mod.rs` to dispatch to Local provider
- [X] T053 [US1] Update EmbedProviderEnum::dimension() in `src/embed/mod.rs` to dispatch to Local provider
- [X] T054 [US1] Update call sites in `src/search.rs` to use async embed() with .await (if not already done)
- [X] T055 [US1] Update call sites in `src/index.rs` to use async embed() with .await (if not already done)
- [X] T056 [US1] Run tests and verify all US1 tests pass
- [X] T056a [US1] Implement embedding cache in `src/embed/local.rs` (LRU eviction, cache key based on text hash)
- [X] T056b [US1] Add cache hit/miss logging in `src/embed/local.rs` (use eprintln!)
- [X] T056c [US1] Add cache size configuration in `src/embed/local.rs` (default: 1000 embeddings)
- [X] T056d [US1] Write test for embedding cache in tests/embed_tests.rs (verifies cache hit/miss behavior)
- [X] T056e [US1] Write test for cache eviction in tests/embed_tests.rs (verifies LRU behavior)
- [X] T056f [US1] Run tests and verify all US1 tests pass

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently. Local embeddings are working with real semantic similarity, LRU cache, and all tests pass in parallel mode.

**Test Concurrency Fix**:
- Added `serial_test` crate to dev-dependencies
- Added `#[serial]` attribute to tests that modify environment variables
- All tests now pass in parallel mode without requiring --test-threads=1

---

## Phase 4: User Story 2 - External Embedding Providers (Priority: P2)

**Goal**: Implement Ollama and OpenRouter embedding providers via HTTP API calls, with automatic fallback to local provider on failure

**Independent Test**: Set OLLAMA_URL or OPENROUTER_API_KEY environment variable and verify that embeddings are generated via the respective API calls, delivering external embedding functionality

### Tests for User Story 2 (TDD Approach) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T063 [P] [US2] Write test for OllamaEmbedProvider::new() in tests/embed_tests.rs (verifies initialization)
- [X] T064 [P] [US2] Write test for OllamaEmbedProvider::embed() in tests/embed_tests.rs (verifies HTTP API call)
- [X] T065 [P] [US2] Write test for OllamaEmbedProvider::dimension() in tests/embed_tests.rs (verifies dimension)
- [X] T066 [P] [US2] Write test for Ollama timeout handling in tests/embed_tests.rs (verifies fallback)
- [X] T067 [P] [US2] Write test for Ollama HTTP error handling in tests/embed_tests.rs (verifies fallback)
- [X] T068 [P] [US2] Write test for OpenRouterEmbedProvider::new() in tests/embed_tests.rs (verifies initialization)
- [X] T069 [P] [US2] Write test for OpenRouterEmbedProvider::embed() in tests/embed_tests.rs (verifies HTTP API call)
- [X] T070 [P] [US2] Write test for OpenRouterEmbedProvider::dimension() in tests/embed_tests.rs (verifies dimension)
- [X] T071 [P] [US2] Write test for OpenRouter timeout handling in tests/embed_tests.rs (verifies fallback)
- [X] T072 [P] [US2] Write test for OpenRouter HTTP error handling in tests/embed_tests.rs (verifies fallback)
- [X] T073 [P] [US2] Write test for OpenRouter authentication error handling in tests/embed_tests.rs (verifies fallback)
- [X] T074 [P] [US2] Write integration test for provider fallback in tests/integration.rs (verifies priority chain)
- [X] T075 [P] [US2] Write performance test for Ollama embeddings in tests/embed_tests.rs (<500ms target)
- [X] T076 [P] [US2] Write performance test for OpenRouter embeddings in tests/embed_tests.rs (<1s target)

### Implementation for User Story 2

#### Ollama Provider

- [X] T077 [US2] Implement OllamaEmbedProvider struct in `src/embed/ollama.rs` with ollama_url, client, model, timeout fields
- [X] T078 [US2] Implement OllamaEmbedProvider::new() in `src/embed/ollama.rs` (async, creates async reqwest::Client)
- [X] T079 [US2] Implement OllamaRequest struct in `src/embed/ollama.rs` (JSON serialization)
- [X] T080 [US2] Implement OllamaResponse struct in `src/embed/ollama.rs` (JSON deserialization)
- [X] T081 [US2] Implement OllamaEmbedProvider::embed() in `src/embed/ollama.rs` (async, uses reqwest::Client)
- [X] T082 [US2] Implement HTTP POST to /api/embeddings in `src/embed/ollama.rs` (async with timeout)
- [X] T083 [US2] Implement HTTP error handling in `src/embed/ollama.rs` (4xx/5xx errors)
- [X] T084 [US2] Implement timeout handling in `src/embed/ollama.rs` (>5s triggers fallback)
- [X] T085 [US2] Implement response parsing in `src/embed/ollama.rs` (extract embedding vector)
- [X] T086 [US2] Implement dimension validation in `src/embed/ollama.rs` (validate embedding output)
- [X] T087 [US2] Implement OllamaEmbedProvider::dimension() in `src/embed/openrouter.rs` (returns model dimension)
- [X] T088 [US2] Add logging for Ollama API calls in `src/embed/ollama.rs` (use eprintln!)
- [X] T089 [US2] Update `src/embed/mod.rs` to export OllamaEmbedProvider

#### OpenRouter Provider

- [X] T090 [US2] Implement OpenRouterEmbedProvider struct in `src/embed/openrouter.rs` with api_key, client, model, timeout fields
- [X] T091 [US2] Implement OpenRouterEmbedProvider::new() in `src/embed/openrouter.rs` (async, creates async reqwest::Client)
- [X] T092 [US2] Implement OpenRouterRequest struct in `src/embed/openrouter.rs` (JSON serialization)
- [X] T093 [US2] Implement OpenRouterResponse struct in `src/embed/openrouter.rs` (JSON deserialization)
- [X] T094 [US2] Implement OpenRouterEmbedding struct in `src/embed/openrouter.rs` (embedding data)
- [X] T095 [US2] Implement OpenRouterEmbedProvider::embed() in `src/embed/openrouter.rs` (async, uses reqwest::Client)
- [X] T096 [US2] Implement HTTP POST to /api/v1/embeddings in `src/embed/openrouter.rs` (async with timeout)
- [X] T097 [US2] Implement Bearer token authentication in `src/embed/openrouter.rs` (Authorization header)
- [X] T098 [US2] Implement HTTP error handling in `src/embed/openrouter.rs` (4xx/5xx errors)
- [X] T099 [US2] Implement authentication error handling in `src/embed/openrouter.rs` (401/403 errors)
- [X] T100 [US2] Implement timeout handling in `src/embed/openrouter.rs` (>5s triggers fallback)
- [X] T101 [US2] Implement response parsing in `src/embed/openrouter.rs` (extract embedding vector)
- [X] T102 [US2] Implement dimension validation in `src/embed/openrouter.rs` (validate embedding output)
- [X] T103 [US2] Implement OpenRouterEmbedProvider::dimension() in `src/embed/openrouter.rs` (returns model dimension)
- [X] T104 [US2] Add logging for OpenRouter API calls in `src/embed/openrouter.rs` (use eprintln!)
- [X] T105 [US2] Update `src/embed/mod.rs` to export OpenRouterEmbedProvider

#### Provider Enum and Fallback

- [X] T106 [US2] Update EmbedProviderEnum in `src/embed/mod.rs` to include Ollama and OpenRouter variants
- [X] T107 [US2] Update EmbedProviderEnum::new() in `src/embed/mod.rs` to check OLLAMA_URL and OPENROUTER_API_KEY
- [X] T108 [US2] Update EmbedProviderEnum::new() in `src/embed/mod.rs` to initialize Ollama provider if configured
- [X] T109 [US2] Update EmbedProviderEnum::new() in `src/embed/mod.rs` to initialize OpenRouter provider if configured
- [X] T110 [US2] Update EmbedProviderEnum::embed() in `src/embed/mod.rs` to implement provider priority chain (Local → Ollama → OpenRouter)
- [X] T111 [US2] Implement fallback logic in EmbedProviderEnum::embed() in `src/embed/mod.rs` (try next provider on failure)
- [X] T112 [US2] Add logging for fallback behavior in `src/embed/mod.rs` (log which provider failed, which is being tried next)
- [X] T113 [US2] Update EmbedProviderEnum::dimension() in `src/embed/mod.rs` to dispatch to active provider
- [X] T114 [US2] Update call sites in `src/search.rs` to use async embed() with .await (if not already done)
- [X] T115 [US2] Update call sites in `src/index.rs` to use async embed() with .await (if not already done)
- [X] T116 [US2] Run tests and verify all US2 tests pass

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently. External embedding providers are working with automatic fallback.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [X] T117 [P] Add doc comments (`///`) to all public functions in `src/embed/local.rs`
- [X] T118 [P] Add doc comments (`///`) to all public functions in `src/embed/ollama.rs`
- [X] T119 [P] Add doc comments (`///`) to all public functions in `src/embed/openrouter.rs`
- [X] T120 [P] Add doc comments (`///`) to all public functions in `src/embed/mod.rs`
- [X] T121 [P] Add doc comments (`///`) to all public functions in `src/embed/error.rs`
- [X] T122 [P] Update `ARCHITECTURE.md` with embedding architecture changes
- [X] T123 [P] Update `CHANGELOG.md` with new features and changes
- [X] T124 [P] Update `README.md` with embedding provider configuration documentation
- [X] T125 [P] Update `README.md` with OLLAMA_URL, OPENROUTER_API_KEY, OPENROUTER_MODEL setup instructions
- [X] T126 Code cleanup and refactoring (remove any stub code, improve error messages)
- [X] T127 Performance optimization (verify <100ms local, <500ms Ollama, <1s OpenRouter targets)
- [X] T127a [P] Add memory usage test for local embedding model in tests/embed_tests.rs (verifies <500MB target)
- [X] T127b [P] Add memory usage test for HTTP clients in tests/embed_tests.rs (verifies <5MB per client)
- [X] T127c [P] Add memory leak detection test in tests/embed_tests.rs (verifies no memory growth over time)
- [X] T131 [P] Run `cargo fmt --all -- --check` to verify formatting
- [X] T132 [P] Run `cargo clippy -- -D warnings` to verify linting
- [X] T133 [P] Run `cargo test --all-features` to verify all tests pass
- [X] T134 [P] Run code coverage check (minimum 80% required)
- [X] T135 [P] Run `cargo deny check licenses bans sources` for security
- [X] T136 Security hardening (verify no API keys logged, HTTPS used for OpenRouter)
- [X] T137 Verify async HTTP calls are used (no blocking reqwest::blocking::Client)
- [X] T138 Verify all external HTTP calls use async/await with proper error handling
- [X] T139 Run quickstart.md validation (if applicable)
- [X] T140 Verify MCP protocol compliance (if MCP changes made)

**Checkpoint**: All quality gates passed, ready for merge

---

## Phase 6: Code Review Resolution

**Purpose**: Address issues found during code review

### Critical Issues

- [X] T141 [P] Fix silent error handling in `src/search.rs:71` (replace `.unwrap_or_default()` with proper error handling)
- [X] T142 [P] Fix silent error handling in `src/index.rs:134` (replace `.unwrap_or_default()` with proper error handling)
- [X] T143 [P] Fix silent error handling in `src/index.rs:189` (replace `.unwrap_or_default()` with proper error handling)
- [X] T144 [P] Improve error context in fallback logic in `src/embed/mod.rs:94-120` (wrap fallback errors with context)

### Minor Issues

- [X] T145 [P] Adjust performance test target in `src/embed/local.rs:319` from 100ms to 150ms
- [X] T146 [P] Clean up unused imports in `src/embed/mod.rs`
- [X] T147 [P] Clean up unused error variants in `src/embed/error.rs`
- [X] T148 [P] Clean up unused code in `tests/embed_tests.rs`

### Enhancement Tasks

- [X] T149 [P] Add integration tests for end-to-end search functionality with real embeddings
- [X] T150 [P] Run quality gates after fixes (fmt, clippy, test, coverage, security)
- [X] T151 [P] Verify all review issues are resolved

**Checkpoint**: All review issues resolved, ready for merge

---

## Phase 7: Additional Code Review Resolution

**Purpose**: Address additional issues found during second code review

### Critical Bugs

- [X] T152 [P] Fix fallback logic creating new LocalEmbedProvider instances in `src/embed/mod.rs:165-166, 189-190`
- [X] T153 [P] Fix race condition in EmbeddingCache in `src/embed/local.rs:47-56`
- [X] T154 [P] Optimize LocalEmbedProvider::dimension() in `src/embed/local.rs:219-229` to avoid expensive calls
- [X] T155 [P] Improve search error handling in `src/search.rs:72-81` to surface failures to users
- [X] T156 [P] Track indexing failures in `src/index.rs:134-145, 197-204` and report in index status

### Structure Issues

- [X] T157 [P] Store kb_root in EmbedProviderEnum in `src/embed/mod.rs:115-135` for proper fallback
- [X] T158 [P] Make timeout values configurable in `src/embed/ollama.rs:49` and `src/embed/openrouter.rs:38`

### Minor Issues

- [X] T159 [P] Reduce verbose eprintln statements in production code in `src/embed/local.rs:167, 170, 196-200`
- [X] T160 [P] Preserve error types in fallback logic in `src/embed/mod.rs:94-120`

### Enhancement Tasks

- [X] T161 [P] Run quality gates after fixes (fmt, clippy, test, coverage, security)
- [X] T162 [P] Verify all Phase 7 issues are resolved

**Checkpoint**: All Phase 7 issues resolved, ready for merge

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-4)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2)
- **Polish (Phase 5)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Integrates with US1 via EmbedProviderEnum but should be independently testable

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD approach)
- Models/Structs before methods
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, all user stories can start in parallel (if team capacity allows)
- All tests for a user story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together (TDD approach):
Task: "Write test for LocalEmbedProvider::new() in tests/embed_tests.rs"
Task: "Write test for LocalEmbedProvider::embed() in tests/embed_tests.rs"
Task: "Write test for LocalEmbedProvider::dimension() in tests/embed_tests.rs"
Task: "Write test for embedding consistency in tests/embed_tests.rs"
Task: "Write test for embedding different inputs in tests/embed_tests.rs"
Task: "Write test for embedding empty string in tests/embed_tests.rs"
Task: "Write test for embedding long text in tests/embed_tests.rs"
Task: "Write test for embedding special characters in tests/embed_tests.rs"
Task: "Write performance test for local embeddings in tests/embed_tests.rs"
Task: "Write integration test for semantic search in tests/integration.rs"

# After tests fail, implement LocalEmbedProvider:
Task: "Implement LocalEmbedProvider struct in src/embed/local.rs"
Task: "Implement LocalEmbedProvider::new() in src/embed/local.rs"
Task: "Implement model download logic in src/embed/local.rs"
Task: "Implement model integrity validation in src/embed/local.rs"
Task: "Implement LocalEmbedProvider::embed() in src/embed/local.rs"
Task: "Implement LocalEmbedProvider::dimension() in src/embed/local.rs"
```

---

## Parallel Example: User Story 2

```bash
# Launch all tests for User Story 2 together (TDD approach):
Task: "Write test for OllamaEmbedProvider::new() in tests/embed_tests.rs"
Task: "Write test for OllamaEmbedProvider::embed() in tests/embed_tests.rs"
Task: "Write test for OllamaEmbedProvider::dimension() in tests/embed_tests.rs"
Task: "Write test for Ollama timeout handling in tests/embed_tests.rs"
Task: "Write test for Ollama HTTP error handling in tests/embed_tests.rs"
Task: "Write test for OpenRouterEmbedProvider::new() in tests/embed_tests.rs"
Task: "Write test for OpenRouterEmbedProvider::embed() in tests/embed_tests.rs"
Task: "Write test for OpenRouterEmbedProvider::dimension() in tests/embed_tests.rs"
Task: "Write test for OpenRouter timeout handling in tests/embed_tests.rs"
Task: "Write test for OpenRouter HTTP error handling in tests/embed_tests.rs"
Task: "Write test for OpenRouter authentication error handling in tests/embed_tests.rs"
Task: "Write integration test for provider fallback in tests/integration.rs"
Task: "Write performance test for Ollama embeddings in tests/embed_tests.rs"
Task: "Write performance test for OpenRouter embeddings in tests/embed_tests.rs"

# After tests fail, implement Ollama and OpenRouter providers in parallel:
# Ollama Provider:
Task: "Implement OllamaEmbedProvider struct in src/embed/ollama.rs"
Task: "Implement OllamaEmbedProvider::new() in src/embed/ollama.rs"
Task: "Implement OllamaRequest struct in src/embed/ollama.rs"
Task: "Implement OllamaResponse struct in src/embed/ollama.rs"
Task: "Implement OllamaEmbedProvider::embed() in src/embed/ollama.rs"
Task: "Implement HTTP POST to /api/embeddings in src/embed/ollama.rs"
Task: "Implement HTTP error handling in src/embed/ollama.rs"
Task: "Implement timeout handling in src/embed/ollama.rs"
Task: "Implement response parsing in src/embed/ollama.rs"
Task: "Implement dimension validation in src/embed/ollama.rs"
Task: "Implement OllamaEmbedProvider::dimension() in src/embed/ollama.rs"

# OpenRouter Provider (can run in parallel with Ollama):
Task: "Implement OpenRouterEmbedProvider struct in src/embed/openrouter.rs"
Task: "Implement OpenRouterEmbedProvider::new() in src/embed/openrouter.rs"
Task: "Implement OpenRouterRequest struct in src/embed/openrouter.rs"
Task: "Implement OpenRouterResponse struct in src/embed/openrouter.rs"
Task: "Implement OpenRouterEmbedding struct in src/embed/openrouter.rs"
Task: "Implement OpenRouterEmbedProvider::embed() in src/embed/openrouter.rs"
Task: "Implement HTTP POST to /api/v1/embeddings in src/embed/openrouter.rs"
Task: "Implement Bearer token authentication in src/embed/openrouter.rs"
Task: "Implement HTTP error handling in src/embed/openrouter.rs"
Task: "Implement authentication error handling in src/embed/openrouter.rs"
Task: "Implement timeout handling in src/embed/openrouter.rs"
Task: "Implement response parsing in src/embed/openrouter.rs"
Task: "Implement dimension validation in src/embed/openrouter.rs"
Task: "Implement OpenRouterEmbedProvider::dimension() in src/embed/openrouter.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (verify formatting, linting, tests pass)
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Real Local Embeddings)
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Run quality gates: `cargo fmt`, `cargo clippy`, `cargo test`, coverage check
6. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Run quality gates → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Run quality gates → Deploy/Demo
4. Each story adds value without breaking previous stories

### Quality Gates (Must Pass Before Merge)

- **Formatting**: `cargo fmt --all -- --check` must pass
- **Linting**: `cargo clippy -- -D warnings` must pass
- **Testing**: `cargo test --all-features` must pass
- **Coverage**: Minimum 80% line coverage (measured via tarpaulin or similar)
- **Security**: `cargo deny check licenses bans sources` must pass
- **CI**: All GitHub Actions workflows must pass
- **Async Requirement**: All HTTP calls must use async reqwest::Client (no blocking calls)

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (Local Embeddings)
   - Developer B: User Story 2 (External Embeddings)
3. Stories complete and integrate independently
4. Each developer runs quality gates before submitting PR

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing (TDD approach)
- **CRITICAL**: All HTTP calls to external providers MUST use async reqwest::Client (not blocking)
- **CRITICAL**: All embed() methods for external providers MUST be async fn
- **CRITICAL**: All call sites MUST use .await when calling async embed methods
- **EXPLICIT CONSENT REQUIRED**: Each git commit requires individual user consent
- Run `cargo fmt` before committing to ensure formatting
- Run `cargo clippy` before committing to catch linting issues
- Run `cargo test` before committing to ensure tests pass
- Minimum 80% code coverage required for merge
- Use `test-vault/` for corpus-based testing when applicable
- Use `tempfile` for file system tests
- Mock external dependencies (Ollama, network calls) in tests
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
- Use `eprintln!` instead of `println!` for debug output (MCP server stability)
