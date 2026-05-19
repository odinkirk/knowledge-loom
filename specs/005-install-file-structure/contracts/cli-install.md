# CLI Contract: `loom install`

## Command

```
loom install [--force]
```

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `--force` | flag | No | Re-download model even if already installed |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Model installed successfully or already valid |
| 1 | Installation failed (network error, disk full, corruption) |

## Output

### Success (first install)
```
Installed fastembed v0.3.2 (120 MB) to .knowledge-loom/models/
```

### Already installed
```
fastembed model already installed and valid. Use --force to re-download.
```

### Error
```
ERROR: Download failed: [reason]
Run with --force to retry: loom install --force
```
