//! A diagnostics pull whose check did not finish is cancelled for the client to retry, never
//! answered with an empty report that would wipe the squiggles it shows. The interrupt state
//! is process-wide, so every test here takes `LOCK`.
use jai_language_server::{JsonSession, Limits};
use serde_json::{Value, json};
use std::sync::Mutex;
use test_support::lsp::{environment, file_uri};

static LOCK: Mutex<()> = Mutex::new(());

const SOURCE: &str = "\
helper :: (a: int) -> int { return a + 1; }
other :: (b: int) -> int { return helper(b) * 2; }
main :: () {
    bad: int = \"text\";
    total := other(helper(2));
}
";

fn send(json: &mut JsonSession, message: &Value) -> Vec<Value> {
    json.handle_json(&message.to_string())
        .unwrap()
        .iter()
        .map(|m| serde_json::from_str(m).unwrap())
        .collect()
}

fn session(with_environment: bool) -> (JsonSession, String) {
    let mut json = if with_environment {
        JsonSession::with_environment(Limits::default(), environment())
    } else {
        JsonSession::new(Limits::default())
    };
    send(
        &mut json,
        &json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "capabilities": {"textDocument": {"diagnostic": {}}}}}),
    );
    let uri = file_uri("/pull/cancelled.jai").as_str().to_string();
    send(
        &mut json,
        &json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {
            "textDocument": {"uri": uri, "languageId": "jai", "version": 1, "text": SOURCE}}}),
    );
    (json, uri)
}

fn pull(json: &mut JsonSession, uri: &str) -> Value {
    send(
        json,
        &json!({"jsonrpc": "2.0", "id": 7, "method": "textDocument/diagnostic",
            "params": {"textDocument": {"uri": uri}}}),
    )
    .remove(0)
}

fn reset() {
    jaic::interrupt::request(false);
    jaic::interrupt::stop_after(0);
    let _ = jaic::interrupt::disarm();
}

#[test]
fn a_pull_whose_check_was_stopped_is_cancelled_with_retrigger() {
    let _guard = LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset();
    let (mut json, uri) = session(true);
    jaic::interrupt::arm();
    jaic::interrupt::stop_after(3);
    let stopped = pull(&mut json, &uri);
    reset();
    assert_eq!(stopped["error"]["code"], -32802, "{stopped}");
    assert_eq!(stopped["error"]["data"]["retriggerRequest"], true);
    assert!(stopped.get("result").is_none());
    // The retry, undisturbed, is the full report.
    let again = pull(&mut json, &uri);
    assert_eq!(again["result"]["kind"], "full");
    assert!(
        !again["result"]["items"].as_array().unwrap().is_empty(),
        "{again}"
    );
}

#[test]
fn a_session_without_an_environment_reports_what_it_has() {
    let _guard = LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset();
    let (mut json, uri) = session(false);
    let report = pull(&mut json, &uri);
    assert_eq!(report["result"]["kind"], "full", "{report}");
    assert!(report.get("error").is_none());
}

#[test]
fn pushed_diagnostics_leave_out_a_document_whose_check_was_stopped() {
    let _guard = LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset();
    let mut session = test_support::lsp::lean_session();
    let uri = file_uri("/pull/pushed.jai");
    session.open(uri.clone(), 1, SOURCE.into()).unwrap();
    jaic::interrupt::arm();
    jaic::interrupt::stop_after(3);
    let stopped = session.publications();
    reset();
    assert!(stopped.is_empty(), "{stopped:?}");
    let again = session.publications();
    assert_eq!(again.len(), 1);
    assert!(!again[0].2.is_empty());
}
