//! Calls: overload resolution, polymorphic inference, argument lowering,
//! builtin procedures, macro expansion and operator overloading.
//!
//! Layout:
//! - `mod.rs`        `check_call`, candidate ranking and matching, argument costs
//! - `infer`         `$T` inference and type-pattern matching
//! - `baking`        baked (`$x`) parameter values and constant packs
//! - `emit`          lowering a resolved call and its arguments
//! - `intrinsics`    `#intrinsic` procedures and atomics
//! - `builtins`      built-in procedures (`size_of`, `type_of`, ...)
//! - `macro_expand`  `#expand` macro calls
//! - `operators`     operator overloading
use super::convert;
use super::lower::{FnCtx, MacroFrame, Operand};
use super::procs::{ProcTarget, Signature};
use super::scope::{BuiltinProc, EntityKind, ScopeKind};
use super::*;
use crate::ast::ExprKind as E;
use crate::ir::Ty;
use crate::macros::raw_enum;
use crate::types::{ArrayKind, TypeKind};

mod baking;
mod builtins;
mod emit;
mod infer;
mod intrinsics;
mod macro_expand;
mod operators;

pub use operators::binop_text;

/// How a `$$` parameter is treated at one call.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AutoBake {
    Runtime,
    Baked,
    /// Constant argument with no `Value` (`"str".data`): runtime, but `is_constant` is true.
    ConstRuntime,
}

/// The hidden constant marking parameter `name` as constant in an auto-bake variant.
fn const_marker(name: Sym) -> Sym {
    Sym::intern(&format!("$const:{name}"))
}

/// A call argument: either already checked, or deferred until the parameter
/// type is known (`.MEMBER`, `.{...}`, `xx value`...).
#[derive(Clone)]
pub struct CallArg {
    pub name: Option<Sym>,
    pub spread: bool,
    pub expr: Option<ast::Expr>,
    pub op: Option<Operand>,
    pub span: Span,
    pub scope: ScopeId,
}

/// How a parameter receives its value.
#[derive(Clone, Debug)]
pub(super) enum Slot {
    Arg(usize),
    Variadic(Vec<usize>),
    Spread(usize),
    Default,
}

struct Candidate {
    proc: ProcId,
    slots: Vec<Slot>,
    cost: u32,
    /// How much type structure a polymorphic header pins down (`*$T` beats `$T`).
    specificity: u32,
}

fn is_deferred(expr: &ast::Expr) -> bool {
    matches!(
        &expr.kind,
        E::InferredMember(_)
            | E::StructLit {
                ty: None,
                ..
            }
            | E::ArrayLit {
                ty: None,
                ..
            }
            | E::Cast {
                ty: None,
                ..
            }
            | E::Ifx { .. }
            | E::Lambda { .. }
    ) || inferred_flags(expr)
        || autocast_arithmetic(expr)
        || literal_arithmetic(expr)
}

/// `.{300, -1} * scale`: arithmetic whose left operand is an untyped struct literal builds
/// that literal as the parameter's type, so the operator is looked up on it (ui_builder).
fn literal_arithmetic(expr: &ast::Expr) -> bool {
    match &expr.kind {
        E::Binary(ast::BinOp::Add | ast::BinOp::Sub | ast::BinOp::Mul | ast::BinOp::Div, a, _) => {
            matches!(
                a.kind,
                E::StructLit {
                    ty: None,
                    ..
                }
            ) || literal_arithmetic(a)
        }
        _ => false,
    }
}

/// `xx a + 1`: arithmetic on an autocast and literals takes the parameter's type. Bitwise
/// operators take any other operand (`xx a1 | h1` with flag constants passed as a `u64`).
fn autocast_arithmetic(expr: &ast::Expr) -> bool {
    let (a, b) = match &expr.kind {
        E::Binary(ast::BinOp::Add | ast::BinOp::Sub | ast::BinOp::Mul | ast::BinOp::Div, a, b) => {
            (a, b)
        }
        E::Binary(ast::BinOp::BitOr | ast::BinOp::BitAnd | ast::BinOp::BitXor, a, b) => {
            let bare = |e: &ast::Expr| {
                matches!(
                    e.kind,
                    E::Cast {
                        ty: None,
                        ..
                    }
                )
            };
            if bare(a) || bare(b) {
                return true;
            }
            (a, b)
        }
        _ => return false,
    };
    let autocast = |e: &ast::Expr| {
        matches!(
            e.kind,
            E::Cast {
                ty: None,
                ..
            }
        ) || autocast_arithmetic(e)
    };
    let literal = |e: &ast::Expr| matches!(e.kind, E::Int(_) | E::Float(_) | E::Char(_));
    (autocast(a) && (literal(b) || autocast(b))) || (literal(a) && autocast(b))
}

/// `.A | .B`, `.A & ~.B`: enum flag arithmetic built only from inferred members.
fn inferred_flags(expr: &ast::Expr) -> bool {
    match &expr.kind {
        E::InferredMember(_) => true,
        E::Unary(ast::UnOp::BitNot, x) => inferred_flags(x),
        E::Binary(ast::BinOp::BitOr | ast::BinOp::BitAnd | ast::BinOp::BitXor, a, b) => {
            inferred_flags(a) && inferred_flags(b)
        }
        _ => false,
    }
}

/// Name of the private intrinsic that reads the workspace's temporary-storage size.
pub(super) const RUNTIME_TEMPORARY_STORAGE_SIZE: &str = "__jaic_runtime_temporary_storage_size";

raw_enum! {
    /// The `atomic_*_order` intrinsics, whose trailing parameter is a constant memory order.
    enum OrderedAtomic by name {
        Load = "atomic_load_order",
        Store = "atomic_store_order",
        Exchange = "atomic_exchange_order",
        Add = "atomic_fetch_add_order",
        Sub = "atomic_fetch_sub_order",
        And = "atomic_fetch_and_order",
        Or = "atomic_fetch_or_order",
        Xor = "atomic_fetch_xor_order",
        CompareExchange = "atomic_compare_exchange_order",
    }
}

