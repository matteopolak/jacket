//! `manual_index_counter`: a counter kept next to a `for` loop that always equals `it_index`.
//!
//! ```jai
//! n := 0;
//! for names {
//!     print("% %\n", n, it);
//!     n += 1;
//! }
//! ```
//!
//! The loop already counts: `it_index` is the position of `it`. A hand-kept counter is one more
//! thing to get wrong (a `continue` that skips the increment, a second increment).
//!
//! Fires when, in one block, `n := 0` (an `s64` starting at zero) comes before a forward
//! `for` over an array, nothing mentions `n` in between or after the loop, and the loop body
//! ends with `n += 1` (with no `continue` that could skip it) or starts with `defer n += 1;`.
//! Every other mention of `n` in the body must read it.
use super::{PlaceUse, place_use};
use crate::syntax::{Cx, Node, walk, walk_node};
use crate::{Edit, Finding, Fix};
use jaic::ast::{AssignOp, BinOp, Decl, DeclKind, ExprKind as E, ForOver, Stmt, StmtKind as S};
use jaic::intern::Sym;
use jaic::types::{TypeId, TypeKind};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.clean || p.is_macro {
            continue;
        }
        examine_block(cx, &p.body.stmts, p.body.span.end, out);
        walk(&p.body.stmts, &mut |n, _| {
            if let Some(s) = n.stmt()
                && let S::Block(b) = &s.kind
            {
                examine_block(cx, &b.stmts, b.span.end, out);
            }
            true
        });
    }
}

/// `n := 0;`: the counter's name.
fn counter(cx: &Cx, d: &Decl) -> Option<Sym> {
    if d.kind != DeclKind::Var || d.names.len() != 1 || d.backtick || d.using {
        return None;
    }
    let name = d.names[0];
    match &d.value {
        Some(v) if matches!(v.kind, E::Int(0)) => {}
        _ => return None,
    }
    // An `s64`, as `it_index` is.
    let ents = cx.facts.locals.get(&name.span)?;
    let all_s64 = ents.iter().all(|&e| {
        matches!(cx.compiler.entity(e).kind, jaic::sema::scope::EntityKind::Local { ty, .. } if ty == TypeId::S64)
    });
    (all_s64 && !ents.is_empty()).then_some(name.name)
}

/// `n += 1`.
fn is_increment(s: &Stmt, n: Sym) -> bool {
    matches!(&s.kind, S::Assign { op: AssignOp::Op(BinOp::Add), lhs, rhs }
        if lhs.len() == 1 && rhs.len() == 1
            && matches!(lhs[0].kind, E::Ident(x) if x == n)
            && matches!(rhs[0].kind, E::Int(1)))
}

fn examine_block(cx: &Cx, stmts: &[Stmt], block_end: u32, out: &mut Vec<Finding>) {
    for (k, s) in stmts.iter().enumerate() {
        let S::For(f) = &s.kind else {
            continue;
        };
        if f.reverse || f.reverse_if.is_some() || f.iterator.is_some() || f.backtick_names {
            continue;
        }
        let ForOver::Collection(c) = &f.over else {
            continue;
        };
        if !cx
            .ty(c)
            .is_some_and(|t| matches!(cx.compiler.types.kind(t), TypeKind::Array { .. }))
        {
            continue;
        }
        let S::Block(body) = &f.body.kind else {
            continue;
        };
        // The counter, declared earlier in this block.
        let Some((decl_stmt, name)) = stmts[..k].iter().rev().find_map(|d| match &d.kind {
            S::Decl(decl) => counter(cx, decl).map(|n| (d, n)),
            _ => None,
        }) else {
            continue;
        };
        let decl_name_span = match &decl_stmt.kind {
            S::Decl(d) => d.names[0].span,
            _ => continue,
        };
        // Nothing between the declaration and the loop, nor after it, mentions the counter.
        if cx.mentions(name, decl_stmt.span.end, s.span.start, decl_name_span)
            || cx.mentions(name, s.span.end, block_end, decl_name_span)
        {
            continue;
        }
        let first = body.stmts.first();
        let last = body.stmts.last();
        let (increment, deferred) = match (first, last) {
            (
                Some(Stmt {
                    kind:
                        S::Defer {
                            body: d,
                            backtick: false,
                        },
                    span,
                    ..
                }),
                _,
            ) if is_increment(d, name) => (*span, true),
            (_, Some(l)) if is_increment(l, name) => (l.span, false),
            _ => continue,
        };
        let index_name = f.index.map_or("it_index", |i| i.name.as_str());
        let mut uses = Vec::new();
        let mut ok = true;
        let mut rebinding: Vec<jaic::source::Span> = Vec::new();
        let mut stack = Vec::new();
        walk_node(Node::Stmt(&f.body), &mut stack, &mut |n, chain| {
            if !ok {
                return false;
            }
            match n {
                Node::Stmt(st) => {
                    if st.span == increment {
                        return false;
                    }
                    match &st.kind {
                        S::Remove(_) => ok = false,
                        // A `continue` of this loop skips an increment at the end.
                        S::Continue(label) if !deferred => {
                            // The walk starts at the body: any loop above is a nested one.
                            let nested = chain.iter().any(|a| {
                                matches!(
                                    a,
                                    Node::Stmt(Stmt {
                                        kind: S::For(_) | S::While { .. },
                                        ..
                                    })
                                )
                            });
                            if !nested || label.is_some() {
                                ok = false;
                            }
                        }
                        S::For(inner)
                            if f.index.is_none()
                                && inner.index.is_none_or(|i| i.name.as_str() == "it_index") =>
                        {
                            rebinding.push(st.span);
                        }
                        _ => {}
                    }
                    true
                }
                Node::Expr(e) => {
                    if let E::Ident(x) = e.kind
                        && x == name
                    {
                        let inside = rebinding
                            .iter()
                            .any(|r| r.start <= e.span.start && e.span.end <= r.end);
                        if inside || place_use(e, chain) != PlaceUse::Read {
                            ok = false;
                        } else {
                            uses.push(e.span);
                        }
                    }
                    true
                }
            }
        });
        if !ok {
            continue;
        }
        let mut edits = Vec::new();
        let (a, b) = cx.statement_removal(decl_stmt.span);
        edits.push(Edit {
            start: a,
            end: b,
            text: String::new(),
        });
        let (a, b) = cx.statement_removal(increment);
        edits.push(Edit {
            start: a,
            end: b,
            text: String::new(),
        });
        for u in &uses {
            edits.push(Edit {
                start: u.start as usize,
                end: u.end as usize,
                text: index_name.into(),
            });
        }
        let help = format!("use `{index_name}` instead of counting by hand");
        out.push(Finding {
            start: decl_name_span.start as usize,
            end: decl_name_span.end as usize,
            message: format!("`{name}` always equals `{index_name}` of the loop that follows"),
            label: None,
            help: Some(help.clone()),
            fix: Some(Fix {
                title: help,
                edits,
                machine_applicable: true,
            }),
        });
    }
}
