use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "envsentry-cli-test-{}-{}",
            std::process::id(),
            NEXT_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn write(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.0.join(name);
        std::fs::write(&path, contents).unwrap();
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_envsentry")).args(args).output().unwrap()
}

#[test]
fn compare_emits_value_free_json_and_allows_warnings_by_default() {
    let dir = TempDir::new();
    let reference = dir.write("reference.env", "TOKEN=REFERENCE_SECRET_SENTINEL\n");
    let target = dir.write("target.env", "TOKEN=TARGET_SECRET_SENTINEL\n");
    let output = run(&[
        "compare",
        reference.to_str().unwrap(),
        target.to_str().unwrap(),
        "--format",
        "json",
    ]);

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("REFERENCE_SECRET_SENTINEL"));
    assert!(!stdout.contains("TARGET_SECRET_SENTINEL"));
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(report["diagnostics"][0]["code"], "VALUE_MISMATCH");
}

#[test]
fn strict_mode_fails_on_warnings() {
    let dir = TempDir::new();
    let reference = dir.write("reference.env", "PORT=3000\n");
    let target = dir.write("target.env", "PORT=4000\n");
    let output = Command::new(env!("CARGO_BIN_EXE_envsentry"))
        .arg("compare")
        .arg(reference)
        .arg(target)
        .arg("--strict")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn check_ignores_value_differences_but_fails_on_missing_keys() {
    let dir = TempDir::new();
    let schema = dir.write("schema.json", r#"{"port":3000,"required":true}"#);
    let target = dir.write("target.json", r#"{"port":4000}"#);
    let output = Command::new(env!("CARGO_BIN_EXE_envsentry"))
        .arg("check")
        .arg("--schema")
        .arg(schema)
        .arg(target)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("MISSING_KEY"));
    assert!(!stdout.contains("VALUE_MISMATCH"));
}

#[test]
fn parse_errors_return_two_without_echoing_configuration_values() {
    let dir = TempDir::new();
    let invalid = dir.write("invalid.json", r#"{"TOKEN":"PARSE_SECRET_SENTINEL",}"#);
    let target = dir.write("target.json", "{}");
    let output = Command::new(env!("CARGO_BIN_EXE_envsentry"))
        .arg("compare")
        .arg(&invalid)
        .arg(target)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("invalid JSON configuration"));
    assert!(!stderr.contains("PARSE_SECRET_SENTINEL"));
}