impl Compiler {
    pub fn check_call(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        callee: &ast::Expr,
        args: &[ast::Arg],
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        if args.iter().any(|a| a.context) {
            // `f(x,, allocator = temp)`: call with a modified copy of the context.
            let (overrides, plain): (Vec<ast::Arg>, Vec<ast::Arg>) =
                args.iter().cloned().partition(|a| a.context);
            let ctx = self.context_with_overrides(f, scope, &overrides, span)?;
            let saved = f.current_context();
            f.set_context(Some(ctx));
            let result = self.check_call(f, scope, callee, &plain, expected, span);
            f.set_context(saved);
            return result;
        }
        let callee_op = self.check_expr(f, scope, callee, None)?;
        match callee_op {
            Operand::Builtin(b) => self.check_builtin(f, scope, b, args, expected, span),
            Operand::PolyStruct(ps) => {
                let mut values = Vec::new();
                for (i, a) in args.iter().enumerate() {
                    let expected = if matches!(a.value.kind, E::InferredMember(_)) {
                        self.poly_struct_param_type(ps, i, a.name.map(|n| n.name))
                    } else {
                        None
                    };
                    values.push((
                        a.name.map(|n| n.name),
                        self.eval_const_or_run(scope, &a.value, expected)?,
                    ));
                }
                let ty = self.instantiate_struct(ps, values, span)?;
                self.ide_note_struct_arguments(ty, args);
                Ok(Operand::Type(ty))
            }
            Operand::Procs(procs) => {
                let as_code = self.macro_code_args(&procs, args);
                let discarded = self.discarded_args(&procs, args);
                let call_args =
                    self.precheck_args_deferring(f, scope, args, &as_code, &discarded)?;
                let wants_code = procs.iter().any(|&p| {
                    let header = &self.proc(p).lit.header;
                    header.params.iter().any(|p| {
                        p.default
                            .as_ref()
                            .is_some_and(|d| matches!(d.kind, E::CallerCode))
                    })
                });
                if !wants_code {
                    return self.call_procs(f, scope, &procs, call_args, expected, span);
                }
                let call = ast::Expr {
                    kind: E::Call {
                        callee: Box::new(callee.clone()),
                        args: args.to_vec(),
                        hint: ast::CallHint::None,
                    },
                    span,
                };
                self.calls_in_flight.push((Rc::new(call), scope));
                let result = self.call_procs(f, scope, &procs, call_args, expected, span);
                self.calls_in_flight.pop();
                result
            }
            Operand::Type(t) => {
                // `Type(x)` is not Jai; but `T.{}` etc are handled elsewhere.
                err(span, format!("cannot call type {}", self.types.name(t)))
            }
            other => {
                let parameters = self.callable_operand_parameters(f, &other)?;
                let (ty, v) = self.rvalue(f, other, span)?;
                let call_args = self.precheck_args(f, scope, args)?;
                self.call_indirect(f, ty, v, parameters, call_args, span)
            }
        }
    }

    /// A copy of the current context with fields replaced; an unnamed
    /// override sets the allocator.
    fn context_with_overrides(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        overrides: &[ast::Arg],
        span: Span,
    ) -> Result<ir::Val> {
        let Some(current) = f.current_context() else {
            return err(
                span,
                "`,,` context arguments need a context (not in #c_call code)",
            );
        };
        let ctx_ty = self.context_type(span)?;
        let copy = self.spill(f, ctx_ty, current, span)?;
        for o in overrides {
            let name = o.name.map_or_else(|| Sym::intern("allocator"), |n| n.name);
            let Some((path, fty)) = self.find_member(ctx_ty, name, o.value.span)? else {
                return err(o.value.span, format!("Context has no member `{name}`"));
            };
            if let Some(n) = o.name
                && self.ide.is_some()
            {
                self.ide_note_member_use(n.span, ctx_ty, name);
                self.ide_note_member_ref(n.span, name, fty);
            }
            let op = self.check_expr(f, scope, &o.value, Some(fty))?;
            let op = self.convert(f, op, fty, o.value.span)?;
            let (_, v) = self.rvalue(f, op, o.value.span)?;
            let addr = self.apply_path(f, copy, &path);
            self.store_value(f, fty, addr, v, o.value.span)?;
        }
        Ok(copy)
    }

    pub(super) fn precheck_args(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        args: &[ast::Arg],
    ) -> Result<Vec<CallArg>> {
        self.precheck_args_deferring(f, scope, args, &[], &[])
    }

    /// Arguments passed to a `#discard` parameter of one of `procs`: they are typechecked
    /// but generate no code.
    fn discarded_args(&self, procs: &[ProcId], args: &[ast::Arg]) -> Vec<bool> {
        args.iter()
            .enumerate()
            .map(|(i, arg)| {
                procs.iter().any(|&p| {
                    let params = &self.proc(p).lit.header.params;
                    let param = match arg.name {
                        Some(n) => params
                            .iter()
                            .find(|p| p.name.map(|pn| pn.name) == Some(n.name)),
                        None => params.get(i),
                    };
                    param.is_some_and(|p| p.discard)
                })
            })
            .collect()
    }

