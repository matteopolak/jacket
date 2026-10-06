//! Procedures: signatures, IR function/foreign targets, polymorphic
//! instantiation and body lowering.
use super::lower::{FnCtx, Operand};
use super::scope::{EntityKind, Resolved, ScopeKind, UsingEntry};
use super::*;
use crate::ir::{Conv, Sig, Ty};
use crate::types::{ArrayKind, ProcType, TypeKind};

#[derive(Clone, Copy, Debug)]
pub enum ProcTarget {
    Func(ir::FuncId),
    Foreign(ir::ForeignId),
}

#[derive(Clone, Debug)]
pub struct ParamInfo {
    pub name: Option<Sym>,
    pub ty: TypeId,
    pub default: Option<ast::Expr>,
    pub variadic: bool,
    pub using: bool,
    /// `#discard`: callers pass no real value and the body may not name it.
    pub discard: bool,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Signature {
    pub params: Vec<ParamInfo>,
    pub returns: Vec<TypeId>,
    pub return_names: Vec<Option<Sym>>,
    pub ty: TypeId,
    pub has_context: bool,
    pub c_call: bool,
    pub c_varargs: bool,
    /// Scope the parameter types were resolved in (holds polymorphic bindings).
    pub scope: ScopeId,
}

pub struct ProcInfo {
    pub name: Sym,
    pub lit: Rc<ast::ProcLit>,
    /// Defining scope.
    pub scope: ScopeId,
    pub span: Span,
    pub is_poly: bool,
    /// Parameter types were checked for bare polymorphic structs (implicit polymorphism).
    pub poly_checked: bool,
    pub is_macro: bool,
    pub sig: Option<Rc<Signature>>,
    pub sig_resolving: bool,
    pub target: Option<ProcTarget>,
    pub body_state: BodyState,
    pub instances: HashMap<Vec<Value>, ProcId>,
    /// For instances: scope with the polymorphic bindings (parent = defining scope).
    pub bindings: Option<ScopeId>,
    pub export: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyState {
    NotNeeded,
    Queued,
    Lowering,
    Done,
}

/// Does a type expression contain `$T` binders?
pub fn has_poly(expr: &ast::Expr) -> bool {
    use ast::ExprKind as E;
    match &expr.kind {
        E::PolyVar {
            ..
        }
        | E::PolyRestricted {
            ..
        } => true,
        E::Unary(_, e) => has_poly(e),
        E::ArrayType {
            size,
            elem,
        } => {
            has_poly(elem)
                || match size {
                    ast::ArraySize::Fixed(n) => has_poly(n),
                    _ => false,
                }
        }
        E::Call {
            callee,
            args,
            ..
        } => has_poly(callee) || args.iter().any(|a| has_poly(&a.value)),
        E::ProcType(h) => {
            h.params.iter().any(|p| p.ty.as_ref().is_some_and(has_poly))
                || h.returns
                    .iter()
                    .any(|r| r.ty.as_ref().is_some_and(has_poly))
        }
        E::TypeDirective {
            ty, ..
        } => has_poly(ty),
        _ => false,
    }
}

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
        let id = ProcId(self.procs.len() as u32);
        self.procs.push(ProcInfo {
            name,
            lit,
            scope,
            span,
            is_poly,
            poly_checked: is_poly,
            is_macro: header.flags.expand,
            sig: None,
            sig_resolving: false,
            target: None,
            body_state: BodyState::NotNeeded,
            instances: HashMap::default(),
            bindings: None,
            export,
        });
        if self.procs[id.0 as usize].export.is_some() {
            self.exports.push(id);
        }
        id
    }
    /// A parameter typed with a bare polymorphic struct (`t: *Table`) makes the
    /// procedure polymorphic; that needs name resolution, so it is decided lazily.
    pub fn refresh_implicit_poly(&mut self, id: ProcId) -> Result<()> {
        if self.proc(id).poly_checked {
            return Ok(());
        }
        self.procs[id.0 as usize].poly_checked = true;
        let header = self.proc(id).lit.header.clone();
        let scope = self.proc(id).scope;
        for p in &header.params {
            if let Some(t) = &p.ty
                && self.mentions_poly_struct(scope, t)?
            {
                self.procs[id.0 as usize].is_poly = true;
                break;
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

    pub fn ident_poly_struct(
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
        Ok(match self.resolve_entity(id) {
            Ok(Resolved::PolyStruct(ps)) => Some(ps),
            _ => None,
        })
    }

    pub fn proc(&self, id: ProcId) -> &ProcInfo {
        &self.procs[id.0 as usize]
    }

    /// Resolve the signature of a non-polymorphic procedure (or instance).
    pub fn signature(&mut self, id: ProcId, span: Span) -> Result<Rc<Signature>> {
        if let Some(sig) = &self.proc(id).sig {
            return Ok(sig.clone());
        }
        if self.proc(id).is_poly {
            return err(
                span,
                format!(
                    "polymorphic procedure '{}' needs arguments to determine its types",
                    self.proc(id).name
                ),
            );
        }
        if self.proc(id).sig_resolving {
            self.in_progress_misses += 1;
            return err(
                self.proc(id).span,
                format!("signature of '{}' depends on itself", self.proc(id).name),
            );
        }
        self.procs[id.0 as usize].sig_resolving = true;
        let result = self.build_signature(id);
        self.procs[id.0 as usize].sig_resolving = false;
        let sig = Rc::new(result?);
        self.procs[id.0 as usize].sig = Some(sig.clone());
        Ok(sig)
    }

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
        let p = self.proc(id);
        let header = p.lit.header.clone();
        let scope = p.bindings.unwrap_or(p.scope);
        let mut params = Vec::new();
        for param in &header.params {
            // Baked (`$x`) parameters are constants of the instance, not runtime parameters.
            if param.baked {
                continue;
            }
            let ty = match (&param.ty, &param.default) {
                (Some(t), _) if self.mentions_param(t, &params) => {
                    self.type_from_params(scope, t, &params)?
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
                        } => self.default_untyped(ty, &value),
                        Operand::Procs(p) if p.len() == 1 => self.proc_type(p[0], param.span)?,
                        Operand::Type(_) => TypeId::TYPE,
                        other => other.ty(),
                    }
                }
                (None, None) => return err(param.span, "parameter needs a type"),
            };
            let ty = if param.variadic
                && !(header.flags.c_call || header.flags.cpp_method || header.foreign.is_some())
            {
                self.types.array(ty, ArrayKind::View)
            } else {
                ty
            };
            params.push(ParamInfo {
                name: param.name.map(|n| n.name),
                ty,
                default: param.default.clone(),
                variadic: param.variadic,
                using: param.using,
                discard: param.discard,
                span: param.span,
            });
        }
        let mut returns = Vec::new();
        let mut return_names = Vec::new();
        for r in &header.returns {
            let ty = match (&r.ty, &r.default) {
                (Some(t), _) if self.mentions_param(t, &params) => {
                    self.type_from_params(scope, t, &params)?
                }
                (Some(t), _) => self.eval_type(scope, t)?,
                (None, Some(d)) => match self.eval_const(scope, d, None)? {
                    Operand::Const {
                        ty,
                        value,
                        untyped: true,
                    } => self.default_untyped(ty, &value),
                    other => other.ty(),
                },
                (None, None) => return err(r.span, "result needs a type"),
            };
            if ty != TypeId::VOID {
                returns.push(ty);
                return_names.push(r.name.map(|n| n.name));
            }
        }
        if let Some(ty) = self.lambda_return_type(id, scope, &params)?
            && ty != TypeId::VOID
        {
            returns.push(ty);
            return_names.push(None);
        }
        let c_call = header.flags.c_call || header.flags.cpp_method || header.foreign.is_some();
        let c_varargs = c_call && params.last().is_some_and(|p| p.variadic);
        let has_context = !c_call && !header.flags.no_context && !header.flags.intrinsic;
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
            non_pod_return: header.flags.cpp_return_type_is_non_pod,
        };
        let ty = self.types.intern(TypeKind::Proc(Rc::new(proc_type)));
        Ok(Signature {
            params,
            returns,
            return_names,
            ty,
            has_context,
            c_call,
            c_varargs,
            scope,
        })
    }

    pub fn proc_type(&mut self, id: ProcId, span: Span) -> Result<TypeId> {
        let ty = self.signature(id, span)?.ty;
        // A procedure used as a value lends its parameter names and defaults to calls
        // through its type (unless a procedure type header already did).
        if !self.proc_type_params.contains_key(&ty) {
            let proc = self.proc(id);
            let params = &proc.lit.header.params;
            if !proc.is_poly && params.iter().any(|p| p.default.is_some()) {
                let info = super::ProcTypeParams {
                    names: params.iter().map(|p| p.name.map(|n| n.name)).collect(),
                    defaults: params.iter().map(|p| p.default.clone()).collect(),
                    scope: proc.scope,
                };
                self.proc_type_params.insert(ty, Rc::new(info));
            }
        }
        Ok(ty)
    }

    /// Lowered IR signature for a procedure type.
    pub fn ir_sig(&mut self, ty: TypeId, span: Span) -> Result<Sig> {
        let TypeKind::Proc(p) = self.types.kind(ty).clone() else {
            return err(
                span,
                format!("{} is not a procedure type", self.types.name(ty)),
            );
        };
        let mut params = Vec::new();
        if !p.no_context {
            params.push(Ty::Ptr);
        }
        let mut c_abi = ir::CAbi::default();
        if !p.no_context {
            c_abi.params.push(None);
        }
        for &t in &p.params {
            params.push(self.ir_ty(t).unwrap_or(Ty::Ptr));
            c_abi.params.push(if p.c_call && self.is_memory_type(t) {
                Some(self.agg_layout(t, span)?)
            } else {
                None
            });
        }
        let mut returns = Vec::new();
        for (i, &t) in p.returns.iter().enumerate() {
            match self.ir_ty(t) {
                Some(rt) => returns.push(rt),
                None => {
                    params.push(Ty::Ptr);
                    if p.c_call && i == 0 {
                        c_abi.ret = Some(self.agg_layout(t, span)?);
                        c_abi.ret_indirect = p.non_pod_return;
                    }
                    c_abi.params.push(None);
                }
            }
        }
        let needs_abi =
            p.c_call && (c_abi.ret.is_some() || c_abi.params.iter().any(Option::is_some));
        let c_fixed = params.len() as u32;
        Ok(Sig {
            params,
            returns,
            conv: if p.c_call {
                Conv::C
            } else {
                Conv::Jai
            },
            c_varargs: p.c_varargs,
            c_fixed,
            c_abi: needs_abi.then(|| Box::new(c_abi)),
        })
    }

    /// Flattened scalar layout of an aggregate for C ABI classification.
    pub fn agg_layout(&mut self, ty: TypeId, span: Span) -> Result<ir::AggLayout> {
        let size = self.size_of(ty, span)?;
        let align = self.align_of(ty, span)?;
        let mut fields = Vec::new();
        self.flatten_fields(ty, 0, &mut fields, span)?;
        Ok(ir::AggLayout {
            size,
            align,
            fields,
        })
    }

    fn flatten_fields(
        &mut self,
        ty: TypeId,
        base: u64,
        out: &mut Vec<(u64, Ty)>,
        span: Span,
    ) -> Result<()> {
        if let Some(t) = self.ir_ty(ty).or_else(|| self.wide_abi_class(ty)) {
            out.push((base, t));
            return Ok(());
        }
        match self.types.kind(ty).clone() {
            TypeKind::Struct(s) => {
                self.layout_struct(s, span)?;
                let fields = self.types.struct_info(s).fields.clone();
                for field in fields {
                    self.flatten_fields(field.ty, base + field.offset, out, span)?;
                }
            }
            TypeKind::Array {
                elem,
                kind: ArrayKind::Fixed(n),
            } => {
                let size = self.size_of(elem, span)?;
                for i in 0..n {
                    self.flatten_fields(elem, base + i * size, out, span)?;
                }
            }
            TypeKind::String
            | TypeKind::Array {
                kind: ArrayKind::View,
                ..
            } => {
                out.push((base, Ty::I64));
                out.push((base + 8, Ty::Ptr));
            }
            TypeKind::Any => {
                out.push((base, Ty::Ptr));
                out.push((base + 8, Ty::Ptr));
            }
            TypeKind::Array {
                kind: ArrayKind::Resizable,
                ..
            } => {
                for i in 0..5 {
                    out.push((base + i * 8, Ty::I64));
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// The callable target of a procedure, reserving it (and queueing its body) on first use.
    pub fn proc_func(&mut self, id: ProcId, span: Span) -> Result<ProcTarget> {
        if let Some(t) = self.proc(id).target {
            return Ok(t);
        }
        if self.proc(id).is_macro {
            return err(
                span,
                format!(
                    "macro '{}' cannot be used as a procedure value",
                    self.proc(id).name
                ),
            );
        }
        let sig = self.signature(id, span)?;
        let ir_sig = self.ir_sig(sig.ty, span)?;
        let header = self.proc(id).lit.header.clone();
        let name = self.proc(id).name;
        // `#entry_point` declarations stand for the program's `main`.
        if lit_is_entry_point(&header) && self.proc(id).lit.body.is_none() {
            let func = self.entry_point_wrapper(id, span)?;
            let target = ProcTarget::Func(func);
            self.procs[id.0 as usize].target = Some(target);
            return Ok(target);
        }
        let target = if let Some(foreign) = &header.foreign {
            let scope = self.proc(id).scope;
            let library = match &foreign.library {
                Some(lib_ident) => self.resolve_library(scope, lib_ident)?,
                None => None,
            };
            let symbol = match &foreign.name {
                ast::ForeignName::Named(s) => s.to_string(),
                ast::ForeignName::Default => name.to_string(),
            };
            ProcTarget::Foreign(self.program.add_foreign(ir::Foreign {
                symbol,
                library,
                sig: ir_sig,
                is_data: false,
            }))
        } else if self.proc(id).lit.body.is_none() {
            if header.flags.intrinsic || header.flags.compiler {
                // Intrinsics are expanded at call sites; a reference as a value gets a wrapper later.
                let func = self.program.reserve_func(name.to_string());
                self.procs[id.0 as usize].body_state = BodyState::Queued;
                self.body_queue.push(id);
                ProcTarget::Func(func)
            } else {
                // `#elsewhere` or a declaration without body: resolve as a process symbol.
                ProcTarget::Foreign(self.program.add_foreign(ir::Foreign {
                    symbol: name.to_string(),
                    library: None,
                    sig: ir_sig,
                    is_data: false,
                }))
            }
        } else {
            let display = self.proc_display_name(id);
            let func = self.program.reserve_func(display);
            self.procs[id.0 as usize].body_state = BodyState::Queued;
            self.body_queue.push(id);
            ProcTarget::Func(func)
        };
        self.procs[id.0 as usize].target = Some(target);
        if let ProcTarget::Func(func) = target {
            self.func_procs.insert(func, id);
            if header.flags.compiler {
                let hook = match name.as_str() {
                    "write_string" => Some(crate::interp::Hook::WriteString),
                    "write_strings" => Some(crate::interp::Hook::WriteStrings),
                    "compile_time_debug_break" => Some(crate::interp::Hook::DebugBreak),
                    other => crate::build::MetaOp::from_name(other)
                        .map(|op| crate::interp::Hook::Meta(op, !header.flags.no_context)),
                };
                if let Some(hook) = hook {
                    let i = func.0 as usize;
                    if self.interp.hooks.len() <= i {
                        self.interp.hooks.resize(i + 1, None);
                    }
                    self.interp.hooks[i] = Some(hook);
                }
            }
        }
        Ok(target)
    }

    /// Body of an `#entry_point` declaration: call the program's `main` and
    /// convert its result (if any) to the declared return type.
    fn entry_point_wrapper(&mut self, id: ProcId, span: Span) -> Result<ir::FuncId> {
        let sig = self.signature(id, span)?;
        let ir_sig = self.ir_sig(sig.ty, span)?;
        let func = self.program.reserve_func("__entry_point".into());
        self.procs[id.0 as usize].target = Some(ProcTarget::Func(func));
        let main = self.program_main(span)?;
        let main_sig = self.signature(main, span)?;
        if !main_sig.params.is_empty() {
            return err(self.proc(main).span, "'main' must not take parameters");
        }
        let ProcTarget::Func(main_func) = self.proc_func(main, span)? else {
            return err(span, "'main' must have a body");
        };
        let main_ir = self.ir_sig(main_sig.ty, span)?;
        let mut f = FnCtx::new("__entry_point".into(), ir_sig, FileId(0));
        let mut args = Vec::new();
        if main_sig.has_context {
            args.push(if sig.has_context {
                f.b.param(0)
            } else {
                f.b.iconst(Ty::Ptr, 0)
            });
        }
        let results =
            f.b.call(ir::Callee::Func(main_func), args, &main_ir.returns);
        let mut rets = Vec::new();
        if let Some(&rt) = sig.returns.first() {
            let want = self.ir_ty(rt).unwrap_or(Ty::I32);
            let v = match (results.first(), main_sig.returns.first()) {
                (Some(&r), Some(&mt)) if self.types.is_integer(mt) => {
                    let have = f.b.val_ty(r);
                    if have.size() > want.size() {
                        f.b.conv(ir::ConvOp::Trunc, have, want, r)
                    } else if have.size() < want.size() {
                        let signed = self.types.int_info(mt).is_some_and(|i| i.1);
                        f.b.conv(
                            if signed {
                                ir::ConvOp::SExt
                            } else {
                                ir::ConvOp::ZExt
                            },
                            have,
                            want,
                            r,
                        )
                    } else {
                        r
                    }
                }
                _ => f.b.iconst(want, 0),
            };
            rets.push(v);
        }
        f.b.ret(rets);
        self.program.funcs[func.0 as usize] = Some(f.b.finish());
        Ok(func)
    }

    fn proc_display_name(&self, id: ProcId) -> String {
        let p = self.proc(id);
        if p.bindings.is_some() {
            format!("{}#{}", p.name, id.0)
        } else {
            p.name.to_string()
        }
    }

    pub(super) fn resolve_library(
        &mut self,
        scope: ScopeId,
        ident: &ast::Ident,
    ) -> Result<Option<usize>> {
        let ids = self.lookup(scope, ident.name)?;
        let Some(&e) = ids.first() else {
            return err(ident.span, format!("unknown library '{}'", ident.name));
        };
        match self.resolve_entity(e)? {
            scope::Resolved::Library(l) => Ok(Some(self.libraries[l.0 as usize].ir)),
            _ => err(ident.span, format!("'{}' is not a library", ident.name)),
        }
    }

    /// Lower all queued procedure bodies.
    /// A body that fails is retried while other bodies still lower: one lowered later may
    /// declare what it needs (`#insert,scope(...)` into a file). The first error of a pass
    /// that made no progress is reported. Only bodies that existed when the pass started
    /// count as progress: a failing body re-creates its nested procedures on every retry,
    /// and lowering those must not keep the loop alive.
    pub fn drain_bodies(&mut self) -> Result<()> {
        self.unpark_bodies();
        loop {
            let mut failed = Vec::new();
            let mut progress = false;
            let existing = self.procs.len();
            while let Some(id) = self.body_queue.pop() {
                if self.proc(id).body_state != BodyState::Queued {
                    continue;
                }
                match self.lower_body(id) {
                    Ok(()) => progress |= (id.0 as usize) < existing,
                    Err(e) => failed.push((id, e)),
                }
            }
            if failed.is_empty() {
                return Ok(());
            }
            if !progress {
                return Err(failed.into_iter().next().unwrap().1);
            }
            self.body_queue
                .extend(failed.into_iter().rev().map(|(id, _)| id));
        }
    }

    /// Lower the queued bodies that can be lowered now, for compile-time code running in the
    /// middle of checking. A body may need something still being computed (the layout of the
    /// struct whose constant is being evaluated); it stays queued, so the final `drain_bodies`
    /// reports its error. Returns the first such error.
    pub fn drain_bodies_lenient(&mut self) -> Option<Box<Diagnostic>> {
        let mut first = None;
        let mut retry = Vec::new();
        if self.lower_epoch() != self.parked_epoch {
            self.unpark_bodies();
        }
        while let Some(id) = self.body_queue.pop() {
            if self.proc(id).body_state != BodyState::Queued {
                continue;
            }
            // A body that failed is only worth another attempt once something it could be
            // waiting for happened (code added, a compile-time run finished). Otherwise every
            // compile-time call redoes the failing work, which compounds when that work itself
            // runs compile-time code (a body using a module that did not load). Such bodies are
            // parked, so later calls in the same epoch skip them without looking.
            let epoch = self.lower_epoch();
            if let Some((at, err)) = self.lenient_failures.get(&id)
                && *at == epoch
            {
                first.get_or_insert_with(|| err.clone());
                if epoch != self.parked_epoch {
                    retry.append(&mut self.parked_bodies);
                    self.parked_epoch = epoch;
                }
                self.parked_bodies.push(id);
                continue;
            }
            let misses = (self.placeholder_misses, self.in_progress_misses);
            match self.lower_body(id) {
                Ok(()) => {
                    self.lenient_failures.remove(&id);
                }
                Err(e) => {
                    // Waiting for a `#placeholder` is settled by the metaprogram, at any time;
                    // something in progress (a signature, a layout) by the caller finishing.
                    if (self.placeholder_misses, self.in_progress_misses) == misses {
                        let epoch = self.lower_epoch();
                        self.lenient_failures.insert(id, (epoch, e.clone()));
                    }
                    first.get_or_insert(e);
                    retry.push(id);
                }
            }
        }
        self.body_queue.extend(retry);
        if first.is_none()
            && let Some(id) = self.parked_bodies.last()
            && let Some((_, err)) = self.lenient_failures.get(id)
        {
            first = Some(err.clone());
        }
        first
    }

    /// Put the parked bodies back in the queue.
    fn unpark_bodies(&mut self) {
        let parked = std::mem::take(&mut self.parked_bodies);
        self.body_queue.extend(parked);
    }

    /// Counts the events after which a body that failed to lower may succeed.
    fn lower_epoch(&self) -> u64 {
        (self.added_sources + self.files.len() + self.modules.len() + self.runs_done) as u64
    }

    /// The procedure behind IR function `func` if its body is queued (not lowered yet).
    pub fn queued_proc_of(&self, func: ir::FuncId) -> Option<ProcId> {
        let i = self.procs.iter().position(|p| {
            p.body_state == BodyState::Queued
                && matches!(p.target, Some(ProcTarget::Func(f)) if f == func)
        })?;
        Some(ProcId(i as u32))
    }

    /// Lower the queued bodies reachable from IR function `root` (calls, procedure addresses,
    /// procedure pointers in globals), and nothing else. Returns the first error.
    pub fn lower_reachable_from(&mut self, root: ir::FuncId) -> Option<Box<Diagnostic>> {
        self.lower_reachable_roots(vec![root], Vec::new())
    }

    /// `lower_reachable_from` for what a global's initializer points at.
    pub fn lower_reachable_from_global(&mut self, root: ir::GlobalId) -> Option<Box<Diagnostic>> {
        self.lower_reachable_roots(Vec::new(), vec![root.0])
    }

    fn lower_reachable_roots(
        &mut self,
        mut work: Vec<ir::FuncId>,
        mut global_work: Vec<u32>,
    ) -> Option<Box<Diagnostic>> {
        use crate::fxhash::HashSet;
        let mut first = None;
        let (mut funcs, mut globals) = (HashSet::default(), HashSet::default());
        loop {
            while let Some(g) = global_work.pop() {
                if !globals.insert(g) {
                    continue;
                }
                let global: &ir::Global = &self.program.globals[g as usize];
                for r in &global.relocs {
                    match r.target {
                        ir::RelocTarget::Func(f) => work.push(f),
                        ir::RelocTarget::Global(h) => global_work.push(h.0),
                        ir::RelocTarget::Foreign(_) => {}
                    }
                }
            }
            let Some(f) = work.pop() else {
                break;
            };
            if !funcs.insert(f) {
                continue;
            }
            if self
                .program
                .funcs
                .get(f.0 as usize)
                .is_none_or(Option::is_none)
                && let Some(p) = self.queued_proc_of(f)
                && let Err(e) = self.lower_body(p)
            {
                first.get_or_insert(e);
            }
            let Some(Some(func)) = self.program.funcs.get(f.0 as usize) else {
                continue;
            };
            for block in &func.blocks {
                for inst in &block.insts {
                    match inst {
                        ir::Inst::Call {
                            callee: ir::Callee::Func(g),
                            ..
                        } => work.push(*g),
                        ir::Inst::FuncAddr {
                            func, ..
                        } => work.push(*func),
                        ir::Inst::GlobalAddr {
                            global, ..
                        } => global_work.push(global.0),
                        _ => {}
                    }
                }
            }
        }
        first
    }

    /// Lower `id` and then the bodies its lowering queued (its callees, transitively), but
    /// nothing queued before. Returns the first error.
    pub fn lower_with_callees(&mut self, id: ProcId) -> Option<Box<Diagnostic>> {
        let mark = self.body_queue.len();
        if let Err(e) = self.lower_body(id) {
            return Some(e);
        }
        let mut first = None;
        let mut failed = Vec::new();
        while self.body_queue.len() > mark {
            let q = self.body_queue.pop().unwrap();
            if self.proc(q).body_state != BodyState::Queued {
                continue;
            }
            if let Err(e) = self.lower_body(q) {
                first.get_or_insert(e);
                failed.push(q);
            }
        }
        self.body_queue.extend(failed);
        first
    }

    /// Lower one procedure body to IR.
    pub fn lower_body(&mut self, id: ProcId) -> Result<()> {
        self.procs[id.0 as usize].body_state = BodyState::Lowering;
        self.lowering_depth += 1;
        let result = self.lower_body_inner(id);
        self.lowering_depth -= 1;
        self.procs[id.0 as usize].body_state = if result.is_ok() {
            BodyState::Done
        } else {
            BodyState::Queued
        };
        result
    }

    fn lower_body_inner(&mut self, id: ProcId) -> Result<()> {
        let Some(ProcTarget::Func(func_id)) = self.proc(id).target else {
            return Ok(());
        };
        self.lower_body_code(id, Some(func_id))
    }

    /// Lower a body into `func_id`, or only typecheck it (`None`: the code is dropped).
    fn lower_body_code(&mut self, id: ProcId, func_id: Option<ir::FuncId>) -> Result<()> {
        let span = self.proc(id).span;
        let sig = self.signature(id, span)?;
        let lit = self.proc(id).lit.clone();
        let header = lit.header.clone();
        let ir_sig = self.ir_sig(sig.ty, span)?;
        let file = self.scope_file(sig.scope);
        let name = match func_id {
            Some(func_id) => self.program.func_names[func_id.0 as usize].clone(),
            None => "typecheck".into(),
        };
        let mut f = FnCtx::new(name, ir_sig, file);
        f.proc = Some(id);
        f.no_abc = header.flags.no_abc || !self.options.array_bounds_check;
        f.no_aoc = header.flags.no_aoc;
        if let Some(name) = &self.proc(id).export {
            f.b.func.linkage = ir::Linkage::Export(name.clone());
        }
        f.b.func.source_file = file.0;
        if func_id.is_some() {
            self.begin_func_debug(&mut f, id, span);
        }
        if sig.has_context
            && func_id.is_some()
            && header.flags.inline != ast::CallHintFlag::Inline
            && !header.flags.no_debug
        {
            let (line, col) = self.sources.get(span.file).line_col(span.start);
            f.b.func.trace = Some(ir::TraceInfo {
                name: self.proc(id).name.to_string(),
                file: span.file.0,
                line,
                col,
            });
        }
        let module = self.scope(sig.scope).module;
        let scope = self.new_scope(ScopeKind::Proc, Some(sig.scope), module, None);
        self.scope_mut(scope).proc = Some(id);
        let Some(body) = &lit.body else {
            // Intrinsic or #compiler without a body: synthesize a forwarding body.
            let Some(func_id) = func_id else {
                return Ok(());
            };
            return self.lower_intrinsic_wrapper(id, func_id, f, &sig, &header);
        };
        let mut next = 0usize;
        if sig.has_context {
            f.context = Some(f.b.param(0));
            next = 1;
        }
        for param in &sig.params {
            let incoming = f.b.param(next);
            next += 1;
            if param.discard {
                if let Some(name) = param.name {
                    let e = self.add_const(scope, name, param.span, Value::Int(0), TypeId::S64);
                    self.discard_params.insert(e);
                }
                continue;
            }
            let addr = if self.is_memory_type(param.ty) {
                incoming
            } else {
                self.spill(&mut f, param.ty, incoming, param.span)?
            };
            if let Some(name) = param.name {
                self.debug_var(&mut f, scope, name, param.span, param.ty, addr, next as u32);
                let depth = self.scope(scope).proc_depth;
                let e = self.add_entity(
                    scope,
                    name,
                    param.span,
                    EntityKind::Local {
                        ty: param.ty,
                        addr,
                        depth,
                    },
                    false,
                );
                if param.using {
                    self.scope_mut(scope).usings.push(UsingEntry::Place {
                        ty: param.ty,
                        entity: e,
                    });
                }
            }
        }
        f.return_types = sig.returns.clone();
        for (i, &rt) in sig.returns.iter().enumerate() {
            if self.is_memory_type(rt) {
                f.return_outs.push(Some(f.b.param(next)));
                next += 1;
            } else {
                f.return_outs.push(None);
            }
            // Named results get a slot initialized to their default.
            if let Some(name) = sig.return_names[i] {
                let addr = match f.return_outs[i] {
                    Some(out) => out,
                    None => {
                        let size = self.size_of(rt, span)?;
                        let align = self.align_of(rt, span)?;
                        f.b.alloca(size, align)
                    }
                };
                let default = header
                    .returns
                    .iter()
                    .filter(|r| r.name.map(|n| n.name) == Some(name))
                    .find_map(|r| r.default.clone());
                match default {
                    Some(d) => {
                        let op = self.check_expr(&mut f, scope, &d, Some(rt))?;
                        let op = self.convert(&mut f, op, rt, d.span)?;
                        let (_, v) = self.rvalue(&mut f, op, d.span)?;
                        self.store_value(&mut f, rt, addr, v, d.span)?;
                    }
                    None => self.init_default(&mut f, rt, addr, span)?,
                }
                // A result named like a parameter is only reachable through `return`.
                if !self.scope(scope).names.contains_key(&name) {
                    self.debug_var(&mut f, scope, name, span, rt, addr, 0);
                    let depth = self.scope(scope).proc_depth;
                    self.add_entity(
                        scope,
                        name,
                        span,
                        EntityKind::Local {
                            ty: rt,
                            addr,
                            depth,
                        },
                        false,
                    );
                }
                f.named_results.push(Some(addr));
            } else {
                f.named_results.push(None);
            }
        }
        // Parameters and named results live in an outer scope; the body may shadow them.
        let body_scope = self.new_block_scope(scope);
        self.ide_scope_span(body_scope, body.span);
        self.check_block_stmts(&mut f, body_scope, &body.stmts)?;
        if !f.b.is_terminated() {
            self.emit_fallthrough_return(&mut f, body.span)?;
        }
        let func = f.b.finish();
        if let Some(func_id) = func_id {
            self.program.funcs[func_id.0 as usize] = Some(func);
        }
        Ok(())
    }

    /// Falling off the end of a body: return named results, or nothing.
    pub fn emit_fallthrough_return(&mut self, f: &mut FnCtx, span: Span) -> Result<()> {
        self.emit_defers(f, 0, span)?;
        if f.return_types.is_empty() {
            f.b.ret(Vec::new());
            return Ok(());
        }
        if f.named_results.iter().all(Option::is_some) {
            let mut values = Vec::new();
            for (i, &rt) in f.return_types.clone().iter().enumerate() {
                if f.return_outs[i].is_none() {
                    let t = self.ir_ty(rt).unwrap();
                    let addr = f.named_results[i].unwrap();
                    values.push(f.b.load(t, addr));
                }
            }
            f.b.ret(values);
            return Ok(());
        }
        // Missing return: trap at runtime (Jai reports this only for reachable ends).
        f.b.intrinsic(ir::Intrinsic::Trap, Vec::new(), &[]);
        f.b.terminate(ir::Term::Unreachable);
        Ok(())
    }

    /// Body for `#intrinsic`/`#compiler` declarations referenced as values.
    fn lower_intrinsic_wrapper(
        &mut self,
        id: ProcId,
        func_id: ir::FuncId,
        mut f: FnCtx,
        sig: &Signature,
        _header: &ast::ProcHeader,
    ) -> Result<()> {
        let name = self.proc(id).name;
        let start = usize::from(sig.has_context);
        let args: Vec<ir::Val> = (start..f.b.func.sig.params.len())
            .map(|i| f.b.param(i))
            .collect();
        let returns = f.b.func.sig.returns.clone();
        let op = match name.as_str() {
            "memcpy" => ir::Intrinsic::Memcpy,
            "memset" => ir::Intrinsic::Memset,
            "memcmp" => ir::Intrinsic::Memcmp,
            "debug_break" | "compile_time_debug_break" => ir::Intrinsic::DebugBreak,
            _ => ir::Intrinsic::Trap,
        };
        if op == ir::Intrinsic::Trap {
            // Compile-time-only primitives (`#compiler` hooks) have no runtime body.
            f.b.intrinsic(op, Vec::new(), &[]);
            f.b.terminate(ir::Term::Unreachable);
        } else {
            let results = f.b.intrinsic(op, args, &returns);
            f.b.ret(results);
        }
        self.program.funcs[func_id.0 as usize] = Some(f.b.finish());
        Ok(())
    }

    /// Instantiate a polymorphic procedure with bindings (name → value).
    pub fn instantiate(
        &mut self,
        id: ProcId,
        bindings: Vec<(Sym, Value, TypeId)>,
        span: Span,
    ) -> Result<ProcId> {
        let key: Vec<Value> = bindings.iter().map(|(_, v, _)| v.clone()).collect();
        if let Some(&inst) = self.proc(id).instances.get(&key) {
            return Ok(inst);
        }
        let parent = self.proc(id).scope;
        let module = self.scope(parent).module;
        let scope = self.new_scope(ScopeKind::Block, Some(parent), module, None);
        for (name, value, ty) in &bindings {
            self.add_const(scope, *name, span, value.clone(), *ty);
        }
        let p = self.proc(id);
        let (name, lit, pspan, is_macro) = (p.name, p.lit.clone(), p.span, p.is_macro);
        let inst = ProcId(self.procs.len() as u32);
        self.procs.push(ProcInfo {
            name,
            lit,
            scope: parent,
            span: pspan,
            is_poly: false,
            poly_checked: true,
            is_macro,
            sig: None,
            sig_resolving: false,
            target: None,
            body_state: BodyState::NotNeeded,
            instances: HashMap::default(),
            bindings: Some(scope),
            export: None,
        });
        self.procs[id.0 as usize].instances.insert(key, inst);
        if let Some(notes) = self.proc_decl_notes.get(&id).cloned() {
            self.proc_decl_notes.insert(inst, notes);
        }
        Ok(inst)
    }
}


fn lit_is_entry_point(header: &ast::ProcHeader) -> bool {
    header
        .flags
        .other
        .iter()
        .any(|f| f.name.as_str() == "entry_point")
}
