use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use serde_json::Value;

pub fn run_init(mut args: impl Iterator<Item = String>) -> Result<(), Box<dyn std::error::Error>> {
    // Skip "init" arg
    let _ = args.next();
    let dir = args.next()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());
    let dir = dir.canonicalize()
        .map_err(|e| format!("Cannot resolve directory '{}': {e}", dir.display()))?;
    let binary_src = std::env::current_exe()?;
    run_init_with_binary(&dir, &binary_src)
}

pub fn run_init_with_binary(dir: &Path, binary_src: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Copy binary
    let bin_dir = dir.join(".loom/bin");
    fs::create_dir_all(&bin_dir)?;
    let dest = bin_dir.join("loom");
    fs::copy(binary_src, &dest)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&dest)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&dest, perms)?;
    }

    // 2. Merge .mcp.json
    let mcp_path = dir.join(".mcp.json");
    let mut root: Value = if mcp_path.exists() {
        serde_json::from_str(&fs::read_to_string(&mcp_path)?)?
    } else {
        serde_json::json!({ "mcpServers": {} })
    };

    if root.get("mcpServers").is_none() {
        root["mcpServers"] = serde_json::json!({});
    }

    root["mcpServers"]["loom"] = serde_json::json!({
        "command": dest.to_str().unwrap(),
        "env": {
            "KB_ROOT": dir.to_str().unwrap()
        }
    });

    // Atomic write: write to temp file, then rename
    let tmp_path = mcp_path.with_extension("json.tmp");
    {
        let mut f = fs::File::create(&tmp_path)?;
        write!(f, "{}", serde_json::to_string_pretty(&root)?)?;
    }
    fs::rename(&tmp_path, &mcp_path)?;

    // 3. Update .gitignore
    let gi_path = dir.join(".gitignore");
    let existing_gi = if gi_path.exists() {
        fs::read_to_string(&gi_path)?
    } else {
        String::new()
    };

    let mut additions = Vec::new();
    if !existing_gi.lines().any(|l| l.trim() == ".loom/") {
        additions.push(".loom/");
    }
    if !existing_gi.lines().any(|l| l.trim() == ".loom-index/") {
        additions.push(".loom-index/");
    }

    if !additions.is_empty() {
        let mut f = fs::OpenOptions::new().create(true).append(true).open(&gi_path)?;
        if !existing_gi.is_empty() && !existing_gi.ends_with('\n') {
            writeln!(f)?;
        }
        for entry in &additions {
            writeln!(f, "{entry}")?;
        }
    }

    // 4. Print next steps
    println!("loom init complete.");
    println!("  binary:  {}", dest.display());
    println!("  KB_ROOT: {}", dir.display());
    println!("  .mcp.json updated");
    println!();
    println!("Next: restart Claude Code and run /mcp to connect.");

    Ok(())
}
