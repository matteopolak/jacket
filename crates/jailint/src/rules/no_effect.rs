//! `no_effect`: a statement that computes a value and throws it away, `x == 5;`.
//!
//! A comparison, arithmetic or a bare name as a statement does nothing. The usual cause is a
//! typo: `x == 5;` for `x = 5;`, `count + 1;` for `count += 1;`, or `flush;` for `flush();`.
//!
//! Fires on an expression statement made only of names, member paths, literals, indexing and
//! operators (no calls, which may have effects). Statements inside an expression (the last
//! statement of a block value, `#code`) are left alone: there the value is used.
use super::{op_text, pure, suggesting};
use crate::syntax::{Cx, Node, walk};
use crate::{Edit, Finding};
use jaic::ast::{BinOp, ExprKind as E, StmtKind as S};

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "no_effect",
    category: crate::config::Category::Suspicious,
    summary: "a statement that computes a value and drops it (`x == 5;`)",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        walk(&p.body.stmts, &mut |n, chain| {
            let Some(s) = n.stmt() else {
                return false;
            };
            let S::Expr(e) = &s.kind else {
                return true;
            };
            if !pure(e) || chain.iter().any(|a| matches!(a, Node::Expr(_))) {
                return true;
            }
            let b = e.span.end as usize;
            let edit = |start: usize, end: usize, text: String| Edit {
                start,
                end,
                text,
            };
            let (help, title, edits) = match &e.kind {
                E::Binary(BinOp::Eq, l, r) => (
                    format!(
                        "to assign, write `{} = {};`",
                        cx.whole_src(l.span).trim(),
                        cx.whole_src(r.span).trim()
                    ),
                    "assign instead of comparing",
                    vec![edit(cx.whole(l.span).1, cx.whole(r.span).0, " = ".into())],
                ),
                E::Binary(
                    op @ (BinOp::Add
                    | BinOp::Sub
                    | BinOp::Mul
                    | BinOp::Div
                    | BinOp::BitAnd
                    | BinOp::BitOr
                    | BinOp::BitXor),
                    l,
                    r,
                ) => (
                    format!(
                        "to update `{}`, write `{} {}= {};`",
                        cx.whole_src(l.span).trim(),
                        cx.whole_src(l.span).trim(),
                        op_text(*op),
                        cx.whole_src(r.span).trim()
                    ),
                    "update the variable instead of computing a value",
                    vec![edit(
                        cx.whole(l.span).1,
                        cx.whole(r.span).0,
                        format!(" {}= ", op_text(*op)),
                    )],
                ),
                E::Ident(_) | E::Member(..)
                    if !super::callee_procs(cx, e).is_empty()
                        || cx.ty(e).is_some_and(|t| {
                            matches!(cx.compiler.ty.types.kind(t), jaic::types::TypeKind::Proc(_))
                        }) =>
                {
                    (
                        format!("to call it, write `{}();`", cx.whole_src(e.span).trim()),
                        "call it",
                        vec![edit(b, b, "()".into())],
                    )
                }
                _ => {
                    let (start, end) = cx.statement_removal(s.span);
                    (
                        "remove it, or use its value".into(),
                        "remove the statement",
                        vec![edit(start, end, String::new())],
                    )
                }
            };
            out.push(suggesting(
                s.span,
                "this statement has no effect".into(),
                help,
                title,
                edits,
            ));
            true
        });
    }
}
