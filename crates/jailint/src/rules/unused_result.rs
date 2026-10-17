//! `unused_result`: `trim(line);`, `to_upper_copy(name);`, `min(a, b);` as statements.
//!
//! These library procedures only compute a value: they do not change their arguments. Calling
//! one as a statement does nothing (or leaks the copy it made), and the usual intent was to
//! update the argument: `line = trim(line);`.
//!
//! Fires on an expression statement that calls one of a fixed list of value-returning
//! procedures from `Basic`, `String` and `Math` (checked by what the name resolved to, so a
//! program's own `trim` is not affected), when the call returns a value and no argument is a
//! pointer (`normalize(*v)` normalizes in place).
use super::library::VALUE_ONLY;
use super::{finding, pure};
use crate::Finding;
use crate::syntax::{Cx, Node, root_ident, walk};
use jaic::ast::{ExprKind as E, StmtKind as S};
use jaic::types::{TypeId, TypeKind};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    if cx.generated() {
        return;
    }
    for p in &cx.procs {
        if !p.clean {
            continue;
        }
        walk(&p.body.stmts, &mut |n, chain| {
            let Some(s) = n.stmt() else {
                return false;
            };
            let S::Expr(e) = &s.kind else {
                return true;
            };
            let E::Call {
                callee,
                args,
                ..
            } = &e.kind
            else {
                return true;
            };
            if chain.iter().any(|a| matches!(a, Node::Expr(_))) || !VALUE_ONLY.called_by(cx, callee)
            {
                return true;
            }
            let types = &cx.compiler.types;
            let returns = cx
                .ty(e)
                .is_some_and(|t| t != TypeId::VOID && !matches!(types.kind(t), TypeKind::Void));
            let by_pointer = args
                .iter()
                .any(|a| a.spread || cx.ty(&a.value).is_none_or(|t| types.is_pointer(t)));
            if !returns || by_pointer {
                return true;
            }
            let name = cx.whole_src(callee.span).trim();
            let call = cx.whole_src(e.span).trim();
            let help = match args.first() {
                Some(first)
                    if pure(&first.value)
                        && root_ident(&first.value).is_some()
                        && first.name.is_none()
                        && cx.ty(&first.value) == cx.ty(e) =>
                {
                    format!(
                        "to update the argument, write `{} = {call};`",
                        cx.whole_src(first.value.span).trim()
                    )
                }
                _ => "use the result, or remove the call".into(),
            };
            out.push(finding(
                e.span,
                format!("the result of `{name}` is discarded; it does not change its arguments"),
                Some(help),
            ));
            true
        });
    }
}
