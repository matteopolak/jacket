//! `shadowed_local`: a local variable declared with the name of another local or a parameter of
//! the same procedure that is still in scope.
//!
//! ```jai
//! scale :: (count: int) -> int {
//!     if count > 10 {
//!         count := 10;        // a new variable; the parameter is out of reach in here
//!         return count;
//!     }
//!     return count;
//! }
//! ```
//!
//! The same name means two variables a few lines apart, and an edit that assumed one meant the
//! other changes the wrong thing without a compile error: assigning to the inner `count` leaves
//! the parameter untouched. Rename one of them, or assign to the outer variable when that was
//! meant.
//!
//! Looks at variable declarations (`name := ...`, `name: T`) in nested blocks, loop bodies and
//! `case` bodies, against the parameters and the variables declared earlier in the blocks
//! around them. Not reported: loop variables, implicit (`it`, `it_index`) or named, as they are
//! scoped to the loop on purpose; `_`; declarations in macros (`#expand`), which have their own
//! naming rules; and anything inside a nested procedure, which cannot see the outer locals.
//! No fix: which of the two to rename is the author's call. Off by default (`allow`): reusing a
//! short name in a sibling block is fine, and in a nested one it is common.
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::{DeclKind, Ident, Stmt, StmtKind as S};
use jaic::intern::Sym;

/// Names visible at some point, innermost scope last; each with the offset where it is declared.
type Scopes = Vec<Vec<(Sym, u32)>>;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    if cx.generated() {
        return;
    }
    for p in &cx.procs {
        if p.is_macro || p.header.flags.expand() {
            continue;
        }
        let params = p
            .header
            .params
            .iter()
            .filter_map(|q| q.name.as_ref())
            .map(|n| (n.name, n.span.start))
            .collect();
        let mut scopes: Scopes = vec![params, Vec::new()];
        block(cx, &p.body.stmts, &mut scopes, out);
    }
}

fn block(cx: &Cx, stmts: &[Stmt], scopes: &mut Scopes, out: &mut Vec<Finding>) {
    for s in stmts {
        stmt(cx, s, scopes, out);
    }
}

/// `body` in a scope of its own.
fn scoped(cx: &Cx, body: &Stmt, scopes: &mut Scopes, out: &mut Vec<Finding>) {
    scopes.push(Vec::new());
    match &body.kind {
        S::Block(b) => block(cx, &b.stmts, scopes, out),
        _ => stmt(cx, body, scopes, out),
    }
    scopes.pop();
}

fn stmt(cx: &Cx, s: &Stmt, scopes: &mut Scopes, out: &mut Vec<Finding>) {
    match &s.kind {
        S::Decl(d) if d.kind == DeclKind::Var && !d.backtick && !d.using => {
            for name in &d.names {
                declare(cx, name, scopes, out);
            }
        }
        S::Block(b) => {
            scopes.push(Vec::new());
            block(cx, &b.stmts, scopes, out);
            scopes.pop();
        }
        S::If {
            then_branch,
            else_branch,
            ..
        } => {
            scoped(cx, then_branch, scopes, out);
            if let Some(e) = else_branch {
                scoped(cx, e, scopes, out);
            }
        }
        S::Switch(sw) | S::StaticSwitch(sw) => {
            for case in &sw.cases {
                scopes.push(Vec::new());
                block(cx, &case.body, scopes, out);
                scopes.pop();
            }
        }
        S::While(w) => {
            scopes.push(Vec::new());
            if w.bind_label
                && let Some(label) = &w.label
            {
                bring_in(label, scopes);
            }
            scoped(cx, &w.body, scopes, out);
            scopes.pop();
        }
        S::For(f) => {
            scopes.push(Vec::new());
            for name in [&f.it, &f.index].into_iter().flatten() {
                bring_in(name, scopes);
            }
            scoped(cx, &f.body, scopes, out);
            scopes.pop();
        }
        S::Defer {
            body, ..
        } => scoped(cx, body, scopes, out),
        S::PushContext(p) => scoped(cx, &p.body, scopes, out),
        S::StaticIf(si) => {
            for branch in [&si.then_branch, &si.else_branch] {
                scopes.push(Vec::new());
                block(cx, branch, scopes, out);
                scopes.pop();
            }
        }
        _ => {}
    }
}

/// A name that is visible from here on, without being checked (a loop variable).
fn bring_in(name: &Ident, scopes: &mut Scopes) {
    if let Some(inner) = scopes.last_mut() {
        inner.push((name.name, name.span.start));
    }
}

/// A variable declared here: reported if an enclosing scope has the name, then visible.
fn declare(cx: &Cx, name: &Ident, scopes: &mut Scopes, out: &mut Vec<Finding>) {
    let text = name.name.as_str();
    if text == "_" {
        return;
    }
    let outer = scopes[..scopes.len() - 1]
        .iter()
        .rev()
        .flat_map(|scope| scope.iter().rev())
        .find(|(n, _)| *n == name.name);
    if let Some(&(_, at)) = outer {
        let (line, _) = cx.lines.line_col(cx.text, at as usize);
        out.push(super::finding(
            name.span,
            format!("`{text}` shadows the variable of that name declared on line {line}"),
            Some("rename one of them; an assignment here no longer reaches the outer one".into()),
        ));
    }
    bring_in(name, scopes);
}
