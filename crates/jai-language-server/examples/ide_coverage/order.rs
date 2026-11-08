//! Reproducible file-open orders and observations before and after the other files open.
use jai_language_server::{DocumentUri, Position, Range, Session};
use serde_json::{Value, json};
use std::io::Write;
use std::path::Path;

/// Fisher–Yates with an explicit seed; zero retains canonical order for the baseline.
pub fn shuffle<T>(values: &mut [T], mut seed: u64) {
    if seed == 0 {
        return;
    }
    for i in (1..values.len()).rev() {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        values.swap(i, (seed % (i as u64 + 1)) as usize);
    }
}

fn answer<T: std::fmt::Debug>(value: Result<T, jai_language_server::Error>) -> Value {
    match value {
        Ok(v) => json!(format!("{v:?}")),
        Err(e) => json!({"error": format!("{e:?}")}),
    }
}

/// Record exact-response fingerprints, never source text, for all files in each open order.
/// At most 64 identifier positions per file are compared; the normal audit still queries every identifier.
pub fn checkpoint(
    path: Option<&Path>,
    details: Option<&Path>,
    phase: &str,
    id: &str,
    session: &Session,
    uri: &DocumentUri,
    text: &str,
) {
    let Some(path) = path else {
        return;
    };
    let diagnostics = answer(session.diagnostics(uri));
    let hints = answer(session.inlay_hints(
        uri,
        Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: text.lines().count() as u32 + 1,
                character: 0,
            },
        },
    ));
    let symbols = answer(session.document_symbols(uri));
    let mut references = Vec::new();
    let mut completions = Vec::new();
    if let Ok(tokens) = jaic::lexer::lex(jaic::source::FileId(0), text) {
        let identifiers: Vec<_> = tokens.iter().filter(|t| matches!(&t.tok, jaic::lexer::Tok::Ident(n) if !jaic::lexicon::is_reserved(n.as_str()))).collect();
        for token in identifiers
            .iter()
            .step_by(identifiers.len().div_ceil(64).max(1))
        {
            let at = super::position(
                text,
                &jaic::source::LineIndex::new(text),
                token.span.start as usize,
            );
            references.push(json!([
                token.span.start,
                answer(session.hover(uri, at)),
                answer(session.definition(uri, at))
            ]));
            if token.span.start > 0 && text.as_bytes()[token.span.start as usize - 1] == b'.' {
                let response = match session.completion(uri, at) {
                    Ok(list) => {
                        let mut items: Vec<_> = list
                            .items
                            .iter()
                            .map(|i| format!("{}:{:?}:{:?}", i.label, i.kind, i.detail))
                            .collect();
                        items.sort();
                        json!({"items":items,"incomplete":list.is_incomplete})
                    }
                    Err(e) => json!({"error":format!("{e:?}")}),
                };
                completions.push(json!([token.span.start, response]));
            }
        }
    }
    let semantic = session.semantic_coverage(uri).ok().flatten().map(|s| {
        json!({
            "loaded":s.source_loaded,"complete":s.program_complete,"errors":s.program_errors,
            "references":s.recorded_references,"fields":s.recorded_fields,"target":s.target,
            "analysis":s.analysis,
        })
    });
    if let Some(path) = details {
        let row = json!({"input_id":id,"phase":phase,"redacted":false,
            "diagnostics":diagnostics,"inlay_hints":hints,"symbols":symbols,
            "references":references,"completions":completions});
        let mut out = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .expect("observation details");
        writeln!(out, "{row}").expect("observation detail record");
    }
    let fingerprint = |value: &Value| super::report::fingerprint(&value.to_string());
    let row = json!({"kind":"observations","phase":phase,"input_id":id,"redacted":true,"compiles":session.compile_counts().0,
        "diagnostics":fingerprint(&diagnostics),"inlay_hints":fingerprint(&hints),
        "symbols":fingerprint(&symbols),"references":fingerprint(&json!(references)),
        "completions":fingerprint(&json!(completions)),"semantic":semantic});
    let mut out = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("progress output");
    writeln!(out, "{row}").expect("progress record");
}
