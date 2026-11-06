#![cfg(not(target_arch = "wasm32"))]
fn encode(body: &str) -> Vec<u8> {
    format!("Content-Length: {}\r\n\r\n{body}", body.len()).into_bytes()
}

use serde_json::json;
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
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "capabilities": {} } }),
        json!({"jsonrpc":"2.0", "method":"initialized", "params":{}}),
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
    let mut reader = std::io::Cursor::new(output.stdout);
    let mut messages = Vec::new();
    while let Some(message) = lsp_server::Message::read(&mut reader).unwrap() {
        messages.push(serde_json::to_value(message).unwrap());
    }
    assert!(
        messages
            .iter()
            .any(|message| message["id"] == 2 && message["result"][0]["name"] == "answer")
    );
    assert!(
        messages
            .iter()
            .any(|message| message["id"] == 3 && message["result"].is_null())
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
        stderr.contains("help: to check `main.jai` from a terminal, use `jaic main.jai -check`"),
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
    assert!(
        stderr.contains("Invalid or truncated LSP frame"),
        "{stderr}"
    );
}

#[test]
fn broken_output_terminates_even_when_client_keeps_input_open() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_jailsp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    drop(child.stdout.take());
    input
        .write_all(&encode(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
        ))
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(!status.success());
            break;
        }
        if std::time::Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("server did not observe a broken output pipe while input remained open");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    drop(input);
}

/// Independent client framing keeps null-ID error receipts visible, since the library's
/// normal Response model intentionally cannot deserialize them.
fn response_values(output: &[u8]) -> Vec<serde_json::Value> {
    use std::io::{BufRead, Read};
    let mut input = std::io::Cursor::new(output);
    let mut values = Vec::new();
    loop {
        let mut header = String::new();
        if input.read_line(&mut header).unwrap() == 0 {
            break;
        }
        let length: usize = header
            .strip_prefix("Content-Length: ")
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        assert!(length <= 1024 * 1024);
        let mut separator = String::new();
        input.read_line(&mut separator).unwrap();
        assert_eq!(separator, "\r\n");
        let mut body = vec![0; length];
        input.read_exact(&mut body).unwrap();
        values.push(serde_json::from_slice(&body).unwrap());
    }
    values
}

#[test]
fn malformed_complete_bodies_recover_in_native_dispatch_order() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_jailsp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    for body in [
        r#"{"jsonrpc":"2.0","id":7,"method":"secret-input",}"#,
        r#"{"jsonrpc":"2.0","id":null,"method":"exit"}"#,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
        r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"unknown"}"#,
        r#"{"jsonrpc":"2.0","id":3,"method":"shutdown","params":null}"#,
        r#"{"jsonrpc":"2.0","id":123,"result":null}"#,
        r#"{"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":123}}"#,
        r#"{"jsonrpc":"2.0","method":"exit"}"#,
        "{",
    ] {
        input.write_all(&encode(body)).unwrap();
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!String::from_utf8_lossy(&output.stderr).contains("secret-input"));
    let values = response_values(&output.stdout);
    assert_eq!(values.len(), 5, "{values:?}");
    assert_eq!(values[0]["id"], serde_json::Value::Null);
    assert_eq!(values[0]["error"]["code"], -32700);
    assert_eq!(values[1]["id"], serde_json::Value::Null);
    assert_eq!(values[1]["error"]["code"], -32600);
    assert_eq!(values[2]["id"], 1);
    assert!(values[2].get("result").is_some());
    assert_eq!(values[3]["id"], 2);
    assert_eq!(values[3]["error"]["code"], -32601);
    assert_eq!(values[4]["id"], 3);
    assert!(values[4].get("result").unwrap().is_null());
}

#[test]
fn shutdown_times_out_with_an_open_input_pipe_and_no_exit() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_jailsp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let started = std::time::Instant::now();
    for body in [
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
        r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}"#,
    ] {
        input.write_all(&encode(body)).unwrap();
    }
    let deadline = started + std::time::Duration::from_secs(40);
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if std::time::Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("shutdown waited indefinitely for an exit notification");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(started.elapsed() >= std::time::Duration::from_secs(30));
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("exit did not follow shutdown"));
    let responses = response_values(&output.stdout);
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[1]["id"], 2);
    assert!(responses[1].get("result").unwrap().is_null());
}

