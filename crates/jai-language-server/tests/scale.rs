//! Large inputs and compile-time code that cannot finish: the server answers instead of stalling,
//! dropping the document or dying.
use jai_language_server::{DocumentUri, JsonSession, Limits, Position};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use test_support::lsp::{environment, file_uri, session};

/// Focus's unresolved parameterized import is exercised only through the bench sandbox.
/// Missing optional corpus keeps the existing skip; enforced runs require the full fixture.
#[test]
fn project_with_an_unresolvable_import_still_answers() {
    assert!(
        !jacket_config::env::SCALE_SANDBOX_INPUTS.is_set(),
        "selected-input markers cannot authorize a child; use the mandatory bench launcher"
    );
    let enforce = jacket_config::env::ENFORCE_SCALE_BUDGET.get().as_deref() == Some("1");
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    let root = repo.join("corpus/upstream/focus-editor--focus/src/main.jai");
    if !root.is_file() {
        assert!(
            !enforce,
            "JAILSP_ENFORCE_SCALE_BUDGET=1 needs corpus/upstream/focus-editor--focus (python3 tools/fetch_upstreams.py --only focus-editor/focus)"
        );
        test_support::toolchain::skip("corpus/upstream/focus-editor--focus is not fetched");
        return;
    }
    let status = std::process::Command::new(if cfg!(windows) {
        "python"
    } else {
        "python3"
    })
    .env("PYTHONPATH", repo.join("tools"))
    .arg(repo.join("tools/lsp/scale_sandbox.py"))
    .arg("--worker")
    .arg(scale_worker())
    .arg("--stdlib")
    .arg(repo.join("stdlib"))
    .arg("--source")
    .arg(root)
    .status()
    .expect("start the mandatory upstream scale sandbox");
    assert!(
        status.success(),
        "sandboxed upstream scale case failed: {status}"
    );
}

// This former selection channel is deliberately rejected, rather than an alternate entry.
const OBSOLETE_SELECTED_INPUTS: &str = jacket_config::env::SCALE_SANDBOX_INPUTS.name;

/// The private example is built explicitly for focused tests and the required CI step.
fn scale_worker() -> PathBuf {
    let executable = std::env::current_exe().unwrap();
    find_scale_worker(&executable).expect("build the private worker with cargo build -p jai-language-server --example lsp_scale_worker")
}

/// Cargo may nest integration artifacts beneath build/package/hash/out rather than deps.
fn find_scale_worker(executable: &std::path::Path) -> Option<PathBuf> {
    let name = format!("lsp_scale_worker{}", std::env::consts::EXE_SUFFIX);
    executable
        .ancestors()
        .skip(1)
        .take(8)
        .map(|directory| directory.join("examples").join(&name))
        .find(|candidate| candidate.is_file())
}

#[test]
fn worker_lookup_accepts_standard_and_nested_cargo_layouts() {
    let temp = test_support::TempDir::new("scale-worker-artifact-layout");
    let profile = temp.path().join("debug");
    let worker = profile
        .join("examples")
        .join(format!("lsp_scale_worker{}", std::env::consts::EXE_SUFFIX));
    std::fs::create_dir_all(worker.parent().unwrap()).unwrap();
    std::fs::write(&worker, "owned locator fixture").unwrap();
    for relative in ["deps/scale-hash", "build/package/hash/out/scale-hash"] {
        assert_eq!(
            find_scale_worker(&profile.join(relative)),
            Some(worker.clone())
        );
    }
}

/// Neither malformed nor valid legacy payloads grant a path around the registered launcher.
#[test]
fn selected_input_markers_are_rejected_before_corpus_discovery() {
    for payload in ["not-json", "[]", "[1,2]", "[\"relative.jai\",\"relative\"]"] {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "project_with_an_unresolvable_import_still_answers",
                "--nocapture",
            ])
            .env(OBSOLETE_SELECTED_INPUTS, payload)
            .output()
            .unwrap();
        assert!(
            !output.status.success(),
            "markers cannot authorize an unconfined child"
        );
        assert!(String::from_utf8_lossy(&output.stderr).contains("cannot authorize"));
        assert!(!String::from_utf8_lossy(&output.stderr).contains("Focus coverage:"));
    }
}

/// Valid path metadata is input selection, never authority to compile outside confinement.
#[test]
fn selected_inputs_cannot_authorize_an_unconfined_scale_child() {
    let temp = test_support::TempDir::new("scale-unconfined-selection");
    let source = temp.path().join("src/main.jai");
    std::fs::create_dir_all(source.parent().unwrap()).unwrap();
    // This is an owned synthetic program, not upstream corpus content.
    std::fs::write(&source, "main :: () { missing_owned_name; }\n").unwrap();
    let selected = serde_json::json!([source.canonicalize().unwrap(), test_support::lsp::stdlib()]);
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "project_with_an_unresolvable_import_still_answers",
            "--nocapture",
        ])
        .env(OBSOLETE_SELECTED_INPUTS, selected.to_string())
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "valid selected paths bypassed the mandatory sandbox"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot authorize"));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("Focus coverage:"));
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
    file_uri("/lsp-scale-test/big.jai")
}

/// A document of 700 KiB, 18,000 tokens past the old caps (256 KiB, 8192 tokens, 1024
/// declarations) opens, is analysed in full and answers.
#[test]
fn large_document_opens_and_answers() {
    let text = generated(13_000);
    assert!(text.len() > 600 * 1024);
    let mut s = session();
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
    assert_ne!(
        s.semantic_tokens(&big_uri()).unwrap(),
        [] as [jai_language_server::SemanticToken; 0]
    );
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

/// Names followed by `(` that the check recorded nothing for (the `trunc` of `cast,trunc(u8)`)
/// are looked up together: the file is parsed once for all of them, not once for each, which took
/// a minute for 25,000 lines.
#[test]
fn semantic_tokens_of_many_unchecked_calls_parse_the_file_once() {
    // Ten times the calls should take about ten times as long. Parsing the file again per call made
    // it about a hundred. Comparing two sizes keeps the test steady on slow or busy machines.
    let time = |count: usize| {
        let mut text = String::new();
        for i in 0..count {
            text.push_str(&format!(
                "narrow_{i} :: (a: int) -> u8 {{ return cast,trunc(u8) a; }}\n"
            ));
        }
        let uri = file_uri(&format!("/lsp-scale-test/unchecked-calls-{count}.jai"));
        let mut s = session();
        s.open(uri.clone(), 1, text).unwrap();
        let start = Instant::now();
        let tokens = s.semantic_tokens(&uri).unwrap();
        let elapsed = start.elapsed();
        assert!(tokens.len() > count, "{} tokens", tokens.len());
        elapsed
    };
    let small = time(300);
    let large = time(3_000);
    assert!(
        large < Duration::from_secs(1) || large < small * 40,
        "300 calls took {small:?}, 3000 took {large:?}"
    );
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
