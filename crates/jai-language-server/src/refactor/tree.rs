//! Syntax tree helpers for refactorings.
use jaic::ast::{Block, Expr, ExprKind as E, ProcHeader, Stmt, StmtKind as S, UnOp};
use jaic::intern::Sym;
use jailint::syntax::{Node, walk};

/// The root variable of an assignable place: `a` of `a.b[2]`.
pub(super) fn root(e: &Expr) -> Option<Sym> {
    match &e.kind {
        E::Ident(n) => Some(*n),
        E::Member(a, _) | E::Index(a, _) | E::Unary(UnOp::Deref | UnOp::Star, a) => root(a),
        _ => None,
    }
}

/// Every procedure literal with a body under `stmts`, at any depth.
pub(super) fn proc_bodies<'a>(stmts: &'a [Stmt], out: &mut Vec<(&'a ProcHeader, &'a Block)>) {
    walk(stmts, &mut |n, _| {
        let Some(x) = n.expr() else {
            return true;
        };
        match &x.kind {
            E::Proc(lit) => {
                if let Some(body) = &lit.body {
                    out.push((&lit.header, body));
                    proc_bodies(&body.stmts, out);
                }
                false
            }
            E::Struct(st) => {
                proc_bodies(&st.body, out);
                false
            }
            _ => true,
        }
    });
}

/// A statement that sits in a list of statements, so more can be added before it: the nodes
/// above it end in a block, a case, or nothing (the top of a body).
pub(super) fn in_list(parents: &[Node<'_>]) -> bool {
    match parents.last() {
        None => true,
        Some(Node::Stmt(s)) => matches!(
            s.kind,
            S::Block(_) | S::Switch(_) | S::StaticSwitch(_) | S::StaticIf(_) | S::Case(_)
        ),
        Some(Node::Expr(e)) => matches!(e.kind, E::Block(_) | E::Run { .. } | E::Code(_)),
    }
}

/// The kinds of statement a hoisted or rewritten expression may belong to.
pub(super) fn plain_statement(s: &Stmt) -> bool {
    matches!(
        s.kind,
        S::Decl(_)
            | S::Expr(_)
            | S::Assign { .. }
            | S::If { .. }
            | S::Switch(_)
            | S::Return { .. }
            | S::For(_)
    )
}
