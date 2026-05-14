# Display Contract: Progress Indicators

**Feature**: Init-Time Model Download
**Date**: 2026-05-12
**Status**: Complete

## Overview

This document defines the display contract for progress indicators, including format specifications, update frequency, and error message formatting.

## Progress Display Format

### Download Progress

**Format**:

```
Downloading model: {percentage}% ({downloaded_mb}MB/{total_mb}MB) - {speed_mb}MB/s - {remaining}s remaining
```

**Fields**:

| Field | Type | Description | Format |
|-------|------|-------------|--------|
| `percentage` | Number | Download progress (0-100) | Integer (no decimal places) |
| `downloaded_mb` | Number | Bytes downloaded in MB | Float (1 decimal place) |
| `total_mb` | Number | Total bytes in MB | Float (1 decimal place) |
| `speed_mb` | Number | Download speed in MB/s | Float (1 decimal place) |
| `remaining` | Number | Estimated time remaining in seconds | Integer (no decimal places) |

**Examples**:

```
Downloading model: 0% (0.0MB/120.0MB) - 0.0MB/s - 0s remaining
Downloading model: 10% (12.0MB/120.0MB) - 2.5MB/s - 43s remaining
Downloading model: 20% (24.0MB/120.0MB) - 2.5MB/s - 38s remaining
Downloading model: 30% (36.0MB/120.0MB) - 2.5MB/s - 34s remaining
Downloading model: 40% (48.0MB/120.0MB) - 2.5MB/s - 29s remaining
Downloading model: 50% (60.0MB/120.0MB) - 2.5MB/s - 24s remaining
Downloading model: 60% (72.0MB/120.0MB) - 2.5MB/s - 19s remaining
Downloading model: 70% (84.0MB/120.0MB) - 2.5MB/s - 14s remaining
Downloading model: 80% (96.0MB/120.0MB) - 2.5MB/s - 10s remaining
Downloading model: 90% (108.0MB/120.0MB) - 2.5MB/s - 5s remaining
Downloading model: 100% (120.0MB/120.0MB) - 2.5MB/s - 0s remaining
```

**Update Frequency**: At least once per second

**Display Rules**:
- Always display percentage as integer (no decimal places)
- Always display MB values with 1 decimal place
- Always display speed with 1 decimal place
- Always display remaining time as integer (no decimal places)
- Always display "remaining" at the end
- Always display "Downloading model:" prefix

### Download Completion

**Format**:

```
Model download complete: 100% ({total_mb}MB/{total_mb}MB) - {speed_mb}MB/s - {elapsed}s total
```

**Fields**:

| Field | Type | Description | Format |
|-------|------|-------------|--------|
| `total_mb` | Number | Total bytes in MB | Float (1 decimal place) |
| `speed_mb` | Number | Average download speed in MB/s | Float (1 decimal place) |
| `elapsed` | Number | Elapsed time in seconds | Integer (no decimal places) |

**Examples**:

```
Model download complete: 100% (120.0MB/120.0MB) - 2.5MB/s - 48s total
```

**Display Rules**:
- Always display "Model download complete:" prefix
- Always display 100% (no variation)
- Always display MB values with 1 decimal place
- Always display speed with 1 decimal place
- Always display elapsed time as integer (no decimal places)
- Always display "total" at the end

### Download Retry

**Format**:

```
Retrying model download (attempt {attempt} of {max_retries})...
```

**Fields**:

| Field | Type | Description | Format |
|-------|------|-------------|--------|
| `attempt` | Number | Current attempt number | Integer (no decimal places) |
| `max_retries` | Number | Maximum retry attempts | Integer (no decimal places) |

**Examples**:

```
Retrying model download (attempt 1 of 3)...
Retrying model download (attempt 2 of 3)...
Retrying model download (attempt 3 of 3)...
```

**Display Rules**:
- Always display "Retrying model download (" prefix
- Always display attempt number and max retries
- Always display ")..." suffix

