//! `string_literal_write`: `s := "abc"; s[0] = #char "z";`.
//!
//! String literals are read-only, and equal literals share storage. Writing into one is a
//! runtime error in the interpreter and a crash in a native build; before literals were
//! read-only it silently changed every equal literal. Copy the string first.
//!
//! Fires on a write through a local that still holds a literal: `s[i] = x`, `s[i] += x`,
//! `s.data[i] = x`, `s.data.* = x`. (`"abc"[i] = x` directly is already a compile error.) The
//! local is declared from a literal (`s := "abc";`, `s: string = "abc";`) and written to by a later statement of the
//! same statement list, with nothing in between that could have given it another value: any
//! assignment to it, its address taken, or another declaration of the name, however deeply
//! nested, ends the check. Writes inside nested blocks and macros are not reported.
use super::{finding, statement_lists};
use crate::Finding;
use crate::syntax::{Cx, Node, root_ident, walk_node};
use jaic::ast::{Expr, ExprKind as E, Stmt, StmtKind as S, UnOp};
use jaic::intern::Sym;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if p.is_macro {
            continue;
        }
        statement_lists(&p.body.stmts, &mut |list: &[Stmt]| {
            // Locals in this list that still hold a literal.
            let mut literals: Vec<Sym> = Vec::new();
            for s in list {
                if let S::Assign {
                    lhs, ..
                } = &s.kind
                {
                    for l in lhs {
                        let shown = cx.whole_src(l.span).trim();
                        if let Some(name) = written_local(l).filter(|n| literals.contains(n)) {
                            out.push(finding(
                                l.span,
                                format!(
                                    "`{shown}` writes into a string literal: `{}` still holds one, and literals are read-only",
                                    name.as_str()
                                ),
                                Some(format!(
                                    "copy it first, for example `{0} = copy_string({0});`",
                                    name.as_str()
                                )),
                            ));
                        }
                    }
                }
                literals.retain(|name| !changes(s, *name));
                if let S::Decl(d) = &s.kind
                    && let [name] = d.names.as_slice()
                    && d.rare.is_none()
                    && !d.using
                    && !d.backtick
                    && matches!(d.value.as_ref().map(|v| &v.kind), Some(E::Str(_)))
                    && d.ty
                        .as_ref()
                        .is_none_or(|t| matches!(t.kind, E::Ident(n) if n.as_str() == "string"))
                {
                    literals.push(name.name);
                }
            }
        });
    }
}

/// The local a write target goes through: `s` of `s[i]`, `s.data[i]` and `s.data.*`.
fn written_local(e: &Expr) -> Option<Sym> {
    let base = match &e.kind {
        E::Index(base, _) => base,
        E::Unary(UnOp::Deref, base) => base,
        _ => return None,
    };
    match &base.kind {
        E::Ident(n) if matches!(e.kind, E::Index(..)) => Some(*n),
        E::Member(inner, field) if field.name.as_str() == "data" => match inner.kind {
            E::Ident(n) => Some(n),
            _ => None,
        },
        _ => None,
    }
}

/// `name` may hold another value after this statement: it is assigned to, has its address
/// taken, or is declared again (in any nested scope).
fn changes(s: &Stmt, name: Sym) -> bool {
    let mut found = false;
    let mut stack = Vec::new();
    walk_node(Node::Stmt(s), &mut stack, &mut |n, _| {
        match n {
            Node::Stmt(t) => match &t.kind {
                // Writing through it is what the rule reports, not a new value.
                S::Assign {
                    lhs, ..
                } => {
                    found |= lhs.iter().any(|l| {
                        written_local(l) != Some(name)
                            && root_ident(l).is_some_and(|(n, _)| n == name)
                    });
                }
                S::Decl(d) => found |= d.names.iter().any(|n| n.name == name),
                S::For(f) => {
                    found |= f.it.is_some_and(|i| i.name == name)
                        || f.index.is_some_and(|i| i.name == name);
                }
                _ => {}
            },
            Node::Expr(e) => {
                if let E::Unary(UnOp::Star, inner) = &e.kind
                    && root_ident(inner).is_some_and(|(n, _)| n == name)
                {
                    found = true;
                }
            }
        }
        !found
    });
    found
}
