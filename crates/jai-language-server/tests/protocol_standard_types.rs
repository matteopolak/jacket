//! Standard LSP schemas independently check the portable JSON boundary.
use jai_language_server::{JsonSession, Limits, Position, Range};
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
    let standard = gen_lsp_types::Range::new(
        gen_lsp_types::Position::new(2, 3),
        gen_lsp_types::Position::new(4, 5),
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
        std::mem::size_of::<gen_lsp_types::Position>()
    );
    assert_eq!(
        std::mem::size_of::<Range>(),
        std::mem::size_of::<gen_lsp_types::Range>()
    );
    let mut session = JsonSession::new(Limits::default());
    request(&mut session, 1, "initialize", json!({"capabilities":{}}));
    let uri = jai_language_server::DocumentUri::parse("file:///wire/changes.jai").unwrap();
    session
        .handle_json(
            &json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
        "textDocument":{"uri":uri.as_str(),"languageId":"jai","version":1,"text":"alpha"}}})
            .to_string(),
        )
        .unwrap();
    let events: [gen_lsp_types::TextDocumentContentChangeEvent; 2] = [
        gen_lsp_types::TextDocumentContentChangePartial::new(
            Range::new(Position::new(0, 0), Position::new(0, 5)),
            Some(7),
            "😀".into(),
        )
        .into(),
        gen_lsp_types::TextDocumentContentChangeWholeDocument::new("whole document".into()).into(),
    ];
    for (index, (event, expected)) in events.into_iter().zip(["😀", "whole document"]).enumerate()
    {
        let version = i32::try_from(index).unwrap() + 2;
        session
            .handle_json(
                &json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{
            "textDocument":{"uri":uri.as_str(),"version":version},"contentChanges":[event]}})
                .to_string(),
            )
            .unwrap();
        assert_eq!(session.session().document_text(&uri).unwrap(), expected);
        assert_eq!(session.session().version(&uri).unwrap(), version);
    }
}

#[test]
fn standard_client_messages_receive_standard_capabilities_symbols_and_hover() {
    let mut session = JsonSession::new(Limits::default());
    let initialized = request(
        &mut session,
        1,
        gen_lsp_types::LspRequestMethod::Initialize.as_str(),
        serde_json::to_value(gen_lsp_types::InitializeParams::default()).unwrap(),
    );
    let initialized: gen_lsp_types::InitializeResult = serde_json::from_value(initialized).unwrap();
    assert_eq!(
        initialized.capabilities.position_encoding,
        Some(gen_lsp_types::PositionEncodingKind::UTF16)
    );
    assert!(initialized.capabilities.rename_provider.is_some());
    assert!(initialized.capabilities.semantic_tokens_provider.is_some());
    let uri: gen_lsp_types::Uri = "file:///wire/main.jai".parse().unwrap();
    let opened = gen_lsp_types::DidOpenTextDocumentParams {
        text_document: gen_lsp_types::TextDocumentItem {
            uri: uri.clone(),
            language_id: "jai".into(),
            version: 1,
            text: "// 😀\nanswer :: 42;\nmain :: () { value := answer; }".into(),
        },
    };
    let notice = json!({"jsonrpc":"2.0","method":gen_lsp_types::LspNotificationMethod::TextDocumentDidOpen.as_str(),"params":opened});
    for message in session.handle_json(&notice.to_string()).unwrap() {
        let value: Value = serde_json::from_str(&message).unwrap();
        if value["method"]
            == gen_lsp_types::LspNotificationMethod::TextDocumentPublishDiagnostics.as_str()
        {
            let _: gen_lsp_types::PublishDiagnosticsParams =
                serde_json::from_value(value["params"].clone()).unwrap();
        }
    }
    let symbols = request(
        &mut session,
        2,
        gen_lsp_types::LspRequestMethod::TextDocumentDocumentSymbol.as_str(),
        json!({"textDocument":{"uri":uri}}),
    );
    let symbols: gen_lsp_types::DocumentSymbolResponse = serde_json::from_value(symbols).unwrap();
    let gen_lsp_types::DocumentSymbolResponse::DocumentSymbolList(symbols) = symbols else {
        panic!("hierarchical symbols")
    };
    assert!(symbols.iter().any(|symbol| symbol.name == "answer"));
    let hover = request(
        &mut session,
        3,
        gen_lsp_types::LspRequestMethod::TextDocumentHover.as_str(),
        json!({"textDocument":{"uri":uri},"position":{"line":1,"character":2}}),
    );
    let hover: Option<gen_lsp_types::Hover> = serde_json::from_value(hover).unwrap();
    assert!(hover.is_some(), "schema checks retain a useful answer");
}

