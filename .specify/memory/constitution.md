# Knowledge Loom Constitution

## Core Principles

### I. Rust-First Architecture
Knowledge Loom is a Rust-based search and analytics engine for document collections. All code must follow Rust best practices:
- Use idiomatic Rust patterns (Result<T, E> for error handling, Option<T> for nullable values)
- Prefer composition over inheritance
- Leverage Rust's ownership system for memory safety
- Use async/await with tokio for concurrent operations

### II. Modular Design
The codebase is organized into focused modules with clear boundaries:
- **BM25** (`src/bm25.rs`): Full-text search engine using Tantivy
- **Graph** (`src/graph.rs`): Wikilink graph analytics using Petgraph
- **Search** (`src/search.rs`): RRF-merged search orchestration
- **Embed** (`src/embed/`): Embedding providers (local, Ollama)
- **Server** (`src/server.rs`): MCP protocol implementation using rmcp
- **Edits** (`src/edits.rs`): Surgical file editing operations
- **Daemon** (`src/daemon.rs`): Background file watching with notify
- **Vault** (`src/vault.rs`): Markdown vault scanning with walkdir
- **Web** (`src/web.rs`): Optional web UI using Axum

Modules must depend only on what they need. Cross-module dependencies should be minimized and well-documented.

### III. Test-First Development (NON-NEGOTIABLE)
TDD is mandatory for all new features:
1. Write tests first → Get user approval → Tests fail → Then implement
2. Red-Green-Refactor cycle strictly enforced
3. All tests must pass before committing
4. **80% code coverage minimum** - features below this threshold cannot be merged

### IV. Integration Testing
Integration tests are required for:
- New search engine contracts (BM25, Vector, Graph)
- MCP protocol changes
- Inter-module communication (e.g., Search → BM25/Vector/Graph)
- File system operations (vault scanning, edits)
- Background services (daemon, file watching)

### V. Quality Gates
All code must pass these quality gates before merging:
- **Formatting**: `cargo fmt --all -- --check` must pass
- **Linting**: `cargo clippy -- -D warnings` must pass
- **Testing**: `cargo test --all-features` must pass
- **Coverage**: Minimum 80% line coverage (measured via tarpaulin or similar)
- **Security**: `cargo deny check licenses bans sources` must pass
- **CI**: All GitHub Actions workflows (build, test, audit) must pass

### VI. MCP Protocol Compliance
All MCP server changes must:
- Follow the rmcp 1.2 specification
- Maintain backward compatibility with existing clients
- Include protocol tests in `tests/mcp_protocol_tests.rs`
- Document tool signatures and return types

### VII. Performance Standards
Knowledge Loom is performance-critical:
- Target: ~150ms unified search for 10k documents
- Avoid blocking operations in async contexts
- Use appropriate data structures (e.g., Petgraph for graph operations)
- Profile performance bottlenecks before optimization

### VIII. Documentation Requirements
- All public functions and structs must have doc comments (`///`)
- Complex algorithms must have inline comments explaining the approach
- Architecture changes must update `ARCHITECTURE.md`
- New features must update `CHANGELOG.md`

### IX. Output Conventions (CRITICAL)
- **Use `eprintln!` instead of `println!`** — `println!` dirties stdio which causes the MCP server to panic
- Reserve `println!` only for user-facing CLI output where stdout is explicitly expected
- All debug/logging output must use `eprintln!` or proper logging frameworks
- This is non-negotiable for MCP server stability

### X. Code Exploration and Analysis
Use code-review-graph (CRG) tools for all code exploration and analysis tasks:
- **ALWAYS use CRG tools first** before Grep/Glob/Read for code exploration
- Use CRG for: understanding code structure, finding dependencies, impact analysis, code reviews
- Use CRG semantic search for finding code entities by name or keyword
- Use CRG graph queries for understanding relationships (callers, callees, imports)
- Use CRG change detection for code reviews and PR analysis
- **EXCEPTION**: Do NOT use CRG for Markdown files - use Knowledge Loom tools instead
- Knowledge Loom tools (`loom_*`) are the single entry point for Markdown operations

CRG tools provide graph-powered code understanding that should be leveraged for all Rust code analysis.

## Code Conventions

### Naming
- **Files**: snake_case (e.g., `bm25.rs`, `search_engine.rs`)
- **Functions/Variables**: snake_case (e.g., `search_and_retrieve`, `top_k`)
- **Types/Structs/Enums**: PascalCase (e.g., `SearchEngine`, `BM25Index`)
- **Constants**: SCREAMING_SNAKE_CASE (e.g., `MAX_RESULTS`)

### Error Handling
- Use `Result<T, E>` for fallible operations
- Use `anyhow::Error` or `thiserror::Error` for error types
- Provide context with `.context()` or custom error messages
- Never use `unwrap()` or `expect()` in production code

### Async Patterns
- Use `tokio` for async operations
- Prefer `async fn` over manual futures
- Use `tokio::spawn` for concurrent tasks when appropriate
- Handle cancellation gracefully

## Git Workflow

### Branching
- All feature work must be done on feature branches: `feature/<description>`
- Never commit directly to `main`
- Branch from the tip of `origin/main` (run `git fetch origin main` first)

### Commit Policy (CRITICAL)
**Explicit individual consent is required for each git commit.**
- Consent for one commit does NOT imply consent for future commits
- Each commit must be explicitly approved before execution
- No blanket consent or "go ahead" for multiple commits
- The agent must ask for consent before every single `git commit` command

