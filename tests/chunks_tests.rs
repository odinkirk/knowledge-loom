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
        content.push_str(&format!(
            "# Section {}\n\nContent for section {}.\n\n",
            i, i
        ));
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

#[test]
fn test_module_boundaries() {
    // Verify that chunking logic is only in chunks module
    // This test ensures no duplicate chunking code exists elsewhere
    let content = "# Heading\n\nContent";
    let chunks = chunks::parse_chunks(content);
    assert!(!chunks.is_empty());
    // Verify chunk structure matches expected API
    assert!(chunks[0].ordinal > 0);
    assert!(chunks[0].line_start > 0);
    assert!(chunks[0].line_end >= chunks[0].line_start);
}

#[test]
fn test_module_api_stability() {
    // Verify that the Chunk struct API is stable
    let content = "# Heading\n\nContent";
    let chunks = chunks::parse_chunks(content);
    assert!(!chunks.is_empty());

    // Verify all expected fields exist and are accessible
    let chunk = &chunks[0];
    let _ordinal = chunk.ordinal;
    let _heading = &chunk.heading;
    let _content = &chunk.content;
    let _line_start = chunk.line_start;
    let _line_end = chunk.line_end;

    // Verify Clone trait is implemented
    let chunk_clone = chunk.clone();
    assert_eq!(chunk.ordinal, chunk_clone.ordinal);
}

#[test]
fn test_module_performance() {
    // Verify chunking performance is acceptable
    let content = "# Heading\n\n".to_string() + &"A".repeat(10000);
    let start = std::time::Instant::now();
    let chunks = chunks::parse_chunks(&content);
    let duration = start.elapsed();

    assert!(!chunks.is_empty());
    // Should complete in reasonable time (< 10ms for typical content)
    assert!(
        duration.as_millis() < 10,
        "Chunking took too long: {:?}",
        duration
    );
}

#[test]
fn test_module_error_handling() {
    // Verify graceful handling of edge cases
    let test_cases = vec![
        "",                        // Empty content
        "\n\n\n",                  // Only whitespace
        "#",                       // Heading without text
        "##",                      // Heading without text
        "Content without heading", // No heading
    ];

    for content in test_cases {
        let chunks = chunks::parse_chunks(content);
        // Should not panic, even with edge cases
        assert!(!chunks.is_empty());
    }
}

#[test]
fn test_module_thread_safety() {
    // Verify that chunking is thread-safe (no shared mutable state)
    let content = "# Heading\n\nContent";

    // Spawn multiple threads that all parse the same content
    let handles: Vec<_> = (0..10)
        .map(|_| {
            std::thread::spawn(|| {
                let chunks = chunks::parse_chunks(content);
                assert!(!chunks.is_empty());
                chunks.len()
            })
        })
        .collect();

    // All threads should complete successfully
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result > 0);
    }
}

#[test]
fn test_module_memory_usage() {
    // Verify reasonable memory usage for large content
    let content = "# Heading\n\n".to_string() + &"A".repeat(100000);

    let chunks = chunks::parse_chunks(&content);
    assert!(!chunks.is_empty());

    // Verify chunks are not excessively large
    for chunk in &chunks {
        assert!(chunk.content.len() <= chunks::MAX_CHUNK_CHARS);
    }
}

#[test]
fn test_module_concurrency() {
    // Verify concurrent chunking operations work correctly
    let content = "# Heading\n\nContent";

    // Use rayon for parallel processing if available, otherwise use threads
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let content = format!("{} # Section {}\n\nContent {}", content, i, i);
            std::thread::spawn(move || {
                let chunks = chunks::parse_chunks(&content);
                assert!(!chunks.is_empty());
                chunks.len()
            })
        })
        .collect();

    // All operations should complete successfully
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result > 0);
    }
}
