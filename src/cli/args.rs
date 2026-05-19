// CLI argument parsing utilities for Knowledge Loom
// Provides robust argument parsing with clear error messages

use std::env::args;

/// Parse a boolean flag from command line arguments
///
/// Supports both long (--flag) and short (-f) forms
///
/// # Arguments
///
/// * `long_flag` - Long form flag name (e.g., "force")
/// * `short_flag` - Short form flag name (e.g., "f"), or None
///
/// # Returns
///
/// true if flag is present, false otherwise
pub fn parse_flag(long_flag: &str, short_flag: Option<&str>) -> bool {
    let args: Vec<String> = args().collect();

    // Check for long form (--flag)
    if args.iter().any(|arg| arg == &format!("--{}", long_flag)) {
        return true;
    }

    // Check for short form (-f)
    if let Some(short) = short_flag {
        if args.iter().any(|arg| arg == &format!("-{}", short)) {
            return true;
        }
    }

    false
}

/// Parse a string value from command line arguments
///
/// Supports both --key value and --key=value forms
///
/// # Arguments
///
/// * `long_key` - Long form key name (e.g., "platform")
/// * `short_key` - Short form key name (e.g., "p"), or None
///
/// # Returns
///
/// * `Ok(Some(String))` - If key with value is found
/// * `Ok(None)` - If key is not present
/// * `Err(String)` - If key is present but value is missing or invalid
pub fn parse_string_value(
    long_key: &str,
    short_key: Option<&str>,
) -> Result<Option<String>, String> {
    let args: Vec<String> = args().collect();

    // Check for --key=value form
    let prefix = format!("--{}=", long_key);
    for arg in &args {
        if arg.starts_with(&prefix) {
            let value = arg
                .strip_prefix(&prefix)
                .ok_or_else(|| format!("Invalid {} format", long_key))?
                .to_string();

            if value.is_empty() {
                return Err(format!("--{} requires a value", long_key));
            }

            return Ok(Some(value));
        }
    }

    // Check for --key value form
    for (i, arg) in args.iter().enumerate() {
        if arg == &format!("--{}", long_key) {
            if let Some(value) = args.get(i + 1) {
                if value.starts_with('-') {
                    return Err(format!("--{} requires a value", long_key));
                }
                return Ok(Some(value.clone()));
            } else {
                return Err(format!("--{} requires a value", long_key));
            }
        }

        // Check short form -p value
        if let Some(short) = short_key {
            if arg == &format!("-{}", short) {
                if let Some(value) = args.get(i + 1) {
                    if value.starts_with('-') {
                        return Err(format!("-{} requires a value", short));
                    }
                    return Ok(Some(value.clone()));
                } else {
                    return Err(format!("-{} requires a value", short));
                }
            }
        }
    }

    Ok(None)
}

/// Parse unknown flags and provide helpful error messages
///
/// # Arguments
///
/// * `valid_flags` - List of valid flag names
/// * `valid_short_flags` - List of valid short flag characters (e.g., &['f', 'h'])
///
/// # Returns
///
/// * `Ok(())` - If no unknown flags found
/// * `Err(String)` - If unknown flag found with suggestions
pub fn validate_flags(valid_flags: &[&str], valid_short_flags: &[char]) -> Result<(), String> {
    validate_flags_from(&args().collect::<Vec<_>>(), valid_flags, valid_short_flags)
}

/// Internal: validate flags from a provided args slice (testable)
fn validate_flags_from(
    args: &[String],
    valid_flags: &[&str],
    valid_short_flags: &[char],
) -> Result<(), String> {
    for arg in args {
        if arg.starts_with("--") {
            let flag = arg.trim_start_matches("--").split('=').next().unwrap();

            // Skip if it's a known flag
            if valid_flags.contains(&flag) {
                continue;
            }

            // Provide suggestions
            let suggestions = find_similar_flags(flag, valid_flags);
            let suggestion_msg = if suggestions.is_empty() {
                String::new()
            } else {
                format!(" Did you mean {}?", suggestions.join(", "))
            };

            return Err(format!("Unknown flag: --{}{}", flag, suggestion_msg));
        }

        // Check short flags (single dash, single character)
        if arg.starts_with('-') && arg.len() == 2 && !arg.starts_with("--") {
            let flag_char = arg.chars().nth(1).unwrap();

            // Skip numeric flags (likely negative numbers, not flags)
            if flag_char.is_numeric() {
                continue;
            }

            // Validate against known short flags
            if !valid_short_flags.contains(&flag_char) {
                let suggestions = valid_short_flags
                    .iter()
                    .map(|c| format!("-{}", c))
                    .collect::<Vec<_>>();
                let suggestion_msg = if suggestions.is_empty() {
                    String::new()
                } else {
                    format!(" Valid short flags: {}", suggestions.join(", "))
                };
                return Err(format!("Unknown flag: -{}{}", flag_char, suggestion_msg));
            }
        }
    }

    Ok(())
}

/// Find similar flags for suggestions using simple string matching
fn find_similar_flags(input: &str, valid_flags: &[&str]) -> Vec<String> {
    let mut suggestions = Vec::new();

    for &flag in valid_flags {
        // Simple prefix match
        if flag.starts_with(input) && flag != input {
            suggestions.push(flag.to_string());
        }
    }

    suggestions.truncate(3); // Limit to 3 suggestions
    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_flags_with_valid_long_flags() {
        let result = validate_flags_from(
            &["--force".to_string(), "--help".to_string()],
            &["force", "help", "version"],
            &['f', 'h', 'v'],
        );
        assert!(result.is_ok(), "Valid flags should pass: {:?}", result);
    }

    #[test]
    fn test_validate_flags_with_valid_short_flags() {
        let result = validate_flags_from(
            &["-f".to_string(), "-h".to_string()],
            &["force", "help", "version"],
            &['f', 'h', 'v'],
        );
        assert!(
            result.is_ok(),
            "Valid short flags should pass: {:?}",
            result
        );
    }

    #[test]
    fn test_validate_flags_with_unknown_long_flag() {
        let result = validate_flags_from(
            &["--unknown".to_string()],
            &["force", "help", "version"],
            &['f', 'h', 'v'],
        );
        assert!(result.is_err(), "Unknown flag should error");
        let err = result.unwrap_err();
        assert!(
            err.contains("Unknown flag"),
            "Error should mention unknown: {err}"
        );
    }

    #[test]
    fn test_validate_flags_with_unknown_short_flag() {
        let result = validate_flags_from(
            &["-x".to_string()],
            &["force", "help", "version"],
            &['f', 'h', 'v'],
        );
        assert!(
            result.is_err(),
            "Unknown short flag should error: {:?}",
            result
        );
        let err = result.unwrap_err();
        assert!(
            err.contains("Unknown flag"),
            "Error should mention unknown: {err}"
        );
    }

    #[test]
    fn test_validate_flags_skips_numeric_short_args() {
        let result = validate_flags_from(
            &["-1".to_string(), "--force".to_string()],
            &["force", "help", "version"],
            &['f', 'h', 'v'],
        );
        assert!(
            result.is_ok(),
            "Numeric args should be skipped: {:?}",
            result
        );
    }
}
