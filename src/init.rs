use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::Path;

pub fn run_init(mut args: impl Iterator<Item = String>) -> Result<(), Box<dyn std::error::Error>> {
    let _ = args.next(); // skip "init"
    let mut platform: Option<String> = None;
    let mut dir_arg: Option<String> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--platform" => {
                platform = args.next();
            }
            other => {
                dir_arg = Some(other.to_string());
            }
        }
    }

    let dir = match dir_arg {
        Some(d) => std::path::PathBuf::from(d),
        None => std::env::current_dir()?,
    };
    let dir = dir
        .canonicalize()
        .map_err(|e| format!("Cannot resolve directory '{}': {e}", dir.display()))?;

    // Resolve binary (existing logic)
    let binary_src = std::env::current_exe()?;
    run_init_with_binary_and_platform(&dir, &binary_src, platform.as_deref())
}

fn write_opencode_json(dir: &Path, binary_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let opencode_path = dir.join("opencode.json");
    let mut root: Value = if opencode_path.exists() {
        serde_json::from_str(&fs::read_to_string(&opencode_path)?)?
    } else {
        serde_json::json!({ "$schema": "https://opencode.ai/config.json" })
    };

    if root.get("mcp").is_none() {
        root["mcp"] = serde_json::json!({});
    }

    root["mcp"]["knowledge-loom"] = serde_json::json!({
        "type": "local",
        "command": [binary_path.to_str().unwrap(), "serve"],
        "environment": {
            "KB_ROOT": dir.to_str().unwrap()
        }
    });

    let tmp_path = opencode_path.with_extension("json.tmp");
    {
        let mut f = fs::File::create(&tmp_path)?;
        write!(f, "{}", serde_json::to_string_pretty(&root)?)?;
    }
    fs::rename(&tmp_path, &opencode_path)?;

    Ok(())
}

pub fn run_init_with_binary(
    dir: &Path,
    binary_src: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Copy binary
    let bin_dir = dir.join(".knowledge-loom/bin");
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

    // 1.5. Emit loom-shell.sh
    let script_path = dir.join("loom-shell.sh");
    let kb_root_placeholder = dir.to_string_lossy();
    let script_content =
        crate::shell::make_shell_script(&dest.to_string_lossy(), &kb_root_placeholder);
    let tmp_script = script_path.with_extension("tmp");
    std::fs::write(&tmp_script, &script_content)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&tmp_script)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&tmp_script, perms)?;
    }
    std::fs::rename(&tmp_script, &script_path)?;

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

    root["mcpServers"]["knowledge-loom"] = serde_json::json!({
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

    // 2.5. Write opencode.json
    write_opencode_json(dir, &dest)?;

    // 3. Update .gitignore
    let gi_path = dir.join(".gitignore");
    let existing_gi = if gi_path.exists() {
        fs::read_to_string(&gi_path)?
    } else {
        String::new()
    };

    let mut additions = Vec::new();
    if !existing_gi.lines().any(|l| l.trim() == ".knowledge-loom/") {
        additions.push(".knowledge-loom/");
    }
    if !existing_gi
        .lines()
        .any(|l| l.trim() == ".knowledge-loom-index/")
    {
        additions.push(".knowledge-loom-index/");
    }

    if !additions.is_empty() {
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&gi_path)?;
        if !existing_gi.is_empty() && !existing_gi.ends_with('\n') {
            writeln!(f)?;
        }
        for entry in &additions {
            writeln!(f, "{entry}")?;
        }
    }

    // 4. Print next steps
    eprintln!("knowledge-loom init complete.");
    eprintln!("  binary:  {}", dest.display());
    eprintln!("  KB_ROOT: {}", dir.display());
    eprintln!("  .mcp.json updated");
    eprintln!("  opencode.json updated");
    eprintln!("  loom-shell.sh created");
    eprintln!();
    eprintln!("Next: restart Claude Code and run /mcp to connect.");

    Ok(())
}

pub fn run_init_with_binary_and_platform(
    dir: &Path,
    binary_src: &Path,
    platform: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Existing binary-copy logic (unchanged)
    run_init_with_binary(dir, binary_src)?;

    let installed_binary = dir.join(".knowledge-loom/bin/loom");

    match platform {
        None | Some("all") => {
            let results = crate::platforms::install_all_detected(dir, &installed_binary);
            for (p, result) in results {
                match result {
                    Ok(files) => eprintln!("  [ok] {p:?}: {} file(s)", files.len()),
                    Err(e) => eprintln!("  [warn] {p:?}: {e}"),
                }
            }
        }
        Some(name) => {
            let p = crate::platforms::PlatformName::from_str(name)
                .ok_or_else(|| format!("unknown platform: {name}. Use one of: claude, cursor, windsurf, zed, continue, opencode, kiro, codex, all"))?;
            let files = crate::platforms::install_platform(p, dir, &installed_binary)?;
            for f in &files {
                eprintln!("  wrote {}", f.display());
            }
        }
    }
    eprintln!("knowledge-loom init complete.");
    Ok(())
}
