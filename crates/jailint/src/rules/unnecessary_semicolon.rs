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
use jaic::ast::StmtKind as S;
use jaic::ast::walk::Node;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "unnecessary_semicolon",
    category: crate::config::Category::Style,
    summary: "a `;` that ends nothing (`x = 1;;`, `if c { ... };`)",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    super::walk_code(cx, &mut |n, _| {
        let Node::Stmt(s) = n else {
            return;
        };
        // An empty statement that is the body of a loop, an `if`, a `defer` or an `#if` arm is
        // deliberate: removing it would leave its parent without a body.
        if !matches!(s.kind, S::Empty) || cx.src(s.span) != ";" || cx.is_bare_body(s.span) {
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
