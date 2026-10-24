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
//! scoped to the loop on purpose; names starting with `_`; declarations in macros (`#expand`), which have their own
//! naming rules; and anything inside a nested procedure, which cannot see the outer locals.
//! A variable that starts from the one it hides (`count := count;`, `text := trim(text);`), the
//! usual way to get a changeable copy of a parameter, is not reported either, nor is a name
//! starting with `_`. No fix: which of the two to rename is the author's call. Off by default
//! (`allow`): reusing a short name in a sibling block is fine, and in a nested one it is common.
use super::local_scopes::{self, Scopes};
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::{Decl, Ident, Stmt, UsingStmt};
use jaic::intern::Sym;

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
        let mut walk = Shadows {
            cx,
            out,
            scopes: vec![params, Vec::new()],
        };
        local_scopes::walk(&p.body.stmts, &mut walk);
    }
}

struct Shadows<'a, 'o> {
    cx: &'a Cx<'a>,
    out: &'o mut Vec<Finding>,
    /// Names visible at the point reached, innermost scope last, each with the offset where it
    /// is declared.
    scopes: Vec<Vec<(Sym, u32)>>,
}

impl Scopes for Shadows<'_, '_> {
    fn enter(&mut self) {
        self.scopes.push(Vec::new());
    }

    fn leave(&mut self) {
        self.scopes.pop();
    }

    /// Reported if an enclosing scope has the name; visible from here on.
    fn variable(&mut self, decl: &Decl, name: &Ident) {
        let text = name.name.as_str();
        if text.starts_with('_') || derived_from_itself(self.cx, decl, name) {
            return;
        }
        let outer = self.scopes[..self.scopes.len() - 1]
            .iter()
            .rev()
            .flat_map(|scope| scope.iter().rev())
            .find(|(n, _)| *n == name.name);
        if let Some(&(_, at)) = outer {
            let (line, _) = self.cx.lines.line_col(self.cx.text, at as usize);
            self.out.push(super::finding(
                name.span,
                format!("`{text}` shadows the variable of that name declared on line {line}"),
                Some(
                    "rename one of them; an assignment here no longer reaches the outer one".into(),
                ),
            ));
        }
        self.bound(name);
    }

    /// Visible from here on, without being checked.
    fn bound(&mut self, name: &Ident) {
        if let Some(inner) = self.scopes.last_mut() {
            inner.push((name.name, name.span.start));
        }
    }

    fn using(&mut self, _: &Stmt, _: &UsingStmt) {
    }
}

/// `x := x;`, `x := x * 2;`: the new variable starts from the one it hides, the usual way to
/// get a changeable copy of a parameter.
fn derived_from_itself(cx: &Cx, decl: &Decl, name: &Ident) -> bool {
    decl.value.as_ref().is_some_and(|v| {
        cx.name_tokens(name.name, v.span.start, v.span.end)
            .next()
            .is_some()
    })
}
