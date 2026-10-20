//! Identifiers, entities and `#this`: resolving a name to an operand.
use crate::ast::ExprKind as E;
use crate::ir::Ty;
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::scope::{EntityKind, Found, Resolved, ScopeKind};
use crate::sema::{
    Compiler, Diagnostic, DiagnosticKind, EntityId, PreludeType, Rc, Result, ScopeId, Span, Sym,
    TypeId, Value, ast, err, procs, scope,
};
use crate::types::{ArrayKind, ProcType, TypeKind};

impl Compiler {
    /// `#jaic_type name`: a type only jaic has, for the `Extensions/Long_Double` module to export
    /// (`docs/language/long-double.md`). `long_double` is the target C compiler's
    /// `long double`: `float64` where the two are the same, else the wide `Long_Double` type.
    pub(super) fn jaic_type(&mut self, operand: Option<&ast::Expr>, span: Span) -> Result<TypeId> {
        let name = match operand.map(|e| &e.kind) {
            Some(E::Ident(name)) => name.as_str(),
            _ => {
                return err(
                    span,
                    "#jaic_type expects a name, as in '#jaic_type long_double'",
                );
            }
        };
        match name {
            "long_double" => Ok(match self.options.long_double {
                Some(fmt) => self.types.intern(TypeKind::WideFloat(fmt)),
                None => TypeId::F64,
            }),
            other => err(span, format!("unknown jaic extension type `{other}`")),
        }
    }

    /// `ifx c then a else { stmts; value }`: a block's value is its last expression.
    pub(super) fn check_block_value(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        block: &ast::Block,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        let Some((
            ast::Stmt {
                kind: ast::StmtKind::Expr(last),
                ..
            },
            rest,
        )) = block.stmts.split_last()
        else {
            return err(span, "a block used as a value must end in an expression");
        };
        let inner = self.new_block_scope(scope);
        self.ide_scope_span(inner, span);
        let depth = f.mark_defers();
        self.check_block_stmts(f, inner, rest)?;
        let mut op = self.check_expr(f, inner, last, expected)?;
        if f.defers.len() > depth.position() {
            // Deferred code runs after the value is computed.
            if !matches!(op, Operand::Void) {
                let op_settled = self.settle_untyped(op, expected);
                let ty = op_settled.ty();
                let (_, v) = self.rvalue(f, op_settled, span)?;
                op = Operand::Value {
                    ty,
                    val: v,
                };
            }
            self.emit_defers(f, depth.position(), span)?;
        }
        f.truncate_defers(depth.position());
        Ok(op)
    }

    /// The type of `expr` if it names a runtime local of an enclosing procedure.
    pub(in crate::sema) fn outer_local_type(
        &mut self,
        scope: ScopeId,
        expr: &ast::Expr,
    ) -> Result<Option<TypeId>> {
        let E::Ident(name) = &expr.kind else {
            return Ok(None);
        };
        let found = match self.lookup(scope, *name)?.as_slice() {
            [id] => match self.entity(*id).kind {
                EntityKind::Local {
                    ty,
                    depth,
                    ..
                } if depth != self.scope(scope).proc_depth => Some((*id, ty)),
                _ => None,
            },
            _ => None,
        };
        if let Some((id, ty)) = found {
            // The name is a use of the local, as it is when `type_of` checks it as an expression.
            self.ide_note_entity_use(expr.span, id, ty);
        }
        Ok(found.map(|(_, ty)| ty))
    }

    /// Evaluate a type expression inside a body.
    pub fn eval_type_in(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        expr: &ast::Expr,
    ) -> Result<TypeId> {
        let op = self.check_expr(f, scope, expr, Some(TypeId::TYPE))?;
        if let Operand::Value {
            ty: TypeId::TYPE, ..
        }
        | Operand::Place {
            ty: TypeId::TYPE, ..
        } = op
        {
            return err(expr.span, "type must be known at compile time");
        }
        self.operand_as_type(op, expr.span)
    }

