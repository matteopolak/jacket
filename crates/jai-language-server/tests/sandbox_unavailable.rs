//! Compile-time code that calls a foreign procedure the editor's sandbox does not provide
//! (a build runs it for real): its result is unknown, not wrong, so no error is shown and the
//! code after it keeps its facts.
use jai_language_server::DiagnosticCode;
mod common;
use common::{open_file, project};

const SOURCE: &str = "#import \"Basic\";\n\
    #import \"POSIX\";\n\
    make_dir :: () -> string {\n    template := copy_string(\"/tmp/jacket-XXXXXX\");\n    \
    if !mkdtemp(template.data) return \"\";\n    return template;\n}\n\
    #run {\n    dir := make_dir();\n    print(\"%\\n\", dir);\n}\n\
    DIR :: #run make_dir();\n\
    main :: () {\n    copy := DIR;\n    print(\"%\\n\", copy);\n}\n\
    broken :: () {\n    wrong: s64 = \"text\";\n}\n";

#[test]
fn sandbox_unavailable_calls_are_unknown_not_errors() {
    let dir = project("sandbox-unavailable", &[("main.jai", SOURCE)]);
    let (s, uri, _) = open_file(&dir, "main.jai");
    let errors: Vec<String> = s
        .diagnostics(&uri)
        .unwrap()
        .into_iter()
        .filter(|d| d.code == DiagnosticCode::Check)
        .map(|d| d.message)
        .collect();
    assert_eq!(
        errors,
        vec!["type mismatch: expected `s64`, found `string`".to_string()],
        "{errors:?}"
    );
}

/// The reported scenario: `build_cpp_dynamic_lib` run at compile time creates its scratch
/// directory with `mkdtemp`, which the editor sandbox does not provide.
const BUILD_CPP: &str = "#import \"Basic\";\n\
    #import \"BuildCpp\";\n\
    #import \"Compiler\";\n\
    gen :: () -> bool {\n    extra: [..] string;\n    array_add(*extra, \"-fPIC\");\n    \
    return build_cpp_dynamic_lib(\"test_lib\", \"test_lib.cpp\", debug = true, extra = extra);\n}\n\
    #run {\n    set_build_options_dc(.{ do_output = false });\n    if !gen() compiler_set_workspace_status(.FAILED);\n}\n\
    main :: () {\n    success := build_cpp_dynamic_lib(\"test_lib\", \"test_lib.cpp\", debug = true);\n    print(\"%\\n\", success);\n}\n\
    broken :: () {\n    wrong: s64 = \"text\";\n}\n";

#[test]
fn build_cpp_at_compile_time_is_not_an_error() {
    let dir = project("sandbox-build-cpp", &[("main.jai", BUILD_CPP)]);
    let (s, uri, _) = open_file(&dir, "main.jai");
    let all = s.diagnostics(&uri).unwrap();
    let messages: Vec<String> = all.iter().map(|d| d.message.clone()).collect();
    assert!(
        !messages
            .iter()
            .any(|m| m.contains("not available here") || m.contains("compile-time execution")),
        "{messages:?}"
    );
    // The check still runs past the unavailable call.
    assert!(
        messages
            .iter()
            .any(|m| m == "type mismatch: expected `s64`, found `string`"),
        "{messages:?}"
    );
}
