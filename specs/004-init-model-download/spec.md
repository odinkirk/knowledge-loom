# Feature Specification: Init-Time Model Download

**Feature Branch**: `004-init-model-download`
**Created**: 2026-05-12
**Status**: Draft
**Input**: User description: "Live-testing has revealed that downloading the model during initial indexing is unwieldy and introduces confusion. Model download should happen during init with progress indications and graceful error handling. In the rare event that model re-download is required, a similar state-handling mechanism to reindexing should happen with the same graceful error handling. If, for some reason, this backup downlod mechanism fails, the user (or their AI assistant) should be given manual download instructions."

## Clarifications

### Session 2026-05-12

- Q: What retry strategy should be used for failed model downloads? → A: Exponential backoff with 3 retries (1s, 2s, 4s delays)
- Q: What checksum algorithm should be used for model validation? → A: SHA-256 checksum (industry standard, strong security)
- Q: How should the model download URL be determined? → A: Hardcoded in binary (single model, no configuration)
- Q: Where should the model file be stored? → A: KB_ROOT/.knowledge-loom-index/models/ (follows existing pattern)
- Q: What format should progress indicators use? → A: Structured plain text format (human-readable and AI-parseable)
- Q: How should output conventions be handled to avoid MCP server instability? → A: Use `println!` ONLY in CLI context for user-facing progress indicators, use `eprintln!` for all debug/logging output
- Q: How should interrupted downloads be handled? → A: System MUST support resuming from last byte downloaded using HTTP Range requests
- Q: How should proxy configurations be handled? → A: System MUST respect system proxy environment variables (HTTP_PROXY, HTTPS_PROXY)
- Q: How should model version mismatch be detected? → A: System MUST detect version mismatch by comparing model metadata version with expected version
- Q: How should Ctrl+C signal handling be implemented? → A: System MUST catch SIGINT signal, clean up partial files, preserve download state for resume

## User Scenarios & Testing

### User Story 1 - Initial Model Download During Init (Priority: P1)

User runs the init command to set up their knowledge base. The system automatically downloads the required model with clear structured plain text progress indicators, ensuring the user (and AI assistants) can easily understand what's happening and when it will complete.

**Why this priority**: This is the primary use case - most users will run init once and expect the model to be available without manual intervention. Without this, users encounter confusing hangs during their first indexing operation. Structured plain text progress indicators ensure both humans and AI assistants can easily understand download progress.

**Independent Test**: Can be fully tested by running `loom init` on a fresh installation and verifying that the model downloads with structured plain text progress indicators before the init command completes. Delivers immediate value by eliminating the confusing first-time indexing hang.

**Acceptance Scenarios**:

1. **Given** a fresh installation with no model downloaded, **When** user runs `loom init`, **Then** the system downloads the model with structured plain text progress indicators and completes successfully
2. **Given** a fresh installation, **When** model download completes successfully, **Then** the init command finishes and the model is available for immediate use
3. **Given** a fresh installation, **When** model download is interrupted by network failure, **Then** the system provides a clear error message explaining what happened and how to retry

---

### User Story 2 - Graceful Error Handling During Download (Priority: P1)

User runs init and encounters a download error (network failure, disk full, permission denied, etc.). The system provides clear, actionable error messages that help the user understand what went wrong and how to fix it, with structured plain text format for any progress information displayed.

**Why this priority**: Download errors are common in real-world usage (network issues, disk space, permissions). Without clear error handling, users are left confused and unable to proceed.

**Independent Test**: Can be fully tested by simulating various error conditions (network failure, disk full, permission denied) during model download and verifying that appropriate error messages are displayed with structured plain text format for any progress information. Delivers value by preventing user frustration and support requests.

**Acceptance Scenarios**:

1. **Given** a fresh installation, **When** network fails during model download, **Then** the system displays a clear error message indicating network failure and suggests checking internet connection
2. **Given** a fresh installation, **When** disk is full during model download, **Then** the system displays a clear error message indicating insufficient disk space and suggests freeing up space
3. **Given** a fresh installation, **When** permission is denied during model download, **Then** the system displays a clear error message indicating permission issues and suggests checking file permissions

---

### User Story 3 - Model Re-Download with State Handling (Priority: P2)

User's model becomes corrupted or needs to be re-downloaded for some reason. The system provides a mechanism to re-download the model with similar state handling to reindexing - preventing concurrent operations and providing clear status messages with structured plain text progress indicators.

**Why this priority**: Model corruption or re-download needs are rare but possible. Having a proper state-handling mechanism prevents race conditions and confusing behavior when multiple operations attempt to download simultaneously. Structured plain text progress indicators ensure both humans and AI assistants can easily understand download progress.

**Independent Test**: Can be fully tested by triggering a model re-download while another operation is in progress and verifying that the system properly queues or rejects the request with appropriate messaging. Delivers value by preventing system instability and user confusion.

**Acceptance Scenarios**:

1. **Given** an existing installation with a corrupted model, **When** user triggers model re-download, **Then** the system downloads the model with structured plain text progress indicators and replaces the corrupted file
2. **Given** an existing installation, **When** model re-download is in progress and another operation requests the model, **Then** the system provides a clear message indicating that download is in progress and suggests waiting
3. **Given** an existing installation, **When** model re-download completes successfully, **Then** the system updates the model state and makes it available for use

