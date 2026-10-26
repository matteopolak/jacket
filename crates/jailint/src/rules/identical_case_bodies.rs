//! `identical_case_bodies`: two cases of a `switch` that do the same thing.
//!
//! ```jai
//! if kind == {
//!     case .PNG;  return "image";
//!     case .JPEG; return "image";
//!     case .WAV;  return "audio";
//! }
//! ```
//!
//! A case may list several values, so `case .PNG, .JPEG; return "image";` says it once and
//! cannot drift apart when one copy is edited. Off by default: separate cases that happen to
//! agree are sometimes clearer, and a table of results reads well as one case per row.
//!
//! Fires on a case whose body has the same tokens (comments and layout aside) as an earlier,
//! non-empty case of the same `switch`. Default cases, cases that end in `#through` and `#if`
//! switches (which usually differ by platform) are left alone. The editor offers the merge:
//! the later case's values join the earlier one's list, and the later case goes. `--fix`
//! leaves it.
use super::suggesting;
use crate::syntax::Cx;
use crate::{Edit, Finding};
use jaic::ast::{Case, Stmt, StmtKind as S};
use jaic::source::Span;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "identical_case_bodies",
    category: crate::config::Category::Pedantic,
    summary: "two cases of a `switch` with the same body, which could list their values together",
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
        for (i, later) in sw.cases.iter().enumerate() {
            let Some(body) = body_span(cx, later) else {
                continue;
            };
            let Some(earlier) = sw.cases[..i]
                .iter()
                .find(|c| body_span(cx, c).is_some_and(|b| cx.same_token_ranges(b, body)))
            else {
                continue;
            };
            let values = |c: &Case| {
                c.values
                    .iter()
                    .map(|v| cx.whole_src(v.span).trim())
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let (first, second) = (values(earlier), values(later));
            // `case 2`, the part of the case that names its values.
            let values_end = later
                .values
                .last()
                .map_or(later.span.start as usize, |v| cx.whole(v.span).1);
            let header = Span::new(cx.file, later.span.start as usize, values_end);
            let last_earlier = cx.whole(earlier.values[earlier.values.len() - 1].span).1;
            let merged = format!("case {first}, {second};");
            let mut finding = suggesting(
                header,
                format!("`case {second}` does the same as `case {first}`"),
                format!("list the values together: `{merged}`"),
                format!("merge into `{merged}`"),
                vec![
                    Edit {
                        start: last_earlier,
                        end: last_earlier,
                        text: format!(", {second}"),
                    },
                    {
                        let (start, end) = cx.tidy_removal(later.span);
                        Edit {
                            start,
                            end,
                            text: String::new(),
                        }
                    },
                ],
            );
            if cx.has_comment_between(earlier.span.start as usize, later.span.end as usize) {
                finding.fix = None;
            }
            out.push(finding);
        }
    });
}

/// The statements of a case that can be merged, as one span: the case has values, no
/// `#through`, and something to do.
fn body_span(cx: &Cx, c: &Case) -> Option<Span> {
    if c.values.is_empty() || c.through {
        return None;
    }
    let stmts: Vec<&Stmt> = c
        .body
        .iter()
        .filter(|s| !matches!(s.kind, S::Empty))
        .collect();
    let (first, last) = (stmts.first()?, stmts.last()?);
    Some(Span::new(
        cx.file,
        first.span.start as usize,
        last.span.end as usize,
    ))
}
