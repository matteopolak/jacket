//! A sweep of `#import` / `#load` handling in a real project folder: navigation and hover on
//! the strings, module-name completion, `#import,file` / `#import,dir`, `../` loads, module
//! folders from `jai.toml`, `Name.jai` against `Name/module.jai`, paths with spaces, and a
//! missing module's diagnostic and quick fix.
use jai_language_server::{
    DiagnosticCode, DiagnosticSeverity, DocumentUri, Environment, Limits, Position, Session,
};
use std::path::{Path, PathBuf};

fn environment() -> Environment {
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    Environment {
        fs: std::rc::Rc::new(jaic::sema::NativeFs),
        options: Box::new(move |_| {
            let mut options = jaic::sema::Options::host();
            options.import_paths = vec![stdlib.clone()];
            options.preload = Some(stdlib.join(jaic::PRELOAD_ENTRY));
            options
        }),
    }
}

fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("jailsp-imports-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (path, text) in files {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    dir.canonicalize().unwrap()
}

fn uri_of(path: &Path) -> DocumentUri {
    DocumentUri::from_path(path).unwrap()
}

fn open(dir: &Path, file: &str) -> (Session, DocumentUri, String) {
    let mut s = Session::with_environment(Limits::default(), environment());
    s.set_workspace_folders(vec![dir.to_path_buf()]);
    let path = dir.join(file);
    let text = std::fs::read_to_string(&path).unwrap();
    let uri = uri_of(&path);
    s.open(uri.clone(), 1, text.clone()).unwrap();
    (s, uri, text)
}

fn at(text: &str, marker: &str, offset: usize) -> Position {
    let byte = text.find(marker).unwrap_or_else(|| panic!("no {marker:?}")) + offset;
    let prefix = &text[..byte];
    Position {
        line: prefix.matches('\n').count() as u32,
        character: prefix.rsplit('\n').next().unwrap().len() as u32,
    }
}

fn errors(s: &Session, uri: &DocumentUri) -> Vec<String> {
    s.diagnostics(uri)
        .unwrap()
        .into_iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .map(|d| d.message)
        .collect()
}

fn goes_to(s: &Session, uri: &DocumentUri, text: &str, marker: &str, suffix: &str) {
    let found = s.definition(uri, at(text, marker, 2)).unwrap();
    assert!(
        found.len() == 1 && found[0].uri.ends_with(suffix),
        "{marker}: {found:?} (wanted {suffix}); {:?}",
        s.diagnostics(uri)
    );
}

