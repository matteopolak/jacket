//! Phase-one lifetime engine: a source CFG backed by checked name/type facts.
//!
//! Joins union possible pointer states; a use is reported only if the sole possible state is
//! invalid. Entry contributes unknown, including on loop backedges. Derivation kills earlier
//! invalidation, and replacing a region forgets its identity. No bounded loop unrolling.
//!
//! Unsupported statements/expressions reject the whole body. In particular defer, macros,
//! inserts, polymorphs and unchecked static branches need semantic expansion facts first.
//! Regions are plain resizable-array variables; fields, aliases and pointer copies are future
//! work. This is opt-in and does not replace the existing rules yet.
use super::{callee_procs, calls_noted, finding};
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::walk::{Node, walk, walk_node};
use jaic::ast::{
    AssignOp, BinOp, DeclKind, Expr, ExprKind as E, ForOver, Stmt, StmtKind as S, UnOp,
};
use jaic::sema::EntityId;
use jaic::sema::scope::EntityKind;
use jaic::source::Span;
use jaic::types::{ArrayKind, TypeKind};
use std::collections::VecDeque;

#[derive(Clone, Copy)]
enum Op {
    Nop,
    Use(EntityId, Span),
    Set(EntityId, Option<EntityId>, Span),
    Grow(EntityId, Span),
    Forget(EntityId),
}

struct Block {
    op: Op,
    next: Vec<usize>,
}

struct Graph<'a, 'c> {
    cx: &'a Cx<'c>,
    blocks: Vec<Block>,
}

fn entity(cx: &Cx, e: &Expr) -> Option<EntityId> {
    if !matches!(e.kind, E::Ident(_)) {
        return None;
    }
    let ids = cx.entities(e.span);
    (ids.len() == 1).then(|| ids[0])
}

fn region(cx: &Cx, e: &Expr) -> Option<EntityId> {
    let id = entity(cx, e)?;
    if !matches!(cx.compiler.entity(id).kind, EntityKind::Local { .. }) {
        return None;
    }
    let ty = cx.ty(e)?;
    matches!(
        cx.compiler.ty.types.kind(ty),
        TypeKind::Array {
            kind: ArrayKind::Resizable,
            ..
        }
    )
    .then_some(id)
}

fn source(cx: &Cx, e: &Expr) -> Option<EntityId> {
    if let E::Unary(UnOp::Star, inner) = &e.kind
        && let E::Index(array, _) = &inner.kind
    {
        return region(cx, array);
    }
    None
}

impl Graph<'_, '_> {
    fn add(&mut self, op: Op, next: Vec<usize>) -> usize {
        let id = self.blocks.len();
        self.blocks.push(Block {
            op,
            next,
        });
        id
    }

