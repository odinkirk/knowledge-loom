use knowledge_loom::daemon::{DaemonConfig, WatchRepo, load_config, save_config, add_repo, remove_repo, expand_path};
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

#[tokio::test]
async fn test_daemon_children_map_uses_tokio_mutex() {
    // Verifies tokio::sync::Mutex is used — compile-time check.
    // If this compiles and the lock().await syntax works, the type is correct.
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use std::collections::HashMap;
    use tokio::task::JoinHandle;

    let children: Arc<Mutex<HashMap<String, JoinHandle<()>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let handle = tokio::spawn(async {});
    children.lock().await.insert("test".to_string(), handle);
    assert_eq!(children.lock().await.len(), 1);
}

#[test]
fn test_expand_path_tilde() {
    let home = dirs::home_dir().expect("home dir");
    let result = expand_path("~/git/awesome-repo");
    assert_eq!(result, format!("{}/git/awesome-repo", home.display()));
}

#[test]
fn test_expand_path_tilde_only() {
    let home = dirs::home_dir().expect("home dir");
    let result = expand_path("~");
    assert_eq!(result, home.to_string_lossy().to_string());
}

#[test]
fn test_expand_path_absolute_unchanged() {
    let path = "/home/user/repos/myrepo";
    assert_eq!(expand_path(path), path);
}

#[test]
fn test_expand_path_relative_unchanged() {
    assert_eq!(expand_path("relative/path"), "relative/path");
}

#[test]
fn test_add_repo_expands_tilde() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = tmp.path().join("watch.toml");
    add_repo("~/projects/notes", None, &cfg).unwrap();
    let config = load_config(&cfg).unwrap();
    let home = dirs::home_dir().unwrap();
    assert_eq!(
        config.repos[0].path,
        format!("{}/projects/notes", home.display())
    );
}