---

### User Story 4 - Manual Download Instructions as Fallback (Priority: P3)

All automatic download mechanisms fail (network issues, proxy problems, etc.). The system provides clear, step-by-step manual download instructions that the user or their AI assistant can follow to obtain the model file, including examples of structured plain text progress format.

**Why this priority**: This is the ultimate fallback for edge cases where automatic download cannot work. While rare, having manual instructions ensures users can always proceed even in problematic environments, with clear examples of structured plain text progress format for both humans and AI assistants.

**Independent Test**: Can be fully tested by verifying that manual download instructions are available and accurate when automatic download fails, including examples of structured plain text progress format. Delivers value by providing a guaranteed path forward regardless of environment constraints.

**Acceptance Scenarios**:

1. **Given** an installation where all automatic download mechanisms have failed, **When** user requests manual download instructions, **Then** the system provides clear, step-by-step instructions for downloading the model manually, including examples of structured plain text progress format
2. **Given** manual download instructions provided, **When** user follows the instructions and places the model file in the correct location, **Then** the system recognizes the model and makes it available for use
3. **Given** manual download instructions provided, **When** user places the model file incorrectly, **Then** the system provides a clear error message indicating the correct location

---

### Edge Cases

- What happens when the user interrupts the download (Ctrl+C)? System MUST catch SIGINT signal, clean up partial files, preserve download state for potential resume using HTTP Range requests
- How does the system handle partial downloads that need to be resumed? System MUST support resuming from last byte downloaded using HTTP Range requests with Range header
- What happens if the model file already exists but is the wrong version? System MUST detect version mismatch by comparing model metadata version with expected version and prompt user for re-download
- How does the system handle concurrent init commands from multiple terminals? System MUST use file locking to prevent concurrent downloads
- What happens if the download URL changes or becomes unavailable? System MUST provide clear error message with manual download instructions
- How does the system handle proxy configurations or corporate firewalls? System MUST respect system proxy environment variables (HTTP_PROXY, HTTPS_PROXY, NO_PROXY)
- What happens if the user has multiple knowledge bases with different model requirements? System MUST support per-knowledge-base model storage
- How does the system handle non-interactive environments (CI/CD, AI assistants)? System MUST use structured plain text format for progress indicators that is both human-readable and machine-parseable
- How does the system ensure MCP server stability? System MUST use `println!` ONLY in CLI context for user-facing progress indicators, use `eprintln!` for all debug/logging output

## Requirements

### Functional Requirements

- **FR-001**: System MUST download the required model during the `loom init` command
- **FR-002**: System MUST display progress indicators during model download using structured plain text format (e.g., "Downloading model: 45% (45MB/100MB) - 2.3MB/s - 24s remaining") that is both human-readable and AI-parseable
- **FR-003**: System MUST handle download errors gracefully with clear, actionable error messages
- **FR-004**: System MUST support model re-download with state handling similar to reindexing
- **FR-005**: System MUST prevent concurrent model download operations
- **FR-006**: System MUST provide clear status messages when model download is in progress
- **FR-007**: System MUST provide manual download instructions as a fallback mechanism
- **FR-008**: System MUST validate the downloaded model file using SHA-256 checksum, file size, and format verification
- **FR-009**: System MUST handle interrupted downloads using exponential backoff with 3 retries (1s, 2s, 4s delays) before requiring manual intervention
- **FR-010**: System MUST store model download state (not started, in progress, completed, failed)
- **FR-011**: System MUST support resuming interrupted downloads using HTTP Range requests
- **FR-012**: System MUST respect system proxy environment variables (HTTP_PROXY, HTTPS_PROXY, NO_PROXY)
- **FR-013**: System MUST detect model version mismatch and prompt user for re-download
- **FR-014**: System MUST handle Ctrl+C signal by cleaning up partial files and preserving download state
- **FR-015**: System MUST use `println!` ONLY in CLI context for user-facing progress indicators, use `eprintln!` for all debug/logging output

### Key Entities

- **Model Download State**: Represents the current status of model download (not started, in progress, completed, failed), including progress percentage, download speed, and error information
- **Model File**: The downloaded model file stored in KB_ROOT/.knowledge-loom-index/models/, with metadata including version, SHA-256 checksum, and download timestamp
- **Download Progress Information**: Real-time information about download progress displayed in structured plain text format (e.g., "Downloading model: 45% (45MB/100MB) - 2.3MB/s - 24s remaining") including bytes downloaded, total bytes, percentage complete, download speed, and estimated time remaining
- **Signal Handler**: Handles Ctrl+C signal for graceful shutdown and cleanup
- **Proxy Configuration**: System proxy environment variables (HTTP_PROXY, HTTPS_PROXY, NO_PROXY) for corporate environments
- **Version Metadata**: Model version information for detecting version mismatches

## Success Criteria

### Measurable Outcomes

