#![allow(dead_code)]

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const MCP_CONFIG_KEY: &str = "knowledge-loom";
const AGENT_INSTRUCTIONS: &str = r"# Knowledge Loom MCP Tools

This project uses knowledge-loom as its MCP server. Use `loom_*` tools
before grep, glob, or file reads.

## Tools

- `loom_search(query)` — BM25+semantic RRF search; returns file+heading+line_start
- `loom_search_file(file, query)` — search within one file
- `loom_outline(file)` — heading hierarchy with line numbers
- `loom_read_section(file, heading)` — content under a heading
- `loom_read_lines(file, start, end)` — exact line range
- `loom_grep(pattern)` — regex across all files
- `loom_list_files` — enumerate all markdown files
- `loom_replace_lines(file, start, end, content)` — in-place edit
- `loom_insert_after_heading(file, heading, content)` — insert under heading
- `loom_append_to_file(file, content)` — append content
- `loom_create_note(title, content)` — create new note
- `loom_reindex` — rebuild indexes after external edits
- `loom_index_status` — check index health
- `loom_rank_notes` — PageRank influence ranking
- `loom_find_connections(note)` — graph links for a note
- `loom_detect_themes` — community detection

## Workflow

1. Start with `loom_search` or `loom_outline` — never read whole files first.
2. Use `loom_read_section` or `loom_read_lines` for targeted reads.
3. Use `loom_replace_lines` or `loom_insert_after_heading` for surgical edits.
4. Call `loom_reindex` after external file changes.
";

#[derive(Debug, Clone, PartialEq)]
pub enum PlatformName {
    Claude,
    Cursor,
    Windsurf,
    Zed,
    Continue,
    OpenCode,
    Kiro,
    Codex,
}

impl PlatformName {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "claude" => Some(Self::Claude),
            "cursor" => Some(Self::Cursor),
            "windsurf" => Some(Self::Windsurf),
            "zed" => Some(Self::Zed),
            "continue" => Some(Self::Continue),
            "opencode" => Some(Self::OpenCode),
            "kiro" => Some(Self::Kiro),
            "codex" => Some(Self::Codex),
            _ => None,
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::Claude,
            Self::Cursor,
            Self::Windsurf,
            Self::Zed,
            Self::Continue,
            Self::OpenCode,
            Self::Kiro,
            Self::Codex,
        ]
    }

    fn is_detected(&self) -> bool {
        match self {
            Self::Claude | Self::OpenCode | Self::Kiro | Self::Codex => true,
            Self::Cursor => dirs::home_dir()
                .map(|h| h.join(".cursor").exists())
                .unwrap_or(false),
            Self::Windsurf => dirs::home_dir()
                .map(|h| h.join(".codeium/windsurf").exists())
                .unwrap_or(false),
            Self::Zed => dirs::home_dir()
                .map(|h| {
                    #[cfg(target_os = "macos")]
                    return h.join("Library/Application Support/Zed").exists();
                    #[cfg(not(target_os = "macos"))]
                    return h.join(".config/zed").exists();
                })
                .unwrap_or(false),
            Self::Continue => dirs::home_dir()
                .map(|h| h.join(".continue").exists())
                .unwrap_or(false),
        }
    }
}

