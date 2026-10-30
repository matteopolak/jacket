//! `unnecessary_semicolon`: a `;` that ends nothing.
//!
//! ```jai
//! total += 1;;
//! if ready { go(); };
//! ```
//!
//! The second `;` is an empty statement. It does nothing, and after a block it makes the reader
//! look for the statement it was meant to end.
//!
//! Fires on an empty statement in a block, a `case`, a procedure body or at the top of a file.
//! The `;` that is the whole body of a loop or an `if` (`while poll() ;`) is how an empty body is
//! written, and is left alone. Macros and generated files are skipped. The fix removes the `;`
//! (and its line when it is alone on one). The `;` that is the whole body of an `#if` or its
//! `else` (`#if debug;`) is left alone for the same reason.
use super::replacing;
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::walk::Node;
use jaic::ast::{Stmt, StmtKind as S};

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "unnecessary_semicolon",
    category: crate::config::Category::Style,
    summary: "a `;` that ends nothing (`x = 1;;`, `if c { ... };`)",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    super::walk_code(cx, &mut |n, up| {
        let Node::Stmt(s) = n else {
            return;
        };
        if !matches!(s.kind, S::Empty) {
            return;
        }
        // An empty statement that is the body of a loop, an `if` or a `defer` is deliberate.
        let in_list = match up.last() {
            None => true,
            Some(Node::Stmt(parent)) => matches!(
                parent.kind,
                S::Block(_) | S::Switch(_) | S::StaticSwitch(_) | S::StaticIf(_) | S::Case(_)
            ),
            Some(Node::Expr(_)) => true,
        };
        if !in_list || cx.src(s.span) != ";" || is_static_if_body(cx, s, up.last()) {
            return;
        }
        let (start, end) = cx.tidy_removal(s.span);
        out.push(replacing(
            s.span,
            "this `;` ends nothing".into(),
            "remove it".into(),
            (start, end),
            String::new(),
            true,
        ));
    });
}

/// The `;` is the whole of an `#if`'s then or else arm, written without braces. Removing it
/// would leave the `#if` without a body, so the file would no longer parse. (An arm written
/// with braces ends at its `}`, and an empty statement inside it is removable.)
fn is_static_if_body(cx: &Cx, empty: &Stmt, parent: Option<&Node<'_>>) -> bool {
    let Some(Node::Stmt(Stmt {
        kind: S::StaticIf(branch),
        ..
    })) = parent
    else {
        return false;
    };
    let arm = if let [only] = branch.then_branch.as_slice()
        && only.span == empty.span
    {
        branch.then_arm
    } else if let [only] = branch.else_branch.as_slice()
        && only.span == empty.span
        && let Some(arm) = branch.else_arm
    {
        arm
    } else {
        return false;
    };
    cx.src(arm).trim_end().ends_with(';')
}
