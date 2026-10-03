//! Check the actual CLI boundary and ensure rejected input cannot write output.
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "jai-cli-test-{}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn source(&self, text: &str) -> PathBuf {
        let p = self.0.join("input.jai");
        fs::write(&p, text).unwrap();
        p
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jai-rs"))
}
#[test]
fn check_performs_semantic_resolution() {
    let dir = Scratch::new();
    let source = dir.source("main :: () { unknown(); }");
    let result = cli().arg("check").arg(source).output().unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("unknown name 'unknown'"));
}
#[test]
fn failed_semantics_preserves_existing_output() {
    let dir = Scratch::new();
    let source = dir.source("main :: () { x: int = true; }");
    let output = dir.0.join("output.ll");
    fs::write(&output, b"preserved").unwrap();
    let result = cli()
        .arg("emit-llvm")
        .arg(source)
        .arg(&output)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert_eq!(fs::read(output).unwrap(), b"preserved");
}
#[test]
fn supplied_executable_is_rejected_before_backend_execution() {
    let dir = Scratch::new();
    let source = dir.source("main :: () {}");
    let output = dir.0.join("program");
    let result = cli()
        .env("JAI_RS_CLANG", reference)
        .arg("build")
        .arg(source)
        .arg(&output)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(!output.exists());
    assert!(String::from_utf8_lossy(&result.stderr).contains("refusing to execute reference tool"));
}
