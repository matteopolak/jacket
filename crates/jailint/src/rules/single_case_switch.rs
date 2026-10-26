//! `single_case_switch`: a `switch` with one case is an `if`.
//!
//! ```jai
//! if mode == {
//!     case .VERBOSE; print("loud\n");
//! }
//! ```
//!
//! reads as `if mode == .VERBOSE print("loud\n");`, and a switch that adds only a default,
//! `case; B;`, as the `else` of that `if`.
//!
//! Fires on a `switch` with a single case of one value, and nothing else but a default case.
//! `#complete` switches (which check that every member of an enum is covered), cases that end in
//! `#through`, cases listing several values and `#if` switches are left alone. The editor offers
//! the `if`; it keeps the bodies' text, so `--fix` leaves the indentation to `jaifmt`.
use super::{statement_range, suggesting};
use crate::syntax::Cx;
use crate::{Edit, Finding};
use jaic::ast::{Case, Stmt, StmtKind as S};
use jaic::source::Span;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "single_case_switch",
    category: crate::config::Category::Style,
    summary: "a `switch` with a single case, which is an `if`",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    super::walk_code(cx, &mut |n, _| {
        let Some(s) = n.stmt() else {
            return;
        };
        let S::Switch(sw) = &s.kind else {
            return;
        };
        if sw.complete {
            return;
        }
        // The one case with a value, and the default after or before it.
        let (case, default) = match sw.cases.as_slice() {
            [c] => (c, None),
            [c, d] if d.values.is_empty() => (c, Some(d)),
            [d, c] if d.values.is_empty() => (c, Some(d)),
            _ => return,
        };
        let [value] = case.values.as_slice() else {
            return;
        };
        if case.through || default.is_some_and(|d| d.through) || body_text(cx, case).is_none() {
            return;
        }
        let subject = cx.whole_src(sw.value.span).trim();
        let wanted = cx.whole_src(value.span).trim();
        let (_, indent) = cx.line_start(s.span.start as usize);
        let mut text = format!(
            "if {subject} == {wanted} {{\n{indent}    {}\n{indent}}}",
            body_text(cx, case).unwrap_or_default()
        );
        // `case;` with nothing to do leaves the `if` without an `else`.
        if let Some(body) = default.and_then(|d| body_text(cx, d)) {
            text.push_str(&format!(" else {{\n{indent}    {body}\n{indent}}}"));
        }
        let header = Span::new(cx.file, s.span.start as usize, sw.header.end as usize);
        let mut finding = suggesting(
            header,
            "this `switch` has a single case".into(),
            "write an `if` instead".into(),
            "rewrite as an `if`",
            vec![Edit {
                start: s.span.start as usize,
                end: s.span.end as usize,
                text,
            }],
        );
        if cx.has_comment_between(s.span.start as usize, s.span.end as usize) {
            finding.fix = None;
        }
        out.push(finding);
    });
}

/// The source of a case's statements, from the first to the last; `None` when there are none.
fn body_text<'a>(cx: &Cx<'a>, c: &Case) -> Option<&'a str> {
    let stmts: Vec<&Stmt> = c
        .body
        .iter()
        .filter(|s| !matches!(s.kind, S::Empty))
        .collect();
    let (first, last) = (stmts.first()?, stmts.last()?);
    let (start, end) = (first.span.start as usize, statement_range(cx, last.span).1);
    cx.text.get(start..end)
}
