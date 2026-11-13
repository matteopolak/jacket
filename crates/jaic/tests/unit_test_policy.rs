//! Tests inside `crates/jaic/src/` never use `test_support`. That crate links its own build of
//! `jaic`, so a unit test that uses it puts two copies of the interpreter's fiber routines in
//! one binary, and the Linux link fails. Such tests belong under `crates/jaic/tests/`.
use std::path::{Path, PathBuf};

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn unit_tests_in_jaic_do_not_use_test_support() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    sources(&src, &mut files);
    assert!(files.len() > 100, "found only {} sources", files.len());
    let mut found = Vec::new();
    for file in files {
        let text = std::fs::read_to_string(&file).unwrap();
        for (number, line) in text.lines().enumerate() {
            if !line.trim_start().starts_with("//") && line.contains("test_support") {
                found.push(format!(
                    "{}:{}",
                    file.strip_prefix(&src).unwrap().display(),
                    number + 1
                ));
            }
        }
    }
    assert!(
        found.is_empty(),
        "move these tests to crates/jaic/tests/ or drop test_support:\n{}",
        found.join("\n")
    );
}
