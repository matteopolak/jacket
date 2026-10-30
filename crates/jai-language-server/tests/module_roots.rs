//! A module file opened on its own is checked as the module a program imports, not as the
//! program: `#module_parameters` only exist there.
use jai_language_server::{DiagnosticSeverity, Session};
use test_support::lsp::{file_uri, virtual_session};

const PARAMS: &str = "#module_parameters(WIDTH := 16);\n#load \"part.jai\";\nwidth :: WIDTH;\n";
const PART: &str = "twice :: () -> s64 { return WIDTH * 2; }\n";

fn session() -> Session {
    virtual_session(
        &[
            ("/m/modules/Extensions/SIMD/module.jai", PARAMS),
            ("/m/modules/Extensions/SIMD/part.jai", PART),
            (
                "/m/modules/Flat.jai",
                "#module_parameters(N := 1);\nflat :: N;\n",
            ),
            ("/m/modules/Plain/module.jai", "plain :: 1;\n"),
        ],
        &["/m/modules"],
        None,
    )
}

fn errors(s: &mut Session, path: &str, text: &str) -> Vec<String> {
    let uri = file_uri(path);
    s.open(uri.clone(), 1, text.into()).unwrap();
    s.diagnostics(&uri)
        .unwrap()
        .into_iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .map(|d| d.message)
        .collect()
}

#[test]
fn nested_module_with_parameters_has_no_program_error() {
    let mut s = session();
    let e = errors(&mut s, "/m/modules/Extensions/SIMD/module.jai", PARAMS);
    assert!(e.is_empty(), "{e:?}");
}

#[test]
fn file_a_module_loads_is_checked_through_the_module() {
    let mut s = session();
    let e = errors(&mut s, "/m/modules/Extensions/SIMD/part.jai", PART);
    assert!(e.is_empty(), "{e:?}");
}

#[test]
fn flat_module_file_with_parameters() {
    let mut s = session();
    let e = errors(
        &mut s,
        "/m/modules/Flat.jai",
        "#module_parameters(N := 1);\nflat :: N;\n",
    );
    assert!(e.is_empty(), "{e:?}");
}

#[test]
fn module_errors_still_map_to_the_real_file() {
    let mut s = session();
    let text = "#module_parameters(WIDTH := 16);\nbad :: nope;\n";
    let e = errors(&mut s, "/m/modules/Extensions/SIMD/module.jai", text);
    assert!(e.iter().any(|m| m.contains("nope")), "{e:?}");
    assert!(!e.iter().any(|m| m.contains("module_parameters")), "{e:?}");
}

#[test]
fn program_files_are_unaffected() {
    let mut s = session();
    let e = errors(&mut s, "/m/main.jai", "main :: () { x := 1; }\n");
    assert!(e.is_empty(), "{e:?}");
}
