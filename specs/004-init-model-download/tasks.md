# Tasks: Init-Time Model Download

**Input**: Design documents from `/specs/004-init-model-download/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are REQUIRED per Knowledge Loom constitution (TDD approach with 80% minimum coverage)

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

**Total Tasks**: 119 (45 for MVP: User Story 1 only)

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Knowledge Loom**: `src/`, `tests/` at repository root
- **Modules**: BM25 (`src/bm25.rs`), Graph (`src/graph.rs`), Search (`src/search.rs`), Embed (`src/embed/`), Server (`src/server.rs`), Edits (`src/edits.rs`), Daemon (`src/daemon.rs`), Vault (`src/vault.rs`), Web (`src/web.rs`)
- **New Modules**: Init (`src/init.rs`), Model (`src/model.rs`), Download (`src/download.rs`)
- **Tests**: `tests/` with `*_tests.rs` naming convention
- **Test corpus**: `test-vault/` for corpus-based testing

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [X] T001 Verify Rust toolchain: `rustc --version` (must be 1.75+)
- [X] T002 [P] Run `cargo fmt --all` to ensure code formatting
- [X] T003 [P] Run `cargo clippy -- -D warnings` to check for linting issues
- [X] T004 [P] Run `cargo test --all-features` to verify existing tests pass
- [X] T005 [P] Run `cargo deny check licenses bans sources` to verify dependency compliance
- [X] T006 Add new dependencies to Cargo.toml: reqwest, sha2, chrono, fs2
- [X] T007 [P] Create new module files: src/init.rs, src/model.rs, src/download.rs
- [X] T008 [P] Create new test files: tests/init_tests.rs, tests/model_tests.rs, tests/download_tests.rs
- [X] T009 Update src/lib.rs to include new modules (init, model, download)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T010 Implement error types in src/model.rs using thiserror (ModelError, DownloadError, InitError)
- [X] T011 [P] Implement data structures in src/model.rs (DownloadState, DownloadStatus, ModelMetadata, DownloadProgress)
- [X] T012 [P] Implement data structures in src/download.rs (DownloadProgress, DownloadManager)
- [X] T013 [P] Implement data structures in src/init.rs (InitManager, InitProgress)
- [X] T014 [P] Add constants in src/model.rs (MODEL_NAME, MODEL_VERSION, MODEL_URL, MODEL_FILE, STATE_FILE, LOCK_FILE)
- [X] T015 [P] Add constants in src/download.rs (MAX_RETRIES, RETRY_DELAY, TIMEOUT, BUFFER_SIZE, PROGRESS_UPDATE_INTERVAL)
- [X] T016 [P] Add helper functions in src/download.rs (format_download_progress, format_download_complete, format_download_error)
- [X] T017 [P] Add helper functions in src/download.rs (calculate_sha256_checksum, acquire_lock, release_lock)
- [X] T018 [P] Add signal handling infrastructure in src/download.rs (signal-hook dependency, INTERRUPTED flag)
- [X] T019 [P] Add HTTP Range request support in src/download.rs (Range header, resume logic)
- [X] T020 [P] Add proxy configuration support in src/download.rs (respect environment variables)
- [X] T021 [P] Add version mismatch detection in src/model.rs (version comparison, validation logic)
- [X] T022 [P] Add integration test scaffolding in tests/integration.rs for model download
- [X] T023 Configure test corpus in test-vault/ for model download testing

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Initial Model Download During Init (Priority: P1) 🎯 MVP

**Goal**: Download the required model during `loom init` with structured plain text progress indicators

**Independent Test**: Run `loom init` on a fresh installation and verify that the model downloads with structured plain text progress indicators before the init command completes

### Tests for User Story 1 (REQUIRED - TDD approach) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T020 [P] [US1] Unit test for ModelManager::new in tests/model_tests.rs
- [X] T021 [P] [US1] Unit test for ModelManager::is_model_valid in tests/model_tests.rs
- [X] T022 [P] [US1] Unit test for ModelManager::model_path in tests/model_tests.rs
- [X] T023 [P] [US1] Unit test for DownloadManager::new in tests/download_tests.rs
- [X] T024 [P] [US1] Unit test for DownloadManager::calculate_checksum in tests/download_tests.rs
- [X] T025 [P] [US1] Unit test for InitManager::new in tests/init_tests.rs
- [X] T026 [P] [US1] Unit test for InitManager::is_initialized in tests/init_tests.rs
- [X] T027 [P] [US1] Integration test for model download during init in tests/integration.rs
- [X] T028 [P] [US1] Integration test for progress display formatting in tests/integration.rs
- [X] T029 [P] [US1] Unit test for output conventions (println! vs eprintln!) in tests/download_tests.rs

### Implementation for User Story 1

- [X] T029 [US1] Implement ModelManager::new in src/model.rs
- [X] T030 [US1] Implement ModelManager::is_model_valid in src/model.rs
- [X] T031 [US1] Implement ModelManager::model_path in src/model.rs
- [X] T032 [US1] Implement DownloadManager::new in src/download.rs
- [X] T033 [US1] Implement DownloadManager::download in src/download.rs
- [X] T034 [US1] Implement DownloadManager::download_with_retry in src/download.rs
- [X] T035 [US1] Implement DownloadManager::calculate_checksum in src/download.rs
- [X] T036 [US1] Implement InitManager::new in src/init.rs
- [X] T037 [US1] Implement InitManager::is_initialized in src/init.rs
- [X] T038 [US1] Implement InitManager::initialize in src/init.rs
- [X] T039 [US1] Implement InitManager::create_directories in src/init.rs
- [X] T040 [US1] Implement InitManager::initialize_indexes in src/init.rs
- [X] T041 [US1] Implement InitManager::create_config_files in src/init.rs
- [X] T042 [US1] Update src/main.rs to add init command with progress display
- [X] T043 [US1] Add doc comments (`///`) to all public functions in src/model.rs
- [X] T044 [US1] Add doc comments (`///`) to all public functions in src/download.rs
- [X] T045 [US1] Add doc comments (`///`) to all public functions in src/init.rs

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - Graceful Error Handling During Download (Priority: P1)

