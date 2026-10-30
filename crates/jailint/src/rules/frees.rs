//! What happens to a variable after a `@FreesArgument` call frees it, shared by
//! `use_after_free` and `double_free`.
//!
//! The scan looks at one statement list at a time. A *free* is a statement that is only a call
//! of a `@FreesArgument` procedure whose first argument is a plain local or parameter (`free(p);`,
//! not `free(p.data)` or `array_free(*xs)`, and not a `defer`). The statements after it in the
//! same list run after the free, so the first one that mentions the variable decides:
//!
//! - another free of it by a call of the same procedure, as a statement of its own:
//!   [`After::Free`];
//! - a read of it: [`After::Use`];
//! - anything else (an assignment that gives it a new value, its address taken, a `return`, `break`
//!   or `continue` before any mention, code the scan cannot follow such as `#insert`, `using`, a
//!   macro call or a nested free call): nothing is reported.
//!
//! What counts as a read depends on what the freeing procedure's first parameter is. A raw
//! pointer (`*void`, `*u8`), a string, an array or a handle is the thing released (`free(p)`,
//! `close(fd)`): any read afterwards counts. A pointer to a struct (`deinit(*Table)`,
//! `close(*Response)`) releases what the struct owns and leaves the struct itself, which may be
//! initialised again or handed to the allocator: only reading through it (`p.field`, `p.*`)
//! counts, and writing through it ends the check.
//!
//! Variables whose address is taken anywhere in the procedure are skipped, as are procedures that
//! use `using`. Equality tests against the variable (`p == null`) are not reads, and `defer`
//! statements are not looked into.
use super::{PlaceUse, callee_procs, calls_noted, place_use, statement_lists};
use crate::syntax::Cx;
use jaic::ast::walk::{Node, walk, walk_node};
use jaic::ast::{BinOp, Expr, ExprKind as E, Stmt, StmtKind as S, UnOp};
use jaic::ide::Found;
use jaic::sema::scope::EntityKind;
use jaic::sema::{EntityId, ProcId};
use jaic::source::Span;
use jaic::types::TypeKind;

/// The first thing that happens to a freed variable.
pub(super) enum After {
    /// It is read at this span.
    Use(Span),
    /// It is freed again by the call at this span.
    Free(Span),
}

pub(super) struct Freed<'a> {
    /// The name as written.
    pub name: &'a str,
    pub after: After,
}

/// A statement that frees a variable.
struct Free<'a> {
    var: EntityId,
    name: &'a str,
    span: Span,
    procs: Found<ProcId>,
    /// The parameter is the thing released (a raw pointer, string, array or handle) rather than
    /// a struct whose contents are released.
    whole: bool,
}

/// Every free in the file's procedures that is followed by a use or another free.
pub(super) fn scan<'a>(cx: &'a Cx, report: &mut dyn FnMut(Freed<'a>)) {
    for p in &cx.procs {
        if !p.clean || p.is_macro || p.header.params.iter().any(|q| q.using) {
            continue;
        }
        if p.body
            .stmts
            .iter()
            .any(|s| super::any_stmt(s, &mut |s| matches!(s.kind, S::Using(_))))
        {
            continue;
        }
        let mut addressed: Vec<EntityId> = Vec::new();
        walk(&p.body.stmts, &mut |n, chain| {
            if let Node::Expr(e) = n
                && let E::Ident(_) = e.kind
                && place_use(e, chain) == PlaceUse::Address
            {
                addressed.extend_from_slice(&cx.entities(e.span));
            }
            true
        });
        statement_lists(&p.body.stmts, &mut |list: &[Stmt]| {
            for (k, s) in list.iter().enumerate() {
                let Some(free) = direct_free(cx, s) else {
                    continue;
                };
                if addressed.contains(&free.var) {
                    continue;
                }
                if let Some(after) = next_event(cx, &free, &list[k + 1..]) {
                    report(Freed {
                        name: free.name,
                        after,
                    });
                }
            }
        });
    }
}

/// The statement is only a call of a `@FreesArgument` procedure with a local or parameter as
/// its first argument.
fn direct_free<'a>(cx: &'a Cx, s: &Stmt) -> Option<Free<'a>> {
    let S::Expr(e) = &s.kind else {
        return None;
    };
    let E::Call {
        callee,
        args,
        ..
    } = &e.kind
    else {
        return None;
    };
    if !calls_noted(cx, callee, "FreesArgument") {
        return None;
    }
    let first = args.first()?;
    if first.spread || first.context {
        return None;
    }
    let E::Ident(_) = first.value.kind else {
        return None;
    };
    let procs = callee_procs(cx, callee);
    Some(Free {
        var: local_of(cx, &first.value)?,
        name: cx.src(first.value.span).trim(),
        span: e.span,
        whole: procs.iter().all(|&p| releases_whole_argument(cx, p)),
        procs,
    })
}

