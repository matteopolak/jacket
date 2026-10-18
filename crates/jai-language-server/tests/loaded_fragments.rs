//! A file another module `#load`s is analysed as part of that module, so it sees the names the
//! module declares or imports, even when the loader lives in another folder.
use jai_language_server::{DiagnosticCode, DocumentUri, Position, Session};
use std::path::PathBuf;
mod common;
use common::{open_file, project};

fn check_errors(s: &Session, uri: &DocumentUri) -> Vec<String> {
    s.diagnostics(uri)
        .unwrap()
        .into_iter()
        .filter(|d| d.code == DiagnosticCode::Check)
        .map(|d| d.message)
        .collect()
}

const FRAGMENT: &str = "total :: () -> int {\n    p: Point;\n    p.x = LIMIT;\n    \
                        print(\"%\\n\", p.x);\n    return p.x;\n}\n";

fn tree(extra: &[(&'static str, &str)]) -> Vec<(&'static str, String)> {
    let mut files = vec![
        (
            "Shapes/module.jai",
            "#import \"Basic\";\nPoint :: struct { x: int; y: int; }\nLIMIT :: 7;\n\
             #load \"../internal/fragment.jai\";\n"
                .to_string(),
        ),
        ("internal/fragment.jai", FRAGMENT.to_string()),
    ];
    files.extend(extra.iter().map(|(p, t)| (*p, t.to_string())));
    files
}

fn make(name: &str, files: &[(&str, String)]) -> PathBuf {
    let borrowed: Vec<(&str, &str)> = files.iter().map(|(p, t)| (*p, t.as_str())).collect();
    project(name, &borrowed)
}

#[test]
fn fragment_sees_its_loaders_struct_constant_and_imports() {
    let folder = make("names", &tree(&[]));
    let (s, uri, text) = open_file(&folder, "internal/fragment.jai");
    assert_eq!(check_errors(&s, &uri), Vec::<String>::new());
    let byte = text.find("LIMIT").unwrap();
    let hover = s
        .hover(
            &uri,
            Position {
                line: 2,
                character: (byte - jaic::source::line_start(&text, byte)) as u32,
            },
        )
        .unwrap()
        .expect("hover")
        .contents
        .value;
    assert!(hover.contains('7'), "{hover}");
    let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn fragment_loaded_by_several_modules_gets_the_same_owner_each_time() {
    // `Zeta` loads the fragment too but declares nothing; `Shapes` sorts first among equals.
    let folder = make(
        "several",
        &tree(&[("Zeta/module.jai", "#load \"../internal/fragment.jai\";\n")]),
    );
    for _ in 0..2 {
        let (s, uri, _) = open_file(&folder, "internal/fragment.jai");
        assert_eq!(check_errors(&s, &uri), Vec::<String>::new());
    }
    let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn module_root_keeps_being_its_own_program() {
    let folder = make("root", &tree(&[]));
    let (s, uri, _) = open_file(&folder, "Shapes/module.jai");
    assert_eq!(check_errors(&s, &uri), Vec::<String>::new());
    let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn editing_a_loaders_loads_moves_the_fragments_owner() {
    let folder = make("edited-loads", &tree(&[]));
    let (mut s, fragment, _) = open_file(&folder, "internal/fragment.jai");
    assert_eq!(check_errors(&s, &fragment), Vec::<String>::new());
    let module_path = folder.join("Shapes/module.jai");
    let module = DocumentUri::from_path(&module_path).unwrap();
    let text = std::fs::read_to_string(&module_path).unwrap();
    s.open(module.clone(), 1, text.clone()).unwrap();
    let full = |text: &str| jai_language_server::TextChange {
        range: None,
        range_length: None,
        text: text.to_string(),
    };
    // Edits that leave the `#load`s alone keep the owner.
    s.change(&module, 2, &[full(&format!("// edited\n{text}"))])
        .unwrap();
    assert_eq!(check_errors(&s, &fragment), Vec::<String>::new());
    // Without its `#load`, `Shapes` no longer owns the fragment, which cannot see `LIMIT`.
    s.change(&module, 3, &[full(&text.replace("#load", "// #load"))])
        .unwrap();
    assert_ne!(check_errors(&s, &fragment), [] as [std::string::String; 0]);
    s.change(&module, 4, &[full(&text)]).unwrap();
    assert_eq!(check_errors(&s, &fragment), Vec::<String>::new());
    let _ = std::fs::remove_dir_all(&folder);
}