    /// Arguments that a macro among `procs` takes as a `Code` parameter: they are
    /// code, not values, so they are not checked before the call is resolved.
    fn macro_code_args(&mut self, procs: &[ProcId], args: &[ast::Arg]) -> Vec<bool> {
        let mut code = Vec::with_capacity(args.len());
        for (i, arg) in args.iter().enumerate() {
            let mut takes_code = false;
            for &p in procs {
                let lit = self.proc(p).lit.clone();
                let params = &lit.header.params;
                let param = match arg.name {
                    Some(n) => params
                        .iter()
                        .find(|p| p.name.map(|pn| pn.name) == Some(n.name)),
                    None => params.get(i),
                };
                if let Some(param) = param
                    && self.param_is_code(p, param)
                {
                    takes_code = true;
                    break;
                }
            }
            code.push(takes_code);
        }
        code
    }

    /// `precheck_args`, leaving the arguments marked in `defer` unchecked.
    fn precheck_args_deferring(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        args: &[ast::Arg],
        defer: &[bool],
        discard: &[bool],
    ) -> Result<Vec<CallArg>> {
        let mut out = Vec::new();
        for (i, a) in args.iter().enumerate() {
            let op = if is_deferred(&a.value) || defer.get(i).copied().unwrap_or(false) {
                None
            } else if discard.get(i).copied().unwrap_or(false) {
                Some(self.check_expr_no_emit(scope, &a.value)?)
            } else {
                Some(self.check_expr(f, scope, &a.value, None)?)
            };
            out.push(CallArg {
                name: a.name.map(|n| n.name),
                spread: a.spread,
                expr: Some(a.value.clone()),
                op,
                span: a.value.span,
                scope,
            });
        }
        Ok(out)
    }

    /// Match `args` against each of `procs`: the cheapest candidates, and why the rest failed.
    fn rank_candidates(
        &mut self,
        procs: &[ProcId],
        args: &[CallArg],
        span: Span,
    ) -> (Vec<Candidate>, Vec<Diagnostic>) {
        let mut best: Vec<Candidate> = Vec::new();
        let mut errors: Vec<Diagnostic> = Vec::new();
        for &proc in procs {
            match self.match_candidate(proc, args, span) {
                Ok(c) => {
                    if best.first().is_none_or(|b| c.cost < b.cost) {
                        best = vec![c];
                    } else if best[0].cost == c.cost {
                        best.push(c);
                    }
                }
                Err(e) => errors.push(*e),
            }
        }
        (best, errors)
    }

    /// Resolve and emit a call to one of `procs`.
    pub fn call_procs(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        procs: &[ProcId],
        mut args: Vec<CallArg>,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        let _ = expected;
        // A multi-value call result passes its first value; when nothing accepts that, the
        // values expand into several arguments (kept for older corpus code).
        let mut spread_args = None;
        if args.len() == 1
            && let Some(Operand::Multi(values)) = &args[0].op
        {
            let a = args[0].clone();
            let value_arg = |value: &Operand| CallArg {
                name: None,
                spread: false,
                expr: None,
                op: Some(value.clone()),
                span: a.span,
                scope: a.scope,
            };
            spread_args = Some(values.iter().map(value_arg).collect::<Vec<_>>());
            args = vec![CallArg {
                name: a.name,
                ..value_arg(&values[0])
            }];
        }
        let (mut best, mut errors) = self.rank_candidates(procs, &args, span);
        // A candidate's own declaration is wrong, whichever overload the call picks.
        if let Some(i) = errors
            .iter()
            .position(|e| matches!(e.kind, DiagnosticKind::ModifyWithoutPolymorphs))
        {
            return Err(Box::new(errors.swap_remove(i)));
        }
        if best.is_empty()
            && let Some(spread) = spread_args
        {
            let (b, e) = self.rank_candidates(procs, &spread, span);
            if !b.is_empty() {
                (best, errors, args) = (b, e, spread);
            }
        }
        if best.is_empty() {
            if self.ide.is_some() {
                self.ide_note_unmatched_inferred(procs, &args, span);
            }
            let name = procs
                .first()
                .map(|&p| self.proc(p).name.to_string())
                .unwrap_or_default();
            if procs.len() == 1 {
                let error = errors.into_iter().next();
                return Err(Box::new(self.call_mismatch(procs[0], &name, error, span)));
            }
            let mut d = Diagnostic::error(
                span,
                format!("no overload of `{name}` matches these arguments"),
            );
            for (i, &p) in procs.iter().enumerate().take(8) {
                d = d.with_note(
                    self.proc(p).span,
                    errors.get(i).map(|e| e.message.clone()).unwrap_or_default(),
                );
            }
            return Err(Box::new(d));
        }
        if best.len() > 1 {
            // Prefer non-polymorphic and more specific candidates; otherwise take the first.
            best.sort_by_key(|c| {
                (
                    self.proc(c.proc).bindings.is_some() as u32,
                    std::cmp::Reverse(c.specificity),
                )
            });
        }
        // Several equally good candidates: the editor keeps the whole overload set.
        let ambiguous = best.len() > 1 && {
            let key = |c: &Candidate| (self.proc(c.proc).bindings.is_some(), c.specificity);
            key(&best[0]) == key(&best[1])
        };
        let chosen = best.swap_remove(0);
        self.check_format_call(span, chosen.proc, &chosen.slots, &args)?;
        if self.ide.is_some() {
            self.ide_note_call(span, procs, chosen.proc, &chosen.slots, &args, ambiguous);
            if f.pending_for_body
                .as_ref()
                .is_some_and(|body| body.call_span == span)
            {
                self.ide_note_for_keyword(span, chosen.proc, ambiguous);
            }
        }
        self.emit_call(f, scope, chosen, args, span)
    }

