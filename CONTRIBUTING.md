# Contributing to Knowledge Loom

Thank you for your interest in contributing to Knowledge Loom! This project is a community-driven search and analytics engine for document collections.

## Development Environment

### Prerequisites
- **Rust 1.75+**: Knowledge Loom requires modern Rust features (Async Trait support).
- **Cargo**: Standard Rust package manager.
- **SQLite**: Local data storage.

### Setup
1. Clone the repository:
   ```bash
   git clone https://github.com/odinkirk/knowledge-loom.git
   cd knowledge-loom
   ```
2. Build the project:
   ```bash
   cargo build
   ```
3. Run tests:
   ```bash
   cargo test
   ```

## Coding Standards

- **Formatting**: Always run `cargo fmt` before submitting a pull request.
- **Linting**: Ensure `cargo clippy` passes without warnings.
- **Async**: Prefer `tokio` for asynchronous operations.
- **Documentation**: Document public functions and structs using doc comments (`///`).

## Testing Requirements

We maintain high test coverage for search logic, graph analytics, and MCP protocol compliance.
- Add unit tests for new features.
- Ensure integration tests pass for any search engine changes.
- Use `test-vault/` for corpus-based testing.

## Pull Request Process

1. Create a feature branch from `main` or `develop`.
2. Implement your changes with tests.
3. Ensure CI passes (Tests, Build, Audit, Deny).
4. Update `CHANGELOG.md` with your changes.
5. Submit the PR for review.

## Licensing

Knowledge Loom is dual-licensed under **MIT** and **Apache-2.0**. By contributing, you agree to release your contributions under these licenses. All third-party dependencies must be compatible with this dual-licensing policy.
