# Data Model: install-file-structure

## Entities

### Runtime Model
- **Location**: `.knowledge-loom/models/`
- **Contents**: fastembed model binary file, cache, config
- **Validation**: SHA-256 checksum verified after download

### Download State
- **State File**: `.knowledge-loom/models/.install-state.json`
- **Fields**: model_version, download_timestamp, checksum, size_bytes
- **Purpose**: Track whether download is valid without re-downloading

## Relationships

- Runtime Model → Index Directory (`.knowledge-loom-index/`): Independent, separate locations
- Runtime Model → MCP Configs (root `opencode.json`, `.mcp.json`): Independent, no dependency

## State Transitions

- **Not installed** → `loom install` → **Installed (validated)**
- **Installed (validated)** → `loom install --force` → **Re-downloading** → **Installed (validated)**
- **Installed (corrupted)** → `loom install` → **Re-downloading** → **Installed (validated)**
