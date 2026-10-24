//! A walk over a procedure body that reports its block structure, for rules that need to know
//! which local names are visible where.
//!
//! The walk goes through blocks, `if`/`else`, loops, `switch` and `#if` bodies, `defer` and
//! `push_context`, in source order. It does not enter nested procedures (they cannot see the
//! locals around them) or expressions.
use jaic::ast::{Decl, DeclKind, Ident, Stmt, StmtKind as S, UsingStmt};

/// What a [`walk`] tells its caller.
pub(crate) trait Scopes {
    /// A block, loop body or `case` begins.
    fn enter(&mut self);

    /// The block that last began ends.
    fn leave(&mut self);

    /// A variable `name` of declaration `decl` (`x := ...`, `x: T`, `using x: T`).
    fn variable(&mut self, decl: &Decl, name: &Ident);

    /// A name that a statement binds for its body: a loop variable (named, not the implicit
    /// `it`) or the `name` of `while name := cond`.
    fn bound(&mut self, name: &Ident);

    /// A `using expr;` statement.
    fn using(&mut self, stmt: &Stmt, using: &UsingStmt);
}

pub(crate) fn walk(stmts: &[Stmt], v: &mut impl Scopes) {
    for s in stmts {
        stmt(s, v);
    }
}

/// `body` in a scope of its own.
fn scoped(body: &Stmt, v: &mut impl Scopes) {
    v.enter();
    match &body.kind {
        S::Block(b) => walk(&b.stmts, v),
        _ => stmt(body, v),
    }
    v.leave();
}

fn stmts_scoped(stmts: &[Stmt], v: &mut impl Scopes) {
    v.enter();
    walk(stmts, v);
    v.leave();
}

fn stmt(s: &Stmt, v: &mut impl Scopes) {
    match &s.kind {
        S::Decl(d) if d.kind == DeclKind::Var && !d.backtick => {
            for name in &d.names {
                v.variable(d, name);
            }
        }
        S::Using(u) => v.using(s, u),
        S::Block(b) => stmts_scoped(&b.stmts, v),
        S::If {
            then_branch,
            else_branch,
            ..
        } => {
            scoped(then_branch, v);
            if let Some(e) = else_branch {
                scoped(e, v);
            }
        }
        S::Switch(sw) | S::StaticSwitch(sw) => {
            for case in &sw.cases {
                stmts_scoped(&case.body, v);
            }
        }
        S::While(w) => {
            v.enter();
            if w.bind_label
                && let Some(label) = &w.label
            {
                v.bound(label);
            }
            scoped(&w.body, v);
            v.leave();
        }
        S::For(f) => {
            v.enter();
            for name in [&f.it, &f.index].into_iter().flatten() {
                v.bound(name);
            }
            scoped(&f.body, v);
            v.leave();
        }
        S::Defer {
            body, ..
        } => scoped(body, v),
        S::PushContext(p) => scoped(&p.body, v),
        S::StaticIf(si) => {
            stmts_scoped(&si.then_branch, v);
            stmts_scoped(&si.else_branch, v);
        }
        _ => {}
    }
}
