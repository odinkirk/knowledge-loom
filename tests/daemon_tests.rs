use knowledge_loom::daemon::{DaemonConfig, WatchRepo, load_config, save_config, add_repo, remove_repo};
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

#[test]
fn test_daemon_add_repo() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("watch.toml");
    fs::write(&config_path, "").unwrap();

    knowledge_loom::daemon::add_repo(
        "/tmp/mynotes",
        Some("notes"),
        &config_path,
    ).unwrap();

    let config = load_config(&config_path).unwrap();
    assert_eq!(config.repos.len(), 1);
    assert_eq!(config.repos[0].alias, "notes");
    assert_eq!(config.repos[0].path, "/tmp/mynotes");
}

#[test]
fn test_daemon_add_repo_deduplicates() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("watch.toml");
    fs::write(&config_path, "").unwrap();

    knowledge_loom::daemon::add_repo("/tmp/notes", Some("notes"), &config_path).unwrap();
    knowledge_loom::daemon::add_repo("/tmp/notes", Some("notes"), &config_path).unwrap();

    let config = load_config(&config_path).unwrap();
    assert_eq!(config.repos.len(), 1, "should deduplicate");
}

#[test]
fn test_daemon_remove_repo() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("watch.toml");

    let config = DaemonConfig {
        poll_interval: 30,
        repos: vec![
            WatchRepo { path: "/tmp/a".into(), alias: "a".into() },
            WatchRepo { path: "/tmp/b".into(), alias: "b".into() },
        ],
    };
    save_config(&config, &config_path).unwrap();

    knowledge_loom::daemon::remove_repo("a", &config_path).unwrap();

    let updated = load_config(&config_path).unwrap();
    assert_eq!(updated.repos.len(), 1);
    assert_eq!(updated.repos[0].alias, "b");
}
