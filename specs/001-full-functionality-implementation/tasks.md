# Tasks: Full Functionality Implementation

**Input**: Design documents from `/specs/001-full-functionality-implementation/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are OPTIONAL - only include them if explicitly requested in the feature specification.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Knowledge Loom**: `src/`, `tests/` at repository root
- **Modules**: BM25 (`src/bm25.rs`), Graph (`src/graph.rs`), Search (`src/search.rs`), Embed (`src/embed/`), Server (`src/server.rs`), Edits (`src/edits.rs`), Daemon (`src/daemon.rs`), Vault (`src/vault.rs`), Web (`src/web.rs`)
- **Tests**: `tests/` with `*_tests.rs` naming convention
- **Test corpus**: `test-vault/` for corpus-based testing

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Verify Rust toolchain: `rustc --version` (must be 1.75+) - ✅ COMPLETED (Rust 1.91.0 installed)
- [x] T002 [P] Run `cargo fmt --all` to ensure code formatting - ✅ COMPLETED
- [x] T003 [P] Run `cargo clippy -- -D warnings` to check for linting issues - ✅ COMPLETED (fixed clippy warnings in embed providers)
- [x] T004 [P] Run `cargo test --all-features` to verify existing tests pass - ✅ COMPLETED (all tests passing)
- [x] T005 [P] Run `cargo deny check licenses bans sources` to verify dependency compliance - ✅ COMPLETED
- [x] T006 Add fastembed dependency to Cargo.toml - ✅ COMPLETED (added fastembed = "4")
- [x] T007 [P] Create test file structure in tests/embed_tests.rs - ✅ COMPLETED (created test file with 15 placeholder tests)

**Checkpoint**: Setup complete - ready for foundational work

**Notes**: 
- Fixed clippy warnings in embed providers (cast-lossless, missing_errors_doc, unused async)
- Changed embed providers from async to sync (no Mutex needed)
- Updated all embed provider references throughout codebase
- All existing tests pass after embed provider refactoring

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T008 Create error types module in src/embed/error.rs
- [ ] T009 [P] Define EmbedError enum variants in src/embed/error.rs
- [ ] T010 [P] Implement thiserror derive macro for EmbedError in src/embed/error.rs
- [ ] T011 Define EmbedProvider trait in src/embed/mod.rs
- [ ] T012 [P] Add async trait attribute to EmbedProvider in src/embed/mod.rs
- [ ] T013 [P] Define embed method signature in EmbedProvider trait in src/embed/mod.rs
- [ ] T014 [P] Define dimension method signature in EmbedProvider trait in src/embed/mod.rs
- [ ] T015 Define EmbedProviderEnum struct in src/embed/mod.rs
- [ ] T016 [P] Add local field to EmbedProviderEnum in src/embed/mod.rs
- [ ] T017 [P] Add ollama field to EmbedProviderEnum in src/embed/mod.rs
- [ ] T018 [P] Add openrouter field to EmbedProviderEnum in src/embed/mod.rs
- [ ] T019 [P] Add use_ollama field to EmbedProviderEnum in src/embed/mod.rs
- [ ] T020 [P] Add use_openrouter field to EmbedProviderEnum in src/embed/mod.rs
- [ ] T021 Implement EmbedProviderEnum::new in src/embed/mod.rs
- [ ] T022 [P] Implement EmbedProviderEnum::embed in src/embed/mod.rs
- [ ] T023 [P] Implement EmbedProviderEnum::dimension in src/embed/mod.rs
- [ ] T024 [P] Add integration test scaffolding in tests/integration.rs
- [ ] T025 [P] Setup performance benchmarks in tests/benchmarks.rs

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Real Local Embeddings (Priority: P1) 🎯 MVP

**Goal**: Implement local embedding provider using fastembed with all-MiniLM-L6-v2 model, enabling accurate semantic search without external dependencies

**Independent Test**: Can be fully tested by indexing a test corpus and verifying that semantically similar documents (e.g., "machine learning" and "neural networks") are ranked higher than unrelated documents, delivering accurate semantic search results

### Implementation for User Story 1

- [ ] T026 [P] Create LocalEmbedProvider struct in src/embed/local.rs
- [ ] T027 [P] Add models_dir field to LocalEmbedProvider in src/embed/local.rs
- [ ] T028 [P] Add model field to LocalEmbedProvider in src/embed/local.rs
- [ ] T029 Implement LocalEmbedProvider::new in src/embed/local.rs
- [ ] T030 [P] Implement LocalEmbedProvider::embed in src/embed/local.rs
- [ ] T031 [P] Implement LocalEmbedProvider::dimension in src/embed/local.rs
- [ ] T032 [P] Implement model download logic in src/embed/local.rs
- [ ] T033 [P] Implement model caching in src/embed/local.rs
- [ ] T034 [P] Implement model integrity validation in src/embed/local.rs
- [ ] T035 [P] Add SHA256 hash validation in src/embed/local.rs
- [ ] T036 [P] Implement model auto-retry on corruption in src/embed/local.rs
- [ ] T037 [P] Add logging for model operations in src/embed/local.rs
- [ ] T038 [P] Add error handling for model download in src/embed/local.rs
- [ ] T039 [P] Add error handling for model loading in src/embed/local.rs
- [ ] T040 [P] Add error handling for embedding generation in src/embed/local.rs
- [ ] T041 [P] Add unit tests for LocalEmbedProvider in tests/embed_tests.rs
- [ ] T042 [P] Add integration tests for local provider in tests/integration.rs
- [ ] T043 [P] Add performance benchmarks for local provider in tests/benchmarks.rs

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - External Embedding Providers (Priority: P2)

**Goal**: Implement Ollama and OpenRouter embedding providers with HTTP API integration, automatic fallback on failure, and configurable priority

**Independent Test**: Can be fully tested by setting OLLAMA_URL or OPENROUTER_API_KEY environment variable and verifying that embeddings are generated via the respective API calls, delivering external embedding functionality

### Implementation for User Story 2

- [ ] T044 [P] Create OllamaEmbedProvider struct in src/embed/ollama.rs
- [ ] T045 [P] Add ollama_url field to OllamaEmbedProvider in src/embed/ollama.rs
- [ ] T046 [P] Add model field to OllamaEmbedProvider in src/embed/ollama.rs
- [ ] T047 [P] Add client field to OllamaEmbedProvider in src/embed/ollama.rs
- [ ] T048 [P] Add timeout field to OllamaEmbedProvider in src/embed/ollama.rs
- [ ] T049 [P] Implement OllamaEmbedProvider::new in src/embed/ollama.rs
- [ ] T050 [P] Implement OllamaEmbedProvider::embed in src/embed/ollama.rs
- [ ] T051 [P] Implement OllamaEmbedProvider::dimension in src/embed/ollama.rs
- [ ] T052 [P] Implement HTTP request logic in src/embed/ollama.rs
- [ ] T053 [P] Implement JSON serialization in src/embed/ollama.rs
- [ ] T054 [P] Implement JSON deserialization in src/embed/ollama.rs
- [ ] T055 [P] Implement timeout handling in src/embed/ollama.rs
- [ ] T056 [P] Implement HTTP error handling in src/embed/ollama.rs
- [ ] T057 [P] Implement dimension validation in src/embed/ollama.rs
- [ ] T058 [P] Add logging for Ollama operations in src/embed/ollama.rs
- [ ] T059 [P] Add error handling for network issues in src/embed/ollama.rs
- [ ] T060 [P] Create OpenRouterEmbedProvider struct in src/embed/openrouter.rs
- [ ] T061 [P] Add api_key field to OpenRouterEmbedProvider in src/embed/openrouter.rs
- [ ] T062 [P] Add model field to OpenRouterEmbedProvider in src/embed/openrouter.rs
- [ ] T063 [P] Add client field to OpenRouterEmbedProvider in src/embed/openrouter.rs
- [ ] T064 [P] Add timeout field to OpenRouterEmbedProvider in src/embed/openrouter.rs
- [ ] T065 [P] Implement OpenRouterEmbedProvider::new in src/embed/openrouter.rs
- [ ] T066 [P] Implement OpenRouterEmbedProvider::embed in src/embed/openrouter.rs
- [ ] T067 [P] Implement OpenRouterEmbedProvider::dimension in src/embed/openrouter.rs
- [ ] T068 [P] Implement HTTP request logic in src/embed/openrouter.rs
- [ ] T069 [P] Implement Bearer token authentication in src/embed/openrouter.rs
- [ ] T070 [P] Implement JSON serialization in src/embed/openrouter.rs
- [ ] T071 [P] Implement JSON deserialization in src/embed/openrouter.rs
- [ ] T072 [P] Implement timeout handling in src/embed/openrouter.rs
- [ ] T073 [P] Implement HTTP error handling in src/embed/openrouter.rs
- [ ] T074 [P] Implement dimension validation in src/embed/openrouter.rs
- [ ] T075 [P] Add logging for OpenRouter operations in src/embed/openrouter.rs
- [ ] T076 [P] Add error handling for network issues in src/embed/openrouter.rs
- [ ] T077 [P] Implement provider priority chain in src/embed/mod.rs
- [ ] T078 [P] Implement fallback logic in EmbedProviderEnum::embed in src/embed/mod.rs
- [ ] T079 [P] Add warning logging for provider failures in src/embed/mod.rs
- [ ] T080 [P] Add unit tests for OllamaEmbedProvider in tests/embed_tests.rs
- [ ] T081 [P] Add unit tests for OpenRouterEmbedProvider in tests/embed_tests.rs
- [ ] T082 [P] Add integration tests for provider switching in tests/integration.rs
- [ ] T083 [P] Add integration tests for fallback behavior in tests/integration.rs
- [ ] T084 [P] Add performance benchmarks for Ollama provider in tests/benchmarks.rs
- [ ] T085 [P] Add performance benchmarks for OpenRouter provider in tests/benchmarks.rs

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T086 [P] Add doc comments (`///`) to all public functions in src/embed/
- [ ] T087 [P] Add doc comments (`///`) to all public structs in src/embed/
- [ ] T088 [P] Add inline comments for complex algorithms in src/embed/
- [ ] T089 [P] Update ARCHITECTURE.md with embedding architecture changes
- [ ] T090 [P] Update CHANGELOG.md with new embedding features
- [ ] T091 [P] Update README.md with embedding provider configuration
- [ ] T092 [P] Update README.md with OLLAMA_URL setup instructions
- [ ] T093 [P] Update README.md with OPENROUTER_API_KEY setup instructions
- [ ] T094 [P] Update README.md with OPENROUTER_MODEL setup instructions
- [ ] T095 [P] Update README.md with fallback behavior documentation
- [ ] T096 Code cleanup and refactoring in src/embed/
- [ ] T097 Performance optimization for embedding generation
- [ ] T098 [P] Run `cargo fmt --all -- --check` to verify formatting
- [ ] T099 [P] Run `cargo clippy -- -D warnings` to verify linting
- [ ] T100 [P] Run `cargo test --all-features` to verify all tests pass
- [ ] T101 [P] Run code coverage check (minimum 80% required)
- [ ] T102 [P] Run `cargo deny check licenses bans sources` for security
- [ ] T103 Security hardening and dependency updates
- [ ] T104 Verify MCP protocol compliance (no MCP changes in this feature)
- [ ] T105 Verify performance targets (<100ms local, <500ms Ollama, <1s OpenRouter)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User Story 1 (P1): Can start after Foundational - No dependencies on US2
  - User Story 2 (P2): Can start after Foundational - Integrates with US1 but independently testable
