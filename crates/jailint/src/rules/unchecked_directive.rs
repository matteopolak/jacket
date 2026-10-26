//! `unchecked_directive`: `#no_abc`, `#no_aoc` or `#asm` with no comment saying why.
//!
//! ```jai
//! sum :: (items: []int) -> int #no_abc {
//!     ...
//! }
//! ```
//!
//! These directives take the compiler's checks away (bounds, overflow) or leave the language
//! (`#asm`), so the reader has to trust the author. A comment on the same line or on the line
//! just above is the place to say what makes it safe. Reports each directive without one.
//! Generated files are skipped. No fix: the reason is the author's to write.
use super::finding;
use crate::Finding;
use crate::syntax::Cx;
use jaic::lexer::Tok;
use jaic::source::Span;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "unchecked_directive",
    category: crate::config::Category::Restriction,
    summary: "`#no_abc`, `#no_aoc` or `#asm` with no comment saying why",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    if cx.generated() {
        return;
    }
    for t in cx.tokens {
        let Tok::Directive(d) = &t.tok else {
            continue;
        };
        let name = d.as_str();
        if !matches!(name, "no_abc" | "no_aoc" | "asm") || explained(cx, t.span) {
            continue;
        }
        let help = if name == "asm" {
            "add a `//` comment on this line or the line above: what the assembly does that Jai cannot"
        } else {
            "add a `//` comment on this line or the line above: why the checks are not needed here"
        };
        out.push(finding(
            t.span,
            format!("`#{name}` has no comment saying why it is needed"),
            Some(help.into()),
        ));
    }
}

/// A comment on the directive's line, or on the line just above it.
fn explained(cx: &Cx, at: Span) -> bool {
    let text = cx.text;
    let (line, _) = cx.line_start(at.start as usize);
    let line_end = text[line..].find('\n').map_or(text.len(), |n| line + n);
    if cx.has_comment_between(line, line_end) {
        return true;
    }
    if line == 0 {
        return false;
    }
    let (above, _) = cx.line_start(line - 1);
    cx.has_comment_between(above, line - 1)
}
