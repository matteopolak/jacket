//! `no_effect`: a statement that computes a value and throws it away, `x == 5;`.
//!
//! A comparison, arithmetic or a bare name as a statement does nothing. The usual cause is a
//! typo: `x == 5;` for `x = 5;`, `count + 1;` for `count += 1;`, or `flush;` for `flush();`.
//!
//! Fires on an expression statement made only of names, member paths, literals, indexing and
//! operators (no calls, which may have effects). Statements inside an expression (the last
//! statement of a block value, `#code`) are left alone: there the value is used.
use super::{finding, op_text, pure};
use crate::Finding;
use crate::syntax::{Cx, Node, walk};
use jaic::ast::{BinOp, ExprKind as E, StmtKind as S};

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
            let help = match &e.kind {
                E::Binary(BinOp::Eq, a, b) => format!(
                    "to assign, write `{} = {};`",
                    cx.whole_src(a.span).trim(),
                    cx.whole_src(b.span).trim()
                ),
                E::Binary(
                    op @ (BinOp::Add
                    | BinOp::Sub
                    | BinOp::Mul
                    | BinOp::Div
                    | BinOp::BitAnd
                    | BinOp::BitOr
                    | BinOp::BitXor),
                    a,
                    b,
                ) => format!(
                    "to update `{}`, write `{} {}= {};`",
                    cx.whole_src(a.span).trim(),
                    cx.whole_src(a.span).trim(),
                    op_text(*op),
                    cx.whole_src(b.span).trim()
                ),
                E::Ident(_) | E::Member(..)
                    if !super::callee_procs(cx, e).is_empty()
                        || cx.ty(e).is_some_and(|t| {
                            matches!(cx.compiler.types.kind(t), jaic::types::TypeKind::Proc(_))
                        }) =>
                {
                    format!("to call it, write `{}();`", cx.whole_src(e.span).trim())
                }
                _ => "remove it, or use its value".into(),
            };
            out.push(finding(
                s.span,
                "this statement has no effect".into(),
                Some(help),
            ));
            true
        });
    }
}
