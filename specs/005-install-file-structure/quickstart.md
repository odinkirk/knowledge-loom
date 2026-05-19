# Quick Start: Install Runtime Data

## Overview

`loom install` downloads fastembed model files into `.knowledge-loom/models/` with cache and config. MCP configuration files remain at root. Index data stays in `.knowledge-loom-index/`.

## Usage

```bash
# First-time installation
loom install

# Force re-download (overwrites existing model)
loom install --force
```

## What Gets Installed

- fastembed model binary → `.knowledge-loom/models/`
- Model cache → `.knowledge-loom/models/`
- Model config → `.knowledge-loom/models/`

## What Stays at Root

- `opencode.json`
- `.mcp.json`

## Behavior

| Scenario | Result |
|----------|--------|
| First run | Downloads model, verifies checksum, reports success |
| Model already valid | Reports "already installed", exits 0 |
| Model corrupted | Re-downloads automatically |
| Download fails | Clear error, recommends `--force` |
| `--force` flag | Re-downloads regardless of existing model |

## Verification

```bash
loom install
cargo test --release
```
