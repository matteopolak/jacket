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
