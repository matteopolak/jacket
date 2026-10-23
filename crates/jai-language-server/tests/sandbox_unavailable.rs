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
