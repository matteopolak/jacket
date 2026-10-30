//! `using_pollution`: a `using` of a struct whose members have the same names as other things
//! already in scope.
//!
//! ```jai
//! draw :: (count: int, using style: Style) {     // `Style` has a `count` member too
//!     for 0..count { ... }                        // which one?
//! }
//! ```
//!
//! `using` brings every member of a struct into scope under its own name. When one of those
//! names is also a parameter, a local variable, a file-level variable or constant, or a member
//! another `using` already brought in, readers (and the next edit) have to know which one wins.
//! List only the members that are meant (`using,only(a, b) style`) or access them as
//! `style.count`.
//!
//! Checked for `using x;` statements, `using x: T;` declarations and `using` parameters, against
//! the parameters and the variables visible at that point, the non-procedure, non-type
//! declarations at the top of the file, and the members of earlier `using`s in the procedure.
//! The members are the struct's own and those of structs it `using`s in turn, through a
//! pointer too. `using,only(...)` and `using,except(...)` lists narrow them; the other filter
//! forms are skipped. A member named `_` or with no name is ignored, as are macros (`#expand`).
//! No fix. Off by default (`allow`): structs often share common member names like `x` or
//! `name` with variables around them and the program is fine.
use super::local_scopes::{self, Scopes};
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::{
    Decl, DeclKind, ExprKind as E, Ident, Stmt, StmtKind as S, UsingFilter, UsingStmt,
};
use jaic::intern::Sym;
use jaic::sema::scope::EntityKind;
use jaic::types::{TypeId, TypeKind};

/// How far to follow structs that `using` other structs.
const DEPTH: usize = 4;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "using_pollution",
    category: crate::config::Category::Pedantic,
    summary: "a `using` of a struct whose members share names with parameters, locals or globals already in scope",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    if cx.generated() {
        return;
    }
    let globals: Vec<Sym> = cx
        .stmts
        .iter()
        .filter_map(|s| match &s.kind {
            S::Decl(d) if holds_a_value(d) => Some(d),
            _ => None,
        })
        .flat_map(|d| d.names.iter().map(|n| n.name))
        .collect();
    for p in &cx.procs {
        if p.is_macro || p.header.flags.expand() {
            continue;
        }
        let mut scan = Scan {
            cx,
            out,
            globals: &globals,
            scopes: vec![Vec::new(), Vec::new()],
        };
        for q in p.header.params.iter().filter_map(|q| q.name.as_ref()) {
            scan.bound(q);
        }
        for q in p.header.params.iter().filter(|q| q.using) {
            let Some(name) = &q.name else {
                continue;
            };
            scan.pull_in(q.span, q.using_filter.as_ref(), name, q.span);
        }
        local_scopes::walk(&p.body.stmts, &mut scan);
    }
}

/// A top-level declaration of a variable or constant (not a procedure, struct or enum).
fn holds_a_value(d: &Decl) -> bool {
    match (&d.kind, &d.value) {
        (DeclKind::Var, _) => true,
        (DeclKind::Const, Some(v)) => !matches!(v.kind, E::Proc(_) | E::Struct(_) | E::Enum(_)),
        (DeclKind::Const, None) => false,
    }
}

struct Scan<'a, 'o> {
    cx: &'a Cx<'a>,
    out: &'o mut Vec<Finding>,
    globals: &'a [Sym],
    /// Names in scope at the point reached, innermost scope last.
    scopes: Vec<Vec<Sym>>,
}

impl Scan<'_, '_> {
    /// `using name` (a local or parameter whose declaration or statement is at `site`, its
    /// type recorded under `key`) brings its type's members into scope: report the ones that
    /// are already there.
    fn pull_in(
        &mut self,
        key: jaic::source::Span,
        filter: Option<&UsingFilter>,
        name: &Ident,
        site: jaic::source::Span,
    ) {
        let Some(ty) = local_type(self.cx, key) else {
            return;
        };
        self.bring_members(ty, filter, name.name, site);
    }