    fn expr(&mut self, e: &Expr, next: usize) -> Option<usize> {
        // Type queries and other constant results need not evaluate their written operands.
        // Dropping any hidden side effects here loses proof, which is safe for must analysis.
        if self.cx.constant(e) {
            return Some(next);
        }
        Some(match &e.kind {
            E::Ident(_) => match entity(self.cx, e) {
                Some(id) => self.add(Op::Use(id, e.span), vec![next]),
                None => next,
            },
            E::Int(..)
            | E::Float(..)
            | E::Str(_)
            | E::Bool(_)
            | E::Null
            | E::Char(_)
            | E::Uninit
            | E::InferredMember(_) => next,
            E::Unary(UnOp::Star, inner) => {
                // A call receiving *p can overwrite p, including through a saved alias later.
                let next =
                    entity(self.cx, inner).map_or(next, |id| self.add(Op::Forget(id), vec![next]));
                self.expr(inner, next)?
            }
            E::Unary(_, inner) | E::Member(inner, _) => self.expr(inner, next)?,
            E::Cast {
                value, ..
            } => self.expr(value, next)?,
            E::Index(a, b) => {
                let tail = self.expr(b, next)?;
                self.expr(a, tail)?
            }
            E::Binary(BinOp::And | BinOp::Or, a, b) => {
                let rhs = self.expr(b, next)?;
                let branch = self.add(Op::Nop, vec![rhs, next]);
                self.expr(a, branch)?
            }
            E::Binary(BinOp::Eq | BinOp::Ne, a, b) => {
                // Comparing a stale pointer value (typically with null) does not access its
                // storage. Still evaluate complex operands for dereferences and calls.
                let tail = if matches!(b.kind, E::Ident(_)) {
                    next
                } else {
                    self.expr(b, next)?
                };
                if matches!(a.kind, E::Ident(_)) {
                    tail
                } else {
                    self.expr(a, tail)?
                }
            }
            E::Binary(_, a, b) => {
                let tail = self.expr(b, next)?;
                self.expr(a, tail)?
            }
            E::Call {
                callee,
                args,
                ..
            } => {
                if callee_procs(self.cx, callee).iter().any(|p| {
                    self.cx.compiler.pr.procs[p.0 as usize]
                        .lit
                        .header
                        .flags
                        .expand()
                }) || args
                    .iter()
                    .any(|a| a.context || a.spread || a.name.is_some())
                {
                    return None;
                }
                let mut tail = next;
                if calls_noted(self.cx, callee, "GrowsArray")
                    && let Some(arg) = args.first()
                    && let E::Unary(UnOp::Star, array) = &arg.value.kind
                    && let Some(id) = region(self.cx, array)
                {
                    tail = self.add(Op::Grow(id, e.span), vec![tail]);
                }
                // Forget address-exposed locals separately, below; the normal *array argument
                // must retain its region for this annotated call.
                for arg in args.iter().rev() {
                    if let E::Unary(UnOp::Star, inner) = &arg.value.kind {
                        tail = self.expr(inner, tail)?;
                    } else {
                        tail = self.expr(&arg.value, tail)?;
                    }
                }
                self.expr(callee, tail)?
            }
            E::StructLit {
                fields, ..
            } => {
                let mut tail = next;
                for field in fields.iter().rev() {
                    if field.target.is_some() {
                        return None;
                    }
                    tail = self.expr(&field.value, tail)?;
                }
                tail
            }
            E::ArrayLit {
                elems, ..
            } => {
                let mut tail = next;
                for elem in elems.iter().rev() {
                    tail = self.expr(elem, tail)?;
                }
                tail
            }
            _ => return None,
        })
    }

    fn list(
        &mut self,
        stmts: &[Stmt],
        mut next: usize,
        jumps: Option<(usize, usize)>,
    ) -> Option<usize> {
        for s in stmts.iter().rev() {
            next = self.stmt(s, next, jumps)?;
        }
        Some(next)
    }