pub fn install_platform(
    platform: PlatformName,
    repo_root: &Path,
    binary: &Path,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut written = Vec::new();
    let binary_str = binary.to_string_lossy().to_string();

    match &platform {
        PlatformName::Claude => {
            let path = repo_root.join(".mcp.json");
            write_json_object_entry(&path, "mcpServers", &binary_str, true)?;
            written.push(path);
        }
        PlatformName::Cursor => {
            let dir = repo_root.join(".cursor");
            fs::create_dir_all(&dir)?;
            let path = dir.join("mcp.json");
            write_json_object_entry(&path, "mcpServers", &binary_str, true)?;
            written.push(path);
            let rules = repo_root.join(".cursorrules");
            write_instruction_file(&rules)?;
            written.push(rules);
        }
        PlatformName::Windsurf => {
            let path = dirs::home_dir()
                .ok_or("no home dir")?
                .join(".codeium/windsurf/mcp_config.json");
            fs::create_dir_all(path.parent().unwrap())?;
            write_json_object_entry(&path, "mcpServers", &binary_str, false)?;
            written.push(path);
            let rules = repo_root.join(".windsurfrules");
            write_instruction_file(&rules)?;
            written.push(rules);
        }
        PlatformName::Zed => {
            let path = zed_settings_path()?;
            fs::create_dir_all(path.parent().unwrap())?;
            write_json_object_entry(&path, "context_servers", &binary_str, false)?;
            written.push(path);
        }
        PlatformName::Continue => {
            let path = dirs::home_dir()
                .ok_or("no home dir")?
                .join(".continue/config.json");
            fs::create_dir_all(path.parent().unwrap())?;
            write_json_array_entry(&path, "mcpServers", &binary_str)?;
            written.push(path);
        }
        PlatformName::OpenCode => {
            let path = repo_root.join("opencode.json");
            let full_path = std::path::PathBuf::from(repo_root);
            let kb_root = format!("{}", full_path.display());
            let entry = serde_json::json!({
                "$schema": "https://opencode.ai/config.json",
                "mcp": {
                    "knowledge-loom": {
                        "type": "local",
                        "command": [binary_str, "serve".to_string()],
                        "environment": { "KB_ROOT": kb_root },
                        "enabled": true
                    }
                }
            });
            write_json_atomic(&path, &entry)?;
            written.push(path);
            let agents = repo_root.join("AGENTS.md");
            write_instruction_file(&agents)?;
            written.push(agents);
        }
        PlatformName::Kiro => {
            let dir = repo_root.join(".kiro/settings");
            fs::create_dir_all(&dir)?;
            let path = dir.join("mcp.json");
            write_json_object_entry(&path, "mcpServers", &binary_str, true)?;
            written.push(path);
            let agents = repo_root.join("AGENTS.md");
            write_instruction_file(&agents)?;
            written.push(agents);
        }
        PlatformName::Codex => {
            let path = dirs::home_dir()
                .ok_or("no home dir")?
                .join(".codex/config.toml");
            fs::create_dir_all(path.parent().unwrap())?;
            write_toml_entry(&path, &binary_str)?;
            written.push(path);
        }
    }
    Ok(written)
}

pub fn install_all_detected(
    repo_root: &Path,
    binary: &Path,
) -> Vec<(
    PlatformName,
    Result<Vec<PathBuf>, Box<dyn std::error::Error>>,
)> {
    PlatformName::all()
        .into_iter()
        .filter(|p| p.is_detected())
        .map(|p| {
            let result = install_platform(p.clone(), repo_root, binary);
            (p, result)
        })
        .collect()
}

fn build_entry(binary: &str, needs_type: bool) -> Value {
    let mut entry = serde_json::json!({
        "command": binary,
        "args": ["serve"],
        "env": { "KB_ROOT": "." }
    });
    if needs_type {
        entry["type"] = Value::String("stdio".into());
    }
    entry
}

fn write_json_object_entry(
    path: &Path,
    servers_key: &str,
    binary: &str,
    needs_type: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut root: Value = if path.exists() {
        serde_json::from_str(&fs::read_to_string(path)?)?
    } else {
        serde_json::json!({})
    };
    if root[servers_key].is_null() {
        root[servers_key] = serde_json::json!({});
    }
    root[servers_key][MCP_CONFIG_KEY] = build_entry(binary, needs_type);
    write_json_atomic(path, &root)
}

fn write_json_array_entry(
    path: &Path,
    servers_key: &str,
    binary: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut root: Value = if path.exists() {
        serde_json::from_str(&fs::read_to_string(path)?)?
    } else {
        serde_json::json!({})
    };
    let entry = {
        let mut e = build_entry(binary, true);
        e["name"] = Value::String(MCP_CONFIG_KEY.to_string());
        e
    };
    match root[servers_key].as_array_mut() {
        Some(arr) => {
            arr.retain(|v| v["name"] != MCP_CONFIG_KEY);
            arr.push(entry);
        }
        None => {
            root[servers_key] = Value::Array(vec![entry]);
        }
    }
    write_json_atomic(path, &root)
}

fn write_toml_entry(path: &Path, binary: &str) -> Result<(), Box<dyn std::error::Error>> {
    let section = format!(
        "\n[mcp_servers.{}]\ncommand = \"{}\"\nargs = [\"serve\"]\ntype = \"stdio\"\n",
        MCP_CONFIG_KEY, binary
    );
    let existing = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    if !existing.contains(&format!("[mcp_servers.{}]", MCP_CONFIG_KEY)) {
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, existing + &section)?;
        fs::rename(&tmp, path)?;
    }
    Ok(())
}

fn write_instruction_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    const MARKER: &str = "<!-- knowledge-loom MCP tools -->";
    let existing = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    if existing.contains(MARKER) {
        return Ok(());
    }
    let content = format!("{MARKER}\n{AGENT_INSTRUCTIONS}\n{existing}");
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, content)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

fn write_json_atomic(path: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, serde_json::to_string_pretty(value)?)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

fn zed_settings_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = dirs::home_dir().ok_or("no home dir")?;
    #[cfg(target_os = "macos")]
    return Ok(home.join("Library/Application Support/Zed/settings.json"));
    #[cfg(not(target_os = "macos"))]
    return Ok(home.join(".config/zed/settings.json"));
}
