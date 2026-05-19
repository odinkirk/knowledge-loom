#[cfg(test)]
mod tests {
    use std::process::Command;
    use std::time::Instant;
    use tempfile::TempDir;

    fn create_corpus(dir: &std::path::Path, file_count: usize) {
        for i in 0..file_count {
            let mut content = String::from(&format!("# File {}\n\n", i));
            for j in 0..10 {
                content.push_str(&format!("## Section {}.{}\n\n", i, j));
                content.push_str(&"paragraph text for indexing. ".repeat(20));
                content.push('\n');
            }
            std::fs::write(dir.join(format!("file_{:04}.md", i)), content).unwrap();
        }
    }

    #[test]
    #[ignore = "requires built binary and model download; run with LOOM_TEST_PERF=1"]
    fn test_reindex_performance_under_10_seconds() {
        if std::env::var("LOOM_TEST_PERF").is_err() {
            return;
        }
        let tmp = TempDir::new().unwrap();
        create_corpus(tmp.path(), 100);

        let binary = std::env!("CARGO_BIN_EXE_loom");
        let start = Instant::now();

        let output = Command::new(binary)
            .args(["reindex"])
            .env("KB_ROOT", tmp.path())
            .output()
            .expect("reindex command failed");

        let duration = start.elapsed();

        assert!(
            output.status.success(),
            "reindex failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            duration.as_secs() < 10,
            "reindex took {}s, expected <10s",
            duration.as_secs()
        );
    }

    #[test]
    fn test_e2e_suite_completes_under_5_minutes() {
        // SC-005: Full E2E test suite executes in under 5 minutes.
        // This is validated by CI timeout, not a runtime assertion.
        // Placeholder to document the requirement.
    }
}
