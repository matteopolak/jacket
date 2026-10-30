//! Editor analyses wait for what a metaprogram or an `#insert` declares later, rather than
//! reporting the names as unknown, and treat a file as a module only when it declares
//! `#module_parameters`.
use jai_language_server::{DiagnosticSeverity, Environment, Limits, Session};
use std::rc::Rc;
use test_support::lsp::{file_uri, virtual_session};

fn errors(files: &[(&str, &str)], import_paths: &[&str], main: &str, text: &str) -> Vec<String> {
    let mut session = if main.starts_with("/waits/full") {
        // Inserting text runs the program's compile-time code, which needs the runtime.
        Session::with_environment(
            Limits::default(),
            Environment {
                fs: Rc::new(jaic::sema::NativeFs),
                options: Box::new(|_| jaic::sema::Options::host()),
                services: None,
            },
        )
    } else if files.is_empty() {
        test_support::lsp::lean_session()
    } else {
        virtual_session(files, import_paths, None)
    };
    let uri = file_uri(main);
    session.open(uri.clone(), 1, text.into()).unwrap();
    session
        .diagnostics(&uri)
        .unwrap()
        .into_iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .map(|d| format!("{} {:?}", d.message, d.code))
        .collect()
}

#[test]
fn a_body_waits_for_a_name_another_body_inserts() {
    let text = "Top :: #code();\n\
        uses :: () -> int {\n\
            g: Generated;\n\
            g.value = 5;\n\
            return g.value;\n\
        }\n\
        declares :: () {\n\
            #insert,scope(Top) \"Generated :: struct { value: int; }\";\n\
        }\n\
        main :: () {\n\
            declares();\n\
            x := uses();\n\
        }\n";
    let errors = errors(&[], &[], "/waits/full.jai", text);
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn a_name_nothing_declares_is_still_unknown() {
    let text = "uses :: () -> int {\n\
            g: Missing;\n\
            return 1;\n\
        }\n\
        main :: () { x := uses(); }\n";
    let errors = errors(&[], &[], "/waits/missing.jai", text);
    assert!(errors.iter().any(|e| e.contains("Missing")), "{errors:?}");
}

#[test]
fn a_comment_naming_module_parameters_does_not_make_a_module() {
    let module = "#module_parameters ()(VERBOSE := false);\n\
        Message :: struct { text: string; }\n\
        is_verbose :: () -> bool { return VERBOSE; }\n";
    let text = "// Program parameters (the second #module_parameters list) configure one instance.\n\
        Plain :: #import \"Program_Param\";\n\
        Configured :: #import \"Program_Param\"()(VERBOSE = true);\n\
        main :: () {\n\
            m: Plain.Message;\n\
            n: Configured.Message = m;\n\
            ok := Configured.is_verbose();\n\
        }\n";
    let errors = errors(
        &[("/params/modules/Program_Param/module.jai", module)],
        &["/params/modules"],
        "/params/main.jai",
        text,
    );
    assert!(errors.is_empty(), "{errors:?}");
}
