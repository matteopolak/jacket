//! `leaked_temporary`: `if sprint("%", n) == "7"` allocates a string nothing ever frees.
//!
//! A procedure declared with the note `@Allocates` returns memory the caller owns. When the
//! call is used only where its value is read and then dropped, there is no name to free it
//! through, so the memory is leaked on every execution.
//!
//! ```jai
//! if sprint("%", n) == "7" { ... }      // compared, then lost
//! length := join(a, b).count;           // length read, string lost
//! print("%\n", sprint("%", n));         // printed, then lost
//! ```
//!
//! Fires on a call of an `@Allocates` procedure that is, directly:
//!
//! - an operand of `==`, `!=`, `<`, `<=`, `>` or `>=`;
//! - the receiver of `.count` or of an index (`x()[i]`);
//! - an argument, other than the format string, of a call of a `@PrintLike` procedure.
//!
//! Anything else (storing it, passing it to some other call, returning it) may hand it on. The
//! rule stays quiet in a procedure that changes the allocator (`push_context`, assigning
//! `context`, a macro call such as `push_allocator`, `#insert`) and for a call that carries a
//! `,,` context change, since the memory may then be temporary. Fix it by keeping the value in
//! a local and freeing it (`defer free(s);`), or by using a temporary-storage procedure such as
//! `tprint`.
use super::leaked_allocation::changes_allocator;
use super::{calls_noted, finding};
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::walk::{Node, walk};
use jaic::ast::{BinOp, Expr, ExprKind as E};

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "leaked_temporary",
    category: crate::config::Category::Suspicious,
    summary: "an `@Allocates` result that is only compared, measured, indexed or printed, so nothing can free it",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.clean || p.is_macro || changes_allocator(cx, p.header, &p.body.stmts) {
            continue;
        }
        walk(&p.body.stmts, &mut |n, chain| {
            let Node::Expr(e) = n else {
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
            if args.iter().any(|a| a.context) || !calls_noted(cx, callee, "Allocates") {
                return true;
            }
            if let Some(how) = transient_use(cx, e, chain) {
                let name = cx.whole_src(callee).trim();
                out.push(finding(
                    e.span,
                    format!("the memory `{name}` allocates is {how} and never freed"),
                    Some(
                        "keep it in a local and `defer free` it, or use a temporary-storage \
                         procedure such as `tprint`"
                            .into(),
                    ),
                ));
            }
            true
        });
    }
}

/// How the call `e` (whose ancestors are `chain`) is used, when that drops it at once.
fn transient_use(cx: &Cx, e: &Expr, chain: &[Node<'_>]) -> Option<&'static str> {
    let Some(Node::Expr(parent)) = chain.last() else {
        return None;
    };
    match &parent.kind {
        E::Binary(BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge, ..) => {
            Some("only compared")
        }
        E::Member(base, name) if base.span == e.span && name.name.as_str() == "count" => {
            Some("only measured")
        }
        E::Index(base, _) if base.span == e.span => Some("only indexed"),
        E::Call {
            callee,
            args,
            ..
        } if args.iter().skip(1).any(|a| a.value.span == e.span)
            && calls_noted(cx, callee, "PrintLike") =>
        {
            Some("only printed")
        }
        _ => None,
    }
}
