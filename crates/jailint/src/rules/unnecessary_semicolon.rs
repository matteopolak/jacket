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
//! (and its line when it is alone on one).
use super::replacing;
use crate::Finding;
use crate::syntax::{Cx, Node};
use jaic::ast::StmtKind as S;

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
        if !in_list || cx.src(s.span) != ";" {
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
