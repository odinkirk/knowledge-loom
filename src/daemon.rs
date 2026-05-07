// src/daemon.rs
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct WatchRepo {
    pub path: String,
    pub alias: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DaemonConfig {
    #[serde(default = "default_poll")]
    pub poll_interval: u64,
    #[serde(default)]
    pub repos: Vec<WatchRepo>,
}

fn default_poll() -> u64 { 30 }

impl Default for DaemonConfig {
    fn default() -> Self { Self { poll_interval: 30, repos: Vec::new() } }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RepoStateEntry {
    pub pid: u32,
    pub path: String,
    pub status: String,
}

pub fn config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".knowledge-loom")
}

pub fn config_path() -> PathBuf { config_dir().join("watch.toml") }
pub fn pid_path() -> PathBuf { config_dir().join("daemon.pid") }
pub fn state_path() -> PathBuf { config_dir().join("daemon-state.json") }
pub fn log_dir() -> PathBuf { config_dir().join("logs") }

pub fn load_config(path: &Path) -> Result<DaemonConfig, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    if content.trim().is_empty() {
        return Ok(DaemonConfig::default());
    }
    // Parse: [daemon] section + [[repos]] array
    #[derive(Deserialize)]
    struct TomlFile {
        #[serde(default)]
        daemon: DaemonSection,
        #[serde(default)]
        repos: Vec<WatchRepo>,
    }
    #[derive(Deserialize, Default)]
    struct DaemonSection {
        #[serde(default = "default_poll")]
        poll_interval: u64,
    }
    let file: TomlFile = toml::from_str(&content)?;
    Ok(DaemonConfig { poll_interval: file.daemon.poll_interval, repos: file.repos })
}

pub fn save_config(config: &DaemonConfig, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(path.parent().unwrap())?;
    let mut out = format!("[daemon]\npoll_interval = {}\n", config.poll_interval);
    for repo in &config.repos {
        out.push_str(&format!("\n[[repos]]\npath = \"{}\"\nalias = \"{}\"\n", repo.path, repo.alias));
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, out)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

fn read_pid() -> Option<u32> {
    fs::read_to_string(pid_path()).ok()?.trim().parse().ok()
}

fn write_pid(pid: u32) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(config_dir())?;
    fs::write(pid_path(), pid.to_string())?;
    Ok(())
}

fn read_state() -> std::collections::HashMap<String, RepoStateEntry> {
    fs::read_to_string(state_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn write_state(state: &std::collections::HashMap<String, RepoStateEntry>) {
    if let Ok(json) = serde_json::to_string_pretty(state) {
        let _ = fs::write(state_path(), json);
    }
}

pub fn add_repo(
    path: &str,
    alias: Option<&str>,
    config_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = if config_path.exists() {
        load_config(config_path)?
    } else {
        DaemonConfig::default()
    };
    let alias = alias
        .map(|s| s.to_string())
        .unwrap_or_else(|| Path::new(path).file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "repo".to_string()));
    // Deduplicate by path
    if config.repos.iter().any(|r| r.path == path) {
        return Ok(());
    }
    config.repos.push(WatchRepo { path: path.to_string(), alias });
    save_config(&config, config_path)
}

pub fn remove_repo(
    path_or_alias: &str,
    config_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = load_config(config_path)?;
    config.repos.retain(|r| r.alias != path_or_alias && r.path != path_or_alias);
    save_config(&config, config_path)
}

pub async fn run_daemon_foreground() -> Result<(), Box<dyn std::error::Error>> {
    let cfg_path = config_path();
    fs::create_dir_all(log_dir())?;

    let config = if cfg_path.exists() {
        load_config(&cfg_path)?
    } else {
        DaemonConfig::default()
    };

    eprintln!("[daemon] watching {} repo(s)", config.repos.len());

    // Shared child map: alias -> tokio JoinHandle
    let children: Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    for repo in &config.repos {
        let handle = spawn_watcher(repo.clone());
        children.lock().unwrap().insert(repo.alias.clone(), handle);
    }

    // Health-check loop
    let poll = tokio::time::Duration::from_secs(config.poll_interval);
    let mut ticker = tokio::time::interval(poll);
    loop {
        ticker.tick().await;
        let mut dead = Vec::new();
        {
            let map = children.lock().unwrap();
            for (alias, handle) in map.iter() {
                if handle.is_finished() {
                    dead.push(alias.clone());
                }
            }
        }
        // Restart dead watchers
        let cfg = load_config(&cfg_path).unwrap_or_default();
        for alias in dead {
            eprintln!("[daemon] restarting watcher for {alias}");
            if let Some(repo) = cfg.repos.iter().find(|r| r.alias == alias) {
                let handle = spawn_watcher(repo.clone());
                children.lock().unwrap().insert(alias, handle);
            }
        }
    }
}

fn spawn_watcher(repo: WatchRepo) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let _log_file_path = log_dir().join(format!("{}.log", repo.alias));
        let binary = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("loom"));

        // Use notify for FS events; trigger reindex via HTTP or direct call
        use notify::{Watcher, RecursiveMode, recommended_watcher, Event};
        use std::sync::mpsc;

        let (tx, rx) = mpsc::channel::<Result<Event, notify::Error>>();
        let mut watcher = match recommended_watcher(tx) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("[watcher:{}] notify failed: {e}, falling back to polling", repo.alias);
                // Polling fallback: just trigger a full reindex every poll_interval
                loop {
                    trigger_reindex(&repo.path, &binary).await;
                    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                }
            }
        };

        if let Err(e) = watcher.watch(std::path::Path::new(&repo.path), RecursiveMode::Recursive) {
            eprintln!("[watcher:{}] watch failed: {e}", repo.alias);
            return;
        }

        eprintln!("[watcher:{}] watching {}", repo.alias, repo.path);

        // Drain FS events, debounce, trigger reindex
        let mut last_event = std::time::Instant::now();
        loop {
            match rx.recv_timeout(std::time::Duration::from_secs(1)) {
                Ok(Ok(event)) => {
                    // Only care about markdown changes
                    let is_md = event.paths.iter().any(|p| {
                        p.extension().map(|e| e == "md").unwrap_or(false)
                    });
                    if is_md && last_event.elapsed() > std::time::Duration::from_secs(2) {
                        last_event = std::time::Instant::now();
                        trigger_reindex(&repo.path, &binary).await;
                    }
                }
                Ok(Err(e)) => eprintln!("[watcher:{}] event error: {e}", repo.alias),
                Err(mpsc::RecvTimeoutError::Timeout) => {} // no event, loop
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    })
}