    /// Why the only candidate of a call does not accept its arguments: the problem at the
    /// argument it concerns, and where the procedure is declared.
    fn call_mismatch(
        &self,
        proc: ProcId,
        name: &str,
        error: Option<Diagnostic>,
        span: Span,
    ) -> Diagnostic {
        let Some(error) = error else {
            return Diagnostic::error(span, format!("in call to `{name}`"));
        };
        // A name that does not exist is the argument's own problem, not the call's.
        if matches!(error.kind, DiagnosticKind::UnknownIdentifier { .. }) {
            return error;
        }
        let inside = error.span.file == span.file
            && error.span.start >= span.start
            && error.span.end <= span.end;
        let mut d = Diagnostic {
            span: if inside {
                error.span
            } else {
                span
            },
            message: format!("in call to `{name}`: {}", error.message),
            // Structural binding failures retain their machine-readable contract kind.
            kind: if error.kind == DiagnosticKind::InterfaceConstraint {
                error.kind
            } else {
                DiagnosticKind::Other
            },
            ..error
        };
        let header = self.proc(proc).lit.header.span;
        let overlaps =
            header.file == span.file && header.start < span.end && span.start < header.end;
        if header != Span::NONE && !overlaps {
            d.notes.push((header, format!("`{name}` is declared here")));
        }
        d
    }

    /// Check whether `proc` accepts `args`, instantiating polymorphic procedures.
    fn match_candidate(&mut self, proc: ProcId, args: &[CallArg], span: Span) -> Result<Candidate> {
        let header = self.proc(proc).lit.header.clone();
        self.refresh_implicit_poly(proc)?;
        let poly_vars = if self.proc(proc).is_poly {
            header_poly_names(&header)
        } else {
            Vec::new()
        };
        let slots = assign_slots(&header.params, args, &poly_vars, span)?;
        // `$$x` parameters receiving a constant argument: match a variant with them baked.
        if header.params.iter().any(|p| p.auto_bake) {
            let mut mask = vec![AutoBake::Runtime; header.params.len()];
            for (i, p) in header.params.iter().enumerate() {
                if !p.auto_bake {
                    continue;
                }
                // An omitted argument takes the default, baked when that is a constant.
                if matches!(slots[i], Slot::Default) {
                    if let Some(default) = &p.default
                        && self.default_is_constant(proc, default)
                    {
                        mask[i] = AutoBake::Baked;
                    }
                    continue;
                }
                let Slot::Arg(a) = &slots[i] else {
                    continue;
                };
                let arg = &args[*a];
                mask[i] = match &arg.op {
                    Some(op) if op.is_const() || matches!(op, Operand::Procs(_)) => AutoBake::Baked,
                    Some(op) => match &arg.expr {
                        Some(e) if self.is_constant_pointer(arg.scope, e, op) => {
                            AutoBake::ConstRuntime
                        }
                        _ => AutoBake::Runtime,
                    },
                    None => AutoBake::Runtime,
                };
            }
            if mask.iter().any(|&b| b != AutoBake::Runtime) {
                let variant = self.auto_bake_variant(proc, mask);
                return self.match_candidate(variant, args, span);
            }
        }
        let mut proc_id = proc;
        let mut extra = 0;
        let mut specificity = 0;
        if self.proc(proc).is_poly {
            specificity = header
                .params
                .iter()
                .filter_map(|p| p.ty.as_ref())
                .map(pattern_specificity)
                .sum();
            let bindings = self.infer_bindings(proc, &header, &slots, args, span)?;
            proc_id = self.instantiate(proc, bindings, span)?;
            let made = self.export.call_instances.entry(span).or_default();
            if !made.contains(&proc_id) {
                made.push(proc_id);
            }
            extra = 1 + self.as_base_matches(proc, &header, &slots, args)? * convert::SUBTYPE;
        }
        let sig = self.signature(proc_id, span)?;
        let runtime_params: Vec<usize> = (0..header.params.len())
            .filter(|&i| !header.params[i].baked)
            .collect();
        if runtime_params.len() != sig.params.len() {
            return err(span, "internal: signature/parameter mismatch");
        }
        let mut cost = extra;
        let is_macro = self.proc(proc).is_macro;
        for (k, &i) in runtime_params.iter().enumerate() {
            let param = &sig.params[k];
            if sig.c_varargs
                && param.variadic
                && match &slots[i] {
                    Slot::Spread(_) => true,
                    Slot::Variadic(list) => list.iter().any(|&a| args[a].spread),
                    _ => false,
                }
            {
                return err(span, "a runtime spread cannot determine a C variadic ABI");
            }
            match &slots[i] {
                Slot::Default => {
                    if param.default.is_none() && !param.variadic {
                        let name = param.name.map(|n| n.to_string()).unwrap_or_default();
                        return Err(Box::new(
                            Diagnostic::error(
                                span,
                                format!("missing argument for parameter `{name}`"),
                            )
                            .with_help(format!(
                                "pass a value for `{name}`, or give the parameter a default value (`{name} := ...`)"
                            )),
                        ));
                    }
                }
                Slot::Arg(a) => cost += self.arg_cost(&args[*a], param.ty, is_macro)?,
                Slot::Spread(a) => cost += self.arg_cost(&args[*a], param.ty, is_macro)?,
                Slot::Variadic(list) => {
                    let elem = if sig.c_varargs {
                        param.ty
                    } else {
                        self.jai_variadic_element(param.ty, span)?
                    };
                    for &a in list {
                        if sig.c_varargs {
                            continue;
                        }
                        let target = if args[a].spread {
                            param.ty
                        } else {
                            elem
                        };
                        cost += self.arg_cost(&args[a], target, is_macro)?;
                    }
                }
            }
        }
        Ok(Candidate {
            proc: proc_id,
            slots,
            cost,
            specificity,
        })
    }

