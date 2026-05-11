# Feature Specification: KB_ROOT Auto-Population on Init

**Feature Branch**: `002-init-root-auto`
**Created**: 2025-05-11
**Status**: Draft
**Input**: User description: "On init, KB_ROOT should be auto-populated with the directory where the init happened, with an optional --document-root flag providing an alternate directory (built-in tilde expansion)."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Auto-Populate KB_ROOT on Init (Priority: P1)

When a user runs the init command in a directory, the system automatically sets KB_ROOT to that directory without requiring manual configuration.

**Why this priority**: This is the primary use case and provides the most common initialization pattern. Users expect the system to automatically detect and use their current working directory.

**Independent Test**: Can be fully tested by running `loom init` in a test directory and verifying that KB_ROOT is set to that directory without any additional flags or configuration.

**Acceptance Scenarios**:

1. **Given** a user is in directory `/home/user/projects/my-kb`, **When** they run `loom init`, **Then** KB_ROOT is automatically set to `/home/user/projects/my-kb`
2. **Given** a user is in directory `~/Documents/notes`, **When** they run `loom init`, **Then** KB_ROOT is automatically set to the expanded path `/home/user/Documents/notes`

---

### User Story 2 - Optional --document-root Flag (Priority: P2)

When a user wants to separate their knowledge base from their working directory, they can use the --document-root flag to specify an alternate location.

**Why this priority**: This is a secondary use case for users who want more control over where their knowledge base is stored. It's less common than the default behavior but still important for flexibility.

**Independent Test**: Can be fully tested by running `loom init --document-root ~/Documents/kb` and verifying that KB_ROOT is set to the expanded path, not the current working directory.

**Acceptance Scenarios**:

1. **Given** a user is in directory `/home/user/projects`, **When** they run `loom init --document-root ~/Documents/kb`, **Then** KB_ROOT is set to `/home/user/Documents/kb` (not `/home/user/projects`)
2. **Given** a user is in directory `/tmp`, **When** they run `loom init --document-root /var/data/kb`, **Then** KB_ROOT is set to `/var/data/kb`

---

### Edge Cases

- What happens when the specified directory does not exist?
- What happens when the path contains invalid characters or is malformed?
- What happens when tilde expansion fails (e.g., home directory not set)?
- What happens when the user does not have write permissions to the directory?
- What happens when both current directory and --document-root are specified?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST auto-populate KB_ROOT with the current working directory when init is run without --document-root flag
- **FR-002**: System MUST support --document-root flag to specify an alternate directory for KB_ROOT
- **FR-003**: System MUST expand tilde (~) in directory paths to the user's home directory
- **FR-004**: System MUST validate that the specified directory exists before setting KB_ROOT
- **FR-005**: System MUST provide clear error messages when directory validation fails
- **FR-006**: System MUST persist KB_ROOT configuration to a standard location for future use
- **FR-007**: System MUST handle relative paths by converting them to absolute paths

### Key Entities

- **KB_ROOT Configuration**: The path to the knowledge base root directory, stored as a string and used by all system components to locate markdown files and indexes

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can successfully initialize a knowledge base by running a single command without manual configuration
- **SC-002**: Users can specify an alternate directory using --document-root flag with tilde expansion working correctly
- **SC-003**: System provides clear error messages when directory validation fails (100% of error cases)
- **SC-004**: KB_ROOT configuration is persisted and available for subsequent operations (100% of successful inits)

## Assumptions

- KB_ROOT is stored in MCP configuration files (not a separate config file)
- KB_ROOT is a path string that supports tilde expansion (~) to the user's home directory
- The system has read and write access to the user's home directory for MCP configuration storage
- The system can detect the current working directory using standard OS APIs
- Tilde expansion follows standard shell conventions (~ expands to $HOME)
- The init command is a CLI tool that modifies MCP configuration files

## Knowledge Loom Specific Requirements

### MCP Protocol Requirements *(if feature involves MCP server)*

- **MCP-001**: Tool MUST follow rmcp 1.2 specification
- **MCP-002**: Tool MUST maintain backward compatibility with existing clients
- **MCP-003**: Tool MUST include protocol tests in `tests/mcp_protocol_tests.rs`
- **MCP-004**: Tool MUST document tool signatures and return types
- **MCP-005**: Tool MUST handle errors gracefully and return appropriate error codes

### Search Engine Requirements *(if feature involves search)*

- **SEARCH-001**: Search MUST use RRF merging for multiple engines (if applicable)
- **SEARCH-002**: Search MUST return results with line_start/heading metadata for surgical editing
- **SEARCH-003**: Search MUST support top_k parameter for result limiting
- **SEARCH-004**: Search MUST handle empty queries gracefully
- **SEARCH-005**: Search MUST target <150ms for 10k documents (performance requirement)

### Graph Analytics Requirements *(if feature involves graph operations)*

- **GRAPH-001**: Graph operations MUST use Petgraph for graph data structures
- **GRAPH-002**: Graph MUST support PageRank ranking
- **GRAPH-003**: Graph MUST support community detection
- **GRAPH-004**: Graph MUST support path finding between nodes
- **GRAPH-005**: Graph MUST handle disconnected graphs gracefully

### Performance Requirements *(if feature is performance-critical)*

- **PERF-001**: Init command completes in under 1 second for typical directory sizes
- **PERF-002**: Directory validation completes in under 100ms
- **PERF-003**: Tilde expansion completes in under 10ms
- **PERF-004**: Configuration persistence completes in under 50ms

### Testing Requirements *(mandatory for all features)*

- **TEST-001**: Unit tests MUST achieve 80% minimum code coverage
- **TEST-002**: Integration tests MUST be added for cross-module interactions
- **TEST-003**: Tests MUST use `test-vault/` for corpus-based testing (if applicable)
- **TEST-004**: Tests MUST be deterministic (no flaky tests)
- **TEST-005**: Error paths MUST be tested alongside success paths
- **TEST-006**: Performance tests MUST be added for critical paths (if applicable)

### Module Impact *(mandatory for all features)*

**Affected Modules** (select all that apply):
- [ ] BM25 (`src/bm25.rs`)
- [ ] Graph (`src/graph.rs`)
- [ ] Search (`src/search.rs`)
- [ ] Embed (`src/embed/`)
- [x] Server (`src/server.rs`)
- [x] Edits (`src/edits.rs`)
- [x] Vault (`src/vault.rs`)
- [ ] Web (`src/web.rs`)
- [ ] Other: CLI initialization and configuration

**New Modules Required** (if any):
- [ ] Yes - [describe new module]
- [x] No

### Documentation Requirements *(mandatory for all features)*

- **DOC-001**: Public functions MUST have doc comments (`///`)
- **DOC-002**: Complex algorithms MUST have inline comments
- **DOC-003**: Architecture changes MUST update `ARCHITECTURE.md`
- **DOC-004**: New features MUST update `CHANGELOG.md`
- **DOC-005**: Breaking changes MUST update migration guide (if applicable)
