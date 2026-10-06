//! `needless_bool`: `ifx c then true else false`, and `if c return true; else return false;`.
//!
//! The condition already is the `bool` being produced. Spelling it out through a branch hides
//! that (and invites inverting one of the literals by mistake).
//!
//! Fires when the condition has type `bool` and:
//!
//! - an `ifx` picks between the literals `true` and `false` → `c` (or `!c`);
//! - an `if`/`else` returns `true` in one branch and `false` in the other → `return c;`;
//! - an `if` that returns `true` or `false` is directly followed by a `return` of the other
//!   literal → `return c;` (unless an earlier `if` in the same block returns a literal too:
//!   then it is the last of a series of guards and reads best like the others);
//! - an `if`/`else` assigns `true` and `false` to the same side-effect-free target →
//!   `x = c;`.
//!
//! The fixes are machine-applicable. Comments inside a rewritten `if` keep it from being
//! offered, so nothing written there is lost.
use super::{op_text, parenthesized, pure, replacing};
use crate::Finding;
use crate::syntax::{Cx, Node, is_atom, squash, walk};
use jaic::ast::{AssignOp, BinOp, Expr, ExprKind as E, Stmt, StmtKind as S};
use jaic::source::Span;
use jaic::types::TypeId;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, chain| {
            match n {
                Node::Expr(e) => ifx(cx, e, chain, out),
                Node::Stmt(s) => {
                    if let S::Block(b) = &s.kind {
                        sequence(cx, &b.stmts, out);
                    }
                    branches(cx, s, chain, out);
                }
            }
            true
        });
        sequence(cx, &p.body.stmts, out);
    }
}

fn literal(e: &Expr) -> Option<bool> {
    match e.kind {
        E::Bool(v) => Some(v),
        _ => None,
    }
}

/// `c` or `!c` as text, parenthesized for a larger expression when `inside`. A negated
/// comparison of integers is written as the opposite comparison (`!(a < b)` is `a >= b`);
/// floats keep the `!`, since NaN fails both.
fn condition(cx: &Cx, cond: &Expr, negate: bool, inside: bool) -> String {
    let text = cx.whole_src(cond.span).trim();
    if negate
        && let E::Binary(op, a, b) = &cond.kind
        && let Some(opposite) = opposite(*op)
        && !parenthesized(cx, cond)
        && cx.ty(a).is_some_and(|t| !cx.compiler.types.is_float(t))
        && cx.ty(b).is_some_and(|t| !cx.compiler.types.is_float(t))
    {
        let flipped = format!(
            "{} {} {}",
            cx.whole_src(a.span).trim(),
            op_text(opposite),
            cx.whole_src(b.span).trim()
        );
        return if inside {
            format!("({flipped})")
        } else {
            flipped
        };
    }
    let grouped = is_atom(cond) || parenthesized(cx, cond);
    match (negate, grouped) {
        (false, true) => text.to_string(),
        (false, false) if inside => format!("({text})"),
        (false, false) => text.to_string(),
        (true, true) => format!("!{text}"),
        (true, false) => format!("!({text})"),
    }
}

fn opposite(op: BinOp) -> Option<BinOp> {
    Some(match op {
        BinOp::Eq => BinOp::Ne,
        BinOp::Ne => BinOp::Eq,
        BinOp::Lt => BinOp::Ge,
        BinOp::Ge => BinOp::Lt,
        BinOp::Gt => BinOp::Le,
        BinOp::Le => BinOp::Gt,
        _ => return None,
    })
}

fn is_bool(cx: &Cx, cond: &Expr) -> bool {
    cx.ty(cond) == Some(TypeId::BOOL)
}

