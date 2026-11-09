//! Typing the quoted code a compile-time procedure receives is a probe that never runs code, so
//! what it cannot decide is not an error of the file.
use jai_language_server::{DiagnosticCode, Session};
use test_support::lsp::{open_file, project};

const CALLER_CODE_MACRO: &str = "#import \"Basic\";\n\
#import \"Compiler\";\n\
\n\
build_text :: (code: Code) -> string {\n\
    assert(compiler_get_nodes(code) != null);\n\
    return \"\";\n\
}\n\
\n\
compile_here :: (loc := #caller_code) #expand {\n\
    #insert,scope(loc) #run build_text(loc);\n\
}\n\
\n\
main :: () {\n\
    compile_here();\n\
}\n";

fn check_errors(s: &Session, uri: &jai_language_server::DocumentUri) -> Vec<String> {
    s.diagnostics(uri)
        .unwrap()
        .into_iter()
        .filter(|d| d.code == DiagnosticCode::Check)
        .map(|d| d.message)
        .collect()
}

#[test]
fn a_macro_with_a_run_insert_called_as_caller_code_has_no_probe_error() {
    let dir = project("caller-code", &[("main.jai", CALLER_CODE_MACRO)]);
    let (s, uri, _) = open_file(&dir, "main.jai");
    assert_eq!(check_errors(&s, &uri), Vec::<String>::new());
}
