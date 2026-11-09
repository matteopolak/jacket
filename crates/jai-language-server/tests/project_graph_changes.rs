//! The program a file belongs to follows changes to the project's graph: `#load` lines added or
//! removed, files created, deleted or renamed on disk, a changed `jacket.toml`, and several
//! workspace folders. Each case compares against what a fresh session says.
use jai_language_server::{DiagnosticCode, DocumentUri, Session};
use std::path::Path;
use test_support::lsp::{open_file, project, uri_of, workspace_session};

fn check_errors(s: &Session, uri: &DocumentUri) -> Vec<String> {
    s.diagnostics(uri)
        .unwrap()
        .into_iter()
        .filter(|d| d.code == DiagnosticCode::Check)
        .map(|d| d.message)
        .collect()
}

/// Rewrite a file on disk and tell the session, as a client's file watcher would.
fn rewrite(s: &mut Session, path: &Path, text: &str) {
    let existed = path.exists();
    std::fs::write(path, text).unwrap();
    s.file_changed(path, !existed);
}

fn remove(s: &mut Session, path: &Path) {
    std::fs::remove_file(path).unwrap();
    s.file_changed(path, true);
}

const MAIN: &str = "#import \"Basic\";\n#load \"a.jai\";\nmain :: () { use_it(); }\n";
const A: &str = "use_it :: () { x := helper(); print(\"%\\n\", x); }\n";
const B: &str = "helper :: () -> int { return 1; }\n";

#[test]
fn a_load_added_on_disk_gives_the_fragment_its_new_siblings() {
    let dir = project(
        "disk-add-load",
        &[("main.jai", MAIN), ("a.jai", A), ("b.jai", B)],
    );
    let (mut s, a, _) = open_file(&dir, "a.jai");
    assert_ne!(
        check_errors(&s, &a),
        Vec::<String>::new(),
        "helper is unknown"
    );
    rewrite(
        &mut s,
        &dir.join("main.jai"),
        &MAIN.replace("#load \"a.jai\";", "#load \"a.jai\";\n#load \"b.jai\";"),
    );
    assert_eq!(check_errors(&s, &a), Vec::<String>::new());
}

#[test]
fn a_load_removed_on_disk_takes_the_siblings_away() {
    let main = "#import \"Basic\";\n#load \"a.jai\";\n#load \"b.jai\";\nmain :: () { use_it(); }\n";
    let dir = project(
        "disk-remove-load",
        &[("main.jai", main), ("a.jai", A), ("b.jai", B)],
    );
    let (mut s, a, _) = open_file(&dir, "a.jai");
    assert_eq!(check_errors(&s, &a), Vec::<String>::new());
    rewrite(&mut s, &dir.join("main.jai"), MAIN);
    assert_ne!(check_errors(&s, &a), Vec::<String>::new());
}

#[test]
fn a_loaded_file_created_later_joins_the_program() {
    let main = "#import \"Basic\";\n#load \"a.jai\";\n#load \"b.jai\";\nmain :: () { use_it(); }\n";
    let dir = project("create-loaded", &[("main.jai", main), ("a.jai", A)]);
    let (mut s, a, _) = open_file(&dir, "a.jai");
    assert_ne!(check_errors(&s, &a), Vec::<String>::new());
    rewrite(&mut s, &dir.join("b.jai"), B);
    assert_eq!(check_errors(&s, &a), Vec::<String>::new());
}

#[test]
fn a_loaded_file_deleted_leaves_the_program() {
    let main = "#import \"Basic\";\n#load \"a.jai\";\n#load \"b.jai\";\nmain :: () { use_it(); }\n";
    let dir = project(
        "delete-loaded",
        &[("main.jai", main), ("a.jai", A), ("b.jai", B)],
    );
    let (mut s, a, _) = open_file(&dir, "a.jai");
    assert_eq!(check_errors(&s, &a), Vec::<String>::new());
    remove(&mut s, &dir.join("b.jai"));
    assert_ne!(check_errors(&s, &a), Vec::<String>::new());
}

#[test]
fn a_renamed_loaded_file_follows_the_load_line() {
    let main = "#import \"Basic\";\n#load \"a.jai\";\n#load \"b.jai\";\nmain :: () { use_it(); }\n";
    let dir = project(
        "rename-loaded",
        &[("main.jai", main), ("a.jai", A), ("b.jai", B)],
    );
    let (mut s, a, _) = open_file(&dir, "a.jai");
    assert_eq!(check_errors(&s, &a), Vec::<String>::new());
    // Rename `b.jai` to `c.jai`, then fix the load.
    remove(&mut s, &dir.join("b.jai"));
    rewrite(&mut s, &dir.join("c.jai"), B);
    assert_ne!(check_errors(&s, &a), Vec::<String>::new(), "b.jai is gone");
    rewrite(
        &mut s,
        &dir.join("main.jai"),
        &main.replace("b.jai", "c.jai"),
    );
    assert_eq!(check_errors(&s, &a), Vec::<String>::new());
}

#[test]
fn a_changed_entry_in_jacket_toml_moves_the_fragments_program() {
    let first = "#import \"Basic\";\n#load \"a.jai\";\nmain :: () { use_it(); }\n";
    let second =
        "#import \"Basic\";\n#load \"a.jai\";\n#load \"b.jai\";\nmain :: () { use_it(); }\n";
    let dir = project(
        "toml-entry",
        &[
            ("jacket.toml", "[project]\nentry = [\"first.jai\"]\n"),
            ("first.jai", first),
            ("second.jai", second),
            ("a.jai", A),
            ("b.jai", B),
        ],
    );
    let (mut s, a, _) = open_file(&dir, "a.jai");
    assert_ne!(
        check_errors(&s, &a),
        Vec::<String>::new(),
        "first.jai has no helper"
    );
    let toml = dir.join("jacket.toml");
    std::fs::write(&toml, "[project]\nentry = [\"second.jai\"]\n").unwrap();
    s.file_changed(&toml, false);
    assert_eq!(check_errors(&s, &a), Vec::<String>::new());
}

#[test]
fn each_workspace_folder_keeps_its_own_entries() {
    let one = project(
        "root-one",
        &[("main.jai", MAIN), ("a.jai", A), ("b.jai", B)],
    );
    let other_main =
        "#import \"Basic\";\n#load \"a.jai\";\n#load \"b.jai\";\nmain :: () { use_it(); }\n";
    let two = project(
        "root-two",
        &[("main.jai", other_main), ("a.jai", A), ("b.jai", B)],
    );
    let mut s = workspace_session(&one);
    s.set_workspace_folders(vec![one.path().to_path_buf(), two.path().to_path_buf()]);
    for (folder, clean) in [(&one, false), (&two, true)] {
        let path = folder.join("a.jai");
        let uri = uri_of(&path);
        s.open(uri.clone(), 1, std::fs::read_to_string(&path).unwrap())
            .unwrap();
        let errors = check_errors(&s, &uri);
        assert_eq!(errors.is_empty(), clean, "{}: {errors:?}", folder.display());
    }
}
