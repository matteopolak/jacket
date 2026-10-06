use jai_language_server::{JsonSession, Limits, RequestId};
use serde_json::{Value, json};

/// Hover text of `answer` (a syntax-only session) after initializing with `capabilities`.
fn hover_with(capabilities: Value) -> Value {
    let mut session = JsonSession::new(Limits::default());
    send(
        &mut session,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "capabilities": capabilities },
        }),
    );
    send(
        &mut session,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace/main.jai",
                    "languageId": "jai",
                    "version": 1,
                    "text": "answer :: () -> int {return 42;}\nmain :: () -> int {return answer();}",
                },
            },
        }),
    );
    let out = send(
        &mut session,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/hover",
            "params": {
                "textDocument": { "uri": "file:///workspace/main.jai" },
                "position": { "line": 1, "character": 27 },
            },
        }),
    );
    out[0]["result"]["contents"].clone()
}

#[test]
fn hovers_are_markdown_when_the_client_renders_it() {
    let markdown = hover_with(
        json!({ "textDocument": { "hover": { "contentFormat": ["markdown", "plaintext"] } } }),
    );
    assert_eq!(markdown["kind"], "markdown");
    let value = markdown["value"].as_str().unwrap();
    assert!(
        value.starts_with("```jai\nanswer :: () -> int {return 42;}\n```\n\nSource syntax"),
        "{value}"
    );
    for capabilities in [
        json!({}),
        json!({ "textDocument": { "hover": { "contentFormat": ["plaintext"] } } }),
    ] {
        let plain = hover_with(capabilities);
        assert_eq!(plain["kind"], "plaintext");
        let value = plain["value"].as_str().unwrap();
        assert!(
            value.starts_with("answer :: () -> int {return 42;}\n\nSource syntax declaration."),
            "{value}"
        );
    }
}

fn send(session: &mut JsonSession, message: Value) -> Vec<Value> {
    session
        .handle_json(&message.to_string())
        .unwrap()
        .iter()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect()
}

fn initialize(session: &mut JsonSession) {
    let output = send(
        session,
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} }),
    );
    assert_eq!(
        output[0]["result"]["capabilities"]["positionEncoding"],
        "utf-16"
    );
    assert_eq!(
        output[0]["result"]["capabilities"]["experimental"]["jai"]["compileTimeExecution"],
        false
    );
}

#[test]
fn jsonrpc_lifecycle_and_notification_response_rules_are_real() {
    let mut session = JsonSession::new(Limits::default());
    let before = send(
        &mut session,
        json!({ "jsonrpc": "2.0", "id": 0, "method": "textDocument/hover", "params": {} }),
    );
    assert_eq!(before[0]["error"]["code"], -32002);
    initialize(&mut session);
    assert!(
        send(
            &mut session,
            json!({ "jsonrpc": "2.0", "method": "unknown/notification" })
        )
        .is_empty()
    );
    let shutdown = send(
        &mut session,
        json!({ "jsonrpc": "2.0", "id": "stop", "method": "shutdown" }),
    );
    assert_eq!(shutdown[0]["id"], "stop");
    assert!(shutdown[0]["result"].is_null());
    assert!(send(&mut session, json!({ "jsonrpc": "2.0", "method": "exit" })).is_empty());
    assert_eq!(session.exit_status(), Some(0));
}

