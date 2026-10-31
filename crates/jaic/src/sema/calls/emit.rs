//! Emitting a resolved call: argument operands, variadics, defaults and indirect calls.

use super::count_hints::remove_extra_arguments;
use super::{
    ArrayKind, CallArg, Candidate, Compiler, Diagnostic, E, FnCtx, Operand, ProcTarget,
    ProcTypeParams, Rc, Result, ScopeId, Signature, Slot, Span, Ty, TypeId, TypeKind, Value, ast,
    c_vararg_promote, err, ir, procs,
};
use crate::diagnostics::Code;
use crate::sema::TypeState;
use crate::sema::lower::Origin;
use crate::source::{Applicability, Suggestion};

impl Compiler {
    /// Emit a resolved call, recording which of its results are `#must`.
    pub(super) fn emit_call(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        c: Candidate,
        args: Vec<CallArg>,
        span: Span,
    ) -> Result<Operand> {
        let proc = c.proc;
        let header = self.proc(proc).lit.header.clone();
        let must: Vec<bool> = header.returns.iter().map(|r| r.must).collect();
        let name = self.proc(proc).name;
        // `pa + pb` on `#type,isa` variants of a base type calls the base's operator; a result
        // of exactly the base type is cast back up to the variant.
        let variant = args.iter().find_map(|a| {
            let ty = a.op.as_ref()?.ty();
            match self.ty.types.kind(ty) {
                TypeKind::Distinct(d) if self.ty.types.distincts[d.0 as usize].isa => {
                    Some((ty, self.ty.types.distincts[d.0 as usize].base))
                }
                _ => None,
            }
        });
        let is_macro = self.proc(proc).is_macro;
        let result = self.emit_call_inner(f, scope, c, args, span);
        let result = match (result, variant) {
            (
                Ok(Operand::Value {
                    ty,
                    val,
                }),
                Some((variant, base)),
            ) if ty == base && !is_macro && header.returns.len() == 1 => Ok(Operand::Value {
                ty: variant,
                val,
            }),
            (result, _) => result,
        };
        self.pr.last_call_must = must.iter().any(|&m| m).then_some((span, name, must));
        result
    }

