use std::fs;
use tempfile::tempdir;

mod e2e_helpers;
use e2e_helpers::{assert_exit_code, assert_no_panic, run_loom_cmd};

#[test]
fn test_daemon_reindex_state_consistency() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let kb_root = temp_dir.path();
    let state_path = kb_root.join(".knowledge-loom-index/reindex-state.json");

    let output = run_loom_cmd(&["init"], kb_root);
    assert_exit_code(&output, 0);

    // Create initial files
    fs::write(kb_root.join("one.md"), "# One\n\nFile one content.\n").unwrap();
    fs::write(kb_root.join("two.md"), "# Two\n\nFile two content.\n").unwrap();
    fs::write(kb_root.join("three.md"), "# Three\n\nFile three content.\n").unwrap();

    // First reindex: full rebuild
    let output = run_loom_cmd(&["reindex"], kb_root);
    assert_exit_code(&output, 0);
    assert_no_panic(&output);
    assert!(
        state_path.exists(),
        "State file must exist after first reindex"
    );

    // Read state and verify all files tracked
    let state_before: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    let files_before = state_before["files"].as_object().unwrap();
    assert!(files_before.contains_key("one.md"));
    assert!(files_before.contains_key("two.md"));
    assert!(files_before.contains_key("three.md"));
    let mtime_two_before = files_before["two.md"]["mtime_secs"].as_u64().unwrap();

    // Simulate daemon edit: modify two.md externally
    // (this is what happens when a user edits via editor and daemon reindexes)
    fs::write(
        kb_root.join("two.md"),
        "# Two\n\nUpdated file two with more content.\n",
    )
    .unwrap();

    // Reindex should detect change via incremental path
    let output = run_loom_cmd(&["reindex"], kb_root);
    assert_exit_code(&output, 0);
    assert_no_panic(&output);
    assert!(
        output.stderr.contains("changed") || output.stdout.contains("changed"),
        "Should report changed files, got stdout={}, stderr={}",
        output.stdout,
        output.stderr
    );

    // Verify state file updated: two.md mtime should have changed
    let state_after: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    let mtime_two_after = state_after["files"]["two.md"]["mtime_secs"]
        .as_u64()
        .unwrap();
    assert!(
        mtime_two_after != mtime_two_before,
        "mtime for two.md should change after edit + reindex (was {}, now {})",
        mtime_two_before,
        mtime_two_after
    );

    // one.md and three.md mtmes should be unchanged
    let mtime_one_before = files_before["one.md"]["mtime_secs"].as_u64().unwrap();
    let mtime_one_after = state_after["files"]["one.md"]["mtime_secs"]
        .as_u64()
        .unwrap();
    assert_eq!(
        mtime_one_before, mtime_one_after,
        "mtime for one.md should be unchanged"
    );

    let mtime_three_before = files_before["three.md"]["mtime_secs"].as_u64().unwrap();
    let mtime_three_after = state_after["files"]["three.md"]["mtime_secs"]
        .as_u64()
        .unwrap();
    assert_eq!(
        mtime_three_before, mtime_three_after,
        "mtime for three.md should be unchanged"
    );

    // Subsequent reindex should report no changes
    let output = run_loom_cmd(&["reindex"], kb_root);
    assert_exit_code(&output, 0);
    assert_no_panic(&output);
    assert!(
        output.stderr.contains("No changes") || output.stdout.contains("No changes"),
        "Should report no changes on unchanged vault, got stdout={}, stderr={}",
        output.stdout,
        output.stderr
    );
}

#[test]
fn test_daemon_state_persists_across_reindex_runs() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let kb_root = temp_dir.path();
    let state_path = kb_root.join(".knowledge-loom-index/reindex-state.json");

    let output = run_loom_cmd(&["init"], kb_root);
    assert_exit_code(&output, 0);

    fs::write(kb_root.join("a.md"), "# A\n\nContent A.\n").unwrap();

    let output = run_loom_cmd(&["reindex"], kb_root);
    assert_exit_code(&output, 0);
    assert!(state_path.exists());

    let state: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    assert!(state["files"].as_object().unwrap().contains_key("a.md"));
    let chunk_count = state["files"]["a.md"]["chunk_count"].as_u64().unwrap();
    assert!(
        chunk_count > 0,
        "chunk count should be > 0 for non-empty file"
    );

    // Edit file to have a different chunk count
    fs::write(
        kb_root.join("a.md"),
        "# A\n\nShort.\n\n# B\n\nAnother section.\n\n# C\n\nThird section.\n",
    )
    .unwrap();

    let output = run_loom_cmd(&["reindex"], kb_root);
    assert_exit_code(&output, 0);

    let state2: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    let chunk_count2 = state2["files"]["a.md"]["chunk_count"].as_u64().unwrap();
    assert!(
        chunk_count2 != chunk_count,
        "chunk count should change when file content changes section count (was {}, now {})",
        chunk_count,
        chunk_count2
    );
    assert_eq!(chunk_count2, 3, "file with 3 sections should have 3 chunks");
}
