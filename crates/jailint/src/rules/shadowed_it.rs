//! `shadowed_it`: a `for` inside another hides the outer loop's `it`, and both are used.
//!
//! ```jai
//! for rows {
//!     for cells_of(it) {
//!         total += it;       // the cell; the row is out of reach
//!     }
//!     print("%\n", it);      // the row
//! }
//! ```
//!
//! The same name means different things a few lines apart, and reaching for the row inside
//! the inner loop silently gets the cell. Name one of the iterators (`for row: rows`).
//!
//! Fires when an unnamed `for` nested in an unnamed loop's body uses its own `it`, and the
//! outer loop reads its `it` again after the inner loop. Copying the outer `it` to a name
//! before the inner loop (`row := it;`), the usual idiom, is not reported. Uses are told
//! apart by what the compiler resolved each `it` to.
use crate::Finding;
use crate::syntax::{Cx, Node, walk, walk_node};
use jaic::ast::{For, Stmt, StmtKind as S};
use jaic::intern::Sym;
use jaic::sema::EntityId;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    let it = Sym::intern("it");
    for p in &cx.procs {
        if !p.clean || p.is_macro {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            let Some(outer) = n.stmt() else {
                return true;
            };
            let S::For(f) = &outer.kind else {
                return true;
            };
            if !binds_it(f, it) {
                return true;
            }
            let outer_vars = loop_vars(cx, outer, it);
            if outer_vars.is_empty() {
                return true;
            }
            // Inner loops binding `it` too (the first level of them).
            let mut stack = Vec::new();
            walk_node(Node::Stmt(&f.body), &mut stack, &mut |m, _| {
                let Some(inner) = m.stmt() else {
                    return true;
                };
                let S::For(g) = &inner.kind else {
                    return true;
                };
                if !binds_it(g, it) {
                    return true;
                }
                let inner_vars = loop_vars(cx, inner, it);
                // The outer `it` is read again after the inner loop: `it` changes meaning
                // and back. (Copying it to a name before the inner loop is the usual idiom.)
                if !inner_vars.is_empty()
                    && uses(cx, &g.body, &inner_vars)
                    && uses_after(cx, &f.body, &outer_vars, inner.span.end)
                {
                    let (line, _) = crate::render::line_col(cx.text, outer.span.start as usize);
                    let header_end = match &g.over {
                        jaic::ast::ForOver::Range(_, b) => b.span.end,
                        jaic::ast::ForOver::Collection(c) => c.span.end,
                    };
                    out.push(Finding {
                        start: inner.span.start as usize,
                        end: header_end as usize,
                        message: format!(
                            "this loop's `it` hides the `it` of the loop on line {line}, which is also used"
                        ),
                        label: None,
                        help: Some("name one of the iterators: `for name: ...`".into()),
                        fix: None,
                    });
                }
                false
            });
            true
        });
    }
}

fn binds_it(f: &For, it: Sym) -> bool {
    !f.backtick_names && f.iterator.is_none() && f.it.is_none_or(|i| i.name == it)
}

fn loop_vars(cx: &Cx, stmt: &Stmt, it: Sym) -> Vec<EntityId> {
    cx.facts
        .locals
        .get(&stmt.span)
        .map(|v| {
            v.iter()
                .copied()
                .filter(|&e| cx.compiler.entity(e).name == it)
                .collect()
        })
        .unwrap_or_default()
}

/// Some identifier under `body` resolved to one of `vars`.
fn uses(cx: &Cx, body: &Stmt, vars: &[EntityId]) -> bool {
    uses_after(cx, body, vars, 0)
}

/// Some identifier under `body` starting at or after `from` resolved to one of `vars`.
fn uses_after(cx: &Cx, body: &Stmt, vars: &[EntityId], from: u32) -> bool {
    let mut found = false;
    let mut stack = Vec::new();
    walk_node(Node::Stmt(body), &mut stack, &mut |n, _| {
        if let Some(e) = n.expr()
            && matches!(e.kind, jaic::ast::ExprKind::Ident(_))
            && e.span.start >= from
            && cx.entities(e.span).iter().any(|x| vars.contains(x))
        {
            found = true;
        }
        !found
    });
    found
}