    fn stmt(&mut self, s: &Stmt, next: usize, jumps: Option<(usize, usize)>) -> Option<usize> {
        Some(match &s.kind {
            S::Block(b) => self.list(&b.stmts, next, jumps)?,
            S::Empty => next,
            S::Decl(d) if d.kind == DeclKind::Var && d.names.len() == 1 && d.rare.is_none() => {
                let ids = self.cx.locals(d.names[0].span);
                if ids.len() != 1 {
                    return None;
                }
                let set = self.add(
                    Op::Set(
                        ids[0],
                        d.value.as_ref().and_then(|v| source(self.cx, v)),
                        d.value.as_ref().map_or(d.names[0].span, |v| v.span),
                    ),
                    vec![next],
                );
                match &d.value {
                    Some(v) => self.expr(v, set)?,
                    None => set,
                }
            }
            S::Assign {
                op: AssignOp::Assign,
                lhs,
                rhs,
            } if lhs.len() == 1 && rhs.len() == 1 => {
                let set = if let Some(id) = entity(self.cx, &lhs[0]) {
                    self.add(
                        Op::Set(id, source(self.cx, &rhs[0]), rhs[0].span),
                        vec![next],
                    )
                } else {
                    // Writes through a pointer do not overwrite the pointer itself.
                    let tail = match &lhs[0].kind {
                        E::Member(base, _) => region(self.cx, base)
                            .map_or(next, |id| self.add(Op::Forget(id), vec![next])),
                        _ => next,
                    };
                    self.expr(&lhs[0], tail)?
                };
                self.expr(&rhs[0], set)?
            }
            S::Assign {
                op: AssignOp::Op(_),
                lhs,
                rhs,
            } if lhs.len() == 1 && rhs.len() == 1 => {
                // Pointer arithmetic loses our precise derivation.
                if let Some(id) = entity(self.cx, &lhs[0]) {
                    let clear = self.add(Op::Set(id, None, s.span), vec![next]);
                    let tail = self.expr(&rhs[0], clear)?;
                    self.expr(&lhs[0], tail)?
                } else {
                    let tail = self.expr(&rhs[0], next)?;
                    self.expr(&lhs[0], tail)?
                }
            }
            S::Expr(e) => self.expr(e, next)?,
            S::Defer {
                body,
                backtick: false,
            } => {
                // A release receiving a descriptor by value cannot rebind the local array.
                // Its eventual free can only add invalidation, so ignoring that effect is a
                // safe underapproximation for this growth-only must pass. Other cleanup needs
                // explicit scope-exit edges (and remains unsupported).
                let S::Expr(Expr {
                    kind:
                        E::Call {
                            callee,
                            args,
                            ..
                        },
                    ..
                }) = &body.kind
                else {
                    return None;
                };
                if args.len() != 1
                    || args[0].name.is_some()
                    || args[0].context
                    || args[0].spread
                    || region(self.cx, &args[0].value).is_none()
                    || !calls_noted(self.cx, callee, "FreesArgument")
                    || calls_noted(self.cx, callee, "GrowsArray")
                    || callee_procs(self.cx, callee).iter().any(|p| {
                        self.cx.compiler.pr.procs[p.0 as usize]
                            .lit
                            .header
                            .flags
                            .expand()
                    })
                {
                    return None;
                }
                next
            }
            S::Return {
                values,
                backtick: false,
            } => {
                let mut tail = 0;
                for v in values.iter().rev() {
                    tail = self.expr(&v.value, tail)?;
                }
                tail
            }
            S::Break(None) => jumps?.0,
            S::Continue(None) => jumps?.1,
            S::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let yes = self.stmt(then_branch, next, jumps)?;
                let no = match else_branch {
                    Some(s) => self.stmt(s, next, jumps)?,
                    None => next,
                };
                let edges = match cond.kind {
                    E::Bool(true) => vec![yes],
                    E::Bool(false) => vec![no],
                    _ => vec![yes, no],
                };
                let fork = self.add(Op::Nop, edges);
                self.expr(cond, fork)?
            }
            S::While(w) if w.label.is_none() && !w.bind_label => {
                let fork = self.add(Op::Nop, vec![]);
                let cond = self.expr(&w.cond, fork)?;
                let body = self.stmt(&w.body, cond, Some((next, cond)))?;
                self.blocks[fork].next = match w.cond.kind {
                    E::Bool(true) => vec![body],
                    E::Bool(false) => vec![next],
                    _ => vec![body, next],
                };
                cond
            }
            S::For(f)
                if f.iterator.is_none()
                    && f.pointer_if.is_none()
                    && f.reverse_if.is_none()
                    && !f.backtick_names
                    && f.flags.is_empty() =>
            {
                let fork = self.add(Op::Nop, vec![]);
                let mut body = self.stmt(&f.body, fork, Some((next, fork)))?;
                if f.by_pointer {
                    let ForOver::Collection(array) = &f.over else {
                        return None;
                    };
                    let r = region(self.cx, array)?;
                    // Implicit iterators have no declaration-name span. Resolved uses carry
                    // the entity whose source span is this loop, including under shadowing.
                    let mut ids = Vec::new();
                    walk_node(Node::Stmt(&f.body), &mut Vec::new(), &mut |n, _| {
                        if let Node::Expr(e) = n
                            && let Some(id) = entity(self.cx, e)
                            && self.cx.compiler.entity(id).span == s.span
                            && self.cx.compiler.entity(id).name
                                == f.it
                                    .map_or_else(|| jaic::intern::Sym::intern("it"), |i| i.name)
                            && !ids.contains(&id)
                        {
                            ids.push(id);
                        }
                        true
                    });
                    for id in ids {
                        body = self.add(Op::Set(id, Some(r), array.span), vec![body]);
                    }
                }
                self.blocks[fork].next = vec![body, next];
                match &f.over {
                    ForOver::Collection(e) => self.expr(e, fork)?,
                    ForOver::Range(a, b) => {
                        let tail = self.expr(b, fork)?;
                        self.expr(a, tail)?
                    }
                }
            }
            _ => return None,
        })
    }
}

