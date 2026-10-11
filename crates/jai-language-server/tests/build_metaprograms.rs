//! Files a build metaprogram (`first.jai`) compiles with `compiler_create_workspace` are
//! checked and answered like any others, with or without a message loop.
use jai_language_server::{DiagnosticCode, DocumentUri, Environment, Limits, Position, Session};
use std::path::{Path, PathBuf};

fn environment() -> Environment {
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    Environment {
        fs: std::rc::Rc::new(jaic::sema::NativeFs),
        options: Box::new(move |_| {
            let mut options = jaic::sema::Options::host();
            options.import_paths = vec![stdlib.clone()];
            options.preload = Some(stdlib.join("prelude/module.jai"));
            options
        }),
    }
}

fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("jailsp-buildmeta-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (path, text) in files {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    dir.canonicalize().unwrap()
}

fn uri_of(path: &Path) -> DocumentUri {
    DocumentUri::parse(&format!("file://{}", path.display())).unwrap()
}

fn open(folder: &Path, file: &str) -> (Session, DocumentUri, String) {
    let mut s = Session::with_environment(Limits::default(), environment());
    s.set_workspace_folders(vec![folder.to_path_buf()]);
    let path = folder.join(file);
    let uri = uri_of(&path);
    let text = std::fs::read_to_string(&path).unwrap();
    s.open(uri.clone(), 1, text.clone()).unwrap();
    (s, uri, text)
}

fn hover_at(s: &Session, uri: &DocumentUri, text: &str, marker: &str) -> String {
    let byte = text.find(marker).unwrap();
    let prefix = &text[..byte];
    let position = Position {
        line: prefix.matches('\n').count() as u32,
        character: prefix.rsplit('\n').next().unwrap().len() as u32,
    };
    s.hover(uri, position)
        .unwrap()
        .expect("hover")
        .contents
        .value
}

fn check_errors(s: &Session, uri: &DocumentUri) -> Vec<String> {
    s.diagnostics(uri)
        .unwrap()
        .into_iter()
        .filter(|d| d.code == DiagnosticCode::Check)
        .map(|d| d.message)
        .collect()
}

const SETUP: &str = "    w := compiler_create_workspace(\"app\");\n    \
                     options := get_build_options(w);\n    options.output_type = .NO_OUTPUT;\n    \
                     set_build_options(options, w);\n    add_build_file(\"main.jai\", w);\n    \
                     add_build_string(\"extra_value :: 41;\", w);\n";

const MAIN: &str = "#import \"Basic\";\nmain :: () {\n    answer := extra_value + 1;\n    \
                    print(\"%\\n\", answer);\n}\n";

const WITH_LOOP: &str = "#import \"Basic\";\n#import \"Compiler\";\n#run build();\nbuild :: () {\n\
    set_build_options_dc(.{do_output = false});\n{SETUP}    compiler_begin_intercept(w);\n    \
    while true {\n        message := compiler_wait_for_message();\n        \
    if message.kind == .COMPLETE break;\n    }\n    compiler_end_intercept(w);\n}\n";

const WITHOUT_LOOP: &str = "#import \"Basic\";\n#import \"Compiler\";\n#run build();\nbuild :: () {\n\
    set_build_options_dc(.{do_output = false});\n{SETUP}}\n";

fn metaprogram(template: &str) -> String {
    template.replace("{SETUP}", SETUP)
}

fn assert_real_hover(name: &str, meta: &str) {
    let dir = project(name, &[("first.jai", meta), ("main.jai", MAIN)]);
    let (s, uri, text) = open(&dir, "main.jai");
    let hover = hover_at(&s, &uri, &text, "answer :=");
    assert!(hover.contains("answer: s64"), "{hover}");
    assert_eq!(check_errors(&s, &uri), Vec::<String>::new());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn file_built_with_an_intercept_loop_gets_a_checked_hover() {
    assert_real_hover("loop", &metaprogram(WITH_LOOP));
}

#[test]
fn file_built_without_a_message_loop_gets_a_checked_hover() {
    assert_real_hover("noloop", &metaprogram(WITHOUT_LOOP));
}

#[test]
fn built_file_without_main_reports_no_missing_entry() {
    let lib = "#import \"Basic\";\nhelper :: () -> int {\n    value := extra_value * 2;\n    return value;\n}\n";
    for (name, template) in [("nomain-loop", WITH_LOOP), ("nomain-plain", WITHOUT_LOOP)] {
        let dir = project(
            name,
            &[("first.jai", &metaprogram(template)), ("main.jai", lib)],
        );
        let (s, uri, text) = open(&dir, "main.jai");
        let errors = check_errors(&s, &uri);
        assert!(errors.is_empty(), "{name}: {errors:?}");
        let hover = hover_at(&s, &uri, &text, "value :=");
        assert!(hover.contains("value: s64"), "{name}: {hover}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// Without the metaprogram nothing defines `extra_value`: the tests above pass because the
/// built program was used.
#[test]
fn control_without_the_metaprogram_reports_the_missing_name() {
    let dir = project("control", &[("main.jai", MAIN)]);
    let (s, uri, _) = open(&dir, "main.jai");
    let errors = check_errors(&s, &uri);
    assert!(
        errors.iter().any(|e| e.contains("extra_value")),
        "{errors:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
