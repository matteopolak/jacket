//! Standard LSP schemas independently check the portable JSON boundary.
use jai_language_server::{JsonSession, Limits, Position, Range, TextChange};
use lsp_types::notification::Notification as _;
use lsp_types::request::Request as _;
use serde_json::{Value, json};

fn request(session: &mut JsonSession, id: i32, method: &str, params: Value) -> Value {
    let input = json!({"jsonrpc":"2.0","id":id,"method":method,"params":params});
    let result = session
        .handle_json(&input.to_string())
        .unwrap()
        .into_iter()
        .map(|message| serde_json::from_str::<Value>(&message).unwrap())
        .find(|message| message["id"] == id)
        .expect("request response")["result"]
        .clone();
    if method == "initialize" {
        session
            .handle_json(r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#)
            .unwrap();
    }
    result
}

#[test]
fn positions_ranges_and_changes_match_the_standard_wire_schema() {
    let standard = lsp_types::Range::new(
        lsp_types::Position::new(2, 3),
        lsp_types::Position::new(4, 5),
    );
    let value = serde_json::to_value(standard).unwrap();
    let range: Range = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(
        range.start,
        Position {
            line: 2,
            character: 3
        }
    );
    assert_eq!(serde_json::to_value(range).unwrap(), value);
    assert_eq!(
        std::mem::size_of::<Position>(),
        std::mem::size_of::<lsp_types::Position>()
    );
    assert_eq!(
        std::mem::size_of::<Range>(),
        std::mem::size_of::<lsp_types::Range>()
    );
    for event in [
        lsp_types::TextDocumentContentChangeEvent {
            range: Some(standard),
            range_length: Some(7),
            text: "😀".into(),
        },
        lsp_types::TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "whole document".into(),
        },
    ] {
        let change: TextChange =
            serde_json::from_value(serde_json::to_value(&event).unwrap()).unwrap();
        assert_eq!(
            change.range.map(|r| serde_json::to_value(r).unwrap()),
            event.range.map(|r| serde_json::to_value(r).unwrap())
        );
        assert_eq!(change.range_length, event.range_length);
        assert_eq!(change.text, event.text);
    }
}

#[test]
fn standard_client_messages_receive_standard_capabilities_symbols_and_hover() {
    let mut session = JsonSession::new(Limits::default());
    let initialized = request(
        &mut session,
        1,
        lsp_types::request::Initialize::METHOD,
        serde_json::to_value(lsp_types::InitializeParams::default()).unwrap(),
    );
    let initialized: lsp_types::InitializeResult = serde_json::from_value(initialized).unwrap();
    assert_eq!(
        initialized.capabilities.position_encoding,
        Some(lsp_types::PositionEncodingKind::UTF16)
    );
    assert!(initialized.capabilities.rename_provider.is_some());
    assert!(initialized.capabilities.semantic_tokens_provider.is_some());
    let uri: lsp_types::Uri = "file:///wire/main.jai".parse().unwrap();
    let opened = lsp_types::DidOpenTextDocumentParams {
        text_document: lsp_types::TextDocumentItem {
            uri: uri.clone(),
            language_id: "jai".into(),
            version: 1,
            text: "// 😀\nanswer :: 42;\nmain :: () { value := answer; }".into(),
        },
    };
    let notice = json!({"jsonrpc":"2.0","method":lsp_types::notification::DidOpenTextDocument::METHOD,"params":opened});
    for message in session.handle_json(&notice.to_string()).unwrap() {
        let value: Value = serde_json::from_str(&message).unwrap();
        if value["method"] == lsp_types::notification::PublishDiagnostics::METHOD {
            let _: lsp_types::PublishDiagnosticsParams =
                serde_json::from_value(value["params"].clone()).unwrap();
        }
    }
    let symbols = request(
        &mut session,
        2,
        lsp_types::request::DocumentSymbolRequest::METHOD,
        json!({"textDocument":{"uri":uri}}),
    );
    let symbols: lsp_types::DocumentSymbolResponse = serde_json::from_value(symbols).unwrap();
    let lsp_types::DocumentSymbolResponse::Nested(symbols) = symbols else {
        panic!("hierarchical symbols")
    };
    assert!(symbols.iter().any(|symbol| symbol.name == "answer"));
    let hover = request(
        &mut session,
        3,
        lsp_types::request::HoverRequest::METHOD,
        json!({"textDocument":{"uri":uri},"position":{"line":1,"character":2}}),
    );
    let hover: Option<lsp_types::Hover> = serde_json::from_value(hover).unwrap();
    assert!(hover.is_some(), "schema checks retain a useful answer");
}

