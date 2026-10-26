//! `manual_array_find`: a loop that looks for a value and breaks, where `array_find` does it.
//!
//! ```jai
//! found := false;
//! for xs {
//!     if it == wanted {
//!         found = true;
//!         break;
//!     }
//! }
//! ```
//!
//! is `found := array_find(xs, wanted);`, and with `index = it_index;` in place of the flag
//! `found, index := array_find(xs, wanted);` (and `array_find` searches long integer arrays a
//! vector at a time).
//!
//! Fires on a `for` over an array whose body is one `if` without an `else`: it compares the
//! loop's value (`it`, or the name given to it) with another expression of the same type that
//! does not mention the loop's names, and its body is assignments of `true`, `false`, the value
//! or the index to variables outside the loop, followed by `break`. Loops over a pointer, in
//! reverse, or with a custom iterator are left alone, as are searches that compare a member
//! (`it.id == key`), which need a comparison function. No fix.
use crate::Finding;
use crate::syntax::{Cx, walk};
use jaic::ast::{AssignOp, BinOp, Expr, ExprKind as E, For, ForOver, Stmt, StmtKind as S};
use jaic::intern::Sym;
use jaic::types::TypeKind;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "manual_array_find",
    category: crate::config::Category::Complexity,
    summary: "a loop that searches an array and breaks with a found flag or index, which `array_find` does",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            if let Some(s) = n.stmt()
                && let S::For(f) = &s.kind
                && let Some(found) = examine(cx, s, f)
            {
                out.push(found);
            }
            true
        });
    }
}

/// The body's one statement, braces or not.
fn only_statement(body: &Stmt) -> Option<&Stmt> {
    match &body.kind {
        S::Block(b) => match b.stmts.as_slice() {
            [one] => only_statement(one),
            _ => None,
        },
        _ => Some(body),
    }
}

fn examine(cx: &Cx, stmt: &Stmt, f: &For) -> Option<Finding> {
    let types = &cx.compiler.ty.types;
    if f.by_pointer
        || f.reverse
        || f.pointer_if.is_some()
        || f.reverse_if.is_some()
        || f.iterator.is_some()
        || f.backtick_names
    {
        return None;
    }
    let ForOver::Collection(array) = &f.over else {
        return None;
    };
    if !matches!(types.kind(cx.ty(array)?), TypeKind::Array { .. }) {
        return None;
    }
    let value = f.it.map_or_else(|| Sym::intern("it"), |i| i.name);
    let index = f.index.map_or_else(|| Sym::intern("it_index"), |i| i.name);
    let S::If {
        cond,
        then_branch,
        else_branch: None,
    } = &only_statement(&f.body)?.kind
    else {
        return None;
    };
    let E::Binary(BinOp::Eq, a, b) = &cond.kind else {
        return None;
    };
    let is_value = |e: &Expr| matches!(e.kind, E::Ident(n) if n == value);
    let wanted = match (is_value(a), is_value(b)) {
        (true, false) => b,
        (false, true) => a,
        _ => return None,
    };
    if mentions(wanted, value, index) || !super::pure(wanted) || cx.ty(a)? != cx.ty(b)? {
        return None;
    }
    // Assignments to outside variables, then `break`.
    let steps = match &then_branch.kind {
        S::Block(b) => b.stmts.as_slice(),
        _ => std::slice::from_ref(&**then_branch),
    };
    let (last, assigns) = steps.split_last()?;
    if !matches!(last.kind, S::Break(None)) {
        return None;
    }
    let mut sets_index = false;
    for s in assigns {
        let S::Assign {
            op: AssignOp::Assign,
            lhs,
            rhs,
        } = &s.kind
        else {
            return None;
        };
        let ([target], [source]) = (lhs.as_slice(), rhs.as_slice()) else {
            return None;
        };
        if !matches!(target.kind, E::Ident(n) if n != value && n != index) {
            return None;
        }
        match &source.kind {
            E::Bool(_) => {}
            E::Ident(n) if *n == index => sets_index = true,
            _ => return None,
        }
    }
    let wanted_src = cx.whole_src(wanted.span).trim();
    let array_src = cx.whole_src(array.span).trim();
    let call = format!("array_find({array_src}, {wanted_src})");
    let help = if sets_index {
        format!("`found, index := {call};` returns whether it was found and where")
    } else {
        format!("`{call}` returns whether it was found, and the index after that")
    };
    let header = jaic::source::Span::new(
        cx.file,
        stmt.span.start as usize,
        f.body.span.start as usize,
    );
    Some(super::finding(
        header,
        format!("this loop looks for `{wanted_src}` in `{array_src}` by hand"),
        Some(help),
    ))
}

/// The expression names the loop's value or index.
fn mentions(e: &Expr, value: Sym, index: Sym) -> bool {
    let mut found = false;
    let mut stack = Vec::new();
    crate::syntax::walk_node(crate::syntax::Node::Expr(e), &mut stack, &mut |n, _| {
        if let Some(x) = n.expr()
            && matches!(x.kind, E::Ident(name) if name == value || name == index)
        {
            found = true;
        }
        !found
    });
    found
}