    pub(super) fn emit_call_inner(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        c: Candidate,
        args: Vec<CallArg>,
        span: Span,
    ) -> Result<Operand> {
        let proc = c.proc;
        if self.proc(proc).is_macro {
            return self.expand_macro(f, scope, proc, &c.slots, args, span);
        }
        let header = self.proc(proc).lit.header.clone();
        let sig = self.signature(proc, span)?;
        // Intrinsics expand inline.
        if header.flags.intrinsic()
            && self.proc(proc).lit.body.is_none()
            && super::procs::llvm_intrinsic(&header).is_none()
        {
            return self.emit_intrinsic(f, proc, (&sig, &header), &c.slots, args, span);
        }
        let mut values = Vec::new();
        if sig.has_context {
            let Some(ctx) = f.current_context() else {
                return err(
                    Code::CallCannotWithoutContext,
                    span,
                    format!(
                        "cannot call `{}` without a context (use push_context in #c_call code)",
                        self.proc(proc).name
                    ),
                );
            };
            values.push(ctx);
        }
        let mut k = 0;
        let mut c_vararg_values = Vec::new();
        // The aggregate layout of each variadic argument that travels as one (a struct, or a wide
        // `Long_Double`).
        let mut c_vararg_layouts: Vec<Option<ir::AggLayout>> = Vec::new();
        for (i, slot) in c.slots.iter().enumerate() {
            if header.params[i].baked {
                continue;
            }
            let param = sig.params[k].clone();
            k += 1;
            if sig.c_varargs && param.variadic {
                if let Slot::Variadic(list) = slot {
                    for &a in list {
                        let op = self.arg_operand(f, &args[a], None)?;
                        let op = self.settle_untyped(op, None);
                        let op = c_vararg_promote(self, f, op, span)?;
                        // A wide `Long_Double` or a struct travels by value, as its layout.
                        let by_value = self.ty.types.wide_float(op.ty()).is_some()
                            || (matches!(self.ty.types.kind(op.ty()), TypeKind::Struct(_))
                                && self.size_of(op.ty(), span)? > 0);
                        let layout = if by_value {
                            Some(self.agg_layout(op.ty(), span)?)
                        } else {
                            None
                        };
                        let (_, v) = self.rvalue(f, op, span)?;
                        c_vararg_values.push(v);
                        c_vararg_layouts.push(layout);
                    }
                }
                continue;
            }
            let v = self.param_value(f, &sig, &param, slot, &args, span)?;
            values.push(v);
        }
        // Out-pointers for memory-class results, before any variadic argument.
        let mut outs = Vec::new();
        for &rt in &sig.returns {
            if self.is_memory_type(rt) {
                let size = self.size_of(rt, span)?;
                let align = self.align_of(rt, span)?;
                let out = f.b.alloca(size, align);
                values.push(out);
                outs.push(Some(out));
            } else {
                outs.push(None);
            }
        }
        values.extend(c_vararg_values.iter().copied());
        let target = self.proc_func(proc, span)?;
        let mut ir_sig = self.ir_sig(sig.ty, span)?;
        if c_vararg_layouts.iter().any(Option::is_some) {
            // The variadic arguments follow the fixed ones in `c_abi.params`, as in the signature.
            let fixed = ir_sig.params.len();
            let abi = ir_sig.c_abi.get_or_insert_with(Default::default);
            abi.params.resize(fixed, None);
            abi.params.extend(c_vararg_layouts.iter().cloned());
        }
        let callee = match target {
            ProcTarget::Func(id) => ir::Callee::Func(id),
            ProcTarget::Foreign(id) => {
                if sig.c_varargs {
                    // Record the actual argument classes for this variadic call site.
                    record_vararg_types(f, &mut ir_sig, &c_vararg_values);
                    let fp = f.b.foreign_addr(id);
                    ir::Callee::Indirect(fp, Box::new(ir_sig.clone()))
                } else {
                    ir::Callee::Foreign(id)
                }
            }
        };
        if sig.c_varargs && !matches!(callee, ir::Callee::Indirect(..)) {
            record_vararg_types(f, &mut ir_sig, &c_vararg_values);
        }
        let results = f.b.call(callee, values, &ir_sig.returns.clone());
        Ok(self.call_results(f, &sig.returns, results, outs, &sig.return_callables))
    }

    pub(super) fn call_results(
        &mut self,
        f: &mut FnCtx,
        returns: &[TypeId],
        results: Vec<ir::Val>,
        outs: Vec<Option<ir::Val>>,
        callables: &[Option<Rc<ProcTypeParams>>],
    ) -> Operand {
        let mut values = Vec::new();
        let mut regs = results.into_iter();
        for (i, &rt) in returns.iter().enumerate() {
            let v = match outs[i] {
                Some(out) => out,
                // A `void` result has no register; its value is an empty slot.
                None if rt == TypeId::VOID => f.b.alloca(1, 1),
                None => regs.next().unwrap(),
            };
            values.push(
                Operand::Value {
                    ty: rt,
                    val: v,
                }
                .with_callable(callables.get(i).cloned().flatten()),
            );
        }
        match values.len() {
            0 => Operand::Void,
            1 => values.remove(0),
            _ => Operand::Multi(values),
        }
    }

    /// Check (if deferred) and convert an argument for a parameter of type `ty`.
    pub(super) fn arg_operand(
        &mut self,
        f: &mut FnCtx,
        arg: &CallArg,
        ty: Option<TypeId>,
    ) -> Result<Operand> {
        if let Some(op) = &arg.op {
            Ok(op.clone())
        } else {
            let expr = arg.expr.as_ref().unwrap();
            self.check_expr(f, arg.scope, expr, ty)
        }
    }