    /// How many arguments matched a bare polymorphic struct parameter (`r: *Reflector`)
    /// only through an `#as` member. The parameter takes the argument's own type, but the
    /// match costs like the `#as` conversion it stands for, so an overload taking the
    /// argument's type directly (`r: *$R`) still wins (n00bmind/reflector's overload test).
    fn as_base_matches(
        &mut self,
        proc: ProcId,
        header: &ast::ProcHeader,
        slots: &[Slot],
        args: &[CallArg],
    ) -> Result<u32> {
        let scope = self.proc(proc).scope;
        let mut count = 0;
        for (i, param) in header.params.iter().enumerate() {
            let (Some(pattern), Slot::Arg(a)) = (&param.ty, &slots[i]) else {
                continue;
            };
            let Some(op) = &args[*a].op else {
                continue;
            };
            let mut pattern = pattern;
            let mut ty = op.ty();
            while let E::Unary(ast::UnOp::Star, inner) = &pattern.kind {
                let Some(p) = self.types.pointee(ty) else {
                    break;
                };
                pattern = inner;
                ty = p;
            }
            let E::Ident(name) = &pattern.kind else {
                continue;
            };
            if let Some(ps) = self.ident_poly_struct(scope, *name)?
                && let Some(base) = self.instance_or_as_base(ps, ty)
                && base != ty
            {
                count += 1;
            }
        }
        Ok(count)
    }

    /// Whether a `$$` parameter's default folds to a constant without running code
    /// (`$$must := false`), checked in the procedure's own scope.
    fn default_is_constant(&mut self, proc: ProcId, default: &ast::Expr) -> bool {
        let scope = {
            let p = self.proc(proc);
            p.bindings.unwrap_or(p.scope)
        };
        let file = self.scope_file(scope);
        let mut f = self.thunk_ctx(ir::FuncOrigin::ConstInit, file);
        let scope = self.thunk_scope(scope);
        matches!(self.check_expr(&mut f, scope, default, None), Ok(op) if op.is_const())
    }

    /// The copy of `proc` with the `$$` parameters flagged in `mask` turned into `$` ones.
    fn auto_bake_variant(&mut self, proc: ProcId, mask: Vec<AutoBake>) -> ProcId {
        if let Some(&v) = self.auto_bake_variants.get(&(proc, mask.clone())) {
            return v;
        }
        let p = self.proc(proc);
        let (name, lit, scope, span) =
            (p.name, p.lit.clone(), p.bindings.unwrap_or(p.scope), p.span);
        let mut header = (*lit.header).clone();
        let mut consts = Vec::new();
        for (param, &bake) in header.params.iter_mut().zip(&mask) {
            match bake {
                AutoBake::Baked => {
                    param.baked = true;
                    param.auto_bake = false;
                }
                AutoBake::ConstRuntime => {
                    param.auto_bake = false;
                    if let Some(n) = param.name {
                        consts.push((const_marker(n.name), Value::Bool(true), TypeId::BOOL));
                    }
                }
                AutoBake::Runtime => {}
            }
        }
        header.flags.program_export = None;
        let lit = Rc::new(ast::ProcLit {
            header: Rc::new(header),
            body: lit.body.clone(),
        });
        let scope = if consts.is_empty() {
            scope
        } else {
            self.const_scope(scope, consts, span)
        };
        let v = self.new_proc(name, lit, scope, span);
        self.auto_bake_variants.insert((proc, mask), v);
        v
    }

    /// Constant expressions that have no `Value` to bake (addresses of globals, `.data` of
    /// constants, `type_info(T)`...). `is_constant` is true for them; a `$$` parameter
    /// receiving one stays a runtime parameter, marked constant.
    pub(super) fn is_constant_pointer(
        &mut self,
        scope: ScopeId,
        expr: &ast::Expr,
        op: &Operand,
    ) -> bool {
        match &expr.kind {
            E::Ident(name) => self
                .lookup(scope, const_marker(*name))
                .is_ok_and(|v| !v.is_empty()),
            E::Member(base, member) => match member.name.as_str() {
                "data" if self.types.is_pointer(op.ty()) => {
                    let Ok(b) = self.check_expr_no_emit(scope, base) else {
                        return false;
                    };
                    let fixed = matches!(
                        self.types.kind(self.types.repr(b.ty())),
                        TypeKind::Array {
                            kind: ArrayKind::Fixed(_),
                            ..
                        }
                    );
                    b.is_const() || (fixed && self.is_global_var(scope, base))
                }
                // `type_info(T).type` / `.runtime_size` are constant.
                "type" | "runtime_size" => {
                    matches!(&base.kind, E::Call { callee, .. }
                        if matches!(&callee.kind, E::Ident(n) if n.as_str() == "type_info"))
                }
                _ => false,
            },
            E::Unary(ast::UnOp::Star, inner) => self.is_global_var(scope, inner),
            E::Cast {
                value, ..
            } if self.types.is_pointer(op.ty()) => self.is_constant_expr(scope, value),
            E::Binary(ast::BinOp::Add | ast::BinOp::Sub, a, b)
                if self.types.is_pointer(op.ty()) =>
            {
                self.is_constant_expr(scope, a) && self.is_constant_expr(scope, b)
            }
            E::Index(base, index) => {
                let Ok(b) = self.check_expr_no_emit(scope, base) else {
                    return false;
                };
                matches!(
                    self.types.kind(self.types.repr(b.ty())),
                    TypeKind::Array { .. } | TypeKind::String
                ) && self.is_constant_expr(scope, base)
                    && self.is_constant_expr(scope, index)
            }
            E::Call {
                callee, ..
            } => matches!(&callee.kind, E::Ident(n)
                if matches!(n.as_str(), "type_info" | "initializer_of")),
            _ => false,
        }
    }

    fn is_constant_expr(&mut self, scope: ScopeId, expr: &ast::Expr) -> bool {
        let Ok(op) = self.check_expr_no_emit(scope, expr) else {
            return false;
        };
        op.is_const()
            || matches!(op, Operand::Procs(_))
            || self.is_constant_pointer(scope, expr, &op)
    }

