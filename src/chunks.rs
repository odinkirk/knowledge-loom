pub const MAX_CHUNK_CHARS: usize = 2000;

/// Represents a chunk of markdown content with metadata.
///
/// A chunk contains the actual content, heading context (as a breadcrumb path),
/// ordinal position within the file, and line number information.
#[derive(Debug, Clone)]
pub struct Chunk {
    /// Sequential ordinal number (1-based) of this chunk within the file
    #[allow(dead_code)]
    pub ordinal: u64,
    /// Heading breadcrumb path (e.g., "Main > Sub") or None if no heading
    pub heading: Option<String>,
    /// The actual chunk content (truncated at MAX_CHUNK_CHARS)
    pub content: String,
    /// Starting line number (1-based) of this chunk in the source file
    pub line_start: usize,
    /// Ending line number (1-based) of this chunk in the source file
    pub line_end: usize,
}

/// Truncates content at whitespace while ensuring character boundary safety.
///
/// This function ensures that truncation never splits multi-byte UTF-8 characters
/// (like emojis, CJK characters, or combining diacritics) by using character
/// boundary detection before slicing.
///
/// # Arguments
///
/// * `content` - The content to truncate
/// * `max` - Maximum number of characters to keep
///
/// # Returns
///
/// A string slice that is safely truncated at a character boundary
///
/// # Examples
///
/// ```
/// use knowledge_loom::chunks::truncate_at_whitespace;
/// let content = "Hello—World"; // em dash is 3 bytes
/// let result = truncate_at_whitespace(content, 7);
/// assert!(result.is_char_boundary(result.len()));
/// ```
#[allow(dead_code)]
pub fn truncate_at_whitespace(content: &str, max: usize) -> &str {
    if content.len() <= max {
        return content;
    }

    // Find safe character boundary by iterating through character indices
    // This ensures we never split a multi-byte UTF-8 character (like emojis or CJK)
    let safe_max = content
        .char_indices()
        .map(|(i, _)| i)
        .take_while(|&i| i <= max)
        .last()
        .unwrap_or(content.len());

    let slice = &content[..safe_max];
    // Try to truncate at whitespace for better readability
    match slice.rfind(|c: char| c.is_whitespace()) {
        Some(pos) if pos > 0 => content[..pos].trim_end(),
        _ => slice,
    }
}

/// Splits section content into multiple chunks of at most `MAX_CHUNK_CHARS` each.
///
/// Each chunk is split at a whitespace boundary for readability. All chunks
/// share the same heading context and are assigned sequential ordinals.
fn push_section_chunks(
    chunks: &mut Vec<Chunk>,
    heading: Option<String>,
    content: &str,
    line_start: usize,
    line_end: usize,
) {
    let mut remaining = content;
    while !remaining.is_empty() {
        if remaining.len() <= MAX_CHUNK_CHARS {
            let trimmed = remaining.trim_end();
            if !trimmed.is_empty() {
                chunks.push(Chunk {
                    ordinal: (chunks.len() + 1) as u64,
                    heading: heading.clone(),
                    content: trimmed.to_string(),
                    line_start,
                    line_end,
                });
            }
            break;
        }

        let boundary = truncate_at_whitespace_boundary(remaining, MAX_CHUNK_CHARS);
        let piece = remaining[..boundary].trim_end();
        chunks.push(Chunk {
            ordinal: (chunks.len() + 1) as u64,
            heading: heading.clone(),
            content: piece.to_string(),
            line_start,
            line_end,
        });

        // Advance past the piece plus any trailing whitespace
        remaining = remaining[boundary..].trim_start();
    }
}

/// Finds a safe character-boundary index within `content` at or near `max`,
/// preferring a whitespace boundary for readability.
fn truncate_at_whitespace_boundary(content: &str, max: usize) -> usize {
    let safe_max = content
        .char_indices()
        .map(|(i, _)| i)
        .take_while(|&i| i <= max)
        .last()
        .unwrap_or(content.len());

    let slice = &content[..safe_max];
    match slice.rfind(|c: char| c.is_whitespace()) {
        Some(pos) if pos > 0 => pos + 1, // include the whitespace in this chunk
        _ => safe_max,
    }
}