    /// Join scalar and spread segments in their original argument order. The
    /// resulting storage belongs to the caller's frame, just as a fixed pack.
    pub(super) fn mixed_variadic_value(
        &mut self,
        f: &mut FnCtx,
        param: &procs::ParamInfo,
        args: &[CallArg],
        list: &[usize],
        span: Span,
    ) -> Result<ir::Val> {
        let elem = self.ty.jai_variadic_element(param.ty, span)?;
        let size = self.size_of(elem, span)?;
        let align = self.align_of(elem, span)?;
        let limit = f.b.iconst(Ty::I64, crate::types::MAX_SIZE / size.max(1));
        let one = f.b.iconst(Ty::I64, 1);
        let mut count = f.b.iconst(Ty::I64, 0);
        let mut segments = Vec::new();
        for &a in list {
            let target = if args[a].spread {
                param.ty
            } else {
                elem
            };
            let op = self.arg_operand(f, &args[a], Some(target))?;
            let op = self.convert(f, op, target, args[a].site())?;
            let (_, value) = self.rvalue(f, op, args[a].span)?;
            let (n, source) = if args[a].spread {
                let n = f.b.load(Ty::I64, value);
                let field = f.b.ptr_offset(value, 8);
                (n, f.b.load(Ty::Ptr, field))
            } else {
                (one, value)
            };
            let remaining = f.b.bin(ir::BinOp::Sub, Ty::I64, limit, count);
            let bad = f.b.cmp(ir::CmpOp::UGt, Ty::I64, n, remaining);
            let fail = f.b.new_block();
            let ready = f.b.new_block();
            f.b.branch(bad, fail, ready);
            f.b.switch_to(fail);
            self.emit_check_failed(f, ir::CheckReason::VariadicSize, n, remaining, true, span)?;
            f.b.jump(ready);
            f.b.switch_to(ready);
            count = f.b.bin(ir::BinOp::Add, Ty::I64, count, n);
            segments.push((args[a].spread, n, source));
        }
        let width = f.b.iconst(Ty::I64, size);
        let bytes = f.b.bin(ir::BinOp::Mul, Ty::I64, count, width);
        let base = f.b.new_val(Ty::Ptr);
        f.b.push(ir::Inst::StackAlloc {
            dst: base,
            size: bytes,
            align,
        });
        let mut cursor = base;
        for (spread, n, source) in segments {
            let bytes = f.b.bin(ir::BinOp::Mul, Ty::I64, n, width);
            if spread {
                f.b.intrinsic(ir::Intrinsic::Memcpy, vec![cursor, source, bytes], &[]);
            } else {
                self.store_value(f, elem, cursor, source, span)?;
            }
            cursor = f.b.ptr_add(cursor, bytes);
        }
        let view = f.b.alloca(16, 8);
        f.b.store(Ty::I64, view, count);
        let data = f.b.ptr_offset(view, 8);
        f.b.store(Ty::Ptr, data, base);
        Ok(view)
    }

