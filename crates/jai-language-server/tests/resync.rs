use jai_language_server::{JsonSession, Limits};
use serde_json::{Value, json};

fn send(session: &mut JsonSession, message: Value) -> Vec<Value> {
    session
        .handle_json(&message.to_string())
        .unwrap()
        .iter()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect()
}

fn started() -> JsonSession {
    let mut session = JsonSession::new(Limits::default());
    send(
        &mut session,
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} }),
    );
    session
}

fn open(session: &mut JsonSession, uri: &str, text: &str) -> Vec<Value> {
    send(
        session,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": uri, "languageId": "jai", "version": 1, "text": text,
            } },
        }),
    )
}

#[test]
fn a_failed_change_drops_the_document_and_says_so() {
    let mut session = started();
    let uri = "file:///workspace/main.jai";
    open(&mut session, uri, "a :: 1;\n");
    let reversed = json!({
        "start": { "line": 0, "character": 5 },
        "end": { "line": 0, "character": 1 },
    });
    let out = send(
        &mut session,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": { "uri": uri, "version": 2 },
                "contentChanges": [{ "range": reversed, "text": "b" }],
            },
        }),
    );
    let methods: Vec<&str> = out.iter().filter_map(|m| m["method"].as_str()).collect();
    assert!(methods.contains(&"window/showMessage"), "{out:?}");
    let cleared = out
        .iter()
        .find(|m| m["method"] == "textDocument/publishDiagnostics")
        .expect("diagnostics of the dropped document are cleared");
    assert_eq!(cleared["params"]["diagnostics"], json!([]));
    // The stale text is gone: nothing answers from it until the client opens the file again.
    let symbols = send(
        &mut session,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/documentSymbol",
            "params": { "textDocument": { "uri": uri } },
        }),
    );
    assert!(symbols[0].get("error").is_some(), "{symbols:?}");
    // Opening it again resyncs.
    open(&mut session, uri, "b :: 2;\n");
    let symbols = send(
        &mut session,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/documentSymbol",
            "params": { "textDocument": { "uri": uri } },
        }),
    );
    assert!(symbols[0].get("error").is_none(), "{symbols:?}");
}