fn ifx(cx: &Cx, e: &Expr, chain: &[Node<'_>], out: &mut Vec<Finding>) {
    let E::Ifx {
        cond,
        then_value: Some(a),
        else_value: Some(b),
        is_static: false,
    } = &e.kind
    else {
        return;
    };
    let (Some(a), Some(b)) = (literal(a), literal(b)) else {
        return;
    };
    if a == b || !is_bool(cx, cond) {
        return;
    }
    let inside = matches!(chain.last(), Some(Node::Expr(_)));
    let replacement = condition(cx, cond, !a, inside);
    out.push(replacing(
        e.span,
        "this `ifx` turns a condition into the same `bool`".into(),
        format!("write `{replacement}`"),
        cx.whole(e.span),
        replacement,
        true,
    ));
}

/// The single statement a branch consists of.
fn only(s: &Stmt) -> Option<&Stmt> {
    match &s.kind {
        S::Block(b) => match b.stmts.as_slice() {
            [one] => only(one),
            _ => None,
        },
        _ => Some(s),
    }
}

fn returned(s: &Stmt) -> Option<bool> {
    match &only(s)?.kind {
        S::Return {
            values,
            backtick: false,
        } => match values.as_slice() {
            [v] if v.name.is_none() => literal(&v.value),
            _ => None,
        },
        _ => None,
    }
}

/// `target = literal;` in a branch.
fn assigned(s: &Stmt) -> Option<(&Expr, bool)> {
    match &only(s)?.kind {
        S::Assign {
            op: AssignOp::Assign,
            lhs,
            rhs,
        } => match (lhs.as_slice(), rhs.as_slice()) {
            ([l], [r]) if pure(l) => Some((l, literal(r)?)),
            _ => None,
        },
        _ => None,
    }
}

/// No comment between `start` and `end` would be dropped by replacing the text.
fn no_comments(cx: &Cx, start: usize, end: usize) -> bool {
    let text = &cx.text[start..end];
    !text.contains("//") && !text.contains("/*")
}

fn branches(cx: &Cx, s: &Stmt, chain: &[Node<'_>], out: &mut Vec<Finding>) {
    let S::If {
        cond,
        then_branch,
        else_branch: Some(other),
    } = &s.kind
    else {
        return;
    };
    // An `else if` link: rewriting it would need the `else` kept.
    if matches!(chain.last(), Some(Node::Stmt(p)) if matches!(&p.kind, S::If { else_branch: Some(e), .. } if e.span == s.span))
    {
        return;
    }
    if !is_bool(cx, cond) {
        return;
    }
    let (start, end) = statement_range(cx, s.span);
    if !no_comments(cx, start, end) {
        return;
    }
    if let (Some(a), Some(b)) = (returned(then_branch), returned(other))
        && a != b
    {
        let replacement = format!("return {};", condition(cx, cond, !a, false));
        out.push(replacing(
            s.span,
            "this `if` returns its own condition as a `bool`".into(),
            format!("write `{replacement}`"),
            (start, end),
            replacement,
            true,
        ));
        return;
    }
    if let (Some((x, a)), Some((y, b))) = (assigned(then_branch), assigned(other))
        && a != b
        && squash(cx.whole_src(x.span)) == squash(cx.whole_src(y.span))
        && is_bool(cx, x)
    {
        let replacement = format!(
            "{} = {};",
            cx.whole_src(x.span).trim(),
            condition(cx, cond, !a, false)
        );
        out.push(replacing(
            s.span,
            "this `if` assigns its own condition as a `bool`".into(),
            format!("write `{replacement}`"),
            (start, end),
            replacement,
            true,
        ));
    }
}

/// `if c return true; return false;` as two statements of one block.
fn sequence(cx: &Cx, stmts: &[Stmt], out: &mut Vec<Finding>) {
    for (i, pair) in stmts.windows(2).enumerate() {
        let [first, second] = pair else {
            continue;
        };
        // The last of a series of guards (`if a return false; x := f(); if b return false;
        // return true;`) reads better kept like the others.
        if stmts[..i].iter().any(|s| {
            matches!(&s.kind, S::If { then_branch, else_branch: None, .. }
                if returned(then_branch).is_some())
        }) {
            continue;
        }
        let S::If {
            cond,
            then_branch,
            else_branch: None,
        } = &first.kind
        else {
            continue;
        };
        let (Some(a), Some(b)) = (returned(then_branch), returned(second)) else {
            continue;
        };
        if a == b || !is_bool(cx, cond) {
            continue;
        }
        let (start, _) = statement_range(cx, first.span);
        let (_, end) = statement_range(cx, second.span);
        if !no_comments(cx, start, end) {
            continue;
        }
        let replacement = format!("return {};", condition(cx, cond, !a, false));
        out.push(replacing(
            first.span.to(second.span),
            "this `if` returns its own condition as a `bool`".into(),
            format!("write `{replacement}`"),
            (start, end),
            replacement,
            true,
        ));
    }
}

/// A statement's bytes with the `;` the span may stop before.
fn statement_range(cx: &Cx, span: Span) -> (usize, usize) {
    let start = span.start as usize;
    let mut end = span.end as usize;
    let rest = &cx.text[end..];
    let trimmed = rest.trim_start_matches([' ', '\t']);
    if trimmed.starts_with(';') && !cx.text[..end].ends_with(';') {
        end += rest.len() - trimmed.len() + 1;
    }
    (start, end)
}