    pub(super) fn param_value(
        &mut self,
        f: &mut FnCtx,
        sig: &Signature,
        param: &procs::ParamInfo,
        slot: &Slot,
        args: &[CallArg],
        span: Span,
    ) -> Result<ir::Val> {
        if param.discard {
            // `#discard`: the argument (or default) is typechecked, never evaluated.
            return self.zero_param(f, param.ty, span);
        }
        let op = match slot {
            Slot::Arg(a) if param.ty == TypeId::CODE && args[*a].op.is_none() => {
                // An expression for a `Code` parameter is passed as code unless it is a Code.
                let arg = &args[*a];
                let expr = arg.expr.as_ref().unwrap();
                match self.check_expr_no_emit(arg.scope, expr) {
                    Ok(op) if op.ty() == TypeId::CODE => {
                        self.arg_operand(f, arg, Some(param.ty))?
                    }
                    _ => {
                        let body = Rc::new(ast::CodeBody::Expr(expr.clone()));
                        Operand::Const {
                            ty: TypeId::CODE,
                            value: Value::Code(self.add_code(body, arg.scope)),
                            untyped: false,
                            origin: Origin::Computed,
                        }
                    }
                }
            }
            Slot::Arg(a) | Slot::Spread(a) => {
                let op = self.arg_operand(f, &args[*a], Some(param.ty))?;
                if self.ty.auto_deref_arg(op.ty(), param.ty) {
                    let (_, p) = self.rvalue(f, op, args[*a].span)?;
                    Operand::Place {
                        ty: param.ty,
                        addr: p,
                    }
                } else {
                    self.convert(f, op, param.ty, args[*a].site())?
                }
            }
            Slot::Default => {
                if param.variadic && param.default.is_none() {
                    // Empty variadic: a zero view.
                    let view = f.b.alloca(16, 8);
                    f.b.zero(view, 16);
                    return Ok(view);
                }
                let d = param.default.clone().unwrap();
                let op = match &d.kind {
                    E::CallerLocation => {
                        if let Some(first) = args.first()
                            && (span.file.0 as usize) < self.src.sources.len()
                        {
                            let (line, col) = self.src.sources.get(span.file).line_col(span.start);
                            self.pr.first_arguments.insert(
                                (span.file.0, line, col),
                                first.expr.as_ref().map_or(first.span, ast::Expr::extent),
                            );
                        }
                        self.location_operand(f, span)?
                    }
                    _ => self.check_expr(f, sig.scope, &d, Some(param.ty))?,
                };
                self.convert(f, op, param.ty, &d)?
            }
            Slot::Variadic(list) => {
                if let Some(&a) = list.iter().find(|&&a| args[a].spread) {
                    self.warn_spread_with_other_arguments(args[a].scope, args[a].span);
                    return self.mixed_variadic_value(f, param, args, list, span);
                }
                let elem = self.ty.jai_variadic_element(param.ty, span)?;
                let size = self.size_of(elem, span)?;
                let align = self.align_of(elem, span)?;
                let n = list.len() as u64;
                let base = f.b.alloca((size * n).max(1), align);
                for (i, &a) in list.iter().enumerate() {
                    let op = self.arg_operand(f, &args[a], Some(elem))?;
                    let op = self.convert(f, op, elem, args[a].site())?;
                    let (_, v) = self.rvalue(f, op, span)?;
                    let at = f.b.ptr_offset(base, size * i as u64);
                    self.store_value(f, elem, at, v, span)?;
                }
                let view = f.b.alloca(16, 8);
                let count = f.b.iconst(Ty::I64, n);
                f.b.store(Ty::I64, view, count);
                let dp = f.b.ptr_offset(view, 8);
                f.b.store(Ty::Ptr, dp, base);
                return Ok(view);
            }
        };
        if self.is_memory_type(param.ty) {
            let (_, addr) = self.address_of(f, op, span)?;
            Ok(addr)
        } else {
            let (_, v) = self.rvalue(f, op, span)?;
            Ok(v)
        }
    }

    /// An all-zero value of a parameter type, passed where an argument is discarded.
    pub(super) fn zero_param(&mut self, f: &mut FnCtx, ty: TypeId, span: Span) -> Result<ir::Val> {
        if self.is_memory_type(ty) {
            let size = self.size_of(ty, span)?;
            let align = self.align_of(ty, span)?;
            let slot = f.b.alloca(size.max(1), align);
            f.b.zero(slot, size);
            return Ok(slot);
        }
        // Memory types returned above; only `void` is left without a scalar type, and a discarded
        // `void` parameter is a word of zero.
        let t = self.ir_ty(ty).unwrap_or(Ty::I64);
        Ok(if t.is_float() {
            f.b.fconst(t, 0.0)
        } else {
            f.b.iconst(t, 0)
        })
    }

