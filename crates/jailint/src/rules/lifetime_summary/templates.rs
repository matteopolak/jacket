//! Lexical context effects of macro templates do not need an executable instance.
use jaic::ast::walk::{Node, walk};
use jaic::ast::{Expr, ExprKind as E, ProcLit, StmtKind as S, UnOp};

pub(super) fn pushes_context(proc: &ProcLit) -> bool {
    if !proc.header.flags.expand() {
        return false;
    }
    let Some(body) = &proc.body else {
        return false;
    };
    let mut found = false;
    walk(&body.stmts, &mut |node, _| {
        if let Node::Stmt(s) = node {
            found |= match &s.kind {
                S::PushContext(_) | S::PushContextDefer(_) => true,
                S::Assign {
                    lhs, ..
                } => lhs.iter().any(context_target),
                _ => false,
            };
        }
        !found
    });
    found
}

fn context_target(expr: &Expr) -> bool {
    match &expr.kind {
        E::Context => true,
        E::Member(base, _) | E::Index(base, _) | E::Unary(UnOp::Deref, base) => {
            context_target(base)
        }
        _ => false,
    }
}