    pub(in crate::sema) fn check_ident(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        name: Sym,
        span: Span,
    ) -> Result<Operand> {
        if f.type_only
            && let Some(ty) = self.struct_field_type(scope, name, span)?
        {
            let addr = f.b.iconst(Ty::Ptr, 0);
            return Ok(Operand::Place {
                ty,
                addr,
            });
        }
        match self.lookup_full(scope, name)? {
            Found::Using(entry, member) => {
                let place = match &entry {
                    scope::UsingEntry::Place {
                        entity,
                        ty,
                    } => {
                        self.ide_note_use(*entity);
                        Some(*ty)
                    }
                    _ => None,
                };
                let op = self.using_member(f, entry, member, span)?;
                // `name` stands for `value.name`: hover and go to definition treat it as the field.
                if let Some(ty) = place {
                    self.ide_note_member_use(span, ty, member);
                    if let Some(ide) = self.ed.facts_mut() {
                        ide.last_using_member = Some(member);
                    }
                }
                Ok(op)
            }
            Found::Entities(ids) => {
                if ids.is_empty() {
                    if f.type_only
                        && let Some(ty) = self.struct_field_type(scope, name, span)?
                    {
                        // Only the type is wanted, so the address is never used.
                        let addr = f.b.iconst(Ty::Ptr, 0);
                        return Ok(Operand::Place {
                            ty,
                            addr,
                        });
                    }
                    return Err(Box::new(
                        Diagnostic::error(span, format!("unknown identifier `{name}`")).with_kind(
                            DiagnosticKind::UnknownIdentifier {
                                scope: Some(scope.0),
                            },
                        ),
                    ));
                }
                let op = self.entities_operand(f, scope, &ids, span);
                // A name whose declaration failed (a `#run` that asserted) still names it, so it
                // is colored and navigable as it is when the declaration succeeds.
                if op.is_err()
                    && let [id] = ids[..]
                {
                    self.ide_note_entity_use(span, id, TypeId::COMPILE_TIME);
                }
                if let Some(ide) = self.ed.facts_mut() {
                    ide.last_entity = Some((span, ids[0]));
                    if ide.lint {
                        ide.used.extend(ids.iter().copied());
                    }
                }
                op
            }
        }
    }

    /// Resolve a lexical field type without requesting the containing struct's offsets.
    pub(in crate::sema) fn struct_field_type(
        &mut self,
        scope: ScopeId,
        name: Sym,
        use_span: Span,
    ) -> Result<Option<TypeId>> {
        let mut s = Some(scope);
        while let Some(sid) = s {
            if self.scope(sid).names.contains_key(&name) {
                return Ok(None);
            }
            if let Some(decl) = self
                .field_declarations
                .get(&sid)
                .and_then(|fields| fields.get(&name))
                .cloned()
            {
                let ty = self.resolve_field_decl_type(sid, &decl)?;
                let declared = decl.names.iter().find(|n| n.name == name).unwrap().span;
                self.ide_note_named_field(use_span, name, declared, ty);
                return Ok(Some(ty));
            }
            if let Some(ty) = self.declared_promoted_field_type(sid, name, use_span)? {
                return Ok(Some(ty));
            }
            s = self.scope(sid).parent;
        }
        Ok(None)
    }

    pub fn entities_operand(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        ids: &[EntityId],
        span: Span,
    ) -> Result<Operand> {
        // A macro's backtick declaration reusing a parameter's name shadows the parameter: the
        // newest local is the one the rest of the body means.
        if ids.len() > 1
            && ids
                .iter()
                .all(|&id| matches!(self.entity(id).kind, EntityKind::Local { .. }))
        {
            return self.entity_operand(f, scope, ids[ids.len() - 1], span);
        }
        if ids.len() > 1 || self.entity_is_overloadable(ids[0]) {
            let mut procs = Vec::new();
            for &id in ids {
                match self.resolve_entity(id)? {
                    Resolved::Proc(p) => procs.push(p),
                    Resolved::Const {
                        value: Value::Proc(p),
                        ..
                    } => procs.push(p),
                    Resolved::ProcSet(set) => procs.extend(set),
                    _ => {
                        if ids.len() == 1 {
                            return self.entity_operand(f, scope, id, span);
                        }
                        return err(
                            span,
                            format!("`{}` is declared more than once", self.entity(id).name),
                        );
                    }
                }
            }
            if ids.len() > 1 {
                self.check_identical_overloads(ids)?;
            }
            return Ok(Operand::Procs(procs));
        }
        self.entity_operand(f, scope, ids[0], span)
    }

