//! This optional corpus is downloaded as text; none of its programs run.
use std::{fs, path::Path};
fn walk(path: &Path, failures: &mut Vec<String>, count: &mut usize) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let kind = entry.file_type().unwrap();
        if kind.is_dir() {
            walk(&path, failures, count);
        } else if kind.is_file() && path.extension().is_some_and(|e| e == "jai") {
            *count += 1;
            let bytes = fs::read(&path).unwrap();
            match jai_lexer::decode_source(&bytes) {
                Ok(source) => {
                    if let Err(e) = jai_lexer::lex(&source) {
                        failures.push(e.render(&path.to_string_lossy(), &source));
                    }
                }
                Err(e) => failures.push(format!("{}: {e}", path.display())),
            }
        }
    }
}
#[test]
#[ignore = "fetch pinned upstream source with tools/fetch_upstreams.py first"]
fn lex_recent_upstream_sources() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/upstream");
    let mut failures = Vec::new();
    let mut count = 0;
    walk(&root, &mut failures, &mut count);
    assert!(count > 0);
    assert!(
        failures.is_empty(),
        "{} lexical failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
