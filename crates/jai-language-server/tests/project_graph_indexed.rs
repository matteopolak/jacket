//! Closed-file diagnostics from background indexing follow changes to the project's graph: a
//! `#load` added on disk, files created or deleted, and a changed `jacket.toml` entry.
use jai_language_server::{JsonSession, Limits};
use serde_json::{Value, json};
use std::path::Path;
use test_support::lsp::{environment, project, uri_of};

const MAIN: &str = "#import \"Basic\";\n#load \"a.jai\";\nmain :: () { use_it(); }\n";

const BOTH: &str =
    "#import \"Basic\";\n#load \"a.jai\";\n#load \"b.jai\";\nmain :: () { use_it(); }\n";

const A: &str = "use_it :: () { x := helper(); print(\"%\\n\", x); }\n";
const B: &str = "helper :: () -> int { return 1; }\n";

fn send(json: &mut JsonSession, message: &Value) -> Vec<Value> {
    json.handle_json(&message.to_string())
        .unwrap()
        .iter()
        .map(|m| serde_json::from_str(m).unwrap())
        .collect()
}

fn start(folder: &Path) -> JsonSession {
    let mut json = JsonSession::with_environment(Limits::default(), environment());
    send(
        &mut json,
        &json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "capabilities": {"textDocument": {"diagnostic": {}}},
            "workspaceFolders": [{"uri": uri_of(folder).as_str(), "name": "w"}]}}),
    );
    send(
        &mut json,
        &json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}),
    );
    json
}

fn settle(json: &mut JsonSession) {
    for _ in 0..100 {
        if !json.has_background_work() {
            return;
        }
        json.background_work().unwrap();
    }
    panic!("indexing did not finish");
}

/// The error messages the workspace pull reports for `file`; `None` when it reports no entry.
fn closed_errors(json: &mut JsonSession, file: &Path) -> Option<Vec<String>> {
    let reply = send(
        json,
        &json!({"jsonrpc": "2.0", "id": 2, "method": "workspace/diagnostic",
            "params": {"previousResultIds": []}}),
    );
    let uri = uri_of(file);
    reply[0]["result"]["items"]
        .as_array()?
        .iter()
        .find(|item| item["uri"] == uri.as_str())
        .map(|item| {
            item["items"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|d| d["severity"] == 1)
                .map(|d| d["message"].as_str().unwrap().to_string())
                .collect()
        })
}

fn watched(json: &mut JsonSession, file: &Path, kind: u32) {
    send(
        json,
        &json!({"jsonrpc": "2.0", "method": "workspace/didChangeWatchedFiles", "params": {
            "changes": [{"uri": uri_of(file).as_str(), "type": kind}]}}),
    );
}

#[test]
fn a_load_added_on_disk_reaches_closed_file_diagnostics() {
    let dir = project(
        "indexed-add-load",
        &[("main.jai", MAIN), ("a.jai", A), ("b.jai", B)],
    );
    let a = dir.join("a.jai");
    let mut json = start(&dir);
    settle(&mut json);
    let before = closed_errors(&mut json, &a).expect("a.jai is part of the program");
    assert!(!before.is_empty(), "helper is unknown: {before:?}");
    let main = dir.join("main.jai");
    std::fs::write(&main, BOTH).unwrap();
    watched(&mut json, &main, 2);
    settle(&mut json);
    assert_eq!(closed_errors(&mut json, &a), Some(Vec::new()));
}

#[test]
fn a_file_created_then_deleted_reaches_closed_file_diagnostics() {
    let dir = project("indexed-create", &[("main.jai", BOTH), ("a.jai", A)]);
    let a = dir.join("a.jai");
    let mut json = start(&dir);
    settle(&mut json);
    assert_ne!(closed_errors(&mut json, &a), Some(Vec::new()));
    let b = dir.join("b.jai");
    std::fs::write(&b, B).unwrap();
    watched(&mut json, &b, 1);
    settle(&mut json);
    assert_eq!(closed_errors(&mut json, &a), Some(Vec::new()));
    std::fs::remove_file(&b).unwrap();
    watched(&mut json, &b, 3);
    settle(&mut json);
    assert_ne!(closed_errors(&mut json, &a), Some(Vec::new()));
}

