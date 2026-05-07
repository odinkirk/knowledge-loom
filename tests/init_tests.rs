use std::env;
use std::fs;
use tempfile::TempDir;

fn fake_binary(path: &std::path::Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, b"#!/bin/sh\necho loom").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).unwrap();
    }
}

#[test]
fn test_init_happy_path() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    let binary_src = dir.join("fake_loom");
    fake_binary(&binary_src);

    knowledge_loom::init::run_init_with_binary(dir, &binary_src).unwrap();

    // Binary copied
    assert!(dir.join(".loom/bin/loom").exists());

    // .mcp.json written with loom key
    let mcp: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(dir.join(".mcp.json")).unwrap()
    ).unwrap();
    assert!(mcp["mcpServers"]["loom"]["command"].is_string());
    let kb_root = mcp["mcpServers"]["loom"]["env"]["KB_ROOT"].as_str().unwrap();
    assert_eq!(kb_root, dir.to_str().unwrap());

    // .gitignore updated
    let gi = fs::read_to_string(dir.join(".gitignore")).unwrap();
    assert!(gi.contains(".loom/"));
    assert!(gi.contains(".loom-index/"));
}

#[test]
fn test_init_preserves_existing_mcp_servers() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    let binary_src = dir.join("fake_loom");
    fake_binary(&binary_src);

    // Pre-populate .mcp.json with an unrelated server
    let existing = serde_json::json!({
        "mcpServers": {
            "other-server": { "command": "other", "args": [] }
        }
    });
    fs::write(dir.join(".mcp.json"), serde_json::to_string_pretty(&existing).unwrap()).unwrap();

    knowledge_loom::init::run_init_with_binary(dir, &binary_src).unwrap();

    let mcp: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(dir.join(".mcp.json")).unwrap()
    ).unwrap();
    // Original server preserved
    assert!(mcp["mcpServers"]["other-server"]["command"].is_string());
    // Loom server added
    assert!(mcp["mcpServers"]["loom"]["command"].is_string());
}

#[test]
fn test_init_idempotent() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    let binary_src = dir.join("fake_loom");
    fake_binary(&binary_src);

    knowledge_loom::init::run_init_with_binary(dir, &binary_src).unwrap();
    knowledge_loom::init::run_init_with_binary(dir, &binary_src).unwrap();

    let mcp: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(dir.join(".mcp.json")).unwrap()
    ).unwrap();
    let servers = mcp["mcpServers"].as_object().unwrap();
    // Exactly one "loom" key, no duplicates
    assert_eq!(servers.keys().filter(|k| k.as_str() == "loom").count(), 1);

    let gi = fs::read_to_string(dir.join(".gitignore")).unwrap();
    assert_eq!(gi.matches(".loom/").count(), 1);
    assert_eq!(gi.matches(".loom-index/").count(), 1);
}

#[test]
fn test_run_init_with_no_args_uses_current_dir() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    let binary_src = dir.join("fake_loom");
    fake_binary(&binary_src);

    // Change to temp directory
    let _ = env::set_current_dir(&dir);
    
    // Call run_init with just "init" arg (simulating command line)
    knowledge_loom::init::run_init(std::iter::once("init".to_string())).unwrap();

    // Binary should be copied to temp directory
    assert!(dir.join(".loom/bin/loom").exists());
}

#[test]
fn test_run_init_with_explicit_dir() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    let target_dir = dir.join("target");
    // Create the target directory
    fs::create_dir_all(&target_dir).unwrap();
    let binary_src = dir.join("fake_loom");
    fake_binary(&binary_src);

    // Call run_init with "init" and target directory
    knowledge_loom::init::run_init(
        std::iter::once("init".to_string())
            .chain(std::iter::once(target_dir.to_string_lossy().to_string()))
    ).unwrap();

    // Binary should be copied to target directory
    assert!(target_dir.join(".loom/bin/loom").exists());
}

#[test]
fn test_run_init_handles_invalid_directory() {
    let result = knowledge_loom::init::run_init(
        std::iter::once("init".to_string())
            .chain(std::iter::once("/nonexistent/path".to_string()))
    );
    
    // Should return an error
    assert!(result.is_err());
}
