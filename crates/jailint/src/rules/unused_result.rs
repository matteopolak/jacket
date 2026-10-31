//! `unused_result`: `trim(line);`, `to_upper_copy(name);`, `min(a, b);` as statements.
//!
//! These procedures only compute a value: they do not change their arguments. Calling
//! one as a statement does nothing (or leaks the copy it made), and the usual intent was to
//! update the argument: `line = trim(line);`.
//!
//! Fires on an expression statement that calls a procedure declared with the note
//! `@UseResult` (the library marks its value-returning `String`, `Basic` and `Math`
//! procedures; a program marks its own the same way), when the call returns a value and no
//! argument is a pointer (`normalize(*v)` normalizes in place).
use super::{calls_noted, finding, pure, suggesting};
use crate::syntax::{Cx, root_ident};
use crate::{Edit, Finding};
use jaic::ast::walk::{Node, walk};
use jaic::ast::{ExprKind as E, StmtKind as S};
use jaic::types::{TypeId, TypeKind};

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "unused_result",
    category: crate::config::Category::Suspicious,
    summary: "a call to a `@UseResult` procedure, as a statement",
    check,
};

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
            if chain.iter().any(|a| matches!(a, Node::Expr(_)))
                || !calls_noted(cx, callee, "UseResult")
            {
                return true;
            }
            let types = &cx.compiler.ty.types;
            let returns = cx
                .ty(e)
                .is_some_and(|t| t != TypeId::VOID && !matches!(types.kind(t), TypeKind::Void));
            let by_pointer = args
                .iter()
                .any(|a| a.spread || cx.ty(&a.value).is_none_or(|t| types.is_pointer(t)));
            if !returns || by_pointer {
                return true;
            }
            let name = cx.whole_src(callee).trim();
            let call = cx.whole_src(e).trim();
            let mut edits = Vec::new();
            let help = match args.first() {
                Some(first)
                    if pure(&first.value)
                        && root_ident(&first.value).is_some()
                        && first.name.is_none()
                        && cx.ty(&first.value) == cx.ty(e) =>
                {
                    let target = cx.whole_src(&first.value).trim();
                    edits.push(Edit {
                        start: e.span.start as usize,
                        end: e.span.start as usize,
                        text: format!("{target} = "),
                    });
                    format!("to update the argument, write `{target} = {call};`")
                }
                _ => "keep the result in a variable, or remove the call".into(),
            };
            let message =
                format!("the result of `{name}` is discarded; it does not change its arguments");
            out.push(if edits.is_empty() {
                finding(e.span, message, Some(help))
            } else {
                suggesting(
                    e.span,
                    message,
                    help,
                    "assign the result back to the argument",
                    edits,
                )
            });
            true
        });
    }
}
