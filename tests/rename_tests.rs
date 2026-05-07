use knowledge_loom::{INDEX_DIR, BIN_DIR, IGNORE_FILE, MCP_CONFIG_KEY,
                     GITIGNORE_ENTRY_BIN, GITIGNORE_ENTRY_INDEX};

#[test]
fn test_constants_have_new_names() {
    assert!(INDEX_DIR.contains("knowledge-loom"), "INDEX_DIR: {INDEX_DIR}");
    assert!(BIN_DIR.contains("knowledge-loom"), "BIN_DIR: {BIN_DIR}");
    assert!(IGNORE_FILE.contains("knowledge-loom"), "IGNORE_FILE: {IGNORE_FILE}");
    assert_eq!(MCP_CONFIG_KEY, "knowledge-loom");
    assert!(GITIGNORE_ENTRY_BIN.contains("knowledge-loom"));
    assert!(GITIGNORE_ENTRY_INDEX.contains("knowledge-loom"));
}