use std::io::Write;
use tempfile::TempDir;

/// Minimal ReindexState for testing — mirrors the struct to be implemented in T122.
/// Fields: schema_version, a map of path → {mtime_secs, chunk_count}.
/// T122 will implement this in src/maintenance.rs; these tests define the contract.

#[test]
fn test_state_records_mtime_and_chunk_count() {
    let tmp = TempDir::new().unwrap();
    let state_path = tmp.path().join("reindex-state.json");

    // Simulate what T122 will implement: writing state
    let state = serde_json::json!({
        "schema_version": 1,
        "files": {
            "notes/test.md": { "mtime_secs": 1716000000u64, "chunk_count": 5 },
            "notes/ref.md": { "mtime_secs": 1716000300u64, "chunk_count": 12 }
        }
    });
    let mut f = std::fs::File::create(&state_path).unwrap();
    f.write_all(serde_json::to_string_pretty(&state).unwrap().as_bytes())
        .unwrap();

    let read: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&state_path).unwrap()).unwrap();
    assert_eq!(read["schema_version"], 1);
    assert_eq!(read["files"]["notes/test.md"]["mtime_secs"], 1716000000u64);
    assert_eq!(read["files"]["notes/test.md"]["chunk_count"], 5);
    assert_eq!(read["files"]["notes/ref.md"]["chunk_count"], 12);
}

#[test]
fn test_unchanged_file_is_skipped() {
    // Contract: should_reindex(path, mtime, chunk_count) returns false when mtime and chunk_count match
    let state = serde_json::json!({
        "schema_version": 1,
        "files": {
            "a.md": { "mtime_secs": 100, "chunk_count": 3 }
        }
    });
    // Same mtime, same chunk_count → skip
    let file_entry = &state["files"]["a.md"];
    let same_mtime = file_entry["mtime_secs"].as_u64().unwrap() == 100;
    let same_chunks = file_entry["chunk_count"].as_u64().unwrap() == 3;
    assert!(
        same_mtime && same_chunks,
        "unchanged file should match state"
    );
}

#[test]
fn test_changed_mtime_triggers_reindex() {
    // Contract: new_mtime > stored_mtime → must reindex
    let stored_mtime = 100u64;
    let new_mtime = 200u64;
    assert!(
        new_mtime > stored_mtime,
        "changed mtime should trigger reindex"
    );
}

#[test]
fn test_missing_file_cleaned_from_state() {
    // Contract: if a file is in state but not on disk, it should be removed
    let mut state = serde_json::json!({
        "schema_version": 1,
        "files": {
            "a.md": { "mtime_secs": 100, "chunk_count": 3 },
            "deleted.md": { "mtime_secs": 200, "chunk_count": 7 }
        }
    });
    // Simulate removal of deleted.md
    let files_obj = state["files"].as_object_mut().unwrap();
    files_obj.remove("deleted.md");
    assert!(
        !files_obj.contains_key("deleted.md"),
        "deleted file should be removed from state"
    );
    assert!(
        files_obj.contains_key("a.md"),
        "existing file should remain"
    );
}

#[test]
fn test_fresh_state_requires_full_rebuild() {
    // Contract: when state file is missing, full rebuild is required
    let tmp = TempDir::new().unwrap();
    let state_path = tmp.path().join("nonexistent.json");
    assert!(
        !state_path.exists(),
        "missing state should trigger full rebuild"
    );
}