- **Polish (Phase 5)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Integrates with US1 via EmbedProviderEnum but independently testable

### Within Each User Story

- Models before services
- Services before endpoints
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, User Story 1 and User Story 2 can start in parallel
- All tests for a user story marked [P] can run in parallel
- Models within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: "Add unit tests for LocalEmbedProvider in tests/embed_tests.rs"
Task: "Add integration tests for local provider in tests/integration.rs"
Task: "Add performance benchmarks for local provider in tests/benchmarks.rs"

# Launch all model setup tasks together:
Task: "Create LocalEmbedProvider struct in src/embed/local.rs"
Task: "Add models_dir field to LocalEmbedProvider in src/embed/local.rs"
Task: "Add model field to LocalEmbedProvider in src/embed/local.rs"
```

---

## Parallel Example: User Story 2

```bash
# Launch Ollama provider tasks together:
Task: "Create OllamaEmbedProvider struct in src/embed/ollama.rs"
Task: "Add ollama_url field to OllamaEmbedProvider in src/embed/ollama.rs"
Task: "Add model field to OllamaEmbedProvider in src/embed/ollama.rs"
Task: "Add client field to OllamaEmbedProvider in src/embed/ollama.rs"
Task: "Add timeout field to OllamaEmbedProvider in src/embed/ollama.rs"

# Launch OpenRouter provider tasks together:
Task: "Create OpenRouterEmbedProvider struct in src/embed/openrouter.rs"
Task: "Add api_key field to OpenRouterEmbedProvider in src/embed/openrouter.rs"
Task: "Add model field to OpenRouterEmbedProvider in src/embed/openrouter.rs"
Task: "Add client field to OpenRouterEmbedProvider in src/embed/openrouter.rs"
Task: "Add timeout field to OpenRouterEmbedProvider in src/embed/openrouter.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (verify formatting, linting, tests pass)
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
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

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1
   - Developer B: User Story 2
3. Stories complete and integrate independently
4. Each developer runs quality gates before submitting PR

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- **EXPLICIT CONSENT REQUIRED**: Each git commit requires individual user consent
- Run `cargo fmt` before committing to ensure formatting
- Run `cargo clippy` before committing to catch linting issues
- Run `cargo test` before committing to ensure tests pass
- Minimum 80% code coverage required for merge
- Use `test-vault/` for corpus-based testing when applicable
- Use `tempfile` for file system tests
- Mock external dependencies (Ollama, network calls)
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