    fn entity_operand(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        id: EntityId,
        span: Span,
    ) -> Result<Operand> {
        if self.discard_params.contains(&id) {
            return err(
                span,
                format!(
                    "`{}` is a #discard parameter and cannot be used in the procedure",
                    self.entity(id).name
                ),
            );
        }
        match self.entity(id).kind.clone() {
            EntityKind::Local {
                ty,
                addr,
                depth,
            } => {
                if let Some((value, ty)) = self.const_macro_params.get(&id).cloned() {
                    if let Value::Type(t) = value {
                        return Ok(Operand::Type(t));
                    }
                    return Ok(Operand::Const {
                        ty,
                        value,
                        untyped: false,
                    }
                    .with_callable(self.callable_entities.get(&id).cloned()));
                }
                if depth != self.scope(scope).proc_depth {
                    if let Some((value, ty)) = self.local_consts.get(&id).cloned() {
                        return Ok(Operand::Const {
                            ty,
                            value,
                            untyped: false,
                        }
                        .with_callable(self.callable_entities.get(&id).cloned()));
                    }
                    if f.type_only {
                        // `type_of(local.*)` at compile time: the address is never used.
                        let addr = f.b.iconst(Ty::Ptr, 0);
                        return Ok(Operand::Place {
                            ty,
                            addr,
                        });
                    }
                    let name = self.entity(id).name;
                    return err(
                        span,
                        if f.compile_time {
                            format!(
                                "cannot use local `{name}` in a compile-time expression: its value is only known at runtime"
                            )
                        } else {
                            format!("cannot access local `{name}` of an enclosing procedure")
                        },
                    );
                }
                return Ok(Operand::Place {
                    ty,
                    addr,
                }
                .with_callable(self.callable_entities.get(&id).cloned()));
            }
            EntityKind::Builtin(scope::Builtin::Proc(p)) => return Ok(Operand::Builtin(p)),
            _ => {}
        }
        let resolved = self.resolve_entity(id)?;
        self.use_runtime_storage_entity(f, id);
        Ok(match resolved {
            Resolved::Const {
                value: Value::Type(t),
                ..
            } => Operand::Type(t),
            Resolved::Const {
                value: Value::Proc(p),
                ..
            } => Operand::Procs(vec![p]),
            Resolved::Const {
                value,
                ty,
            } => Operand::Const {
                ty,
                value,
                untyped: self.entity(id).untyped_const,
            },
            Resolved::Proc(p) => Operand::Procs(vec![p]),
            Resolved::ProcSet(set) => Operand::Procs(set),
            Resolved::Global {
                storage,
                ty,
            } => {
                if let Some(what) = f.constant_only
                    && !f.type_only
                {
                    let name = self.entity(id).name;
                    return err(span, format!("{what}, but `{name}` is a variable"));
                }
                let addr = f.b.storage_addr(storage);
                let parameters = if matches!(self.types.kind(ty), TypeKind::Proc(_)) {
                    self.callable_entity_parameters(id)?
                } else {
                    None
                };
                Operand::Place {
                    ty,
                    addr,
                }
                .with_callable(parameters)
            }
            Resolved::Module(m) => Operand::Module(m),
            Resolved::Library(l) => Operand::Library(l),
            Resolved::PolyStruct(p) => Operand::PolyStruct(p),
        })
    }

    pub(super) fn check_inferred_member(
        &mut self,
        f: &mut FnCtx,
        _scope: ScopeId,
        name: &ast::Ident,
        expected: Option<TypeId>,
    ) -> Result<Operand> {
        let Some(t) = expected else {
            return err(
                name.span,
                format!("cannot infer the type of `.{}` here", name.name),
            );
        };
        if self.ed.is_live()
            && self.ide_wants_pub(name.span.file)
            && let Some(ide) = self.ed.facts_mut()
        {
            let seen = ide.inferred_expected.entry(name.span).or_default();
            if !seen.contains(&t) {
                seen.push(t);
            }
        }
        // Allow pointer-to-enum targets? No: enums only, but look through distinct.
        match self.types.kind(t).clone() {
            TypeKind::Enum(_) | TypeKind::Struct(_) => {
                let result =
                    self.member_access(f, ScopeId(0), Operand::Type(t), name.name, name.span);
                if result.is_ok() {
                    self.ide_note_member_use(name.span, t, name.name);
                }
                result
            }
            _ => err(
                name.span,
                format!(
                    "cannot infer `.{}` for type {}",
                    name.name,
                    self.types.name(t)
                ),
            ),
        }
    }

    /// Is `name` a field (not a constant) of struct type `ty`? Reading one needs a value.
    pub(super) fn names_field(&mut self, ty: TypeId, name: Sym) -> Result<bool> {
        Ok(self.types.as_struct(ty).is_some()
            && self.struct_constant(ty, name)?.is_none()
            && self.find_member(ty, name, Span::NONE)?.is_some())
    }

