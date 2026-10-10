//! `self_assignment`: `x = x;`, `p.pos = p.pos;`.
//!
//! Assigning a value to itself does nothing. It is almost always a slip: a local copied to a
//! field of the same name after `using info;` (where both sides now name the field), or one
//! side of a copy pasted wrong.
//!
//! Fires on a plain `=` with one target whose two sides are the same side-effect-free
//! expression (names, member paths, indexing; no calls), spelled the same. Macros and
//! polymorphic bodies are checked too: the statement does nothing in every instance. The
//! editor offers to remove the statement; `--fix` does not, since the slip may be the other
//! side.
//!
//! The two sides must also resolve to the same entities, name by name. Under `using info;`
//! a target `mode` is `info.mode` while a value `mode` may be a local declared before the
//! `using`; a name found only through `using` that is also declared earlier in the procedure
//! is ambiguous and not reported.
use super::{finding, same_pure};
use crate::syntax::{Cx, Node, walk, walk_node};
use crate::{Edit, Finding, Fix};
use jaic::ast::{AssignOp, Expr, ExprKind, StmtKind as S};
use jaic::intern::Sym;
use jaic::lexer::{P, Tok};
use jaic::sema::EntityId;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        walk(&p.body.stmts, &mut |n, _| {
            let Some(s) = n.stmt() else {
                return true;
            };
            if let S::Assign {
                op: AssignOp::Assign,
                lhs,
                rhs,
            } = &s.kind
                && let ([l], [r]) = (lhs.as_slice(), rhs.as_slice())
                && same_pure(cx, l, r)
                && same_place(cx, p.header.span.start, s.span.start, l, r)
            {
                let target = cx.whole_src(l.span).trim();
                let mut found = finding(
                    s.span,
                    format!("`{target}` is assigned to itself"),
                    Some("this does nothing; was another value or target meant? (after `using`, both sides name the same field)".into()),
                );
                // The statement's own `;` goes with it.
                let start = s.span.start as usize;
                let mut end = s.span.end as usize;
                let rest = cx.text.get(end..).unwrap_or("");
                let skipped = rest.len() - rest.trim_start().len();
                if rest[skipped..].starts_with(';') {
                    end += skipped + 1;
                }
                found.fix = Some(Fix {
                    title: "remove the statement".into(),
                    edits: vec![Edit {
                        start,
                        end,
                        text: String::new(),
                    }],
                    machine_applicable: false,
                });
                out.push(found);
            }
            true
        });
    }
}

/// Both sides denote the same place. Spelling is not enough: after `using info;` the target
/// `mode` can be the field `info.mode` while the value `mode` is a local declared outside the
/// `using` scope. Names are compared by what the compiler resolved them to; a name it did not
/// resolve to an entity (one found through `using`) that is also declared as a local or
/// parameter earlier in the procedure is ambiguous, so it is left alone.
fn same_place(cx: &Cx, from: u32, to: u32, l: &Expr, r: &Expr) -> bool {
    let (a, b) = (resolved(cx, l), resolved(cx, r));
    a == b
        && a.iter()
            .all(|(name, ents)| !ents.is_empty() || !declared_before(cx, *name, from, to))
}

fn resolved(cx: &Cx, e: &Expr) -> Vec<(Sym, Vec<EntityId>)> {
    let mut out = Vec::new();
    let mut stack = Vec::new();
    walk_node(Node::Expr(e), &mut stack, &mut |n, _| {
        if let Some(x) = n.expr()
            && let ExprKind::Ident(name) = x.kind
        {
            out.push((name, cx.entities(x.span).to_vec()));
        }
        true
    });
    out
}

/// `name :`, `name :=` or `name ::` is written in `from..to`.
fn declared_before(cx: &Cx, name: Sym, from: u32, to: u32) -> bool {
    cx.name_tokens(name, from, to).any(|i| {
        matches!(
            cx.tokens.get(i + 1).map(|t| &t.tok),
            Some(Tok::Punct(P::Colon | P::ColonEq | P::ColonColon))
        )
    })
}
