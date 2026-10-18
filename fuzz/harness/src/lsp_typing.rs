//! The typing-session checker (`crates/jai-language-server/examples/support/typing_session.rs`)
//! with the fuzzer's bytes as its choices: a document is opened, then edited the way someone types
//! (keystrokes, backspacing, retyped lines, pastes, unclosed brackets and strings, undo) with
//! requests in between, and every answer is checked against a fresh session on the same text and
//! against the original answers once the text is back. A failed check panics.
//!
//! The input is a document, then optionally a NUL byte and the bytes that choose the edits;
//! without a NUL the choices come from the document itself, so every seed still drives edits.
//! The checks that need an edit mapped through the text (`Config::outside`) are off here: they
//! have known cases (code another target compiles, compile-time code reading its own text) that
//! a fuzzer would only rediscover.
use arbitrary::Unstructured;
use jai_language_server::DocumentUri;
use std::time::Duration;

#[path = "../../../crates/jai-language-server/examples/support/typing_session.rs"]
#[allow(dead_code, clippy::pedantic)]
mod typing_session;

use typing_session::{Choices, Config, Input};

/// Choices read from the fuzzer's bytes. Past the end every choice is the first, and the session
/// is told it is exhausted so it stops opening scenarios.
struct Bytes<'a>(Unstructured<'a>);

impl Choices for Bytes<'_> {
    fn below(&mut self, n: usize) -> usize {
        self.0.int_in_range(0..=n - 1).unwrap_or(0)
    }

    fn exhausted(&self) -> bool {
        self.0.is_empty()
    }
}

/// The second document: loaded by the first when the fuzzer's text says `#load "b.jai";`.
const LIBRARY: &str = "helper :: (a: int) -> int {\n    b := a + 1;\n    return b;\n}\n";

pub fn lsp_typing(data: &[u8]) {
    let (document, script) = match data.iter().position(|&b| b == 0) {
        Some(at) => (&data[..at], data[at + 1..].to_vec()),
        None => (data, data.iter().rev().copied().collect()),
    };
    let text = String::from_utf8_lossy(document).into_owned();
    let failures = super::on_compiler_stack(move || {
        let main = DocumentUri::parse("file:///workspace/main.jai").expect("valid uri");
        let library = DocumentUri::parse("file:///workspace/b.jai").expect("valid uri");
        let environment = jai_wasm::lsp_environment;
        let input = Input {
            setup: Vec::new(),
            docs: vec![(main, text), (library, LIBRARY.to_string())],
            environment: &environment,
        };
        let config = Config {
            scenarios: 2,
            diff_every: 4,
            probes: 4,
            full_probes: 12,
            budget: Duration::from_secs(15),
            hang: None,
            outside: false,
            max_failures: 1,
        };
        typing_session::run(&input, config, Bytes(Unstructured::new(&script))).failures
    });
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
