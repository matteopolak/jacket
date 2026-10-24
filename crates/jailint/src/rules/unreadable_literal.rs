//! `unreadable_literal`: a long number with no `_` separators.
//!
//! ```jai
//! timeout := 1000000;       // one million, or ten?
//! seed := 0xdeadbeef;
//! ```
//!
//! Jai allows `_` anywhere inside a number, and grouping the digits (`1_000_000`,
//! `0xdead_beef`) makes the size visible at a glance. Off by default: the standard library alone
//! has thousands of long constants written without separators.
//!
//! Fires on a literal with no `_` in it that has:
//!
//! - six or more decimal digits (grouped in threes from the right: `1_000_000`);
//! - eight or more hex digits (in fours: `0xdead_beef`);
//! - nine or more binary digits (in fours: `0b1_0000_1111`);
//! - six or more digits before the point of a float (`1_234_567.5`).
//!
//! A literal with any `_` is taken as grouped the way its author wanted. Octal literals, floats
//! with an exponent, literals in macros and generated files are skipped. The fix adds the
//! separators and is machine-applicable.
use super::replacing;
use crate::Finding;
use crate::syntax::Cx;
use jaic::lexer::Tok;
use jaic::source::Span;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    if cx.generated() {
        return;
    }
    for t in cx.tokens {
        let text = cx.src(t.span);
        let grouped = match &t.tok {
            Tok::Int(_) => integer(text),
            Tok::Float(_) => float(text),
            _ => None,
        };
        let Some(grouped) = grouped else {
            continue;
        };
        if cx.in_macro(t.span.start as usize) {
            continue;
        }
        out.push(replacing(
            Span::new(cx.file, t.span.start as usize, t.span.end as usize),
            format!("`{text}` is hard to read without separators"),
            format!("write `{grouped}`"),
            (t.span.start as usize, t.span.end as usize),
            grouped,
            true,
        ));
    }
}

/// The literal with separators, when it has none and is long enough to want them.
fn integer(text: &str) -> Option<String> {
    if text.contains('_') {
        return None;
    }
    if let Some(hex) = text.strip_prefix("0x") {
        return (hex.len() >= 8 && hex.bytes().all(|b| b.is_ascii_hexdigit()))
            .then(|| format!("0x{}", group(hex, 4)));
    }
    if let Some(bits) = text.strip_prefix("0b") {
        return (bits.len() >= 9 && bits.bytes().all(|b| matches!(b, b'0' | b'1')))
            .then(|| format!("0b{}", group(bits, 4)));
    }
    (text.len() >= 6 && text.bytes().all(|b| b.is_ascii_digit())).then(|| group(text, 3))
}

/// `1234567.5` with separators in the part before the point.
fn float(text: &str) -> Option<String> {
    if text.contains(['_', 'e', 'E', 'x', 'h']) {
        return None;
    }
    let (whole, fraction) = text.split_once('.')?;
    (whole.len() >= 6 && whole.bytes().all(|b| b.is_ascii_digit()))
        .then(|| format!("{}.{fraction}", group(whole, 3)))
}

/// `digits` in groups of `size` from the right, separated by `_`.
fn group(digits: &str, size: usize) -> String {
    let first = digits.len() % size;
    let mut out = String::with_capacity(digits.len() + digits.len() / size);
    if first > 0 {
        out.push_str(&digits[..first]);
    }
    for chunk in digits.as_bytes()[first..].chunks(size) {
        if !out.is_empty() {
            out.push('_');
        }
        out.push_str(std::str::from_utf8(chunk).unwrap_or(""));
    }
    out
}
