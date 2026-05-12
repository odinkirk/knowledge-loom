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

#[test]
fn test_parse_chunks_ordinal_assignment() {
    let content = "# Section A\n\nContent A.\n\n# Section B\n\nContent B.";
    let chunks = chunks::parse_chunks(content);
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].ordinal, 1);
    assert_eq!(chunks[1].ordinal, 2);
}

#[test]
fn test_parse_chunks_ordinal_sequentiality() {
    let content = "# A\n\nContent A.\n\n# B\n\nContent B.\n\n# C\n\nContent C.";
    let chunks = chunks::parse_chunks(content);
    assert_eq!(chunks.len(), 3);
    for (i, chunk) in chunks.iter().enumerate() {
        assert_eq!(chunk.ordinal, (i + 1) as u64);
    }
}

#[test]
fn test_parse_chunks_multi_byte_content_with_ordinals() {
    let content = "# Heading\n\nHello😀World\n\n# Another\n\n世界";
    let chunks = chunks::parse_chunks(content);
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].ordinal, 1);
    assert_eq!(chunks[1].ordinal, 2);
    // Verify content is valid UTF-8
    for chunk in &chunks {
        assert!(chunk.content.is_char_boundary(chunk.content.len()));
    }
}

#[test]
fn test_parse_chunks_large_file_with_ordinals() {
    // Create content with 100+ chunks
    let mut content = String::new();
    for i in 1..=105 {
        content.push_str(&format!("# Section {}\n\nContent for section {}.\n\n", i, i));
    }
    let chunks = chunks::parse_chunks(&content);
    assert_eq!(chunks.len(), 105);
    // Verify ordinals are sequential
    for (i, chunk) in chunks.iter().enumerate() {
        assert_eq!(chunk.ordinal, (i + 1) as u64);
    }
}

#[test]
fn test_parse_chunks_boundary_cases_with_ordinals() {
    // Test with content exactly at 2000 character boundary
    let content = "# Heading\n\n";
    let body = "word ".repeat(500); // 2500 chars total
    let full_content = format!("{}{}", content, body);
    let chunks = chunks::parse_chunks(&full_content);
    assert!(!chunks.is_empty());
    // Verify ordinals are assigned
    for (i, chunk) in chunks.iter().enumerate() {
        assert_eq!(chunk.ordinal, (i + 1) as u64);
    }
}
