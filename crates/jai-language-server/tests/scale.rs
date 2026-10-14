//! Large inputs and compile-time code that cannot finish: the server answers instead of stalling,
//! dropping the document or dying.
use jai_language_server::{DocumentUri, Environment, JsonSession, Limits, Position, Session};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn environment() -> Environment {
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    Environment {
        fs: std::rc::Rc::new(jaic::sema::NativeFs),
        options: Box::new(move |_| {
            let mut options = jaic::sema::Options::host();
            options.import_paths = vec![stdlib.clone()];
            options.preload = Some(stdlib.join(jaic::PRELOAD_ENTRY));
            options
        }),
    }
}

/// Focus's `src/main.jai` imports `Basic` with a parameter its build metaprogram defines, so opened
/// alone the import never resolves and dozens of language files `#insert` code that waits for it.
/// Settling those items used to expand them again for every compile-time run beneath them, which
/// grew exponentially: no diagnostics within ten minutes, several GiB. Skipped without the corpus.
#[test]
fn project_with_an_unresolvable_import_still_answers() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../corpus/upstream/focus-editor--focus/src/main.jai");
    let Ok(text) = std::fs::read_to_string(&root) else {
        return;
    };
    let mut s = Session::with_environment(Limits::default(), environment());
    let uri = DocumentUri::parse(&format!(
        "file://{}",
        root.canonicalize().unwrap().display()
    ))
    .unwrap();
    let started = Instant::now();
    s.open(uri.clone(), 1, text).unwrap();
    let diagnostics = s.diagnostics(&uri).unwrap();
    assert!(!diagnostics.is_empty());
    assert!(
        started.elapsed() < Duration::from_secs(60),
        "took {:?}",
        started.elapsed()
    );
}

/// `count` small procedures (about 55 bytes and 24 tokens each) and a `main` that calls the last.
fn generated(count: usize) -> String {
    let mut text = String::from("#import \"Basic\";\n");
    for i in 0..count {
        text.push_str(&format!(
            "proc_{i} :: (a: int) -> int {{ return a + {i}; }}\n"
        ));
    }
    text.push_str(&format!(
        "main :: () {{ x := proc_{}(1); print(\"%\\n\", x); }}\n",
        count - 1
    ));
    text
}

fn big_uri() -> DocumentUri {
    DocumentUri::parse("file:///lsp-scale-test/big.jai").unwrap()
}

/// A document of 700 KiB, 18,000 tokens past the old caps (256 KiB, 8192 tokens, 1024
/// declarations) opens, is analysed in full and answers.
#[test]
fn large_document_opens_and_answers() {
    let text = generated(13_000);
    assert!(text.len() > 600 * 1024);
    let mut s = Session::with_environment(Limits::default(), environment());
    s.open(big_uri(), 1, text.clone()).unwrap();
    let diagnostics = s.diagnostics(&big_uri()).unwrap();
    assert!(
        !diagnostics
            .iter()
            .any(|d| d.code == jai_language_server::DiagnosticCode::Limit),
        "{:?}",
        diagnostics.iter().find(|d| d.message.contains("budget"))
    );
    let symbols = s.document_symbols(&big_uri()).unwrap();
    assert_eq!(symbols.len(), 13_001);
    assert!(!s.semantic_tokens(&big_uri()).unwrap().is_empty());
    // Hover on the call in `main` has a type-checked answer.
    let line = text.lines().count() as u32 - 1;
    let column = text.lines().last().unwrap().find("proc_12999").unwrap() as u32 + 2;
    let hover = s
        .hover(
            &big_uri(),
            Position {
                line,
                character: column,
            },
        )
        .unwrap();
    assert!(hover.is_some(), "hover on a call in a large document");
}

/// A `didOpen` message past the old 1 MiB cap is accepted, and later requests find the document.
#[test]
fn large_message_is_accepted_through_the_json_session() {
    let mut json = JsonSession::with_environment(Limits::default(), environment());
    let text = generated(30_000);
    assert!(text.len() > 1024 * 1024);
    let send = |json: &mut JsonSession, value: serde_json::Value| -> Vec<serde_json::Value> {
        json.handle_json(&value.to_string())
            .unwrap()
            .iter()
            .map(|m| serde_json::from_str(m).unwrap())
            .collect()
    };
    send(
        &mut json,
        serde_json::json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} }),
    );
    send(
        &mut json,
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": big_uri().as_str(), "languageId": "jai", "version": 1, "text": text,
            } },
        }),
    );
    let out = send(
        &mut json,
        serde_json::json!({
            "jsonrpc": "2.0", "id": 2, "method": "textDocument/documentSymbol",
            "params": { "textDocument": { "uri": big_uri().as_str() } },
        }),
    );
    assert_eq!(out[0]["result"].as_array().unwrap().len(), 30_001);
}

/// A document over the byte cap is refused out loud: a `window/showMessage` names it, instead of
/// the later requests alone failing with "document is not open".
#[test]
fn refused_document_is_reported_to_the_client() {
    let mut json = JsonSession::new(Limits {
        document_bytes: 1024,
        ..Limits::default()
    });
    json.handle_json(
        &serde_json::json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} })
            .to_string(),
    )
    .unwrap();
    let out = json
        .handle_json(
            &serde_json::json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": { "textDocument": {
                    "uri": big_uri().as_str(), "languageId": "jai", "version": 1,
                    "text": generated(100),
                } },
            })
            .to_string(),
        )
        .unwrap();
    let messages: Vec<serde_json::Value> = out
        .iter()
        .map(|m| serde_json::from_str(m).unwrap())
        .collect();
    let shown = messages
        .iter()
        .find(|m| m["method"] == "window/showMessage")
        .expect("a showMessage");
    let text = shown["params"]["message"].as_str().unwrap();
    assert!(
        text.contains("big.jai") && text.contains("budget"),
        "{text}"
    );
}
