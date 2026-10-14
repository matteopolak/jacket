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
            options.preload = Some(stdlib.join(jaic::PRELOAD_ENTRY));
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

#[test]
fn file_of_a_project_module_is_checked_through_the_program_importing_it() {
    let folder = project(
        "module",
        &[
            (
                "main.jai",
                "#import \"Basic\";\n#import \"Grid\"(SIZE = 3);\nmain :: () { print(\"%\\n\", cells); }\n",
            ),
            (
                "modules/Grid/module.jai",
                "#module_parameters(SIZE: int);\n#load \"cells.jai\";\n",
            ),
            ("modules/Grid/cells.jai", "cells :: SIZE * 2;\n"),
        ],
    );
    let (s, uri, text) = open(&folder, "modules/Grid/cells.jai");
    assert!(hover_at(&s, &uri, &text, "cells ::").contains('6'));
    assert_eq!(check_errors(&s, &uri), Vec::<String>::new());
}

#[test]
fn stray_files_do_not_push_out_the_projects_compile() {
    let folder = project(
        "stray",
        &[
            (
                "main.jai",
                "#import \"Basic\";\nmain :: () { total := 1; print(\"%\\n\", total); }\n",
            ),
            ("scratch/one.jai", "one :: 1;\n"),
            ("scratch/two.jai", "two :: 2;\n"),
        ],
    );
    let (mut s, main, main_text) = open(&folder, "main.jai");
    let mut strays = Vec::new();
    for name in ["scratch/one.jai", "scratch/two.jai"] {
        let path = folder.join(name);
        let text = std::fs::read_to_string(&path).unwrap();
        s.open(uri_of(&path), 1, text.clone()).unwrap();
        strays.push((uri_of(&path), text));
    }
    hover_at(&s, &main, &main_text, "total :=");
    for (uri, text) in &strays {
        hover_at(&s, uri, text, &text[..3]);
    }
    let (compiled, _) = s.compile_counts();
    hover_at(&s, &main, &main_text, "total :=");
    assert_eq!(
        s.compile_counts().0,
        compiled,
        "the project's compile was kept"
    );
}

#[test]
fn polymorphs_nothing_calls_keep_their_compile_time_constants_after_the_release() {
    let folder = project(
        "released",
        &[(
            "main.jai",
            "#import \"Basic\";\nhelper :: () -> int { return 5; }\nVALUE :: #run helper();\n\
             later :: (x: $T) -> T {\n    k :: #run helper();\n    scaled := x * k;\n    return scaled;\n}\n\
             main :: () { print(\"%\\n\", VALUE); }\n",
        )],
    );
    let (s, uri, text) = open(&folder, "main.jai");
    assert!(hover_at(&s, &uri, &text, "VALUE ::").contains('5'));
    // `later` is never called: its sketch ran `helper` before the compile released it.
    assert!(hover_at(&s, &uri, &text, "scaled :=").contains("scaled"));
    assert!(hover_at(&s, &uri, &text, "VALUE ::").contains('5'));
}

#[test]
fn constants_built_in_freed_compile_time_memory_still_hover() {
    let folder = project(
        "heap",
        &[(
            "main.jai",
            "#import \"Basic\";\nmake :: () -> string { return copy_string(\"hello\"); }\n\
             NAME :: #run make();\nmain :: () { print(\"%\\n\", NAME); }\n",
        )],
    );
    let (s, uri, text) = open(&folder, "main.jai");
    // The string `make` allocated is freed with the compile; the constant keeps its own copy.
    assert!(hover_at(&s, &uri, &text, "NAME ::").contains("hello"));
    assert!(hover_at(&s, &uri, &text, "NAME);").contains("hello"));
}