### Commit Flow Guidance (NEW)

#### Correct Flows ✅

**Flow 1: Single Commit**
```
Agent: "I'm ready to commit these changes. Should I commit these changes? (yes/no)"
User: "yes"
Agent: git commit -m "feat(init): implement model download feature"
```

**Flow 2: Multiple Sequential Commits**
```
Agent: "Ready to commit feature implementation. Should I commit these changes? (yes/no)"
User: "yes"
Agent: git commit -m "feat(init): implement model download feature"

Agent: "Ready to commit documentation updates. Should I commit these changes? (yes/no)"
User: "yes"
Agent: git commit -m "docs: add troubleshooting section for model download"
```

**Flow 3: Commit with Message Approval**
```
Agent: "Ready to commit with message: 'feat(init): implement model download feature'. Should I commit these changes? (yes/no)"
User: "yes"
Agent: git commit -m "feat(init): implement model download feature"
```

#### Incorrect Flows ❌

**Incorrect 1: Blanket Consent**
```
❌ User: "Go ahead and commit all your changes"
✅ Agent: "I need explicit consent for each commit individually. Should I commit these changes? (yes/no)"
```

**Incorrect 2: Silent Commits**
```
❌ Agent: commits without asking
✅ Agent: "Should I commit these changes? (yes/no)"
```

**Incorrect 3: Multiple Commits in One Command**
```
❌ Agent: git add . && git commit -m "feat: multiple changes"
✅ Agent: "Should I commit these changes? (yes/no)"
```

#### Explicit Consent Requirements

**Valid Explicit Consent:**
- "yes"
- "commit"
- "proceed"
- "y"
- Any clear affirmative response

**Invalid Consent:**
- "go ahead" (implies blanket consent)
- "continue" (too vague)
- "ok" (ambiguous)
- No response (requires explicit consent)

#### Edge Cases

**Edge Case 1: Automated Tools**
```
Agent: "The build system requires a commit. Should I commit these changes? (yes/no)"
User: "yes"
Agent: git commit -m "chore: update dependencies"
```

**Edge Case 2: Emergency Commits**
```
Agent: "Critical security fix. Should I commit these changes immediately? (yes/no)"
User: "yes"
Agent: git commit -m "fix(security): patch critical vulnerability"
```

**Edge Case 3: Interactive Sessions**
```
Agent: "Should I commit these changes? (yes/no)"
User: "yes"
Agent: git commit -m "feat: implemented model download"
```

#### Constitutional Enforcement

**Violation Protocol:**
1. Any commit made without explicit consent is a constitutional violation
2. Must be documented and justified in the commit message
3. Requires retroactive approval from project maintainers
4. Future commits must follow explicit consent protocol

**Quality Gate Check:**
- All commits must verify constitutional compliance
- PRs must include "Constitution: ✅ PASSED" or "Constitution: ❌ VIOLATION [justification]"
- Audit logs must track consent verification for each commit

### Commit Messages
- Follow Conventional Commits format: `type(scope): description`
- Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`
- Examples:
  - `feat(search): add RRF merging for BM25 and vector results`
  - `fix(bm25): resolve race condition in index updates`
  - `refactor(graph): extract PageRank calculation to separate function`

### Pull Request Process
1. Create feature branch from `main`
2. Implement changes with tests (TDD)
3. Ensure all quality gates pass
4. Update documentation (ARCHITECTURE.md, CHANGELOG.md)
5. Submit PR for review
6. Address review feedback
7. Merge after approval

## Testing Requirements

### Test Organization
- **Unit tests**: In `tests/` directory with `*_tests.rs` naming
- **Integration tests**: In `tests/integration.rs`
- **Behavioral tests**: In `tests/behavioral_tests.rs`
- **Protocol tests**: In `tests/mcp_protocol_tests.rs`

### Test Coverage
- **Minimum 80% line coverage** for all modules
- Critical paths (search, indexing, MCP protocol) should have >90% coverage
- Use `test-vault/` for corpus-based testing
- Test both success and error paths

### Test Quality
- Tests must be deterministic (no flaky tests)
- Use `tempfile` for file system tests
- Mock external dependencies (Ollama, network calls)
- Include edge cases and error conditions

## Security Requirements

### Dependency Management
- All dependencies must pass `cargo deny check`
- Accepted licenses: MIT, Apache-2.0, BSD-3-Clause, and compatible licenses
- Regularly audit dependencies with `cargo audit`
- Update dependencies promptly for security vulnerabilities

### Code Security
- Never log secrets or sensitive data
- Validate all user inputs
- Use parameterized queries for database operations
- Follow Rust security best practices

## Performance Requirements

### Benchmarks
- Target: ~150ms unified search for 10k documents
- BM25 search: <50ms for 10k documents
- Vector search: <100ms for 10k documents
- Graph operations: <200ms for 10k nodes

### Optimization
- Profile before optimizing
- Use appropriate data structures
- Avoid unnecessary allocations
- Cache expensive operations

## Governance

### Constitution Authority
This constitution supersedes all other practices and conventions.
- All PRs and code reviews must verify compliance
- Violations must be explicitly justified and documented
- Use `AGENTS.md` for runtime development guidance

### Amendments
Constitution amendments require:
- Documentation of the change
- Approval from project maintainers
- Migration plan for existing code
- Update of this constitution's version

**Version**: 1.0.0 | **Ratified**: 2025-05-09 | **Last Amended**: 2025-05-09