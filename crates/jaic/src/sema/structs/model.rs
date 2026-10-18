//! The data a struct declaration is read into: sources, member aliases, field items and `using` filters.
use crate::ast::ExprKind as E;
use crate::sema::value::PolyStructId;
use crate::sema::*;
use crate::types::{Field, StructId};

/// Where a struct's fields come from.
#[derive(Clone)]
pub struct StructSource {
    pub lit: Rc<ast::StructLit>,
    /// Body scope (constants, `#this`, polymorphic bindings).
    pub scope: ScopeId,
    /// Field declarations evaluated in their own scopes (the synthesized `#Context`).
    pub extra: Vec<(Rc<ast::Decl>, ScopeId)>,
    /// Initializer of each laid-out field, with the scope to evaluate it in.
    pub inits: Vec<(Option<ast::Expr>, ScopeId)>,
}

pub struct PolyStruct {
    pub name: Sym,
    pub lit: Rc<ast::StructLit>,
    pub scope: ScopeId,
    pub instances: HashMap<Vec<InstanceKeyValue>, TypeId>,
    /// Parameters fixed by `#bake_arguments`: constants of every instance.
    pub baked: Vec<(Sym, Value, TypeId)>,
    /// For `#bake_arguments S(...)`: the struct baked from. Instances are
    /// instances of the origin (`Mat4(f32)` is `Matrix(f32, 4, 4)`).
    pub origin: Option<PolyStructId>,
}

/// Names of a struct that stand for members reached through a path of its fields
/// (`using,only(width, height) texture.desc;`).
#[derive(Clone, Debug)]
pub struct MemberAlias {
    /// Field names from the struct to the member whose members are reused.
    pub path: Vec<Sym>,
    pub filter: ast::UsingFilter,
}

impl MemberAlias {
    /// The member of the target that `name` stands for, if the filter lets it through.
    pub(super) fn target_name(&self, name: Sym) -> Option<Sym> {
        match &self.filter {
            ast::UsingFilter::None => Some(name),
            ast::UsingFilter::Only(names) => names.iter().any(|n| n.name == name).then_some(name),
            ast::UsingFilter::Except(names) => {
                (!names.iter().any(|n| n.name == name)).then_some(name)
            }
            ast::UsingFilter::Map(pairs) => pairs
                .iter()
                .find(|(new, _)| new.name == name)
                .map(|(_, old)| old.name),
            ast::UsingFilter::Computed {
                ..
            } => None,
        }
    }
}

/// The names of a member path expression (`a.b.c`), or `None`.
pub(super) fn member_path(e: &ast::Expr) -> Option<Vec<Sym>> {
    match &e.kind {
        E::Ident(name) => Some(vec![*name]),
        E::Member(base, field) => {
            let mut path = member_path(base)?;
            path.push(field.name);
            Some(path)
        }
        _ => None,
    }
}

/// One step of a member path through `using` fields.
#[derive(Clone, Copy, Debug)]
pub enum PathStep {
    Offset(u64),
    /// Load a pointer stored at the current address (`using p: *T`).
    Deref,
}

pub(super) type MemberResolution = (Vec<PathStep>, TypeId, Option<(StructId, usize)>);

pub(crate) struct FieldDecl {
    pub(crate) name: Option<Sym>,
    pub(crate) ty: TypeId,
    pub(crate) using: bool,
    pub(super) as_: bool,
    pub(super) notes: Vec<Rc<str>>,
    pub(crate) span: Span,
    pub(super) init: Option<ast::Expr>,
    pub(crate) scope: ScopeId,
    pub(super) align: Option<u64>,
    /// `using,only(...)` / `using,except(...)` on a `using` member.
    pub(crate) filter: Option<ast::UsingFilter>,
}

pub(crate) enum FieldItem {
    Declaration(ScopeId, Rc<ast::Decl>),
    Field(FieldDecl),
    /// `using,only(a, b) field.inner;`: names reached through a path of members.
    Alias(MemberAlias),
    /// `#place f;`: following fields start at `f`'s offset.
    Place(Sym, Span),
    /// `#overlay(f)`: the next field shares `f`'s storage.
    Overlay(Sym, Span),
}

/// An offset depends on at most one other offset: align(dependency + addend).
/// Field nodes keep declaration indices; cursor nodes express sequential storage separately.
#[derive(Clone, Copy)]
pub(super) struct OffsetNode {
    pub(super) dependency: Option<usize>,
    pub(super) addend: u64,
    pub(super) align: u64,
    pub(crate) span: Span,
}


pub(super) enum LayoutItem {
    Field(usize),
    Place(Sym, Span),
    Overlay(Sym, Span),
}

#[derive(Clone)]
pub(in crate::sema) enum FieldUsing {
    Declaration(ScopeId, Rc<ast::Decl>),
    Inline(TypeId, ast::UsingFilter),
}

/// The `using,only(...)`/`using,except(...)` filters of a struct's fields, by field index.
#[derive(Default)]
pub(in crate::sema) struct UsingFilters(Vec<(usize, ast::UsingFilter)>);

impl UsingFilters {
    /// Record the filter of field `index`; fields are visited in order.
    pub(super) fn push(&mut self, index: usize, filter: ast::UsingFilter) {
        debug_assert!(self.0.last().is_none_or(|(last, _)| *last < index));
        self.0.push((index, filter));
    }

    pub(super) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn get(&self, index: usize) -> Option<&ast::UsingFilter> {
        let at = self.0.binary_search_by_key(&index, |(i, _)| *i).ok()?;
        Some(&self.0[at].1)
    }

    /// The name to look up in the `using` field `index` for the promoted `name`: `name` itself
    /// when the field has no filter, the name after the filter's renames, or `None` when the
    /// filter hides it.
    pub(super) fn target(&self, index: usize, name: Sym) -> Option<Sym> {
        match self.get(index) {
            Some(filter) => crate::sema::using::filter_target(filter, name),
            None => Some(name),
        }
    }
}

/// Candidate fields for a promoted name, applying each embedding's filter.
pub(super) fn using_candidates(
    fields: &[Field],
    filters: &UsingFilters,
    name: Sym,
) -> Vec<(TypeId, u64, Sym)> {
    fields
        .iter()
        .enumerate()
        .filter_map(|(index, field)| {
            if !field.using {
                return None;
            }
            let target = filters.target(index, name)?;
            Some((field.ty, field.offset, target))
        })
        .collect()
}
