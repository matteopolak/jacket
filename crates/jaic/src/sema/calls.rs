//! Calls: overload resolution, polymorphic inference, argument lowering,
//! builtin procedures, macro expansion and operator overloading.
use super::convert;
use super::lower::{FnCtx, MacroFrame, Operand};
use super::procs::{ProcTarget, Signature};
use super::scope::{BuiltinProc, EntityKind, ScopeKind};
use super::*;
use crate::ast::ExprKind as E;
use crate::ir::Ty;
use crate::types::{ArrayKind, TypeKind};

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
enum Slot {
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
}

/// `xx a + 1`: arithmetic on an autocast and literals takes the parameter's type.
fn autocast_arithmetic(expr: &ast::Expr) -> bool {
    let E::Binary(ast::BinOp::Add | ast::BinOp::Sub | ast::BinOp::Mul | ast::BinOp::Div, a, b) =
        &expr.kind
    else {
        return false;
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
            let saved = f.context.replace(ctx);
            let result = self.check_call(f, scope, callee, &plain, expected, span);
            f.context = saved;
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
                Ok(Operand::Type(self.instantiate_struct(ps, values, span)?))
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
                let (ty, v) = self.rvalue(f, other, span)?;
                let call_args = self.precheck_args(f, scope, args)?;
                self.call_indirect(f, ty, v, call_args, span)
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
        let Some(current) = f.context else {
            return err(
                span,
                "',,' context arguments need a context (not in #c_call code)",
            );
        };
        let ctx_ty = self.context_type(span)?;
        let copy = self.spill(f, ctx_ty, current, span)?;
        for o in overrides {
            let name = o.name.map_or_else(|| Sym::intern("allocator"), |n| n.name);
            let Some((path, fty)) = self.find_member(ctx_ty, name, o.value.span)? else {
                return err(o.value.span, format!("Context has no member '{name}'"));
            };
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
    fn macro_code_args(&self, procs: &[ProcId], args: &[ast::Arg]) -> Vec<bool> {
        let is_code = |p: &ast::Param| matches!(&p.ty, Some(ast::Expr { kind: E::Ident(n), .. }) if n.as_str() == "Code");
        args.iter()
            .enumerate()
            .map(|(i, arg)| {
                procs.iter().any(|&p| {
                    let info = self.proc(p);
                    let params = &info.lit.header.params;
                    let param = match arg.name {
                        Some(n) => params
                            .iter()
                            .find(|p| p.name.map(|pn| pn.name) == Some(n.name)),
                        None => params.get(i),
                    };
                    param.is_some_and(is_code)
                })
            })
            .collect()
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
    ) -> (Vec<Candidate>, Vec<String>) {
        let mut best: Vec<Candidate> = Vec::new();
        let mut errors: Vec<String> = Vec::new();
        for &proc in procs {
            match self.match_candidate(proc, args, span) {
                Ok(c) => {
                    if best.first().is_none_or(|b| c.cost < b.cost) {
                        best = vec![c];
                    } else if best[0].cost == c.cost {
                        best.push(c);
                    }
                }
                Err(e) => errors.push(e.message.clone()),
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
            let value_arg = |&(ty, val): &(TypeId, ir::Val)| CallArg {
                name: None,
                spread: false,
                expr: None,
                op: Some(Operand::Value {
                    ty,
                    val,
                }),
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
        if best.is_empty()
            && let Some(spread) = spread_args
        {
            let (b, e) = self.rank_candidates(procs, &spread, span);
            if !b.is_empty() {
                (best, errors, args) = (b, e, spread);
            }
        }
        if best.is_empty() {
            let name = procs
                .first()
                .map(|&p| self.proc(p).name.to_string())
                .unwrap_or_default();
            if procs.len() == 1 {
                return err(span, format!("in call to '{name}': {}", errors.join("; ")));
            }
            let mut d = Diagnostic::error(
                span,
                format!("no overload of '{name}' matches these arguments"),
            );
            for (i, &p) in procs.iter().enumerate().take(8) {
                d = d.with_note(
                    self.proc(p).span,
                    errors.get(i).cloned().unwrap_or_default(),
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
        let chosen = best.swap_remove(0);
        self.emit_call(f, scope, chosen, args, span)
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
                let Slot::Arg(a) = &slots[i] else {
                    continue;
                };
                if !p.auto_bake {
                    continue;
                }
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
            extra = 1;
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
            match &slots[i] {
                Slot::Default => {
                    if param.default.is_none() && !param.variadic {
                        let name = param.name.map(|n| n.to_string()).unwrap_or_default();
                        return err(span, format!("missing argument for parameter '{name}'"));
                    }
                }
                Slot::Arg(a) => cost += self.arg_cost(&args[*a], param.ty, is_macro)?,
                Slot::Spread(a) => cost += self.arg_cost(&args[*a], param.ty, is_macro)?,
                Slot::Variadic(list) => {
                    let elem = match self.types.kind(param.ty) {
                        TypeKind::Array {
                            elem, ..
                        } => *elem,
                        _ => param.ty,
                    };
                    for &a in list {
                        if sig.c_varargs {
                            continue;
                        }
                        cost += self.arg_cost(&args[a], elem, is_macro)?;
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

    /// `macro_call`: a Code parameter of a macro also binds a variable by name.
    fn arg_cost(&mut self, arg: &CallArg, param: TypeId, macro_call: bool) -> Result<u32> {
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
                Some(
                    E::StructLit {
                        ..
                    }
                    | E::ArrayLit {
                        ..
                    },
                ) => !scalar,
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
            Operand::Type(_) if param == TypeId::TYPE => return Ok(convert::EXACT),
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
            Operand::Const {
                value: Value::Int(v),
                untyped: true,
                ..
            } if self.types.is_integer(param) => {
                let (bits, signed) = self.types.int_info(param).unwrap();
                if !convert::int_fits(*v, bits, signed) {
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
        // `__reg` (Code) macro parameters bind to the caller's variable by name.
        if param == TypeId::CODE
            && macro_call
            && matches!(op, Operand::Place { .. })
            && matches!(arg.expr.as_ref().map(|e| &e.kind), Some(E::Ident(_)))
            && self.implicit_cost(from, untyped, param).is_none()
        {
            return Ok(convert::LITERAL);
        }
        self.implicit_cost(from, untyped, param).ok_or_else(|| {
            Box::new(Diagnostic::error(
                arg.span,
                format!(
                    "argument of type {} does not match parameter type {}",
                    self.types.name(from),
                    self.types.name(param)
                ),
            ))
        })
    }

    /// Infer `$T` bindings of a polymorphic procedure from its arguments.
    fn infer_bindings(
        &mut self,
        proc: ProcId,
        header: &ast::ProcHeader,
        slots: &[Slot],
        args: &[CallArg],
        span: Span,
    ) -> Result<Vec<(Sym, Value, TypeId)>> {
        let mut bindings: Vec<(Sym, Value, TypeId)> = Vec::new();
        let mut deferred_procs = Vec::new();
        let mut null_patterns: Vec<ast::Expr> = Vec::new();
        let def_scope = self.proc(proc).scope;
        let poly_vars = header_poly_names(header);
        for arg in args {
            if !is_poly_var_arg(&header.params, arg, &poly_vars) {
                continue;
            }
            let name = arg.name.unwrap();
            let op = match (&arg.op, &arg.expr) {
                (Some(op), _) => op.clone(),
                (None, Some(e)) => self.eval_const(arg.scope, e, None)?,
                (None, None) => return err(arg.span, "missing value"),
            };
            let Operand::Type(t) = op else {
                return err(arg.span, format!("'${name}' must be given a type"));
            };
            bindings.push((name, Value::Type(t), TypeId::TYPE));
        }
        for (i, param) in header.params.iter().enumerate() {
            let arg_ops: Vec<&CallArg> = match &slots[i] {
                Slot::Arg(a) | Slot::Spread(a) => vec![&args[*a]],
                Slot::Variadic(list) => list.iter().map(|&a| &args[a]).collect(),
                Slot::Default => Vec::new(),
            };
            if param.baked && param.variadic {
                let name = param.name.map(|n| n.name).unwrap();
                // `..names` passes on a constant view (another baked variadic) as it is.
                if let Slot::Spread(a) = &slots[i] {
                    let (value, ty) = self.baked_spread(def_scope, param, &args[*a])?;
                    bindings.push((name, value, ty));
                    continue;
                }
                let (value, ty) = self.baked_pack(def_scope, param, &arg_ops)?;
                bindings.push((name, value, ty));
                continue;
            }
            if param.baked {
                let name = param.name.map(|n| n.name).unwrap();
                let Some(arg) = arg_ops.first() else {
                    if let Some(d) = &param.default {
                        // `$type: Query = .X`: the default takes the declared type. A type
                        // naming earlier bindings (`$compare: (T, T) -> bool = ...`) is read
                        // in a scope holding them.
                        let mut scope = def_scope;
                        if !bindings.is_empty() {
                            // One scope per set of bindings, so a lambda default
                            // (`$compare := (a, b) => a == b`) is the same procedure on every
                            // call and the instance is found again.
                            let key = (def_scope, bindings.iter().map(|b| b.1.clone()).collect());
                            scope = match self.default_scopes.get(&key) {
                                Some(&s) => s,
                                None => {
                                    let module = self.scope(def_scope).module;
                                    let s = self.new_scope(
                                        ScopeKind::Block,
                                        Some(def_scope),
                                        module,
                                        None,
                                    );
                                    for (n, v, t) in &bindings {
                                        self.add_const(s, *n, span, v.clone(), *t);
                                    }
                                    self.default_scopes.insert(key, s);
                                    s
                                }
                            };
                        }
                        let declared = match &param.ty {
                            Some(t) if !procs::has_poly(t) => Some(self.eval_type(scope, t)?),
                            _ => None,
                        };
                        let (v, ty) = match declared {
                            Some(t) => (self.const_value_of_type(scope, d, t)?, t),
                            None => {
                                let ty = self.eval_const(scope, d, None)?.ty();
                                (self.const_value_of_type(scope, d, ty)?, ty)
                            }
                        };
                        bindings.push((name, v, ty));
                        continue;
                    }
                    return err(
                        span,
                        format!("missing argument for baked parameter '{name}'"),
                    );
                };
                // `$c: Code` takes the argument expression itself, unevaluated.
                if let Some(t) = &param.ty
                    && !procs::has_poly(t)
                    && self.eval_type(def_scope, t).ok() == Some(TypeId::CODE)
                    && !matches!(
                        arg.op,
                        Some(Operand::Const {
                            value: Value::Code(_),
                            ..
                        })
                    )
                    && let Some(expr) = &arg.expr
                {
                    // A name of a `Code` constant (another `$c: Code`) passes that code on.
                    let named = match &expr.kind {
                        ast::ExprKind::Ident(_) | ast::ExprKind::Member(..) => {
                            match self.check_expr_no_emit(arg.scope, expr) {
                                Ok(Operand::Const {
                                    value: Value::Code(code),
                                    ..
                                }) => Some(code),
                                _ => None,
                            }
                        }
                        _ => None,
                    };
                    let id = match named {
                        Some(code) => code,
                        None => {
                            self.add_code(Rc::new(ast::CodeBody::Expr(expr.clone())), arg.scope)
                        }
                    };
                    bindings.push((name, Value::Code(id), TypeId::CODE));
                    continue;
                }
                let op = match arg.op.clone() {
                    Some(op) => op,
                    None => {
                        // `.{...}` / `.X` for a baked parameter: check against its declared
                        // type, or the type of its default (`$info := Info.{}`).
                        let declared = match (&param.ty, &param.default) {
                            (Some(t), _) if !procs::has_poly(t) => {
                                Some(self.eval_type(def_scope, t)?)
                            }
                            (None, Some(d)) => Some(self.eval_const(def_scope, d, None)?.ty()),
                            _ => None,
                        };
                        let expr = arg.expr.clone().unwrap();
                        let op = self.eval_const(arg.scope, &expr, declared)?;
                        match declared {
                            Some(t) => {
                                let value = self.const_value_of_type(arg.scope, &expr, t)?;
                                Operand::Const {
                                    ty: t,
                                    value,
                                    untyped: false,
                                }
                            }
                            None => op,
                        }
                    }
                };
                let op_ty = op.ty();
                let value = match op {
                    Operand::Type(t) => Value::Type(t),
                    Operand::Const {
                        value, ..
                    } => value,
                    Operand::Procs(p) if p.len() == 1 => Value::Proc(p[0]),
                    _ => {
                        return err(
                            arg.span,
                            format!("argument for '${name}' must be a compile-time constant"),
                        );
                    }
                };
                // The declared type may itself be polymorphic (`$T: Type`), or mention
                // another parameter's binding (`$compare: (T, T) -> bool`).
                let ty = match &param.ty {
                    Some(t) if !procs::has_poly(t) => match self.eval_type(def_scope, t) {
                        Ok(ty) => ty,
                        Err(_) => self.type_of_value(&value),
                    },
                    // Aggregate constants carry no type of their own.
                    _ => match self.type_of_value(&value) {
                        TypeId::VOID => op_ty,
                        t => t,
                    },
                };
                if let Some(t) = &param.ty
                    && procs::has_poly(t)
                {
                    // Aggregate constants (`.[1, 2]`) carry no type of their own.
                    let vt = match self.type_of_value(&value) {
                        TypeId::VOID => op_ty,
                        vt => vt,
                    };
                    self.match_pattern(t, vt, &mut bindings, def_scope)?;
                }
                bindings.push((name, value, ty));
                continue;
            }
            let Some(pattern) = &param.ty else {
                continue;
            };
            let implicit =
                !procs::has_poly(pattern) && self.mentions_poly_struct(def_scope, pattern)?;
            if !procs::has_poly(pattern) && !implicit {
                continue;
            }
            for (k, arg) in arg_ops.iter().enumerate() {
                let Some(op) = &arg.op else {
                    continue;
                };
                // `null` binds nothing until the other arguments are seen.
                if matches!(
                    op,
                    Operand::Const {
                        value: Value::Null,
                        untyped: true,
                        ..
                    }
                ) && !param.variadic
                {
                    null_patterns.push(pattern.clone());
                    continue;
                }
                // An overloaded or polymorphic procedure passed for a procedure-typed
                // parameter is resolved once the other bindings are known.
                if let Operand::Procs(ps) = op
                    && (ps.len() > 1 || self.proc(ps[0]).is_poly)
                    && matches!(pattern.kind, E::ProcType(_))
                {
                    deferred_procs.push((pattern.clone(), ps.clone(), arg.span));
                    continue;
                }
                let mut ty = match op {
                    // `#char "x"` binds a type variable as `u8` (`split(s, #char ",")`).
                    Operand::Const {
                        ty: TypeId::U8,
                        untyped: true,
                        ..
                    } => TypeId::U8,
                    Operand::Const {
                        ty,
                        value,
                        untyped: true,
                    } => self.default_untyped(*ty, value),
                    Operand::Type(_) => TypeId::TYPE,
                    Operand::Procs(p) if p.len() == 1 && !self.proc(p[0]).is_poly => {
                        self.proc_type(p[0], arg.span)?
                    }
                    other => other.ty(),
                };
                if param.variadic
                    && matches!(slots[i], Slot::Spread(_))
                    && let TypeKind::Array {
                        elem, ..
                    } = self.types.kind(ty)
                {
                    ty = *elem;
                }
                if k > 0 && param.variadic {
                    // Later variadic elements must match the first binding.
                    let mut probe = bindings.clone();
                    if self
                        .match_pattern(pattern, ty, &mut probe, def_scope)
                        .is_err()
                    {
                        continue;
                    }
                }
                self.match_pattern(pattern, ty, &mut bindings, def_scope)?;
                // Later parameter types may use this one's instance (`item: array.type`).
                if implicit
                    && k == 0
                    && let Some(n) = param.name
                    && !bindings.iter().any(|(b, _, _)| *b == n.name)
                {
                    bindings.push((n.name, Value::Type(ty), TypeId::TYPE));
                }
            }
        }
        // Lambdas are checked last: their parameter types come from the other bindings.
        for (i, param) in header.params.iter().enumerate() {
            let (Slot::Arg(a), Some(pattern)) = (&slots[i], &param.ty) else {
                continue;
            };
            let CallArg {
                op: None,
                expr:
                    Some(ast::Expr {
                        kind:
                            E::Lambda {
                                header: lh,
                                body,
                            },
                        ..
                    }),
                scope,
                span,
                ..
            } = &args[*a]
            else {
                continue;
            };
            if procs::has_poly(pattern) {
                self.infer_lambda_bindings(
                    pattern,
                    (*scope, lh, body),
                    def_scope,
                    &mut bindings,
                    *span,
                )?;
            }
        }
        for (pattern, ps, arg_span) in deferred_procs {
            let E::ProcType(ph) = &pattern.kind else {
                continue;
            };
            let known = self.const_scope(def_scope, bindings.clone(), arg_span);
            let mut params = Vec::new();
            for p in &ph.params {
                let Some(t) = &p.ty else {
                    return err(
                        arg_span,
                        "cannot infer the parameter types of this procedure",
                    );
                };
                params.push(self.eval_type(known, t)?);
            }
            let mut matched = None;
            for p in ps {
                if let Some(inst) = self.proc_for_param_types(p, &params, arg_span) {
                    matched = Some(inst);
                    break;
                }
            }
            let Some(inst) = matched else {
                return err(arg_span, "no overload matches the procedure parameter");
            };
            let ty = self.proc_type(inst, arg_span)?;
            self.match_pattern(&pattern, ty, &mut bindings, def_scope)?;
        }
        // A type variable seen only through `null` is `*void`.
        let void_ptr = self.types.pointer(TypeId::VOID);
        for pattern in null_patterns {
            let mut probe = bindings.clone();
            if self
                .match_pattern(&pattern, void_ptr, &mut probe, def_scope)
                .is_ok()
            {
                bindings = probe;
            }
        }
        // Defaults of the form `$T` without arguments are an error unless bound elsewhere.
        // A `#modify` block may still bind what the arguments did not determine.
        if header.modify.is_none() {
            for param in &header.params {
                if let Some(t) = &param.ty {
                    for name in poly_names(t) {
                        if !bindings.iter().any(|(n, _, _)| *n == name) {
                            return err(
                                span,
                                format!("could not infer polymorphic type '${name}'"),
                            );
                        }
                    }
                }
            }
        }
        match &header.modify {
            Some(block) => self.run_modify(proc, header, block, bindings, span),
            None => Ok(bindings),
        }
    }

    /// A baked variadic parameter given `..view`: the view must be a compile-time constant
    /// of the parameter's array type.
    fn baked_spread(
        &mut self,
        def_scope: ScopeId,
        param: &ast::Param,
        arg: &CallArg,
    ) -> Result<(Value, TypeId)> {
        let elem = match &param.ty {
            Some(t) if !procs::has_poly(t) => self.eval_type(def_scope, t)?,
            _ => TypeId::TYPE,
        };
        let ty = self.types.array(elem, ArrayKind::View);
        let op = match (&arg.op, &arg.expr) {
            (Some(op), _) => op.clone(),
            (None, Some(e)) => self.eval_const(arg.scope, e, Some(ty))?,
            (None, None) => return err(arg.span, "missing value"),
        };
        let value = self.const_value_of_operand(arg.scope, op, ty, arg.span)?;
        Ok((value, ty))
    }

    /// The value of a baked variadic parameter (`$types: ..Type`): a constant `[] T`
    /// view over the arguments, each of which must be a compile-time constant.
    fn baked_pack(
        &mut self,
        def_scope: ScopeId,
        param: &ast::Param,
        args: &[&CallArg],
    ) -> Result<(Value, TypeId)> {
        let span = param.span;
        let elem = match &param.ty {
            Some(t) if !procs::has_poly(t) => self.eval_type(def_scope, t)?,
            _ => TypeId::TYPE,
        };
        let esize = self.size_of(elem, span)?;
        let mut array = value::Aggregate {
            bytes: vec![0; esize as usize * args.len()],
            relocs: Vec::new(),
        };
        for (i, arg) in args.iter().enumerate() {
            let op = match (&arg.op, &arg.expr) {
                (Some(op), _) => op.clone(),
                (None, Some(e)) => self.eval_const(arg.scope, e, Some(elem))?,
                (None, None) => return err(arg.span, "missing value"),
            };
            let op = self.const_value_of_operand(arg.scope, op, elem, arg.span)?;
            self.write_value(&mut array, i as u64 * esize, &op, elem, arg.span)?;
        }
        let align = self.align_of(elem, span)?;
        let data = self.program.add_global(ir::Global {
            name: "baked.pack".into(),
            size: array.bytes.len().max(1) as u64,
            align,
            init: array.bytes,
            relocs: array.relocs,
            read_only: true,
            export: None,
        });
        let mut view = value::Aggregate {
            bytes: vec![0; 16],
            relocs: vec![ir::Reloc {
                offset: 8,
                target: ir::RelocTarget::Global(data),
                addend: 0,
            }],
        };
        view.bytes[..8].copy_from_slice(&(args.len() as u64).to_le_bytes());
        Ok((
            Value::Bytes(Rc::new(view)),
            self.types.array(elem, ArrayKind::View),
        ))
    }

    /// Match a polymorphic type pattern against a concrete type, adding bindings.
    /// `ty` if it is an instance of `ps`, else the first `#as` member (searched
    /// depth-first) that is one.
    /// Whether struct `ty` is `want` or reaches it through `#as` members.
    fn has_as_base(&mut self, ty: TypeId, want: TypeId) -> bool {
        if ty == want {
            return true;
        }
        let Some(s) = self.types.as_struct(ty) else {
            return false;
        };
        if self.layout_struct(s, Span::default()).is_err() {
            return false;
        }
        let fields = self.types.struct_info(s).fields.clone();
        fields
            .iter()
            .filter(|f| f.as_)
            .any(|f| self.has_as_base(f.ty, want))
    }

    /// The polymorphic struct a type expression names, if it is a bare identifier for one.
    fn ident_poly_struct_expr(
        &mut self,
        scope: ScopeId,
        expr: &ast::Expr,
    ) -> Result<Option<value::PolyStructId>> {
        match &expr.kind {
            E::Ident(name) => self.ident_poly_struct(scope, *name),
            _ => Ok(None),
        }
    }

    fn instance_or_as_base(&mut self, ps: value::PolyStructId, ty: TypeId) -> Option<TypeId> {
        // A `#bake_arguments` struct's instances are its origin's with the baked values.
        let poly = &self.poly_structs[ps.0 as usize];
        let (origin, baked) = match poly.origin {
            Some(origin) => (origin, poly.baked.clone()),
            None => (ps, Vec::new()),
        };
        if self.poly_structs[origin.0 as usize]
            .instances
            .values()
            .any(|&t| t == ty)
            && (baked.is_empty() || {
                let bindings = self.poly_struct_bindings(self.types.as_struct(ty)?);
                baked.iter().all(|(name, value, _)| {
                    bindings.iter().any(|(n, v, _)| n == name && v == value)
                })
            })
        {
            return Some(ty);
        }
        let s = self.types.as_struct(ty)?;
        self.layout_struct(s, Span::default()).ok()?;
        let fields = self.types.struct_info(s).fields.clone();
        fields
            .iter()
            .filter(|f| f.as_)
            .find_map(|f| self.instance_or_as_base(ps, f.ty))
    }

    pub fn match_pattern(
        &mut self,
        pattern: &ast::Expr,
        ty: TypeId,
        bindings: &mut Vec<(Sym, Value, TypeId)>,
        scope: ScopeId,
    ) -> Result<()> {
        let span = pattern.span;
        let bind = |c: &mut Compiler,
                    bindings: &mut Vec<(Sym, Value, TypeId)>,
                    name: Sym,
                    value: Value,
                    vty: TypeId|
         -> Result<()> {
            if let Some((_, existing, _)) = bindings.iter().find(|(n, _, _)| *n == name) {
                if *existing != value {
                    return err(
                        span,
                        format!(
                            "conflicting types for '${name}': {} and {}",
                            existing.render(&c.types),
                            value.render(&c.types)
                        ),
                    );
                }
                return Ok(());
            }
            bindings.push((name, value, vty));
            Ok(())
        };
        match &pattern.kind {
            E::PolyVar {
                name, ..
            } => bind(self, bindings, *name, Value::Type(ty), TypeId::TYPE),
            // A bare polymorphic struct matches any of its instances; binding its
            // name makes the instance signature name the instance.
            E::Ident(name) => match self.ident_poly_struct(scope, *name)? {
                Some(ps) => {
                    // A `*Instance` argument is dereferenced for a by-value parameter;
                    // a struct with an `#as` instance member matches as that instance.
                    let instance = self.instance_or_as_base(ps, ty).or_else(|| {
                        let p = self.types.pointee(ty)?;
                        self.instance_or_as_base(ps, p)
                    });
                    match instance {
                        Some(t) => bind(self, bindings, *name, Value::Type(t), TypeId::TYPE),
                        None => err(
                            span,
                            format!("{} is not an instance of '{name}'", self.types.name(ty)),
                        ),
                    }
                }
                None => Ok(()),
            },
            E::PolyRestricted {
                name,
                restriction,
                interface,
            } => {
                // `$T/Table`: T must be an instance of the polymorphic struct `Table` (or hold one
                // as an `#as` member).
                let mut ty = ty;
                if !interface
                    && let E::Ident(r) = &restriction.kind
                    && let Some(ps) = self.ident_poly_struct(scope, *r)?
                    && self.instance_or_as_base(ps, ty).is_none()
                {
                    // A `*Instance` argument is dereferenced for a by-value parameter.
                    match self.types.pointee(ty) {
                        Some(p) if self.instance_or_as_base(ps, p).is_some() => ty = p,
                        _ => {
                            return err(
                                span,
                                format!("{} is not an instance of '{r}'", self.types.name(ty)),
                            );
                        }
                    }
                }
                // `$T/Entity`: a struct argument must be `Entity` or have it as an `#as` base.
                if !interface
                    && let E::Ident(_) | E::Member(..) = &restriction.kind
                    && self.ident_poly_struct_expr(scope, restriction)?.is_none()
                    && let Ok(want) = self.eval_type(scope, restriction)
                    && self.types.as_struct(want).is_some()
                    && self.types.as_struct(ty).is_some()
                    && !self.has_as_base(ty, want)
                {
                    return err(
                        span,
                        format!(
                            "{} does not satisfy the restriction '{}'",
                            self.types.name(ty),
                            self.types.name(want)
                        ),
                    );
                }
                bind(self, bindings, *name, Value::Type(ty), TypeId::TYPE)
            }
            E::Unary(ast::UnOp::Star, inner) => match self.types.kind(ty).clone() {
                TypeKind::Pointer(p) => self.match_pattern(inner, p, bindings, scope),
                _ => err(
                    span,
                    format!("expected a pointer, found {}", self.types.name(ty)),
                ),
            },
            E::ArrayType {
                size,
                elem,
            } => {
                let TypeKind::Array {
                    elem: actual,
                    kind,
                } = self.types.kind(ty).clone()
                else {
                    return err(
                        span,
                        format!("expected an array, found {}", self.types.name(ty)),
                    );
                };
                match (size, kind) {
                    (ast::ArraySize::View, _) => {}
                    (ast::ArraySize::Resizable, ArrayKind::Resizable) => {}
                    (ast::ArraySize::Fixed(n), ArrayKind::Fixed(count)) => {
                        if let E::PolyVar {
                            name, ..
                        } = &n.kind
                        {
                            bind(
                                self,
                                bindings,
                                *name,
                                Value::Int(count as i128),
                                TypeId::S64,
                            )?;
                        }
                    }
                    _ => {
                        return err(
                            span,
                            format!("array kind mismatch with {}", self.types.name(ty)),
                        );
                    }
                }
                self.match_pattern(elem, actual, bindings, scope)
            }
            E::Call {
                callee,
                args,
                ..
            } => {
                // `Table($K, $V)` against an instance of the same polymorphic struct
                // (a `*Instance` argument is dereferenced for a by-value parameter).
                let ty = match self.types.pointee(ty) {
                    Some(p) if self.types.as_struct(p).is_some() => p,
                    _ => ty,
                };
                // The instance itself, or (for a named struct) an `#as` member that is one.
                let instance = match &callee.kind {
                    E::Ident(name) => match self.ident_poly_struct(scope, *name)? {
                        Some(ps) => self.instance_or_as_base(ps, ty),
                        None => None,
                    },
                    _ => self.types.as_struct(ty).map(|_| ty),
                };
                let Some(ty) = instance else {
                    return err(
                        span,
                        format!("{} is not an instance of this struct", self.types.name(ty)),
                    );
                };
                let s = self.types.as_struct(ty).unwrap();
                let info = self.types.struct_info(s).clone();
                for (i, a) in args.iter().enumerate() {
                    let Some(value) = info.poly_args.get(i) else {
                        break;
                    };
                    match &a.value.kind {
                        E::PolyVar {
                            name, ..
                        } => {
                            let vty = self.type_of_value(value);
                            bind(self, bindings, *name, value.clone(), vty)?;
                        }
                        _ if procs::has_poly(&a.value) => {
                            if let Value::Type(t) = value {
                                self.match_pattern(&a.value, *t, bindings, scope)?;
                            }
                        }
                        _ => {}
                    }
                }
                Ok(())
            }
            E::ProcType(header) => {
                let TypeKind::Proc(pt) = self.types.kind(ty).clone() else {
                    return err(
                        span,
                        format!("expected a procedure, found {}", self.types.name(ty)),
                    );
                };
                for (p, &actual) in header.params.iter().zip(&pt.params) {
                    if let Some(t) = &p.ty {
                        self.match_pattern(t, actual, bindings, scope)?;
                    }
                }
                for (r, &actual) in header.returns.iter().zip(&pt.returns) {
                    if let Some(t) = &r.ty {
                        self.match_pattern(t, actual, bindings, scope)?;
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Emit a resolved call, recording which of its results are `#must`.
    fn emit_call(
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
            match self.types.kind(ty) {
                TypeKind::Distinct(d) if self.types.distincts[d.0 as usize].isa => {
                    Some((ty, self.types.distincts[d.0 as usize].base))
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
        self.last_call_must = must.iter().any(|&m| m).then_some((span, name, must));
        result
    }

    /// Error when a call at `call_span` leaves one of its `#must` results unused; only the
    /// first `used` results are taken (0 for a call statement). Checked when the call's code
    /// is generated, so a violation inside never-expanded macro code is harmless.
    pub(super) fn check_must_used(&mut self, call_span: Span, used: usize) -> Result<()> {
        let Some((span, name, must)) = self.last_call_must.take() else {
            return Ok(());
        };
        if span != call_span {
            return Ok(());
        }
        if must.iter().skip(used).any(|&m| m) {
            return err(
                call_span,
                format!("the result of '{name}' is marked #must and cannot be discarded"),
            );
        }
        Ok(())
    }

    fn emit_call_inner(
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
        if header.flags.intrinsic && self.proc(proc).lit.body.is_none() {
            return self.emit_intrinsic(f, proc, &sig, &c.slots, &header, args, span);
        }
        let mut values = Vec::new();
        if sig.has_context {
            let Some(ctx) = f.context else {
                return err(
                    span,
                    format!(
                        "cannot call '{}' without a context (use push_context in #c_call code)",
                        self.proc(proc).name
                    ),
                );
            };
            values.push(ctx);
        }
        let mut k = 0;
        let mut c_vararg_values = Vec::new();
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
                        let (_, v) = self.rvalue(f, op, span)?;
                        c_vararg_values.push(v);
                    }
                }
                continue;
            }
            let v = self.param_value(f, &sig, &param, slot, &args, span)?;
            values.push(v);
        }
        values.extend(c_vararg_values.iter().copied());
        // Out-pointers for memory-class results.
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
        let target = self.proc_func(proc, span)?;
        let mut ir_sig = self.ir_sig(sig.ty, span)?;
        let callee = match target {
            ProcTarget::Func(id) => ir::Callee::Func(id),
            ProcTarget::Foreign(id) => {
                if sig.c_varargs {
                    // Record the actual argument classes for this variadic call site.
                    for &v in &c_vararg_values {
                        ir_sig.params.push(f.b.val_ty(v));
                    }
                    let fp = f.b.foreign_addr(id);
                    ir::Callee::Indirect(fp, ir_sig.clone())
                } else {
                    ir::Callee::Foreign(id)
                }
            }
        };
        if sig.c_varargs && !matches!(callee, ir::Callee::Indirect(..)) {
            ir_sig
                .params
                .extend(c_vararg_values.iter().map(|&v| f.b.val_ty(v)));
        }
        let results = f.b.call(callee, values, &ir_sig.returns.clone());
        Ok(self.call_results(&sig.returns, results, outs))
    }

    fn call_results(
        &mut self,
        returns: &[TypeId],
        results: Vec<ir::Val>,
        outs: Vec<Option<ir::Val>>,
    ) -> Operand {
        let mut values = Vec::new();
        let mut regs = results.into_iter();
        for (i, &rt) in returns.iter().enumerate() {
            let v = match outs[i] {
                Some(out) => out,
                None => regs.next().unwrap(),
            };
            values.push((rt, v));
        }
        match values.len() {
            0 => Operand::Void,
            1 => Operand::Value {
                ty: values[0].0,
                val: values[0].1,
            },
            _ => Operand::Multi(values),
        }
    }

    /// Check (if deferred) and convert an argument for a parameter of type `ty`.
    fn arg_operand(&mut self, f: &mut FnCtx, arg: &CallArg, ty: Option<TypeId>) -> Result<Operand> {
        match &arg.op {
            Some(op) => Ok(op.clone()),
            None => {
                let expr = arg.expr.as_ref().unwrap();
                self.check_expr(f, arg.scope, expr, ty)
            }
        }
    }

    /// Produce the lowered value passed for one parameter.
    fn param_value(
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
                        }
                    }
                }
            }
            Slot::Arg(a) | Slot::Spread(a) => {
                let op = self.arg_operand(f, &args[*a], Some(param.ty))?;
                if self.auto_deref_arg(op.ty(), param.ty) {
                    let (_, p) = self.rvalue(f, op, args[*a].span)?;
                    Operand::Place {
                        ty: param.ty,
                        addr: p,
                    }
                } else {
                    self.convert(f, op, param.ty, args[*a].span)?
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
                    E::CallerLocation => self.location_operand(f, span)?,
                    _ => self.check_expr(f, sig.scope, &d, Some(param.ty))?,
                };
                self.convert(f, op, param.ty, d.span)?
            }
            Slot::Variadic(list) => {
                let TypeKind::Array {
                    elem, ..
                } = self.types.kind(param.ty).clone()
                else {
                    unreachable!()
                };
                let size = self.size_of(elem, span)?;
                let align = self.align_of(elem, span)?;
                let n = list.len() as u64;
                let base = f.b.alloca((size * n).max(1), align);
                for (i, &a) in list.iter().enumerate() {
                    let op = self.arg_operand(f, &args[a], Some(elem))?;
                    let op = self.convert(f, op, elem, args[a].span)?;
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
    fn zero_param(&mut self, f: &mut FnCtx, ty: TypeId, span: Span) -> Result<ir::Val> {
        if self.is_memory_type(ty) {
            let size = self.size_of(ty, span)?;
            let align = self.align_of(ty, span)?;
            let slot = f.b.alloca(size.max(1), align);
            f.b.zero(slot, size);
            return Ok(slot);
        }
        let t = self.ir_ty(ty).unwrap_or(Ty::I64);
        Ok(if t.is_float() {
            f.b.fconst(t, 0.0)
        } else {
            f.b.iconst(t, 0)
        })
    }

    /// Place named arguments of a call through a procedure value and fill in the defaults its
    /// type was written with.
    fn order_indirect_args(
        &mut self,
        info: &ProcTypeParams,
        args: Vec<CallArg>,
        span: Span,
    ) -> Result<Vec<CallArg>> {
        let n = info.names.len();
        let mut slots: Vec<Option<CallArg>> = vec![None; n];
        let mut next = 0;
        for arg in args {
            let i = match arg.name {
                Some(name) => info
                    .names
                    .iter()
                    .position(|&p| p == Some(name))
                    .ok_or_else(|| {
                        Box::new(Diagnostic::error(
                            arg.span,
                            format!("no parameter named '{name}'"),
                        ))
                    })?,
                None => {
                    while next < n && slots[next].is_some() {
                        next += 1;
                    }
                    next
                }
            };
            if i >= n {
                return err(
                    arg.span,
                    format!("too many arguments (expected at most {n})"),
                );
            }
            slots[i] = Some(CallArg {
                name: None,
                ..arg
            });
        }
        slots
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
                (None, None) => err(span, format!("missing argument {}", i + 1)),
            })
            .collect()
    }

    fn call_indirect(
        &mut self,
        f: &mut FnCtx,
        ty: TypeId,
        callee: ir::Val,
        mut args: Vec<CallArg>,
        span: Span,
    ) -> Result<Operand> {
        let TypeKind::Proc(pt) = self.types.kind(ty).clone() else {
            return err(
                span,
                format!("cannot call a value of type {}", self.types.name(ty)),
            );
        };
        if let Some(info) = self.proc_type_params.get(&ty).cloned() {
            args = self.order_indirect_args(&info, args, span)?;
        }
        if args.len() < pt.params.len()
            || (!pt.c_varargs && !pt.variadic && args.len() > pt.params.len())
        {
            return err(
                span,
                format!(
                    "expected {} arguments, found {}",
                    pt.params.len(),
                    args.len()
                ),
            );
        }
        let mut values = Vec::new();
        if !pt.no_context {
            // Without a context (e.g. inside #c_call code) the callee receives null.
            let ctx = match f.context {
                Some(c) => c,
                None => f.b.iconst(Ty::Ptr, 0),
            };
            values.push(ctx);
        }
        for (i, &param_ty) in pt.params.iter().enumerate() {
            let op = self.arg_operand(f, &args[i], Some(param_ty))?;
            let op = self.convert(f, op, param_ty, args[i].span)?;
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
        let results =
            f.b.call(ir::Callee::Indirect(callee, sig), values, &returns);
        Ok(self.call_results(&pt.returns, results, outs))
    }

    /// Intrinsic procedures declared in Preload (`memcpy`, `compare_and_swap`...).
    #[allow(clippy::too_many_arguments)]
    fn emit_intrinsic(
        &mut self,
        f: &mut FnCtx,
        proc: ProcId,
        sig: &Signature,
        slots: &[Slot],
        header: &ast::ProcHeader,
        args: Vec<CallArg>,
        span: Span,
    ) -> Result<Operand> {
        let name = self.proc(proc).name;
        let mut values = Vec::new();
        let mut k = 0;
        for (i, slot) in slots.iter().enumerate() {
            if header.params[i].baked {
                continue;
            }
            let param = sig.params[k].clone();
            k += 1;
            values.push(self.param_value(f, sig, &param, slot, &args, span)?);
        }
        let op = match name.as_str() {
            "memcpy" => ir::Intrinsic::Memcpy,
            "memset" => ir::Intrinsic::Memset,
            "memcmp" => ir::Intrinsic::Memcmp,
            "compare_and_swap" => ir::Intrinsic::CompareAndSwap,
            "debug_break" => ir::Intrinsic::DebugBreak,
            "sqrt" => ir::Intrinsic::Sqrt,
            "sin" => ir::Intrinsic::Sin,
            "cos" => ir::Intrinsic::Cos,
            "floor" => ir::Intrinsic::Floor,
            "ceil" => ir::Intrinsic::Ceil,
            "round" => ir::Intrinsic::Round,
            "trunc" => ir::Intrinsic::Trunc,
            "abs" | "fabs" => ir::Intrinsic::Fabs,
            "rdtsc" | "get_cpu_cycle_count" => ir::Intrinsic::CycleCounter,
            "pause" | "mm_pause" => ir::Intrinsic::Pause,
            other => return err(span, format!("unknown intrinsic '{other}'")),
        };
        let returns: Vec<Ty> = sig
            .returns
            .iter()
            .map(|&t| self.ir_ty(t).unwrap_or(Ty::Ptr))
            .collect();
        if op == ir::Intrinsic::CompareAndSwap && values.len() == 3 {
            // The interpreter needs the operand width: 1, 2, 4 or 8 bytes.
            let width = self.size_of(sig.params[1].ty, span)?;
            values.push(f.b.iconst(Ty::I64, width));
        }
        let results = f.b.intrinsic(op, values, &returns);
        let outs = vec![None; sig.returns.len()];
        Ok(self.call_results(&sig.returns, results, outs))
    }

    // -----------------------------------------------------------------------
    // Builtins
    // -----------------------------------------------------------------------

    fn check_builtin(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        b: BuiltinProc,
        args: &[ast::Arg],
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        let _ = expected;
        let Some(arg) = args.first() else {
            return err(span, "builtin needs an argument");
        };
        match b {
            BuiltinProc::SizeOf | BuiltinProc::AlignOf => {
                let op = match self.type_field_type(scope, &arg.value)? {
                    Some(t) => Operand::Type(t),
                    None => self.check_expr_no_emit(scope, &arg.value)?,
                };
                let ty = match op {
                    Operand::Type(t) => t,
                    other => other.ty(),
                };
                let v = if b == BuiltinProc::SizeOf {
                    self.size_of(ty, span)?
                } else {
                    self.align_of(ty, span)?
                };
                Ok(Operand::Const {
                    ty: TypeId::S64,
                    value: Value::Int(v as i128),
                    untyped: true,
                })
            }
            BuiltinProc::CodeOf => {
                // The code of a procedure (its header and body), else of the expression.
                let proc = match self.check_expr_no_emit(scope, &arg.value) {
                    Ok(Operand::Procs(p)) if p.len() == 1 => Some(p[0]),
                    Ok(Operand::Const {
                        value: Value::Proc(p),
                        ..
                    }) => Some(p),
                    _ => None,
                };
                let id = match proc {
                    Some(p) => {
                        let (lit, span, pscope) = (
                            self.proc(p).lit.clone(),
                            self.proc(p).span,
                            self.proc(p).scope,
                        );
                        let expr = ast::Expr {
                            kind: ast::ExprKind::Proc(lit),
                            span,
                        };
                        let id = self.add_code(Rc::new(ast::CodeBody::Expr(expr)), pscope);
                        self.code_procs.insert(id.0 as usize, p);
                        id
                    }
                    None => self.add_code(Rc::new(ast::CodeBody::Expr(arg.value.clone())), scope),
                };
                Ok(Operand::Const {
                    ty: TypeId::CODE,
                    value: Value::Code(id),
                    untyped: false,
                })
            }
            BuiltinProc::TypeOf => {
                if let Some(t) = self.type_field_type(scope, &arg.value)? {
                    return Ok(Operand::Type(t));
                }
                // Only the type is needed, so an enclosing procedure's local is fine
                // (`#run f(type_of(local))` inside a body).
                if let Some(t) = self.outer_local_type(scope, &arg.value)? {
                    return Ok(Operand::Type(t));
                }
                let op = match self.check_expr_no_emit(scope, &arg.value) {
                    Ok(op) => op,
                    Err(e) => match self.enclosing_struct_field_type(scope, &arg.value)? {
                        Some(t) => return Ok(Operand::Type(t)),
                        None => return Err(e),
                    },
                };
                let ty = match op {
                    Operand::Type(_) => TypeId::TYPE,
                    Operand::Const {
                        ty,
                        value,
                        untyped: true,
                    } => self.default_untyped(ty, &value),
                    Operand::Procs(p) if p.len() == 1 && self.proc(p[0]).is_poly => {
                        self.poly_proc_type(p[0])
                    }
                    Operand::Procs(p) if p.len() == 1 => self.proc_type(p[0], span)?,
                    other => other.ty(),
                };
                Ok(Operand::Type(ty))
            }
            BuiltinProc::TypeInfo => {
                let op = self.check_expr(f, scope, &arg.value, Some(TypeId::TYPE))?;
                if matches!(
                    op,
                    Operand::Value {
                        ty: TypeId::TYPE,
                        ..
                    } | Operand::Place {
                        ty: TypeId::TYPE,
                        ..
                    }
                ) {
                    // A runtime `Type` is already a `*Type_Info`.
                    let (_, v) = self.rvalue(f, op, span)?;
                    let info = self.preload_type("Type_Info", span)?;
                    return Ok(Operand::Value {
                        ty: self.types.pointer(info),
                        val: v,
                    });
                }
                let ty = self.operand_as_type(op, arg.value.span)?;
                let info_ty = self.type_info_struct_type(ty, span)?;
                let ptr = self.types.pointer(info_ty);
                let global = self.type_info_global(ty, span)?;
                let v = f.b.global_addr(global);
                Ok(Operand::Value {
                    ty: ptr,
                    val: v,
                })
            }
            BuiltinProc::IsConstant => {
                let op = self.check_expr_no_emit(scope, &arg.value)?;
                let constant = op.is_const()
                    || matches!(op, Operand::Procs(_))
                    || self.is_constant_pointer(scope, &arg.value, &op);
                Ok(Operand::bool(constant))
            }
            BuiltinProc::InitializerOf => {
                let ty = self.eval_type_in(f, scope, &arg.value)?;
                let proc = self.initializer_proc(ty, span)?;
                // A type that is all zeroes by default has no initializer (`#if initializer_of(T)`).
                if matches!(self.default_initializer(ty, span), Ok(None)) {
                    return Ok(Operand::Const {
                        ty: proc.1,
                        value: Value::Null,
                        untyped: false,
                    });
                }
                Ok(Operand::Value {
                    ty: proc.1,
                    val: f.b.func_addr(proc.0),
                })
            }
            BuiltinProc::OffsetOf => err(span, "offset_of is not supported yet"),
            BuiltinProc::IsValueType => err(span, "unsupported builtin"),
        }
    }

    /// Check an expression only for its type; any IR it emits is discarded.
    /// Argument of `type_of`/`size_of`: `Struct.field` names the field's type
    /// there, without a value.
    fn type_field_type(&mut self, scope: ScopeId, expr: &ast::Expr) -> Result<Option<TypeId>> {
        if let E::Member(base, field) = &expr.kind
            && let Ok(Operand::Type(t)) = self.check_expr_no_emit(scope, base)
            && self.types.as_struct(self.types.repr_struct(t)).is_some()
            && self.struct_constant(t, field.name)?.is_none()
            && let Some((_, fty)) = self.find_member(t, field.name, expr.span)?
        {
            return Ok(Some(fty));
        }
        Ok(None)
    }

    /// `type_of(x)` inside a procedure nested in a struct: `x` is a field of that struct, which
    /// has a type though not a value.
    fn enclosing_struct_field_type(
        &mut self,
        scope: ScopeId,
        expr: &ast::Expr,
    ) -> Result<Option<TypeId>> {
        if let E::Ident(name) = &expr.kind {
            let mut s = Some(scope);
            while let Some(sid) = s {
                if let ScopeKind::Struct(t) = self.scope(sid).kind
                    && let Some((_, fty)) = self.find_member(t, *name, expr.span)?
                {
                    return Ok(Some(fty));
                }
                s = self.scope(sid).parent;
            }
        }
        Ok(None)
    }

    /// A throwaway function context for checking code whose output is discarded.
    pub(super) fn scratch_ctx(&self, scope: ScopeId) -> FnCtx {
        let file = self.scope_file(scope);
        let mut scratch = FnCtx::new(
            "scratch".into(),
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
        scratch.context = Some(scratch.b.param(0));
        scratch.type_only = true;
        scratch
    }

    pub fn check_expr_no_emit(&mut self, scope: ScopeId, expr: &ast::Expr) -> Result<Operand> {
        let mut scratch = self.scratch_ctx(scope);
        self.check_expr(&mut scratch, scope, expr, None)
    }

    /// Instantiate polymorphic `proc` so that it has procedure type `target`, as when a
    /// polymorphic procedure is passed to a parameter of procedure type: the target's
    /// parameter types play the role of call arguments.
    pub fn instantiate_for_proc_type(
        &mut self,
        proc: ProcId,
        target: TypeId,
        span: Span,
    ) -> Option<ProcId> {
        let TypeKind::Proc(info) = self.types.kind(target).clone() else {
            return None;
        };
        let inst = self.proc_for_param_types(proc, &info.params, span)?;
        let ty = self.proc_type(inst, span).ok()?;
        (ty == target || self.proc_types_compatible(ty, target)).then_some(inst)
    }

    /// The type of a polymorphic procedure for compile-time inspection (`type_of(poly)`):
    /// parameters and results that mention a type variable are shown as `void`.
    fn poly_proc_type(&mut self, proc: ProcId) -> TypeId {
        let header = self.proc(proc).lit.header.clone();
        let scope = self.proc(proc).scope;
        let eval = |c: &mut Self, t: &Option<ast::Expr>| match t {
            Some(t) if !procs::has_poly(t) => c.eval_type(scope, t).unwrap_or(TypeId::VOID),
            _ => TypeId::VOID,
        };
        let params: Vec<TypeId> = header
            .params
            .iter()
            .filter(|p| !p.baked)
            .map(|p| eval(self, &p.ty))
            .collect();
        let returns: Vec<TypeId> = header
            .returns
            .iter()
            .map(|r| eval(self, &r.ty))
            .filter(|&t| t != TypeId::VOID)
            .collect();
        self.types
            .intern(TypeKind::Proc(Rc::new(crate::types::ProcType {
                params,
                returns,
                variadic: false,
                c_varargs: false,
                c_call: false,
                no_context: false,
                non_pod_return: false,
            })))
    }

    /// The instance of `proc` (or `proc` itself) callable with arguments of the given types.
    pub fn proc_for_param_types(
        &mut self,
        proc: ProcId,
        params: &[TypeId],
        span: Span,
    ) -> Option<ProcId> {
        let scope = self.proc(proc).scope;
        let scratch = self.scratch_ctx(scope);
        let placeholder = scratch.b.param(0);
        let args: Vec<CallArg> = params
            .iter()
            .map(|&ty| CallArg {
                name: None,
                spread: false,
                expr: None,
                op: Some(Operand::Value {
                    ty,
                    val: placeholder,
                }),
                span,
                scope,
            })
            .collect();
        Some(self.match_candidate(proc, &args, span).ok()?.proc)
    }

    pub fn check_procedure_of_call(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        call: &ast::Expr,
    ) -> Result<Operand> {
        let E::Call {
            callee,
            args,
            ..
        } = &call.kind
        else {
            return err(call.span, "#procedure_of_call needs a call expression");
        };
        let callee_op = self.check_expr(f, scope, callee, None)?;
        let Operand::Procs(procs) = callee_op else {
            return err(call.span, "#procedure_of_call needs a procedure call");
        };
        // Only the argument types matter: runtime locals are fine in a constant.
        let mut scratch = self.scratch_ctx(scope);
        let call_args = self.precheck_args(&mut scratch, scope, args)?;
        for &p in &procs {
            if let Ok(c) = self.match_candidate(p, &call_args, call.span) {
                return Ok(Operand::Procs(vec![c.proc]));
            }
        }
        err(call.span, "no procedure matches this call")
    }

    // -----------------------------------------------------------------------
    // Macros
    // -----------------------------------------------------------------------

    /// Whether a block's source text may assign to (or take the address of) `name`:
    /// `name =`, `name op=`, `name.field =` or `*name`. Conservative: comments and strings
    /// count too.
    fn text_may_write(&self, body: &ast::Block, name: Sym) -> bool {
        let text = &self.sources.get(body.span.file).text;
        let text = &text.as_bytes()[body.span.start as usize..body.span.end as usize];
        // `#asm` operands may write any name they mention.
        if text.windows(4).any(|w| w == b"#asm") {
            return true;
        }
        let name = name.as_str();
        let word = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
        let skip_ws = |mut i: usize| {
            while i < text.len() && text[i].is_ascii_whitespace() {
                i += 1;
            }
            i
        };
        let assigns = |i: usize| {
            let i = skip_ws(i);
            let rest = &text[i.min(text.len())..];
            match rest {
                [b'=', b'=', ..] => false,
                [b'=', ..] => true,
                [op, b'=', ..] => b"+-*/%|&^".contains(op),
                [b'<' | b'>', b'<' | b'>', b'=', ..] => true,
                _ => false,
            }
        };
        let mut from = 0;
        while let Some(pos) = text[from..]
            .windows(name.len())
            .position(|w| w == name.as_bytes())
        {
            let start = from + pos;
            let end = start + name.len();
            from = end;
            if (start > 0 && word(text[start - 1])) || (end < text.len() && word(text[end])) {
                continue;
            }
            let mut before = start;
            while before > 0 && text[before - 1].is_ascii_whitespace() {
                before -= 1;
            }
            if before > 0 && text[before - 1] == b'*' {
                return true;
            }
            if assigns(end) {
                return true;
            }
            // `name.field = ...`
            let mut i = skip_ws(end);
            if i < text.len() && text[i] == b'.' {
                i = skip_ws(i + 1);
                while i < text.len() && word(text[i]) {
                    i += 1;
                }
                if assigns(i) {
                    return true;
                }
            }
        }
        false
    }

    fn expand_macro(
        &mut self,
        f: &mut FnCtx,
        caller: ScopeId,
        proc: ProcId,
        slots: &[Slot],
        args: Vec<CallArg>,
        span: Span,
    ) -> Result<Operand> {
        let sig = self.signature(proc, span)?;
        let header = self.proc(proc).lit.header.clone();
        let Some(body) = self.proc(proc).lit.body.clone() else {
            return err(span, "macro has no body");
        };
        if f.macros.len() > 64 {
            return err(span, "macro expansion is too deeply nested");
        }
        let module = self.scope(sig.scope).module;
        let mscope = self.new_scope(ScopeKind::Macro, Some(sig.scope), module, None);
        // Locals of the macro belong to the caller's procedure.
        self.scopes[mscope.0 as usize].proc_depth = self.scope(caller).proc_depth;
        let mut k = 0;
        for (i, slot) in slots.iter().enumerate() {
            if header.params[i].baked {
                continue;
            }
            let param = sig.params[k].clone();
            k += 1;
            let Some(name) = param.name else {
                continue;
            };
            if param.discard {
                let e = self.add_const(mscope, name, param.span, Value::Int(0), TypeId::S64);
                self.discard_params.insert(e);
                continue;
            }
            if param.ty == TypeId::CODE {
                // Code arguments are passed unevaluated, bound to the caller's scope.
                let code = match slot {
                    Slot::Arg(a)
                        if let Some(Operand::Const {
                            value: Value::Code(code),
                            ..
                        }) = &args[*a].op =>
                    {
                        *code
                    }
                    Slot::Arg(a) => {
                        let Some(expr) = args[*a].expr.clone() else {
                            return err(args[*a].span, "Code parameter needs a code argument");
                        };
                        // A name of a `#code` constant passes that code; a name of a
                        // `Code` variable passes its value.
                        let mut runtime_code = false;
                        let named = match &expr.kind {
                            E::Ident(_) | E::Member(..) => {
                                match self.check_expr_no_emit(args[*a].scope, &expr) {
                                    Ok(Operand::Const {
                                        value: Value::Code(code),
                                        ..
                                    }) => Some(code),
                                    Ok(op) if op.ty() == TypeId::CODE => {
                                        runtime_code = true;
                                        None
                                    }
                                    _ => None,
                                }
                            }
                            _ => None,
                        };
                        match named {
                            Some(code) => code,
                            None if runtime_code => {
                                let v = self.param_value(f, &sig, &param, slot, &args, span)?;
                                let addr = self.spill(f, param.ty, v, span)?;
                                let depth = self.scope(mscope).proc_depth;
                                self.add_entity(
                                    mscope,
                                    name,
                                    param.span,
                                    EntityKind::Local {
                                        ty: param.ty,
                                        addr,
                                        depth,
                                    },
                                    false,
                                );
                                continue;
                            }
                            None => {
                                let body = match &expr.kind {
                                    E::Code(c) => c.clone(),
                                    _ => Rc::new(ast::CodeBody::Expr(expr)),
                                };
                                self.add_code(body, caller)
                            }
                        }
                    }
                    Slot::Default
                        if param
                            .default
                            .as_ref()
                            .is_some_and(|d| matches!(d.kind, E::CallerCode)) =>
                    {
                        let d = param.default.as_ref().unwrap();
                        match self.check_expr(f, sig.scope, d, Some(TypeId::CODE))? {
                            Operand::Const {
                                value: Value::Code(code),
                                ..
                            } => code,
                            _ => return err(d.span, "#caller_code did not produce Code"),
                        }
                    }
                    _ => return err(span, "Code parameter needs an argument"),
                };
                self.add_const(mscope, name, param.span, Value::Code(code), TypeId::CODE);
                continue;
            }
            // A for_expansion's `flags` is a compile-time constant (`#assert(flags == 0)`).
            if let Slot::Arg(a) = slot
                && let Some(Operand::Const {
                    value: value @ Value::Int(_),
                    ..
                }) = &args[*a].op
                && self.preload_type("For_Flags", span).ok() == Some(param.ty)
            {
                self.add_const(mscope, name, param.span, value.clone(), param.ty);
                continue;
            }
            // `v: $T` given an untyped literal stays a literal inside the macro, so
            // `` `return ERR, v `` adapts `0` to the caller's (e.g. distinct) result type.
            if let Slot::Arg(a) = slot
                && let Some(Operand::Const {
                    value,
                    untyped: true,
                    ..
                }) = &args[*a].op
                && matches!(
                    header.params[i].ty.as_ref().map(|t| &t.kind),
                    Some(E::PolyVar { .. })
                )
            {
                let e = self.add_const(mscope, name, param.span, value.clone(), param.ty);
                self.entity_mut(e).untyped_const = true;
                continue;
            }
            let v = self.param_value(f, &sig, &param, slot, &args, span)?;
            let addr = if self.is_memory_type(param.ty) {
                v
            } else {
                self.spill(f, param.ty, v, span)?
            };
            let depth = self.scope(mscope).proc_depth;
            let e = self.add_entity(
                mscope,
                name,
                param.span,
                EntityKind::Local {
                    ty: param.ty,
                    addr,
                    depth,
                },
                false,
            );
            // The argument's value when it is a constant: a literal or constant given, a
            // procedure, or a literal default (`var_name := "var"`).
            let constant = match slot {
                Slot::Arg(a) => match &args[*a].op {
                    Some(Operand::Const {
                        value,
                        ty,
                        untyped,
                    }) => Some((value.clone(), *ty, *untyped)),
                    Some(Operand::Procs(p)) if p.len() == 1 => {
                        Some((Value::Proc(p[0]), param.ty, false))
                    }
                    _ => None,
                },
                Slot::Default => match &header.params[i].default {
                    Some(d)
                        if matches!(d.kind, E::Str(_) | E::Int(_) | E::Float(_) | E::Bool(_)) =>
                    {
                        let pscope = self.proc(proc).scope;
                        match self.eval_const(pscope, d, Some(param.ty)) {
                            Ok(Operand::Const {
                                value,
                                ty,
                                untyped,
                            }) => Some((value, ty, untyped)),
                            _ => None,
                        }
                    }
                    _ => None,
                },
                _ => None,
            };
            if let Some((value, ty, untyped)) = constant
                && (ty == param.ty
                    || (untyped
                        && matches!(value, Value::Int(_))
                        && self.types.is_integer(param.ty)))
            {
                self.local_consts.insert(e, (value.clone(), param.ty));
                // A constant argument the body never writes stays a constant.
                if matches!(
                    value,
                    Value::String(_)
                        | Value::Int(_)
                        | Value::Bool(_)
                        | Value::Float(_)
                        | Value::Proc(_)
                ) && !self.text_may_write(&body, name)
                {
                    self.const_macro_params.insert(e, (value, param.ty));
                }
            }
            if param.using {
                self.scope_mut(mscope)
                    .usings
                    .push(scope::UsingEntry::Place {
                        ty: param.ty,
                        entity: e,
                    });
            }
        }
        let exit = f.b.new_block();
        let mut result_slots = Vec::new();
        for &rt in &sig.returns {
            let size = self.size_of(rt, span)?;
            let align = self.align_of(rt, span)?;
            result_slots.push((rt, f.b.alloca(size.max(1), align)));
        }
        f.macros.push(MacroFrame {
            caller_scope: caller,
            exit_block: exit,
            result_slots: result_slots.clone(),
            defer_depth: f.defers.len(),
            loop_depth: f.loops.len(),
            caller_macro_depth: f.macros.len(),
            for_body: f.pending_for_body.take(),
        });
        // Backticks inside this expansion refer to its caller, not to a running `` `defer ``.
        let saved_backtick = f.backtick_scope.take();
        let result = self.check_block_stmts(f, mscope, &body.stmts);
        f.backtick_scope = saved_backtick;
        let frame = f.macros.pop().unwrap();
        result?;
        if !f.b.is_terminated() {
            self.emit_defers(f, frame.defer_depth, span)?;
            f.b.jump(exit);
        }
        f.defers.truncate(frame.defer_depth);
        f.b.switch_to(exit);
        let mut values = Vec::new();
        for (rt, addr) in result_slots {
            let v = match self.ir_ty(rt) {
                Some(t) => f.b.load(t, addr),
                None => addr,
            };
            values.push((rt, v));
        }
        Ok(match values.len() {
            0 => Operand::Void,
            1 => Operand::Value {
                ty: values[0].0,
                val: values[0].1,
            },
            _ => Operand::Multi(values),
        })
    }

    // -----------------------------------------------------------------------
    // Operator overloading
    // -----------------------------------------------------------------------

    /// Scope a struct (or pointer to one) was declared in, for names that
    /// travel with the type: operator overloads and `for_expansion`.
    pub fn struct_home_scope(&self, t: TypeId) -> Option<ScopeId> {
        let t = self.types.pointee(t).unwrap_or(t);
        let s = self.types.as_struct(self.types.repr_struct(t))?;
        self.struct_asts.get(&s).map(|src| src.scope)
    }

    /// Operator overloads visible from the use site, plus those visible where
    /// the operand struct types were declared (`Bit_Array` brings its `operator []`).
    pub(super) fn operator_candidates(
        &mut self,
        scope: ScopeId,
        text: &str,
        operands: &[TypeId],
    ) -> Result<Vec<ProcId>> {
        let name = Sym::intern(&format!("operator{text}"));
        let mut scopes = vec![scope];
        for &t in operands {
            if let Some(home) = self.struct_home_scope(t)
                && !scopes.contains(&home)
            {
                scopes.push(home);
            }
        }
        let mut procs = Vec::new();
        for sc in scopes {
            let r = self.lookup(sc, name);
            for id in r.unwrap_or_default() {
                let found = match self.resolve_entity(id)? {
                    scope::Resolved::Proc(p)
                    | scope::Resolved::Const {
                        value: Value::Proc(p),
                        ..
                    } => vec![p],
                    scope::Resolved::ProcSet(set) => set,
                    _ => continue,
                };
                for p in found {
                    if !procs.contains(&p) {
                        procs.push(p);
                    }
                }
            }
        }
        Ok(procs)
    }

    pub(super) fn overloadable(&self, ty: TypeId) -> bool {
        matches!(
            self.types.kind(ty),
            TypeKind::Struct(_)
                | TypeKind::Array { .. }
                | TypeKind::Pointer(_)
                | TypeKind::Enum(_)
                | TypeKind::Distinct(_)
        )
    }

    pub fn try_binary_operator_overload(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        op: ast::BinOp,
        lhs: &Operand,
        rhs: &Operand,
        span: Span,
    ) -> Result<Option<Operand>> {
        let (lt, rt) = (lhs.ty(), rhs.ty());
        if !(self.overloadable(lt) || self.overloadable(rt)) || matches!(lhs, Operand::Type(_)) {
            return Ok(None);
        }
        if (self.types.is_pointer(lt) || matches!(self.types.kind(lt), TypeKind::Enum(_)))
            && !matches!(self.types.kind(rt), TypeKind::Struct(_))
        {
            return Ok(None);
        }
        let text = binop_text(op);
        let procs = self.operator_candidates(scope, text, &[lt, rt])?;
        // `a != b` without a matching `operator !=` is `!(a == b)`.
        let ne_fallback = if op == ast::BinOp::Ne {
            self.operator_candidates(scope, "==", &[lt, rt])?
        } else {
            Vec::new()
        };
        if procs.is_empty() && ne_fallback.is_empty() {
            return Ok(None);
        }
        let args = vec![
            CallArg {
                name: None,
                spread: false,
                expr: None,
                op: Some(lhs.clone()),
                span,
                scope,
            },
            CallArg {
                name: None,
                spread: false,
                expr: None,
                op: Some(rhs.clone()),
                span,
                scope,
            },
        ];
        if !procs.is_empty()
            && let Ok(r) = self.try_call(f, scope, &procs, args.clone(), span)
        {
            return Ok(Some(r));
        }
        if !ne_fallback.is_empty()
            && let Ok(r) = self.try_call(f, scope, &ne_fallback, args.clone(), span)
        {
            let (ty, v) = self.rvalue(f, r, span)?;
            let b = self.truthy(f, ty, v, span)?;
            let one = f.b.iconst(ir::Ty::I8, 1);
            return Ok(Some(Operand::Value {
                ty: TypeId::BOOL,
                val: f.b.bin(ir::BinOp::Xor, ir::Ty::I8, b, one),
            }));
        }
        if procs.is_empty() {
            return Ok(None);
        }
        // #symmetric operators accept swapped operands.
        let symmetric: Vec<ProcId> = procs
            .iter()
            .copied()
            .filter(|&p| self.proc(p).lit.header.flags.symmetric)
            .collect();
        if !symmetric.is_empty() {
            let swapped = vec![args[1].clone(), args[0].clone()];
            if let Ok(r) = self.try_call(f, scope, &symmetric, swapped, span) {
                return Ok(Some(r));
            }
        }
        if matches!(self.types.kind(lt), TypeKind::Struct(_))
            || matches!(self.types.kind(rt), TypeKind::Struct(_))
        {
            // Report the overload failure for struct operands.
            self.call_procs(f, scope, &procs, args, None, span)
                .map(Some)
        } else {
            Ok(None)
        }
    }

    pub(super) fn try_call(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        procs: &[ProcId],
        args: Vec<CallArg>,
        span: Span,
    ) -> Result<Operand> {
        let mut best: Option<Candidate> = None;
        for &p in procs {
            if let Ok(c) = self.match_candidate(p, &args, span)
                && best.as_ref().is_none_or(|b| c.cost < b.cost)
            {
                best = Some(c);
            }
        }
        match best {
            Some(c) => self.emit_call(f, scope, c, args, span),
            None => err(span, "no operator overload matches"),
        }
    }

    pub fn try_unary_operator_overload(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        op: ast::UnOp,
        operand: &Operand,
        span: Span,
    ) -> Result<Option<Operand>> {
        if !matches!(self.types.kind(operand.ty()), TypeKind::Struct(_)) {
            return Ok(None);
        }
        let text = match op {
            ast::UnOp::Neg => "-",
            ast::UnOp::Not => "!",
            ast::UnOp::BitNot => "~",
            _ => return Ok(None),
        };
        let procs = self.operator_candidates(scope, text, &[operand.ty()])?;
        if procs.is_empty() {
            return Ok(None);
        }
        let args = vec![CallArg {
            name: None,
            spread: false,
            expr: None,
            op: Some(operand.clone()),
            span,
            scope,
        }];
        self.call_procs(f, scope, &procs, args, None, span)
            .map(Some)
    }

    pub fn try_index_operator_overload(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        text: &str,
        base: &Operand,
        index: &Operand,
        span: Span,
    ) -> Result<Option<Operand>> {
        let bt = base.ty();
        let target = self.types.pointee(bt).unwrap_or(bt);
        if !matches!(self.types.kind(target), TypeKind::Struct(_)) {
            return Ok(None);
        }
        let procs = self.operator_candidates(scope, text, &[bt])?;
        if procs.is_empty() {
            return Ok(None);
        }
        let args = vec![
            CallArg {
                name: None,
                spread: false,
                expr: None,
                op: Some(base.clone()),
                span,
                scope,
            },
            CallArg {
                name: None,
                spread: false,
                expr: None,
                op: Some(index.clone()),
                span,
                scope,
            },
        ];
        let r = self.call_procs(f, scope, &procs, args, None, span)?;
        // `operator []` returning a pointer is an lvalue.
        Ok(Some(match r {
            Operand::Value {
                ty,
                val,
            } if self.types.pointee(ty).is_some_and(|p| p != TypeId::VOID)
                && self.types.pointee(ty) != Some(target) =>
            {
                let _ = val;
                Operand::Value {
                    ty,
                    val,
                }
            }
            other => other,
        }))
    }
}

pub fn binop_text(op: ast::BinOp) -> &'static str {
    use ast::BinOp::*;
    match op {
        Add => "+",
        Sub => "-",
        Mul => "*",
        Div => "/",
        Rem => "%",
        BitAnd => "&",
        BitOr => "|",
        BitXor => "^",
        Shl => "<<",
        Shr => ">>",
        Rotl => "<<<",
        Rotr => ">>>",
        Eq => "==",
        Ne => "!=",
        Lt => "<",
        Le => "<=",
        Gt => ">",
        Ge => ">=",
        And => "&&",
        Or => "||",
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
                return err(arg.span, format!("no parameter named '{name}'"));
            };
            if slots[p].is_some() {
                return err(arg.span, format!("parameter '{name}' given twice"));
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
            return err(
                arg.span,
                format!("too many arguments (expected at most {})", params.len()),
            );
        }
        if Some(positional) == variadic_index {
            if arg.spread {
                slots[positional] = Some(Slot::Spread(i));
                positional += 1;
                continue;
            }
            match &mut slots[positional] {
                Some(Slot::Variadic(list)) => list.push(i),
                None => slots[positional] = Some(Slot::Variadic(vec![i])),
                _ => return err(arg.span, "variadic parameter already given"),
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
