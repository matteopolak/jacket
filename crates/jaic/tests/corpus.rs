//! Corpus sweep: `JAIC_CORPUS_ROOT=<repo> JAIC_CORPUS_STAGE=lex|parse cargo test -p jaic --test corpus -- --ignored --nocapture`.
//! Walks public upstream projects and standard-library sources and reports failures.
use std::path::{Path, PathBuf};

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.extension().is_some_and(|e| e == "jai") {
            out.push(path);
        }
    }
}

#[test]
#[ignore]
fn corpus_sweep() {
    // `JAIC_CORPUS_ROOT` points the sweep at another checkout (the corpora are gitignored).
    let root = std::env::var_os("JAIC_CORPUS_ROOT").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
        PathBuf::from,
    );
    let mut files = Vec::new();
    for dir in ["corpus/upstream", "stdlib"] {
        collect(&root.join(dir), &mut files);
    }
    files.sort();
    let stage = std::env::var("JAIC_CORPUS_STAGE").unwrap_or_else(|_| "parse".into());
    let mut sources = jaic::source::SourceMap::default();
    let mut failures = Vec::new();
    for path in &files {
        let bytes = std::fs::read(path).unwrap();
        let text: std::rc::Rc<str> = String::from_utf8_lossy(&bytes).into();
        let id = sources.add(path.display().to_string(), text.clone());
        let result = if stage == "lex" {
            jaic::lexer::lex(id, &text).map(|_| ())
        } else {
            jaic::parser::parse_file(id, &text).map(|_| ())
        };
        if let Err(diagnostic) = result {
            failures.push(diagnostic.render(&sources));
        }
    }
    for failure in &failures {
        eprint!("{failure}");
    }
    eprintln!(
        "{stage}: {} / {} files ok",
        files.len() - failures.len(),
        files.len()
    );
}
