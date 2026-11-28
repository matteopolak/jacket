//! The lifetime lints know which procedures only compile-time code runs, as the command line
//! does, although the editor's compile releases the lowered code once it is checked.
use jai_language_server::DiagnosticCode;
use test_support::lsp::{open_file, project};

/// `make` returns a boxed value that points at its own frame; only `#run` calls it, and a
/// compile-time result is copied out before the frame is reused.
const RUN_ONLY: &str = "Settings :: struct { boxed: Any; }\n\
\n\
make :: () -> Settings {\n\
    s: Settings;\n\
    s.boxed = 42;\n\
    return s;\n\
}\n\
\n\
VALUE :: #run make();\n\
\n\
main :: () {\n\
}\n";

/// The same procedure, called by the program.
const CALLED: &str = "Settings :: struct { boxed: Any; }\n\
\n\
make :: () -> Settings {\n\
    s: Settings;\n\
    s.boxed = 42;\n\
    return s;\n\
}\n\
\n\
main :: () {\n\
    value := make();\n\
    _ = value;\n\
}\n";

fn lints(name: &str, source: &str) -> Vec<String> {
    let dir = project(name, &[("main.jai", source)]);
    let (s, uri, _) = open_file(&dir, "main.jai");

    s.diagnostics(&uri)
        .unwrap()
        .into_iter()
        .filter(|d| matches!(d.code, DiagnosticCode::Lint(_)))
        .map(|d| d.message)
        .collect()
}

#[test]
fn a_procedure_only_a_run_calls_may_return_its_frozen_result() {
    assert_eq!(lints("run-only", RUN_ONLY), Vec::<String>::new());
}

#[test]
fn a_procedure_the_program_calls_still_reports_the_returned_address() {
    let found = lints("called", CALLED);

    assert!(
        found.iter().any(|m| m.contains("returns the address")),
        "{found:?}"
    );
}

/// The open files of one program are linted in one pass: the lifetime summaries of the program
/// are built once, not once per file.
#[test]
fn the_open_files_of_a_program_share_one_lint_pass() {
    let main = format!("#load \"other.jai\";\n{CALLED}");
    let other =
        "helper :: () -> Settings {\n    s: Settings;\n    s.boxed = 1;\n    return s;\n}\n";
    let dir = project("shared-pass", &[("main.jai", &main), ("other.jai", other)]);
    let (mut s, uri, _) = open_file(&dir, "main.jai");
    let (other_uri, _) = test_support::lsp::open_from_disk(&mut s, &dir, "other.jai");
    let before = jailint::summaries_built();
    let published = s.publications();

    assert_eq!(published.len(), 2);
    assert_eq!(jailint::summaries_built() - before, 1);
    let found: Vec<_> = s
        .diagnostics(&other_uri)
        .unwrap()
        .into_iter()
        .filter(|d| matches!(d.code, DiagnosticCode::Lint(_)))
        .map(|d| d.message)
        .collect();

    assert!(
        found.iter().any(|m| m.contains("returns the address")),
        "{found:?}"
    );
    assert!(s.diagnostics(&uri).is_ok());
}