async fn trigger_reindex(kb_root: &str, binary: &Path) {
    // Spawn a one-shot `loom serve` process that reindexes and exits.
    // We use the `loom_reindex` MCP tool by spawning with a flag.
    // For now, spawn binary with KB_ROOT set and a dedicated `reindex` subcommand.
    // (This requires adding a `reindex` subcommand to main.rs — see Task 3.)
    let status = tokio::process::Command::new(binary)
        .arg("reindex")
        .env("KB_ROOT", kb_root)
        .status()
        .await;
    match status {
        Ok(s) if s.success() => eprintln!("[daemon] reindexed {kb_root}"),
        Ok(s) => eprintln!("[daemon] reindex exit {s} for {kb_root}"),
        Err(e) => eprintln!("[daemon] reindex error for {kb_root}: {e}"),
    }
}

pub fn daemonize() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(unix)]
    {
        use nix::unistd::{fork, ForkResult, setsid};
        match unsafe { fork()? } {
            ForkResult::Parent { .. } => std::process::exit(0),
            ForkResult::Child => {}
        }
        setsid()?;
        match unsafe { fork()? } {
            ForkResult::Parent { .. } => std::process::exit(0),
            ForkResult::Child => {}
        }
        // Redirect stdio
        let dev_null = std::fs::File::open("/dev/null")?;
        use std::os::unix::io::IntoRawFd;
        let null_fd = dev_null.into_raw_fd();
        unsafe {
            libc::dup2(null_fd, 0);
            libc::dup2(null_fd, 1);
        }
        // Stderr → daemon log
        fs::create_dir_all(log_dir())?;
        let log = std::fs::OpenOptions::new()
            .create(true).append(true)
            .open(log_dir().join("daemon.log"))?;
        unsafe { libc::dup2(log.into_raw_fd(), 2); }
    }
    Ok(())
}

pub fn daemon_stop() -> Result<(), Box<dyn std::error::Error>> {
    match read_pid() {
        Some(pid) => {
            #[cfg(unix)]
            unsafe { libc::kill(pid as i32, libc::SIGTERM); }
            fs::remove_file(pid_path()).ok();
            eprintln!("Sent SIGTERM to daemon (PID {pid}).");
        }
        None => eprintln!("No daemon PID file found — daemon may not be running."),
    }
    Ok(())
}

pub fn daemon_status() {
    match read_pid() {
        Some(pid) => {
            eprintln!("Daemon PID: {pid}");
            let state = read_state();
            for (alias, entry) in &state {
                eprintln!("  {alias}: {} ({})", entry.status, entry.path);
            }
        }
        None => eprintln!("Daemon is not running (no PID file)."),
    }
}

pub fn daemon_logs(repo: Option<&str>, follow: bool, lines: usize) {
    let log_path = match repo {
        Some(alias) => log_dir().join(format!("{alias}.log")),
        None => log_dir().join("daemon.log"),
    };
    if !log_path.exists() {
        eprintln!("Log file not found: {}", log_path.display());
        return;
    }
    if follow {
        let _ = std::process::Command::new("tail")
            .arg("-f").arg("-n").arg(lines.to_string()).arg(&log_path)
            .status();
    } else {
        let _ = std::process::Command::new("tail")
            .arg("-n").arg(lines.to_string()).arg(&log_path)
            .status();
    }
}