**Goal**: Handle download errors gracefully with clear, actionable error messages

**Independent Test**: Simulate various error conditions (network failure, disk full, permission denied) during model download and verify that appropriate error messages are displayed

### Tests for User Story 2 (REQUIRED - TDD approach) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T046 [P] [US2] Unit test for network error handling in tests/download_tests.rs
- [ ] T047 [P] [US2] Unit test for disk full error handling in tests/download_tests.rs
- [ ] T048 [P] [US2] Unit test for permission denied error handling in tests/download_tests.rs
- [ ] T049 [P] [US2] Unit test for checksum mismatch error handling in tests/model_tests.rs
- [ ] T050 [P] [US2] Unit test for timeout error handling in tests/download_tests.rs
- [ ] T051 [P] [US2] Integration test for error message display in tests/integration.rs
- [ ] T052 [P] [US2] Unit test for proxy configuration in tests/download_tests.rs
- [ ] T053 [P] [US2] Unit test for proxy bypass rules in tests/download_tests.rs

### Implementation for User Story 2

- [ ] T052 [US2] Implement network error handling in src/download.rs
- [ ] T053 [US2] Implement disk full error handling in src/download.rs
- [ ] T054 [US2] Implement permission denied error handling in src/download.rs
- [ ] T055 [US2] Implement checksum mismatch error handling in src/model.rs
- [ ] T056 [US2] Implement timeout error handling in src/download.rs
- [ ] T057 [US2] Add error message formatting functions in src/download.rs
- [ ] T058 [US2] Update error types in src/model.rs with specific error variants
- [ ] T059 [US2] Update error types in src/download.rs with specific error variants
- [ ] T060 [US2] Add doc comments (`///`) to error handling functions in src/download.rs
- [ ] T061 [US2] Add doc comments (`///`) to error handling functions in src/model.rs

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - Model Re-Download with State Handling (Priority: P2)

**Goal**: Support model re-download with state handling similar to reindexing

**Independent Test**: Trigger a model re-download while another operation is in progress and verify that the system properly queues or rejects the request with appropriate messaging

### Tests for User Story 3 (REQUIRED - TDD approach) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T062 [P] [US3] Unit test for download state persistence in tests/model_tests.rs
- [ ] T063 [P] [US3] Unit test for download state recovery in tests/model_tests.rs
- [ ] T064 [P] [US3] Unit test for file locking in tests/model_tests.rs
- [ ] T065 [P] [US3] Unit test for concurrent download prevention in tests/model_tests.rs
- [ ] T066 [P] [US3] Integration test for model re-download in tests/integration.rs
- [ ] T067 [P] [US3] Integration test for concurrent download prevention in tests/integration.rs
- [ ] T068 [P] [US3] Unit test for HTTP Range request support in tests/download_tests.rs
- [ ] T069 [P] [US3] Unit test for download resume capability in tests/download_tests.rs
- [ ] T070 [P] [US3] Unit test for Ctrl+C signal handling in tests/download_tests.rs
- [ ] T071 [P] [US3] Unit test for signal cleanup and state preservation in tests/download_tests.rs
- [ ] T072 [P] [US3] Unit test for model version mismatch detection in tests/model_tests.rs
- [ ] T073 [P] [US3] Unit test for version re-download prompt in tests/model_tests.rs

### Implementation for User Story 3

