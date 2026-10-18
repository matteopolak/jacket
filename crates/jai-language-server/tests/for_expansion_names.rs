//! The name after `for :` in `for :utf8_iter LYRICS { ... }` is a `for_expansion` macro: it
//! navigates, hovers, is found by references and renamed, and is highlighted like a macro.
use jai_language_server::SemanticTokenKind;
use std::path::Path;
mod common;
use common::{at_offset, open_file, project};

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
    let (s, uri, text) = open_file(dir, file);
    let diagnostics = s.diagnostics(&uri).unwrap();
    assert!(
        diagnostics
            .iter()
            .all(|d| d.severity != jai_language_server::DiagnosticSeverity::Error),
        "{diagnostics:?}"
    );
    // The name follows `for :`.
    let skip = 5;
    let on = at_offset(&text, name_marker, skip + 2);
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
    let want = at_offset(&text, name_marker, skip);
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
