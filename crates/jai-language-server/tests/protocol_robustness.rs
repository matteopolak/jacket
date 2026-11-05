use jai_language_server::{JsonSession, Limits};
use serde_json::{Value, json};

fn send(session: &mut JsonSession, message: Value) -> Vec<Value> {
    let output: Vec<Value> = session
        .handle_json(&message.to_string())
        .unwrap()
        .iter()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    if message["method"] == "initialize"
        && output.first().is_some_and(|m| m.get("result").is_some())
    {
        assert!(
            session
                .handle_json(r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#)
                .unwrap()
                .is_empty()
        );
    }
    output
}

fn started() -> JsonSession {
    let mut session = JsonSession::new(Limits::default());
    send(
        &mut session,
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "capabilities": {} } }),
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
            .cancel_request(jai_language_server::RequestId::from(id))
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
fn an_oversized_declared_body_is_rejected_before_reading_it() {
    use jai_language_server::transport::BoundedReader;
    let wire = b"Content-Length: 1000000000\r\n\r\n";
    let mut reader = BoundedReader::new(std::io::Cursor::new(wire), 128);
    assert!(reader.read_frame().is_err());
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
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "capabilities": {} } }),
    );
    // This resource test deliberately caps even the initialization result. Advance the
    // test client explicitly so the publication path, rather than initialization, is tested.
    session
        .handle_json(r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#)
        .unwrap();
    let broken = "main :: ( { } ) ;;; ( ( ( x := ; y := ; z := ;\n".repeat(20);
    open(&mut session, "file:///workspace/main.jai", &broken);
    let flushed = session
        .flush_publications()
        .expect("the limit is not fatal");
    let logged: Value = serde_json::from_str(&flushed[0]).unwrap();
    assert_eq!(logged["method"], "window/logMessage");
}

#[test]
fn library_framing_accepts_a_quoted_utf8_charset() {
    use jai_language_server::transport::{BoundedReader, Input};
    let body = r#"{"jsonrpc":"2.0","method":"exit"}"#;
    let wire = format!(
        "Content-Length: {}\r\nContent-Type: application/vscode-jsonrpc; charset=\"utf-8\"\r\n\r\n{body}",
        body.len()
    );
    let mut reader = BoundedReader::new(std::io::Cursor::new(wire), 128);
    assert!(matches!(
        reader.read_frame().unwrap(),
        Some(Input::Message { .. })
    ));
    assert!(reader.read_frame().unwrap().is_none());
}

#[test]
fn local_file_uris_have_one_spelling() {
    use jai_language_server::DocumentUri;
    let plain = DocumentUri::parse("file:///work/a.jai").unwrap();
    for spelling in [
        "file://localhost/work/a.jai",
        "file:/work/a.jai",
        "FILE:///work/a.jai",
    ] {
        assert_eq!(DocumentUri::parse(spelling).unwrap(), plain, "{spelling}");
    }
    assert!(DocumentUri::parse("file://other/work/a.jai").is_err());
}

#[test]
fn documents_that_are_not_local_files_are_left_alone() {
    let mut session = started();
    let out = open(&mut session, "untitled:Untitled-1", "x :: 1;");
    assert!(out.is_empty(), "{out:?}");
    let out = send(
        &mut session,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": { "uri": "untitled:Untitled-1", "version": 2 },
                "contentChanges": [{ "text": "y :: 2;" }],
            },
        }),
    );
    assert!(out.is_empty(), "{out:?}");
}

#[test]
fn a_jai_file_opens_whatever_the_client_calls_its_language() {
    let mut session = started();
    let out = send(
        &mut session,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": "file:///workspace/main.jai", "languageId": "Jai", "version": 1,
                "text": "a :: 1;\n",
            } },
        }),
    );
    assert_eq!(out[0]["method"], "textDocument/publishDiagnostics");
    let out = send(
        &mut session,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": "file:///workspace/other.jai", "languageId": "plaintext", "version": 1,
                "text": "a :: 1;\n",
            } },
        }),
    );
    assert!(
        out.iter()
            .any(|m| m["method"] == "textDocument/publishDiagnostics"),
        "{out:?}"
    );
}
