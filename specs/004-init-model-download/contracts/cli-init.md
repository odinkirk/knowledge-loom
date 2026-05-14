# CLI Contract: Init Command

**Feature**: Init-Time Model Download
**Date**: 2026-05-12
**Status**: Complete

## Overview

This document defines the CLI contract for the `loom init` command, including command syntax, options, behavior, and output format.

## Command Syntax

### Basic Usage

```bash
loom init [OPTIONS]
```

### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--kb-root` | String | Current directory | Knowledge base root directory |
| `--force` | Boolean | false | Force re-initialization even if already initialized |
| `--help` | Boolean | false | Display help information |
| `--version` | Boolean | false | Display version information |

## Behavior

### Initialization Flow

1. **Check if already initialized**:
   - If `--force` is not set and already initialized: Display error and exit
   - If `--force` is set or not initialized: Proceed with initialization

2. **Create directory structure**:
   - Create `{KB_ROOT}/.knowledge-loom-index/` if it doesn't exist
   - Create `{KB_ROOT}/.knowledge-loom-index/models/` if it doesn't exist
   - Create `{KB_ROOT}/.knowledge-loom-index/tantivy/` if it doesn't exist

3. **Download model**:
   - Check if model already exists and is valid
   - If model exists and is valid: Skip download
   - If model doesn't exist or is invalid: Download model with progress indicators

4. **Initialize indexes**:
   - Initialize BM25 index
   - Initialize vector index
   - Initialize graph index

5. **Create configuration files**:
   - Create `.mcp.json` if it doesn't exist
   - Create `opencode.json` if it doesn't exist
   - Create `loom-shell.sh` if it doesn't exist

6. **Display completion message**:
   - Display success message with next steps

### Model Download Behavior

#### Download Progress

During model download, display structured plain text progress updates:

```
Downloading model: {percentage}% ({downloaded_mb}MB/{total_mb}MB) - {speed_mb}MB/s - {remaining}s remaining
```

**Update Frequency**: At least once per second

**Example Output**:

```
Downloading model: 0% (0MB/120MB) - 0MB/s - 0s remaining
Downloading model: 10% (12MB/120MB) - 2.5MB/s - 43s remaining
Downloading model: 20% (24MB/120MB) - 2.5MB/s - 38s remaining
...
Downloading model: 100% (120MB/120MB) - 2.5MB/s - 48s total
```

#### Download Completion

On successful download completion:

```
Model download complete: 100% (120MB/120MB) - 2.5MB/s - 48s total
Validating model... OK
```

#### Download Failure

On download failure:

```
Model download failed: Network error: Connection timeout
Please check your internet connection and try again.
```

#### Download Retry

On download retry (after failure):

```
Retrying model download (attempt 1 of 3)...
Downloading model: 0% (0MB/120MB) - 0MB/s - 0s remaining
```

#### Download in Progress

If download is already in progress:

```
Model download is already in progress. Please wait for the current download to complete.
```

### Error Handling

#### Network Errors

```
Network error: {details}
Please check your internet connection and try again.
```

#### Disk Full

```
Insufficient disk space: {required_mb}MB required, {available_mb}MB available
Please free up space and try again.
```

#### Permission Denied

```
Permission denied: Cannot write to {path}
Please check file permissions and try again.
```

#### Checksum Mismatch

```
Model validation failed: Checksum mismatch
The downloaded file may be corrupted. Please try downloading again.
```

#### Timeout

```
Download timeout: The download took too long
Please check your internet connection and try again.
```

## Output Format

### Success Output

```
knowledge-loom init complete.
  binary:  /path/to/loom
  KB_ROOT: /path/to/knowledge-base
  .mcp.json updated
  opencode.json updated
  loom-shell.sh created

Next: restart Claude Code and run /mcp to connect.
```

### Error Output

```
Error: {error message}

Please fix the issue and run `loom init` again.
```

### Help Output

```
Initialize a knowledge base with model download and index setup.

USAGE:
    loom init [OPTIONS]

OPTIONS:
    --kb-root <PATH>    Knowledge base root directory [default: current directory]
    --force             Force re-initialization even if already initialized
    -h, --help          Display help information
    -V, --version       Display version information
```

