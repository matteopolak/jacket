//! A checked three-copy exchange preserves the identities of two pointer arguments.
use super::{Compiler, ProcId};
use jaic::ast::{AssignOp, DeclKind, Expr, ExprKind as E, StmtKind as S, UnOp};
use jaic::intern::Sym;

pub(in crate::rules) fn swaps_arguments(c: &Compiler, id: ProcId) -> bool {
    let proc = &c.pr.procs[id.0 as usize];
    if proc.body_state != jaic::sema::procs::BodyState::Done
        || !c
            .ed
            .facts()
            .and_then(|f| f.lifetime_bodies.get(&id))
            .is_some_and(|body| std::sync::Arc::ptr_eq(&body.syntax, &proc.lit))
    {
        return false;
    }
    let Some(sig) = &proc.sig else {
        return false;
    };
    if sig.params.len() != 2
        || !sig.returns.is_empty()
        || sig.params[0].ty != sig.params[1].ty
        || !c.ty.types.is_pointer(sig.params[0].ty)
    {
        return false;
    }
    let Some(body) = &proc.lit.body else {
        return false;
    };
    let [save, first, second] = body.stmts.as_slice() else {
        return false;
    };
    let mut params = proc.lit.header.params.iter().filter(|p| !p.baked);
    let Some(a) = params.next().and_then(|p| p.name).map(|n| n.name) else {
        return false;
    };
    let Some(b) = params.next().and_then(|p| p.name).map(|n| n.name) else {
        return false;
    };
    if a == b {
        return false;
    }
    let S::Decl(d) = &save.kind else {
        return false;
    };
    if d.kind != DeclKind::Var
        || d.names.len() != 1
        || d.using
        || d.backtick
        || d.names[0].name == a
        || d.names[0].name == b
        || d.value.as_ref().and_then(dereference) != Some(a)
    {
        return false;
    }
    let assign = |stmt: &jaic::ast::Stmt, target, source, deref| {
        let S::Assign {
            op: AssignOp::Assign,
            lhs,
            rhs,
        } = &stmt.kind
        else {
            return false;
        };
        lhs.len() == 1
            && rhs.len() == 1
            && dereference(&lhs[0]) == Some(target)
            && (if deref {
                dereference(&rhs[0])
            } else {
                identifier(&rhs[0])
            }) == Some(source)
    };
    assign(first, a, b, true) && assign(second, b, d.names[0].name, false)
}

fn identifier(e: &Expr) -> Option<Sym> {
    match e.kind {
        E::Ident(n) => Some(n),
        _ => None,
    }
}

fn dereference(e: &Expr) -> Option<Sym> {
    match &e.kind {
        E::Unary(UnOp::Deref, base) => identifier(base),
        _ => None,
    }
}
