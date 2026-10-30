//! `bool_to_int_with_if`: `ifx c then 1 else 0` is `cast(int) c`.
//!
//! ```jai
//! bit := ifx flag then 1 else 0;
//! if flag count = 1; else count = 0;
//! ```
//!
//! A `bool` converts to an integer with a cast, with no branch. Off by default: the `ifx` reads
//! more plainly than the cast to many programmers, and the cast says nothing about which value
//! `true` becomes.
//!
//! Fires, when the condition is a `bool` and the result an integer, on an `ifx` that picks the
//! literals `1` and `0` (either way round, the second as `cast(int) !c`), and on an `if`/`else`
//! that assigns them to the same side-effect-free target. The editor offers the cast, naming the
//! result's type; `--fix` leaves it.
use super::needless_bool::condition;
use super::{pure, suggesting};
use crate::syntax::Cx;
use crate::{Edit, Finding};
use jaic::ast::walk::{Node, walk};
use jaic::ast::{AssignOp, Expr, ExprKind as E, Stmt, StmtKind as S};
use jaic::source::Span;
use jaic::types::TypeId;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "bool_to_int_with_if",
    category: crate::config::Category::Pedantic,
    summary: "`ifx c then 1 else 0`, a `bool` turned into an integer with a branch",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            match n {
                Node::Expr(e) => {
                    if let E::Ifx {
                        cond,
                        then_value: Some(a),
                        else_value: Some(b),
                        is_static: false,
                    } = &e.kind
                        && let Some(inverted) = zero_one(a, b)
                        && let Some(ty) = int_type(cx, e)
                    {
                        report(cx, e.span, cond, inverted, &ty, "", out);
                    }
                }
                Node::Stmt(s) => assignments(cx, s, out),
            }
            true
        });
    }
}

/// `if c x = 1; else x = 0;` (or `0` then `1`).
fn assignments(cx: &Cx, s: &Stmt, out: &mut Vec<Finding>) {
    let S::If {
        cond,
        then_branch,
        else_branch: Some(other),
    } = &s.kind
    else {
        return;
    };
    let (Some((target, a)), Some((target2, b))) = (assignment(then_branch), assignment(other))
    else {
        return;
    };
    if !pure(target) || !cx.same_tokens(target.span, target2.span) {
        return;
    }
    let (Some(inverted), Some(ty)) = (zero_one(a, b), cx.ty(target)) else {
        return;
    };
    let types = &cx.compiler.ty.types;
    if !types.is_integer(ty) {
        return;
    }
    let target_text = cx.whole_src(target.span).trim();
    report(
        cx,
        s.span,
        cond,
        inverted,
        &types.name(ty),
        &format!("{target_text} = "),
        out,
    );
}

/// `x = v;` as a statement, or as the only statement of a block.
fn assignment(s: &Stmt) -> Option<(&Expr, &Expr)> {
    match &s.kind {
        S::Assign {
            op: AssignOp::Assign,
            lhs,
            rhs,
        } => match (lhs.as_slice(), rhs.as_slice()) {
            ([target], [value]) => Some((target, value)),
            _ => None,
        },
        S::Block(b) => {
            let mut stmts = b.stmts.iter().filter(|s| !matches!(s.kind, S::Empty));
            let only = stmts.next()?;
            if stmts.next().is_some() {
                return None;
            }
            assignment(only)
        }
        _ => None,
    }
}

/// The literals `1` and `0` in that order (`Some(false)`) or `0` and `1` (`Some(true)`, the
/// condition inverted).
fn zero_one(a: &Expr, b: &Expr) -> Option<bool> {
    let value = |e: &Expr| match &e.kind {
        E::Int(v, ..) => Some(v.get()),
        _ => None,
    };
    match (value(a)?, value(b)?) {
        (1, 0) => Some(false),
        (0, 1) => Some(true),
        _ => None,
    }
}

/// The integer type of the `ifx`.
fn int_type(cx: &Cx, e: &Expr) -> Option<String> {
    let ty: TypeId = cx.ty(e)?;
    let types = &cx.compiler.ty.types;
    types.is_integer(ty).then(|| types.name(ty))
}

fn report(
    cx: &Cx,
    span: Span,
    cond: &Expr,
    inverted: bool,
    ty: &str,
    prefix: &str,
    out: &mut Vec<Finding>,
) {
    if cx.ty(cond) != Some(TypeId::BOOL) {
        return;
    }
    let operand = condition(cx, cond, inverted, true);
    let cast = format!("cast({ty}) {operand}");
    let semicolon = if prefix.is_empty() {
        ""
    } else {
        ";"
    };
    let replacement = format!("{prefix}{cast}{semicolon}");
    out.push(suggesting(
        span,
        "a `bool` is turned into an integer with a branch".into(),
        format!("write `{cast}`"),
        format!("write `{cast}`"),
        vec![Edit {
            start: span.start as usize,
            end: span.end as usize,
            text: replacement,
        }],
    ));
}
