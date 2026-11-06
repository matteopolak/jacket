//! Framing is consumed by lsp-server; the host adapter enforces allocation/encoding limits.
use jai_language_server::transport::{BoundedReader, Input};
use std::io::{BufReader, Cursor};

fn frame(body: &str) -> Vec<u8> {
    format!("Content-Length: {}\r\n\r\n{body}", body.len()).into_bytes()
}

#[test]
fn fragmented_and_coalesced_frames_count_utf8_bytes() {
    let bodies = [
        r#"{"jsonrpc":"2.0","id":"🦀é","method":"unknown"}"#,
        r#"{"jsonrpc":"2.0","method":"exit"}"#,
    ];
    let wire = bodies
        .iter()
        .flat_map(|body| frame(body))
        .collect::<Vec<_>>();
    let mut reader = BoundedReader::new(BufReader::with_capacity(3, Cursor::new(wire)), 1024);
    for body in bodies {
        let Some(Input::Message {
            message,
            bytes,
        }) = reader.read_frame().unwrap()
        else {
            panic!("valid message")
        };
        assert_eq!(bytes, body.len());
        assert_eq!(
            serde_json::to_value(message).unwrap()["method"],
            serde_json::from_str::<serde_json::Value>(body).unwrap()["method"]
        );
    }
    assert!(reader.read_frame().unwrap().is_none());
}

#[test]
fn conflicting_lengths_unsupported_encoding_and_truncation_are_terminal() {
    for wire in [
        b"Content-Length: 1\r\ncontent-length: 1\r\n\r\na".as_slice(),
        b"Content-Length: 1\r\nContent-Type: application/vscode-jsonrpc; charset=utf-16\r\n\r\na",
        b"Content-Length: 3\r\n\r\na",
        b"Content-Length: 3\r\n",
    ] {
        assert!(
            BoundedReader::new(Cursor::new(wire), 128)
                .read_frame()
                .is_err()
        );
    }
    let wire = b"Content-Length: 1\r\n\r\n\xff";
    assert!(matches!(
        BoundedReader::new(Cursor::new(wire), 128)
            .read_frame()
            .unwrap(),
        Some(Input::Rejected { .. })
    ));
}

#[test]
fn an_oversized_body_is_terminal_before_consumption() {
    let wire = frame(&"x".repeat(300));
    assert!(
        BoundedReader::new(Cursor::new(wire), 128)
            .read_frame()
            .is_err()
    );
}

#[test]
fn a_message_at_the_limit_can_be_followed_by_another() {
    let body = format!(
        r#"{{"jsonrpc":"2.0","method":"unknown","params":{{"text":"{}"}}}}"#,
        "x".repeat(20000)
    );
    let small = r#"{"jsonrpc":"2.0","method":"exit"}"#;
    let wire = [frame(&body), frame(small)].concat();
    let mut reader = BoundedReader::new(BufReader::with_capacity(7, Cursor::new(wire)), body.len());
    assert!(matches!(
        reader.read_frame().unwrap(),
        Some(Input::Message { .. })
    ));
    assert!(matches!(
        reader.read_frame().unwrap(),
        Some(Input::Message { .. })
    ));
    assert!(reader.read_frame().unwrap().is_none());
}