const UNKNOWN: u8 = 1;
const LIVE: u8 = 2;
const DEAD: u8 = 4;

pub(super) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    let mut cfgs = 0;
    let mut solved = 0;
    for p in &cx.procs {
        if !p.typed() || p.header.params.iter().any(|p| p.using) {
            continue;
        }
        let mut graph = Graph {
            cx,
            blocks: vec![Block {
                op: Op::Nop,
                next: vec![],
            }],
        };
        let Some(entry) = graph.list(&p.body.stmts, 0, None) else {
            continue;
        };
        cfgs += 1;
        let mut candidates = Vec::new();
        for b in &graph.blocks {
            if let Op::Set(id, Some(r), origin) = b.op {
                candidates.push((id, r, origin));
            }
        }
        for (pointer, region, origin) in candidates {
            // Taking the pointer's address permits invisible reassignments. Taking the array's
            // address is expected, but only a noted grow may act on it in this slice.
            let mut exposed = false;
            walk(&p.body.stmts, &mut |n, chain| {
                if let Node::Expr(e) = n
                    && let E::Unary(UnOp::Star, inner) = &e.kind
                {
                    let id = entity(cx, inner);
                    exposed |= id == Some(pointer);
                    if id == Some(region) {
                        let annotated = matches!(chain.last(), Some(Node::Expr(Expr {
                            kind: E::Call { callee, args, .. }, ..
                        })) if args.first().is_some_and(|arg| arg.value.span == e.span)
                            && calls_noted(cx, callee, "GrowsArray"));
                        exposed |= !annotated;
                    }
                }
                true
            });
            if exposed {
                continue;
            }
            solved += 1;
            solve(cx, &graph.blocks, entry, pointer, region, origin, out);
        }
    }
    if jacket_config::env::LIFETIME_STATS.is_set() {
        eprintln!(
            "lifetime-stats file={} bodies={} cfgs={cfgs} candidates={solved}",
            cx.compiler.src.sources.get(cx.file).path,
            cx.procs.len()
        );
    }
}

fn solve(
    cx: &Cx,
    blocks: &[Block],
    entry: usize,
    pointer: EntityId,
    region: EntityId,
    origin: Span,
    out: &mut Vec<Finding>,
) {
    let mut states = vec![0u8; blocks.len()];
    // Evidence is independent from the lattice, so two branches can invalidate at different
    // calls and still establish DEAD. A source-order witness suffices for this spike.
    let mut witnesses: Vec<Option<Span>> = vec![None; blocks.len()];
    states[entry] = UNKNOWN;
    let mut queue = VecDeque::from([entry]);
    while let Some(i) = queue.pop_front() {
        let mut state = states[i];
        let mut witness = witnesses[i];
        match blocks[i].op {
            Op::Set(id, r, at) if id == pointer => {
                state = if r == Some(region) && at == origin {
                    LIVE
                } else {
                    UNKNOWN
                };
                witness = None;
            }
            Op::Set(id, _, _) | Op::Forget(id) if id == region || id == pointer => {
                state = UNKNOWN;
                witness = None;
            }
            Op::Grow(id, at) if id == region && state & LIVE != 0 => {
                state = (state & !LIVE) | DEAD;
                witness = Some(at);
            }
            _ => {}
        }
        for &next in &blocks[i].next {
            let joined = states[next] | state;
            let evidence = witnesses[next].or(witness);
            if joined != states[next] || evidence != witnesses[next] {
                states[next] = joined;
                witnesses[next] = evidence;
                queue.push_back(next);
            }
        }
    }
    for (i, b) in blocks.iter().enumerate() {
        if states[i] == DEAD
            && let Op::Use(id, at) = b.op
            && id == pointer
        {
            let Some(growth) = witnesses[i] else {
                continue;
            };
            let line = |s: Span| cx.lines.line_col(cx.text, s.start as usize).0;
            out.push(finding(at, format!("`{}` points into `{}`, which may be resized: {}",
                cx.src(at), cx.compiler.entity(region).name.as_str(), cx.src(growth).trim()),
                Some(format!("pointer derived at line {}; invalidating call at line {}; take the pointer again after growth",
                    line(origin), line(growth)))));
        }
    }
}
