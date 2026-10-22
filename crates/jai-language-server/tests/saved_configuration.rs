//! Saved settings stay active through unfinished and discarded edits.
use jai_language_server::{
    DiagnosticCode, DiagnosticSeverity, DocumentUri, JsonSession, Limits, Session, TextChange,
};
use serde_json::{Value, json};
mod common;

fn unused(s: &Session, uri: &DocumentUri) -> Option<DiagnosticSeverity> {
    s.diagnostics(uri)
        .unwrap()
        .into_iter()
        .find(|d| d.code == DiagnosticCode::Lint("unused_variable"))
        .map(|d| d.severity)
}

#[test]
fn valid_changes_wait_for_save_and_unsaved_edits_are_discarded() {
    let dir = common::project(
        "saved-config-close",
        &[
            ("main.jai", "main :: () {x:=1;}"),
            ("jacket.toml", "[lint.rules]\nunused_variable=\"deny\"\n"),
        ],
    );
    let mut s = common::session();
    s.set_workspace_folders(vec![dir.clone()]);
    let config = common::uri_of(&dir.join("jacket.toml"));
    let main = common::uri_of(&dir.join("main.jai"));
    s.open(
        config.clone(),
        1,
        "[lint.rules]\nunused_variable=\"allow\"\n".into(),
    )
    .unwrap();
    s.open(main.clone(), 1, "main :: () {x:=1;}".into())
        .unwrap();
    assert_eq!(
        unused(&s, &main),
        Some(DiagnosticSeverity::Error),
        "opening an unsaved buffer must retain disk settings"
    );
    s.save(&config).unwrap();
    assert_eq!(unused(&s, &main), None);
    // Model the disk write done by the editor when saving, then begin another edit.
    std::fs::write(
        dir.join("jacket.toml"),
        "[lint.rules]\nunused_variable=\"allow\"\n",
    )
    .unwrap();
    s.change(
        &config,
        2,
        &[TextChange {
            range: None,
            range_length: None,
            text: "[lint.rules]\nunused_variable=\"deny\"\n".into(),
        }],
    )
    .unwrap();
    assert_eq!(unused(&s, &main), None);
    s.close(&config).unwrap();
    assert_eq!(unused(&s, &main), None);
    std::fs::remove_dir_all(dir).unwrap();
}

fn send(s: &mut JsonSession, value: Value) -> Vec<Value> {
    s.handle_json(&value.to_string())
        .unwrap()
        .iter()
        .map(|t| serde_json::from_str(t).unwrap())
        .collect()
}

fn warnings(messages: &[Value]) -> usize {
    messages
        .iter()
        .filter(|m| m["method"] == "window/showMessage")
        .count()
}

#[test]
fn save_and_watcher_notifications_report_one_problem() {
    let dir = common::project(
        "saved-config-watcher",
        &[
            ("main.jai", "main :: () {}"),
            ("jacket.toml", "[project]\n"),
        ],
    );
    let mut s = JsonSession::with_environment(Limits::default(), common::environment());
    let init = send(
        &mut s,
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"rootUri":common::uri_of(&dir).as_str()}}),
    );
    assert_eq!(
        init[0]["result"]["capabilities"]["textDocumentSync"]["save"],
        true
    );
    let config = common::uri_of(&dir.join("jacket.toml"))
        .as_str()
        .to_string();
    let main = common::uri_of(&dir.join("main.jai")).as_str().to_string();
    for (uri, language, text) in [
        (&main, "jai", "main :: () {}"),
        (&config, "toml", "[project]\n"),
    ] {
        send(
            &mut s,
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":language,"version":1,"text":text}}}),
        );
    }
    let out = send(
        &mut s,
        json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":config,"version":2},"contentChanges":[{"text":"import_path ="}]}}),
    );
    assert_eq!(warnings(&out), 0);
    std::fs::write(dir.join("jacket.toml"), "import_path =").unwrap();
    let save = json!({"jsonrpc":"2.0","method":"textDocument/didSave","params":{"textDocument":{"uri":config}}});
    let watch = json!({"jsonrpc":"2.0","method":"workspace/didChangeWatchedFiles","params":{"changes":[{"uri":config,"type":2}]}});
    assert_eq!(warnings(&send(&mut s, save.clone())), 1);
    assert_eq!(warnings(&send(&mut s, watch.clone())), 0);
    assert_eq!(warnings(&send(&mut s, save)), 0);
    assert_eq!(warnings(&send(&mut s, watch)), 0);
    std::fs::remove_dir_all(dir).unwrap();
}