    /// Is `expr` the name of a global variable (not a local or constant)?
    fn is_global_var(&mut self, scope: ScopeId, expr: &ast::Expr) -> bool {
        let E::Ident(name) = &expr.kind else {
            return false;
        };
        let Ok(ids) = self.lookup(scope, *name) else {
            return false;
        };
        matches!(ids[..], [id] if matches!(
            &self.entity(id).kind,
            EntityKind::Decl { decl, .. } if decl.kind == ast::DeclKind::Var
        ))
    }

    /// A `*Struct` argument passed to a by-value `Struct` parameter is dereferenced.
    fn auto_deref_arg(&self, from: TypeId, param: TypeId) -> bool {
        self.types.pointee(from) == Some(param) && self.types.as_struct(param).is_some()
    }

    /// The operand an `ifx` argument offers to polymorphic inference: its first branch with a
    /// type of its own (`ifx c then x else y` with `x: s16` binds `$T` to `s16`), else its first
    /// branch that checks at all (an untyped constant, defaulted like a plain literal argument).
    fn ifx_binding_operand(&mut self, arg: &CallArg) -> Option<Operand> {
        let expr = arg.expr.as_ref()?;
        let mut fallback = None;
        let typed = self.ifx_branch_operand(arg.scope, expr, &mut fallback);
        typed.or(fallback)
    }

    /// [`Self::ifx_binding_operand`] for one `ifx` expression. A branch that is itself an `ifx`
    /// (`ifx a then (ifx b then x else y) else z`) offers its own branches, so nesting binds too.
    fn ifx_branch_operand(
        &mut self,
        scope: ScopeId,
        expr: &ast::Expr,
        fallback: &mut Option<Operand>,
    ) -> Option<Operand> {
        let E::Ifx {
            then_value,
            else_value,
            ..
        } = &expr.kind
        else {
            return None;
        };
        for branch in [then_value, else_value].into_iter().flatten() {
            if matches!(branch.kind, E::Ifx { .. }) {
                if let Some(op) = self.ifx_branch_operand(scope, branch, fallback) {
                    return Some(op);
                }
                continue;
            }
            if is_deferred(branch) {
                continue;
            }
            let Ok(op) = self.check_expr_no_emit(scope, branch) else {
                continue;
            };
            match op {
                Operand::Const {
                    untyped: true, ..
                } => {
                    fallback.get_or_insert(op);
                }
                Operand::Value {
                    ..
                }
                | Operand::Place {
                    ..
                }
                | Operand::Const {
                    ..
                } => {
                    return Some(op);
                }
                _ => {}
            }
        }
        None
    }

