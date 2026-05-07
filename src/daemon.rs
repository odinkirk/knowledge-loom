// src/daemon.rs
use std::path::{Path, PathBuf};
use std::fs;
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
