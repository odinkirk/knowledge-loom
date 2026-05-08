#[cfg(test)]
mod tests {
    
    use tempfile::TempDir;
    use std::fs;
    
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
}