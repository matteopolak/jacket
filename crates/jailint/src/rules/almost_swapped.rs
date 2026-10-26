//! `almost_swapped`: `a = b; b = a;`.
//!
//! Meant as a swap, this sets both to `b`: the first assignment overwrites `a` before the
//! second one reads it. A swap needs a temporary, or Jai's parallel assignment `a, b = b, a;`.
//!
//! Fires on two consecutive statements `x = y;` and `y = x;` with side-effect-free sides
//! (names, member paths, indexing; no calls), spelled the same, of the same type. Between
//! two types the pair is a conversion demo or round trip (`u = handle; handle = u;`), not a
//! swap.
use super::{finding, same_pure, statement_lists, statement_range, suggesting};
use crate::syntax::Cx;
use crate::{Edit, Finding};
use jaic::ast::{AssignOp, Expr, Stmt, StmtKind as S};

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "almost_swapped",
    category: crate::config::Category::Correctness,
    summary: "`a = b; b = a;`, a swap that sets both to `b`",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        statement_lists(&p.body.stmts, &mut |list| {
            for pair in list.windows(2) {
                let (Some((a, b)), Some((c, d))) = (simple(&pair[0]), simple(&pair[1])) else {
                    continue;
                };
                if same_pure(cx, a, d)
                    && same_pure(cx, b, c)
                    && !same_pure(cx, a, b)
                    && cx.ty(a).is_some()
                    && cx.ty(a) == cx.ty(b)
                {
                    let x = cx.whole_src(a.span).trim();
                    let y = cx.whole_src(b.span).trim();
                    let span = pair[0].span.to(pair[1].span);
                    let message = format!("this sets both `{x}` and `{y}` to `{y}`");
                    let help = format!("to swap them, write `{x}, {y} = {y}, {x};`");
                    let (start, _) = statement_range(cx, pair[0].span);
                    let (_, end) = statement_range(cx, pair[1].span);
                    // Comments between the two statements would be lost with them.
                    if cx.text[start..end].contains("//") || cx.text[start..end].contains("/*") {
                        out.push(finding(span, message, Some(help)));
                        continue;
                    }
                    out.push(suggesting(
                        span,
                        message,
                        help,
                        "swap the two values",
                        vec![Edit {
                            start,
                            end,
                            text: format!("{x}, {y} = {y}, {x};"),
                        }],
                    ));
                }
            }
        });
    }
}

/// `lhs = rhs;` with one target.
fn simple(s: &Stmt) -> Option<(&Expr, &Expr)> {
    match &s.kind {
        S::Assign {
            op: AssignOp::Assign,
            lhs,
            rhs,
        } => match (lhs.as_slice(), rhs.as_slice()) {
            ([l], [r]) => Some((l, r)),
            _ => None,
        },
        _ => None,
    }
}
