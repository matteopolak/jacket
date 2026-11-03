//! Semantic tokens for a range of the document, and as an edit of the previous answer.
use jai_language_server::{JsonSession, Limits};
use serde_json::{Value, json};

const URI: &str = "file:///workspace/tokens.jai";
const TEXT: &str = "first :: () -> int { return 1; }\nsecond :: () -> int { return 2; }\nthird :: () -> int { return 3; }\n";

fn send(session: &mut JsonSession, message: Value) -> Vec<Value> {
    session
        .handle_json(&message.to_string())
        .unwrap()
        .iter()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect()
}

fn opened() -> JsonSession {
    let mut session = JsonSession::new(Limits::default());
    send(
        &mut session,
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}),
    );
    send(
        &mut session,
        json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {"textDocument": {
            "uri": URI, "languageId": "jai", "version": 1, "text": TEXT}}}),
    );
    session
}

fn request(session: &mut JsonSession, id: u64, method: &str, params: Value) -> Value {
    let out = send(
        session,
        json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}),
    );
    out[0]["result"].clone()
}

fn integers(value: &Value) -> Vec<u64> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap())
        .collect()
}

/// The (line, column, length) of every token an encoded array holds.
fn positions(data: &[u64]) -> Vec<(u64, u64, u64)> {
    let (mut line, mut column) = (0, 0);
    data.chunks(5)
        .map(|t| {
            line += t[0];
            column = if t[0] == 0 {
                column + t[1]
            } else {
                t[1]
            };
            (line, column, t[2])
        })
        .collect()
}

#[test]
fn the_capabilities_offer_range_and_delta() {
    let mut session = JsonSession::new(Limits::default());
    let out = send(
        &mut session,
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}),
    );
    let provider = &out[0]["result"]["capabilities"]["semanticTokensProvider"];
    assert_eq!(provider["range"], true);
    assert_eq!(provider["full"]["delta"], true);
}

#[test]
fn a_range_holds_the_tokens_that_start_in_it() {
    let mut session = opened();
    let all = request(
        &mut session,
        2,
        "textDocument/semanticTokens/full",
        json!({"textDocument": {"uri": URI}}),
    );
    let all = positions(&integers(&all["data"]));
    let range = request(
        &mut session,
        3,
        "textDocument/semanticTokens/range",
        json!({"textDocument": {"uri": URI},
               "range": {"start": {"line": 1, "character": 0}, "end": {"line": 2, "character": 0}}}),
    );
    let data = integers(&range["data"]);
    let found = positions(&data);
    let expected: Vec<_> = all
        .iter()
        .filter(|(line, _, _)| *line == 1)
        .copied()
        .collect();
    assert!(!expected.is_empty());
    // Encoded from the start of the document, so the first token's line is absolute.
    assert_eq!(found, expected);
}

#[test]
fn a_delta_is_the_edit_from_the_last_answer() {
    let mut session = opened();
    let first = request(
        &mut session,
        2,
        "textDocument/semanticTokens/full",
        json!({"textDocument": {"uri": URI}}),
    );
    let before = integers(&first["data"]);
    send(
        &mut session,
        json!({"jsonrpc": "2.0", "method": "textDocument/didChange", "params": {
            "textDocument": {"uri": URI, "version": 2},
            "contentChanges": [{"text": TEXT.replace("second", "renamed_second")}]}}),
    );
    let delta = request(
        &mut session,
        3,
        "textDocument/semanticTokens/full/delta",
        json!({"textDocument": {"uri": URI}, "previousResultId": first["resultId"]}),
    );
    let edits = delta["edits"]
        .as_array()
        .expect("an edit, not the whole array");
    assert_eq!(edits.len(), 1);
    let start = edits[0]["start"].as_u64().unwrap() as usize;
    let delete = edits[0]["deleteCount"].as_u64().unwrap() as usize;
    let mut patched = before.clone();
    patched.splice(start..start + delete, integers(&edits[0]["data"]));
    let full = request(
        &mut session,
        4,
        "textDocument/semanticTokens/full",
        json!({"textDocument": {"uri": URI}}),
    );
    assert_eq!(patched, integers(&full["data"]));
    assert_ne!(patched, before);
}

#[test]
fn a_delta_against_an_unknown_result_is_the_whole_answer() {
    let mut session = opened();
    let delta = request(
        &mut session,
        2,
        "textDocument/semanticTokens/full/delta",
        json!({"textDocument": {"uri": URI}, "previousResultId": "nothing"}),
    );
    assert!(delta["edits"].is_null());
    assert!(!integers(&delta["data"]).is_empty());
}
