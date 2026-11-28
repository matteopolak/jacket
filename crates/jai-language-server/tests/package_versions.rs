//! A package that needs another version of a shared dependency than the program's shared copy
//! resolves its imports to the copy `jacket build` gives it.
use jai_language_server::DiagnosticCode;
use test_support::lsp::{open_file, project};

const TOML: &str = r#"[project]
entry = ["main.jai"]

[dependencies]
c = { git = "x/c", tag = "v1.0.0" }
foo = { git = "x/foo", tag = "v1.4.0", shared = true }
"foo@2" = { git = "x/foo", tag = "v2.0.0", shared = true, path = "modules/.versions/foo@2/foo" }
"#;

fn errors(main: &str) -> Vec<String> {
    let dir = project(
        "package-versions",
        &[
            ("jacket.toml", TOML),
            ("main.jai", main),
            (
                "modules/c/jacket.toml",
                "[package]\nname = \"c\"\n[dependencies]\nfoo = { git = \"x/foo\", version = \"^2\" }\n",
            ),
            (
                "modules/c/module.jai",
                "#import \"foo\";\nc_two :: () -> s64 { return only_in_two(); }\n",
            ),
            (
                "modules/foo/module.jai",
                "foo_id :: () -> s64 { return 1; }\n",
            ),
            (
                "modules/.versions/foo@2/foo/module.jai",
                "foo_id :: () -> s64 { return 2; }\nonly_in_two :: () -> s64 { return 2; }\n",
            ),
        ],
    );
    let (s, uri, _) = open_file(&dir, "main.jai");

    s.diagnostics(&uri)
        .unwrap()
        .into_iter()
        .filter(|d| d.code == DiagnosticCode::Check)
        .map(|d| d.message)
        .collect()
}

#[test]
fn a_package_imports_the_copy_its_range_admits() {
    let found = errors("#import \"c\";\nmain :: () { c_two(); }\n");

    assert_eq!(found, Vec::<String>::new());
}

#[test]
fn the_program_itself_still_gets_the_shared_copy() {
    let found = errors("#import \"foo\";\nmain :: () { only_in_two(); }\n");

    assert!(!found.is_empty(), "the shared copy is version 1");
}
