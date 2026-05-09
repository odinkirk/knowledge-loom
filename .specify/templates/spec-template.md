# Feature Specification: [FEATURE NAME]

**Feature Branch**: `[###-feature-name]`  
**Created**: [DATE]  
**Status**: Draft  
**Input**: User description: "$ARGUMENTS"

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.
  
  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently
  - Deployed independently
  - Demonstrated to users independently
-->

### User Story 1 - [Brief Title] (Priority: P1)

[Describe this user journey in plain language]

**Why this priority**: [Explain the value and why it has this priority level]

**Independent Test**: [Describe how this can be tested independently - e.g., "Can be fully tested by [specific action] and delivers [specific value]"]

**Acceptance Scenarios**:

1. **Given** [initial state], **When** [action], **Then** [expected outcome]
2. **Given** [initial state], **When** [action], **Then** [expected outcome]

---

### User Story 2 - [Brief Title] (Priority: P2)

[Describe this user journey in plain language]

**Why this priority**: [Explain the value and why it has this priority level]

**Independent Test**: [Describe how this can be tested independently]

**Acceptance Scenarios**:

1. **Given** [initial state], **When** [action], **Then** [expected outcome]

---

### User Story 3 - [Brief Title] (Priority: P3)

[Describe this user journey in plain language]

**Why this priority**: [Explain the value and why it has this priority level]

**Independent Test**: [Describe how this can be tested independently]

**Acceptance Scenarios**:

1. **Given** [initial state], **When** [action], **Then** [expected outcome]

---

[Add more user stories as needed, each with an assigned priority]

### Edge Cases

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right edge cases.
-->

- What happens when [boundary condition]?
- How does system handle [error scenario]?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right functional requirements.
-->

### Functional Requirements

- **FR-001**: System MUST [specific capability, e.g., "allow users to create accounts"]
- **FR-002**: System MUST [specific capability, e.g., "validate email addresses"]  
- **FR-003**: Users MUST be able to [key interaction, e.g., "reset their password"]
- **FR-004**: System MUST [data requirement, e.g., "persist user preferences"]
- **FR-005**: System MUST [behavior, e.g., "log all security events"]

*Example of marking unclear requirements:*

- **FR-006**: System MUST authenticate users via [NEEDS CLARIFICATION: auth method not specified - email/password, SSO, OAuth?]
- **FR-007**: System MUST retain user data for [NEEDS CLARIFICATION: retention period not specified]

### Key Entities *(include if feature involves data)*

- **[Entity 1]**: [What it represents, key attributes without implementation]
- **[Entity 2]**: [What it represents, relationships to other entities]

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria.
  These must be technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: [Measurable metric, e.g., "Users can complete account creation in under 2 minutes"]
- **SC-002**: [Measurable metric, e.g., "System handles 1000 concurrent users without degradation"]
- **SC-003**: [User satisfaction metric, e.g., "90% of users successfully complete primary task on first attempt"]
- **SC-004**: [Business metric, e.g., "Reduce support tickets related to [X] by 50%"]

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
-->

- [Assumption about target users, e.g., "Users have stable internet connectivity"]
- [Assumption about scope boundaries, e.g., "Mobile support is out of scope for v1"]
- [Assumption about data/environment, e.g., "Existing authentication system will be reused"]
- [Dependency on existing system/service, e.g., "Requires access to the existing user profile API"]

## Knowledge Loom Specific Requirements

### MCP Protocol Requirements *(if feature involves MCP server)*

<!--
  ACTION REQUIRED: Fill out MCP-specific requirements if this feature
  involves changes to the MCP server or tools.
-->

- **MCP-001**: Tool MUST follow rmcp 1.2 specification
- **MCP-002**: Tool MUST maintain backward compatibility with existing clients
- **MCP-003**: Tool MUST include protocol tests in `tests/mcp_protocol_tests.rs`
- **MCP-004**: Tool MUST document tool signatures and return types
- **MCP-005**: Tool MUST handle errors gracefully and return appropriate error codes

### Search Engine Requirements *(if feature involves search)*

<!--
  ACTION REQUIRED: Fill out search-specific requirements if this feature
  involves BM25, vector, or graph search functionality.
-->

- **SEARCH-001**: Search MUST use RRF merging for multiple engines (if applicable)
- **SEARCH-002**: Search MUST return results with line_start/heading metadata for surgical editing
- **SEARCH-003**: Search MUST support top_k parameter for result limiting
- **SEARCH-004**: Search MUST handle empty queries gracefully
- **SEARCH-005**: Search MUST target <150ms for 10k documents (performance requirement)

### Graph Analytics Requirements *(if feature involves graph operations)*

<!--
  ACTION REQUIRED: Fill out graph-specific requirements if this feature
  involves wikilink graph, PageRank, or community detection.
-->

- **GRAPH-001**: Graph operations MUST use Petgraph for graph data structures
- **GRAPH-002**: Graph MUST support PageRank ranking
- **GRAPH-003**: Graph MUST support community detection
- **GRAPH-004**: Graph MUST support path finding between nodes
- **GRAPH-005**: Graph MUST handle disconnected graphs gracefully

### Performance Requirements *(if feature is performance-critical)*

<!--
  ACTION REQUIRED: Define performance requirements for this feature.
  Knowledge Loom is performance-critical; specify targets if applicable.
-->

- **PERF-001**: [Specific performance target, e.g., "Search operation <150ms for 10k documents"]
- **PERF-002**: [Memory constraint, e.g., "Index size <500MB for 10k documents"]
- **PERF-003**: [Concurrency requirement, e.g., "Support 10 concurrent search operations"]
- **PERF-004**: [Scalability target, e.g., "Linear performance degradation up to 100k documents"]

### Testing Requirements *(mandatory for all features)*

<!--
  ACTION REQUIRED: Define testing requirements for this feature.
  Knowledge Loom requires 80% minimum code coverage.
-->

- **TEST-001**: Unit tests MUST achieve 80% minimum code coverage
- **TEST-002**: Integration tests MUST be added for cross-module interactions
- **TEST-003**: Tests MUST use `test-vault/` for corpus-based testing (if applicable)
- **TEST-004**: Tests MUST be deterministic (no flaky tests)
- **TEST-005**: Error paths MUST be tested alongside success paths
- **TEST-006**: Performance tests MUST be added for critical paths (if applicable)

### Module Impact *(mandatory for all features)*

<!--
  ACTION REQUIRED: Identify which modules this feature affects.
  This helps with code review and testing strategy.
-->

**Affected Modules** (select all that apply):
- [ ] BM25 (`src/bm25.rs`)
- [ ] Graph (`src/graph.rs`)
- [ ] Search (`src/search.rs`)
- [ ] Embed (`src/embed/`)
- [ ] Server (`src/server.rs`)
- [ ] Edits (`src/edits.rs`)
- [ ] Daemon (`src/daemon.rs`)
- [ ] Vault (`src/vault.rs`)
- [ ] Web (`src/web.rs`)
- [ ] Other: [specify]

**New Modules Required** (if any):
- [ ] Yes - [describe new module]
- [ ] No

### Documentation Requirements *(mandatory for all features)*

<!--
  ACTION REQUIRED: Define documentation requirements for this feature.
-->

- **DOC-001**: Public functions MUST have doc comments (`///`)
- **DOC-002**: Complex algorithms MUST have inline comments
- **DOC-003**: Architecture changes MUST update `ARCHITECTURE.md`
- **DOC-004**: New features MUST update `CHANGELOG.md`
- **DOC-005**: Breaking changes MUST update migration guide (if applicable)
