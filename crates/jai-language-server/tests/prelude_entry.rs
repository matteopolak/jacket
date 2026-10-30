//! A root prelude wrapper must preserve relative loads and editor navigation.
use jai_language_server::{DiagnosticCode, Limits, Session};
use std::path::Path;
use test_support::TempDir;
use test_support::lsp::{modules_environment, position, uri_of};

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &to.join(entry.file_name()));
        } else {
            std::fs::copy(entry.path(), to.join(entry.file_name())).unwrap();
        }
    }
}

#[test]
fn root_prelude_preserves_diagnostics_hover_and_definition() {
    if jacket_config::env::PRELUDE_TEST_CHILD.os().is_some() {
        let root = std::path::PathBuf::from(jacket_config::env::STDLIB.os().unwrap());
        let text = "#import \"Basic\";\nmain :: () { print(\"hello\\n\"); }\n";
        let main = root.parent().unwrap().join("main.jai");
        std::fs::write(&main, text).unwrap();
        let mut session = Session::with_environment(Limits::default(), modules_environment(None));
        session.set_workspace_folders(vec![root.parent().unwrap().to_path_buf()]);
        let uri = uri_of(&main);
        session.open(uri.clone(), 1, text.into()).unwrap();
        let errors: Vec<_> = session
            .diagnostics(&uri)
            .unwrap()
            .into_iter()
            .filter(|d| d.code == DiagnosticCode::Check)
            .collect();
        assert!(errors.is_empty(), "{errors:?}");
        let at = position(text, text.find("print").unwrap() + 1);
        let hover = session.hover(&uri, at).unwrap().unwrap();
        assert!(hover.contents.value.contains("print"), "{hover:?}");
        let definitions = session.definition(&uri, at).unwrap();
        assert!(!definitions.is_empty());
        assert!(
            definitions
                .iter()
                .all(|d| d.uri.ends_with("/stdlib/internal/basic/scalar.jai")),
            "{definitions:?}"
        );
        let completion_text = "main :: () { Ba }";
        let completion_uri = uri_of(&root.parent().unwrap().join("completion.jai"));
        session
            .open(completion_uri.clone(), 1, completion_text.into())
            .unwrap();
        let list = session
            .completion(
                &completion_uri,
                position(completion_text, completion_text.find("Ba").unwrap() + 2),
            )
            .unwrap();
        assert!(
            list.items.iter().all(|item| item.label != "Base"),
            "{list:?}"
        );
        return;
    }
    let dir = TempDir::new("jai-prelude-lsp");
    let stdlib = dir.join("stdlib");
    copy_tree(&test_support::lsp::stdlib(), &stdlib);
    std::fs::rename(stdlib.join("prelude"), stdlib.join("core")).unwrap();
    std::fs::write(stdlib.join("Base.jai"), "#load \"core/module.jai\";\n").unwrap();
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "root_prelude_preserves_diagnostics_hover_and_definition",
            "--nocapture",
        ])
        .env("JAIC_PRELUDE_TEST_CHILD", "1")
        .env("JAIC_STDLIB", &stdlib)
        .env("JAIC_PRELUDE", "Base.jai")
        .status()
        .unwrap();
    assert!(status.success());
}