/// The first parameter of `proc` is a raw pointer, string, array or handle: the thing a call
/// releases. A typed pointer (to a struct, or one the signature leaves open) is an object whose
/// contents are released.
fn releases_whole_argument(cx: &Cx, proc: ProcId) -> bool {
    let types = &cx.compiler.ty.types;
    let Some(sig) = &cx.compiler.pr.procs[proc.0 as usize].sig else {
        return false;
    };
    let Some(first) = sig.params.first() else {
        return false;
    };
    match types.kind(types.repr_struct(first.ty)) {
        TypeKind::Pointer(inner) => !matches!(
            types.kind(types.repr_struct(*inner)),
            TypeKind::Struct(_) | TypeKind::PolyParam | TypeKind::PolyStruct { .. }
        ),
        TypeKind::Struct(_)
        | TypeKind::PolyParam
        | TypeKind::PolyStruct {
            ..
        } => false,
        _ => true,
    }
}

/// The one local variable or parameter an identifier names.
pub(super) fn local_of(cx: &Cx, e: &Expr) -> Option<EntityId> {
    match &*cx.entities(e.span) {
        [only] if matches!(cx.compiler.entity(*only).kind, EntityKind::Local { .. }) => Some(*only),
        _ => None,
    }
}

/// Code the scan cannot follow: it may touch the variable by name or leave the procedure.
pub(super) fn opaque(cx: &Cx, s: &Stmt) -> bool {
    let mut found = false;
    let mut stack = Vec::new();
    walk_node(Node::Stmt(s), &mut stack, &mut |n, _| {
        match n {
            Node::Stmt(t) => {
                found |= matches!(t.kind, S::Insert(_) | S::Using(_));
            }
            Node::Expr(e) => match &e.kind {
                E::Insert(_) | E::Asm(_) | E::Backtick(_) => found = true,
                E::Call {
                    callee, ..
                } => {
                    found |= callee_procs(cx, callee)
                        .iter()
                        .any(|&p| cx.compiler.pr.procs[p.0 as usize].lit.header.flags.expand());
                }
                _ => {}
            },
        }
        !found
    });
    found
}

/// What the statements after a free do with the variable first.
fn next_event(cx: &Cx, free: &Free, rest: &[Stmt]) -> Option<After> {
    for s in rest {
        if opaque(cx, s) {
            return None;
        }
        if let Some(again) = direct_free(cx, s)
            && again.var == free.var
        {
            return (*again.procs == *free.procs).then_some(After::Free(again.span));
        }
        let mut event: Option<Option<After>> = None;
        let mut stack = Vec::new();
        walk_node(Node::Stmt(s), &mut stack, &mut |n, chain| {
            if event.is_some() {
                return false;
            }
            if let Node::Stmt(t) = n
                && matches!(t.kind, S::Defer { .. })
            {
                return false;
            }
            let Node::Expr(e) = n else {
                return true;
            };
            if !matches!(e.kind, E::Ident(_)) || !cx.entities(e.span).contains(&free.var) {
                return true;
            }
            event = Some(match place_use(e, chain) {
                PlaceUse::Write | PlaceUse::Address => None,
                // The struct itself may be handed on (to the allocator, say); only reaching
                // into it is a use.
                PlaceUse::Read if !free.whole && !projected(e, chain) => return true,
                // `p.* = .{};` starts the struct over.
                PlaceUse::Read if !free.whole && assigned_through(e, chain) => None,
                PlaceUse::Read if compared(chain) => return true,
                // A nested free is not a statement of its own: the path is not straight-line.
                PlaceUse::Read if super::is_call_argument(e, chain) && freeing_call(cx, chain) => {
                    None
                }
                PlaceUse::Read => Some(After::Use(e.span)),
            });
            true
        });
        if let Some(found) = event {
            return found;
        }
        if matches!(
            s.kind,
            S::Return { .. } | S::Break(_) | S::Continue(_) | S::Remove(_)
        ) {
            return None;
        }
    }
    None
}

/// The identifier is the base of a member access, an index or a dereference.
fn projected(e: &Expr, chain: &[Node<'_>]) -> bool {
    match chain.last() {
        Some(Node::Expr(p)) => match &p.kind {
            E::Member(base, _) | E::Index(base, _) => base.span == e.span,
            E::Unary(UnOp::Deref, inner) => inner.span == e.span,
            _ => false,
        },
        _ => false,
    }
}

/// The identifier heads an access path (`p.*`, `p.field`, `p[i]`) that is assigned to.
fn assigned_through(e: &Expr, chain: &[Node<'_>]) -> bool {
    let mut cur = e;
    for node in chain.iter().rev() {
        match node {
            Node::Expr(p) => match &p.kind {
                E::Member(base, _) | E::Index(base, _) if base.span == cur.span => cur = p,
                E::Unary(UnOp::Deref, inner) if inner.span == cur.span => cur = p,
                _ => return false,
            },
            Node::Stmt(s) => {
                return matches!(&s.kind, S::Assign { lhs, .. } if lhs.iter().any(|l| l.span == cur.span));
            }
        }
    }
    false
}

/// The identifier is a direct operand of `==` or `!=`.
fn compared(chain: &[Node<'_>]) -> bool {
    matches!(
        chain.last(),
        Some(Node::Expr(Expr {
            kind: E::Binary(BinOp::Eq | BinOp::Ne, ..),
            ..
        }))
    )
}

/// The innermost call around the identifier is a call of a `@FreesArgument` procedure.
fn freeing_call(cx: &Cx, chain: &[Node<'_>]) -> bool {
    matches!(chain.last(), Some(Node::Expr(Expr { kind: E::Call { callee, .. }, .. }))
        if calls_noted(cx, callee, "FreesArgument"))
}
