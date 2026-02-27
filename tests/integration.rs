use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn romv_bin() -> PathBuf {
    // cargo test puts the binary in target/debug
    let mut path = PathBuf::from(env!("CARGO_BIN_EXE_romv"));
    if !path.exists() {
        path = PathBuf::from("target/debug/romv");
    }
    path
}

fn create_temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("failed to create temp dir")
}

#[test]
fn dry_run_does_not_rename() {
    let dir = create_temp_dir();
    let file = dir.path().join("テスト.txt");
    fs::write(&file, "").unwrap();

    let output = Command::new(romv_bin())
        .arg(file.to_str().unwrap())
        .output()
        .unwrap();

    assert!(output.status.success());
    // File should still exist with original name
    assert!(
        file.exists(),
        "original file should still exist after dry-run"
    );
    assert!(
        !dir.path().join("tesuto.txt").exists(),
        "target should not exist after dry-run"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("->"), "should show rename preview");
}

#[test]
fn rename_with_yes_flag() {
    let dir = create_temp_dir();
    let file = dir.path().join("テスト.txt");
    fs::write(&file, "content").unwrap();

    let output = Command::new(romv_bin())
        .args(["-y", file.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(!file.exists(), "original should be gone");
    let target = dir.path().join("tesuto.txt");
    assert!(target.exists(), "target should exist");
    assert_eq!(fs::read_to_string(&target).unwrap(), "content");
}

#[test]
fn no_clobber() {
    let dir = create_temp_dir();
    let source = dir.path().join("テスト.txt");
    let target = dir.path().join("tesuto.txt");
    fs::write(&source, "source").unwrap();
    fs::write(&target, "existing").unwrap();

    let output = Command::new(romv_bin())
        .args(["-y", source.to_str().unwrap()])
        .output()
        .unwrap();

    // Should fail because target exists
    assert!(!output.status.success());
    // Source should still exist
    assert!(source.exists());
    // Target should be unchanged
    assert_eq!(fs::read_to_string(&target).unwrap(), "existing");
}

#[test]
fn batch_collision_detection() {
    let dir = create_temp_dir();
    // Pass the same file twice to trigger collision detection
    let file = dir.path().join("テスト.txt");
    fs::write(&file, "").unwrap();
    let path_str = file.to_str().unwrap();

    let output = Command::new(romv_bin())
        .args(["-y", path_str, path_str])
        .output()
        .unwrap();

    // Should detect collision and abort
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("collision"),
        "should report collision, got: {stderr}"
    );
}

#[test]
fn skip_unchanged_ascii() {
    let dir = create_temp_dir();
    let file = dir.path().join("hello.txt");
    fs::write(&file, "").unwrap();

    let output = Command::new(romv_bin())
        .args(["-y", file.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    // File should remain
    assert!(file.exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Nothing to rename"),
        "should indicate nothing to do, got: {stderr}"
    );
}

#[test]
fn stdin_input() {
    let dir = create_temp_dir();
    let file = dir.path().join("テスト.txt");
    fs::write(&file, "content").unwrap();

    let mut child = Command::new(romv_bin())
        .args(["-y"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(format!("{}\n", file.to_str().unwrap()).as_bytes())
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(!file.exists(), "original should be renamed");
    assert!(
        dir.path().join("tesuto.txt").exists(),
        "target should exist"
    );
}

#[test]
fn hidden_file_rename() {
    let dir = create_temp_dir();
    let file = dir.path().join(".テスト.conf");
    fs::write(&file, "").unwrap();

    let output = Command::new(romv_bin())
        .args(["-y", file.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(!file.exists());
    let target = dir.path().join(".tesuto.conf");
    assert!(
        target.exists(),
        "hidden file should be renamed preserving dot"
    );
}

#[test]
fn verbose_output() {
    let dir = create_temp_dir();
    let file = dir.path().join("テスト.txt");
    fs::write(&file, "").unwrap();

    let output = Command::new(romv_bin())
        .args(["-y", "-v", file.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("1 renamed"),
        "verbose should show summary, got: {stderr}"
    );
}

#[test]
fn custom_separator() {
    let dir = create_temp_dir();
    let file = dir.path().join("新しい ファイル.txt");
    fs::write(&file, "").unwrap();

    let output = Command::new(romv_bin())
        .args(["-y", "-s", "-", file.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    // The converted name should use '-' instead of '_'
    let entries: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    assert!(
        entries.iter().any(|e| !e.contains(' ')),
        "spaces should be replaced, entries: {entries:?}"
    );
}

#[test]
fn nonexistent_source() {
    let output = Command::new(romv_bin())
        .args(["-y", "/tmp/definitely_does_not_exist_テスト.txt"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found"),
        "should report source not found, got: {stderr}"
    );
}
