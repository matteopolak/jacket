//! Member access expressions and the types of promoted and pending members.
use super::model::FieldUsing;
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::scope::{EntityId, EntityKind, Found, ScopeKind};
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
                            && let Some(ide) = self.ed.facts_mut()
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
                    self.ide_note_constant_use(span, &ids);
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
        if let Some((path, mty, origin)) =
            self.find_member_in(target, name, span, &mut Vec::new())?
        {
            let a = self.apply_path(f, addr, &path);
            let parameters = origin
                .and_then(|origin| self.callable_fields.get(&origin))
                .cloned();
            return Ok(Operand::Place {
                ty: mty,
                addr: a,
            }
            .with_callable(parameters));
        }
        if let Some(ids) = self.struct_constant(target, name)? {
            self.ide_note_constant_use(span, &ids);
            return self.entities_operand(f, scope, &ids, span);
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

    pub(in crate::sema) fn declared_promoted_field_type(
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
            let Some(target) = crate::sema::using::filter_target(&filter, name) else {
                continue;
            };
            if let Some((_, member_ty, _)) =
                self.find_promoted_member(vec![(ty, 0, target)], span, &mut vec![owner])?
            {
                self.ide_note_member_use(span, self.types.pointee(ty).unwrap_or(ty), target);
                return Ok(Some(member_ty));
            }
        }
        Ok(None)
    }

    pub(in crate::sema) fn pending_struct_field_type(
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
        let declared = decl
            .names
            .iter()
            .find(|n| n.name == name)
            .map_or(decl.span, |n| n.span);
        self.ide_note_named_field(span, name, declared, ty);
        Ok(Some(ty))
    }

    /// `name` at `span` named the constant `ids` declare in a struct: a use of it, like any name.
    fn ide_note_constant_use(&mut self, span: Span, ids: &[EntityId]) {
        if let Some(ide) = self.ed.facts_mut()
            && let Some(&first) = ids.first()
        {
            ide.last_entity = Some((span, first));
        }
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
        if let Some(p) = self.types.pointee(t)
            && self.types.as_struct(p).is_some()
            && let Some(ids) = self.struct_constant(p, name)?
        {
            self.ide_note_constant_use(span, &ids);
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
                    self.ide_note_constant_use(span, &ids);
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
}