    /// `local.CONSTANT` where `local` belongs to an enclosing procedure: compile-time code
    /// (an `#insert` procedure) may read a constant member of the local's struct type,
    /// since that needs the type but not the value.
    pub(super) fn enclosing_local_constant(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        base: &ast::Expr,
        member: &ast::Ident,
    ) -> Result<Option<Operand>> {
        let E::Ident(name) = &base.kind else {
            return Ok(None);
        };
        let Ok(ids) = self.lookup(scope, *name) else {
            return Ok(None);
        };
        let [id] = ids[..] else {
            return Ok(None);
        };
        let EntityKind::Local {
            ty,
            depth,
            ..
        } = self.entity(id).kind.clone()
        else {
            return Ok(None);
        };
        if depth == self.scope(scope).proc_depth {
            return Ok(None);
        }
        let ty = self.types.pointee(ty).unwrap_or(ty);
        if self.types.as_struct(ty).is_none() || self.names_field(ty, member.name)? {
            return Ok(None);
        }
        Ok(self
            .member_access(f, scope, Operand::Type(ty), member.name, member.span)
            .ok()
            .filter(|op| {
                matches!(
                    op,
                    Operand::Const { .. } | Operand::Type(_) | Operand::Procs(_)
                )
            }))
    }

    /// `#this`: the innermost enclosing struct type or, inside a procedure body, the procedure.
    pub(super) fn check_this(&mut self, scope: ScopeId, span: Span) -> Result<Operand> {
        let mut s = Some(scope);
        while let Some(sid) = s {
            match self.scope(sid).kind {
                ScopeKind::Struct(t) => return Ok(Operand::Type(t)),
                ScopeKind::Proc => {
                    if let Some(p) = self.scope(sid).proc {
                        return Ok(Operand::Procs(vec![p]));
                    }
                }
                _ => {}
            }
            s = self.scope(sid).parent;
        }
        err(span, "#this used outside of a struct or procedure")
    }

    pub(super) fn proc_type_from_header(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        header: &ast::ProcHeader,
    ) -> Result<TypeId> {
        procs::validate_c_variadic_header(header)?;
        // `#cpp_method` procedures use the C calling convention with the object as first argument.
        let c_call = header.uses_c_abi();
        let mut params = Vec::new();
        let mut variadic = false;
        let mut c_varargs = false;
        for p in &header.params {
            // `(s: string, start := 0) -> s64`: a parameter may take its default's type.
            let ty = match (&p.ty, &p.default) {
                (Some(t), _) => self.eval_type_in(f, scope, t)?,
                (None, Some(d)) => {
                    let op = self.check_expr_no_emit(scope, d)?;
                    self.settle_untyped(op, None).ty()
                }
                (None, None) => return err(p.span, "procedure type parameter needs a type"),
            };
            if let Some(name) = &p.name
                && self.ed.is_live()
            {
                self.ide_note_param_name(name.span, name.name, ty);
            }
            if p.variadic {
                if c_call {
                    c_varargs = true;
                    continue;
                }
                variadic = true;
                params.push(self.types.array(ty, ArrayKind::View));
            } else {
                params.push(ty);
            }
        }
        let mut returns = Vec::new();
        for r in &header.returns {
            let Some(rt) = &r.ty else {
                return err(r.span, "procedure type result needs a type");
            };
            let t = self.eval_type_in(f, scope, rt)?;
            if let Some(name) = r.name {
                self.ide_note_param_name(name.span, name.name, t);
            }
            if t != TypeId::VOID {
                returns.push(t);
            }
        }
        let ty = self.types.intern(TypeKind::Proc(Rc::new(ProcType {
            params,
            returns,
            variadic,
            c_varargs,
            c_call,
            no_context: c_call || header.flags.no_context(),
            non_pod_return: header.flags.cpp_return_type_is_non_pod(),
        })));
        Ok(ty)
    }

    /// `#location` / `#caller_location` as a `Source_Code_Location` constant.
    pub fn location_operand(&mut self, f: &mut FnCtx, span: Span) -> Result<Operand> {
        let ty = self.preload_type(PreludeType::SourceCodeLocation, span)?;
        let file = self.sources.get(span.file);
        let (line, col) = file.line_col(span.start);
        let path = file.path.to_string();
        let _ = f;
        let value = self.build_location_value(ty, &path, line as i64, col as i64, span)?;
        Ok(Operand::Const {
            ty,
            value,
            untyped: false,
        })
    }

    pub fn preload_type(&mut self, ty: PreludeType, span: Span) -> Result<TypeId> {
        let name = ty.name();
        let Some(preload) = self.preload else {
            return err(span, format!("`{name}` requires Preload"));
        };
        let ids = self.module_declarations(preload, Sym::intern(name))?;
        let Some(&id) = ids.first() else {
            return err(span, format!("Preload does not define `{name}`"));
        };
        match self.resolve_entity(id)? {
            Resolved::Const {
                value: Value::Type(t),
                ..
            } => Ok(t),
            _ => err(span, format!("Preload's `{name}` is not a type")),
        }
    }
}
