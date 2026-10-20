//! Creating procedures and building their signatures and types.
use super::model::{
    BodyState, ParamInfo, ProcInfo, Signature, has_poly, validate_c_variadic_header,
};
use crate::sema::SourceState;
use crate::sema::lower::Operand;
use crate::sema::scope::{EntityKind, Resolved, ScopeKind};
use crate::sema::{
    Compiler, Diagnostic, EntityId, HashMap, ProcId, Rc, Result, ScopeId, Span, Sym, TypeId, Value,
    ast, err, ir, value,
};
use crate::types::{ArrayKind, ProcType, TypeKind};

impl Compiler {
    pub fn new_proc(
        &mut self,
        name: Sym,
        lit: Rc<ast::ProcLit>,
        scope: ScopeId,
        span: Span,
    ) -> ProcId {
        let header = lit.header.clone();
        let is_poly = header
            .params
            .iter()
            .any(|p| p.baked || p.ty.as_ref().is_some_and(has_poly))
            || header
                .returns
                .iter()
                .any(|r| r.ty.as_ref().is_some_and(has_poly));
        let export = header
            .flags
            .program_export
            .as_ref()
            .map(|export| match export {
                Some(bytes) => String::from_utf8_lossy(bytes).into_owned(),
                None => name.to_string(),
            });
        let id = ProcId(self.pr.procs.len() as u32);
        self.pr.procs.push(ProcInfo {
            name,
            lit,
            scope,
            span,
            is_poly,
            poly_checked: is_poly,
            is_macro: header.flags.expand(),
            sig: None,
            sig_resolving: false,
            target: None,
            body_state: BodyState::NotNeeded,
            instances: HashMap::default(),
            bindings: None,
            export,
        });
        if self.pr.procs[id.0 as usize].export.is_some() {
            self.pr.exports.push(id);
        }
        id
    }