#[test]
fn opt_in_trace_preserves_frame_origins_without_recording_protocol_content() {
    let directory = test_support::TempDir::new("native-redacted-trace");
    let trace_path = directory.path().join("trace.json");
    let bodies = [
        r#"{"jsonrpc":"2.0","id":"PRIVATE_ID","method":"PRIVATE_METHOD",}"#,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
        r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#,
        r#"{"jsonrpc":"2.0","id":null,"result":"PRIVATE_PEER_VALUE"}"#,
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///PRIVATE_PATH/main.jai","version":1,"languageId":"jai","text":"PRIVATE_SOURCE :: 42;"}}}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/documentSymbol","params":{"textDocument":{"uri":"file:///PRIVATE_PATH/main.jai"}}}"#,
        r#"{"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":"PRIVATE_CANCEL_ID"}}"#,
        r#"{"jsonrpc":"2.0","id":3,"method":"shutdown","params":null}"#,
        r#"{"jsonrpc":"2.0","method":"exit"}"#,
    ];
    let mut child = Command::new(env!("CARGO_BIN_EXE_jailsp"))
        .env(jacket_config::env::LSP_TRACE.name, "1")
        .env(jacket_config::env::LSP_TRACE_FILE.name, &trace_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    for body in bodies {
        input.write_all(&encode(body)).unwrap();
    }
    drop(input);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while child.try_wait().unwrap().is_none() {
        if std::time::Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("traced native server failed to finish after closed input");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let replies = response_values(&output.stdout);
    assert!(
        replies
            .iter()
            .any(|reply| reply["id"] == 2 && reply["result"][0]["name"] == "PRIVATE_SOURCE")
    );
    let serialized = std::fs::read_to_string(trace_path).expect("enabled trace artifact");
    assert!(
        !serialized.contains("PRIVATE_"),
        "trace may retain only enum categories, timing and counts"
    );
    let trace: serde_json::Value = serde_json::from_str(&serialized).unwrap();
    let dropped = trace["dropped"].as_u64().unwrap() as usize;
    if dropped != 0 {
        assert_eq!(trace["incomplete"], true);
    }
    let records = trace["records"].as_array().unwrap();
    let decoded: Vec<_> = records
        .iter()
        .filter(|row| row["stage"] == "decoded")
        .collect();
    let mut missing = bodies.len() - decoded.len();
    let mut input_records = vec![None; bodies.len()];
    for row in &decoded {
        let candidates: Vec<_> = bodies
            .iter()
            .enumerate()
            .filter(|(_, body)| row["bytes"] == body.len())
            .collect();
        assert_eq!(
            candidates.len(),
            1,
            "fixture frame sizes must identify inputs independently"
        );
        let at = candidates[0].0;
        assert!(input_records[at].replace(*row).is_none());
    }
    let outputs: Vec<_> = records
        .iter()
        .filter(|row| row["stage"] == "output_enqueue_start")
        .collect();
    let mut seen_outputs = std::collections::BTreeSet::new();
    let mut seen_replies = std::collections::BTreeSet::new();
    for output in &outputs {
        assert!(seen_outputs.insert(output["output_ordinal"].as_u64().unwrap()));
        let candidates: Vec<_> = replies
            .iter()
            .enumerate()
            .filter(|(_, reply)| output["bytes"] == serde_json::to_vec(reply).unwrap().len())
            .collect();
        assert_eq!(
            candidates.len(),
            1,
            "fixture output sizes must identify replies independently"
        );
        let (reply_at, reply) = candidates[0];
        assert!(
            seen_replies.insert(reply_at),
            "a reply cannot acquire duplicate enqueue rows"
        );
        let expected_frame = match reply.get("id") {
            Some(serde_json::Value::Null) => Some(0),
            Some(id) if *id == 1 => Some(1),
            Some(id) if *id == 2 => Some(5),
            Some(id) if *id == 3 => Some(7),
            None => None,
            _ => panic!("unexpected response"),
        };
        if let Some(frame) = expected_frame.and_then(|at| input_records[at]) {
            assert_eq!(output["origin"], frame["origin"]);
        }
        for stage in ["write_start", "write_done"] {
            let matching: Vec<_> = records
                .iter()
                .filter(|row| {
                    row["stage"] == stage && row["output_ordinal"] == output["output_ordinal"]
                })
                .collect();
            assert!(matching.len() <= 1);
            if let Some(row) = matching.first() {
                assert_eq!(row["origin"], output["origin"]);
                assert_eq!(row["bytes"], output["bytes"]);
            } else {
                missing += 1;
            }
        }
    }
    missing += replies.len() - seen_replies.len();
    let mut write_rows = std::collections::BTreeSet::new();
    for row in records
        .iter()
        .filter(|row| row["stage"] == "write_start" || row["stage"] == "write_done")
    {
        assert!(write_rows.insert((
            row["output_ordinal"].as_u64().unwrap(),
            row["stage"].as_str().unwrap()
        )));
    }
    for frame in [2, 3, 6, 8] {
        if let Some(input) = input_records[frame] {
            assert!(
                !outputs.iter().any(|row| row["origin"] == input["origin"]),
                "zero-output input cannot acquire a fabricated reply"
            );
        }
    }
    for input in decoded {
        for stage in ["dispatch_start", "dispatch_end"] {
            let count = records
                .iter()
                .filter(|row| row["stage"] == stage && row["origin"] == input["origin"])
                .count();
            assert!(count <= 1);
            missing += 1 - count;
        }
    }
    // Native producers can contend on the deliberately nonblocking recorder. Every absent
    // checked event must be covered by explicit loss, never mistaken for a complete trace.
    assert!(missing <= dropped, "missing={missing}, dropped={dropped}");
}

#[test]
fn trace_destination_is_fresh_and_startup_errors_do_not_disclose_it() {
    let directory = test_support::TempDir::new("trace-fresh-destination");
    let destination = directory.path().join("PRIVATE_TRACE_DESTINATION");
    std::fs::write(&destination, b"existing contents").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_jailsp"))
        .env(jacket_config::env::LSP_TRACE.name, "1")
        .env(jacket_config::env::LSP_TRACE_FILE.name, &destination)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("trace"), "{error}");
    assert!(!error.contains("PRIVATE_TRACE_DESTINATION"));
    assert_eq!(std::fs::read(destination).unwrap(), b"existing contents");
}

#[test]
fn traced_exit_with_open_input_reports_an_incomplete_reader_tail() {
    traced_open_input_tail(false);
}

#[test]
fn traced_broken_output_records_write_failure_without_waiting_for_input() {
    traced_open_input_tail(true);
}

fn traced_open_input_tail(broken_output: bool) {
    let directory = test_support::TempDir::new("trace-open-input");
    let destination = directory.path().join("trace.json");
    let mut child = Command::new(env!("CARGO_BIN_EXE_jailsp"))
        .env(jacket_config::env::LSP_TRACE.name, "1")
        .env(jacket_config::env::LSP_TRACE_FILE.name, &destination)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    if broken_output {
        drop(child.stdout.take());
    }
    input
        .write_all(&encode(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
        ))
        .unwrap();
    if !broken_output {
        for body in [
            r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"shutdown"}"#,
            r#"{"jsonrpc":"2.0","method":"exit"}"#,
        ] {
            input.write_all(&encode(body)).unwrap();
        }
    }
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while child.try_wait().unwrap().is_none() {
        if std::time::Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("trace export must not join an open-input reader");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    // Retain the live client input through process exit: reader quiescence is unproven.
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.success(), !broken_output);
    let trace: serde_json::Value =
        serde_json::from_slice(&std::fs::read(destination).unwrap()).unwrap();
    assert_eq!(trace["incomplete"], true);
    if broken_output {
        let failure_recorded = trace["records"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["stage"] == "write_failed");
        assert!(
            failure_recorded || trace["dropped"].as_u64().unwrap() >= 1,
            "the required failure event is recorded or explicitly lost"
        );
    }
}
