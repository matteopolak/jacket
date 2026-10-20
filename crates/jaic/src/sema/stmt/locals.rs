//! Local declarations and `using` statements.
use crate::ast::ExprKind as E;
use crate::sema::TypeState;
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::scope::{EntityKind, Resolved, UsingEntry};
use crate::sema::{
    Compiler, Diagnostic, DiagnosticKind, ProcId, Rc, Result, ScopeId, Span, Sym, TypeId, ast, err,
};
use crate::types::TypeKind;

impl Compiler {
    pub(super) fn check_local_decl(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        decl: &Rc<ast::Decl>,
    ) -> Result<()> {
        let target = if decl.backtick {
            f.macros.last().map_or(scope, |m| m.caller_scope)
        } else {
            scope
        };
        if decl.kind == ast::DeclKind::Const {
            if !f.hoisted_consts.contains(&(scope, decl.id)) {
                self.declare_local_consts(target, scope, decl)?;
            }
            return Ok(());
        }
        let span = decl.span;
        let declared = match &decl.ty {
            Some(t) => Some(self.eval_type_in(f, scope, t)?),
            None => None,
        };
        // Evaluate the initializers before the names come into scope.
        let mut values = self.check_decl_values(f, scope, decl, declared)?;
        values.resize(decl.names.len(), None);
        let existing = |i: usize| decl.existing().get(i).copied().unwrap_or(false);
        for (i, (name, value)) in decl.names.iter().zip(values).enumerate() {
            let target = if decl.backtick_names().get(i).copied().unwrap_or(false) {
                f.macros.last().map_or(scope, |m| m.caller_scope)
            } else {
                target
            };
            if existing(i) {
                // `a=, b := ...` assigns to a variable that is already in scope.
                let Some(value) = value else {
                    return err(name.span, "assignment needs a value");
                };
                // `ok:, t.str = f()` assigns to a member, index or dereference.
                let place = if let Some(Some(e)) = decl.targets().get(i) {
                    self.reject_constant_target(f, scope, e)?;
                    self.check_expr(f, scope, e, None)?
                } else {
                    let place = self.check_ident(f, scope, name.name, name.span)?;
                    if self.ed.is_live() {
                        let expr = ast::Expr {
                            kind: ast::ExprKind::Ident(name.name),
                            span: name.span,
                        };
                        self.ide_note_expr(&expr, &place);
                    }
                    place
                };
                let place = place.without_callable();
                let Operand::Place {
                    ty,
                    addr,
                } = place
                else {
                    return err(
                        name.span,
                        format!("cannot assign to {}", self.describe(&place)),
                    );
                };
                let value = self.convert(f, value, ty, name.span)?;
                let (_, v) = self.rvalue(f, value, name.span)?;
                self.store_value(f, ty, addr, v, name.span)?;
                continue;
            }
            let ty = match (declared, &value) {
                (Some(t), _) => t,
                (None, Some(op)) => {
                    let settled = self.settle_untyped(op.clone(), None);
                    match settled {
                        Operand::Procs(p) if p.len() == 1 => self.proc_type(p[0], span)?,
                        Operand::Type(_) | Operand::PolyStruct(_) => TypeId::TYPE,
                        Operand::Void => {
                            let value = decl.value.as_ref().map_or(span, |v| v.span);
                            let text = self.src.sources.snippet(value).trim();
                            let shown = if text.is_empty() || text.contains('\n') || text.len() > 60
                            {
                                "this expression".to_string()
                            } else {
                                format!("`{text}`")
                            };
                            return Err(Box::new(
                                Diagnostic::error(
                                    value,
                                    format!(
                                        "cannot declare `{}` from {shown}, which has no value",
                                        name.name
                                    ),
                                )
                                .with_label("this gives no value")
                                .with_help(
                                    "a procedure without a return type returns nothing: call it on its own line, or give the variable a value of its own",
                                ),
                            ));
                        }
                        // `p := null;` declares a `*void`.
                        ref o if o.ty() == TypeId::NULL => self.ty.types.pointer(TypeId::VOID),
                        other => other.ty(),
                    }
                }
                (None, None) => return err(span, "declaration needs a type or a value"),
            };
            self.ensure_storage_type(ty, span)?;
            // Metaprograms read local declaration types from the exported syntax tree.
            self.pr.local_decl_types.entry((decl.id, i)).or_insert(ty);
            let size = self.size_of(ty, span)?;
            let mut align = self.align_of(ty, span)?;
            if let Some(a) = &decl.align() {
                align = align.max(self.eval_int(scope, a)? as u64);
            }
            let addr = f.b.alloca(size.max(1), align);
            let callable = if matches!(self.ty.types.kind(ty), TypeKind::Proc(_)) {
                match &decl.ty {
                    Some(source) => self.callable_expression_parameters(scope, Some(source))?,
                    None => match &value {
                        Some(op) => self.callable_operand_parameters(op)?,
                        None => None,
                    },
                }
            } else {
                None
            };
            match value {
                Some(op) => {
                    let value_span = match &decl.value {
                        Some(v) if decl.names.len() == 1 => v.span,
                        _ => span,
                    };
                    let op = self
                        .convert(f, op, ty, value_span)
                        .map_err(|e| self.declared_type_mismatch(e, decl, decl.names.len() == 1))?;
                    let (_, v) = self.rvalue(f, op, span)?;
                    self.store_value(f, ty, addr, v, span)?;
                }
                None if decl.value.is_none() => self.init_default(f, ty, addr, span)?,
                None => {}
            }
            // `_` discards a value: it names no local (but `using _ := *x;` still uses).
            let discard = name.name.as_str() == "_";
            if discard && !decl.using {
                continue;
            }
            if !discard && self.declares_local(target, name.name) {
                let mut d = Diagnostic::error(
                    name.span,
                    format!("`{}` is already declared in this scope", name.name),
                )
                .with_label("declared again here");
                if let Some(first) = self.local_declaration(target, name.name) {
                    d = d.with_note(first, format!("`{}` is first declared here", name.name));
                }
                let help = if decl.value.is_some() {
                    format!(
                        "to change its value, assign with `{} = ...`; to make a new variable, use another name",
                        name.name
                    )
                } else {
                    "use another name for the new variable".to_string()
                };
                return Err(Box::new(d.with_help(help)));
            }
            let depth = self.scope(target).proc_depth;
            let kind = EntityKind::Local {
                ty,
                addr,
                depth,
            };
            let e = self.add_entity(target, name.name, name.span, kind.clone(), false);
            if let Some(info) = callable {
                self.pr.callable_entities.insert(e, info);
            }
            self.debug_var(f, target, name.name, name.span, (ty, addr), 0);
            if target != scope {
                // A backtick name is visible to the rest of the macro body too.
                let alias = self.add_entity(scope, name.name, name.span, kind, false);
                if let Some(info) = self.pr.callable_entities.get(&e).cloned() {
                    self.pr.callable_entities.insert(alias, info);
                }
            }
            if decl.using {
                let entry = self.filter_using_entry(
                    scope,
                    UsingEntry::Place {
                        ty,
                        entity: e,
                    },
                    decl.using_filter().unwrap_or(&ast::UsingFilter::None),
                    name.span,
                )?;
                self.push_using(target, entry);
            }
        }
        Ok(())
    }

