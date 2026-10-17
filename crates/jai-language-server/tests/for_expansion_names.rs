//! The name after `for :` in `for :utf8_iter LYRICS { ... }` is a `for_expansion` macro: it
//! navigates, hovers, is found by references and renamed, and is highlighted like a macro.
use jai_language_server::{DocumentUri, Environment, Limits, Position, SemanticTokenKind, Session};
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
    let dir = std::env::temp_dir().join(format!("jailsp-forexp-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (path, text) in files {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    dir.canonicalize().unwrap()
}

fn open(dir: &Path, file: &str) -> (Session, DocumentUri, String) {
    let mut s = Session::with_environment(Limits::default(), environment());
    s.set_workspace_folders(vec![dir.to_path_buf()]);
    let path = dir.join(file);
    let text = std::fs::read_to_string(&path).unwrap();
    let uri = DocumentUri::from_path(&path).unwrap();
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

const ITER: &str = "utf8_iter :: (s: string, body: Code, flags: For_Flags) #expand {\n    \
    for i: 0..s.count - 1 {\n        `it := s[i];\n        `it_index := i;\n        #insert body;\n    }\n}\n";

fn same_file(text: &str) -> String {
    format!(
        "#import \"Basic\";\n{ITER}LYRICS :: \"abc\";\nmain :: () {{\n    \
         for :utf8_iter LYRICS {{\n        print(\"%\\n\", it);\n    }}\n    \
         for :utf8_iter LYRICS {{ print(\"%\\n\", it_index); }}\n}}\n{text}"
    )
}

fn check(dir: &Path, file: &str, name_marker: &str, decl_file: &str) {
    let (s, uri, text) = open(dir, file);
    let diagnostics = s.diagnostics(&uri).unwrap();
    assert!(
        diagnostics
            .iter()
            .all(|d| d.severity != jai_language_server::DiagnosticSeverity::Error),
        "{diagnostics:?}"
    );
    // The name follows `for :`.
    let skip = 5;
    let on = at(&text, name_marker, skip + 2);
    let found = s.definition(&uri, on).unwrap();
    assert!(
        found.len() == 1 && found[0].uri.ends_with(decl_file),
        "definition: {found:?}"
    );
    let hover = s.hover(&uri, on).unwrap().expect("hover").contents.value;
    assert!(hover.contains("utf8_iter"), "{hover}");
    let refs = s.references(&uri, on, true).unwrap();
    let in_use = refs.iter().filter(|l| l.uri == uri.as_str()).count();
    let declared_here = usize::from(decl_file == file);
    assert_eq!(in_use, 2 + declared_here, "uses in this file: {refs:?}");
    assert!(
        refs.iter().any(|l| l.uri.ends_with(decl_file)),
        "declaration among references: {refs:?}"
    );
    // Rename reaches the declaration and both uses.
    let prepared = s.prepare_rename(&uri, on).unwrap();
    assert!(prepared.is_some(), "prepare_rename");
    let edits = s.rename(&uri, on, "chars_of").unwrap().expect("rename");
    let total: usize = edits.iter().map(|(_, e)| e.len()).sum();
    assert_eq!(total, 3, "{edits:?}");
    // Semantic tokens: a function with the macro flag.
    let tokens = s.semantic_tokens(&uri).unwrap();
    let want = at(&text, name_marker, skip);
    let token = tokens
        .iter()
        .find(|t| t.position == want)
        .unwrap_or_else(|| panic!("no token at {want:?}: {tokens:?}"));
    assert_eq!(token.kind, SemanticTokenKind::Function, "{token:?}");
    assert!(token.expand, "{token:?}");
}

#[test]
fn custom_iterator_in_the_same_file() {
    let main = same_file("");
    let dir = project("same", &[("main.jai", &main)]);
    check(&dir, "main.jai", "for :utf8_iter LYRICS {\n", "main.jai");
    let _ = std::fs::remove_dir_all(&dir);
}

fn use_site(import: &str, call: &str) -> String {
    format!(
        "#import \"Basic\";\n{import}\nLYRICS :: \"abc\";\nmain :: () {{\n    \
         for :{call} LYRICS {{\n        print(\"%\\n\", it);\n    }}\n    \
         for :{call} LYRICS {{ print(\"%\\n\", it_index); }}\n}}\n"
    )
}

#[test]
fn custom_iterator_imported_from_a_module() {
    let dir = project(
        "imported",
        &[
            ("main.jai", &use_site("#import \"Iters\";", "utf8_iter")),
            ("modules/Iters/module.jai", ITER),
        ],
    );
    check(
        &dir,
        "main.jai",
        "for :utf8_iter LYRICS {\n",
        "modules/Iters/module.jai",
    );
    let _ = std::fs::remove_dir_all(&dir);
}
