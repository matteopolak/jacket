//! `dropped_result`: `value := to_integer(text);` throws away the `bool` that says whether the
//! conversion worked.
//!
//! A `bool` returned right after the value it describes almost always reports success or
//! "found". Dropping it silently turns a failed parse or lookup into a zero value nobody
//! notices.
//!
//! Fires on a declaration (`x := f();`, `x: T = f();`) or an assignment (`x = f();`) whose only
//! right-hand side is a call, when the call's procedure type returns more values than the
//! left-hand side names and the first value left over, the one right after the last value kept, is
//! a `bool`. Other dropped values, including a `bool` that comes after further results (status
//! extras of a wide API), may be dropped. Quiet for:
//!
//! - an explicit discard: `x, _ := f();`, `_ := f();`;
//! - a call used as a statement (nothing receives its values: `unused_result` and the
//!   compiler's `#must` cover those);
//! - compound assignments (`x += f();`), mixed declare-and-assign lists, and calls whose
//!   procedure type the compiler did not record (overload sets, unchecked code).
//!
//! Calls through procedure values count when their type is known.
use super::{callee_procs, suggesting};
use crate::syntax::Cx;
use crate::{Edit, Finding};
use jaic::ast::walk::walk;
use jaic::ast::{AssignOp, Expr, ExprKind as E, StmtKind as S};
use jaic::types::{TypeId, TypeKind};

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "dropped_result",
    category: crate::config::Category::Suspicious,
    summary: "a call drops the `bool` that follows the values it keeps (`n := to_integer(s);`)",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.clean || p.is_macro {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            let Some(s) = n.stmt() else {
                return true;
            };
            match &s.kind {
                S::Decl(d) => {
                    if d.rare.is_none()
                        && !d.names.is_empty()
                        && let Some(value) = &d.value
                    {
                        let discard = d.names.len() == 1 && d.names[0].name.as_str() == "_";
                        if !discard {
                            let last = d.names.last().map(|n| n.span.end as usize);
                            // `x: T = f();` stays: where the `_` goes there is not settled.
                            examine(
                                cx,
                                value,
                                d.names.len(),
                                last.filter(|_| d.ty.is_none()),
                                out,
                            );
                        }
                    }
                }
                S::Assign {
                    op: AssignOp::Assign,
                    lhs,
                    rhs,
                } => {
                    let discard =
                        lhs.len() == 1 && matches!(lhs[0].kind, E::Ident(n) if n.as_str() == "_");
                    if let [value] = rhs.as_slice()
                        && !discard
                    {
                        examine(
                            cx,
                            value,
                            lhs.len(),
                            Some(lhs[lhs.len() - 1].span.end as usize),
                            out,
                        );
                    }
                }
                _ => {}
            }
            true
        });
    }
}

/// `value` is a call whose procedure returns more than `kept` values, with a `bool` among the
/// ones dropped.
fn examine(cx: &Cx, value: &Expr, kept: usize, after: Option<usize>, out: &mut Vec<Finding>) {
    let E::Call {
        callee, ..
    } = &value.kind
    else {
        return;
    };
    let types = &cx.compiler.ty.types;
    // Every procedure the name stands for returns a `bool` among the values dropped; a
    // procedure value is judged by its type.
    let named = callee_procs(cx, callee);
    let lists: Vec<Vec<TypeId>> = if named.is_empty() {
        match cx.ty(callee).map(|t| types.kind(t)) {
            Some(TypeKind::Proc(sig)) => vec![sig.returns.clone()],
            _ => return,
        }
    } else {
        let mut lists = Vec::new();
        for &p in named {
            match &cx.compiler.pr.procs[p.0 as usize].sig {
                Some(sig) => lists.push(sig.returns.clone()),
                None => return,
            }
        }
        lists
    };
    let drops_bool = |returns: &Vec<TypeId>| {
        returns
            .get(kept)
            .is_some_and(|&t| matches!(types.kind(types.repr_struct(t)), TypeKind::Bool))
    };
    if !lists.iter().all(drops_bool) {
        return;
    }
    let name = cx.whole_src(callee.span).trim();
    let message = format!(
        "`{name}` returns {} values but this keeps {kept}; the next value is a `bool`, which usually says whether the call worked",
        lists[0].len()
    );
    let help = "receive the flag, or write `, _` to show it is ignored on purpose".to_string();
    out.push(match after {
        Some(at) => suggesting(
            value.span,
            message,
            help,
            "ignore the `bool` on purpose with `, _`",
            vec![Edit {
                start: at,
                end: at,
                text: ", _".into(),
            }],
        ),
        None => super::finding(value.span, message, Some(help)),
    });
}
