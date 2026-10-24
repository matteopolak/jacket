//! `assert_constant`: `assert(true)` checks nothing.
//!
//! ```jai
//! assert(true, "never fails");
//! #assert true;
//! ```
//!
//! An assertion whose condition is the literal `true` can never fail, so it is left over from a
//! placeholder, a condition that was replaced, or a constant that was folded by hand. The
//! opposite, `assert(false, "unreachable")`, fails every time it runs: it is how a program says
//! a path must not be reached, and is never reported.
//!
//! The call must be to a procedure declared with the note `@AssertsArgument` (the library's
//! `assert`), whatever it is called where it is used. Macros and generated files are left alone. The fix removes the
//! statement when it stands in a block and its other arguments have no side effects.
use super::{calls_noted, finding, pure, replacing, walk_code};
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::{Expr, ExprKind as E, StmtKind as S};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    walk_code(cx, &mut |n, up| {
        let Some(s) = n.stmt() else {
            return;
        };
        let (what, spent): (&str, Vec<&Expr>) = match &s.kind {
            S::Assert(a) if matches!(a.cond.kind, E::Bool(true)) => {
                ("`#assert true`", a.message.iter().chain(&a.args).collect())
            }
            S::Expr(Expr {
                kind:
                    E::Call {
                        callee,
                        args,
                        ..
                    },
                ..
            }) if args.first().is_some_and(|a| {
                a.name.is_none() && !a.spread && matches!(a.value.kind, E::Bool(true))
            }) && calls_noted(cx, callee, "AssertsArgument") =>
            {
                (
                    "`assert(true)`",
                    args[1..].iter().map(|a| &a.value).collect(),
                )
            }
            _ => return,
        };
        let in_block = up
            .last()
            .is_none_or(|p| matches!(p.stmt().map(|p| &p.kind), Some(S::Block(_))));
        let (start, end) = cx.tidy_removal(s.span);
        let message = format!("{what} can never fail");
        let help = "remove it, or assert the condition it stands in for".to_string();
        if in_block && spent.iter().all(|e| pure(e)) && !cx.has_comment_between(start, end) {
            out.push(replacing(
                s.span,
                message,
                help,
                (start, end),
                String::new(),
                true,
            ));
        } else {
            out.push(finding(s.span, message, Some(help)));
        }
    });
}
