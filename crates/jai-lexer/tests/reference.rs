use std::{
    fs,
    path::{Path, PathBuf},
};
fn collect(path: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(path).unwrap() {
        let p = entry.unwrap().path();
        if p.is_dir() {
            collect(&p, out);
        } else if p.extension().is_some_and(|s| s == "jai") {
            out.push(p);
        }
    }
}
#[test]
fn lex_entire_reference_without_executing_it() {
    let reference = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../reference");
    let mut files = Vec::new();
    collect(&reference, &mut files);
    files.sort();
    assert_eq!(
        files.len(),
        702,
        "update the corpus manifest when the reference changes"
    );
    let mut failures = Vec::new();
    for p in files {
        let bytes = fs::read(&p).unwrap();
        let source =
            jai_lexer::decode_source(&bytes).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        if let Err(e) = jai_lexer::lex(&source) {
            failures.push(e.render(&p.display().to_string(), &source));
        }
    }
    assert!(
        failures.is_empty(),
        "lexical failures (not compile results):\n{}",
        failures.join("\n")
    );
}
