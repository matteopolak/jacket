//! `jaic -doc_comments` and `-doc_gaps` find imported modules through `-import_dir`.
use std::{path::PathBuf, process::Command};
use test_support::TempDir;

fn stdlib() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib")
}

/// A library whose `make` forwards to `Shapes`, a module that lives in another folder.
fn library(tag: &str) -> TempDir {
    let dir = TempDir::new(tag);
    dir.write(
        "lib/module.jai",
        "// A module.\n\nShapes :: #import \"Shapes\";\n\nmake :: Shapes.make;\n",
    );
    dir.write("more/Shapes/module.jai", "// Makes one.\nmake :: () {}\n");
    dir
}

fn run(dir: &TempDir, flag: &str, import_dir: bool) -> String {
    let mut command = Command::new(env!("CARGO_BIN_EXE_jaic"));
    command.env("JAIC_STDLIB", stdlib());
    if import_dir {
        command.arg("-import_dir").arg(dir.join("more"));
    }
    let output = command.arg(flag).arg(dir.join("lib")).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn doc_comments_follow_an_alias_into_an_import_dir() {
    let dir = library("doc-comments-import-dir");
    assert!(!run(&dir, "-doc_comments", false).contains("Makes one."));
    assert!(run(&dir, "-doc_comments", true).contains("Makes one."));
}

#[test]
fn doc_gaps_follow_an_alias_into_an_import_dir() {
    let dir = library("doc-gaps-import-dir");
    // `file<TAB>line<TAB>kind<TAB>symbol<TAB>doc<TAB>example`
    let documented = |output: String| {
        let row = output
            .lines()
            .find(|row| row.split('\t').nth(3) == Some("make"))
            .unwrap()
            .to_owned();
        row.split('\t').nth(4).unwrap().to_owned()
    };
    assert_ne!(documented(run(&dir, "-doc_gaps", false)), "1");
    assert_eq!(documented(run(&dir, "-doc_gaps", true)), "1");
}