### Download in Progress

**Format**:

```
Model download is already in progress. Please wait for the current download to complete.
```

**Display Rules**:
- Always display exact message (no variation)
- No fields to format

## Error Message Format

### Network Error

**Format**:

```
Model download failed: Network error: {details}
Please check your internet connection and try again.
```

**Fields**:

| Field | Type | Description | Format |
|-------|------|-------------|--------|
| `details` | String | Network error details | As-is (no formatting) |

**Examples**:

```
Model download failed: Network error: Connection timeout
Please check your internet connection and try again.

Model download failed: Network error: DNS resolution failed
Please check your internet connection and try again.

Model download failed: Network error: Connection refused
Please check your internet connection and try again.
```

**Display Rules**:
- Always display "Model download failed: Network error:" prefix
- Always display error details
- Always display "Please check your internet connection and try again." on next line

### Disk Full

**Format**:

```
Model download failed: Insufficient disk space: {required_mb}MB required, {available_mb}MB available
Please free up space and try again.
```

**Fields**:

| Field | Type | Description | Format |
|-------|------|-------------|--------|
| `required_mb` | Number | Required disk space in MB | Integer (no decimal places) |
| `available_mb` | Number | Available disk space in MB | Integer (no decimal places) |

**Examples**:

```
Model download failed: Insufficient disk space: 150MB required, 50MB available
Please free up space and try again.
```

**Display Rules**:
- Always display "Model download failed: Insufficient disk space:" prefix
- Always display required and available MB values as integers
- Always display "Please free up space and try again." on next line

### Permission Denied

**Format**:

```
Model download failed: Permission denied: Cannot write to {path}
Please check file permissions and try again.
```

**Fields**:

| Field | Type | Description | Format |
|-------|------|-------------|--------|
| `path` | String | File path | As-is (no formatting) |

**Examples**:

```
Model download failed: Permission denied: Cannot write to /path/to/.knowledge-loom-index/models/
Please check file permissions and try again.
```

**Display Rules**:
- Always display "Model download failed: Permission denied: Cannot write to" prefix
- Always display file path
- Always display "Please check file permissions and try again." on next line

### Checksum Mismatch

**Format**:

```
Model download failed: Checksum mismatch
The downloaded file may be corrupted. Please try downloading again.
```

**Display Rules**:
- Always display exact message (no variation)
- No fields to format

### Timeout

**Format**:

```
Model download failed: Download timeout: The download took too long
Please check your internet connection and try again.
```

**Display Rules**:
- Always display exact message (no variation)
- No fields to format

### Generic Error

**Format**:

```
Model download failed: {error_message}
Please try again or contact support if the issue persists.
```

**Fields**:

| Field | Type | Description | Format |
|-------|------|-------------|--------|
| `error_message` | String | Error message | As-is (no formatting) |

**Examples**:

```
Model download failed: Unknown error
Please try again or contact support if the issue persists.
```

**Display Rules**:
- Always display "Model download failed:" prefix
- Always display error message
- Always display "Please try again or contact support if the issue persists." on next line

## Validation Display Format

### Validating Model

**Format**:

```
Validating model... OK
```

**Display Rules**:
- Always display exact message (no variation)
- No fields to format

### Validation Failed

**Format**:

```
Validating model... FAILED
{error_message}
```

**Fields**:

| Field | Type | Description | Format |
|-------|------|-------------|--------|
| `error_message` | String | Error message | As-is (no formatting) |

**Examples**:

```
Validating model... FAILED
Checksum mismatch: expected a1b2c3..., got d4e5f6...
```

**Display Rules**:
- Always display "Validating model... FAILED" on first line
- Always display error message on next line

## Initialization Display Format

### Creating Directories

**Format**:

```
Creating directory structure...
```

**Display Rules**:
- Always display exact message (no variation)
- No fields to format

### Initializing Indexes

