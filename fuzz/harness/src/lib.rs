//! Fuzz harnesses for the compiler, shared by the libFuzzer targets in `fuzz/fuzz_targets/` and
//! the regression replay test (`tests/regressions.rs`).
//!
//! Every function takes raw fuzzer bytes and must return normally for every input: compile
//! errors are diagnostics, never panics. The harnesses call the library entry points directly
//! (there is no `catch_unwind` anywhere on these paths), so a panic reaches libFuzzer as a crash.
//!
//! Anything that executes Jai code (`#run`, `main`) goes through the playground's sandbox: the
//! bundled stdlib in a virtual file system, the `SandboxHost` libc (no dynamic linker, no real
//! files, network or clock) and an interpreter block budget, so an input cannot do I/O or loop
//! forever.
pub mod generate;
mod jaifmt;
mod lsp_edits;

pub use jaifmt::jaifmt;
pub use lsp_edits::lsp_edits;

use jai_language_server::{DocumentUri, Limits, Position, Session, TextChange};
use jai_wasm::play::{PlayOptions, run_with};
use jaic::source::FileId;
use std::collections::BTreeMap;

/// Interpreter basic blocks one input may execute (compile time and `main` together). Importing
/// `Basic` and printing costs well under a tenth of this.
pub const BLOCK_BUDGET: u64 = 2_000_000;

/// Stack for the compiling harnesses: the size the WebAssembly build links with, so a stack
/// overflow found here also overflows in the browser. (The native CLI runs on 1 GiB.)
pub const COMPILER_STACK: usize = 256 << 20;

