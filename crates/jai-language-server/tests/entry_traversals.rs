//! Working out the workspace's entry programs costs a walk over the open documents' projects:
//! it is done once per change of what it reads, not once per message, per progress check or
//! per request. These are call counts, not timings.
use jai_language_server::{JsonSession, Limits};
use serde_json::{Value, json};
use test_support::lsp::{environment, project, uri_of};

fn send(json: &mut JsonSession, message: &Value) {
    json.handle_json(&message.to_string()).unwrap();
}

fn settle(json: &mut JsonSession) {
    for _ in 0..200 {
        if !json.has_background_work() {
            return;
        }
        json.background_work().unwrap();
    }
    panic!("indexing did not finish");
}

const MAIN: &str = "#import \"Basic\";\nmain :: () { print(\"hi\\n\"); }\n";

fn started(files: &[(&str, &str)]) -> (JsonSession, test_support::TempDir) {
    let dir = project("entry-traversals", files);
    let mut json = JsonSession::with_environment(Limits::default(), environment());
    send(
        &mut json,
        &json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "capabilities": {}, "workspaceFolders": [{"uri": uri_of(dir.path()).as_str(), "name": "w"}]}}),
    );
    send(
        &mut json,
        &json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}),
    );
    (json, dir)
}

#[test]
fn idle_checks_do_not_walk_the_entries_again() {
    let (mut json, dir) = started(&[
        ("jacket.toml", "[project]\nentry = [\"a.jai\", \"b.jai\"]\n"),
        ("a.jai", MAIN),
        ("b.jai", MAIN),
    ]);
    settle(&mut json);
    let settled = json.session().entry_scans();
    // Whole indexing, four entries worth of steps: a handful of walks, not one per check.
    assert!(settled <= 8, "{settled} walks to index two entries");
    for _ in 0..50 {
        assert!(!json.indexing_pending());
        assert!(!json.has_background_work());
    }
    assert_eq!(json.session().entry_scans(), settled);
    // A change on disk is one more walk, however often progress is asked about it.
    let a = dir.join("a.jai");
    std::fs::write(&a, format!("{MAIN}// edited\n")).unwrap();
    send(
        &mut json,
        &json!({"jsonrpc": "2.0", "method": "workspace/didChangeWatchedFiles", "params": {
            "changes": [{"uri": uri_of(&a).as_str(), "type": 2}]}}),
    );
    for _ in 0..50 {
        let _ = json.indexing_pending();
    }
    assert!(
        json.session().entry_scans() <= settled + 1,
        "{} walks after one change",
        json.session().entry_scans() - settled
    );
}
