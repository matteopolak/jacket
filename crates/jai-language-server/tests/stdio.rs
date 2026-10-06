#![cfg(not(target_arch = "wasm32"))]
use jai_language_server::framing::{FrameDecoder, encode};
use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};

#[test]
fn native_stdio_serves_the_same_core_and_shuts_down_cleanly() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_jailsp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let requests = [
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} }),
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace/main.jai",
                    "version": 1,
                    "languageId": "jai",
                    "text": "answer :: 42; main :: () -> int {return answer;}",
                },
            },
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/documentSymbol",
            "params": { "textDocument": { "uri": "file:///workspace/main.jai" } },
        }),
        json!({ "jsonrpc": "2.0", "id": 3, "method": "shutdown" }),
        json!({ "jsonrpc": "2.0", "method": "exit" }),
    ];
    for request in requests {
        for part in encode(&request.to_string()).chunks(7) {
            input.write_all(part).unwrap();
        }
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut decoder = FrameDecoder::new(1024 * 1024);
    let messages = decoder
        .push(&output.stdout)
        .unwrap()
        .iter()
        .map(|s| serde_json::from_str::<Value>(s).unwrap())
        .collect::<Vec<_>>();
    decoder.finish().unwrap();
    assert!(
        messages
            .iter()
            .any(|message| message["id"] == 2 && message["result"][0]["name"] == "answer")
    );
    assert!(
        messages
            .iter()
            .any(|message| message["method"] == "textDocument/publishDiagnostics")
    );
}

/// Run by hand, jailsp explains what it is instead of waiting silently or failing obscurely.
#[test]
fn command_line_mistakes_explain_what_jailsp_is() {
    let run = |args: &[&str], input: &[u8]| {
        let mut child = Command::new(env!("CARGO_BIN_EXE_jailsp"))
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(input).unwrap();
        let output = child.wait_with_output().unwrap();
        (
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        )
    };
    let (code, _, stderr) = run(&["main.jai"], b"");
    assert_eq!(code, Some(2));
    assert!(
        stderr.contains("help: to check `main.jai` from a terminal, use `jaic check main.jai`"),
        "{stderr}"
    );
    let (code, _, stderr) = run(&["--bogus"], b"");
    assert_eq!(code, Some(2));
    assert!(
        stderr.starts_with("error: unknown option `--bogus`"),
        "{stderr}"
    );
    let (code, stdout, _) = run(&["--help"], b"");
    assert_eq!(code, Some(0));
    assert!(stdout.starts_with("usage: jailsp"), "{stdout}");
    let (code, _, stderr) = run(&["--stdio"], b"hello\r\n\r\n");
    assert_eq!(code, Some(1));
    assert!(stderr.contains("expected `Content-Length: N`"), "{stderr}");
}
