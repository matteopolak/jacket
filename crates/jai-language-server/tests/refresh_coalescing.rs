//! A refresh request asked for while the client still owes the answer to the last one is
//! neither sent twice nor lost: it is sent once when the answer arrives.
use jai_language_server::{JsonSession, Limits};
use serde_json::{Value, json};
use test_support::lsp::{environment, file_uri};

fn send(json: &mut JsonSession, message: &Value) -> Vec<Value> {
    json.handle_json(&message.to_string())
        .unwrap()
        .iter()
        .map(|m| serde_json::from_str(m).unwrap())
        .collect()
}

fn refreshes(messages: &[Value], method: &str) -> Vec<Value> {
    messages
        .iter()
        .filter(|m| m["method"] == method)
        .cloned()
        .collect()
}

fn started() -> JsonSession {
    let mut json = JsonSession::with_environment(Limits::default(), environment());
    send(
        &mut json,
        &json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"capabilities": {
            "textDocument": {"diagnostic": {}},
            "workspace": {"diagnostics": {"refreshSupport": true}}}}}),
    );
    send(
        &mut json,
        &json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}),
    );
    json
}

/// A settings file changing on disk asks a pulling client to pull again.
fn settings_changed(json: &mut JsonSession) -> Vec<Value> {
    send(
        json,
        &json!({"jsonrpc": "2.0", "method": "workspace/didChangeWatchedFiles", "params": {
            "changes": [{"uri": file_uri("/refresh/jacket.toml").as_str(), "type": 2}]}}),
    )
}

#[test]
fn a_refresh_asked_for_while_one_is_outstanding_is_sent_once_on_the_answer() {
    let mut json = started();
    let first = refreshes(&settings_changed(&mut json), "workspace/diagnostic/refresh");
    assert_eq!(first.len(), 1);
    // Two more while the client has not answered: nothing is sent.
    for _ in 0..2 {
        let more = settings_changed(&mut json);
        assert!(refreshes(&more, "workspace/diagnostic/refresh").is_empty());
    }
    // The answer brings exactly one request for what was asked meanwhile.
    let answer = send(
        &mut json,
        &json!({"jsonrpc": "2.0", "id": first[0]["id"], "result": null}),
    );
    let again = refreshes(&answer, "workspace/diagnostic/refresh");
    assert_eq!(again.len(), 1, "{answer:?}");
    // Answering that one leaves nothing further.
    let last = send(
        &mut json,
        &json!({"jsonrpc": "2.0", "id": again[0]["id"], "result": null}),
    );
    assert!(last.is_empty(), "{last:?}");
    // And with nothing outstanding the next request goes straight out.
    let next = refreshes(&settings_changed(&mut json), "workspace/diagnostic/refresh");
    assert_eq!(next.len(), 1);
}
