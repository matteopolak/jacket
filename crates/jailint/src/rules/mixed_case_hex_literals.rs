//! `mixed_case_hex_literals`: `0xDeadbeef`, a hex literal with letters in both cases.
//!
//! ```jai
//! mask := 0xFFfF_00ff;
//! ```
//!
//! Upper and lower case digits mean the same, so mixing them only makes the number harder to
//! read and to search for. Pick one case for the whole literal.
//!
//! Fires on a `0x` literal whose digits `a`-`f` appear both as lower and as upper case. Literals
//! in macros and generated files are skipped. The editor offers the literal in the case most of
//! its letters already use (lower on a tie); the choice is a matter of taste, so `--fix` leaves
//! it.
use super::suggesting;
use crate::syntax::Cx;
use crate::{Edit, Finding};
use jaic::lexer::Tok;
use jaic::source::Span;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "mixed_case_hex_literals",
    category: crate::config::Category::Style,
    summary: "a hex literal with both upper and lower case digits (`0xDeadbeef`)",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    if cx.generated() {
        return;
    }
    for t in cx.tokens {
        if !matches!(t.tok, Tok::Int(_)) {
            continue;
        }
        let text = cx.src(t.span);
        let Some(digits) = text.strip_prefix("0x") else {
            continue;
        };
        let lower = digits.chars().filter(|c| matches!(c, 'a'..='f')).count();
        let upper = digits.chars().filter(|c| matches!(c, 'A'..='F')).count();
        if lower == 0 || upper == 0 || cx.in_macro(t.span.start as usize) {
            continue;
        }
        let one_case = if upper > lower {
            digits.to_ascii_uppercase()
        } else {
            digits.to_ascii_lowercase()
        };
        let fixed = format!("0x{one_case}");
        out.push(suggesting(
            Span::new(cx.file, t.span.start as usize, t.span.end as usize),
            format!("`{text}` mixes upper and lower case digits"),
            format!("write `{fixed}`"),
            format!("write `{fixed}`"),
            vec![Edit {
                start: t.span.start as usize,
                end: t.span.end as usize,
                text: fixed,
            }],
        ));
    }
}
