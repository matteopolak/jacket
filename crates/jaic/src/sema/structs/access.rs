//! Member access expressions and the types of promoted and pending members.
use super::model::FieldUsing;
use crate::diagnostics::Code;
use crate::sema::lower::{FnCtx, Operand, Origin};
use crate::sema::scope::{EntityId, EntityKind, Found, Resolved, ScopeKind};
use crate::sema::{Compiler, Rc, Result, ScopeId, Span, Sym, TypeId, Value, ast, err, scope};
use crate::types::{ArrayKind, LayoutState, TypeKind};

impl Compiler {
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
                    Found::Entities(ids) if ids.is_empty() => {
                        Err(Box::new(self.no_module_member(m, name, span)))
                    }
                    Found::Entities(ids) => {
                        let result = self.entities_operand(f, scope, &ids, span);
                        // Preserve the actual lookup identity for IDE references, just
                        // as check_ident does. Set it after operand resolution because
                        // an initializer may itself resolve other named expressions.
                        if result.is_ok() {
                            self.ed.resolved(span, ids[0]);
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
                let body = self.pr.codes[code.0 as usize].clone();
                let ast::CodeBody::Expr(e) = &*body else {
                    return err(
                        Code::StructOnlyCodeExpression,
                        span,
                        "only a Code expression has a type",
                    );
                };
                let code_scope = self.pr.code_scopes[code.0 as usize];
                let ty = match self.check_expr_no_emit(code_scope, e)? {
                    Operand::Const {
                        ty,
                        value,
                        untyped: true,
                        ..
                    } => self.ty.default_untyped(ty, &value),
                    other => other.ty(),
                };
                return Ok(Operand::Type(ty));
            }
            Operand::Const {
                ty, ..
            } if matches!(
                self.ty.types.kind(ty),
                TypeKind::Array {
                    kind: ArrayKind::Fixed(_),
                    ..
                }
            ) && name.as_str() == "count" =>
            {
                let TypeKind::Array {
                    kind: ArrayKind::Fixed(n),
                    ..
                } = *self.ty.types.kind(ty)
                else {
                    unreachable!()
                };
                return Ok(Operand::untyped_int(n as i128));
            }
            _ => {}
        }
        let ty = base.ty();
        // `value.MEMBER` on an enum value yields the member constant (`flags.POLYMORPHIC`).
        if matches!(self.ty.types.kind(ty), TypeKind::Enum(_)) && self.type_has_member(ty, name)? {
            return self.type_member(f, scope, ty, name, span);
        }
        // Fixed arrays: count is constant, data is the base address.
        if let TypeKind::Array {
            elem,
            kind: ArrayKind::Fixed(n),
        } = *self.ty.types.kind(self.ty.types.repr(ty))
        {
            match name.as_str() {
                "count" => return Ok(Operand::untyped_int(n as i128)),
                "data" => {
                    let p = self.ty.types.pointer(elem);
                    // An empty array has no storage: its data is null, as for an empty view.
                    if n == 0 {
                        return Ok(Operand::Const {
                            ty: p,
                            value: Value::Null,
                            untyped: false,
                            origin: Origin::Computed,
                        });
                    }
                    let (_, addr) = self.address_of(f, base, span)?;
                    return Ok(Operand::Value {
                        ty: p,
                        val: addr,
                    });
                }
                _ => {
                    return err(
                        Code::StructFixedArray,
                        span,
                        format!("fixed array has no member `{name}`"),
                    );
                }
            }
        }
        // Auto-dereference one level of pointer to struct.
        let (target, addr) = match self.ty.types.pointee(ty) {
            Some(p)
                if p != TypeId::VOID
                    && !matches!(
                        self.ty.types.kind(p),
                        TypeKind::Int { .. } | TypeKind::Float { .. } | TypeKind::Bool
                    ) =>
            {
                let (_, v) = self.rvalue(f, base, span)?;
                (p, v)
            }
            _ => {
                if let Some(ids) = self.struct_constant(ty, name)? {
                    return self.constant_operand(f, scope, &ids, span);
                }
                let (_, addr) = self.address_of(f, base, span)?;
                (ty, addr)
            }
        };
        // Pointer to fixed array: `p.count`.
        if let TypeKind::Array {
            kind: ArrayKind::Fixed(n),
            elem,
        } = *self.ty.types.kind(target)
        {
            return match name.as_str() {
                "count" => Ok(Operand::untyped_int(n as i128)),
                "data" => {
                    let p = self.ty.types.pointer(elem);
                    Ok(Operand::Value {
                        ty: p,
                        val: addr,
                    })
                }
                _ => err(
                    Code::StructFixedArray,
                    span,
                    format!("fixed array has no member `{name}`"),
                ),
            };
        }
        if let Some((path, mty, origin)) =
            self.find_member_in(target, name, span, &mut Vec::new())?
        {
            let a = self.apply_path(f, addr, &path);
            let parameters = origin
                .and_then(|origin| self.pr.callable_fields.get(&origin))
                .cloned();
            return Ok(Operand::Place {
                ty: mty,
                addr: a,
            }
            .with_callable(parameters));
        }
        if let Some(ids) = self.struct_constant(target, name)? {
            return self.constant_operand(f, scope, &ids, span);
        }
        if self.is_stand_in(target) {
            // The value is unknown because an earlier error already said why: its members
            // are unknown too, not wrong.
            let ty = self.ide_stand_in_member(target, name, span);
            return Ok(self.ide_sketch_value(f, ty, span));
        }
        Err(Box::new(self.no_member(target, name, span)))
    }

    pub(super) fn field_using_filter(
        &mut self,
        scope: ScopeId,
        decl: &Rc<ast::Decl>,
        ty: TypeId,
    ) -> Result<Option<ast::UsingFilter>> {
        let Some(filter) = decl.using_filter().filter(|_| decl.using) else {
            return Ok(None);
        };
        let key = (scope, decl.id);
        if let Some(filter) = self.ty.field_filter_cache.get(&key) {
            return Ok(Some(filter.clone()));
        }
        if !self.ty.field_filter_pending.insert(key) {
            return err(
                Code::UsingFilterInvalid,
                decl.span,
                "cycle in struct using filters",
            );
        }
        let result = self.filter_using_entry(scope, scope::UsingEntry::Type(ty), filter, decl.span);
        self.ty.field_filter_pending.remove(&key);
        let resolved = match result? {
            scope::UsingEntry::Filtered(_, filter) => (*filter).clone(),
            _ => filter.clone(),
        };
        self.ty.field_filter_cache.insert(key, resolved.clone());
        Ok(Some(resolved))
    }

    pub(in crate::sema) fn declared_promoted_field_type(
        &mut self,
        scope: ScopeId,
        name: Sym,
        span: Span,
    ) -> Result<Option<TypeId>> {
        let ScopeKind::Struct(owner_ty) = self.scope(scope).kind else {
            return Ok(None);
        };
        self.declared_promoted_field_type_inner(scope, name, span, &mut vec![(owner_ty, name)])
    }

    fn declared_promoted_field_type_inner(
        &mut self,
        scope: ScopeId,
        name: Sym,
        span: Span,
        searched: &mut Vec<(TypeId, Sym)>,
    ) -> Result<Option<TypeId>> {
        let ScopeKind::Struct(owner_ty) = self.scope(scope).kind else {
            return Ok(None);
        };
        let owner = self.ty.types.as_struct(owner_ty).unwrap();
        let candidates = self
            .ty
            .field_using_order
            .get(&scope)
            .cloned()
            .unwrap_or_else(|| {
                // Before conditional/insert expansion, only direct declarations are available.
                self.ty.struct_asts[&owner]
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
            let Some(target) = crate::sema::using::filter_target(&filter, name) else {
                continue;
            };
            if let Some(member_ty) = self.declared_member_type(ty, target, span, searched)? {
                self.ide_note_member_use(span, self.ty.types.pointee(ty).unwrap_or(ty), target);
                return Ok(Some(member_ty));
            }
        }
        Ok(None)
    }

    /// A member's declared type, without an offset or a storage-layout dependency.
    pub(super) fn declared_member_type(
        &mut self,
        ty: TypeId,
        name: Sym,
        span: Span,
        searched: &mut Vec<(TypeId, Sym)>,
    ) -> Result<Option<TypeId>> {
        let ty = self.ty.types.pointee(ty).unwrap_or(ty);
        let ty = self.ty.types.repr_struct(ty);
        if searched.contains(&(ty, name)) {
            return Ok(None);
        }
        let Some(record) = self.ty.types.as_struct(ty) else {
            return Ok(self
                .builtin_members(ty, span)?
                .into_iter()
                .find(|(member, _, _)| *member == name)
                .map(|(_, ty, _)| ty));
        };
        searched.push((ty, name));
        let scope = self.ty.struct_asts[&record].scope;
        let result = if let Some(decl) = self
            .ty
            .field_declarations
            .get(&scope)
            .and_then(|fields| fields.get(&name))
            .cloned()
        {
            self.resolve_field_decl_type(scope, &decl).map(Some)
        } else {
            self.declared_promoted_field_type_inner(scope, name, span, searched)
        };
        searched.pop();
        result
    }

    pub(in crate::sema) fn pending_struct_field_type(
        &mut self,
        ty: TypeId,
        name: Sym,
        span: Span,
    ) -> Result<Option<TypeId>> {
        let Some(s) = self.ty.types.as_struct(self.ty.types.repr_struct(ty)) else {
            return Ok(None);
        };
        if self.ty.types.struct_info(s).layout != LayoutState::InProgress {
            return Ok(None);
        }
        let scope = self.ty.struct_asts[&s].scope;
        let Some(decl) = self
            .ty
            .field_declarations
            .get(&scope)
            .and_then(|fields| fields.get(&name))
            .cloned()
        else {
            return self.declared_promoted_field_type(scope, name, span);
        };
        let ty = self.resolve_field_decl_type(scope, &decl)?;
        let declared = decl
            .names
            .iter()
            .find(|n| n.name == name)
            .map_or(decl.span, |n| n.span);
        self.ide_note_named_field(span, name, declared, ty);
        Ok(Some(ty))
    }

    /// The value of the struct constant `ids` that `name` at `span` named. The use is noted after
    /// the value is checked: an initializer that names something (`count :: N;`) would otherwise
    /// take the name's place as the last one resolved.
    fn constant_operand(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        ids: &[EntityId],
        span: Span,
    ) -> Result<Operand> {
        let operand = self.entities_operand(f, scope, ids, span);
        if operand.is_ok()
            && let Some(&first) = ids.first()
        {
            self.ed.resolved(span, first);
        }
        operand
    }

    pub(super) fn type_member(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        t: TypeId,
        name: Sym,
        span: Span,
    ) -> Result<Operand> {
        // `array.type` where `array: *Struct(...)` is bound as a type (implicit polymorphs).
        if let Some(p) = self.ty.types.pointee(t)
            && self.ty.types.as_struct(p).is_some()
            && let Some(ids) = self.struct_constant(p, name)?
        {
            return self.constant_operand(f, scope, &ids, span);
        }
        match self.ty.types.kind(t).clone() {
            TypeKind::Enum(_) if name.as_str() == "loose" => {
                self.warn_lenient_loose_enum(scope, span, t);
                Ok(Operand::Type(self.ty.types.loose_enum(t)))
            }
            TypeKind::Enum(e) => {
                if let Some(&member_scope) = self.ty.enum_member_scopes.get(&t)
                    && let Some(ids) = self.scope(member_scope).names.get(&name).cloned()
                    && ids
                        .iter()
                        .any(|&id| matches!(self.entity(id).kind, EntityKind::EnumMember { .. }))
                {
                    return self.entities_operand(f, member_scope, &ids, span);
                }
                if let Some(&(_, v)) = self
                    .ty
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
                        origin: Origin::Computed,
                    });
                }
                if let Some(&(_, borrowed)) = self
                    .ty
                    .enum_borrowed
                    .get(&t)
                    .and_then(|names| names.iter().find(|(n, _)| *n == name))
                {
                    let Resolved::Const {
                        value: Value::Int(v),
                        ..
                    } = self.resolve_entity(borrowed)?
                    else {
                        unreachable!()
                    };
                    return Ok(Operand::Const {
                        ty: t,
                        value: Value::Int(v),
                        untyped: false,
                        origin: Origin::Computed,
                    });
                }
                Err(Box::new(self.no_member(t, name, span)))
            }
            TypeKind::Struct(_) => {
                if let Some(ids) = self.struct_constant(t, name)? {
                    return self.constant_operand(f, scope, &ids, span);
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
                    Code::StructTypeNoConstant,
                    span,
                    format!(
                        "type {} has no constant member `{name}`",
                        self.ty.types.name(t)
                    ),
                )
            }
            TypeKind::Array {
                kind: ArrayKind::Fixed(n),
                ..
            } if name.as_str() == "count" => Ok(Operand::untyped_int(n as i128)),
            TypeKind::Distinct(d) => {
                let base = self.ty.types.distincts[d.0 as usize].base;
                self.type_member(f, scope, base, name, span)
            }
            _ => Err(Box::new(self.no_member(t, name, span))),
        }
    }
}