    /// `macro_call`: a Code parameter of a macro also binds a variable by name.
    fn arg_cost(&mut self, arg: &CallArg, param: TypeId, macro_call: bool) -> Result<u32> {
        if self.is_register_operand_type(param, arg.span) {
            if !macro_call {
                return err(arg.span, "register operands require an expanded procedure");
            }
            if let Some(expr) = &arg.expr
                && self.asm_reg_named(arg.scope, expr)?.is_some()
            {
                return Ok(convert::EXACT);
            }
            return err(
                arg.span,
                "expected an #asm register operand, not a scalar or memory value",
            );
        }

        let Some(op) = &arg.op else {
            if let Some(ast::Expr {
                kind: E::Lambda {
                    header, ..
                },
                ..
            }) = &arg.expr
            {
                match self.types.kind(param) {
                    TypeKind::Proc(pt) if pt.params.len() == header.params.len() => {}
                    TypeKind::Proc(pt) => {
                        return err(
                            arg.span,
                            format!(
                                "lambda takes {} parameters, but {} expects {}",
                                header.params.len(),
                                self.types.name(param),
                                pt.params.len()
                            ),
                        );
                    }
                    _ => {
                        return err(
                            arg.span,
                            format!("a lambda cannot be passed as {}", self.types.name(param)),
                        );
                    }
                }
            }
            // Any expression converts to a `Code` parameter (plain procedures too).
            if param == TypeId::CODE {
                return Ok(convert::LITERAL);
            }
            // `ifx c then a else b`: each branch must fit (`"-->"` does not fit a `u8`).
            if let Some(ast::Expr {
                kind:
                    E::Ifx {
                        then_value,
                        else_value,
                        ..
                    },
                ..
            }) = &arg.expr
            {
                let mut cost = convert::LITERAL;
                for branch in [then_value, else_value].into_iter().flatten() {
                    if is_deferred(branch) {
                        continue;
                    }
                    let Ok(op) = self.check_expr_no_emit(arg.scope, branch) else {
                        continue;
                    };
                    let probe = CallArg {
                        op: Some(op),
                        expr: Some((**branch).clone()),
                        span: branch.span,
                        ..arg.clone()
                    };
                    cost = cost.max(self.arg_cost(&probe, param, macro_call)?);
                }
                return Ok(cost);
            }
            // Deferred arguments fit any plausible target; prefer exact-looking ones.
            let scalar = matches!(
                self.types.kind(self.types.repr_struct(param)),
                TypeKind::Bool
                    | TypeKind::Int { .. }
                    | TypeKind::Float { .. }
                    | TypeKind::String
                    | TypeKind::Type
            );
            let fits = match arg.expr.as_ref().map(|e| &e.kind) {
                // `.{...}` and `.[...]` build aggregates, `.NAME` picks an enum member.
                Some(E::StructLit {
                    ..
                }) => {
                    !scalar
                        || !self
                            .builtin_members(self.types.repr(param), arg.span)?
                            .is_empty()
                }
                Some(E::ArrayLit {
                    ..
                }) => !scalar,
                Some(_) if arg.expr.as_ref().is_some_and(inferred_flags) => matches!(
                    self.types.kind(param),
                    TypeKind::Enum(_) | TypeKind::Struct(_)
                ),
                _ => true,
            };
            if !fits {
                return err(
                    arg.span,
                    format!(
                        "argument cannot be inferred as parameter type {}",
                        self.types.name(param)
                    ),
                );
            }
            return Ok(convert::LITERAL);
        };
        let untyped = matches!(
            op,
            Operand::Const {
                untyped: true,
                ..
            }
        );
        match op {
            Operand::Void => return err(arg.span, "the argument has no value"),
            Operand::Type(_) | Operand::PolyStruct(_) if param == TypeId::TYPE => {
                return Ok(convert::EXACT);
            }
            Operand::PolyStruct(_) if param == TypeId::ANY => return Ok(convert::TO_ANY),
            Operand::Type(_) if param == TypeId::ANY => return Ok(convert::TO_ANY),
            Operand::Procs(procs) => {
                if let TypeKind::Proc(_) = self.types.kind(param) {
                    for &p in procs.clone().iter() {
                        if !self.proc(p).is_poly && self.proc_type(p, arg.span)? == param {
                            return Ok(convert::EXACT);
                        }
                    }
                    for &p in procs.clone().iter() {
                        if self.proc(p).is_poly
                            && self.instantiate_for_proc_type(p, param, arg.span).is_some()
                        {
                            return Ok(convert::WIDEN);
                        }
                    }
                    if procs.len() == 1 && !self.proc(procs[0]).is_poly {
                        let pt = self.proc_type(procs[0], arg.span)?;
                        if let (TypeKind::Proc(a), TypeKind::Proc(b)) =
                            (self.types.kind(pt), self.types.kind(param))
                            && a.params == b.params
                            && a.returns == b.returns
                        {
                            return Ok(convert::WIDEN);
                        }
                    }
                    return err(
                        arg.span,
                        format!(
                            "procedure does not match parameter type {}",
                            self.types.name(param)
                        ),
                    );
                }
                if param == TypeId::VOID_PTR || param == TypeId::ANY {
                    return Ok(convert::POINTER);
                }
            }
            // `split(s, ".")` into a `u8` parameter: a one-character string is that byte
            // (`convert` makes the value). A `string` overload is still the closer match.
            Operand::Const {
                value: Value::String(s),
                ..
            } if s.len() == 1 && param == TypeId::U8 => return Ok(convert::WIDEN),
            Operand::Const {
                value: Value::Int(v),
                untyped: true,
                ..
            } if self.types.is_integer(param) => {
                let (bits, signed) = self.types.int_info(param).unwrap();
                if !convert::int_constant_fits(*v, bits, signed, self.bit_pattern(arg.span)) {
                    return err(
                        arg.span,
                        format!("constant {v} does not fit in {}", self.types.name(param)),
                    );
                }
                // Jai lets an untyped integer constant (`#char ","`, `3`) stand for an
                // enum value; rank it below every ordinary integer conversion.
                if matches!(self.types.kind(param), TypeKind::Enum(_))
                    && !self.types.is_loose_enum(param)
                {
                    return Ok(convert::SUBTYPE);
                }
                return Ok(if param == TypeId::S64 {
                    convert::EXACT
                } else {
                    convert::LITERAL
                });
            }
            Operand::Const {
                value: Value::Float(_, precision),
                untyped: true,
                ..
            } if precision.requires_float64 && self.types.repr(param) == TypeId::F32 => {
                return err(
                    arg.span,
                    "this floating literal requires float64 precision (use an explicit cast to round it)",
                );
            }
            // A string literal is NUL-terminated, so it converts to a C string.
            Operand::Const {
                value: Value::String(_),
                ..
            } if self.types.pointee(param) == Some(TypeId::U8) => return Ok(convert::POINTER),
            // A string literal converts to a `#type,distinct` / `#type,isa` string variant.
            Operand::Const {
                value: Value::String(_),
                ..
            } if matches!(self.types.kind(param), TypeKind::Distinct(_))
                && self.types.repr(param) == TypeId::STRING =>
            {
                return Ok(convert::LITERAL);
            }
            _ => {}
        }
        let from = op.ty();
        if from == TypeId::F32 && untyped && param == TypeId::F32 {
            return Ok(convert::EXACT);
        }
        if self.auto_deref_arg(from, param) {
            return Ok(convert::SUBTYPE);
        }
        // Code macro parameters may quote a caller variable by name.
        if param == TypeId::CODE
            && macro_call
            && matches!(op, Operand::Place { .. })
            && matches!(arg.expr.as_ref().map(|e| &e.kind), Some(E::Ident(_)))
            && self.implicit_cost(from, untyped, param).is_none()
        {
            return Ok(convert::LITERAL);
        }
        self.implicit_cost(from, untyped, param).ok_or_else(|| {
            let mut d = Diagnostic::error(
                arg.span,
                format!(
                    "argument of type `{}` does not match parameter type `{}`",
                    self.types.name(from),
                    self.types.name(param)
                ),
            );
            if let Some(help) = self.conversion_help(from, param) {
                d = d.with_help(help);
            }
            Box::new(d)
        })
    }
}

/// C default argument promotions for variadic arguments.
fn c_vararg_promote(c: &mut Compiler, f: &mut FnCtx, op: Operand, span: Span) -> Result<Operand> {
    let ty = op.ty();
    if ty == TypeId::F32 {
        return c.convert(f, op, TypeId::F64, span);
    }
    if let Some((bits, signed)) = c.types.int_info(ty)
        && bits < 32
    {
        let target = if signed {
            TypeId::S32
        } else {
            TypeId::U32
        };
        return c.explicit_cast(f, op, target, ast::CastFlags::default(), span);
    }
    if ty == TypeId::BOOL {
        return c.explicit_cast(f, op, TypeId::S32, ast::CastFlags::default(), span);
    }
    Ok(op)
}

