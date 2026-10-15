//! Structs, enums and polymorphic structs: declaration, layout, member
//! access, default initialization and aggregate literals.
use super::lower::{FnCtx, Operand};
use super::scope::{EntityKind, Found, Resolved, ScopeKind, UsingEntry};
use super::value::{Aggregate, PolyStructId};
use super::*;
use crate::ast::ExprKind as E;
use crate::ir::Ty;
use crate::types::{ArrayKind, Field, LayoutState, MAX_SIZE, StructId, StructInfo, TypeKind};

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
    fn target_name(&self, name: Sym) -> Option<Sym> {
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
fn member_path(e: &ast::Expr) -> Option<Vec<Sym>> {
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

struct FieldDecl {
    name: Option<Sym>,
    ty: TypeId,
    using: bool,
    as_: bool,
    notes: Vec<Rc<str>>,
    span: Span,
    init: Option<ast::Expr>,
    scope: ScopeId,
    align: Option<u64>,
    /// `using,only(...)` / `using,except(...)` on a `using` member.
    filter: Option<ast::UsingFilter>,
}

enum FieldItem {
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
struct OffsetNode {
    dependency: Option<usize>,
    addend: u64,
    align: u64,
    span: Span,
}

fn solve_offsets(nodes: &[OffsetNode]) -> Result<Vec<u64>> {
    let mut state = vec![0u8; nodes.len()];
    let mut values = vec![0; nodes.len()];
    let mut stack = Vec::new();
    for root in 0..nodes.len() {
        if state[root] == 2 {
            continue;
        }
        stack.push(root);
        while let Some(&at) = stack.last() {
            state[at] = 1;
            let node = nodes[at];
            let base = if let Some(dep) = node.dependency {
                match state[dep] {
                    0 => {
                        stack.push(dep);
                        continue;
                    }
                    1 => {
                        return Err(Box::new(
                            Diagnostic::error(node.span, "cycle in struct field layout").with_note(
                                nodes[dep].span,
                                "this storage offset depends on itself",
                            ),
                        ));
                    }
                    _ => values[dep],
                }
            } else {
                0u64
            };
            values[at] = base
                .checked_add(node.addend)
                .and_then(|n| n.checked_next_multiple_of(node.align))
                .filter(|&n| n <= MAX_SIZE)
                .ok_or_else(|| {
                    Box::new(Diagnostic::error(
                        node.span,
                        format!("struct is too large (the limit is {MAX_SIZE} bytes)"),
                    ))
                })?;
            state[at] = 2;
            stack.pop();
        }
    }
    Ok(values)
}

enum LayoutItem {
    Field(usize),
    Place(Sym, Span),
    Overlay(Sym, Span),
}

#[derive(Clone)]
pub(super) enum FieldUsing {
    Declaration(ScopeId, Rc<ast::Decl>),
    Inline(TypeId, ast::UsingFilter),
}

/// Candidate fields for a promoted name, applying each embedding's filter.
fn using_candidates(
    fields: &[Field],
    filters: &[(usize, ast::UsingFilter)],
    name: Sym,
) -> Vec<(TypeId, u64, Sym)> {
    fields
        .iter()
        .enumerate()
        .filter_map(|(index, field)| {
            if !field.using {
                return None;
            }
            let target = match filters.iter().find(|(i, _)| *i == index) {
                Some((_, filter)) => super::using::filter_target(filter, name)?,
                None => name,
            };
            Some((field.ty, field.offset, target))
        })
        .collect()
}

impl Compiler {
    pub fn new_struct_type(
        &mut self,
        name: Sym,
        lit: Rc<ast::StructLit>,
        parent: ScopeId,
        bindings: Vec<(Sym, Value, TypeId)>,
        poly: Option<(PolyStructId, Vec<Value>)>,
    ) -> TypeId {
        let ty = self.types.new_struct(StructInfo {
            name,
            ast: Some(lit.id),
            // A tagged union is a struct: the tag, then the members' union.
            is_union: lit.kind == ast::StructKind::Union && lit.tag.is_none(),
            fields: Vec::new(),
            size: 0,
            align: 1,
            layout: LayoutState::Pending,
            poly_parent: None,
            poly_args: poly.as_ref().map(|(_, v)| v.clone()).unwrap_or_default(),
            span: lit.span,
        });
        let module = self.scope(parent).module;
        let scope = self.new_scope(ScopeKind::Struct(ty), Some(parent), module, None);
        for (n, v, t) in bindings {
            self.add_const(scope, n, lit.span, v, t);
        }
        let file_scope = scope;
        for stmt in &lit.body {
            match &stmt.kind {
                ast::StmtKind::Decl(decl) if decl.kind == ast::DeclKind::Var => {
                    for n in &decl.names {
                        self.field_declarations
                            .entry(scope)
                            .or_default()
                            .insert(n.name, decl.clone());
                    }
                }
                ast::StmtKind::Decl(decl) if decl.kind == ast::DeclKind::Const => {
                    for (index, n) in decl.names.iter().enumerate() {
                        self.add_entity(
                            scope,
                            n.name,
                            n.span,
                            EntityKind::Decl {
                                decl: decl.clone(),
                                index,
                            },
                            true,
                        );
                    }
                }
                ast::StmtKind::StaticIf(_) => {
                    self.push_pending(
                        scope,
                        scope::Pending {
                            stmt: stmt.clone(),
                            exported: true,
                            file_scope,
                            state: scope::PendingState::Waiting,
                        },
                    );
                }
                _ => {}
            }
        }
        let s = self.types.as_struct(ty).unwrap();
        self.struct_asts.insert(
            s,
            StructSource {
                lit,
                scope,
                extra: Vec::new(),
                inits: Vec::new(),
            },
        );
        ty
    }

    /// The `Type` value standing for a polymorphic struct that has no arguments yet.
    pub fn poly_struct_type(&mut self, ps: PolyStructId) -> TypeId {
        let name = self.poly_structs[ps.0 as usize].name;
        self.types.intern(TypeKind::PolyStruct {
            id: ps,
            name,
        })
    }

    pub fn new_poly_struct(
        &mut self,
        name: Sym,
        lit: Rc<ast::StructLit>,
        scope: ScopeId,
    ) -> PolyStructId {
        self.poly_structs.push(PolyStruct {
            name,
            lit,
            scope,
            instances: HashMap::default(),
            baked: Vec::new(),
            origin: None,
        });
        PolyStructId(self.poly_structs.len() as u32 - 1)
    }

    /// A bare generic name denotes its default instance when every parameter has a default.
    /// Keep the source type available separately for generic patterns and reflection.
    pub(super) fn poly_struct_has_defaults(&self, ps: PolyStructId) -> bool {
        self.poly_structs[ps.0 as usize]
            .lit
            .params
            .iter()
            .all(|p| p.default.is_some())
    }

    /// `Table(int, string)`: instantiate a polymorphic struct.
    /// The constant value of a polymorphic struct argument. An aggregate whose type
    /// differs from the parameter's (a fixed array for a `[] T` parameter) is converted.
    pub(super) fn struct_arg_value(
        &mut self,
        scope: ScopeId,
        op: Operand,
        ty: TypeId,
        span: Span,
    ) -> Result<Value> {
        if ty == TypeId::TYPE {
            return self.operand_as_type(op, span).map(Value::Type);
        }
        let converts = ty != TypeId::VOID
            && op.ty() != ty
            && matches!(
                op,
                Operand::Const {
                    value: Value::Bytes(_),
                    ..
                }
            );
        if converts {
            return self.const_value_of_operand(scope, op, ty, span);
        }
        match op {
            Operand::Type(t) => Ok(Value::Type(t)),
            Operand::Const {
                value, ..
            } => Ok(value),
            Operand::Procs(p) if p.len() == 1 => Ok(Value::Proc(p[0])),
            other => err(
                span,
                format!("expected a constant, found {}", self.describe(&other)),
            ),
        }
    }

    /// The declared type of a polymorphic struct's value parameter (by position or name), when
    /// it can be evaluated outside the parameter scope; used to type `.MEMBER` arguments.
    pub fn poly_struct_param_type(
        &mut self,
        ps: PolyStructId,
        index: usize,
        name: Option<Sym>,
    ) -> Option<TypeId> {
        let (lit, scope) = {
            let p = &self.poly_structs[ps.0 as usize];
            (p.lit.clone(), p.scope)
        };
        let param = match name {
            Some(n) => lit
                .params
                .iter()
                .find(|p| p.name.map(|i| i.name) == Some(n))?,
            None => lit.params.get(index)?,
        };
        let ty = param.ty.as_ref()?;
        self.eval_type(scope, ty).ok()
    }

    pub fn instantiate_struct(
        &mut self,
        ps: PolyStructId,
        args: Vec<(Option<Sym>, Operand)>,
        span: Span,
    ) -> Result<TypeId> {
        let (name, lit, def_scope) = {
            let p = &self.poly_structs[ps.0 as usize];
            (p.name, p.lit.clone(), p.scope)
        };
        if let Some(origin) = self.poly_structs[ps.0 as usize].origin {
            // Name the arguments for the origin and add the baked ones.
            let mut named = Vec::new();
            for (i, (n, v)) in args.into_iter().enumerate() {
                let n = n.or_else(|| {
                    lit.params
                        .get(i)
                        .filter(|p| !p.variadic)
                        .and_then(|p| p.name)
                        .map(|p| p.name)
                });
                named.push((n, v));
            }
            for (n, value, ty) in self.poly_structs[ps.0 as usize].baked.clone() {
                named.push((Some(n), const_operand(value, ty)));
            }
            return self.instantiate_struct(origin, named, span);
        }
        let module = self.scope(def_scope).module;
        let param_scope = self.new_scope(ScopeKind::StructParams, Some(def_scope), module, None);
        let mut values: Vec<Option<Operand>> = vec![None; lit.params.len()];
        // `struct(types: ..Type)`: the positional arguments from the variadic parameter on
        // become one constant `[] Type`.
        let variadic = lit.params.iter().position(|p| p.variadic);
        let mut pack: Vec<Operand> = Vec::new();
        let mut next = 0;
        for (n, v) in args {
            if n.is_none() && variadic.is_some_and(|vi| next >= vi) {
                pack.push(v);
                continue;
            }
            let index = match n {
                Some(n) => lit
                    .params
                    .iter()
                    .position(|p| p.name.map(|i| i.name) == Some(n))
                    .ok_or_else(|| {
                        Box::new(Diagnostic::error(
                            span,
                            format!("`{name}` has no parameter `{n}`"),
                        ))
                    })?,
                None => {
                    let i = next;
                    next += 1;
                    i
                }
            };
            if index >= values.len() {
                return err(span, format!("too many arguments for `{name}`"));
            }
            values[index] = Some(v);
        }
        let mut bindings = self.poly_structs[ps.0 as usize].baked.clone();
        let mut key = Vec::new();
        for (i, p) in lit.params.iter().enumerate() {
            let pname = p.name.map(|n| n.name).unwrap_or_else(|| Sym::intern("_"));
            // `x: $T`: the argument's type binds `T` (also `[$N] $T` and other patterns).
            if let Some(t) = &p.ty
                && super::procs::has_poly(t)
                && let Some(op) = &values[i]
            {
                let arg_ty = match op {
                    Operand::Const {
                        ty,
                        value,
                        untyped: true,
                    } => self.default_untyped(*ty, value),
                    Operand::Procs(procs) if procs.len() == 1 => self.proc_type(procs[0], span)?,
                    Operand::Type(_) => TypeId::TYPE,
                    other => other.ty(),
                };
                let mut found = Vec::new();
                let represented = match op.const_value() {
                    Some(Value::Type(t)) => Some(t),
                    _ => None,
                };
                self.match_pattern_value(t, arg_ty, &mut found, param_scope, represented)?;
                for (n, v, vty) in found {
                    self.add_const(param_scope, n, p.span, v.clone(), vty);
                    key.push(v.clone());
                    bindings.push((n, v, vty));
                }
            }
            let ty = match &p.ty {
                Some(t) => self.eval_type(param_scope, t)?,
                None => TypeId::VOID,
            };
            if Some(i) == variadic {
                let elem = if ty == TypeId::VOID {
                    TypeId::TYPE
                } else {
                    ty
                };
                let mut items = std::mem::take(&mut pack);
                if let Some(op) = values[i].take() {
                    items.insert(0, op);
                }
                let count = items.len() as u64;
                let esize = self.size_of(elem, p.span)?;
                let mut array =
                    value::Aggregate::zeroed(esize.saturating_mul(items.len() as u64), span)?;
                for (k, op) in items.into_iter().enumerate() {
                    let v = self.struct_arg_value(param_scope, op, elem, p.span)?;
                    let v = match (v, self.types.is_float(elem)) {
                        (Value::Int(i), true) => Value::float(i as f64),
                        (v, _) => v,
                    };
                    self.write_value(&mut array, k as u64 * esize, &v, elem, p.span)?;
                }
                let value = self.const_pack(elem, array, count, p.span)?;
                let ty = self.types.array(elem, ArrayKind::View);
                self.add_const(param_scope, pname, p.span, value.clone(), ty);
                key.push(super::instance_key_value(&self.types, &value, ty).value);
                bindings.push((pname, value, ty));
                continue;
            }
            let argument = match values[i].take() {
                Some(op) => op,
                None => match &p.default {
                    Some(d) => {
                        // The declared type lets `.FIRST` defaults find their enum.
                        let expected = (ty != TypeId::VOID).then_some(ty);
                        self.eval_const_or_run(param_scope, d, expected)?
                    }
                    None => {
                        return err(
                            span,
                            format!("missing argument for parameter `{pname}` of `{name}`"),
                        );
                    }
                },
            };
            let argument_ty = match &argument {
                Operand::Const {
                    ty,
                    value,
                    untyped: true,
                } => self.default_untyped(*ty, value),
                other => other.ty(),
            };
            let value = self.struct_arg_value(param_scope, argument, ty, p.span)?;
            let value = match (value, self.types.is_float(ty)) {
                (Value::Int(i), true) => Value::float(i as f64),
                (v, _) => v,
            };
            let ty = if ty == TypeId::VOID && argument_ty != TypeId::COMPILE_TIME {
                argument_ty
            } else if ty == TypeId::VOID {
                self.type_of_value(&value)
            } else {
                ty
            };
            self.add_const(param_scope, pname, p.span, value.clone(), ty);
            key.push(value.clone());
            bindings.push((pname, value, ty));
        }
        if let Some(block) = &lit.modify {
            let baked = self.poly_structs[ps.0 as usize].baked.len();
            let params = bindings.split_off(baked);
            let modified = self.run_struct_modify(def_scope, name, block, params, span)?;
            key = modified
                .iter()
                .map(|(_, v, t)| super::instance_key_value(&self.types, v, *t).value)
                .collect();
            bindings.extend(modified);
        }
        let cache_key = bindings
            .iter()
            .map(|(_, value, ty)| super::instance_key_value(&self.types, value, *ty))
            .collect::<Vec<_>>();
        if let Some(&t) = self.poly_structs[ps.0 as usize].instances.get(&cache_key) {
            return Ok(t);
        }
        if self.poly_structs[ps.0 as usize].instances.len() >= super::MAX_INSTANCES {
            return err(span, super::too_many_instances(name));
        }
        let t = self.new_struct_type(name, lit, def_scope, bindings, Some((ps, key.clone())));
        self.poly_structs[ps.0 as usize]
            .instances
            .insert(cache_key, t);
        Ok(t)
    }

    // -----------------------------------------------------------------------
    // Layout
    // -----------------------------------------------------------------------

    pub fn layout_struct(&mut self, s: StructId, span: Span) -> Result<()> {
        match self.types.struct_info(s).layout {
            LayoutState::Done => return Ok(()),
            LayoutState::InProgress => {
                self.in_progress_misses += 1;
                let name = self.types.struct_info(s).name;
                return err(
                    span,
                    format!("struct `{name}` contains itself (use a pointer)"),
                );
            }
            LayoutState::Pending => {}
        }
        self.types.struct_info_mut(s).layout = LayoutState::InProgress;
        let result = self.layout_struct_inner(s);
        if result.is_err() {
            self.types.struct_info_mut(s).layout = LayoutState::Pending;
        }
        result
    }

    fn layout_struct_inner(&mut self, s: StructId) -> Result<()> {
        let src = self
            .struct_asts
            .get(&s)
            .cloned()
            .expect("struct without source");
        let mut items = Vec::new();
        self.field_types.remove(&src.scope);
        self.field_using_order.remove(&src.scope);
        if let Some(ast::UnionTag {
            name: tag,
            ty,
            value,
        }) = &src.lit.tag
        {
            // `union tag: T { ... }`: the tag field, then an anonymous union of the members.
            let tag_ty = match (ty, value) {
                (Some(ty), _) => self.eval_type(src.scope, ty)?,
                (None, Some(value)) => self.eval_const(src.scope, value, None)?.ty(),
                (None, None) => return err(tag.span, "union tag needs a type"),
            };
            // `.A ,, a: int;`: each member's tag value names a member of the tag's enum.
            if self.ide.is_some() {
                for stmt in &src.lit.body {
                    if let ast::StmtKind::Decl(d) = &stmt.kind
                        && let Some(t) = &d.union_tag()
                        && let ast::ExprKind::InferredMember(m) = &t.kind
                    {
                        self.ide_note_member_use(m.span, tag_ty, m.name);
                        self.ide_note_member_ref(m.span, m.name, tag_ty);
                    }
                }
            }
            let mut members = (*src.lit).clone();
            members.id = ast::AstId::fresh();
            members.tag = None;
            members.params = Vec::new();
            members.notes = Vec::new();
            let members = self.new_struct_type(
                Sym::intern("anonymous"),
                Rc::new(members),
                src.scope,
                Vec::new(),
                None,
            );
            let tag_init = value.as_deref().cloned();
            for (name, ty, using, init) in [
                (Some(tag.name), tag_ty, false, tag_init),
                (None, members, true, None),
            ] {
                items.push(FieldItem::Field(FieldDecl {
                    name,
                    ty,
                    using,
                    as_: false,
                    notes: Vec::new(),
                    span: tag.span,
                    init,
                    scope: src.scope,
                    align: None,
                    filter: None,
                }));
            }
        } else {
            self.collect_fields(src.scope, &src.lit.body, &mut items)?;
        }
        for (decl, scope) in &src.extra {
            self.collect_decl_fields(*scope, decl, &mut items)?;
        }
        let mut using_order = Vec::new();
        for item in &mut items {
            match item {
                FieldItem::Declaration(scope, decl) if decl.using => {
                    using_order.push(FieldUsing::Declaration(*scope, decl.clone()))
                }
                FieldItem::Field(d) if d.using => {
                    let filter = match d.filter.as_ref() {
                        Some(filter) => match self.filter_using_entry(
                            d.scope,
                            scope::UsingEntry::Type(d.ty),
                            filter,
                            d.span,
                        )? {
                            scope::UsingEntry::Filtered(_, filter) => (*filter).clone(),
                            _ => filter.clone(),
                        },
                        None => ast::UsingFilter::None,
                    };
                    d.filter = Some(filter.clone());
                    using_order.push(FieldUsing::Inline(d.ty, filter));
                }
                _ => {}
            }
        }
        self.field_using_order.insert(src.scope, using_order);
        let mut typed_items = Vec::with_capacity(items.len());
        for item in items {
            match item {
                FieldItem::Declaration(scope, decl) => {
                    self.resolve_decl_fields(scope, &decl, &mut typed_items)?
                }
                other => typed_items.push(other),
            }
        }
        let items = typed_items;
        let is_union = src.lit.kind == ast::StructKind::Union && src.lit.tag.is_none();
        let no_padding = src.lit.flags.no_padding;
        let mut fields = Vec::new();
        let mut inits = Vec::new();
        let mut end = 0u64;
        let mut align = 1u64;
        let mut aliases = Vec::new();
        let mut filters = Vec::new();
        let mut sizes = Vec::new();
        let mut layout = Vec::new();
        let mut names = HashMap::default();
        for item in items {
            match item {
                FieldItem::Declaration(..) => unreachable!("field declarations were resolved"),
                FieldItem::Alias(alias) => aliases.push(alias),
                FieldItem::Place(name, span) => layout.push(LayoutItem::Place(name, span)),
                FieldItem::Overlay(name, span) => layout.push(LayoutItem::Overlay(name, span)),
                FieldItem::Field(d) => {
                    self.ensure_storage_type(d.ty, d.span)?;
                    let size = self.size_of(d.ty, d.span)?;
                    let a = match d.align {
                        Some(n) if n > 0 => n,
                        _ => self.align_of(d.ty, d.span)?,
                    };
                    align = align.max(a);
                    let index = fields.len();
                    if let Some(name) = d.name
                        && names.insert(name, index).is_some()
                    {
                        return err(
                            d.span,
                            format!("member `{name}` is declared more than once"),
                        );
                    }
                    if d.using
                        && let Some(filter) = d.filter.clone()
                    {
                        let resolved = self.filter_using_entry(
                            d.scope,
                            scope::UsingEntry::Type(d.ty),
                            &filter,
                            d.span,
                        )?;
                        let filter = match resolved {
                            scope::UsingEntry::Filtered(_, filter) => (*filter).clone(),
                            _ => filter,
                        };
                        filters.push((index, filter));
                    }
                    fields.push(Field {
                        name: d.name,
                        ty: d.ty,
                        offset: 0,
                        align: a,
                        using: d.using,
                        as_: d.as_,
                        overlay: false,
                        notes: d.notes,
                        span: d.span,
                    });
                    sizes.push(size);
                    inits.push((d.init, d.scope));
                    layout.push(LayoutItem::Field(index));
                }
            }
        }
        // Node zero is the initial cursor, nodes 1..=fields.len() are field offsets.
        let zero = OffsetNode {
            dependency: None,
            addend: 0,
            align: 1,
            span: src.lit.span,
        };
        let mut nodes = vec![zero; fields.len() + 1];
        let mut cursor = 0;
        let mut overlay = None;
        let mut targets = HashMap::default();
        for item in layout {
            match item {
                LayoutItem::Field(index) => {
                    let field = &mut fields[index];
                    let overlaid = overlay.take();
                    field.overlay = overlaid.is_some();
                    nodes[index + 1] = OffsetNode {
                        dependency: Some(overlaid.unwrap_or(if is_union {
                            0
                        } else {
                            cursor
                        })),
                        addend: 0,
                        align: if field.overlay || no_padding || is_union {
                            1
                        } else {
                            field.align
                        },
                        span: field.span,
                    };
                    if !field.overlay && !is_union {
                        cursor = nodes.len();
                        nodes.push(OffsetNode {
                            dependency: Some(index + 1),
                            addend: sizes[index],
                            align: 1,
                            span: field.span,
                        });
                    }
                }
                LayoutItem::Place(name, span) | LayoutItem::Overlay(name, span) => {
                    let (index, relative) = if let Some(&target) = targets.get(&name) {
                        target
                    } else {
                        let target =
                            self.layout_anchor(s, &fields, &filters, &names, name, span)?;
                        targets.insert(name, target);
                        target
                    };
                    let node = if relative == 0 {
                        index + 1
                    } else {
                        let node = nodes.len();
                        nodes.push(OffsetNode {
                            dependency: Some(index + 1),
                            addend: relative,
                            align: 1,
                            span,
                        });
                        node
                    };
                    if let Some(&index) = names.get(&name) {
                        self.ide_note_named_field(span, name, fields[index].span, fields[index].ty);
                    }
                    if matches!(item, LayoutItem::Place(..)) {
                        cursor = node;
                    } else {
                        overlay = Some(node);
                    }
                }
            }
        }
        if overlay.is_some() {
            return err(src.lit.span, "#overlay needs a following stored member");
        }
        let offsets = solve_offsets(&nodes)?;
        for (index, field) in fields.iter_mut().enumerate() {
            field.offset = offsets[index + 1];
            let extent = field
                .offset
                .checked_add(sizes[index])
                .filter(|&n| n <= MAX_SIZE)
                .ok_or_else(|| {
                    Box::new(Diagnostic::error(
                        field.span,
                        format!("struct is too large (the limit is {MAX_SIZE} bytes)"),
                    ))
                })?;
            end = end.max(extent);
        }
        if let Some(a) = &src.lit.flags.align {
            let a = self.eval_align(src.scope, a)?;
            align = align.max(a);
        }
        let size = if no_padding {
            end
        } else {
            end.next_multiple_of(align.max(1))
        };
        if size > MAX_SIZE {
            return err(
                src.lit.span,
                format!("struct is too large (the limit is {MAX_SIZE} bytes)"),
            );
        }
        self.check_using_clashes(&fields, &filters)?;
        let info = self.types.struct_info_mut(s);
        info.fields = fields;
        if !filters.is_empty() {
            self.using_filters.insert(s, filters);
        }
        if !aliases.is_empty() {
            self.member_aliases.insert(s, aliases);
        }
        info.size = size;
        info.align = align;
        info.layout = LayoutState::Done;
        self.struct_asts.get_mut(&s).unwrap().inits = inits;
        if self.ide.is_some() {
            self.ide_note_layout(s);
        }
        Ok(())
    }

    /// Resolve a storage anchor against all declarations, without needing outer offsets yet.
    fn layout_anchor(
        &mut self,
        owner: StructId,
        fields: &[Field],
        filters: &[(usize, ast::UsingFilter)],
        names: &HashMap<Sym, usize>,
        name: Sym,
        span: Span,
    ) -> Result<(usize, u64)> {
        if let Some(&index) = names.get(&name) {
            return Ok((index, 0));
        }
        for (index, field) in fields.iter().enumerate().filter(|(_, f)| f.using) {
            let target = match filters.iter().find(|(i, _)| *i == index) {
                Some((_, filter)) => match super::using::filter_target(filter, name) {
                    Some(name) => name,
                    None => continue,
                },
                None => name,
            };
            if let Some((path, _)) =
                self.find_promoted_member(vec![(field.ty, 0, target)], span, &mut vec![owner])?
            {
                let mut relative = 0u64;
                for step in path {
                    match step {
                        PathStep::Offset(at) => {
                            relative = relative
                                .checked_add(at)
                                .filter(|&n| n <= MAX_SIZE)
                                .ok_or_else(|| {
                                    Box::new(Diagnostic::error(
                                        span,
                                        "overlay target offset is too large",
                                    ))
                                })?
                        }
                        PathStep::Deref => {
                            return err(
                                span,
                                format!(
                                    "`{name}` is behind a using pointer and has no inline storage offset"
                                ),
                            );
                        }
                    }
                }
                return Ok((index, relative));
            }
        }
        err(span, format!("no field named `{name}` in this struct"))
    }

    /// Two `using` members that bring in the same name are an error, as in Jai. A field of the
    /// struct itself may repeat a promoted name (it shadows it, {#using.15}). Only field names
    /// are compared; the constants and procedures declared in the members' bodies are not.
    fn check_using_clashes(
        &mut self,
        fields: &[Field],
        filters: &[(usize, ast::UsingFilter)],
    ) -> Result<()> {
        if !fields.iter().any(|f| f.using) {
            return Ok(());
        }
        // Where each name came from: the `using` member that brought it in. A field of the
        // struct itself may share the name: the direct member shadows the promoted ones.
        let mut seen: HashMap<Sym, (Span, Option<Sym>)> = HashMap::default();
        for (i, f) in fields.iter().enumerate() {
            if !f.using {
                continue;
            }
            let filter = filters.iter().find(|(at, _)| *at == i).map(|(_, x)| x);
            let mut names = Vec::new();
            if !self.exposed_names(f.ty, filter, &mut Vec::new(), &mut names) {
                continue;
            }
            for name in names {
                match seen.get(&name) {
                    Some(&(first, via)) if first != f.span => {
                        let how = match via {
                            Some(m) => format!("through `using {m}`"),
                            None => "through an earlier `using` member".to_string(),
                        };
                        let member = f.name.map_or("this `using` member".to_string(), |n| {
                            format!("`using {n}`")
                        });
                        return Err(Box::new(
                            Diagnostic::error(
                                f.span,
                                format!("the name `{name}` is brought in twice"),
                            )
                            .with_label(format!("{member} also brings in `{name}`"))
                            .with_note(first, format!("`{name}` is already in this struct, {how}"))
                            .with_help("a `using,only(...)` or `using,except(...)` can leave one of them out"),
                        ));
                    }
                    Some(_) => {}
                    None => {
                        seen.insert(name, (f.span, f.name));
                    }
                }
            }
        }
        Ok(())
    }

    /// The field names a `using` member of type `ty` brings in (through nested `using`s too).
    /// False when they cannot be told: a filter that renames, or a type not laid out yet.
    fn exposed_names(
        &self,
        ty: TypeId,
        filter: Option<&ast::UsingFilter>,
        searched: &mut Vec<StructId>,
        out: &mut Vec<Sym>,
    ) -> bool {
        let ty = self.types.repr_struct(ty);
        let ty = self.types.pointee(ty).unwrap_or(ty);
        let Some(s) = self.types.as_struct(self.types.repr_struct(ty)) else {
            return true;
        };
        let info = self.types.struct_info(s);
        if searched.contains(&s) || info.layout != LayoutState::Done {
            return !searched.is_empty();
        }
        searched.push(s);
        let own_filters = self.using_filters.get(&s);
        let mut names = Vec::new();
        for (i, f) in info.fields.iter().enumerate() {
            names.extend(f.name);
            if f.using {
                let inner = own_filters.and_then(|l| l.iter().find(|(at, _)| *at == i));
                if !self.exposed_names(f.ty, inner.map(|(_, x)| x), searched, &mut names) {
                    return false;
                }
            }
        }
        match filter {
            None | Some(ast::UsingFilter::None) => {}
            Some(ast::UsingFilter::Only(keep)) => {
                names.retain(|n| keep.iter().any(|k| k.name == *n));
            }
            Some(ast::UsingFilter::Except(drop)) => {
                names.retain(|n| drop.iter().all(|k| k.name != *n));
            }
            Some(_) => return false,
        }
        out.extend(names);
        true
    }

    /// An `#align N` value. Generated bindings use odd values (`#align 9`), so only the range is
    /// checked; the bound keeps layout arithmetic from overflowing.
    fn eval_align(&mut self, scope: ScopeId, expr: &ast::Expr) -> Result<u64> {
        const MAX_ALIGN: i128 = 1 << 30;
        let n = self.eval_int(scope, expr)?;
        if !(0..=MAX_ALIGN).contains(&n) {
            return err(
                expr.span,
                format!("alignment {n} is out of range (0 to {MAX_ALIGN})"),
            );
        }
        Ok(n as u64)
    }

    fn collect_fields(
        &mut self,
        scope: ScopeId,
        stmts: &[ast::Stmt],
        out: &mut Vec<FieldItem>,
    ) -> Result<()> {
        for stmt in stmts {
            match &stmt.kind {
                ast::StmtKind::Decl(decl) if decl.kind == ast::DeclKind::Var => {
                    self.collect_decl_fields(scope, decl, out)?
                }
                ast::StmtKind::StaticIf(boxed) => {
                    let crate::ast::StaticIfStmt {
                        cond,
                        then_branch,
                        else_branch,
                    } = &**boxed;
                    let taken = self.eval_static_condition(scope, cond)?;
                    self.ide_note_skipped(
                        cond,
                        if taken {
                            else_branch
                        } else {
                            then_branch
                        },
                    );
                    self.collect_fields(
                        scope,
                        if taken {
                            then_branch
                        } else {
                            else_branch
                        },
                        out,
                    )?;
                }
                ast::StmtKind::Insert(boxed) => {
                    let crate::ast::InsertStmt {
                        value, ..
                    } = &**boxed;
                    let inserted = self.eval_insert_stmts(scope, value)?;
                    // Constants the inserted text declares are members like any written in the
                    // body, visible to the fields that follow and to the struct's users.
                    for stmt in &inserted {
                        if let ast::StmtKind::Decl(decl) = &stmt.kind
                            && decl.kind == ast::DeclKind::Const
                        {
                            for (index, n) in decl.names.iter().enumerate() {
                                self.add_entity(
                                    scope,
                                    n.name,
                                    n.span,
                                    EntityKind::Decl {
                                        decl: decl.clone(),
                                        index,
                                    },
                                    true,
                                );
                            }
                        }
                    }
                    self.collect_fields(scope, &inserted, out)?;
                }
                ast::StmtKind::Place(e) => {
                    let E::Ident(name) = &e.kind else {
                        return err(e.span, "#place needs a field name");
                    };
                    out.push(FieldItem::Place(*name, e.span));
                }
                ast::StmtKind::Overlay(e) => {
                    let E::Ident(name) = &e.kind else {
                        return err(e.span, "#overlay needs a field name");
                    };
                    out.push(FieldItem::Overlay(*name, e.span));
                }
                ast::StmtKind::Expr(e) => match &e.kind {
                    E::Struct(lit) => {
                        // Anonymous nested struct/union: its members are reachable directly.
                        self.check_struct_modify(Sym::intern("anonymous"), lit)?;
                        let t = self.new_struct_type(
                            Sym::intern("anonymous"),
                            lit.clone(),
                            scope,
                            Vec::new(),
                            None,
                        );
                        out.push(FieldItem::Field(FieldDecl {
                            name: None,
                            ty: t,
                            using: true,
                            as_: false,
                            notes: Vec::new(),
                            span: e.span,
                            init: None,
                            scope,
                            align: None,
                            filter: None,
                        }));
                    }
                    _ => return err(e.span, "unexpected expression in struct body"),
                },
                ast::StmtKind::Using(boxed) => {
                    let crate::ast::UsingStmt {
                        value,
                        filter,
                    } = &**boxed;
                    // `using inner.member;` naming fields of this struct: an alias.
                    if let Some(path) = member_path(value)
                        && out.iter().any(|item| match item {
                            FieldItem::Field(d) => d.name == Some(path[0]),
                            FieldItem::Declaration(_, d) => {
                                d.names.iter().any(|n| n.name == path[0])
                            }
                            _ => false,
                        })
                    {
                        out.push(FieldItem::Alias(MemberAlias {
                            path,
                            filter: filter.clone(),
                        }));
                        continue;
                    }
                    // `using Other_Struct;` imports another struct's fields.
                    let t = self.eval_type(scope, value)?;
                    out.push(FieldItem::Field(FieldDecl {
                        name: None,
                        ty: t,
                        using: true,
                        as_: false,
                        notes: Vec::new(),
                        span: stmt.span,
                        init: None,
                        scope,
                        align: None,
                        filter: Some(filter.clone()),
                    }));
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn collect_decl_fields(
        &mut self,
        scope: ScopeId,
        decl: &Rc<ast::Decl>,
        out: &mut Vec<FieldItem>,
    ) -> Result<()> {
        for name in &decl.names {
            self.field_declarations
                .entry(scope)
                .or_default()
                .insert(name.name, decl.clone());
        }
        out.push(FieldItem::Declaration(scope, decl.clone()));
        Ok(())
    }

    pub(super) fn resolve_field_decl_type(
        &mut self,
        scope: ScopeId,
        decl: &Rc<ast::Decl>,
    ) -> Result<TypeId> {
        if let Some(ty) = decl
            .names
            .first()
            .and_then(|n| self.field_types.get(&scope)?.get(&n.name))
        {
            return Ok(*ty);
        }
        let key = (scope, decl.id);
        if !self.field_type_pending.insert(key) {
            return err(decl.span, "cycle in struct field types");
        }
        let result = self.field_decl_type_inner(scope, decl);
        self.field_type_pending.remove(&key);
        if let Ok(ty) = result {
            for name in &decl.names {
                self.field_types
                    .entry(scope)
                    .or_default()
                    .insert(name.name, ty);
            }
        }
        result
    }

    fn field_decl_type_inner(&mut self, scope: ScopeId, decl: &Rc<ast::Decl>) -> Result<TypeId> {
        let ty = match (&decl.ty, &decl.value) {
            (Some(t), _) => self.eval_type(scope, t)?,
            (None, Some(v)) => {
                let op = self.eval_const(scope, v, None)?;
                match op {
                    Operand::Const {
                        ty,
                        value,
                        untyped: true,
                    } => self.default_untyped(ty, &value),
                    Operand::Type(_) => TypeId::TYPE,
                    Operand::Procs(p) if p.len() == 1 => self.proc_type(p[0], v.span)?,
                    other => other.ty(),
                }
            }
            (None, None) => return err(decl.span, "field needs a type"),
        };
        Ok(ty)
    }

    fn resolve_decl_fields(
        &mut self,
        scope: ScopeId,
        decl: &Rc<ast::Decl>,
        out: &mut Vec<FieldItem>,
    ) -> Result<()> {
        let ty = self.resolve_field_decl_type(scope, decl)?;
        let filter = self.field_using_filter(scope, decl, ty)?;
        let align = match &decl.align() {
            Some(a) => Some(self.eval_align(scope, a)?),
            None => None,
        };
        let notes: Vec<Rc<str>> = decl.notes.iter().map(|n| n.text.clone()).collect();
        for name in &decl.names {
            out.push(FieldItem::Field(FieldDecl {
                name: Some(name.name),
                ty,
                using: decl.using,
                as_: decl.as_,
                notes: notes.clone(),
                span: name.span,
                init: decl.value.clone(),
                scope,
                align,
                filter: filter.clone(),
            }));
        }
        Ok(())
    }

    /// The synthesized `#Context` type: Preload's FIRST_ADD_CONTEXT followed by
    /// every `#add_context` in load order.
    pub fn context_type(&mut self, span: Span) -> Result<TypeId> {
        if let Some(t) = self.context_type {
            return Ok(t);
        }
        self.expand_all()?;
        let mut extra: Vec<(Rc<ast::Decl>, ScopeId)> = Vec::new();
        if let Some(preload) = self.preload {
            let ids = self.module_declarations(preload, Sym::intern("FIRST_ADD_CONTEXT"))?;
            if let Some(&id) = ids.first()
                && let Resolved::Const {
                    value: Value::Code(code),
                    ..
                } = self.resolve_entity(id)?
            {
                let body = self.codes[code.0 as usize].clone();
                // The base context struct lives in Runtime_Support.
                let scope = match self.runtime_support {
                    Some(m) => self.modules[m.0 as usize].scope,
                    None => self.code_scopes[code.0 as usize],
                };
                if let ast::CodeBody::Block(block) = &*body {
                    for stmt in &block.stmts {
                        if let ast::StmtKind::AddContext(decl) = &stmt.kind {
                            extra.push((decl.clone(), scope));
                        }
                    }
                }
            }
        }
        extra.extend(self.add_contexts.iter().cloned());
        let lit = Rc::new(ast::StructLit {
            id: ast::AstId::fresh(),
            kind: ast::StructKind::Struct,
            params: Vec::new(),
            body: Vec::new(),
            flags: Default::default(),
            tag: None,
            modify: None,
            notes: Vec::new(),
            span,
        });
        let root = self.root_scope;
        let ty = self.new_struct_type(Sym::intern("Context"), lit, root, Vec::new(), None);
        let s = self.types.as_struct(ty).unwrap();
        // `#add_context name :: value;` adds a constant member (`#Context._Iprof.Zone()`), not a
        // field. It is resolved where it was declared.
        let (consts, extra): (Vec<_>, Vec<_>) = extra
            .into_iter()
            .partition(|(decl, _)| decl.kind == ast::DeclKind::Const);
        let struct_scope = self.struct_asts[&s].scope;
        for (decl, home) in consts {
            for (index, n) in decl.names.iter().enumerate() {
                let kind = EntityKind::Decl {
                    decl: decl.clone(),
                    index,
                };
                let id = self.add_entity(struct_scope, n.name, n.span, kind, true);
                self.entity_mut(id).home = home;
            }
        }
        self.struct_asts.get_mut(&s).unwrap().extra = extra;
        self.context_type = Some(ty);
        self.layout_struct(s, span)?;
        Ok(ty)
    }

    // -----------------------------------------------------------------------
    // Members
    // -----------------------------------------------------------------------

    /// Built-in aggregate members of non-struct types.
    pub(super) fn builtin_members(
        &mut self,
        ty: TypeId,
        span: Span,
    ) -> Result<Vec<(Sym, TypeId, u64)>> {
        let s = |n: &str| Sym::intern(n);
        Ok(match self.types.kind(ty).clone() {
            TypeKind::String => vec![(s("count"), TypeId::S64, 0), (s("data"), TypeId::U8_PTR, 8)],
            TypeKind::Array {
                elem,
                kind: ArrayKind::View,
            } => {
                let p = self.types.pointer(elem);
                vec![(s("count"), TypeId::S64, 0), (s("data"), p, 8)]
            }
            TypeKind::Array {
                elem,
                kind: ArrayKind::Resizable,
            } => {
                let p = self.types.pointer(elem);
                let alloc = self.preload_type("Allocator", span)?;
                vec![
                    (s("count"), TypeId::S64, 0),
                    (s("data"), p, 8),
                    (s("allocated"), TypeId::S64, 16),
                    (s("allocator"), alloc, 24),
                ]
            }
            TypeKind::Any => {
                let ti = self.preload_type("Type_Info", span)?;
                let p = self.types.pointer(ti);
                vec![(s("type"), p, 0), (s("value_pointer"), TypeId::VOID_PTR, 8)]
            }
            _ => Vec::new(),
        })
    }

    /// Find `name` among the fields of `ty`, following `using` fields.
    pub fn find_member(
        &mut self,
        ty: TypeId,
        name: Sym,
        span: Span,
    ) -> Result<Option<(Vec<PathStep>, TypeId)>> {
        self.find_member_in(ty, name, span, &mut Vec::new())
    }

    /// The byte offset and type of field `name` of `ty`, when it sits inline (not behind a
    /// `using` pointer).
    pub fn member_offset(
        &mut self,
        ty: TypeId,
        name: &str,
        span: Span,
    ) -> Result<Option<(u64, TypeId)>> {
        let Some((path, fty)) = self.find_member(ty, Sym::intern(name), span)? else {
            return Ok(None);
        };
        let mut offset = 0;
        for step in &path {
            match step {
                PathStep::Offset(o) => offset += o,
                PathStep::Deref => return Ok(None),
            }
        }
        Ok(Some((offset, fty)))
    }

    /// Where `Source_Code_Location`'s path, line and column sit in it.
    pub fn location_layout(&mut self, span: Span) -> Result<crate::interp::LocationLayout> {
        let ty = self.preload_type("Source_Code_Location", span)?;
        let mut at = |name: &str| -> Result<u64> {
            match self.member_offset(ty, name, span)? {
                Some((offset, _)) => Ok(offset),
                None => err(
                    span,
                    format!("Preload's `Source_Code_Location` has no `{name}`"),
                ),
            }
        };
        Ok(crate::interp::LocationLayout {
            path: at("fully_pathed_filename")?,
            line: at("line_number")?,
            column: at("character_number")?,
        })
    }

    /// `find_member`, skipping structs already searched: `using` pointers can form a cycle
    /// (`S :: struct { using next: *S; }`), which would otherwise recurse forever.
    fn find_member_in(
        &mut self,
        ty: TypeId,
        name: Sym,
        span: Span,
        searched: &mut Vec<StructId>,
    ) -> Result<Option<(Vec<PathStep>, TypeId)>> {
        let ty = self.types.repr_struct(ty);
        let Some(s) = self.types.as_struct(ty) else {
            let members = self.builtin_members(ty, span)?;
            return Ok(members
                .into_iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, t, o)| (vec![PathStep::Offset(o)], t)));
        };
        if searched.contains(&s) {
            return Ok(None);
        }
        searched.push(s);
        self.layout_struct(s, span)?;
        // Without cloning the field list: this runs for every member access.
        let fields = &self.types.struct_info(s).fields;
        if let Some(f) = fields.iter().find(|f| f.name == Some(name)) {
            return Ok(Some((vec![PathStep::Offset(f.offset)], f.ty)));
        }
        let filters = self.using_filters.get(&s).map_or(&[][..], Vec::as_slice);
        let candidates = using_candidates(fields, filters, name);
        if let Some(found) = self.find_promoted_member(candidates, span, searched)? {
            return Ok(Some(found));
        }
        let aliases = self.member_aliases.get(&s).cloned().unwrap_or_default();
        for alias in aliases {
            let Some(target) = alias.target_name(name) else {
                continue;
            };
            let mut full = Vec::new();
            let mut at = ty;
            for &step in alias.path.iter().chain([&target]) {
                // A pointer field on the path is followed.
                if let Some(p) = self.types.pointee(at)
                    && self.types.as_struct(p).is_some()
                {
                    full.push(PathStep::Deref);
                    at = p;
                }
                let Some((mut path, t)) = self.find_member(at, step, span)? else {
                    return err(
                        span,
                        format!("`{step}` is not a member of {}", self.types.name(at)),
                    );
                };
                full.append(&mut path);
                at = t;
            }
            return Ok(Some((full, at)));
        }
        Ok(None)
    }

    /// Shared promoted-field resolution for completed structs and the laid-out
    /// prefix at a #place/#overlay directive. A dynamic pointer step is explicit.
    fn find_promoted_member(
        &mut self,
        candidates: Vec<(TypeId, u64, Sym)>,
        span: Span,
        searched: &mut Vec<StructId>,
    ) -> Result<Option<(Vec<PathStep>, TypeId)>> {
        for (field_ty, offset, name) in candidates {
            let (inner, deref) = match self.types.pointee(field_ty) {
                Some(p) if self.types.as_struct(p).is_some() => (p, true),
                _ => (field_ty, false),
            };
            if let Some((mut path, ty)) = self.find_member_in(inner, name, span, searched)? {
                let mut full = vec![PathStep::Offset(offset)];
                if deref {
                    full.push(PathStep::Deref);
                }
                full.append(&mut path);
                return Ok(Some((full, ty)));
            }
        }
        Ok(None)
    }

    /// Constant members of a struct type (declared in its body), including through `using`.
    pub fn struct_constant(&mut self, ty: TypeId, name: Sym) -> Result<Option<Vec<EntityId>>> {
        self.struct_constant_in(ty, name, &mut Vec::new())
    }

    /// `struct_constant`, skipping structs already searched (cycles of `using` pointers).
    fn struct_constant_in(
        &mut self,
        ty: TypeId,
        name: Sym,
        searched: &mut Vec<StructId>,
    ) -> Result<Option<Vec<EntityId>>> {
        let ty = self.types.repr_struct(ty);
        let Some(s) = self.types.as_struct(ty) else {
            return Ok(None);
        };
        if searched.contains(&s) {
            return Ok(None);
        }
        searched.push(s);
        if let Some(src) = self.struct_asts.get(&s) {
            let scope = src.scope;
            self.expand_pending(scope)?;
            if let Some(ids) = self.scope(scope).names.get(&name)
                && !ids.is_empty()
            {
                return Ok(Some(ids.clone()));
            }
        }
        // Constants of `using` fields are reachable too (`context.default_allocator`). A struct
        // being laid out has no fields yet: a name looked up from its own field types finds
        // only its declared constants.
        if self.types.struct_info(s).layout == LayoutState::InProgress {
            return Ok(None);
        }
        self.layout_struct(s, Span::NONE)?;
        let usings: Vec<TypeId> = self
            .types
            .struct_info(s)
            .fields
            .iter()
            .filter(|f| f.using)
            .map(|f| f.ty)
            .collect();
        for field_ty in usings {
            let inner = self.types.pointee(field_ty).unwrap_or(field_ty);
            if let Some(ids) = self.struct_constant_in(inner, name, searched)? {
                return Ok(Some(ids));
            }
        }
        Ok(None)
    }

    pub fn type_has_member(&mut self, ty: TypeId, name: Sym) -> Result<bool> {
        let target = self.types.pointee(ty).unwrap_or(ty);
        if let Some(&scope) = self.enum_member_scopes.get(&target) {
            return Ok(self.scope(scope).names.get(&name).is_some_and(|ids| {
                ids.iter()
                    .any(|&id| matches!(self.entity(id).kind, EntityKind::EnumMember { .. }))
            }));
        }
        if let TypeKind::Enum(e) = self.types.kind(target) {
            return Ok(self
                .types
                .enum_info(*e)
                .members
                .iter()
                .any(|(n, _)| *n == name));
        }
        if self.struct_constant(target, name)?.is_some() {
            return Ok(true);
        }
        // A name looked up while this struct is being laid out, through a `using g;` of a
        // global of its type (no_api's `using gpu_context;`), from its own body. Layout goes
        // through declarations independently of offsets; asking only whether a declared
        // field exists must not request storage layout recursively.
        if let Some(s) = self.types.as_struct(self.types.repr_struct(target))
            && self.types.struct_info(s).layout == LayoutState::InProgress
            && let Some(src) = self.struct_asts.get(&s)
        {
            let tag = src.lit.tag.as_ref().map(|t| t.name.name);
            let declared = self
                .field_declarations
                .get(&src.scope)
                .is_some_and(|fields| fields.contains_key(&name));
            return Ok(tag == Some(name) || declared);
        }
        Ok(self.find_member(target, name, Span::NONE)?.is_some())
    }

    pub(super) fn apply_path(
        &mut self,
        f: &mut FnCtx,
        mut addr: ir::Val,
        path: &[PathStep],
    ) -> ir::Val {
        for step in path {
            addr = match step {
                PathStep::Offset(o) => f.b.ptr_offset(addr, *o),
                PathStep::Deref => f.b.load(Ty::Ptr, addr),
            };
        }
        addr
    }

    pub fn member_access(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        base: Operand,
        name: Sym,
        span: Span,
    ) -> Result<Operand> {
        match base {
            Operand::Type(t) => return self.type_member(f, scope, t, name, span),
            Operand::Module(m) => {
                return match self.module_member_lookup(m, name)? {
                    Found::Using(entry, member) => self.using_member(f, entry, member, span),
                    Found::Entities(ids) if ids.is_empty() => err(
                        span,
                        format!(
                            "module `{}` has no exported member `{name}`",
                            self.modules[m.0 as usize].name
                        ),
                    ),
                    Found::Entities(ids) => {
                        let result = self.entities_operand(f, scope, &ids, span);
                        // Preserve the actual lookup identity for IDE references, just
                        // as check_ident does. Set it after operand resolution because
                        // an initializer may itself resolve other named expressions.
                        if result.is_ok()
                            && let Some(ide) = self.ide.as_mut()
                        {
                            ide.last_entity = ids.first().map(|&id| (span, id));
                        }
                        result
                    }
                };
            }
            Operand::Const {
                value: Value::String(ref s),
                ..
            } if name.as_str() == "count" => return Ok(Operand::untyped_int(s.len() as i128)),
            // `code.type`: the type of a Code expression, checked where it was written.
            Operand::Const {
                value: Value::Code(code),
                ..
            } if name.as_str() == "type" => {
                let body = self.codes[code.0 as usize].clone();
                let ast::CodeBody::Expr(e) = &*body else {
                    return err(span, "only a Code expression has a type");
                };
                let code_scope = self.code_scopes[code.0 as usize];
                let ty = match self.check_expr_no_emit(code_scope, e)? {
                    Operand::Const {
                        ty,
                        value,
                        untyped: true,
                    } => self.default_untyped(ty, &value),
                    other => other.ty(),
                };
                return Ok(Operand::Type(ty));
            }
            Operand::Const {
                ty, ..
            } if matches!(
                self.types.kind(ty),
                TypeKind::Array {
                    kind: ArrayKind::Fixed(_),
                    ..
                }
            ) && name.as_str() == "count" =>
            {
                let TypeKind::Array {
                    kind: ArrayKind::Fixed(n),
                    ..
                } = *self.types.kind(ty)
                else {
                    unreachable!()
                };
                return Ok(Operand::untyped_int(n as i128));
            }
            _ => {}
        }
        let ty = base.ty();
        // `value.MEMBER` on an enum value yields the member constant (`flags.POLYMORPHIC`).
        if matches!(self.types.kind(ty), TypeKind::Enum(_)) && self.type_has_member(ty, name)? {
            return self.type_member(f, scope, ty, name, span);
        }
        // Fixed arrays: count is constant, data is the base address.
        if let TypeKind::Array {
            elem,
            kind: ArrayKind::Fixed(n),
        } = *self.types.kind(self.types.repr(ty))
        {
            match name.as_str() {
                "count" => return Ok(Operand::untyped_int(n as i128)),
                "data" => {
                    let p = self.types.pointer(elem);
                    // An empty array has no storage: its data is null, as for an empty view.
                    if n == 0 {
                        return Ok(Operand::Const {
                            ty: p,
                            value: Value::Null,
                            untyped: false,
                        });
                    }
                    let (_, addr) = self.address_of(f, base, span)?;
                    return Ok(Operand::Value {
                        ty: p,
                        val: addr,
                    });
                }
                _ => return err(span, format!("fixed array has no member `{name}`")),
            }
        }
        // Auto-dereference one level of pointer to struct.
        let (target, addr) = match self.types.pointee(ty) {
            Some(p)
                if p != TypeId::VOID
                    && !matches!(
                        self.types.kind(p),
                        TypeKind::Int { .. } | TypeKind::Float { .. } | TypeKind::Bool
                    ) =>
            {
                let (_, v) = self.rvalue(f, base, span)?;
                (p, v)
            }
            _ => {
                if let Some(ids) = self.struct_constant(ty, name)? {
                    return self.entities_operand(f, scope, &ids, span);
                }
                let (_, addr) = self.address_of(f, base, span)?;
                (ty, addr)
            }
        };
        // Pointer to fixed array: `p.count`.
        if let TypeKind::Array {
            kind: ArrayKind::Fixed(n),
            elem,
        } = *self.types.kind(target)
        {
            return match name.as_str() {
                "count" => Ok(Operand::untyped_int(n as i128)),
                "data" => {
                    let p = self.types.pointer(elem);
                    Ok(Operand::Value {
                        ty: p,
                        val: addr,
                    })
                }
                _ => err(span, format!("fixed array has no member `{name}`")),
            };
        }
        if let Some((path, mty)) = self.find_member(target, name, span)? {
            let a = self.apply_path(f, addr, &path);
            return Ok(Operand::Place {
                ty: mty,
                addr: a,
            });
        }
        if let Some(ids) = self.struct_constant(target, name)? {
            return self.entities_operand(f, scope, &ids, span);
        }
        Err(Box::new(self.no_member(target, name, span)))
    }

    fn field_using_filter(
        &mut self,
        scope: ScopeId,
        decl: &Rc<ast::Decl>,
        ty: TypeId,
    ) -> Result<Option<ast::UsingFilter>> {
        let Some(filter) = decl.using_filter().filter(|_| decl.using) else {
            return Ok(None);
        };
        let key = (scope, decl.id);
        if let Some(filter) = self.field_filter_cache.get(&key) {
            return Ok(Some(filter.clone()));
        }
        if !self.field_filter_pending.insert(key) {
            return err(decl.span, "cycle in struct using filters");
        }
        let result = self.filter_using_entry(scope, scope::UsingEntry::Type(ty), filter, decl.span);
        self.field_filter_pending.remove(&key);
        let resolved = match result? {
            scope::UsingEntry::Filtered(_, filter) => (*filter).clone(),
            _ => filter.clone(),
        };
        self.field_filter_cache.insert(key, resolved.clone());
        Ok(Some(resolved))
    }

    pub(super) fn declared_promoted_field_type(
        &mut self,
        scope: ScopeId,
        name: Sym,
        span: Span,
    ) -> Result<Option<TypeId>> {
        let ScopeKind::Struct(owner_ty) = self.scope(scope).kind else {
            return Ok(None);
        };
        let owner = self.types.as_struct(owner_ty).unwrap();
        let candidates = self
            .field_using_order
            .get(&scope)
            .cloned()
            .unwrap_or_else(|| {
                // Before conditional/insert expansion, only direct declarations are available.
                self.struct_asts[&owner]
                    .lit
                    .body
                    .iter()
                    .filter_map(|stmt| match &stmt.kind {
                        ast::StmtKind::Decl(decl)
                            if decl.using && decl.kind == ast::DeclKind::Var =>
                        {
                            Some(FieldUsing::Declaration(scope, decl.clone()))
                        }
                        _ => None,
                    })
                    .collect()
            });
        for candidate in candidates {
            let (ty, filter) = match candidate {
                FieldUsing::Declaration(home, decl) => {
                    let ty = self.resolve_field_decl_type(home, &decl)?;
                    let filter = self
                        .field_using_filter(home, &decl, ty)?
                        .unwrap_or(ast::UsingFilter::None);
                    (ty, filter)
                }
                FieldUsing::Inline(ty, filter) => (ty, filter),
            };
            let Some(target) = super::using::filter_target(&filter, name) else {
                continue;
            };
            if let Some((_, member_ty)) =
                self.find_promoted_member(vec![(ty, 0, target)], span, &mut vec![owner])?
            {
                self.ide_note_member_use(span, self.types.pointee(ty).unwrap_or(ty), target);
                return Ok(Some(member_ty));
            }
        }
        Ok(None)
    }

    pub(super) fn pending_struct_field_type(
        &mut self,
        ty: TypeId,
        name: Sym,
        span: Span,
    ) -> Result<Option<TypeId>> {
        let Some(s) = self.types.as_struct(self.types.repr_struct(ty)) else {
            return Ok(None);
        };
        if self.types.struct_info(s).layout != LayoutState::InProgress {
            return Ok(None);
        }
        let scope = self.struct_asts[&s].scope;
        let Some(decl) = self
            .field_declarations
            .get(&scope)
            .and_then(|fields| fields.get(&name))
            .cloned()
        else {
            return self.declared_promoted_field_type(scope, name, span);
        };
        let ty = self.resolve_field_decl_type(scope, &decl)?;
        let declared = decl.names.iter().find(|n| n.name == name).unwrap().span;
        self.ide_note_named_field(span, name, declared, ty);
        Ok(Some(ty))
    }

    fn type_member(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        t: TypeId,
        name: Sym,
        span: Span,
    ) -> Result<Operand> {
        // `array.type` where `array: *Struct(...)` is bound as a type (implicit polymorphs).
        if let Some(p) = self.types.pointee(t)
            && self.types.as_struct(p).is_some()
            && let Some(ids) = self.struct_constant(p, name)?
        {
            return self.entities_operand(f, scope, &ids, span);
        }
        match self.types.kind(t).clone() {
            TypeKind::Enum(_) if name.as_str() == "loose" => {
                Ok(Operand::Type(self.types.loose_enum(t)))
            }
            TypeKind::Enum(e) => {
                if let Some(&member_scope) = self.enum_member_scopes.get(&t)
                    && let Some(ids) = self.scope(member_scope).names.get(&name).cloned()
                    && ids
                        .iter()
                        .any(|&id| matches!(self.entity(id).kind, EntityKind::EnumMember { .. }))
                {
                    return self.entities_operand(f, member_scope, &ids, span);
                }
                if let Some(&(_, v)) = self
                    .types
                    .enum_info(e)
                    .members
                    .iter()
                    .find(|(n, _)| *n == name)
                {
                    return Ok(Operand::Const {
                        ty: t,
                        value: Value::Int(v),
                        untyped: false,
                    });
                }
                Err(Box::new(self.no_member(t, name, span)))
            }
            TypeKind::Struct(_) => {
                if let Some(ids) = self.struct_constant(t, name)? {
                    return self.entities_operand(f, scope, &ids, span);
                }
                if let Some(ty) = self.pending_struct_field_type(t, name, span)? {
                    return Ok(Operand::Type(ty));
                }
                // `Type.field` without an instance names the field's type, for reaching its
                // constants (`Anim.joint_map.Entry`).
                if let Some((_, field_ty)) = self.find_member(t, name, span)? {
                    return Ok(Operand::Type(field_ty));
                }
                err(
                    span,
                    format!(
                        "type {} has no constant member `{name}`",
                        self.types.name(t)
                    ),
                )
            }
            TypeKind::Array {
                kind: ArrayKind::Fixed(n),
                ..
            } if name.as_str() == "count" => Ok(Operand::untyped_int(n as i128)),
            TypeKind::Distinct(d) => {
                let base = self.types.distincts[d.0 as usize].base;
                self.type_member(f, scope, base, name, span)
            }
            _ => Err(Box::new(self.no_member(t, name, span))),
        }
    }

    pub fn using_member(
        &mut self,
        f: &mut FnCtx,
        entry: UsingEntry,
        member: Sym,
        span: Span,
    ) -> Result<Operand> {
        match entry {
            UsingEntry::Filtered(base, filter) => {
                let Some(target) = super::using::filter_target(&filter, member) else {
                    return err(span, format!("using filter excludes `{member}`"));
                };
                self.using_member(f, *base, target, span)
            }
            UsingEntry::Type(t) => self.type_member(f, ScopeId(0), t, member, span),
            UsingEntry::Module(m, _) => match self.module_lookup(m, member)? {
                Found::Using(entry, member) => self.using_member(f, entry, member, span),
                Found::Entities(ids) => self.entities_operand(f, ScopeId(0), &ids, span),
            },
            UsingEntry::Place {
                ty,
                entity,
            } => {
                let base = match self.entity(entity).kind.clone() {
                    EntityKind::Local {
                        ty,
                        addr,
                        ..
                    } => Operand::Place {
                        ty,
                        addr,
                    },
                    _ => match self.resolve_entity(entity)? {
                        Resolved::Global {
                            storage,
                            ty,
                        } => Operand::Place {
                            ty,
                            addr: f.b.storage_addr(storage),
                        },
                        _ => return err(span, "invalid using target"),
                    },
                };
                let _ = ty;
                self.member_access(f, ScopeId(0), base, member, span)
            }
        }
    }

    /// Resolve the target of a `using` statement at file/struct scope.
    pub fn using_target(&mut self, scope: ScopeId, value: &ast::Expr) -> Result<UsingEntry> {
        match self.eval_const(scope, value, None)? {
            Operand::Module(m) => Ok(UsingEntry::Module(m, None)),
            Operand::Type(t) => Ok(UsingEntry::Type(t)),
            other => {
                // `using global_var;`
                if let E::Ident(name) = &value.kind {
                    let ids = self.lookup(scope, *name)?;
                    if let Some(&id) = ids.first()
                        && let Resolved::Global {
                            ty, ..
                        } = self.resolve_entity(id)?
                    {
                        return Ok(UsingEntry::Place {
                            ty,
                            entity: id,
                        });
                    }
                }
                // `using module.global_var;`
                if let E::Member(base, member) = &value.kind
                    && let Operand::Module(m) = self.eval_const(scope, base, None)?
                    && let Found::Entities(ids) = self.module_lookup(m, member.name)?
                    && let Some(&id) = ids.first()
                    && let Resolved::Global {
                        ty, ..
                    } = self.resolve_entity(id)?
                {
                    return Ok(UsingEntry::Place {
                        ty,
                        entity: id,
                    });
                }
                err(
                    value.span,
                    format!("cannot use `using` on {}", self.describe(&other)),
                )
            }
        }
    }

    // -----------------------------------------------------------------------
    // Default values
    // -----------------------------------------------------------------------

    /// Offset and type of the member `name` of a `using` field of `s` (searched through nested `using`s).
    fn find_used_member(
        &mut self,
        s: StructId,
        name: Sym,
        span: Span,
    ) -> Result<Option<(u64, TypeId)>> {
        self.layout_struct(s, span)?;
        let fields = self.types.struct_info(s).fields.clone();
        for f in fields.iter().filter(|f| f.using) {
            let Some(inner) = self.types.as_struct(f.ty) else {
                continue;
            };
            self.layout_struct(inner, span)?;
            let inner_fields = self.types.struct_info(inner).fields.clone();
            if let Some(m) = inner_fields.iter().find(|m| m.name == Some(name)) {
                return Ok(Some((f.offset + m.offset, m.ty)));
            }
            if let Some((o, t)) = self.find_used_member(inner, name, span)? {
                return Ok(Some((f.offset + o, t)));
            }
        }
        Ok(None)
    }

    /// Offset and type of the member a struct-body override assigns: `name` (a member of a
    /// `using` field) or a path into a field (`base.callback`).
    fn override_target(
        &mut self,
        s: StructId,
        lhs: &ast::Expr,
        span: Span,
    ) -> Result<Option<(u64, TypeId)>> {
        match &lhs.kind {
            E::Ident(name) => self.find_used_member(s, *name, span),
            E::Member(base, name) => {
                let Some((offset, ty)) = self.override_path(s, base, span)? else {
                    return Ok(None);
                };
                let Some(inner) = self.types.as_struct(ty) else {
                    return Ok(None);
                };
                Ok(self
                    .struct_member(inner, name.name, span)?
                    .map(|(o, t)| (offset + o, t)))
            }
            _ => Ok(None),
        }
    }

    /// `override_target` for an intermediate path segment, which may also be a direct field.
    fn override_path(
        &mut self,
        s: StructId,
        expr: &ast::Expr,
        span: Span,
    ) -> Result<Option<(u64, TypeId)>> {
        if let E::Ident(name) = &expr.kind {
            return self.struct_member(s, *name, span);
        }
        self.override_target(s, expr, span)
    }

    /// A field of `s` by name, directly or through `using` fields.
    fn struct_member(
        &mut self,
        s: StructId,
        name: Sym,
        span: Span,
    ) -> Result<Option<(u64, TypeId)>> {
        self.layout_struct(s, span)?;
        let fields = self.types.struct_info(s).fields.clone();
        if let Some(f) = fields.iter().find(|f| f.name == Some(name)) {
            return Ok(Some((f.offset, f.ty)));
        }
        self.find_used_member(s, name, span)
    }

    /// Constant image of a default-initialized value (`None` = all zero).
    pub fn default_initializer(&mut self, ty: TypeId, span: Span) -> Result<Option<Rc<Aggregate>>> {
        if let Some(img) = self.default_images.get(&ty) {
            return Ok(img.clone());
        }
        let img = match self.types.kind(ty).clone() {
            TypeKind::Struct(s) => {
                self.layout_struct(s, span)?;
                let size = self.types.struct_info(s).size;
                let fields = self.types.struct_info(s).fields.clone();
                let inits = self.struct_asts[&s].inits.clone();
                // Built on the first nonzero default: a struct whose defaults are all zero
                // (however large) never gets an image.
                let mut image: Option<Aggregate> = None;
                for (field, (init, scope)) in fields.iter().zip(inits) {
                    match init {
                        Some(e) if matches!(e.kind, E::Uninit) => {}
                        Some(e) => {
                            let value = self.const_value_of_type(scope, &e, field.ty)?;
                            let agg = image_of(&mut image, size, span)?;
                            self.write_value(agg, field.offset, &value, field.ty, e.span)?;
                        }
                        None => {
                            if let Some(inner) = self.default_initializer(field.ty, span)? {
                                write_agg(image_of(&mut image, size, span)?, field.offset, &inner);
                            }
                        }
                    }
                }
                // `member = value;` in the body overrides the default of a member
                // reached through a `using` field.
                let (lit, scope) = {
                    let src = &self.struct_asts[&s];
                    (src.lit.clone(), src.scope)
                };
                for stmt in &lit.body {
                    let ast::StmtKind::Assign {
                        op: ast::AssignOp::Assign,
                        lhs,
                        rhs,
                    } = &stmt.kind
                    else {
                        continue;
                    };
                    let ([l], [r]) = (lhs.as_slice(), rhs.as_slice()) else {
                        continue;
                    };
                    if let Some((offset, fty)) = self.override_target(s, l, span)? {
                        let value = self.const_value_of_type(scope, r, fty)?;
                        let agg = image_of(&mut image, size, span)?;
                        self.write_value(agg, offset, &value, fty, r.span)?;
                    }
                }
                image.map(Rc::new)
            }
            TypeKind::Array {
                elem,
                kind: ArrayKind::Fixed(n),
            } => match self.default_initializer(elem, span)? {
                Some(inner) => {
                    let size = self.size_of(elem, span)?;
                    let mut agg = Aggregate::zeroed(size.saturating_mul(n), span)?;
                    for i in 0..n {
                        write_agg(&mut agg, i * size, &inner);
                    }
                    Some(Rc::new(agg))
                }
                None => None,
            },
            TypeKind::Distinct(d) => {
                let base = self.types.distincts[d.0 as usize].base;
                self.default_initializer(base, span)?
            }
            _ => None,
        };
        self.default_images.insert(ty, img.clone());
        Ok(img)
    }

    /// Evaluate a constant expression converted to `ty`.
    pub fn const_value_of_type(
        &mut self,
        scope: ScopeId,
        expr: &ast::Expr,
        ty: TypeId,
    ) -> Result<Value> {
        let op = self.eval_const(scope, expr, Some(ty))?;
        self.const_value_of_operand(scope, op, ty, expr.span)
    }

    /// Convert an evaluated constant operand to type `ty` and read its value.
    pub fn const_value_of_operand(
        &mut self,
        scope: ScopeId,
        op: Operand,
        ty: TypeId,
        span: Span,
    ) -> Result<Value> {
        let file = self.scope_file(scope);
        let mut scratch = FnCtx::new(
            "const".into(),
            ir::Sig {
                params: vec![Ty::Ptr],
                returns: vec![],
                conv: ir::Conv::Jai,
                c_varargs: false,
                c_fixed: 0,
                c_abi: None,
            },
            file,
        );
        scratch.compile_time = true;
        scratch.set_context(Some(scratch.b.param(0)));
        let converted = self.convert(&mut scratch, op, ty, span)?;
        match converted {
            Operand::Const {
                value, ..
            } => Ok(value),
            Operand::Type(t) => Ok(Value::Type(t)),
            Operand::Procs(p) if p.len() == 1 => Ok(Value::Proc(p[0])),
            other => {
                // Conversions that need code (e.g. boxing into Any) run as a thunk.
                let op = self.run_thunk(scratch, other, span)?;
                match op {
                    Operand::Const {
                        value, ..
                    } => Ok(value),
                    other => err(
                        span,
                        format!("expected a constant, found {}", self.describe(&other)),
                    ),
                }
            }
        }
    }

    /// Initialize memory at `addr` with the default value of `ty`.
    pub fn init_default(
        &mut self,
        f: &mut FnCtx,
        ty: TypeId,
        addr: ir::Val,
        span: Span,
    ) -> Result<()> {
        let size = self.size_of(ty, span)?;
        match self.default_initializer(ty, span)? {
            None => f.b.zero(addr, size),
            Some(img) => {
                let g = match self.default_globals.get(&ty) {
                    Some(&g) => g,
                    None => {
                        let align = self.align_of(ty, span)?;
                        let g = self.program.add_global(ir::Global {
                            name: format!("default.{}", self.types.name(ty)),
                            size,
                            align,
                            init: img.bytes.clone(),
                            relocs: img.relocs.clone(),
                            read_only: true,
                            export: None,
                        });
                        self.default_globals.insert(ty, g);
                        g
                    }
                };
                let src = f.b.global_addr(g);
                f.b.copy(addr, src, size);
            }
        }
        Ok(())
    }

    /// `initializer_of(T)`: a `(*void) #no_context` procedure that default-initializes.
    pub fn initializer_proc(&mut self, ty: TypeId, span: Span) -> Result<(ir::FuncId, TypeId)> {
        let proc_ty = self
            .types
            .intern(TypeKind::Proc(Rc::new(crate::types::ProcType {
                params: vec![TypeId::VOID_PTR],
                returns: Vec::new(),
                variadic: false,
                c_varargs: false,
                c_call: false,
                no_context: true,
                non_pod_return: false,
            })));
        if let Some(&func) = self.initializers.get(&ty) {
            return Ok((func, proc_ty));
        }
        let name = format!("__init_{}", self.types.name(ty));
        let func = self.program.reserve_func(name.clone());
        self.initializers.insert(ty, func);
        let sig = ir::Sig {
            params: vec![Ty::Ptr],
            returns: vec![],
            conv: ir::Conv::Jai,
            c_varargs: false,
            c_fixed: 0,
            c_abi: None,
        };
        let mut f = FnCtx::new(name, sig, FileId(0));
        let addr = f.b.param(0);
        self.init_default(&mut f, ty, addr, span)?;
        f.b.ret(Vec::new());
        self.program.funcs[func.0 as usize] = Some(f.b.finish());
        Ok((func, proc_ty))
    }

    // -----------------------------------------------------------------------
    // Literals
    // -----------------------------------------------------------------------

    #[allow(clippy::too_many_arguments)]
    pub fn check_struct_literal(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        ty_expr: Option<&ast::Expr>,
        args: &[ast::Arg],
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        let ty = match ty_expr {
            Some(t) => self.eval_type_in(f, scope, t)?,
            None => match expected {
                Some(t) => t,
                None => {
                    return err(
                        span,
                        "cannot infer the type of `.{...}` here; write `Type.{...}`",
                    );
                }
            },
        };
        let size = self.size_of(ty, span)?;
        // Member list for positional arguments.
        let members: Vec<(Option<Sym>, TypeId, u64)> =
            match self.types.as_struct(self.types.repr_struct(ty)) {
                Some(s) => self
                    .types
                    .struct_info(s)
                    .fields
                    .iter()
                    .map(|f| (f.name, f.ty, f.offset))
                    .collect(),
                None => self
                    .builtin_members(self.types.repr(ty), span)?
                    .into_iter()
                    .map(|(n, t, o)| (Some(n), t, o))
                    .collect(),
            };
        if members.is_empty()
            && !args.is_empty()
            && self.types.as_struct(self.types.repr_struct(ty)).is_none()
        {
            return err(
                span,
                format!(
                    "cannot use a struct literal for type {}",
                    self.types.name(ty)
                ),
            );
        }
        // Check each argument against its member type.
        let mut values: Vec<(Vec<PathStep>, TypeId, Operand, Span)> = Vec::new();
        let mut positional = 0;
        // `.{a.b = v, xs[1] = w}`: assigned after the ordinary fields.
        let (targeted, args): (Vec<&ast::Arg>, Vec<&ast::Arg>) =
            args.iter().partition(|a| a.target.is_some());
        for arg in args {
            let (path, mty) = match arg.name {
                Some(n) => match self.find_member(ty, n.name, n.span)? {
                    Some(m) => {
                        self.ide_note_member_use(n.span, ty, n.name);
                        self.ide_note_member_ref(n.span, n.name, m.1);
                        m
                    }
                    None => {
                        return Err(Box::new(self.no_member(ty, n.name, n.span)));
                    }
                },
                None => {
                    let Some(&(_, mty, off)) = members.get(positional) else {
                        return err(
                            arg.value.span,
                            format!("too many values for {}", self.types.name(ty)),
                        );
                    };
                    positional += 1;
                    (vec![PathStep::Offset(off)], mty)
                }
            };
            let op = self.check_expr(f, scope, &arg.value, Some(mty))?;
            let op = self.convert(f, op, mty, arg.value.span)?;
            values.push((path, mty, op, arg.value.span));
        }
        let all_const = values.iter().all(|(path, _, op, _)| {
            path.iter().all(|s| matches!(s, PathStep::Offset(_)))
                && (op.is_const() || matches!(op, Operand::Procs(p) if p.len() == 1))
        });
        if all_const && targeted.is_empty() {
            let mut agg = match self.default_initializer(ty, span)? {
                Some(img) => (*img).clone(),
                None => Aggregate::zeroed(size, span)?,
            };
            for (path, mty, op, vspan) in &values {
                let offset: u64 = path
                    .iter()
                    .map(|s| {
                        if let PathStep::Offset(o) = s {
                            *o
                        } else {
                            0
                        }
                    })
                    .sum();
                let value = match op {
                    Operand::Procs(p) => Value::Proc(p[0]),
                    other => other.const_value().unwrap(),
                };
                self.write_value(&mut agg, offset, &value, *mty, *vspan)?;
            }
            return Ok(Operand::Const {
                ty,
                value: Value::Bytes(Rc::new(agg)),
                untyped: false,
            });
        }
        let align = self.align_of(ty, span)?;
        let tmp = f.b.alloca(size.max(1), align);
        self.init_default(f, ty, tmp, span)?;
        for (path, mty, op, vspan) in values {
            let at = self.apply_path(f, tmp, &path);
            let (_, v) = self.rvalue(f, op, vspan)?;
            self.store_value(f, mty, at, v, vspan)?;
        }
        for arg in targeted {
            let target = arg.target.as_ref().unwrap();
            let Operand::Place {
                ty: pty,
                addr,
            } = self.literal_target(f, scope, ty, tmp, target)?
            else {
                return err(target.span, "this field target cannot be assigned");
            };
            let op = self.check_expr(f, scope, &arg.value, Some(pty))?;
            let op = self.convert(f, op, pty, arg.value.span)?;
            let (_, v) = self.rvalue(f, op, arg.value.span)?;
            self.store_value(f, pty, addr, v, arg.value.span)?;
        }
        Ok(Operand::Value {
            ty,
            val: tmp,
        })
    }

    /// The place a literal field target (`a.b`, `xs[i]`) names inside the
    /// literal being built at `base`.
    fn literal_target(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        ty: TypeId,
        base: ir::Val,
        target: &ast::Expr,
    ) -> Result<Operand> {
        match &target.kind {
            E::Ident(name) => {
                let whole = Operand::Place {
                    ty,
                    addr: base,
                };
                self.member_access(f, scope, whole, *name, target.span)
            }
            E::Member(inner, member) => {
                let inner = self.literal_target(f, scope, ty, base, inner)?;
                self.member_access(f, scope, inner, member.name, member.span)
            }
            E::Index(inner, index) => {
                let inner = self.literal_target(f, scope, ty, base, inner)?;
                let index_op = self.check_expr(f, scope, index, Some(TypeId::S64))?;
                self.index_operand(f, scope, inner, index_op, index.span, target.span)
            }
            _ => err(
                target.span,
                "a literal field target must be a member or index path",
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn check_array_literal(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        ty_expr: Option<&ast::Expr>,
        elems: &[ast::Expr],
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        let elem_ty = match ty_expr {
            Some(t) => Some(self.eval_type_in(f, scope, t)?),
            None => match expected.map(|e| self.types.kind(self.types.repr(e)).clone()) {
                Some(TypeKind::Array {
                    elem, ..
                }) => Some(elem),
                _ => None,
            },
        };
        let mut ops = Vec::new();
        let mut elem_ty = elem_ty;
        for e in elems {
            let op = self.check_expr(f, scope, e, elem_ty)?;
            if elem_ty.is_none() {
                let settled = self.settle_untyped(op.clone(), None);
                elem_ty = Some(match &settled {
                    Operand::Procs(p) if p.len() == 1 => self.proc_type(p[0], e.span)?,
                    other => other.ty(),
                });
            }
            let op = self.convert(f, op, elem_ty.unwrap(), e.span)?;
            ops.push((op, e.span));
        }
        let Some(elem) = elem_ty else {
            return err(
                span,
                "cannot infer the element type of an empty array literal",
            );
        };
        let n = ops.len() as u64;
        let mut ty = self.types.array(elem, ArrayKind::Fixed(n));
        // `g: Grid3i = .[1, 4, 9]`: a literal takes a `#type,distinct` variant of its own type.
        if ty_expr.is_none()
            && let Some(e) = expected
            && matches!(self.types.kind(e), TypeKind::Distinct(_))
            && self.types.repr(e) == ty
        {
            ty = e;
        }
        let esize = self.size_of(elem, span)?;
        if ops
            .iter()
            .all(|(op, _)| op.is_const() || matches!(op, Operand::Procs(p) if p.len() == 1))
        {
            let mut agg = Aggregate::zeroed(esize.saturating_mul(n), span)?;
            for (i, (op, vspan)) in ops.iter().enumerate() {
                let value = match op {
                    Operand::Procs(p) => Value::Proc(p[0]),
                    other => other.const_value().unwrap(),
                };
                self.write_value(&mut agg, i as u64 * esize, &value, elem, *vspan)?;
            }
            return Ok(Operand::Const {
                ty,
                value: Value::Bytes(Rc::new(agg)),
                untyped: false,
            });
        }
        let align = self.align_of(elem, span)?;
        let tmp = f.b.alloca((esize * n).max(1), align);
        for (i, (op, vspan)) in ops.into_iter().enumerate() {
            let at = f.b.ptr_offset(tmp, i as u64 * esize);
            let (_, v) = self.rvalue(f, op, vspan)?;
            self.store_value(f, elem, at, v, vspan)?;
        }
        Ok(Operand::Value {
            ty,
            val: tmp,
        })
    }

    // -----------------------------------------------------------------------
    // Constant images
    // -----------------------------------------------------------------------

    /// Write a constant `value` of type `ty` into `agg` at `offset`.
    pub fn write_value(
        &mut self,
        agg: &mut Aggregate,
        offset: u64,
        value: &Value,
        ty: TypeId,
        span: Span,
    ) -> Result<()> {
        let size = self.size_of(ty, span)?;
        let o = offset as usize;
        let r = self.types.repr(ty);
        // Overwriting (a literal field over the struct's default): drop the old pointers there.
        agg.relocs
            .retain(|reloc| reloc.offset < offset || reloc.offset >= offset + size);
        match value {
            Value::Int(i) => {
                if self.types.is_float(r) {
                    return self.write_value(agg, offset, &Value::float(*i as f64), ty, span);
                }
                let bytes = (*i as u128).to_le_bytes();
                agg.bytes[o..o + size as usize].copy_from_slice(&bytes[..size as usize]);
            }
            Value::Float(x, _) => {
                if r == TypeId::F32 {
                    agg.bytes[o..o + 4].copy_from_slice(&(*x as f32).to_le_bytes());
                } else {
                    agg.bytes[o..o + 8].copy_from_slice(&x.to_le_bytes());
                }
            }
            Value::Bool(b) => agg.bytes[o] = *b as u8,
            // `null` over a default (a union member sharing a string's storage) clears it.
            Value::Null => agg.bytes[o..o + size as usize].fill(0),
            Value::Void => {}
            Value::String(s) => {
                if r != TypeId::STRING {
                    return err(
                        span,
                        format!("string constant cannot initialize {}", self.types.name(ty)),
                    );
                }
                agg.bytes[o..o + 8].copy_from_slice(&(s.len() as u64).to_le_bytes());
                if !s.is_empty() {
                    let g = self.string_global(s);
                    agg.relocs.push(ir::Reloc {
                        offset: offset + 8,
                        target: ir::RelocTarget::Global(g),
                        addend: 0,
                    });
                }
            }
            Value::Type(t) => {
                let g = self.type_info_global(*t, span)?;
                agg.relocs.push(ir::Reloc {
                    offset,
                    target: ir::RelocTarget::Global(g),
                    addend: 0,
                });
            }
            Value::Proc(p) => {
                let target = match self.proc_func(*p, span)? {
                    procs::ProcTarget::Func(id) => ir::RelocTarget::Func(id),
                    procs::ProcTarget::Foreign(id) => ir::RelocTarget::Foreign(id),
                };
                agg.relocs.push(ir::Reloc {
                    offset,
                    target,
                    addend: 0,
                });
            }
            Value::Bytes(inner) => write_agg(agg, offset, inner),
            Value::Code(c) => agg.bytes[o..o + 8].copy_from_slice(&(c.0 as u64).to_le_bytes()),
        }
        Ok(())
    }
}

/// The image being built in `slot`, allocated (zeroed, `size` bytes) on first use.
fn image_of(slot: &mut Option<Aggregate>, size: u64, span: Span) -> Result<&mut Aggregate> {
    if slot.is_none() {
        *slot = Some(Aggregate::zeroed(size, span)?);
    }
    Ok(slot.as_mut().unwrap())
}

/// Copy a nested aggregate image into `agg` at `offset`.
pub fn write_agg(agg: &mut Aggregate, offset: u64, inner: &Aggregate) {
    let o = offset as usize;
    agg.bytes[o..o + inner.bytes.len()].copy_from_slice(&inner.bytes);
    agg.relocs
        .retain(|r| r.offset < offset || r.offset >= offset + inner.bytes.len() as u64);
    for r in &inner.relocs {
        agg.relocs.push(ir::Reloc {
            offset: r.offset + offset,
            target: r.target,
            addend: r.addend,
        });
    }
}

fn const_operand(value: Value, ty: TypeId) -> Operand {
    match value {
        Value::Type(t) => Operand::Type(t),
        value => Operand::Const {
            ty,
            value,
            untyped: false,
        },
    }
}
