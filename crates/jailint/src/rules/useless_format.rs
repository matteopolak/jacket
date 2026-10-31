//! `useless_format`: `tprint("%", s)` or `sprint("%", s)` where `s` is already a string.
//!
//! Formatting a string with the format `"%"` builds a copy of it through a `String_Builder`.
//! If a copy is not wanted, use `s`; if it is, `copy_string(s)` says so (and is faster).
//!
//! Fires on a call of a procedure noted `@FormatsToString` (the library's `tprint` and
//! `sprint`) whose format is the literal `"%"` and whose single value has type `string`. The
//! editor offers `s` for a `tprint` and `copy_string(s)` for a `sprint`. They are suggestions
//! rather than fixes because a `tprint` result lives in temporary storage and a `sprint`
//! result is owned by the caller, so replacing either changes who may free what.
use super::{calls_noted, suggesting};
use crate::Edit;
use crate::Finding;
use crate::syntax::{Cx, is_atom};
use jaic::ast::ExprKind as E;
use jaic::ast::walk::walk;
use jaic::types::TypeId;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "useless_format",
    category: crate::config::Category::Complexity,
    summary: "`tprint(\"%\", s)` or `sprint(\"%\", s)` with a string `s`, which only copies it",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            let Some(e) = n.expr() else {
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
            let [format, value] = args.as_slice() else {
                return true;
            };
            if !matches!(&format.value.kind, E::Str(s) if &**s == b"%")
                || format.name.is_some()
                || value.name.is_some()
                || value.spread
                || value.context
                || cx.ty(&value.value) != Some(TypeId::STRING)
                || !calls_noted(cx, callee, "FormatsToString")
            {
                return true;
            }
            let name = cx.src(callee.span).trim();
            let shown = cx.whole_src(&value.value).trim();
            let owned = calls_noted(cx, callee, "Allocates");
            let text = if owned {
                format!("copy_string({shown})")
            } else if is_atom(&value.value) {
                shown.to_string()
            } else {
                format!("({shown})")
            };
            out.push(suggesting(
                e.span,
                format!("`{name}(\"%\", {shown})` only copies a string"),
                if owned {
                    "use `copy_string` for a copy you own, or the string itself".into()
                } else {
                    "use the string itself, or `copy_string` for a copy you own".into()
                },
                format!("write `{text}`"),
                vec![Edit {
                    start: e.span.start as usize,
                    end: e.span.end as usize,
                    text,
                }],
            ));
            true
        });
    }
}