#[test]
fn versioned_diagnostics_completion_hover_and_definition_use_standard_shapes() {
    let mut session = JsonSession::new(Limits::default());
    initialize(&mut session);
    let text = "answer :: () -> int {return 42;}\nmain :: () -> int {return answer();}";
    let open = send(
        &mut session,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace/main.jai",
                    "languageId": "jai",
                    "version": 1,
                    "text": text,
                },
            },
        }),
    );
    assert_eq!(open[0]["method"], "textDocument/publishDiagnostics");
    assert_eq!(open[0]["params"]["version"], 1);
    assert_eq!(open[0]["params"]["diagnostics"], json!([]));
    for (id, method) in [
        (2, "textDocument/definition"),
        (3, "textDocument/hover"),
        (4, "textDocument/completion"),
    ] {
        let out = send(
            &mut session,
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": method,
                "params": {
                    "textDocument": { "uri": "file:///workspace/main.jai" },
                    "position": { "line": 1, "character": 27 },
                },
            }),
        );
        assert!(out[0].get("error").is_none(), "{out:?}");
        assert!(!out[0]["result"].is_null());
        if method == "textDocument/definition" {
            assert_eq!(out[0]["result"][0]["range"]["start"]["line"], 0);
        }
        if method == "textDocument/hover" {
            assert_eq!(out[0]["result"]["contents"]["kind"], "plaintext");
        }
        if method == "textDocument/completion" {
            let answer = out[0]["result"]["items"]
                .as_array()
                .unwrap()
                .iter()
                .find(|item| item["label"] == "answer")
                .unwrap();
            assert_eq!(answer["kind"], 3);
            assert!(answer["detail"].as_str().unwrap().contains("return 42"));
        }
    }
    let symbols = send(
        &mut session,
        json!({
            "jsonrpc": "2.0",
            "id": 9,
            "method": "textDocument/documentSymbol",
            "params": { "textDocument": { "uri": "file:///workspace/main.jai" } },
        }),
    );
    assert_eq!(symbols[0]["result"][0]["kind"], 12);
    assert!(symbols[0]["result"][0].get("children").is_none());
    let tokens = send(
        &mut session,
        json!({
            "jsonrpc": "2.0",
            "id": 10,
            "method": "textDocument/semanticTokens/full",
            "params": { "textDocument": { "uri": "file:///workspace/main.jai" } },
        }),
    );
    let data = tokens[0]["result"]["data"].as_array().unwrap();
    assert!(
        data.as_chunks::<5>()
            .0
            .iter()
            .any(|token| token[3] == 4 && token[4] == 1)
    );
    let changed = send(
        &mut session,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": { "uri": "file:///workspace/main.jai", "version": 2 },
                "contentChanges": [{ "text": "main :: (" }],
            },
        }),
    );
    assert_eq!(changed[0]["params"]["version"], 2);
    assert_eq!(changed[0]["params"]["diagnostics"][0]["severity"], 1);
    assert_eq!(changed[0]["params"]["diagnostics"][0]["code"], "jai-parser");
    assert!(
        !changed[0]["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let closed = send(
        &mut session,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didClose",
            "params": { "textDocument": { "uri": "file:///workspace/main.jai" } },
        }),
    );
    assert_eq!(closed[0]["params"]["diagnostics"], json!([]));
}

#[test]
fn queued_cancellation_and_completed_responses_have_distinct_semantics() {
    let mut session = JsonSession::new(Limits::default());
    initialize(&mut session);
    session.cancel_request(RequestId::Number(7)).unwrap();
    let output = send(
        &mut session,
        json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "textDocument/documentSymbol",
            "params": { "textDocument": { "uri": "file:///workspace/missing.jai" } },
        }),
    );
    assert_eq!(output[0]["error"]["code"], -32800);
    session.cancel_request(RequestId::Number(1)).unwrap();
    let repeated = send(
        &mut session,
        json!({ "jsonrpc": "2.0", "id": 1, "method": "shutdown" }),
    );
    assert!(repeated[0].get("error").is_none());
}

#[test]
fn malformed_json_invalid_identifiers_and_message_limits_fail_explicitly() {
    let mut session = JsonSession::new(Limits::default());
    let output: Value = serde_json::from_str(&session.handle_json("{").unwrap()[0]).unwrap();
    assert_eq!(output["error"]["code"], -32700);
    assert!(output["id"].is_null());
    let invalid = send(
        &mut session,
        json!({ "jsonrpc": "2.0", "id": true, "method": "initialize" }),
    );
    assert_eq!(invalid[0]["error"]["code"], -32600);
    let mut tiny = JsonSession::new(Limits {
        message_bytes: 4,
        ..Limits::default()
    });
    assert!(tiny.handle_json("{\"x\":1}").is_err());
}
