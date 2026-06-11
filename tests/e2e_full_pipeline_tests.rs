use std::fs;
use tempfile::tempdir;

mod e2e_helpers;
use e2e_helpers::{assert_contains, assert_exit_code, assert_no_panic, run_loom_cmd};

#[test]
fn test_full_pipeline_init_install_reindex_incremental() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let kb_root = temp_dir.path();

    // 1. loom init
    let output = run_loom_cmd(&["init"], kb_root);
    assert_exit_code(&output, 0);
    assert_no_panic(&output);

    // 2. loom install (redundant after init, should be idempotent)
    let output = run_loom_cmd(&["install"], kb_root);
    assert_exit_code(&output, 0);
    assert_no_panic(&output);

    // 3. Create markdown files
    let file_a = kb_root.join("alpha.md");
    let file_b = kb_root.join("beta.md");
    fs::write(
        &file_a,
        "# Alpha\n\nFirst test file content.\n\n## Subsection\n\nMore content here.\n",
    )
    .unwrap();
    fs::write(&file_b, "# Beta\n\nSecond test file.\n").unwrap();

    // 4. First reindex: full rebuild
    let output = run_loom_cmd(&["reindex"], kb_root);
    assert_exit_code(&output, 0);
    assert_no_panic(&output);
    // Should print "Full rebuild in progress" since state file doesn't exist yet
    assert_contains(&output, "Full rebuild");

    // 5. Verify state file was saved
    let state_path = kb_root.join(".knowledge-loom-index/reindex-state.json");
    assert!(
        state_path.exists(),
        "reindex-state.json should exist after first reindex"
    );
    let state_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    assert!(state_json["files"]
        .as_object()
        .unwrap()
        .contains_key("alpha.md"));
    assert!(state_json["files"]
        .as_object()
        .unwrap()
        .contains_key("beta.md"));

    // 6. Second reindex: should be incremental (no changes)
    let output = run_loom_cmd(&["reindex"], kb_root);
    assert_exit_code(&output, 0);
    assert_no_panic(&output);
    // Incremental should report "No changes detected" or very fast completion
    assert!(
        output.stdout.contains("No changes")
            || output.stderr.contains("No changes")
            || output.stderr.contains("Incremental")
    );

    // 7. Edit a file externally (simulating user editing via editor)
    let _timestamp_before = fs::metadata(&state_path).unwrap().modified().unwrap();
    fs::write(
        &file_b,
        "# Beta\n\nUpdated second test file with new content.\n",
    )
    .unwrap();

    // 8. Third reindex: incremental should detect the changed file
    let output = run_loom_cmd(&["reindex"], kb_root);
    assert_exit_code(&output, 0);
    assert_no_panic(&output);
    // Should report the changed file count
    assert!(output.stderr.contains("changed") || output.stdout.contains("changed"));

    // 9. State file should have updated mtime for beta.md
    let state_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    let beta_mtime = state_json["files"]["beta.md"]["mtime_secs"]
        .as_u64()
        .unwrap();
    let alpha_mtime = state_json["files"]["alpha.md"]["mtime_secs"]
        .as_u64()
        .unwrap();
    // mtimes should be set (beta may differ from alpha depending on write timing)
    assert!(beta_mtime > 0, "beta.md mtime should be recorded");
    assert!(alpha_mtime > 0, "alpha.md mtime should be recorded");

    // 9a. Verify turbovec index files exist after reindex
    let tvim = kb_root.join(".knowledge-loom-index/turbovec.tvim");
    let meta = kb_root.join(".knowledge-loom-index/turbovec_meta.bin");
    let config = kb_root.join(".knowledge-loom-index/turbovec_config.bin");
    assert!(tvim.exists(), "turbovec.tvim should exist after reindex");
    assert!(
        meta.exists(),
        "turbovec_meta.bin should exist after reindex"
    );
    assert!(
        config.exists(),
        "turbovec_config.bin should exist after reindex"
    );

    // sqlite-vec embeddings.db should NOT exist
    let legacy = kb_root.join(".knowledge-loom-index/embeddings.db");
    assert!(
        !legacy.exists(),
        "Legacy embeddings.db should not exist — turbovec replaced sqlite-vec"
    );

    // Verify turbovec index files have non-zero size
    let tvim_size = fs::metadata(&tvim).unwrap().len();
    let meta_size = fs::metadata(&meta).unwrap().len();
    assert!(tvim_size > 0, "turbovec.tvim should not be empty");
    assert!(meta_size > 0, "turbovec_meta.bin should not be empty");

    // 10. Fourth reindex: should report no changes again
    let output = run_loom_cmd(&["reindex"], kb_root);
    assert_exit_code(&output, 0);
    assert_no_panic(&output);
    assert!(
        output.stdout.contains("No changes")
            || output.stderr.contains("No changes")
            || output.stderr.contains("Incremental")
    );
}

#[test]
fn test_full_pipeline_force_flag_bypasses_incremental() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let kb_root = temp_dir.path();

    let output = run_loom_cmd(&["init"], kb_root);
    assert_exit_code(&output, 0);

    fs::write(kb_root.join("test.md"), "# Test\n\nContent.\n").unwrap();

    // First reindex
    let output = run_loom_cmd(&["reindex"], kb_root);
    assert_exit_code(&output, 0);

    // Second reindex with --force: must trigger full rebuild
    let output = run_loom_cmd(&["reindex", "--force"], kb_root);
    assert_exit_code(&output, 0);
    assert_no_panic(&output);
    assert!(
        output.stderr.contains("Full rebuild") || output.stdout.contains("Full rebuild"),
        "Expected full rebuild message, got stdout={}, stderr={}",
        output.stdout,
        output.stderr
    );
}