/// Run `f` on a thread with the compiler's stack and forward its panic, if any.
fn on_compiler_stack<R: Send>(f: impl FnOnce() -> R + Send) -> R {
    std::thread::scope(|scope| {
        let handle = std::thread::Builder::new()
            .name("fuzz-compiler".into())
            .stack_size(COMPILER_STACK)
            .spawn_scoped(scope, f)
            .expect("spawn compiler thread");
        match handle.join() {
            Ok(value) => value,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    })
}

/// Arbitrary bytes into the lexer (invalid UTF-8 replaced, as a reader of the file would).
pub fn lexer(data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    let _ = jaic::lexer::lex(FileId(0), &text);
}

/// Bytes through the lexer and parser.
pub fn parser(data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    let _ = jaic::parser::parse_file(FileId(0), &text);
}

fn play(source: &[u8], compile_only: bool) {
    let mut files = BTreeMap::new();
    files.insert("main.jai".to_string(), source.to_vec());
    let options = PlayOptions {
        budget: Some(BLOCK_BUDGET),
        compile_only,
        styled: false,
    };
    let result = on_compiler_stack(|| run_with(&files, "main.jai", options));
    if std::env::var_os("JAI_FUZZ_VERBOSE").is_some() {
        eprintln!(
            "{}{:?}\n{}",
            result.rendered, result.diagnostics, result.stdout
        );
    }
}

/// The whole front end on a single-file program (raw bytes, so invalid UTF-8 reaches the
/// compiler's own reader) with the real stdlib: parsing, every `#run`, type checking and
/// lowering, but not `main`.
pub fn check(data: &[u8]) {
    play(data, true);
}

/// [`check`], then `main` under the remaining interpreter budget.
pub fn interp(data: &[u8]) {
    play(data, false);
}

/// A program from the grammar-based generator, compiled and run like [`interp`]. Set
/// `JAI_FUZZ_VERBOSE` to print the generated source.
pub fn generated(data: &[u8]) {
    let source = generate::program(data);
    if std::env::var_os("JAI_FUZZ_VERBOSE").is_some() {
        eprintln!("{source}");
    }
    play(source.as_bytes(), false);
}

/// The LSP position of byte `at` (snapped to a character boundary) in `text`.
fn position(text: &str, mut at: usize) -> Position {
    at = at.min(text.len());
    while !text.is_char_boundary(at) {
        at -= 1;
    }
    let line_start = text[..at].rfind('\n').map_or(0, |i| i + 1);
    Position {
        line: text[..line_start].matches('\n').count() as u32,
        character: text[line_start..at].encode_utf16().count() as u32,
    }
}

/// Byte offsets to query in `text`: spread over the document, the end of every word in the
/// first stretch, and one past the end.
fn probe_offsets(text: &str) -> Vec<usize> {
    let mut out: Vec<usize> = (0..=16).map(|i| text.len() * i / 16).collect();
    let bytes = text.as_bytes();
    out.extend(
        (1..bytes.len().min(2048))
            .filter(|&i| {
                (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_')
                    && !(bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_')
            })
            .take(48),
    );
    out.sort_unstable();
    out.dedup();
    out
}

fn query_all(session: &Session, uri: &DocumentUri, text: &str) {
    let _ = session.diagnostics(uri);
    let _ = session.document_symbols(uri);
    let _ = session.semantic_tokens(uri);
    let offsets = probe_offsets(text);
    for &at in &offsets {
        let p = position(text, at);
        let _ = session.hover(uri, p);
        let _ = session.definition(uri, p);
    }
    // Completion compiles the text with the word at the cursor cut out, a fresh compile per
    // offset, so it gets a few offsets only (libFuzzer needs many executions per second).
    let step = offsets.len().div_ceil(4).max(1);
    for &at in offsets.iter().skip(step / 2).step_by(step) {
        let _ = session.completion(uri, position(text, at));
    }
    // Positions past the end of a line and past the end of the document are errors, not panics.
    let last = position(text, text.len());
    for p in [
        Position {
            line: last.line,
            character: last.character + 1,
        },
        Position {
            line: last.line + 1,
            character: 0,
        },
        Position {
            line: 0,
            character: u32::MAX,
        },
    ] {
        let _ = session.hover(uri, p);
        let _ = session.completion(uri, p);
    }
}

/// A document opened in a type-checking session (the playground's bundled stdlib), queried for
/// hover, definition and completion at many offsets, then edited and queried again.
pub fn lsp(data: &[u8]) {
    let text = String::from_utf8_lossy(data).into_owned();
    on_compiler_stack(|| {
        let mut session = Session::with_environment(Limits::default(), jai_wasm::lsp_environment());
        let uri = DocumentUri::parse("file:///workspace/main.jai").expect("valid uri");
        if session.open(uri.clone(), 1, text.clone()).is_err() {
            return;
        }
        query_all(&session, &uri, &text);
        // An edit typical of typing: cut the middle third and insert a member access there.
        let (a, b) = (
            position(&text, text.len() / 3),
            position(&text, text.len() * 2 / 3),
        );
        let change = TextChange {
            range: Some(jai_language_server::Range {
                start: a,
                end: b,
            }),
            range_length: None,
            text: "x.".to_string(),
        };
        if session.change(&uri, 2, &[change]).is_ok()
            && let Ok(edited) = session.document_text(&uri).map(str::to_owned)
        {
            query_all(&session, &uri, &edited);
        }
    });
}

/// Newline-separated JSON-RPC messages into the language server's JSON session, after a
/// standard `initialize` handshake.
pub fn lsp_json(data: &[u8]) {
    let Ok(text) = std::str::from_utf8(data) else {
        // The framing layer sees raw bytes.
        let mut decoder = jai_language_server::framing::FrameDecoder::new(1 << 20);
        let _ = decoder.push(data);
        let _ = decoder.finish();
        return;
    };
    on_compiler_stack(|| {
        let mut session = jai_language_server::JsonSession::with_environment(
            Limits::default(),
            jai_wasm::lsp_environment(),
        );
        let _ = session.handle_json(
            r#"{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"capabilities":{}}}"#,
        );
        let _ = session.handle_json(r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#);
        for line in text.split('\n') {
            let _ = session.handle_json(line);
        }
    });
}
