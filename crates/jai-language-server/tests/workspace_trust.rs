//! Workspace trust: an untrusted workspace's compile-time code (`#run`, build metaprograms,
//! computed `#insert`) is never run, everything else keeps working, and granting trust runs
//! it and brings its results.
use jai_language_server::{
    DiagnosticCode, DocumentUri, Environment, JsonSession, Limits, Position, Session,
};
use serde_json::{Value, json};
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
    let dir = std::env::temp_dir().join(format!("jailsp-trust-{name}-{}", std::process::id()));
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

fn position(text: &str, marker: &str) -> Position {
    let byte = text.find(marker).unwrap();
    let prefix = &text[..byte];
    Position {
        line: prefix.matches('\n').count() as u32,
        character: prefix.rsplit('\n').next().unwrap().len() as u32,
    }
}

fn hover_at(s: &Session, uri: &DocumentUri, text: &str, marker: &str) -> String {
    s.hover(uri, position(text, marker))
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

/// A `#run` whose result is a visible number, a `#run` that is a statement of its own, and
/// code that does not depend on either.
const RUNS: &str = "#import \"Basic\";\n\
    compute :: () -> s64 {\n    print(\"computing\\n\");\n    return 6 * 7;\n}\n\
    ANSWER :: #run compute();\n\
    #run print(\"at compile time\\n\");\n\
    main :: () {\n    local := ANSWER + 1;\n    print(\"%\\n\", local);\n}\n\
    broken :: () {\n    wrong: s64 = \"text\";\n}\n";

#[test]
fn untrusted_runs_are_unknown_not_wrong_and_trust_brings_their_results() {
    let dir = project("runs", &[("main.jai", RUNS)]);
    let (mut s, uri, text) = open(&dir, "main.jai");
    // Trusted (the default): the `#run` ran.
    assert_eq!(
        check_errors(&s, &uri),
        vec!["type mismatch: expected `s64`, found `string`".to_string()]
    );
    let answer = hover_at(&s, &uri, &text, "ANSWER + 1");
    assert!(answer.contains("ANSWER :: s64 = 42"), "{answer}");

    s.set_trusted(false);
    assert!(!s.trusted());
    // Nothing ran, and only the real error is reported: no error for `ANSWER`, `local`.
    let errors = check_errors(&s, &uri);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("mismatch") || errors[0].contains("text"),
        "{errors:?}"
    );
    let answer = hover_at(&s, &uri, &text, "ANSWER + 1");
    assert!(!answer.contains("= 42"), "{answer}");
    // Hover and go to definition on the code around the `#run` still work.
    let compute = hover_at(&s, &uri, &text, "compute();");
    assert!(compute.contains("compute"), "{compute}");
    let definition = s.definition(&uri, position(&text, "compute();")).unwrap();
    assert_eq!(definition.len(), 1);
    assert_eq!(definition[0].range.start.line, 1);

    s.set_trusted(true);
    let answer = hover_at(&s, &uri, &text, "ANSWER + 1");
    assert!(answer.contains("ANSWER :: s64 = 42"), "{answer}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn untrusted_computed_insert_reports_no_unknown_names() {
    let src = "#import \"Basic\";\n\
        CODE :: #run -> string { return \"GENERATED :: 7;\"; };\n\
        #insert CODE;\n\
        main :: () {\n    g := GENERATED;\n    print(\"%\\n\", g);\n}\n";
    let dir = project("insert", &[("main.jai", src)]);
    let (mut s, uri, _) = open(&dir, "main.jai");
    assert_eq!(check_errors(&s, &uri), Vec::<String>::new());
    s.set_trusted(false);
    assert_eq!(check_errors(&s, &uri), Vec::<String>::new());
    let _ = std::fs::remove_dir_all(&dir);
}

const SETUP: &str = "    w := compiler_create_workspace(\"app\");\n    \
    options := get_build_options(w);\n    options.output_type = .NO_OUTPUT;\n    \
    set_build_options(options, w);\n    add_build_file(\"main.jai\", w);\n    \
    add_build_string(\"extra_value :: 41;\", w);\n";

const BUILD: &str = "#import \"Basic\";\n#import \"Compiler\";\n#run build();\nbuild :: () {\n\
    set_build_options_dc(.{do_output = false});\n{SETUP}}\n";

#[test]
fn untrusted_build_metaprogram_does_not_run_and_the_files_it_names_still_check() {
    let main = "#import \"Basic\";\nmain :: () {\n    count := 1 + 2;\n    print(\"%\\n\", count);\n    bad: s64 = \"text\";\n}\n";
    let uses_built = "#import \"Basic\";\nmain :: () {\n    answer := extra_value + 1;\n    print(\"%\\n\", answer);\n}\n";
    let build = BUILD.replace("{SETUP}", SETUP);
    let dir = project("meta", &[("first.jai", &build), ("main.jai", main)]);
    let (mut s, uri, text) = open(&dir, "main.jai");
    s.set_trusted(false);
    // The program the metaprogram would build is not built, yet the file has its parse and
    // type errors, hover and definitions.
    let errors = check_errors(&s, &uri);
    assert_eq!(errors.len(), 1, "{errors:?}");
    let count = hover_at(&s, &uri, &text, "count :=");
    assert!(count.contains("count: s64"), "{count}");
    let build_uri = uri_of(&dir.join("first.jai"));
    s.open(build_uri.clone(), 1, build.clone()).unwrap();
    assert_eq!(check_errors(&s, &build_uri), Vec::<String>::new());
    let _ = std::fs::remove_dir_all(&dir);

    // A name only the metaprogram defines is unknown without it, and defined with it.
    let dir = project(
        "meta-built",
        &[("first.jai", &build), ("main.jai", uses_built)],
    );
    let (mut s, uri, _) = open(&dir, "main.jai");
    assert_eq!(check_errors(&s, &uri), Vec::<String>::new());
    s.set_trusted(false);
    let errors = check_errors(&s, &uri);
    assert!(
        errors.iter().any(|e| e.contains("extra_value")),
        "{errors:?}"
    );
    assert!(
        errors.iter().all(|e| e.starts_with("unknown identifier")),
        "{errors:?}"
    );
    s.set_trusted(true);
    assert_eq!(check_errors(&s, &uri), Vec::<String>::new());
    let _ = std::fs::remove_dir_all(&dir);
}

fn send(session: &mut JsonSession, message: Value) -> Vec<Value> {
    session
        .handle_json(&message.to_string())
        .unwrap()
        .iter()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect()
}

fn hover_json(session: &mut JsonSession, uri: &str, at: Position, id: u64) -> String {
    let out = send(
        session,
        json!({
            "jsonrpc": "2.0", "id": id, "method": "textDocument/hover",
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": at.line, "character": at.character },
            },
        }),
    );
    out[0]["result"]["contents"]["value"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn protocol_starts_untrusted_and_switches_at_runtime() {
    let dir = project("protocol", &[("main.jai", RUNS)]);
    let uri = uri_of(&dir.join("main.jai")).as_str().to_string();
    let at = position(RUNS, "ANSWER + 1");
    for switch in ["jai/setTrusted", "workspace/didChangeConfiguration"] {
        let mut s = JsonSession::with_environment(Limits::default(), environment());
        send(
            &mut s,
            json!({
                "jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": {
                    "rootUri": uri_of(&dir).as_str(),
                    "initializationOptions": { "trusted": false },
                },
            }),
        );
        send(
            &mut s,
            json!({
                "jsonrpc": "2.0", "method": "textDocument/didOpen",
                "params": { "textDocument": {
                    "uri": uri, "languageId": "jai", "version": 1, "text": RUNS,
                } },
            }),
        );
        assert!(!hover_json(&mut s, &uri, at, 2).contains("= 42"));
        let params = if switch == "jai/setTrusted" {
            json!({ "trusted": true })
        } else {
            json!({ "settings": { "jai": { "trusted": true } } })
        };
        let out = send(
            &mut s,
            json!({ "jsonrpc": "2.0", "method": switch, "params": params }),
        );
        // Diagnostics are published again for the open document.
        assert!(
            out.iter()
                .any(|m| m["method"] == "textDocument/publishDiagnostics"),
            "{switch}: {out:?}"
        );
        assert!(hover_json(&mut s, &uri, at, 3).contains("= 42"), "{switch}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn command_line_flag_is_documented() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_jailsp"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&out.stdout).contains("--untrusted"));
}