    fn bring_members(
        &mut self,
        ty: TypeId,
        filter: Option<&UsingFilter>,
        from: Sym,
        site: jaic::source::Span,
    ) {
        let mut names = Vec::new();
        members(self.cx, ty, DEPTH, &mut names);
        match filter {
            None | Some(UsingFilter::None) => {}
            Some(UsingFilter::Only(list)) => names.retain(|n| list.iter().any(|i| i.name == *n)),
            Some(UsingFilter::Except(list)) => names.retain(|n| !list.iter().any(|i| i.name == *n)),
            Some(_) => return,
        }
        names.dedup();
        let clashes: Vec<Sym> = names
            .iter()
            .copied()
            .filter(|n| {
                self.globals.contains(n) || self.scopes.iter().flatten().any(|visible| visible == n)
            })
            .collect();
        if !clashes.is_empty() {
            let shown: Vec<String> = clashes
                .iter()
                .take(4)
                .map(|n| format!("`{}`", n.as_str()))
                .collect();
            let more = clashes.len().saturating_sub(4);
            self.out.push(super::finding(
                site,
                format!(
                    "`using {}` brings in {}{}, which {} already in scope",
                    from.as_str(),
                    shown.join(", "),
                    if more > 0 {
                        format!(" and {more} more")
                    } else {
                        String::new()
                    },
                    if clashes.len() == 1 {
                        "is"
                    } else {
                        "are"
                    }
                ),
                Some(
                    "list the members that are meant (`using,only(...)`) or write `name.member`"
                        .into(),
                ),
            ));
        }
        if let Some(inner) = self.scopes.last_mut() {
            inner.extend(names);
        }
    }
}

impl Scopes for Scan<'_, '_> {
    fn enter(&mut self) {
        self.scopes.push(Vec::new());
    }

    fn leave(&mut self) {
        self.scopes.pop();
    }

    fn variable(&mut self, decl: &Decl, name: &Ident) {
        self.bound(name);
        if decl.using {
            self.pull_in(name.span, decl.using_filter(), name, name.span);
        }
    }

    fn bound(&mut self, name: &Ident) {
        if let Some(inner) = self.scopes.last_mut() {
            inner.push(name.name);
        }
    }

    fn using(&mut self, stmt: &Stmt, using: &UsingStmt) {
        let Some(ty) = self.cx.ty(&using.value).or_else(|| {
            // The statement's operand may carry no type of its own; the variable it names does.
            let named = self.cx.entities(using.value.span);
            let mut types = named
                .iter()
                .filter_map(|&e| match self.cx.compiler.entity(e).kind {
                    EntityKind::Local {
                        ty, ..
                    } => Some(ty),
                    _ => None,
                });
            let first = types.next()?;
            types.all(|t| t == first).then_some(first)
        }) else {
            return;
        };
        let from = match &using.value.kind {
            E::Ident(name) => *name,
            _ => Sym::intern("expression"),
        };
        self.bring_members(ty, Some(&using.filter), from, stmt.span);
    }
}

/// The type of the local or parameter declared at `key`, if every check agrees.
fn local_type(cx: &Cx, key: jaic::source::Span) -> Option<TypeId> {
    let mut found = None;
    let declared = cx.locals(key);
    if declared.is_empty() {
        return None;
    }
    for &e in &*declared {
        let EntityKind::Local {
            ty, ..
        } = cx.compiler.entity(e).kind
        else {
            return None;
        };
        match found {
            None => found = Some(ty),
            Some(other) if other == ty => {}
            Some(_) => return None,
        }
    }
    found
}

/// The member names of the struct `ty` (behind a pointer too), those of the structs it `using`s
/// included.
fn members(cx: &Cx, ty: TypeId, depth: usize, out: &mut Vec<Sym>) {
    let types = &cx.compiler.ty.types;
    let ty = match types.kind(ty) {
        TypeKind::Pointer(inner) => *inner,
        _ => ty,
    };
    let TypeKind::Struct(id) = types.kind(ty) else {
        return;
    };
    let fields = types.struct_info(*id).fields.clone();
    for f in fields.iter() {
        if let Some(name) = f.name
            && name.as_str() != "_"
        {
            out.push(name);
        }
        if f.using && depth > 0 {
            members(cx, f.ty, depth - 1, out);
        }
    }
}
