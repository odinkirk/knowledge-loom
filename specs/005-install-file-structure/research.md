# Research Findings: install-file-structure

## Decision: Implementation Technology
- **Decision**: Rust binary via existing `loom` CLI (`loom install [--force]`)
- **Rationale**: Aligns with existing Rust codebase, enables proper error handling, integrates naturally with `loom` command structure
- **Alternatives Considered**: Shell script (would add maintenance burden, no error handling), Docker container (overkill for model download)

## Decision: Model Storage Location
- **Decision**: `.knowledge-loom/models/`
- **Rationale**: Already established pattern for runtime data; index data stays separate in `.knowledge-loom-index/`
- **Alternatives Considered**: `.knowledge-loom-index/models/` (would mix concerns), system-level cache (not portable)

## Decision: MCP Config Handling
- **Decision**: Leave `opencode.json`, `.mcp.json` untouched at repository root
- **Rationale**: Structural requirement - tools expect these at root; runtime data belongs in `.knowledge-loom`

## Decision: Integrity Verification
- **Decision**: SHA-256 checksum validation after download, re-download on mismatch
- **Rationale**: Industry standard, already used in feature 004, strong security properties

## Decision: Already-Installed Behavior
- **Decision**: Skip download if valid model exists, report "already installed", exit 0
- **Rationale**: Avoids redundant downloads; --force for explicit re-download

## Decision: Error Handling
- **Decision**: Clear error messages recommending `--force` on failure; partial cleanup on mid-download failure
- **Rationale**: User-friendly guidance prevents confusion about recovery path