    /// Where the local `name` in `scope` was declared.
    fn local_declaration(&self, scope: ScopeId, name: Sym) -> Option<Span> {
        let ids = self.scope(scope).names.get(&name)?;
        ids.iter()
            .find(|&&e| matches!(self.entity(e).kind, EntityKind::Local { .. }))
            .map(|&e| self.entity(e).span)
    }

    /// A returned value that does not fit the procedure's result type: a note points at the
    /// type it had to match in the procedure's header.
    #[expect(
        clippy::unnecessary_box_returns,
        reason = "the diagnostic is boxed because the error type carries it boxed; callers pass it through `map_err`"
    )]
    pub(super) fn return_type_mismatch(
        &self,
        mut e: Box<Diagnostic>,
        proc: Option<ProcId>,
        index: usize,
    ) -> Box<Diagnostic> {
        if e.kind != DiagnosticKind::TypeMismatch {
            return e;
        }
        let Some(p) = proc else {
            return e;
        };
        let header = &self.proc(p).lit.header;
        if let Some(ty) = header.returns.get(index).and_then(|r| r.ty.as_ref()) {
            e.notes.insert(
                0,
                (ty.span, "expected because of this return type".to_string()),
            );
        }
        e
    }

    /// A value that does not fit the declared type of `x: T = value;`: the error points at
    /// the value, with the type it had to match as a note.
    #[expect(
        clippy::unnecessary_box_returns,
        reason = "the diagnostic is boxed because the error type carries it boxed; callers pass it through `map_err`"
    )]
    fn declared_type_mismatch(
        &self,
        mut e: Box<Diagnostic>,
        decl: &ast::Decl,
        single: bool,
    ) -> Box<Diagnostic> {
        if !single || e.kind != DiagnosticKind::TypeMismatch {
            return e;
        }
        if let (Some(value), Some(ty)) = (&decl.value, &decl.ty) {
            e.span = value.span;
            e.notes
                .insert(0, (ty.span, "expected because of this type".to_string()));
        }
        e
    }

    /// Whether `scope` itself already holds a runtime local called `name`.
    pub(crate) fn declares_local(&self, scope: ScopeId, name: Sym) -> bool {
        self.scope(scope).names.get(&name).is_some_and(|ids| {
            ids.iter()
                .any(|&e| matches!(self.entity(e).kind, EntityKind::Local { .. }))
        })
    }

    /// The initializer operands of a declaration, one per name where the declaration gives
    /// enough values: `a, b := f()` splits a multi-value call, `a, b := 0` repeats the value,
    /// and `a, b := 1, 2` pairs them up.
    fn check_decl_values(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        decl: &ast::Decl,
        declared: Option<TypeId>,
    ) -> Result<Vec<Option<Operand>>> {
        let names = decl.names.len();
        let Some(first) = &decl.value else {
            return Ok(vec![None; names]);
        };
        if matches!(first.kind, E::Uninit) {
            return Ok(vec![None; names]);
        }
        if !decl.extra_values().is_empty() {
            let exprs: Vec<&ast::Expr> =
                std::iter::once(first).chain(decl.extra_values()).collect();
            if exprs.len() != names {
                return err(
                    decl.span,
                    format!("expected {names} values, found {}", exprs.len()),
                );
            }
            let mut values = Vec::new();
            for e in exprs {
                let op = self.check_expr(f, scope, e, declared)?;
                values.push(Some(if decl.existing().is_empty() {
                    op
                } else {
                    // Snapshot, so assigning to an earlier name cannot change a later value.
                    let op = self.settle_untyped(op, declared);
                    let (ty, v) = self.rvalue(f, op, e.span)?;
                    let tmp = self.spill(f, ty, v, e.span)?;
                    let val = match self.ir_ty(ty) {
                        Some(t) => f.b.load(t, tmp),
                        None => tmp,
                    };
                    Operand::Value {
                        ty,
                        val,
                    }
                }));
            }
            return Ok(values);
        }
        // `x, y := #run f();` keeps each of f's results, as the constant form does.
        if names > 1
            && let E::Run {
                body, ..
            } = &first.kind
            && matches!(&**body, ast::RunBody::Expr(_))
        {
            let ops = self.eval_const_all(scope, first)?;
            if ops.len() < names {
                return err(
                    first.span,
                    format!("{names} names but {} values", ops.len()),
                );
            }
            return Ok(ops.into_iter().take(names).map(Some).collect());
        }
        self.pr.last_call_must = None;
        let first_op = self.check_expr(f, scope, first, declared)?;
        if matches!(first.kind, E::Call { .. }) {
            self.pr.check_must_used(first.span, names)?;
            // `a, b := f(1);` with one result: the call has too few values to split.
            let count = match &first_op {
                Operand::Multi(vals) => vals.len(),
                Operand::Void => 0,
                _ => 1,
            };
            if names > 1 && count < names {
                return err(
                    first.span,
                    format!(
                        "{names} names but the call returns {count} value{}",
                        if count == 1 {
                            ""
                        } else {
                            "s"
                        }
                    ),
                );
            }
        }
        Ok(match first_op {
            Operand::Multi(vals) if names > 1 => vals.into_iter().map(Some).collect(),
            // `a, b := 0;` gives every name the same value.
            op => vec![Some(op); names],
        })
    }

    pub(super) fn check_using(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        value: &ast::Expr,
        filter: &ast::UsingFilter,
    ) -> Result<()> {
        if let E::Ident(name) = &value.kind {
            let ids = self.lookup(scope, *name)?;
            if let Some(&id) = ids.first() {
                match self.entity(id).kind.clone() {
                    EntityKind::Local {
                        ty, ..
                    } => {
                        self.ty.check_using_target(ty, value.span)?;
                        let entry = self.filter_using_entry(
                            scope,
                            UsingEntry::Place {
                                ty,
                                entity: id,
                            },
                            filter,
                            value.span,
                        )?;
                        self.push_using(scope, entry);
                        self.ide_note_entity_use(value.span, id, ty);
                        return Ok(());
                    }
                    _ => {
                        if let Resolved::Global {
                            ty, ..
                        } = self.resolve_entity(id)?
                        {
                            let entry = self.filter_using_entry(
                                scope,
                                UsingEntry::Place {
                                    ty,
                                    entity: id,
                                },
                                filter,
                                value.span,
                            )?;
                            self.push_using(scope, entry);
                            self.ide_note_entity_use(value.span, id, ty);
                            return Ok(());
                        }
                    }
                }
            }
        }
        let op = self.check_expr(f, scope, value, None)?;
        let entry = match op {
            Operand::Module(m) => UsingEntry::Module(m, None),
            Operand::Type(t) => UsingEntry::Type(t),
            // `using fruit.tag;` on an enum value brings in the enum's names (match-jai).
            op if matches!(self.ty.types.kind(op.ty()), TypeKind::Enum(_)) => {
                UsingEntry::Type(op.ty())
            }
            op @ (Operand::Place {
                ..
            }
            | Operand::Value {
                ..
            }) => {
                // `using a.b;`: bind a hidden local pointing at the place (or holding the
                // pointer itself when `a.b` is one).
                self.ty.check_using_target(op.ty(), value.span)?;
                let (ptr, ptr_value) = if self.ty.types.pointee(op.ty()).is_some() {
                    let ty = op.ty();
                    let (_, v) = self.rvalue(f, op, value.span)?;
                    (ty, v)
                } else {
                    let ty = self.ty.types.pointer(op.ty());
                    let (_, a) = self.address_of(f, op, value.span)?;
                    (ty, a)
                };
                let slot = self.spill(f, ptr, ptr_value, value.span)?;
                let depth = self.scope(scope).proc_depth;
                let e = self.add_entity(
                    scope,
                    Sym::intern("\u{0}using"),
                    value.span,
                    EntityKind::Local {
                        ty: ptr,
                        addr: slot,
                        depth,
                    },
                    false,
                );
                UsingEntry::Place {
                    ty: ptr,
                    entity: e,
                }
            }
            Operand::Const {
                ty, ..
            } if self.ty.types.as_struct(ty).is_some() => {
                // `using Ice_Cream.{...};`: the literal lives in an anonymous local.
                let (ty, v) = self.rvalue(f, op, value.span)?;
                let size = self.size_of(ty, value.span)?;
                let align = self.align_of(ty, value.span)?;
                let addr = f.b.alloca(size.max(1), align);
                self.store_value(f, ty, addr, v, value.span)?;
                let ptr = self.ty.types.pointer(ty);
                let slot = self.spill(f, ptr, addr, value.span)?;
                let depth = self.scope(scope).proc_depth;
                let e = self.add_entity(
                    scope,
                    Sym::intern("\u{0}using"),
                    value.span,
                    EntityKind::Local {
                        ty: ptr,
                        addr: slot,
                        depth,
                    },
                    false,
                );
                UsingEntry::Place {
                    ty: ptr,
                    entity: e,
                }
            }
            other => {
                return err(
                    value.span,
                    format!("cannot use `using` on {}", self.describe(&other)),
                );
            }
        };
        let entry = self.filter_using_entry(scope, entry, filter, value.span)?;
        self.push_using(scope, entry);
        Ok(())
    }
}

impl TypeState {
    /// `using x;` on a variable needs a struct (or union) or enum, or a pointer to one.
    fn check_using_target(&self, ty: TypeId, span: Span) -> Result<()> {
        let target = self.types.pointee(ty).unwrap_or(ty);
        if self
            .types
            .as_struct(self.types.repr_struct(target))
            .is_some()
            || matches!(self.types.kind(self.types.repr(target)), TypeKind::Enum(_))
        {
            return Ok(());
        }
        err(
            span,
            format!(
                "`using` needs a struct, an enum or a pointer to one; this is `{}`",
                self.types.name(ty)
            ),
        )
    }
}