- **SC-001**: Users can complete `loom init` on a fresh installation in under 5 minutes (including model download)
- **SC-002**: Model download progress is displayed in structured plain text format and updated at least once per second during download
- **SC-003**: 95% of users successfully complete model download on first attempt without manual intervention
- **SC-004**: Error messages are clear and actionable - users can resolve 90% of download errors without support assistance
- **SC-005**: Model re-download completes successfully in under 3 minutes on average
- **SC-006**: Manual download instructions enable 100% of users to obtain the model even when automatic download fails

## Assumptions

- Users have internet connectivity available during init (or can obtain the model file through other means)
- The model file size is reasonable (<500MB) for download on typical internet connections
- The model download URL is hardcoded in the binary and stable
- Model files are stored in KB_ROOT/.knowledge-loom-index/models/ (follows existing index pattern)
- Progress indicators use structured plain text format that is both human-readable and AI-parseable
- Progress indicators use `println!` ONLY in CLI context for user-facing output, all debug/logging uses `eprintln!`
- Users have sufficient disk space for the model file
- Users have write permissions to the model storage location
- The model file format is stable and backward-compatible
- Existing fastembed library can be used for model download and management
- Model download state can be persisted in a simple file-based format
- Only one model version is supported (all-MiniLM-L6-v2)
- HTTP Range requests are supported by the download server for resuming interrupted downloads
- System proxy environment variables (HTTP_PROXY, HTTPS_PROXY, NO_PROXY) are available for corporate environments

## Knowledge Loom Specific Requirements

### MCP Protocol Requirements

This feature does not involve MCP protocol changes.

### Output Conventions

- **OUTPUT-001**: System MUST use `println!` ONLY in CLI context for user-facing progress indicators
- **OUTPUT-002**: System MUST use `eprintln!` for all debug/logging output to avoid MCP server instability
- **OUTPUT-003**: Progress indicators MUST use `println!` in CLI context, never in MCP server context

### Search Engine Requirements

This feature does not involve search engine changes.

### Graph Analytics Requirements

This feature does not involve graph analytics changes.

### Performance Requirements

- **PERF-001**: Structured plain text progress updates must be visible within 1 second of download start
- **PERF-002**: Model download state checks must complete in under 10ms
- **PERF-003**: SHA-256 checksum validation must complete in under 5 seconds for a 500MB model
- **PERF-004**: The `loom init` command must complete in under 5 minutes on a typical internet connection (10 Mbps)
- **PERF-005**: HTTP Range request resume must complete within 1 second of request initiation
- **PERF-006**: Ctrl+C signal handling and cleanup must complete within 500ms

### Testing Requirements

- **TEST-001**: Unit tests MUST achieve 80% minimum code coverage for model download logic
- **TEST-002**: Integration tests MUST verify model download during init with structured plain text progress indicators
- **TEST-003**: Tests MUST simulate various error conditions (network failure, disk full, permission denied)
- **TEST-004**: Tests MUST verify concurrent download prevention
- **TEST-005**: Tests MUST verify model validation (SHA-256 checksum, size, format)
- **TEST-006**: Tests MUST verify manual download instructions are accurate and complete
- **TEST-007**: Tests MUST verify download state persistence and recovery
- **TEST-008**: Tests MUST verify HTTP Range request support for resuming interrupted downloads
- **TEST-009**: Tests MUST verify proxy configuration support (HTTP_PROXY, HTTPS_PROXY, NO_PROXY)
- **TEST-010**: Tests MUST verify model version mismatch detection and re-download prompt
- **TEST-011**: Tests MUST verify Ctrl+C signal handling and cleanup
- **TEST-012**: Tests MUST verify output conventions (println! in CLI context, eprintln! for debug)

### Module Impact

**Affected Modules** (select all that apply):
- [ ] BM25 (`src/bm25.rs`)
- [ ] Graph (`src/graph.rs`)
- [ ] Search (`src/search.rs`)
- [x] Embed (`src/embed/`)
- [x] Server (`src/server.rs`)
- [ ] Edits (`src/edits.rs`)
- [x] Daemon (`src/daemon.rs`)
- [ ] Vault (`src/vault.rs`)
- [ ] Web (`src/web.rs`)
- [x] Other: `src/init.rs` (new module), `src/model.rs` (new module)

**New Modules Required**:
- [x] Yes - `src/init.rs`: Handles init command logic including model download
- [x] Yes - `src/model.rs`: Manages model download, validation, and state
- [x] Yes - `src/download.rs`: Handles download progress and error handling

### Documentation Requirements

- **DOC-001**: Public functions MUST have doc comments (`///`)
- **DOC-002**: Complex algorithms MUST have inline comments
- **DOC-003**: Architecture changes MUST update `ARCHITECTURE.md` (add Model Download Flow section)
- **DOC-004**: New features MUST update `CHANGELOG.md`
- **DOC-005**: Breaking changes MUST update migration guide (if applicable)
- **DOC-006**: Manual download instructions MUST be documented in `README.md` or separate troubleshooting guide, including examples of structured plain text progress format
- **DOC-007**: Output conventions MUST be documented in code comments (println! vs eprintln! usage)
- **DOC-008**: Edge case handling MUST be documented (HTTP Range requests, proxy support, signal handling)
