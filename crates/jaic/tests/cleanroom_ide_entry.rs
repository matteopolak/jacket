//! Passive IDE entry metadata must report captured paths without guessing.
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource, ide::IdeFacts};
use std::{path::PathBuf, rc::Rc};

fn compiler(editor: bool) -> Compiler {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    if editor {
        compiler.ide = Some(Box::new(IdeFacts::new(Vec::new())));
    }
    compiler
}

fn entry_file(name: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("compiler-ide-entry-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("entry space λ.jai");
    // Source preparation does not execute this failing assertion.
    std::fs::write(&path, "#assert false;").unwrap();
    path
}

#[test]
fn captured_file_entry_is_canonical_and_reading_it_does_not_settle_sources() {
    let path = entry_file("passive");
    let mut compiler = compiler(true);
    assert!(compiler.ide_selected_entry_path().is_none());
    compiler
        .prepare_sources(&[ProgramSource::File(path.clone())])
        .unwrap();
    let before = (
        compiler.sources.len(),
        compiler.procs.len(),
        compiler.files.len(),
    );
    assert_eq!(
        compiler.ide_selected_entry_path(),
        Some(path.canonicalize().unwrap().as_path())
    );
    assert_eq!(
        before,
        (
            compiler.sources.len(),
            compiler.procs.len(),
            compiler.files.len()
        )
    );
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[test]
fn generated_strings_and_later_added_files_do_not_invent_an_entry() {
    let path = entry_file("late");
    let mut compiler = compiler(true);
    compiler
        .prepare_sources(&[ProgramSource::String("value :: 7;".into())])
        .unwrap();
    assert!(compiler.ide_selected_entry_path().is_none());
    compiler
        .add_source(&ProgramSource::File(path.clone()))
        .unwrap();
    assert!(compiler.ide_selected_entry_path().is_none());
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[test]
fn a_non_editor_compile_has_no_captured_ide_entry() {
    let path = entry_file("plain");
    let mut compiler = compiler(false);
    compiler
        .prepare_sources(&[ProgramSource::File(path.clone())])
        .unwrap();
    assert!(compiler.ide_selected_entry_path().is_none());
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}
