//! Expression predicates and name suggestions.
use jaic::ast::{Expr, ExprKind as E, ProcHeader, Stmt, StmtKind as S, UnOp};
use jaic::intern::Sym;
use jailint::syntax::{Node, walk};

pub(super) fn header_poly(h: &ProcHeader) -> bool {
    h.params
        .iter()
        .any(|p| p.baked || p.auto_bake || p.ty.as_ref().is_some_and(jaic::sema::procs::has_poly))
        || h.returns
            .iter()
            .any(|r| r.ty.as_ref().is_some_and(jaic::sema::procs::has_poly))
}

/// Is `name` used by an expression in `stmts`?
pub(super) fn mentioned_in(stmts: &[Stmt], name: Sym) -> bool {
    let mut found = false;
    walk(stmts, &mut |n, _| {
        if let Some(x) = n.expr()
            && matches!(x.kind, E::Ident(i) if i == name)
        {
            found = true;
        }
        !found
    });
    found
}

pub(super) fn contains_range((a, b): (usize, usize), (x, y): (usize, usize)) -> bool {
    a <= x && y <= b
}

/// The only statement of a branch: itself, or the one in its block.
pub(super) fn only(branch: &Stmt) -> Option<&Stmt> {
    match &branch.kind {
        S::Block(b) if b.stmts.len() == 1 => Some(&b.stmts[0]),
        S::Block(_) => None,
        _ => Some(branch),
    }
}

/// Expressions that name a value a variable can hold and that do not depend on where they run.
pub(super) fn hoistable(e: &Expr) -> bool {
    match &e.kind {
        E::Ident(_)
        | E::Int(_)
        | E::Float(_)
        | E::Str(_)
        | E::Bool(_)
        | E::Null
        | E::Binary(..)
        | E::Call {
            ..
        }
        | E::Member(..)
        | E::Index(..)
        | E::Cast {
            ..
        }
        | E::Char(_)
        | E::Ifx {
            ..
        } => true,
        E::Unary(op, _) => !matches!(op, UnOp::Star),
        E::StructLit {
            ty, ..
        }
        | E::ArrayLit {
            ty, ..
        } => ty.is_some(),
        _ => false,
    }
}

/// A name that reads as a type (`Vector3`, `Hash_Table`): a variable cannot hold it.
pub(super) fn type_like(e: &Expr) -> bool {
    let name = match &e.kind {
        E::Ident(n) => n.as_str(),
        E::Member(_, n) => n.name.as_str(),
        _ => return false,
    };
    name.starts_with(|c: char| c.is_ascii_uppercase())
        && name.chars().any(|c| c.is_ascii_lowercase())
}

/// A name for a value: what the call or member says it is.
pub(super) fn suggest_name(e: &Expr) -> String {
    let name = match &e.kind {
        E::Call {
            callee, ..
        } => match &callee.kind {
            E::Ident(n) => n.as_str(),
            E::Member(_, n) => n.name.as_str(),
            _ => "",
        },
        E::Member(_, n) => n.name.as_str(),
        _ => "",
    };
    let name = name.strip_prefix("get_").unwrap_or(name);
    let valid = name.starts_with(|c: char| c.is_ascii_lowercase() || c == '_')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !crate::analysis::KEYWORDS.contains(&name);
    if valid && name.len() <= 24 {
        name.to_string()
    } else {
        "value".into()
    }
}

/// Does the expression call a procedure, so that running it twice (or later) matters?
pub(super) fn has_call(e: &Expr) -> bool {
    let mut found = false;
    walk_expr(e, &mut |x| {
        found |= matches!(x.kind, E::Call { .. } | E::Run { .. } | E::Insert(_));
    });
    found
}

fn walk_expr<'a>(e: &'a Expr, visit: &mut impl FnMut(&'a Expr)) {
    visit(e);
    for child in jailint::syntax::children(Node::Expr(e)) {
        if let Node::Expr(x) = child {
            walk_expr(x, visit);
        }
    }
}

/// Expressions that need no parentheses wherever they go.
pub(super) fn atomic(e: &Expr) -> bool {
    matches!(
        e.kind,
        E::Ident(_)
            | E::Int(_)
            | E::Float(_)
            | E::Str(_)
            | E::Bool(_)
            | E::Null
            | E::Call { .. }
            | E::Member(..)
            | E::Index(..)
            | E::Char(_)
            | E::StructLit { .. }
            | E::ArrayLit { .. }
    )
}

/// Does the use `x`, under `parents`, assign to, or take the address of, its variable?
pub(super) fn changes(parents: &[Node<'_>], x: &Expr) -> bool {
    let mut place = x.span;
    for node in parents.iter().rev() {
        match node {
            Node::Expr(up) => match &up.kind {
                E::Member(inner, _) | E::Index(inner, _) | E::Unary(UnOp::Deref, inner)
                    if inner.span == place =>
                {
                    place = up.span;
                }
                E::Unary(UnOp::Star, _) => return true,
                _ => return false,
            },
            Node::Stmt(s) => {
                return match &s.kind {
                    S::Assign {
                        lhs, ..
                    } => lhs.iter().any(|l| l.span == place),
                    S::For(f) => f.by_pointer,
                    _ => false,
                };
            }
        }
    }
    false
}
