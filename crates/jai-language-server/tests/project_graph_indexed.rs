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