#[test]
fn a_changed_toml_entry_reaches_closed_file_diagnostics() {
    let dir = project(
        "indexed-toml",
        &[
            ("jacket.toml", "[project]\nentry = [\"first.jai\"]\n"),
            ("first.jai", MAIN),
            ("second.jai", BOTH),
            ("a.jai", A),
            ("b.jai", B),
        ],
    );
    let a = dir.join("a.jai");
    let mut json = start(&dir);
    settle(&mut json);
    assert_ne!(closed_errors(&mut json, &a), Some(Vec::new()));
    let toml = dir.join("jacket.toml");
    std::fs::write(&toml, "[project]\nentry = [\"second.jai\"]\n").unwrap();
    watched(&mut json, &toml, 2);
    settle(&mut json);
    assert_eq!(closed_errors(&mut json, &a), Some(Vec::new()));
}

#[test]
fn a_file_loaded_by_two_entries_reports_errors_from_either_program() {
    // `x.jai` is fine under `one.jai`, which defines `helper`, and broken under `two.jai`.
    let dir = project(
        "indexed-two-entries",
        &[
            (
                "jacket.toml",
                "[project]\nentry = [\"one.jai\", \"two.jai\"]\n",
            ),
            (
                "one.jai",
                "#import \"Basic\";\nhelper :: () -> int { return 1; }\n#load \"x.jai\";\nmain :: () { use_it(); }\n",
            ),
            (
                "two.jai",
                "#import \"Basic\";\n#load \"x.jai\";\nmain :: () { use_it(); }\n",
            ),
            ("x.jai", A),
        ],
    );
    let x = dir.join("x.jai");
    let mut json = start(&dir);
    settle(&mut json);
    let errors = closed_errors(&mut json, &x).expect("x.jai is part of both programs");
    assert!(!errors.is_empty(), "two.jai has no helper: {errors:?}");
}

fn open_and_pull(json: &mut JsonSession, file: &Path, text: &str) -> Value {
    send(
        json,
        &json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {
            "textDocument": {"uri": uri_of(file).as_str(), "languageId": "jai",
                "version": 1, "text": text}}}),
    );
    let mut reply = send(
        json,
        &json!({"jsonrpc": "2.0", "id": 9, "method": "textDocument/diagnostic",
            "params": {"textDocument": {"uri": uri_of(file).as_str()}}}),
    );
    reply.remove(0)["result"].clone()
}

/// A file indexed while closed, then opened with the same text, reports what a server that
/// opened it from the start reports: the warning about a `#load` nothing can read included.
#[test]
fn opening_an_indexed_file_keeps_its_unreadable_load_warning() {
    let main = "#import \"Basic\";\n#load \"gone.jai\";\nmain :: () {}\n";
    let dir = project(
        "indexed-open-warning",
        &[("main.jai", main), ("gone.jai", "gone :: () {}\n")],
    );
    let path = dir.join("main.jai");
    let mut json = start(&dir);
    settle(&mut json);
    let gone = dir.join("gone.jai");
    std::fs::remove_file(&gone).unwrap();
    watched(&mut json, &gone, 3);
    settle(&mut json);
    let opened = open_and_pull(&mut json, &path, main);
    let mut cold = start(&dir);
    let fresh = open_and_pull(&mut cold, &path, main);
    settle(&mut cold);
    assert_eq!(opened["items"], fresh["items"]);
}