/// Polymorphic names introduced by a type pattern.
fn poly_names(expr: &ast::Expr) -> Vec<Sym> {
    let mut out = Vec::new();
    fn walk(e: &ast::Expr, out: &mut Vec<Sym>) {
        match &e.kind {
            E::PolyVar {
                name, ..
            }
            | E::PolyRestricted {
                name, ..
            } => out.push(*name),
            E::Unary(_, x) => walk(x, out),
            E::ArrayType {
                size,
                elem,
            } => {
                if let ast::ArraySize::Fixed(n) = size {
                    walk(n, out);
                }
                walk(elem, out);
            }
            E::Call {
                args, ..
            } => args.iter().for_each(|a| walk(&a.value, out)),
            E::ProcType(h) => {
                h.params
                    .iter()
                    .filter_map(|p| p.ty.as_ref())
                    .for_each(|t| walk(t, out));
                h.returns
                    .iter()
                    .filter_map(|r| r.ty.as_ref())
                    .for_each(|t| walk(t, out));
            }
            _ => {}
        }
    }
    walk(expr, &mut out);
    out
}

/// Type constructors wrapped around the polymorphic variables of a parameter type.
fn pattern_specificity(expr: &ast::Expr) -> u32 {
    match &expr.kind {
        E::PolyRestricted {
            ..
        } => 1,
        E::Unary(_, x) => 1 + pattern_specificity(x),
        E::ArrayType {
            elem, ..
        } => 1 + pattern_specificity(elem),
        E::Call {
            args, ..
        } => {
            1 + args
                .iter()
                .map(|a| pattern_specificity(&a.value))
                .sum::<u32>()
        }
        E::ProcType(_) => 1,
        _ => 0,
    }
}

/// Map call arguments onto declared parameters.
/// Every `$T` binder in a header's parameter and result types.
pub(super) fn header_poly_names(header: &ast::ProcHeader) -> Vec<Sym> {
    let mut names = Vec::new();
    for t in header
        .params
        .iter()
        .filter_map(|p| p.ty.as_ref())
        .chain(header.returns.iter().filter_map(|r| r.ty.as_ref()))
    {
        names.extend(poly_names(t));
    }
    names
}

/// A named argument binding a polymorphic variable (`f(x, T = float64)`).
fn is_poly_var_arg(params: &[ast::Param], arg: &CallArg, poly_vars: &[Sym]) -> bool {
    arg.name.is_some_and(|name| {
        poly_vars.contains(&name) && !params.iter().any(|p| p.name.map(|n| n.name) == Some(name))
    })
}

fn assign_slots(
    params: &[ast::Param],
    args: &[CallArg],
    poly_vars: &[Sym],
    span: Span,
) -> Result<Vec<Slot>> {
    let mut slots: Vec<Option<Slot>> = vec![None; params.len()];
    let variadic_index = params.iter().position(|p| p.variadic);
    let mut positional = 0usize;
    for (i, arg) in args.iter().enumerate() {
        if is_poly_var_arg(params, arg, poly_vars) {
            continue;
        }
        if let Some(name) = arg.name {
            let Some(p) = params
                .iter()
                .position(|p| p.name.map(|n| n.name) == Some(name))
            else {
                let names: Vec<&str> = params
                    .iter()
                    .filter_map(|p| p.name.map(|n| n.name.as_str()))
                    .collect();
                let help = match crate::suggest::closest(name.as_str(), names.iter().copied()) {
                    Some(near) => format!("did you mean `{near}`?"),
                    None if names.is_empty() => "this procedure has no named parameters".into(),
                    None => format!(
                        "its parameters are {}",
                        names
                            .iter()
                            .map(|n| format!("`{n}`"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                };
                return Err(Box::new(
                    Diagnostic::error(arg.span, format!("no parameter named `{name}`"))
                        .with_help(help),
                ));
            };
            if slots[p].is_some() {
                return Err(Box::new(
                    Diagnostic::error(arg.span, format!("parameter `{name}` given twice"))
                        .with_help("remove one of the two arguments"),
                ));
            }
            slots[p] = Some(if arg.spread {
                Slot::Spread(i)
            } else if Some(p) == variadic_index {
                // `v = a, b, c`: later positional arguments extend the list.
                Slot::Variadic(vec![i])
            } else {
                Slot::Arg(i)
            });
            // Positional arguments after a named one continue from the next parameter.
            positional = if Some(p) == variadic_index {
                p
            } else {
                p + 1
            };
            continue;
        }
        while positional < params.len()
            && slots[positional].is_some()
            && Some(positional) != variadic_index
        {
            positional += 1;
        }
        if positional >= params.len() {
            return Err(Box::new(
                Diagnostic::error(
                    arg.span,
                    format!("too many arguments: it takes at most {}", params.len()),
                )
                .with_label("this argument has no parameter"),
            ));
        }
        if Some(positional) == variadic_index {
            match &mut slots[positional] {
                Some(Slot::Variadic(list)) => list.push(i),
                Some(Slot::Spread(first)) => {
                    slots[positional] = Some(Slot::Variadic(vec![*first, i]))
                }
                None => {
                    slots[positional] = Some(if arg.spread {
                        Slot::Spread(i)
                    } else {
                        Slot::Variadic(vec![i])
                    })
                }
                _ => return err(arg.span, "variadic parameter already given"),
            }
            if arg.spread && positional + 1 < params.len() {
                positional += 1;
            }
            continue;
        }
        slots[positional] = Some(if arg.spread {
            Slot::Spread(i)
        } else {
            Slot::Arg(i)
        });
        positional += 1;
    }
    let _ = span;
    Ok(slots
        .into_iter()
        .map(|s| s.unwrap_or(Slot::Default))
        .collect())
}
