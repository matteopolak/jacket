//! `bool_comparison`: `x == true`, `x != false`, `x == false`.
//!
//! A `bool` is already the condition: `if done` reads better than `if done == true`, and
//! `if !done` better than `if done == false`.
//!
//! Fires when one side is the literal `true` or `false` and the other is a non-constant
//! expression of type `bool` (checked by the compiler, so `x == true` on an integer or on a
//! type with `operator ==` is left alone).
use crate::syntax::{Cx, Node, is_atom, walk};
use crate::{Edit, Finding, Fix};
use jaic::ast::{BinOp, ExprKind as E};
use jaic::types::TypeId;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, chain| {
            let Some(e) = n.expr() else {
                return true;
            };
            let E::Binary(op @ (BinOp::Eq | BinOp::Ne), a, b) = &e.kind else {
                return true;
            };
            let (lit, other) = match (&a.kind, &b.kind) {
                (E::Bool(v), _) => (*v, b),
                (_, E::Bool(v)) => (*v, a),
                _ => return true,
            };
            if matches!(other.kind, E::Bool(_))
                || cx.ty(other) != Some(TypeId::BOOL)
                || cx.constant(other)
            {
                return true;
            }
            let negate = (*op == BinOp::Eq) != lit;
            let text = cx.src(other.span).trim();
            // Whether the replacement stands alone (a condition, a statement's value) or sits
            // inside a larger expression.
            let alone = matches!(chain.last(), Some(Node::Stmt(_)) | None);
            let replacement = match (negate, is_atom(other)) {
                (false, true) => text.to_string(),
                (false, false) if alone => text.to_string(),
                (false, false) => format!("({text})"),
                (true, true) => format!("!{text}"),
                (true, false) => format!("!({text})"),
            };
            let written = cx.src(e.span).trim();
            let help = format!("write `{replacement}`");
            out.push(Finding {
                start: e.span.start as usize,
                end: e.span.end as usize,
                message: format!("comparing a bool with `{lit}`: `{written}`"),
                label: None,
                help: Some(help.clone()),
                fix: Some(Fix {
                    title: help,
                    edits: vec![Edit {
                        start: e.span.start as usize,
                        end: e.span.end as usize,
                        text: replacement,
                    }],
                    machine_applicable: true,
                }),
            });
            true
        });
    }
}