fn error_messages(report: &Value) -> Vec<String> {
    report["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["severity"] == 1)
        .map(|d| d["message"].as_str().unwrap().to_string())
        .collect()
}

fn pull(json: &mut JsonSession, file: &Path) -> Value {
    let mut reply = send(
        json,
        &json!({"jsonrpc": "2.0", "id": 10, "method": "textDocument/diagnostic",
            "params": {"textDocument": {"uri": uri_of(file).as_str()}}}),
    );
    reply.remove(0)["result"].clone()
}

/// An open file that two entries load is checked under both, also when the indexed snapshot
/// does not cover it (before indexing, or after an edit): what only the second program finds
/// wrong still shows, and what both find wrong shows once.
#[test]
fn an_open_file_loaded_by_two_entries_reports_errors_from_either_program() {
    let dir = project(
        "open-two-entries",
        &[
            (
                "jacket.toml",
                "[project]\nentry = [\"one.jai\", \"two.jai\"]\n",
            ),
            (
                "one.jai",
                "#import \"Basic\";\nhelper :: () -> int { return 1; }\n#load \"x.jai\";\nmain :: () { use_it(); }\n",
            ),
            (
                "two.jai",
                "#import \"Basic\";\n#load \"x.jai\";\nmain :: () { use_it(); }\n",
            ),
            ("x.jai", A),
        ],
    );
    let x = dir.join("x.jai");
    let mut json = start(&dir);
    let opened = open_and_pull(&mut json, &x, A);
    assert_eq!(error_messages(&opened).len(), 1, "{opened:?}");
    settle(&mut json);
    send(
        &mut json,
        &json!({"jsonrpc": "2.0", "method": "textDocument/didChange", "params": {
            "textDocument": {"uri": uri_of(&x).as_str(), "version": 2},
            "contentChanges": [{"text": format!("{A}\n")}]}}),
    );
    let edited = pull(&mut json, &x);
    assert_eq!(error_messages(&edited).len(), 1, "{edited:?}");
}

/// `(severity, code)` of everything the workspace pull reports for `file`.
fn closed_codes(json: &mut JsonSession, file: &Path) -> Option<Vec<(u64, String)>> {
    let reply = send(
        json,
        &json!({"jsonrpc": "2.0", "id": 3, "method": "workspace/diagnostic",
            "params": {"previousResultIds": []}}),
    );
    let uri = uri_of(file);
    reply[0]["result"]["items"]
        .as_array()?
        .iter()
        .find(|item| item["uri"] == uri.as_str())
        .map(|item| {
            item["items"]
                .as_array()
                .unwrap()
                .iter()
                .map(|d| {
                    (
                        d["severity"].as_u64().unwrap(),
                        d["code"].as_str().unwrap().to_string(),
                    )
                })
                .collect()
        })
}

fn is_noted(codes: &[(u64, String)]) -> bool {
    codes.contains(&(3, "not-in-program".to_string()))
}

fn open_codes(json: &mut JsonSession, file: &Path, text: &str) -> Vec<(u64, String)> {
    open_and_pull(json, file, text)["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| {
            (
                d["severity"].as_u64().unwrap(),
                d["code"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

/// A file no entry reaches is said to be in no program, and is not checked as one: the names
/// it takes from elsewhere are not reported as unknown.
#[test]
fn a_closed_file_no_entry_reaches_is_noted_without_false_errors() {
    let loose = "uses_elsewhere :: () { x := helper(); print(\"%\\n\", x); }\n";
    let dir = project(
        "unreached-closed",
        &[
            ("main.jai", MAIN),
            ("a.jai", A),
            ("b.jai", B),
            ("loose.jai", loose),
        ],
    );
    let mut json = start(&dir);
    settle(&mut json);
    let codes = closed_codes(&mut json, &dir.join("loose.jai")).expect("loose.jai is reported");
    assert_eq!(codes, vec![(3, "not-in-program".to_string())]);
    let reached = closed_codes(&mut json, &dir.join("a.jai")).expect("a.jai is reported");
    assert!(!is_noted(&reached), "{reached:?}");
}

#[test]
fn a_closed_file_no_entry_reaches_still_reports_its_syntax_errors() {
    let dir = project(
        "unreached-syntax",
        &[
            ("main.jai", MAIN),
            ("a.jai", A),
            ("b.jai", B),
            ("loose.jai", "oops :: (\n"),
        ],
    );
    let mut json = start(&dir);
    settle(&mut json);
    let codes = closed_codes(&mut json, &dir.join("loose.jai")).unwrap();
    assert!(is_noted(&codes), "{codes:?}");
    assert!(
        codes.iter().any(|(severity, _)| *severity == 1),
        "{codes:?}"
    );
}

#[test]
fn a_load_added_to_an_entry_takes_the_note_away() {
    let dir = project(
        "unreached-load-added",
        &[("main.jai", MAIN), ("a.jai", A), ("b.jai", B)],
    );
    let mut json = start(&dir);
    settle(&mut json);
    let before = closed_codes(&mut json, &dir.join("b.jai")).unwrap();
    assert!(is_noted(&before), "{before:?}");
    let main = dir.join("main.jai");
    std::fs::write(&main, BOTH).unwrap();
    watched(&mut json, &main, 2);
    settle(&mut json);
    let after = closed_codes(&mut json, &dir.join("b.jai")).unwrap();
    assert!(!is_noted(&after), "{after:?}");
}

#[test]
fn an_open_file_no_entry_reaches_is_noted() {
    let dir = project(
        "unreached-open",
        &[
            ("main.jai", MAIN),
            ("a.jai", A),
            ("b.jai", B),
            ("loose.jai", B),
        ],
    );
    let mut json = start(&dir);
    settle(&mut json);
    let loose = open_codes(&mut json, &dir.join("loose.jai"), B);
    assert_eq!(loose, vec![(3, "not-in-program".to_string())]);
    // Names it takes from elsewhere are not reported as unknown, as when it is closed.
    let uses = "uses_elsewhere :: () { x := helper(); print(\"%\\n\", x); }\n";
    let loose = open_codes(&mut json, &dir.join("scratch.jai"), uses);
    assert_eq!(loose, vec![(3, "not-in-program".to_string())]);
    let unloaded = open_codes(&mut json, &dir.join("b.jai"), B);
    assert!(is_noted(&unloaded), "b.jai is not loaded by main.jai");
    let reached = open_codes(&mut json, &dir.join("a.jai"), A);
    assert!(!is_noted(&reached), "{reached:?}");
}

/// What an entry imports from the project's modules or by path is part of its program, though
/// no `#load` names it.
#[test]
fn imported_files_are_part_of_the_program() {
    let main =
        "#import \"Basic\";\n#import \"Util\";\n#import,file \"extra.jai\";\nmain :: () {}\n";
    let dir = project(
        "unreached-imports",
        &[
            ("main.jai", main),
            ("modules/Util/module.jai", "#load \"part.jai\";\n"),
            ("modules/Util/part.jai", B),
            ("extra.jai", "extra :: () {}\n"),
            ("loose.jai", B),
        ],
    );
    let mut json = start(&dir);
    settle(&mut json);
    for reached in [
        "modules/Util/module.jai",
        "modules/Util/part.jai",
        "extra.jai",
    ] {
        let codes = closed_codes(&mut json, &dir.join(reached)).unwrap_or_default();
        assert!(!is_noted(&codes), "{reached}: {codes:?}");
    }
    let codes = closed_codes(&mut json, &dir.join("loose.jai")).unwrap();
    assert!(is_noted(&codes), "{codes:?}");
}

/// Without an entry there is no program to be left out of.
#[test]
fn a_workspace_without_entries_notes_nothing() {
    let dir = project("unreached-none", &[("a.jai", A), ("b.jai", B)]);
    let mut json = start(&dir);
    settle(&mut json);
    let opened = open_codes(&mut json, &dir.join("b.jai"), B);
    assert!(!is_noted(&opened), "{opened:?}");
}
