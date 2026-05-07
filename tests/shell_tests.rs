// Tests shell.rs public interface without spawning real processes
use knowledge_loom::shell::make_shell_script;

#[test]
fn test_make_shell_script_contains_kb_root() {
    let script = make_shell_script("/usr/local/bin/loom", "/home/user/notes");
    assert!(script.contains("KB_ROOT=\"/home/user/notes\"") || script.contains("KB_ROOT=/home/user/notes"), "script: {script}");
    assert!(script.contains("exec"), "should exec shell: {script}");
    assert!(script.contains("/usr/local/bin/loom"), "binary path in script: {script}");
}

#[test]
fn test_make_shell_script_has_shebang() {
    let script = make_shell_script("/usr/bin/loom", "/tmp/kb");
    assert!(script.starts_with("#!/bin/sh"), "missing shebang: {script}");
}

#[test]
fn test_init_emits_shell_script() {
    let tmp = tempfile::TempDir::new().unwrap();
    let bin = tmp.path().join("loom");
    std::fs::write(&bin, b"#!/bin/sh").unwrap();

    knowledge_loom::init::run_init_with_binary(tmp.path(), &bin).unwrap();

    let script_path = tmp.path().join("loom-shell.sh");
    assert!(script_path.exists(), "loom-shell.sh not created");
    let script = std::fs::read_to_string(&script_path).unwrap();
    assert!(script.starts_with("#!/bin/sh"));
    assert!(script.contains("KB_ROOT="));

    // Should be executable on unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&script_path).unwrap().permissions().mode();
        assert!(mode & 0o111 != 0, "script not executable");
    }
}