## Exit Codes

| Exit Code | Description |
|-----------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Network error |
| 3 | Disk full |
| 4 | Permission denied |
| 5 | Checksum mismatch |
| 6 | Timeout |
| 7 | Already initialized (without --force) |
| 8 | Download in progress |

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `KB_ROOT` | Knowledge base root directory | Current directory |
| `LOOM_MODEL_DIR` | Model directory override | `{KB_ROOT}/.knowledge-loom-index/models/` |
| `LOOM_DOWNLOAD_TIMEOUT` | Download timeout in seconds | 30 |
| `LOOM_MAX_RETRIES` | Maximum download retries | 3 |
| `HTTP_PROXY` | HTTP proxy URL | None |
| `HTTPS_PROXY` | HTTPS proxy URL | None |

## Examples

### Basic Initialization

```bash
loom init
```

Output:

```
Downloading model: 0% (0MB/120MB) - 0MB/s - 0s remaining
Downloading model: 10% (12MB/120MB) - 2.5MB/s - 43s remaining
...
Downloading model: 100% (120MB/120MB) - 2.5MB/s - 48s total
Validating model... OK
knowledge-loom init complete.
  binary:  /path/to/loom
  KB_ROOT: /path/to/knowledge-base
  .mcp.json updated
  opencode.json updated
  loom-shell.sh created

Next: restart Claude Code and run /mcp to connect.
```

### Initialization with Custom KB_ROOT

```bash
loom init --kb-root /path/to/knowledge-base
```

### Force Re-initialization

```bash
loom init --force
```

### Initialization with Proxy

```bash
HTTP_PROXY=http://proxy.example.com:8080 loom init
```

## Testing

### Test Cases

1. **Basic initialization**:
   - Run `loom init` on fresh installation
   - Verify model downloads with progress indicators
   - Verify indexes are created
   - Verify configuration files are created

2. **Re-initialization without --force**:
   - Run `loom init` on already initialized knowledge base
   - Verify error message is displayed
   - Verify exit code is 7

3. **Re-initialization with --force**:
   - Run `loom init --force` on already initialized knowledge base
   - Verify re-initialization succeeds
   - Verify model is re-downloaded if corrupted

4. **Network failure**:
   - Simulate network failure during download
   - Verify error message is displayed
   - Verify exit code is 2

5. **Disk full**:
   - Simulate disk full during download
   - Verify error message is displayed
   - Verify exit code is 3

6. **Permission denied**:
   - Simulate permission denied during download
   - Verify error message is displayed
   - Verify exit code is 4

7. **Checksum mismatch**:
   - Simulate checksum mismatch during validation
   - Verify error message is displayed
   - Verify exit code is 5

8. **Timeout**:
   - Simulate timeout during download
   - Verify error message is displayed
   - Verify exit code is 6

9. **Download in progress**:
   - Start download in one terminal
   - Run `loom init` in another terminal
   - Verify error message is displayed
   - Verify exit code is 8

10. **Ctrl+C interruption**:
    - Start download and interrupt with Ctrl+C
    - Verify cleanup happens (<500ms)
    - Verify download state is preserved
    - Verify resume capability on next init

11. **HTTP Range request resume**:
    - Start download and interrupt after partial download
    - Run `loom init` again
    - Verify download resumes from last byte
    - Verify progress starts from correct percentage

12. **Proxy configuration**:
    - Set HTTP_PROXY environment variable
    - Run `loom init`
    - Verify proxy is used for download
    - Verify download succeeds through proxy

13. **Model version mismatch**:
    - Place model file with wrong version
    - Run `loom init`
    - Verify version mismatch is detected
    - Verify re-download prompt is displayed

10. **Progress display**:
    - Run `loom init` and verify progress updates
    - Verify progress updates at least once per second
    - Verify progress format is correct

## References

- [Feature specification](../spec.md)
- [Research document](../research.md)
- [Data model](../data-model.md)
- [Knowledge Loom constitution](../../.specify/memory/constitution.md)