    /// Place named arguments of a call through a procedure value and fill in the defaults its
    /// type was written with.
    pub(super) fn order_indirect_args(
        &mut self,
        info: &ProcTypeParams,
        args: Vec<CallArg>,
        c_varargs: bool,
        span: Span,
    ) -> Result<Vec<CallArg>> {
        let n = info.names.len();
        let mut slots: Vec<Option<CallArg>> = vec![None; n];
        let mut extra = Vec::new();
        let mut next = 0;
        let all_args = args.clone();
        for (position, arg) in args.into_iter().enumerate() {
            let i = if let Some(name) = arg.name {
                info.names
                    .iter()
                    .position(|&p| p == Some(name))
                    .ok_or_else(|| {
                        Box::new(Diagnostic::new(
                            Code::CallNoParameterNamed,
                            arg.span,
                            format!("no parameter named `{name}`"),
                        ))
                    })?
            } else {
                while next < n && slots[next].is_some() {
                    next += 1;
                }
                next
            };
            if i >= n {
                if c_varargs && arg.name.is_none() {
                    extra.push(arg);
                    continue;
                }
                let mut d = Diagnostic::new(
                    Code::ArgumentCount,
                    arg.span,
                    format!("too many arguments: it takes at most {n}"),
                )
                .with_label("this argument has no parameter");
                if let Some(remove) = remove_extra_arguments(&all_args, position) {
                    d = d.with_suggestion(remove);
                }
                return Err(Box::new(d));
            }
            if slots[i].is_some() {
                return err(
                    Code::CallArgument,
                    arg.span,
                    format!("argument {} supplied more than once", i + 1),
                );
            }
            slots[i] = Some(CallArg {
                name: None,
                ..arg
            });
        }
        let mut ordered = slots
            .into_iter()
            .enumerate()
            .map(|(i, slot)| match (slot, &info.defaults[i]) {
                (Some(arg), _) => Ok(arg),
                (None, Some(default)) => Ok(CallArg {
                    name: None,
                    spread: false,
                    expr: Some(default.clone()),
                    op: None,
                    span: default.span,
                    scope: info.scope,
                }),
                (None, None) => {
                    let mut d = Diagnostic::new(
                        Code::ArgumentCount,
                        span,
                        format!("missing argument {}", i + 1),
                    );
                    if let Some(name) = info.names[i] {
                        d = d.with_label(format!("no value for parameter `{name}`"));
                    }
                    Err(Box::new(d))
                }
            })
            .collect::<Result<Vec<_>>>()?;
        ordered.extend(extra);
        Ok(ordered)
    }

    pub(super) fn call_indirect(
        &mut self,
        f: &mut FnCtx,
        ty: TypeId,
        callee: ir::Val,
        parameters: Option<Rc<ProcTypeParams>>,
        mut args: Vec<CallArg>,
        span: Span,
    ) -> Result<Operand> {
        let TypeKind::Proc(pt) = self.ty.types.kind(ty).clone() else {
            return err(
                Code::CallCannotValueType,
                span,
                format!("cannot call a value of type {}", self.ty.types.name(ty)),
            );
        };
        let result_parameters = parameters
            .as_ref()
            .map(|info| info.returns.clone())
            .unwrap_or_default();
        if let Some(info) = parameters {
            args = self
                .order_indirect_args(&info, args, pt.c_varargs, span)
                .map_err(|d| self.with_type_note(*d, ty))?;
        } else if args.iter().any(|arg| arg.name.is_some()) {
            return err(
                Code::CallNamedArguments,
                span,
                "named arguments require a known callable declaration",
            );
        }
        if args.len() < pt.params.len()
            || (!pt.c_varargs && !pt.variadic && args.len() > pt.params.len())
        {
            let d = Diagnostic::new(
                Code::ArgumentCount,
                span,
                format!(
                    "expected {} arguments, found {}",
                    pt.params.len(),
                    args.len()
                ),
            );
            return Err(self.with_type_note(d, ty));
        }
        let mut values = Vec::new();
        if !pt.no_context {
            // Without a context (e.g. inside #c_call code) the callee receives null.
            let ctx = match f.current_context() {
                Some(c) => c,
                None => f.b.iconst(Ty::Ptr, 0),
            };
            values.push(ctx);
        }
        for (i, &param_ty) in pt.params.iter().enumerate() {
            let op = self.arg_operand(f, &args[i], Some(param_ty))?;
            let op = self.convert(f, op, param_ty, args[i].site())?;
            if self.is_memory_type(param_ty) {
                let (_, a) = self.address_of(f, op, span)?;
                values.push(a);
            } else {
                let (_, v) = self.rvalue(f, op, span)?;
                values.push(v);
            }
        }
        let mut sig = self.ir_sig(ty, span)?;
        for arg in &args[pt.params.len()..] {
            let op = self.arg_operand(f, arg, None)?;
            let op = self.settle_untyped(op, None);
            let op = c_vararg_promote(self, f, op, span)?;
            let (_, v) = self.rvalue(f, op, span)?;
            sig.params.push(f.b.val_ty(v));
            values.push(v);
        }
        let mut outs = Vec::new();
        for &rt in &pt.returns {
            if self.is_memory_type(rt) {
                let size = self.size_of(rt, span)?;
                let align = self.align_of(rt, span)?;
                let out = f.b.alloca(size, align);
                values.push(out);
                outs.push(Some(out));
            } else {
                outs.push(None);
            }
        }
        let returns = sig.returns.clone();
        let results = f.b.call(
            ir::Callee::Indirect(callee, Box::new(sig)),
            values,
            &returns,
        );
        Ok(self.call_results(f, &pt.returns, results, outs, &result_parameters))
    }
}