**Format**:

```
Initializing indexes...
```

**Display Rules**:
- Always display exact message (no variation)
- No fields to format

### Creating Config Files

**Format**:

```
Creating configuration files...
```

**Display Rules**:
- Always display exact message (no variation)
- No fields to format

### Initialization Complete

**Format**:

```
knowledge-loom init complete.
  binary:  {binary_path}
  KB_ROOT: {kb_root}
  .mcp.json updated
  opencode.json updated
  loom-shell.sh created

Next: restart Claude Code and run /mcp to connect.
```

**Fields**:

| Field | Type | Description | Format |
|-------|------|-------------|--------|
| `binary_path` | String | Path to loom binary | As-is (no formatting) |
| `kb_root` | String | Knowledge base root directory | As-is (no formatting) |

**Examples**:

```
knowledge-loom init complete.
  binary:  /path/to/loom
  KB_ROOT: /path/to/knowledge-base
  .mcp.json updated
  opencode.json updated
  loom-shell.sh created

Next: restart Claude Code and run /mcp to connect.
```

**Display Rules**:
- Always display "knowledge-loom init complete." on first line
- Always display binary path with "  binary: " prefix
- Always display KB_ROOT with "  KB_ROOT: " prefix
- Always display ".mcp.json updated"
- Always display "opencode.json updated"
- Always display "loom-shell.sh created"
- Always display blank line
- Always display "Next: restart Claude Code and run /mcp to connect."

## Parsing Rules

### AI Assistant Parsing

AI assistants can parse progress updates using the following regex patterns:

**Download Progress**:

```regex
^Downloading model: (\d+)%\s+\((\d+\.?\d*)MB/(\d+\.?\d*)MB\)\s+-\s+(\d+\.?\d*)MB/s\s+-\s+(\d+)s remaining$
```

**Groups**:
1. Percentage (integer)
2. Downloaded MB (float)
3. Total MB (float)
4. Speed MB/s (float)
5. Remaining seconds (integer)

**Download Completion**:

```regex
^Model download complete: 100%\s+\((\d+\.?\d*)MB/(\d+\.?\d*)MB\)\s+-\s+(\d+\.?\d*)MB/s\s+-\s+(\d+)s total$
```

**Groups**:
1. Total MB (float)
2. Total MB (float) - duplicate
3. Speed MB/s (float)
4. Elapsed seconds (integer)

**Download Retry**:

```regex
^Retrying model download \(attempt (\d+) of (\d+)\)\.\.\.$
```

**Groups**:
1. Attempt number (integer)
2. Max retries (integer)

**Error Messages**:

```regex
^Model download failed: (.+)$
```

**Groups**:
1. Error message (string)

## Testing

### Display Tests

1. **Progress formatting**:
   - Test all progress values (0%, 10%, 50%, 100%)
   - Verify format matches specification
   - Verify decimal places are correct

2. **Error message formatting**:
   - Test all error types
   - Verify format matches specification
   - Verify error details are included

3. **Completion formatting**:
   - Test completion message
   - Verify format matches specification
   - Verify all fields are included

4. **Parsing tests**:
   - Test AI assistant parsing with regex
   - Verify all groups are captured correctly
   - Verify edge cases are handled

### Integration Tests

1. **End-to-end display**:
   - Run full download and verify all messages
   - Verify message order is correct
   - Verify message timing is correct

2. **Error display**:
   - Simulate all error scenarios
   - Verify error messages are displayed correctly
   - Verify error messages are actionable

3. **Progress updates**:
   - Verify progress updates at least once per second
   - Verify progress values are accurate
   - Verify progress format is consistent

## References

- [Feature specification](../spec.md)
- [Research document](../research.md)
- [Data model](../data-model.md)
- [CLI contract](./cli-init.md)
- [API contract](./api-model-download.md)
- [Knowledge Loom constitution](../../.specify/memory/constitution.md)
