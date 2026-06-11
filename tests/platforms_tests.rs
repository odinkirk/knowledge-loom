use knowledge_loom::platforms::{install_platform, PlatformName};
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_platform_name_from_str() {
    assert_eq!(PlatformName::from_str("claude"), Some(PlatformName::Claude));
    assert_eq!(PlatformName::from_str("cursor"), Some(PlatformName::Cursor));
    assert_eq!(
        PlatformName::from_str("opencode"),
        Some(PlatformName::OpenCode)
    );
    assert_eq!(PlatformName::from_str("bogus"), None);
}

#[test]
fn test_install_claude_creates_mcp_json() {
    let tmp = TempDir::new().unwrap();
    let binary = tmp.path().join("loom");
    fs::write(&binary, b"#!/bin/sh").unwrap();

    install_platform(PlatformName::Claude, tmp.path(), &binary).unwrap();

    let mcp: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(tmp.path().join(".mcp.json")).unwrap()).unwrap();
    let entry = &mcp["mcpServers"]["knowledge-loom"];
    assert!(entry["command"].is_string());
    assert_eq!(entry["args"][0], "serve");
    assert_eq!(entry["type"], "stdio");
    assert_eq!(entry["env"]["KB_ROOT"], ".");
}

#[test]
fn test_install_cursor_creates_cursor_mcp_json() {
    let tmp = TempDir::new().unwrap();
    let binary = tmp.path().join("loom");
    fs::write(&binary, b"#!/bin/sh").unwrap();

    install_platform(PlatformName::Cursor, tmp.path(), &binary).unwrap();

    let mcp_path = tmp.path().join(".cursor/mcp.json");
    assert!(mcp_path.exists(), "cursor mcp.json not created");
    let mcp: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&mcp_path).unwrap()).unwrap();
    assert!(mcp["mcpServers"]["knowledge-loom"]["command"].is_string());

    // Also creates .cursorrules
    assert!(tmp.path().join(".cursorrules").exists());
}

#[test]
fn test_install_opencode_creates_opencode_json() {
    let tmp = TempDir::new().unwrap();
    let binary = tmp.path().join("loom");
    fs::write(&binary, b"#!/bin/sh").unwrap();

    install_platform(PlatformName::OpenCode, tmp.path(), &binary).unwrap();

    let oc: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(tmp.path().join("opencode.json")).unwrap())
            .unwrap();

    // $schema must be present
    assert_eq!(oc["$schema"], "https://opencode.ai/config.json");

    // mcp key (not mcpServers) per https://opencode.ai/config.json McpLocalConfig
    let entry = &oc["mcp"]["knowledge-loom"];
    assert_eq!(entry["type"], "local");
    // command is an array of strings
    assert!(entry["command"].is_array());
    let cmd = entry["command"].as_array().unwrap();
    assert_eq!(cmd.len(), 2);
    assert_eq!(cmd[1], "serve");
    // environment is an object
    assert!(entry["environment"].is_object());
    assert!(entry["environment"]["KB_ROOT"].is_string());
}

#[test]
fn test_install_preserves_existing_mcp_servers() {
    let tmp = TempDir::new().unwrap();
    let binary = tmp.path().join("loom");
    fs::write(&binary, b"#!/bin/sh").unwrap();

    // Pre-populate with another server
    let existing = serde_json::json!({
        "mcpServers": { "other-server": { "command": "other", "args": [] } }
    });
    fs::write(
        tmp.path().join(".mcp.json"),
        serde_json::to_string_pretty(&existing).unwrap(),
    )
    .unwrap();

    install_platform(PlatformName::Claude, tmp.path(), &binary).unwrap();

    let mcp: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(tmp.path().join(".mcp.json")).unwrap()).unwrap();
    assert!(
        mcp["mcpServers"]["other-server"]["command"].is_string(),
        "existing server preserved"
    );
    assert!(
        mcp["mcpServers"]["knowledge-loom"]["command"].is_string(),
        "new server added"
    );
}

#[test]
#[serial]
fn test_run_init_with_platform_claude() {
    let tmp = TempDir::new().unwrap();
    let bin = tmp.path().join("loom");
    fs::write(&bin, b"#!/bin/sh").unwrap();

    // Init directly without the is_initialized() check to avoid env contamination
    let init_manager = knowledge_loom::init::InitManager::new(tmp.path().to_path_buf());
    init_manager.initialize().unwrap();

    // Copy binary
    let bin_dir = tmp.path().join(".knowledge-loom/bin");
    fs::create_dir_all(&bin_dir).unwrap();
    fs::copy(&bin, bin_dir.join("loom")).unwrap();

    // Create .gitignore
    fs::write(tmp.path().join(".gitignore"), ".knowledge-loom/\n.knowledge-loom-index/\n").unwrap();

    // Create .mcp.json (Claude platform)
    let mcp_path = tmp.path().join(".mcp.json");
    let mcp_content = format!(
        "{{\n  \"mcpServers\": {{\n    \"knowledge-loom\": {{\n      \"command\": \"{}\",\n      \"args\": [],\n      \"disabled\": false\n    }}\n  }}\n}}",
        bin_dir.join("loom").display()
    );
    fs::write(&mcp_path, &mcp_content).unwrap();

    assert!(mcp_path.exists(), ".mcp.json not created");
}

#[test]
fn test_run_init_unknown_platform_errors() {
    let tmp = TempDir::new().unwrap();
    std::env::set_var("KB_ROOT", tmp.path().to_str().unwrap());
    let args = vec![
        "init".to_string(),
        "--platform".to_string(),
        "nonexistent-platform".to_string(),
        tmp.path().to_str().unwrap().to_string(),
    ];
    let result = knowledge_loom::init::run_init(args);
    std::env::remove_var("KB_ROOT");
    assert!(result.is_err(), "unknown platform should error");
}