#[test]
fn completion_lists_use_standard_item_fields_and_enums() {
    let mut session = JsonSession::new(Limits::default());
    request(
        &mut session,
        1,
        lsp_types::request::Initialize::METHOD,
        json!({"capabilities":{}}),
    );
    let uri = "file:///wire/completion.jai";
    let opened = json!({"jsonrpc":"2.0","method":lsp_types::notification::DidOpenTextDocument::METHOD,
        "params":{"textDocument":{"uri":uri,"languageId":"jai","version":1,
            "text":"answer :: 42;\nmain :: () { ans }"}}});
    session.handle_json(&opened.to_string()).unwrap();
    let result = request(
        &mut session,
        2,
        lsp_types::request::Completion::METHOD,
        json!({"textDocument":{"uri":uri},"position":{"line":1,"character":16}}),
    );
    let list: lsp_types::CompletionList = serde_json::from_value(result).unwrap();
    let item = list
        .items
        .iter()
        .find(|item| item.label == "answer")
        .expect("useful matching completion");
    assert_eq!(item.kind, Some(lsp_types::CompletionItemKind::CONSTANT));
    assert!(item.text_edit.is_some());
}

#[test]
fn standard_action_schema_rejects_malformed_parameters_without_panicking() {
    let mut session = JsonSession::new(Limits::default());
    request(
        &mut session,
        1,
        lsp_types::request::Initialize::METHOD,
        json!({"capabilities":{}}),
    );
    for (params, expected) in [
        (Value::Null, -32602),
        // A scalar violates the JSON-RPC structured-params envelope before LSP decoding.
        (json!(1), -32600),
        (json!([]), -32602),
        (json!({"context":1}), -32602),
        (json!({"context":{"diagnostics":[1]}}), -32602),
    ] {
        let message = json!({"jsonrpc":"2.0","id":2,
            "method":lsp_types::request::CodeActionRequest::METHOD,"params":params});
        let replies = session.handle_json(&message.to_string()).unwrap();
        let reply: Value = serde_json::from_str(&replies[0]).unwrap();
        assert_eq!(reply["error"]["code"], expected, "{params}: {reply}");
    }
}

#[test]
fn code_actions_require_the_standard_context_and_diagnostic_fields() {
    let mut session = JsonSession::new(Limits::default());
    request(
        &mut session,
        1,
        lsp_types::request::Initialize::METHOD,
        json!({"capabilities":{}}),
    );
    let uri = "file:///wire/actions.jai";
    session
        .handle_json(
            &json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
        "textDocument":{"uri":uri,"languageId":"jai","version":1,"text":"main :: () {}"}}})
            .to_string(),
        )
        .unwrap();
    let range = json!({"start":{"line":0,"character":0},"end":{"line":0,"character":0}});
    for context in [
        None,
        Some(json!({})),
        Some(json!({"diagnostics":[{"range":range,"code":"check","source":"jaic"}]})),
    ] {
        let mut params = json!({"textDocument":{"uri":uri},"range":range});
        if let Some(context) = context {
            params["context"] = context;
        }
        let replies = session
            .handle_json(
                &json!({"jsonrpc":"2.0","id":2,"method":"textDocument/codeAction","params":params})
                    .to_string(),
            )
            .unwrap();
        let reply: Value = serde_json::from_str(&replies[0]).unwrap();
        assert_eq!(reply["error"]["code"], -32602, "{reply}");
    }
    let result = request(
        &mut session,
        3,
        lsp_types::request::CodeActionRequest::METHOD,
        json!({"textDocument":{"uri":uri},"range":range,"context":{"diagnostics":[]}}),
    );
    assert!(result.is_array());
}
