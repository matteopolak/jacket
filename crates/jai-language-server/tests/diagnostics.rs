//! Several type-checker errors per document, and pull diagnostics.
use jai_language_server::{DiagnosticCode, DocumentUri, Environment, JsonSession, Limits, Session};
use std::path::PathBuf;

fn environment() -> Environment {
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    Environment {
        fs: std::rc::Rc::new(jaic::sema::NativeFs),
        options: Box::new(move |_| {
            let mut options = jaic::sema::Options::host();
            options.import_paths = vec![stdlib.clone()];
            options.preload = Some(stdlib.join("prelude/module.jai"));
            options
        }),
    }
}

fn session() -> Session {
    Session::with_environment(Limits::default(), environment())
}

fn uri() -> DocumentUri {
    DocumentUri::parse("file:///lsp-diagnostics-test/main.jai").unwrap()
}

const TWO_BODIES: &str = r#"#import "Basic";
first :: () { x: int = "hello"; }
second :: () { y: string = 5; }
fine :: () { z := 1 + 2; print("%\n", z); }
Broken :: struct { field: Nope; }
user :: (b: Broken) { b.field = 1; }
main :: () { first(); second(); fine(); }
"#;

fn checks(s: &Session) -> Vec<(u32, String)> {
    s.diagnostics(&uri())
        .unwrap()
        .into_iter()
        .filter(|d| d.code == DiagnosticCode::Check)
        .map(|d| (d.range.start.line, d.message))
        .collect()
}

#[test]
fn independent_errors_are_all_reported() {
    let mut s = session();
    s.open(uri(), 1, TWO_BODIES.into()).unwrap();
    let found = checks(&s);
    let mut lines: Vec<u32> = found.iter().map(|(l, _)| *l).collect();
    lines.sort();
    lines.dedup();
    assert_eq!(lines, [1, 2, 4], "{found:?}");
}

fn messages(json: &mut JsonSession, message: serde_json::Value) -> Vec<serde_json::Value> {
    json.handle_json(&message.to_string())
        .unwrap()
        .iter()
        .map(|m| serde_json::from_str(m).unwrap())
        .collect()
}

fn initialized(capabilities: serde_json::Value) -> (JsonSession, serde_json::Value) {
    let mut json = JsonSession::with_environment(Limits::default(), environment());
    let init = messages(
        &mut json,
        serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"capabilities": capabilities}}),
    );
    (json, init[0]["result"]["capabilities"].clone())
}

fn open(json: &mut JsonSession, text: &str) -> Vec<serde_json::Value> {
    messages(
        json,
        serde_json::json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {
            "textDocument": {"uri": uri().as_str(), "languageId": "jai", "version": 1, "text": text}}}),
    )
}

#[test]
fn pull_diagnostics_replace_pushed_ones_for_clients_that_pull() {
    let (mut json, capabilities) = initialized(serde_json::json!({
        "textDocument": {"diagnostic": {"dynamicRegistration": false}},
        "workspace": {"diagnostics": {"refreshSupport": true}}
    }));
    assert_eq!(
        capabilities["diagnosticProvider"]["workspaceDiagnostics"],
        true
    );
    // Nothing is pushed to a client that pulls.
    assert!(open(&mut json, TWO_BODIES).is_empty());
    let report = &messages(
        &mut json,
        serde_json::json!({"jsonrpc": "2.0", "id": 2, "method": "textDocument/diagnostic",
            "params": {"textDocument": {"uri": uri().as_str()}}}),
    )[0]["result"];
    assert_eq!(report["kind"], "full");
    let lines: std::collections::BTreeSet<u64> = report["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["range"]["start"]["line"].as_u64().unwrap())
        .collect();
    assert_eq!(lines, [1, 2, 4].into());
    // The same result id: nothing changed.
    let again = &messages(
        &mut json,
        serde_json::json!({"jsonrpc": "2.0", "id": 3, "method": "textDocument/diagnostic",
            "params": {"textDocument": {"uri": uri().as_str()}, "previousResultId": report["resultId"]}}),
    )[0]["result"];
    assert_eq!(again["kind"], "unchanged");
    // The workspace report covers the open documents.
    let workspace = &messages(
        &mut json,
        serde_json::json!({"jsonrpc": "2.0", "id": 4, "method": "workspace/diagnostic",
            "params": {"previousResultIds": []}}),
    )[0]["result"];
    assert_eq!(workspace["items"][0]["uri"], uri().as_str());
    assert_eq!(workspace["items"][0]["kind"], "full");
    assert_eq!(workspace["items"][0]["version"], 1);
    // A change to the settings asks the client to pull again; its answer is ignored.
    let refresh = messages(
        &mut json,
        serde_json::json!({"jsonrpc": "2.0", "method": "workspace/didChangeConfiguration",
            "params": {"settings": {}}}),
    );
    assert_eq!(refresh[0]["method"], "workspace/diagnostic/refresh");
    let answer = serde_json::json!({"jsonrpc": "2.0", "id": refresh[0]["id"], "result": null});
    assert!(messages(&mut json, answer).is_empty());
}

#[test]
fn clients_that_do_not_pull_get_pushed_diagnostics() {
    let (mut json, capabilities) = initialized(serde_json::json!({}));
    assert!(capabilities.get("diagnosticProvider").is_none());
    let published = open(&mut json, TWO_BODIES);
    assert_eq!(published[0]["method"], "textDocument/publishDiagnostics");
    let request = messages(
        &mut json,
        serde_json::json!({"jsonrpc": "2.0", "id": 2, "method": "textDocument/diagnostic",
            "params": {"textDocument": {"uri": uri().as_str()}}}),
    );
    assert_eq!(request[0]["error"]["code"], -32601);
}

#[test]
fn a_clean_program_has_no_check_errors() {
    let mut s = session();
    let text = "#import \"Basic\";\nmain :: () { print(\"hi\\n\"); }\n";
    s.open(uri(), 1, text.into()).unwrap();
    assert!(checks(&s).is_empty());
}

/// A print whose format string and arguments disagree is a compiler warning: it shows as a
/// warning at the call, and no error.
#[test]
fn a_format_argument_count_mismatch_is_a_warning() {
    let mut s = session();
    let text = "#import \"Basic\";\nmain :: () {\n    print(\"done\\n\", 1);\n}\n";
    s.open(uri(), 1, text.into()).unwrap();
    let found: Vec<_> = s
        .diagnostics(&uri())
        .unwrap()
        .into_iter()
        .filter(|d| d.code == DiagnosticCode::Check)
        .collect();
    assert_eq!(found.len(), 1, "{found:?}");
    // The compiler's warning is the only report: the editor adds none of its own.
    assert_eq!(s.diagnostics(&uri()).unwrap().len(), 1);
    assert_eq!(
        found[0].severity,
        jai_language_server::DiagnosticSeverity::Warning
    );
    assert_eq!(found[0].range.start.line, 2);
    assert!(
        found[0]
            .message
            .contains("incorrect number of arguments supplied to `print`")
    );
}