#[test]
fn completion_lists_use_standard_item_fields_and_enums() {
    let mut session = JsonSession::new(Limits::default());
    request(
        &mut session,
        1,
        gen_lsp_types::LspRequestMethod::Initialize.as_str(),
        json!({"capabilities":{}}),
    );
    let uri = "file:///wire/completion.jai";
    let opened = json!({"jsonrpc":"2.0","method":gen_lsp_types::LspNotificationMethod::TextDocumentDidOpen.as_str(),
        "params":{"textDocument":{"uri":uri,"languageId":"jai","version":1,
            "text":"answer :: 42;\nmain :: () { ans }"}}});
    session.handle_json(&opened.to_string()).unwrap();
    let result = request(
        &mut session,
        2,
        gen_lsp_types::LspRequestMethod::TextDocumentCompletion.as_str(),
        json!({"textDocument":{"uri":uri},"position":{"line":1,"character":16}}),
    );
    let list: gen_lsp_types::CompletionList = serde_json::from_value(result).unwrap();
    let item = list
        .items
        .iter()
        .find(|item| item.label == "answer")
        .expect("useful matching completion");
    assert_eq!(item.kind, Some(gen_lsp_types::CompletionItemKind::Constant));
    assert!(item.text_edit.is_some());
}

#[test]
fn standard_action_schema_rejects_malformed_parameters_without_panicking() {
    let mut session = JsonSession::new(Limits::default());
    request(
        &mut session,
        1,
        gen_lsp_types::LspRequestMethod::Initialize.as_str(),
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
            "method":gen_lsp_types::LspRequestMethod::TextDocumentCodeAction.as_str(),"params":params});
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
        gen_lsp_types::LspRequestMethod::Initialize.as_str(),
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
        gen_lsp_types::LspRequestMethod::TextDocumentCodeAction.as_str(),
        json!({"textDocument":{"uri":uri},"range":range,"context":{"diagnostics":[]}}),
    );
    assert!(result.is_array());
}

#[test]
fn standard_nullable_initialization_and_uri_spelling_are_preserved() {
    for folders in [None, Some(Value::Null), Some(json!([]))] {
        let mut input = json!({"processId":null,"rootUri":null,"capabilities":{}});
        if let Some(folders) = folders {
            input["workspaceFolders"] = folders;
        }
        let typed: gen_lsp_types::InitializeParams = serde_json::from_value(input.clone()).unwrap();
        assert_eq!(serde_json::to_value(typed).unwrap(), input);
        let mut session = JsonSession::new(Limits::default());
        let initialized = request(&mut session, 1, "initialize", input);
        let typed: gen_lsp_types::InitializeResult =
            serde_json::from_value(initialized.clone()).unwrap();
        assert_eq!(serde_json::to_value(typed).unwrap(), initialized);
    }
    for uri in [
        "file:///workspace/a%20b.jai",
        "file:///C:/Work/Main.jai",
        "file://server/share/main.jai",
        "file:///workspace/a%23b.jai",
    ] {
        let params = json!({"textDocument":{"uri":uri}});
        let typed: gen_lsp_types::DocumentDiagnosticParams =
            serde_json::from_value(params.clone()).unwrap();
        assert_eq!(serde_json::to_value(typed).unwrap(), params);
    }
}
