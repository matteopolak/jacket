//! A file another module `#load`s is analysed as part of that module, so it sees the names the
//! module declares or imports, even when the loader lives in another folder.
use jai_language_server::{DiagnosticCode, DocumentUri, Environment, Limits, Position, Session};
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
    let dir = std::env::temp_dir().join(format!("jailsp-loaded-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (path, text) in files {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    dir.canonicalize().unwrap()
}

fn open(folder: &Path, file: &str) -> (Session, DocumentUri, String) {
    let mut s = Session::with_environment(Limits::default(), environment());
    s.set_workspace_folders(vec![folder.to_path_buf()]);
    let path = folder.join(file);
    let uri = DocumentUri::from_path(&path).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    s.open(uri.clone(), 1, text.clone()).unwrap();
    (s, uri, text)
}

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
    let (s, uri, text) = open(&folder, "internal/fragment.jai");
    assert_eq!(check_errors(&s, &uri), Vec::<String>::new());
    let byte = text.find("LIMIT").unwrap();
    let hover = s
        .hover(
            &uri,
            Position {
                line: 2,
                character: (byte - text[..byte].rfind('\n').map_or(0, |n| n + 1)) as u32,
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
        let (s, uri, _) = open(&folder, "internal/fragment.jai");
        assert_eq!(check_errors(&s, &uri), Vec::<String>::new());
    }
    let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn module_root_keeps_being_its_own_program() {
    let folder = make("root", &tree(&[]));
    let (s, uri, _) = open(&folder, "Shapes/module.jai");
    assert_eq!(check_errors(&s, &uri), Vec::<String>::new());
    let _ = std::fs::remove_dir_all(&folder);
}
