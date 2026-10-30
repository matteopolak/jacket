//! Passive IDE entry metadata must report captured paths without guessing.
use jaic::ide::IdeFacts;
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use std::{path::PathBuf, rc::Rc};
use test_support::TempDir;

fn compiler(editor: bool) -> Compiler {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    if editor {
        compiler.ed.record(IdeFacts::new(Vec::new()));
    }
    compiler
}

/// A scratch directory holding the entry file, and the file's path.
fn entry_file(name: &str) -> (TempDir, PathBuf) {
    let root = TempDir::new(&format!("compiler-ide-entry-{name}"));
    // Source preparation does not execute this failing assertion.
    let path = root.write("entry space λ.jai", "#assert false;");
    (root, path)
}

#[test]
fn captured_file_entry_is_canonical_and_reading_it_does_not_settle_sources() {
    let (_dir, path) = entry_file("passive");
    let mut compiler = compiler(true);
    assert!(compiler.ed.selected_entry().is_none());
    compiler
        .prepare_sources(&[ProgramSource::File(path.clone())])
        .unwrap();
    let before = (
        compiler.src.sources.len(),
        compiler.pr.procs.len(),
        compiler.src.files.len(),
    );
    assert_eq!(
        compiler.ed.selected_entry(),
        Some(path.canonicalize().unwrap().as_path())
    );
    assert_eq!(
        before,
        (
            compiler.src.sources.len(),
            compiler.pr.procs.len(),
            compiler.src.files.len()
        )
    );
}

#[test]
fn generated_strings_and_later_added_files_do_not_invent_an_entry() {
    let (_dir, path) = entry_file("late");
    let mut compiler = compiler(true);
    compiler
        .prepare_sources(&[ProgramSource::String("value :: 7;".into())])
        .unwrap();
    assert!(compiler.ed.selected_entry().is_none());
    compiler
        .add_source(&ProgramSource::File(path.clone()))
        .unwrap();
    assert!(compiler.ed.selected_entry().is_none());
}

#[test]
fn a_non_editor_compile_has_no_captured_ide_entry() {
    let (_dir, path) = entry_file("plain");
    let mut compiler = compiler(false);
    compiler
        .prepare_sources(&[ProgramSource::File(path.clone())])
        .unwrap();
    assert!(compiler.ed.selected_entry().is_none());
}
