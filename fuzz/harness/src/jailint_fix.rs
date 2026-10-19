//! `jailint --fix` as the language server applies it (`source.fixAll.jailint`), checked on any
//! program that parses:
//!
//! - the fixed text still parses, and applying the fixes settles (a fresh pass finds none);
//! - the fixes add no compile errors to a program that had some;
//! - a program that compiled cleanly still compiles and prints the same (digits blanked, since
//!   line numbers move) with the same exit code.
//!
//! A failed check panics. Text that does not parse is skipped, as the linter skips it.
use jai_language_server::lints::{ActionContext, FIX_ALL_KIND};
use jai_language_server::{DocumentUri, Limits, Position, Range, Session, TextChange, TextEdit};

/// Passes before the fixes are called unsettled (`jailint --fix` makes five).
const MAX_PASSES: usize = 8;

/// Byte offset of an LSP position (UTF-16 columns) in `text`.
pub(crate) fn offset(text: &str, at: Position) -> usize {
    let mut start = 0;
    for _ in 0..at.line {
        match text[start..].find('\n') {
            Some(i) => start += i + 1,
            None => return text.len(),
        }
    }
    let line_end = text[start..].find('\n').map_or(text.len(), |i| start + i);
    let mut units = 0;
    for (i, c) in text[start..line_end].char_indices() {
        if units >= at.character {
            return start + i;
        }
        units += c.len_utf16() as u32;
    }
    line_end
}

pub(crate) fn apply(text: &str, edits: &[TextEdit]) -> String {
    let mut spans: Vec<(usize, usize, &str)> = edits
        .iter()
        .map(|e| {
            (
                offset(text, e.range.start),
                offset(text, e.range.end),
                e.new_text.as_str(),
            )
        })
        .collect();
    spans.sort_by_key(|s| std::cmp::Reverse((s.0, s.1)));
    let mut out = text.to_string();
    for (a, b, new) in spans {
        out.replace_range(a..b.max(a), new);
    }
    out
}

pub fn jailint_fix(data: &[u8]) {
    let Ok(original) = std::str::from_utf8(data) else {
        return;
    };
    if original.contains('\r')
        || jaic::parser::parse_file(jaic::source::FileId(0), original).is_err()
    {
        return;
    }
    let fixed = crate::on_compiler_stack(|| fix_all(original));
    let Some(fixed) = fixed else {
        return;
    };
    if std::env::var_os("JAI_FUZZ_VERBOSE").is_some() {
        eprintln!("--- fixed ---\n{fixed}");
    }
    assert!(
        jaic::parser::parse_file(jaic::source::FileId(0), &fixed).is_ok(),
        "the fixed text does not parse:\n{fixed}"
    );
    let (before, after) = (
        crate::Meaning::of(data),
        crate::Meaning::of(fixed.as_bytes()),
    );
    if before.errors_empty() {
        assert!(
            before == after,
            "the fixes change what the program does:\n{before:?}\n{after:?}\n{fixed}"
        );
    } else {
        // Removing an unused import that fails to resolve trades its error for the uses'.
        assert!(
            after.error_count() <= before.error_count(),
            "the fixes add compile errors:\n{before:?}\n{after:?}\n{fixed}"
        );
    }
}

/// `original` with every safe fix applied until none is left; `None` when there was nothing to
/// fix (or the session refused the text).
fn fix_all(original: &str) -> Option<String> {
    let mut session = Session::with_environment(Limits::default(), jai_wasm::lsp_environment());
    let uri = DocumentUri::parse("file:///workspace/main.jai").expect("valid uri");
    session.open(uri.clone(), 1, original.to_string()).ok()?;
    let context = ActionContext {
        only: Some(vec![FIX_ALL_KIND.to_string()]),
        diagnostics: Vec::new(),
    };
    let mut text = original.to_string();
    for version in 2..2 + MAX_PASSES as i32 {
        let end = Position {
            line: text.matches('\n').count() as u32,
            character: text
                .rsplit('\n')
                .next()
                .map_or(0, |l| l.encode_utf16().count() as u32),
        };
        let actions = session
            .code_actions_in(
                &uri,
                Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end,
                },
                &context,
            )
            .ok()?;
        let Some((_, edits)) = actions.into_iter().find_map(|a| a.edit) else {
            return (text != original).then_some(text);
        };
        text = apply(&text, &edits);
        session
            .change(
                &uri,
                version,
                &[TextChange {
                    range: None,
                    range_length: None,
                    text: text.clone(),
                }],
            )
            .ok()?;
    }
    panic!("the fixes do not settle in {MAX_PASSES} passes:\n{text}");
}