- [ ] T078 [US3] Implement ModelManager::get_download_state in src/model.rs
- [ ] T079 [US3] Implement ModelManager::set_download_state in src/model.rs
- [ ] T080 [US3] Implement ModelManager::download_model in src/model.rs
- [ ] T081 [US3] Implement ModelManager::validate_model in src/model.rs
- [ ] T082 [US3] Implement ModelManager::delete_model in src/model.rs
- [ ] T083 [US3] Implement ModelManager::get_model_metadata in src/model.rs
- [ ] T084 [US3] Implement file locking in src/model.rs (acquire_lock, release_lock)
- [ ] T085 [US3] Implement state persistence during download in src/model.rs
- [ ] T086 [US3] Implement concurrent download prevention in src/model.rs
- [ ] T087 [US3] Implement HTTP Range request support in src/download.rs
- [ ] T088 [US3] Implement download resume capability in src/download.rs
- [ ] T089 [US3] Implement Ctrl+C signal handling in src/download.rs
- [ ] T090 [US3] Implement signal cleanup and state preservation in src/download.rs
- [ ] T091 [US3] Implement model version mismatch detection in src/model.rs
- [ ] T092 [US3] Implement version re-download prompt in src/model.rs
- [ ] T093 [US3] Add doc comments (`///`) to state management functions in src/model.rs
- [ ] T094 [US3] Add doc comments (`///`) to HTTP Range functions in src/download.rs
- [ ] T095 [US3] Add doc comments (`///`) to signal handling functions in src/download.rs
- [ ] T096 [US3] Add doc comments (`///`) to version mismatch functions in src/model.rs

**Checkpoint**: All user stories should now be independently functional

---

## Phase 6: User Story 4 - Manual Download Instructions as Fallback (Priority: P3)

**Goal**: Provide clear, step-by-step manual download instructions as a fallback mechanism

**Independent Test**: Verify that manual download instructions are available and accurate when automatic download fails

### Tests for User Story 4 (REQUIRED - TDD approach) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T097 [P] [US4] Unit test for manual download instructions generation in tests/init_tests.rs
- [ ] T098 [P] [US4] Integration test for manual download instructions display in tests/integration.rs

### Implementation for User Story 4

- [ ] T099 [US4] Implement manual download instructions generation in src/init.rs
- [ ] T100 [US4] Add manual download instructions to error messages in src/model.rs
- [ ] T101 [US4] Add manual download instructions to error messages in src/download.rs
- [ ] T102 [US4] Add doc comments (`///`) to manual download instructions functions in src/init.rs
- [ ] T103 [US4] Update README.md with manual download instructions

**Checkpoint**: All user stories should now be independently functional

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T104 [P] Add inline comments to complex algorithms in src/download.rs
- [ ] T105 [P] Add inline comments to complex algorithms in src/model.rs
- [ ] T106 [P] Add inline comments to complex algorithms in src/init.rs
- [ ] T107 [P] Update ARCHITECTURE.md with Model Download Flow section
- [ ] T108 [P] Update CHANGELOG.md with new features and changes
- [ ] T109 [P] Update README.md with manual download instructions
- [ ] T110 Code cleanup and refactoring across all new modules
- [ ] T111 Performance optimization (target: <5s SHA-256 validation, <10ms state checks)
- [ ] T112 [P] Run `cargo fmt --all -- --check` to verify formatting
- [ ] T113 [P] Run `cargo clippy -- -D warnings` to verify linting
- [ ] T114 [P] Run `cargo test --all-features` to verify all tests pass
- [ ] T115 [P] Run code coverage check (minimum 80% required)
- [ ] T116 [P] Run `cargo deny check licenses bans sources` for security
- [ ] T117 Security hardening and dependency updates
- [ ] T118 Verify all error messages use eprintln! instead of println! for debug output
- [ ] T119 Verify all progress indicators use println! for user-facing output

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P1 → P2 → P3)
- **Polish (Phase 7)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P1)**: Can start after Foundational (Phase 2) - May integrate with US1 but should be independently testable
- **User Story 3 (P2)**: Can start after Foundational (Phase 2) - May integrate with US1/US2 but should be independently testable
- **User Story 4 (P3)**: Can start after Foundational (Phase 2) - May integrate with US1/US2/US3 but should be independently testable

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD approach)
- Models before services
- Services before endpoints
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, all user stories can start in parallel (if team capacity allows)
- All tests for a user story marked [P] can run in parallel
- Models within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: "Unit test for ModelManager::new in tests/model_tests.rs"
Task: "Unit test for ModelManager::is_model_valid in tests/model_tests.rs"
Task: "Unit test for DownloadManager::new in tests/download_tests.rs"
Task: "Unit test for InitManager::new in tests/init_tests.rs"

# Launch all implementations for User Story 1 together:
Task: "Implement ModelManager::new in src/model.rs"
Task: "Implement DownloadManager::new in src/download.rs"
Task: "Implement InitManager::new in src/init.rs"
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
4. Add User Story 3 → Test independently → Run quality gates → Deploy/Demo
5. Add User Story 4 → Test independently → Run quality gates → Deploy/Demo
6. Each story adds value without breaking previous stories

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
   - Developer C: User Story 3
   - Developer D: User Story 4
3. Stories complete and integrate independently
4. Each developer runs quality gates before submitting PR

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing (TDD approach)
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