/// Parses markdown content into chunks with heading context and ordinal assignment.
///
/// This function splits markdown content into chunks of maximum size (2000 characters)
/// while preserving heading context (as breadcrumb paths) and assigning sequential ordinal numbers to each chunk.
///
/// # Arguments
///
/// * `content` - The markdown content to parse
///
/// # Returns
///
/// A vector of chunks with ordinal numbers, heading context (breadcrumb paths), and line numbers
///
/// # Examples
///
/// ```
/// use knowledge_loom::chunks::parse_chunks;
/// let content = "# Heading\n\nContent";
/// let chunks = parse_chunks(content);
/// assert_eq!(chunks[0].ordinal, 1);
/// ```
pub fn parse_chunks(content: &str) -> Vec<Chunk> {
    let lines: Vec<&str> = content.lines().collect();
    let mut chunks = Vec::new();
    // heading_stack: (level, text)
    let mut heading_stack: Vec<(usize, String)> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim_start();
        let level = trimmed.chars().take_while(|&c| c == '#').count();

        if level > 0 && level <= 6 && trimmed.len() >= level {
            let after = &trimmed[level..];
            if after.starts_with(' ') || after.starts_with('\t') || after.is_empty() {
                let heading_text = after.trim().to_string();
                // Pop same-or-deeper headings
                while heading_stack.last().is_some_and(|(l, _)| *l >= level) {
                    heading_stack.pop();
                }
                heading_stack.push((level, heading_text.clone()));

                let breadcrumb = heading_stack
                    .iter()
                    .map(|(_, t)| t.as_str())
                    .collect::<Vec<_>>()
                    .join(" > ");

                let section_start = i + 1; // 1-indexed heading line

                // Collect content until next heading
                let mut j = i + 1;
                while j < lines.len() {
                    let next = lines[j].trim_start();
                    let next_level = next.chars().take_while(|&c| c == '#').count();
                    if next_level > 0 && next_level <= 6 && next.len() > next_level {
                        let next_after = &next[next_level..];
                        if next_after.starts_with(' ') || next_after.starts_with('\t') {
                            break;
                        }
                    }
                    j += 1;
                }

                let section_content = lines[i + 1..j].join("\n");
                let section_content_trimmed = section_content.trim();
                let section_end = if j > i + 1 { j } else { i + 1 };

                if !section_content_trimmed.is_empty() {
                    push_section_chunks(
                        &mut chunks,
                        Some(breadcrumb),
                        section_content_trimmed,
                        section_start,
                        section_end,
                    );
                }

                i = j;
                continue;
            }
        }
        i += 1;
    }

    // Headingless fallback: treat entire content as chunks split at MAX_CHUNK_CHARS
    if chunks.is_empty() {
        push_section_chunks(&mut chunks, None, content.trim(), 1, lines.len());
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_at_whitespace_short_content_unchanged() {
        let content = "Short content";
        let result = truncate_at_whitespace(content, 100);
        assert_eq!(result, "Short content");
    }

    #[test]
    fn test_truncate_at_whitespace_cuts_at_space() {
        let content = "This is a long string that needs to be truncated";
        let result = truncate_at_whitespace(content, 20);
        assert_eq!(result, "This is a long");
    }

    #[test]
    fn test_truncate_at_whitespace_hard_cuts_when_no_space() {
        let content = "VeryLongStringWithoutSpaces";
        let result = truncate_at_whitespace(content, 10);
        assert!(result.len() <= 10);
        assert!(result.is_char_boundary(result.len()));
    }

    #[test]
    fn test_truncate_at_whitespace_multi_byte() {
        let content = "Hello—World"; // em dash is 3 bytes
        let result = truncate_at_whitespace(content, 7);
        assert!(result.is_char_boundary(result.len()));
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_parse_chunks_headingless_fallback() {
        let content = "Just some content\nwithout any headings";
        let chunks = parse_chunks(content);
        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].heading, None);
    }

    #[test]
    fn test_parse_chunks_two_sections() {
        let content = "# First\n\nContent 1\n\n# Second\n\nContent 2";
        let chunks = parse_chunks(content);
        assert!(chunks.len() >= 2);
        assert_eq!(chunks[0].heading, Some("First".to_string()));
        assert_eq!(chunks[1].heading, Some("Second".to_string()));
    }

    #[test]
    fn test_parse_chunks_breadcrumb() {
        let content = "# Main\n## Sub\n\nContent";
        let chunks = parse_chunks(content);
        assert!(!chunks.is_empty());
        // Breadcrumb path includes both headings
        assert_eq!(chunks[0].heading, Some("Main > Sub".to_string()));
    }

    #[test]
    fn test_parse_chunks_empty_section_skipped() {
        let content = "# Heading\n\n\n\nContent";
        let chunks = parse_chunks(content);
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_parse_chunks_splits_large_section() {
        let content = "# Heading\n\n".to_string() + &"A".repeat(3000);
        let chunks = parse_chunks(&content);
        assert!(
            chunks.len() > 1,
            "3000 chars should produce multiple chunks"
        );
        for chunk in &chunks {
            assert!(chunk.content.len() <= MAX_CHUNK_CHARS);
        }
    }

    #[test]
    fn test_extract_title() {
        let content = "# Main Heading\n\nSome content";
        let chunks = parse_chunks(content);
        assert_eq!(chunks[0].heading, Some("Main Heading".to_string()));
    }

    #[test]
    fn test_extract_title_no_heading() {
        let content = "Just content without heading";
        let chunks = parse_chunks(content);
        assert_eq!(chunks[0].heading, None);
    }

    #[test]
    fn test_extract_title_empty_heading() {
        let content = "#\n\nContent";
        let chunks = parse_chunks(content);
        assert_eq!(chunks[0].heading, Some("".to_string()));
    }
}