impl TypeState {
    /// The checked representation of a Jai runtime pack, distinct from C ellipses.
    pub(super) fn jai_variadic_element(&self, ty: TypeId, span: Span) -> Result<TypeId> {
        match *self.types.kind(ty) {
            TypeKind::Array {
                elem,
                kind: ArrayKind::View,
            } => Ok(elem),
            _ => err(
                Code::CallJaiVariadicPack,
                span,
                "a Jai variadic pack requires an array-view parameter",
            ),
        }
    }
}

impl Compiler {
    /// Error when a call at `call_span` leaves one of its `#must` results unused; only the
    /// first `used` results are taken (0 for a call statement). Checked when the call's code
    /// is generated, so a violation inside never-expanded macro code is harmless.
    pub(in crate::sema) fn check_must_used(&mut self, call_span: Span, used: usize) -> Result<()> {
        let Some((span, name, must)) = self.pr.last_call_must.take() else {
            return Ok(());
        };
        if span != call_span {
            return Ok(());
        }
        if !must.iter().skip(used).any(|&m| m) {
            return Ok(());
        }
        let mut d = Diagnostic::new(
            Code::CallResultMarked,
            call_span,
            format!("the result of `{name}` is marked #must and cannot be discarded"),
        );
        if used > 0 {
            return Err(Box::new(d.with_help(format!(
                "`{name}` returns {} results: take all of them, using `_` for the ones you do not need",
                must.len()
            ))));
        }
        d = d.with_help(format!("or use the result: `result := {name}(...);`"));
        // `_ := f();` (`_, _ :=` for several results) is the way to drop a result on purpose,
        // written in front of a call that starts its statement.
        let text = &self.src.sources.get(call_span.file).text;
        let line_start = text
            .get(..call_span.start as usize)
            .and_then(|head| head.rfind('\n'))
            .map_or(0, |n| n + 1);
        let before = text
            .get(line_start..call_span.start as usize)
            .unwrap_or("x");
        if before.trim().is_empty() && !self.src.sources.snippet(call_span).is_empty() {
            let discard = vec!["_"; must.len()].join(", ");
            d = d.with_suggestion(
                Suggestion::insert(
                    format!("discard the result on purpose with `{discard} := ...`"),
                    call_span,
                    format!("{discard} := "),
                    Applicability::MaybeIncorrect,
                )
                .with_action("Discard the result with `_`"),
            );
        }
        Err(Box::new(d))
    }
}

/// Appends the IR types of a C variadic call's extra arguments to its signature. A type-only
/// probe skips it: its arguments may name values of the enclosing function, not of the probe.
fn record_vararg_types(f: &FnCtx, sig: &mut ir::Sig, values: &[ir::Val]) {
    if !f.type_only {
        sig.params.extend(values.iter().map(|&v| f.b.val_ty(v)));
    }
}
