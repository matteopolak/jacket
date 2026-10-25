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

fn symbols_request(id: i64, uri: &str) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "textDocument/documentSymbol",
        "params": { "textDocument": { "uri": uri } },
    })
    .to_string()
}

fn cancel(id: i64) -> String {
    json!({ "jsonrpc": "2.0", "method": "$/cancelRequest", "params": { "id": id } }).to_string()
}

#[test]
fn a_request_queued_ahead_of_its_cancellation_is_cancelled() {
    let mut session = started();
    let uri = "file:///workspace/main.jai";
    open(&mut session, uri, "a :: 1;\n");
    let batch = [symbols_request(2, uri), symbols_request(3, uri), cancel(3)];
    let answers = session.handle_batch(&batch).unwrap();
    let first: Value = serde_json::from_str(&answers[0][0]).unwrap();
    let second: Value = serde_json::from_str(&answers[1][0]).unwrap();
    assert!(first.get("error").is_none(), "{first}");
    assert_eq!(second["error"]["code"], -32800);
}

#[test]
fn cancellations_that_never_match_do_not_fill_the_store() {
    let mut session = started();
    let uri = "file:///workspace/main.jai";
    open(&mut session, uri, "a :: 1;\n");
    for id in 100..1000 {
        session
            .cancel_request(jai_language_server::RequestId::Number(id))
            .unwrap();
    }
    let batch = [
        symbols_request(5000, uri),
        cancel(5000),
        symbols_request(5001, uri),
    ];
    // The cancellation for 5000 arrives after it ran; the one for 5001 is the latest.
    let batch = [batch[0].clone(), batch[2].clone(), cancel(5001)];
    let answers = session.handle_batch(&batch).unwrap();
    let second: Value = serde_json::from_str(&answers[1][0]).unwrap();
    assert_eq!(second["error"]["code"], -32800);
}

#[test]
fn an_oversized_request_gets_an_error_response() {
    use jai_language_server::framing::{FrameDecoder, encode};
    let padding = "x".repeat(300);
    let body = format!(
        r#"{{"jsonrpc":"2.0","id":41,"method":"textDocument/hover","params":{{"pad":"{padding}"}}}}"#
    );
    let mut decoder = FrameDecoder::new(128);
    assert!(decoder.push(&encode(&body)).unwrap().is_empty());
    let skipped = decoder.take_oversized();
    let replies: Vec<Value> = jai_language_server::oversized_replies(&skipped[0], 128)
        .iter()
        .map(|reply| serde_json::from_str(reply).unwrap())
        .collect();
    assert_eq!(replies[0]["id"], 41);
    assert!(replies[0]["error"]["code"].is_i64(), "{replies:?}");
    assert_eq!(replies[1]["method"], "window/showMessage");
}

#[test]
fn publications_over_the_output_limit_do_not_end_the_session() {
    let mut session = JsonSession::new(Limits {
        output_bytes: 300,
        ..Limits::default()
    });
    session.defer_publications(true);
    send(
        &mut session,
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} }),
    );
    let broken = "main :: ( { } ) ;;; ( ( ( x := ; y := ; z := ;\n".repeat(20);
    open(&mut session, "file:///workspace/main.jai", &broken);
    let flushed = session
        .flush_publications()
        .expect("the limit is not fatal");
    let logged: Value = serde_json::from_str(&flushed[0]).unwrap();
    assert_eq!(logged["method"], "window/logMessage");
}
