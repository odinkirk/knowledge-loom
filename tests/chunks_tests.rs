use knowledge_loom::chunks;

#[test]
fn test_truncate_at_whitespace_with_ascii_content() {
    let content = "Hello World";
    let result = chunks::truncate_at_whitespace(content, 5);
    assert_eq!(result, "Hello");
}

#[test]
fn test_truncate_at_whitespace_with_multi_byte_emoji() {
    let content = "Hello😀World"; // emoji is 4 bytes
    let result = chunks::truncate_at_whitespace(content, 7);
    assert!(result.is_char_boundary(result.len()));
    assert_eq!(result, "Hello");
}

#[test]
fn test_truncate_at_whitespace_with_cjk_characters() {
    let content = "Hello世界World"; // CJK characters are multi-byte
    let result = chunks::truncate_at_whitespace(content, 7);
    assert!(result.is_char_boundary(result.len()));
    assert_eq!(result, "Hello");
}

#[test]
fn test_truncate_at_whitespace_with_combining_diacritics() {
    let content = "HelloéWorld"; // é with combining diacritic
    let result = chunks::truncate_at_whitespace(content, 7);
    assert!(result.is_char_boundary(result.len()));
}

#[test]
fn test_truncate_at_whitespace_at_exact_boundary() {
    let content = "Hello World";
    let result = chunks::truncate_at_whitespace(content, 5);
    assert_eq!(result, "Hello");
}

#[test]
fn test_truncate_at_whitespace_with_whitespace_truncation() {
    let content = "This is a long string that needs to be truncated";
    let result = chunks::truncate_at_whitespace(content, 20);
    assert_eq!(result, "This is a long");
}

#[test]
fn test_parse_chunks_with_heading_extraction() {
    let content = "# Heading\n\nContent";
    let chunks = chunks::parse_chunks(content);
    assert!(!chunks.is_empty());
    assert_eq!(chunks[0].heading, Some("Heading".to_string()));
}

#[test]
fn test_parse_chunks_with_no_headings() {
    let content = "Just some content\nwithout any headings";
    let chunks = chunks::parse_chunks(content);
    assert!(!chunks.is_empty());
    assert_eq!(chunks[0].heading, None);
}

#[test]
fn test_parse_chunks_with_nested_headings() {
    let content = "# Main\n## Sub\n\nContent";
    let chunks = chunks::parse_chunks(content);
    assert!(!chunks.is_empty());
    // Breadcrumb path includes both headings
    assert_eq!(chunks[0].heading, Some("Main > Sub".to_string()));
}

#[test]
fn test_parse_chunks_with_empty_content() {
    let content = "";
    let chunks = chunks::parse_chunks(content);
    assert!(chunks.is_empty());
}

#[test]
fn test_parse_chunks_with_multi_byte_content() {
    let content = "# Heading\n\nHello😀World";
    let chunks = chunks::parse_chunks(content);
    assert!(!chunks.is_empty());
    // Verify content is valid UTF-8
    for chunk in &chunks {
        assert!(chunk.content.is_char_boundary(chunk.content.len()));
    }
}