#[test]
fn imports_and_loads_resolve_through_every_kind_of_path() {
    let main = "#import \"Basic\";\n#import \"Single\";\n#import \"Folder\";\n#import \"Vendored\";\n\
        #import,file \"lib/rel.jai\";\n#import,dir \"dirmod\";\n#load \"with space/spaced.jai\";\n\
        #import \"Spaced Dir\";\n\
        main :: () { print(\"%\\n\", single_value + folder_value + vendored_value + rel_value + dir_value + spaced_value + spaced_dir_value); }\n";
    let dir = project(
        "kinds",
        &[
            ("jai.toml", "import_path = [\"vendor\"]\n"),
            ("src/main.jai", main),
            ("src/modules/Single.jai", "single_value :: 1;\n"),
            ("src/modules/Folder/module.jai", "folder_value :: 2;\n"),
            ("vendor/Vendored.jai", "vendored_value :: 3;\n"),
            ("src/lib/rel.jai", "rel_value :: 4;\n"),
            ("src/dirmod/module.jai", "dir_value :: 5;\n"),
            ("src/with space/spaced.jai", "spaced_value :: 6;\n"),
            (
                "src/modules/Spaced Dir/module.jai",
                "spaced_dir_value :: 7;\n",
            ),
        ],
    );
    let (s, uri, text) = open(&dir, "src/main.jai");
    assert_eq!(errors(&s, &uri), Vec::<String>::new());
    for (marker, suffix) in [
        ("\"Single\"", "/src/modules/Single.jai"),
        ("\"Folder\"", "/src/modules/Folder/module.jai"),
        ("\"Vendored\"", "/vendor/Vendored.jai"),
        ("\"lib/rel.jai\"", "/src/lib/rel.jai"),
        ("\"dirmod\"", "/src/dirmod/module.jai"),
        ("\"with space/spaced.jai\"", "/src/with%20space/spaced.jai"),
        ("\"Spaced Dir\"", "/src/modules/Spaced%20Dir/module.jai"),
    ] {
        goes_to(&s, &uri, &text, marker, suffix);
    }
    let links = s.document_links(&uri).unwrap();
    assert_eq!(links.len(), 8, "{links:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn load_with_parent_directory_segments() {
    let main = "#import \"Basic\";\n#load \"../shared/util.jai\";\nmain :: () { print(\"%\\n\", util_value); }\n";
    let dir = project(
        "parent",
        &[
            ("app/main.jai", main),
            (
                "shared/util.jai",
                "#load \"more.jai\";\nutil_value :: more_value + 1;\n",
            ),
            ("shared/more.jai", "more_value :: 1;\n"),
        ],
    );
    let (s, uri, text) = open(&dir, "app/main.jai");
    assert_eq!(errors(&s, &uri), Vec::<String>::new());
    assert_eq!(s.diagnostics(&uri).unwrap(), vec![]);
    goes_to(
        &s,
        &uri,
        &text,
        "\"../shared/util.jai\"",
        "/shared/util.jai",
    );
    // The loaded file is checked as part of the program that loads it.
    let (s2, uri2, text2) = open(&dir, "shared/util.jai");
    assert_eq!(errors(&s2, &uri2), Vec::<String>::new());
    goes_to(&s2, &uri2, &text2, "\"more.jai\"", "/shared/more.jai");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn hover_on_a_module_string_names_the_file() {
    let main =
        "#import \"Basic\";\n#import \"Single\";\nmain :: () { print(\"%\\n\", single_value); }\n";
    let dir = project(
        "hover",
        &[
            ("main.jai", main),
            ("modules/Single.jai", "single_value :: 1;\n"),
        ],
    );
    let (s, uri, text) = open(&dir, "main.jai");
    let hover = s.hover(&uri, at(&text, "\"Single\"", 3)).unwrap();
    let hover = hover.expect("hover on the module string").contents.value;
    assert!(hover.contains("modules/Single.jai"), "{hover}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn completion_inside_an_import_string_lists_modules() {
    let main = "#import \"Basic\";\n#import \"\";\nmain :: () { }\n";
    let dir = project(
        "complete",
        &[
            ("main.jai", main),
            ("modules/Single.jai", "single_value :: 1;\n"),
            ("modules/Folder/module.jai", "folder_value :: 2;\n"),
        ],
    );
    let (s, uri, text) = open(&dir, "main.jai");
    let list = s.completion(&uri, at(&text, "\"\";", 1)).unwrap();
    let labels: Vec<_> = list.items.iter().map(|i| i.label.as_str()).collect();
    for want in ["Single", "Folder", "Basic", "String"] {
        assert!(labels.contains(&want), "{want} missing from {labels:?}");
    }
    // Completing a partial name inside the string filters and replaces the word.
    let partial = "#import \"Basic\";\n#import \"Sing\";\nmain :: () { }\n";
    let (s, uri, _) = {
        let mut s = Session::with_environment(Limits::default(), environment());
        s.set_workspace_folders(vec![dir.clone()]);
        let uri = uri_of(&dir.join("main.jai"));
        s.open(uri.clone(), 2, partial.into()).unwrap();
        (s, uri, ())
    };
    let list = s.completion(&uri, at(partial, "Sing", 4)).unwrap();
    assert!(list.items.iter().any(|i| i.label == "Single"), "{list:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn missing_module_is_reported_with_a_quick_fix_to_the_closest_name() {
    let main = "#import \"Basic\";\n#import \"Singel\";\nmain :: () { }\n";
    let dir = project(
        "missing",
        &[
            ("main.jai", main),
            ("modules/Single.jai", "single_value :: 1;\n"),
        ],
    );
    let (s, uri, text) = open(&dir, "main.jai");
    let diagnostics = s.diagnostics(&uri).unwrap();
    let missing = diagnostics
        .iter()
        .find(|d| d.message.contains("module `Singel` not found"))
        .unwrap_or_else(|| panic!("{diagnostics:?}"));
    assert_eq!(missing.code, DiagnosticCode::Check);
    let actions = s.code_actions(&uri, at_range(&text, "Singel")).unwrap();
    let fix = actions
        .iter()
        .find(|a| a.kind == Some("quickfix") && a.title.contains("Single"))
        .unwrap_or_else(|| panic!("{actions:?}"));
    let (_, edits) = fix.edit.as_ref().unwrap();
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].new_text, "Single");
    assert_eq!(edits[0].range.start, at(&text, "Singel", 0));
    assert_eq!(edits[0].range.end, at(&text, "Singel", 6));
    let _ = std::fs::remove_dir_all(&dir);
}

fn at_range(text: &str, marker: &str) -> jai_language_server::Range {
    let p = at(text, marker, 1);
    jai_language_server::Range {
        start: p,
        end: p,
    }
}
