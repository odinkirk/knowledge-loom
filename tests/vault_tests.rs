#[cfg(test)]
mod tests {

    use std::fs;
    use tempfile::TempDir;

    use knowledge_loom::vault::VaultState;

    #[tokio::test]
    async fn test_vault_scan_files() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create test files
        fs::create_dir_all(kb_root.join("subdir")).unwrap();
        fs::write(kb_root.join("test1.md"), "# Test 1\nContent 1").unwrap();
        fs::write(kb_root.join("test2.md"), "# Test 2\nContent 2").unwrap();
        fs::write(kb_root.join("subdir/test3.md"), "# Test 3\nContent 3").unwrap();

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        let files = vault.scan_files().await;

        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|f| f.ends_with("test1.md")));
        assert!(files.iter().any(|f| f.ends_with("test2.md")));
        assert!(files.iter().any(|f| f.ends_with("subdir/test3.md")));
    }

    #[tokio::test]
    async fn test_vault_read_file() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        let test_content = "# Test Note\n\nThis is test content.";
        fs::write(kb_root.join("test.md"), test_content).unwrap();

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        let content = vault.read_file(&kb_root.join("test.md")).await;

        assert_eq!(content, Some(test_content.to_string()));
    }

    #[tokio::test]
    async fn test_vault_loomignore() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create .knowledge-loom-ignore
        fs::write(kb_root.join(".knowledge-loom-ignore"), "*.tmp\nignored/").unwrap();

        // Create files
        fs::create_dir_all(kb_root.join("ignored")).unwrap();
        fs::write(kb_root.join("test.md"), "# Test").unwrap();
        fs::write(kb_root.join("file.tmp"), "# Temp").unwrap();
        fs::write(kb_root.join("ignored/test.md"), "# Ignored").unwrap();

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        let files = vault.scan_files().await;

        // Should only find test.md
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("test.md"));
    }

    #[tokio::test]
    async fn test_vault_get_file_mod_time() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        let test_path = kb_root.join("test.md");
        fs::write(&test_path, "# Test").unwrap();

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        let mod_time = vault.get_file_mod_time(&test_path).await;

        assert!(mod_time.is_some());
    }

    #[tokio::test]
    async fn test_vault_uses_chunks_module() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        // Create a test file with multiple sections
        let test_content = "# Section A\n\nContent A.\n\n# Section B\n\nContent B.";
        fs::write(kb_root.join("test.md"), test_content).unwrap();

        // Verify that chunks module is used by parsing the file
        let chunks = knowledge_loom::chunks::parse_chunks(test_content);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].heading, Some("Section A".to_string()));
        assert_eq!(chunks[1].heading, Some("Section B".to_string()));

        // Verify ordinals are assigned
        assert_eq!(chunks[0].ordinal, 1);
        assert_eq!(chunks[1].ordinal, 2);
    }

    #[tokio::test]
    async fn test_glob_ignore_subdir_matching() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        fs::write(kb_root.join(".knowledge-loom-ignore"), ".claude/**").unwrap();

        fs::create_dir_all(kb_root.join(".claude/worktrees/foo")).unwrap();
        fs::write(kb_root.join(".claude/worktrees/foo/file.md"), "# Duplicate").unwrap();
        fs::write(kb_root.join("world.md"), "# World").unwrap();

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        let files = vault.scan_files().await;

        assert_eq!(
            files.len(),
            1,
            ".claude/ subtree should be ignored via glob"
        );
        assert!(files[0].ends_with("world.md"));
    }

    #[tokio::test]
    async fn test_glob_ignore_wildcard_extension() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        fs::write(kb_root.join(".knowledge-loom-ignore"), "*.log").unwrap();

        fs::write(kb_root.join("build.log"), "# Build log").unwrap();
        fs::write(kb_root.join("catalogue.md"), "# Catalogue").unwrap();

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        assert!(vault.should_ignore(&kb_root.join("build.log")));
        assert!(vault.should_ignore(&kb_root.join("sub/build.log")));
        assert!(vault.should_ignore(&kb_root.join("errors.log")));
        assert!(!vault.should_ignore(&kb_root.join("catalogue.md")));
    }

    #[tokio::test]
    async fn test_glob_ignore_no_false_positives() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        fs::write(
            kb_root.join(".knowledge-loom-ignore"),
            "sub/ignored.txt\n.claude/**",
        )
        .unwrap();

        fs::write(kb_root.join("readme.md"), "# Readme").unwrap();
        fs::write(kb_root.join("claude.md"), "# Claude").unwrap();
        fs::create_dir_all(kb_root.join("sub")).unwrap();
        fs::write(kb_root.join("sub/notes.md"), "# Notes").unwrap();

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        let files = vault.scan_files().await;

        assert_eq!(
            files.len(),
            3,
            "all three md files should be included (none match glob patterns)"
        );
        let names: Vec<&str> = files
            .iter()
            .map(|f| f.file_name().unwrap().to_str().unwrap())
            .collect();
        assert!(names.contains(&"readme.md"));
        assert!(names.contains(&"claude.md"));
        assert!(names.contains(&"notes.md"));
    }

    #[tokio::test]
    async fn test_glob_ignore_blanks_and_comments() {
        let temp_dir = TempDir::new().unwrap();
        let kb_root = temp_dir.path();

        fs::write(
            kb_root.join(".knowledge-loom-ignore"),
            "\n# comment\n*.log\n\n",
        )
        .unwrap();

        let vault = VaultState::new(kb_root.to_str().unwrap()).await;
        assert!(vault.should_ignore(&kb_root.join("build.log")));
        assert!(!vault.should_ignore(&kb_root.join("readme.md")));
        assert!(!vault.should_ignore(&kb_root.join("# comment")));
    }
}
