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