    /// A parameter typed with a bare polymorphic struct (`t: *Table`) makes the
    /// procedure polymorphic; that needs name resolution, so it is decided lazily.
    ///
    /// The answer is final only once every name it needed resolved: a name that cannot be
    /// resolved yet (a `#placeholder` a metaprogram has not defined) leaves the procedure
    /// unchecked, to be decided again, instead of fixing it as monomorphic.
    pub fn refresh_implicit_poly(&mut self, id: ProcId) -> Result<()> {
        if self.proc(id).poly_checked {
            return Ok(());
        }
        // Set while resolving, so the names it resolves cannot start the same decision again.
        self.pr.procs[id.0 as usize].poly_checked = true;
        let header = self.proc(id).lit.header.clone();
        let scope = self.proc(id).scope;
        for p in &header.params {
            let Some(t) = &p.ty else {
                continue;
            };
            match self.mentions_poly_struct_resolved(scope, t) {
                Ok(true) => {
                    self.pr.procs[id.0 as usize].is_poly = true;
                    break;
                }
                Ok(false) => {}
                Err(_) => {
                    self.pr.procs[id.0 as usize].poly_checked = false;
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    /// Does a type expression name a polymorphic struct without parameters?
    pub fn mentions_poly_struct(&mut self, scope: ScopeId, expr: &ast::Expr) -> Result<bool> {
        use ast::ExprKind as E;
        Ok(match &expr.kind {
            E::Ident(name) => self.ident_poly_struct(scope, *name)?.is_some(),
            E::Unary(_, e) => self.mentions_poly_struct(scope, e)?,
            E::ArrayType {
                elem, ..
            } => self.mentions_poly_struct(scope, elem)?,
            E::TypeDirective {
                ty, ..
            } => self.mentions_poly_struct(scope, ty)?,
            _ => false,
        })
    }

    /// `mentions_poly_struct`, but a name that exists and fails to resolve is an error rather
    /// than "no".
    fn mentions_poly_struct_resolved(&mut self, scope: ScopeId, expr: &ast::Expr) -> Result<bool> {
        use ast::ExprKind as E;
        Ok(match &expr.kind {
            E::Ident(name) => self.ident_poly_struct_resolved(scope, *name)?.is_some(),
            E::Unary(_, e) => self.mentions_poly_struct_resolved(scope, e)?,
            E::ArrayType {
                elem, ..
            } => self.mentions_poly_struct_resolved(scope, elem)?,
            E::TypeDirective {
                ty, ..
            } => self.mentions_poly_struct_resolved(scope, ty)?,
            _ => false,
        })
    }

    pub fn ident_poly_struct(
        &mut self,
        scope: ScopeId,
        name: Sym,
    ) -> Result<Option<value::PolyStructId>> {
        Ok(self.ident_poly_struct_resolved(scope, name).unwrap_or(None))
    }

    /// The polymorphic struct `name` denotes, `None` when it denotes something else or nothing,
    /// and an error when it names an entity that fails to resolve.
    fn ident_poly_struct_resolved(
        &mut self,
        scope: ScopeId,
        name: Sym,
    ) -> Result<Option<value::PolyStructId>> {
        let Ok(ids) = self.lookup(scope, name) else {
            return Ok(None);
        };
        let [id] = ids[..] else {
            return Ok(None);
        };
        if matches!(self.entity(id).kind, EntityKind::Local { .. }) {
            return Ok(None);
        }
        Ok(match self.resolve_entity(id)? {
            Resolved::PolyStruct(ps) => Some(ps),
            _ => None,
        })
    }

    pub fn proc(&self, id: ProcId) -> &ProcInfo {
        &self.pr.procs[id.0 as usize]
    }

    /// Resolve the signature of a non-polymorphic procedure (or instance).
    /// Whether `param` of `proc` is declared with the type `Code`: the type itself or a name
    /// for it, resolved where the procedure is defined. Only a bare name can denote it, so any
    /// other spelling (`*T`, `[] T`, `$T`) is answered without resolving.
    pub(crate) fn param_is_code(&mut self, proc: ProcId, param: &ast::Param) -> bool {
        let Some(ty) = &param.ty else {
            return false;
        };
        if !matches!(ty.kind, ast::ExprKind::Ident(_)) {
            return false;
        }
        let scope = self.proc(proc).scope;
        self.eval_type(scope, ty).ok() == Some(TypeId::CODE)
    }

    pub fn signature(&mut self, id: ProcId, span: Span) -> Result<Rc<Signature>> {
        if let Some(sig) = &self.proc(id).sig {
            return Ok(sig.clone());
        }
        crate::sema::body_probe::bump(crate::sema::body_probe::C::SigResolve);
        // A parameter naming a polymorphic struct bare (`f: *Table`) makes it polymorphic,
        // which must be known before its types are: `*Table` alone is a type too.
        self.refresh_implicit_poly(id)?;
        if self.proc(id).is_poly {
            return err(
                span,
                format!(
                    "polymorphic procedure `{}` needs arguments to determine its types",
                    self.proc(id).name
                ),
            );
        }
        if self.proc(id).sig_resolving {
            self.epochs.in_progress_misses += 1;
            return err(
                self.proc(id).span,
                format!("signature of `{}` depends on itself", self.proc(id).name),
            );
        }
        let mut seen: Vec<Sym> = Vec::new();
        for param in &self.proc(id).lit.header.params {
            if let Some(name) = param.name
                && name.name.as_str() != "_"
            {
                if seen.contains(&name.name) {
                    return err(
                        name.span,
                        format!("`{}` is already declared in this scope", name.name),
                    );
                }
                seen.push(name.name);
            }
        }
        self.pr.procs[id.0 as usize].sig_resolving = true;
        let result = self.build_signature(id);
        self.pr.procs[id.0 as usize].sig_resolving = false;
        let sig = Rc::new(result?);
        self.pr.procs[id.0 as usize].sig = Some(sig.clone());
        Ok(sig)
    }

    /// A parameter or result type naming parameters: checked with the parameters in scope
    /// (only their types are known).
    fn type_from_params(
        &mut self,
        scope: ScopeId,
        t: &ast::Expr,
        params: &[ParamInfo],
    ) -> Result<TypeId> {
        let module = self.scope(scope).module;
        let inner = self.new_scope(ScopeKind::Block, Some(scope), module, None);
        let mut scratch = self.scratch_ctx(inner);
        let depth = self.scope(inner).proc_depth;
        for param in params {
            let Some(name) = param.name else {
                continue;
            };
            let addr = scratch.b.iconst(ir::Ty::Ptr, 0);
            self.add_entity(
                inner,
                name,
                param.span,
                EntityKind::Local {
                    ty: param.ty,
                    addr,
                    depth,
                },
                false,
            );
        }
        let op = self.check_expr(&mut scratch, inner, t, Some(TypeId::TYPE))?;
        self.operand_as_type(op, t.span)
    }

    fn build_signature(&mut self, id: ProcId) -> Result<Signature> {
        self.check_proc_modify(id)?;
        let p = self.proc(id);
        let header = p.lit.header.clone();
        validate_c_variadic_header(&header)?;
        let scope = p.bindings.unwrap_or(p.scope);
        // Baked (`$x`) parameters are constants of the instance, not runtime parameters.
        let runtime: Vec<&ast::Param> = header.params.iter().filter(|p| !p.baked).collect();
        // A type may name a later parameter (`info: *r.Info, r: *$R`, n00bmind/reflector):
        // such types are resolved after the others.
        let names_later = |c: &Compiler, i: usize| {
            runtime[i].ty.as_ref().is_some_and(|t| {
                (t.span.file.0 as usize) < c.src.sources.len() && {
                    let text = c.src.sources.snippet(t.span);
                    let word = |ch: char| ch.is_alphanumeric() || ch == '_';
                    text.split(|ch: char| !word(ch)).any(|w| {
                        runtime[i + 1..]
                            .iter()
                            .any(|p| p.name.is_some_and(|n| n.name.as_str() == w))
                    })
                }
            })
        };
        let deferred: Vec<bool> = (0..runtime.len()).map(|i| names_later(self, i)).collect();
        let mut slots: Vec<Option<ParamInfo>> = vec![None; runtime.len()];
        for pass in [false, true] {
            for (i, param) in runtime.iter().enumerate() {
                if deferred[i] != pass {
                    continue;
                }
                let params: Vec<ParamInfo> = slots.iter().flatten().cloned().collect();
                slots[i] = Some(self.param_info(scope, &header, param, &params)?);
            }
        }
        let params: Vec<ParamInfo> = slots.into_iter().flatten().collect();
        self.finish_signature(id, scope, &header, params)
    }

    fn param_info(
        &mut self,
        scope: ScopeId,
        header: &ast::ProcHeader,
        param: &ast::Param,
        params: &[ParamInfo],
    ) -> Result<ParamInfo> {
        {
            let ty = match (&param.ty, &param.default) {
                (Some(t), _) if self.src.mentions_param(t, params) => {
                    self.type_from_params(scope, t, params)?
                }
                (Some(t), _) => self.eval_type(scope, t)?,
                (None, Some(d)) if matches!(d.kind, ast::ExprKind::CallerCode) => TypeId::CODE,
                (None, Some(d)) => {
                    // Only the type matters; defaults like `context.allocator` are runtime values.
                    let op = self.check_expr_no_emit(scope, d)?;
                    match op {
                        Operand::Const {
                            ty,
                            value,
                            untyped: true,
                        } => self.ty.default_untyped(ty, &value),
                        Operand::Procs(p) if p.len() == 1 => self.proc_type(p[0], param.span)?,
                        Operand::Type(_) => TypeId::TYPE,
                        other => other.ty(),
                    }
                }
                (None, None) => return err(param.span, "parameter needs a type"),
            };
            let ty = if param.variadic && !header.uses_c_abi() {
                self.ty.types.array(ty, ArrayKind::View)
            } else {
                ty
            };
            Ok(ParamInfo {
                name: param.name.map(|n| n.name),
                ty,
                default: param.default.clone(),
                callable: if matches!(self.ty.types.kind(ty), TypeKind::Proc(_)) {
                    self.callable_expression_parameters(
                        scope,
                        param.ty.as_ref().or(param.default.as_ref()),
                    )?
                } else {
                    None
                },
                variadic: param.variadic,
                using: param.using,
                using_filter: param.using_filter.clone(),
                discard: param.discard,
                span: param.span,
            })
        }
    }

    fn finish_signature(
        &mut self,
        id: ProcId,
        scope: ScopeId,
        header: &ast::ProcHeader,
        params: Vec<ParamInfo>,
    ) -> Result<Signature> {
        let mut returns = Vec::new();
        let mut return_callables = Vec::new();
        let mut return_names = Vec::new();
        for r in &header.returns {
            let ty = match (&r.ty, &r.default) {
                (Some(t), _) if self.src.mentions_param(t, &params) => {
                    self.type_from_params(scope, t, &params)?
                }
                (Some(t), _) => self.eval_type(scope, t)?,
                (None, Some(d)) => match self.eval_const(scope, d, None)? {
                    Operand::Const {
                        ty,
                        value,
                        untyped: true,
                    } => self.ty.default_untyped(ty, &value),
                    other => other.ty(),
                },
                (None, None) => return err(r.span, "result needs a type"),
            };
            if let Some(n) = r.name
                && self.ed.is_live()
            {
                self.ide_note_param_name(n.span, n.name, ty);
            }
            if ty != TypeId::VOID {
                returns.push(ty);
                return_callables.push(if matches!(self.ty.types.kind(ty), TypeKind::Proc(_)) {
                    self.callable_expression_parameters(
                        scope,
                        r.ty.as_ref().or(r.default.as_ref()),
                    )?
                } else {
                    None
                });
                return_names.push(r.name.map(|n| n.name));
            }
        }
        if let Some(ty) = self.lambda_return_type(id, scope, &params)?
            && ty != TypeId::VOID
        {
            returns.push(ty);
            return_callables.push(None);
            return_names.push(None);
        }
        if !self.proc(id).is_macro {
            for param in &params {
                self.ensure_storage_type(param.ty, param.span)?;
            }
        }
        for &ty in &returns {
            self.ensure_storage_type(ty, self.proc(id).span)?;
        }
        let c_call = header.uses_c_abi();
        let c_varargs = c_call && params.last().is_some_and(|p| p.variadic);
        let has_context = !c_call && !header.flags.no_context() && !header.flags.intrinsic();
        let proc_type = ProcType {
            params: params
                .iter()
                .filter(|p| !(c_varargs && p.variadic))
                .map(|p| p.ty)
                .collect(),
            returns: returns.clone(),
            variadic: params.last().is_some_and(|p| p.variadic) && !c_varargs,
            c_varargs,
            c_call,
            no_context: !has_context,
            non_pod_return: header.flags.cpp_return_type_is_non_pod(),
        };
        let ty = self.ty.types.intern(TypeKind::Proc(Rc::new(proc_type)));
        Ok(Signature {
            params,
            returns,
            return_callables,
            return_names,
            ty,
            has_context,
            c_call,
            c_varargs,
            scope,
        })
    }

    pub fn proc_type(&mut self, id: ProcId, span: Span) -> Result<TypeId> {
        Ok(self.signature(id, span)?.ty)
    }

    /// Two procedures declared in one scope under one name must differ in their parameter types
    /// (polymorphic ones are told apart when a call picks between them). Checked when the
    /// overload set is first used.
    pub fn check_identical_overloads(&mut self, ids: &[EntityId]) -> Result<()> {
        if self.ns.overload_sets_checked.contains(ids) {
            return Ok(());
        }
        let mut seen: Vec<(EntityId, ProcId, Rc<[TypeId]>, bool)> = Vec::new();
        // A member that cannot be resolved yet (a `#placeholder` in its parameters) reports
        // its own error; the set is checked again once it can be compared.
        let mut complete = true;
        for &id in ids {
            let Ok(
                Resolved::Proc(proc)
                | Resolved::Const {
                    value: Value::Proc(proc),
                    ..
                },
            ) = self.resolve_entity(id)
            else {
                complete = false;
                continue;
            };
            if self.proc(proc).is_poly || self.proc(proc).lit.body.is_none() {
                continue;
            }
            let span = self.entity(id).span;
            let Ok(sig) = self.signature(proc, span) else {
                complete = false;
                continue;
            };
            let TypeKind::Proc(ty) = self.ty.types.kind(sig.ty).clone() else {
                continue;
            };
            // Two aliases of one procedure are the same overload, not a clash.
            if seen.iter().any(|(_, other, ..)| *other == proc) {
                continue;
            }
            let scope = self.binding_scope(self.entity(id).scope);
            let params: Rc<[TypeId]> = ty.params.iter().copied().collect();
            if let Some((first, ..)) = seen.iter().find(|(other, _, p, v)| {
                self.binding_scope(self.entity(*other).scope) == scope
                    && **p == *params
                    && *v == ty.variadic
            }) {
                let first = self.entity(*first).span;
                return Err(Box::new(
                    Diagnostic::error(
                        span,
                        "two procedures with this name have identical parameter types",
                    )
                    .with_label("this declaration cannot be told apart from the first")
                    .with_note(first, "the first declaration")
                    .with_help("change a parameter's type, or remove one of them"),
                ));
            }
            seen.push((id, proc, params, ty.variadic));
        }
        // Only a clean, fully compared set is remembered: a caller that swallows the error (a
        // speculative lookup) must not hide it from the next check.
        if complete {
            self.ns.overload_sets_checked.insert(ids.to_vec());
        }
        Ok(())
    }
}

impl SourceState {
    /// Does type `t` name one of the parameters (`-> type_of(asset)`, `p: cache.Proc`)?
    fn mentions_param(&self, t: &ast::Expr, params: &[ParamInfo]) -> bool {
        if (t.span.file.0 as usize) >= self.sources.len() {
            return false;
        }
        let text = self.sources.snippet(t.span);
        let word = |c: char| c.is_alphanumeric() || c == '_';
        text.split(|c: char| !word(c)).any(|w| {
            params
                .iter()
                .any(|p| p.name.is_some_and(|n| n.as_str() == w))
        })
    }
}
