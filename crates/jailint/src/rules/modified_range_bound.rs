//! `modified_range_bound`: `for 0..n-1 { ...; n -= 1; }`.
//!
//! A range's ends are worked out once, before the first iteration. Changing a variable that
//! bounds the range does not change how many times the loop runs, so code written to shorten
//! or stretch the loop that way does neither.
//!
//! Fires on an assignment (`=` or an operator form like `-=`) to a local variable named directly
//! in either end of a `for a..b` range (`n` in `0..n-1`, not `xs` in `0..xs.count-1`), anywhere
//! in the loop's body. Quiet when a `break` or `return` comes after the assignment, or after the
//! statement holding it, in the same block: the loop is over, so the variable is being given a
//! result (`n = it; break;`, `if found n = it; break;`).
use super::{finding, statement_lists};
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::walk::walk;
use jaic::ast::{Expr, ExprKind as E, ForOver, Stmt, StmtKind as S, UnOp};
use jaic::sema::EntityId;
use jaic::sema::scope::EntityKind;
use jaic::source::Span;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "modified_range_bound",
    category: crate::config::Category::Suspicious,
    summary: "a variable bounding a `for` range assigned in the loop, which does not change it",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.clean || p.is_macro {
            continue;
        }
        // Statements with a `break` or `return` after them in the same block.
        let mut stopping: Vec<Span> = Vec::new();
        statement_lists(&p.body.stmts, &mut |list: &[Stmt]| {
            for (k, s) in list.iter().enumerate() {
                if list[k + 1..]
                    .iter()
                    .any(|t| matches!(t.kind, S::Break(_) | S::Return { .. }))
                {
                    stopping.push(s.span);
                }
            }
        });
        walk(&p.body.stmts, &mut |n, _| {
            let Some(S::For(f)) = n.stmt().map(|s| &s.kind) else {
                return true;
            };
            let ForOver::Range(lo, hi) = &f.over else {
                return true;
            };
            let mut bounds: Vec<(EntityId, &str)> = Vec::new();
            for end in [lo, hi] {
                named_locals(cx, end, &mut bounds);
            }
            if bounds.is_empty() {
                return true;
            }
            walk(std::slice::from_ref(&*f.body), &mut |m, inside| {
                let Some(s) = m.stmt() else {
                    return true;
                };
                let S::Assign {
                    lhs, ..
                } = &s.kind
                else {
                    return true;
                };
                let stops = |t: &Stmt| stopping.contains(&t.span);
                if stops(s) || inside.iter().filter_map(|a| a.stmt()).any(stops) {
                    return true;
                }
                for target in lhs {
                    if !matches!(target.kind, E::Ident(_)) {
                        continue;
                    }
                    let ents = cx.entities(target.span);
                    if let Some((_, name)) = bounds.iter().find(|(b, _)| ents.contains(b)) {
                        out.push(finding(
                            target.span,
                            format!(
                                "`{name}` bounds this loop's range, which was fixed before the \
                                 first iteration: changing it here does not change the loop"
                            ),
                            Some(
                                "use a `while` loop if the bound should follow the change, or \
                                 a separate variable if it should not"
                                    .into(),
                            ),
                        ));
                    }
                }
                true
            });
            true
        });
    }
}

/// Local variables named directly in a range end: `n` in `n - 1`, not `xs` in `xs.count`.
fn named_locals<'a>(cx: &'a Cx, e: &Expr, out: &mut Vec<(EntityId, &'a str)>) {
    match &e.kind {
        E::Ident(_) => {
            if let [only] = &*cx.entities(e.span)
                && matches!(cx.compiler.entity(*only).kind, EntityKind::Local { .. })
            {
                out.push((*only, cx.src(e.span).trim()));
            }
        }
        E::Binary(_, a, b) => {
            named_locals(cx, a, out);
            named_locals(cx, b, out);
        }
        E::Unary(UnOp::Neg | UnOp::Plus | UnOp::BitNot, a) => named_locals(cx, a, out),
        E::Cast {
            value, ..
        } => named_locals(cx, value, out),
        _ => {}
    }
}
