use knowledge_loom::daemon::{DaemonConfig, WatchRepo, load_config, save_config};
use tempfile::TempDir;
use std::fs;

#[test]
fn test_load_config_roundtrip() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("watch.toml");

    let original = DaemonConfig {
        poll_interval: 30,
        repos: vec![
            WatchRepo { path: "/tmp/notes".to_string(), alias: "notes".to_string() },
            WatchRepo { path: "/tmp/work".to_string(), alias: "work".to_string() },
        ],
    };

    save_config(&original, &config_path).unwrap();
    let loaded = load_config(&config_path).unwrap();

    assert_eq!(loaded.poll_interval, 30);
    assert_eq!(loaded.repos.len(), 2);
    assert_eq!(loaded.repos[0].alias, "notes");
    assert_eq!(loaded.repos[1].path, "/tmp/work");
}

#[test]
fn test_load_config_empty_file_gives_defaults() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("watch.toml");
    fs::write(&config_path, "").unwrap();
    let config = load_config(&config_path).unwrap();
    assert_eq!(config.repos.len(), 0);
}

#[test]
fn test_config_dir_path() {
    let dir = knowledge_loom::daemon::config_dir();
    assert!(dir.to_string_lossy().contains("knowledge-loom"),
        "config dir should contain knowledge-loom: {}", dir.display());
}
