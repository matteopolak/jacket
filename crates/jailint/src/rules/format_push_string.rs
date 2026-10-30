//! `format_push_string`: `append(*b, tprint(...))`, formatting into a new string only to add it
//! to a `String_Builder`.
//!
//! ```jai
//! append(*builder, tprint("%: %\n", name, value));
//! ```
//!
//! `tprint` builds the text in a builder of its own and copies it out (a `sprint` result also
//! has to be freed, which this form tends to forget); `print_to_builder(*builder, "%: %\n",
//! name, value)` writes into the builder directly.
//!
//! Fires on a call of a procedure noted `@AppendsString` (the library's `append` of a string)
//! whose string is a call of one noted `@FormatsToString` (`tprint`, `sprint`). The editor offers
//! the `print_to_builder` call; it is not applied by `--fix` because a formatting failure leaves
//! the builder with the part that was written instead of nothing.
use super::{calls_noted, suggesting};
use crate::Edit;
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::ExprKind as E;
use jaic::ast::walk::walk;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "format_push_string",
    category: crate::config::Category::Perf,
    summary: "`append(*b, tprint(...))`, which formats into a new string to add it to a builder",
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
            let [builder, text] = args.as_slice() else {
                return true;
            };
            let E::Call {
                callee: format,
                args: inner,
                ..
            } = &text.value.kind
            else {
                return true;
            };
            let (Some(first), Some(last)) = (inner.first(), inner.last()) else {
                return true;
            };
            if builder.name.is_some()
                || text.name.is_some()
                || builder.spread
                || text.spread
                || inner.iter().any(|a| a.context)
                || !calls_noted(cx, callee, "AppendsString")
                || !calls_noted(cx, format, "FormatsToString")
            {
                return true;
            }
            let name = cx.src(format.span).trim();
            let leaks = calls_noted(cx, format, "Allocates");
            let (_, end) = cx.whole(last.value.span);
            let values = cx
                .text
                .get(first.value.span.start as usize..end)
                .unwrap_or("")
                .trim();
            let replacement = format!(
                "print_to_builder({}, {values})",
                cx.whole_src(builder.value.span).trim()
            );
            out.push(suggesting(
                e.span,
                format!("`{name}` formats into a new string just to append it to a builder"),
                if leaks {
                    "`print_to_builder` writes into the builder directly, and the string `sprint` returns is never freed here"
                        .into()
                } else {
                    "`print_to_builder` writes into the builder directly".into()
                },
                format!("write `{replacement}`"),
                vec![Edit {
                    start: e.span.start as usize,
                    end: e.span.end as usize,
                    text: replacement,
                }],
            ));
            true
        });
    }
}
