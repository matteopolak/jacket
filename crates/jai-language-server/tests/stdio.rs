#![cfg(not(target_arch = "wasm32"))]
use jai_language_server::framing::{FrameDecoder, encode};
use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};
#[test]
fn native_stdio_serves_the_same_core_and_shuts_down_cleanly() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_jai-lsp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let requests = [
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
        json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///workspace/main.jai","version":1,"languageId":"jai","text":"answer :: 42; main :: () -> int {return answer;}"}}}),
        json!({"jsonrpc":"2.0","id":2,"method":"textDocument/documentSymbol","params":{"textDocument":{"uri":"file:///workspace/main.jai"}}}),
        json!({"jsonrpc":"2.0","id":3,"method":"shutdown"}),
        json!({"jsonrpc":"2.0","method":"exit"}),
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
