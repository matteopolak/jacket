//! Inferring `$T` bindings of a polymorphic procedure from its arguments, and matching type
//! patterns.

use crate::sema::lower::Origin;
use crate::types::Types;

use super::poly_hints::{Binders, CONFLICT};
use super::{
    ArrayKind, CallArg, Compiler, Diagnostic, DiagnosticKind, E, Operand, ProcId, Result, ScopeId,
    Slot, Span, Sym, TypeId, TypeKind, Value, ast, err, header_poly_names, is_deferred,
    is_poly_var_arg, poly_names, procs,
};

/// What `infer_bindings` accumulates while it walks the parameters of a polymorphic header.
struct Inference<'a> {
    header: &'a ast::ProcHeader,
    slots: &'a [Slot],
    args: &'a [CallArg],
    span: Span,
    def_scope: ScopeId,
    poly_vars: Vec<Sym>,
    bindings: Vec<(Sym, Value, TypeId)>,
    deferred_procs: Vec<(ast::Expr, Vec<ProcId>, Span)>,
    null_patterns: Vec<ast::Expr>,
    deferred_defaults: Vec<(Sym, &'a ast::Param, &'a ast::Expr)>,
    /// The first argument to bind each variable, for the notes of a failure.
    binders: Binders,
}

impl Compiler {
    /// Infer `$T` bindings of a polymorphic procedure from its arguments.
    pub(super) fn infer_bindings(
        &mut self,
        proc: ProcId,
        header: &ast::ProcHeader,
        slots: &[Slot],
        args: &[CallArg],
        span: Span,
    ) -> Result<Vec<(Sym, Value, TypeId)>> {
        let mut st = Inference {
            header,
            slots,
            args,
            span,
            def_scope: self.proc(proc).scope,
            poly_vars: header_poly_names(header),
            bindings: Vec::new(),
            deferred_procs: Vec::new(),
            null_patterns: Vec::new(),
            deferred_defaults: Vec::new(),
            binders: Vec::new(),
        };
        self.bind_explicit_poly_args(&mut st)?;
        for (i, param) in header.params.iter().enumerate() {
            let arg_ops: Vec<&CallArg> = match &slots[i] {
                Slot::Arg(a) | Slot::Spread(a) => vec![&args[*a]],
                Slot::Variadic(list) => list.iter().map(|&a| &args[a]).collect(),
                Slot::Default => Vec::new(),
            };
            if param.baked {
                self.infer_baked_param(&mut st, i, param, &arg_ops)?;
            } else {
                self.infer_pattern_param(&mut st, param, &arg_ops)?;
            }
        }
        self.infer_lambda_params(&mut st)?;
        self.settle_deferred_bindings(&mut st)?;
        self.bind_omitted_defaults(&mut st)?;
        Self::check_literal_defaults(&st)?;
        if header.modify.is_none() {
            self.require_all_bound(&mut st)?;
        }
        let Inference {
            bindings, ..
        } = st;
        match &header.modify {
            Some(block) => self.run_modify(proc, header, block, bindings, span),
            None => Ok(bindings),
        }
    }

    /// Binds the `$T` parameters that were passed explicitly.
    fn bind_explicit_poly_args(&mut self, st: &mut Inference<'_>) -> Result<()> {
        let args = st.args;
        let header = st.header;
        for arg in args {
            if !is_poly_var_arg(&header.params, arg, &st.poly_vars) {
                continue;
            }
            let name = arg.name.unwrap();
            let op = match (&arg.op, &arg.expr) {
                (Some(op), _) => op.clone(),
                (None, Some(e)) => self.eval_const(arg.scope, e, None)?,
                (None, None) => return err(arg.span, "missing value"),
            };
            let Operand::Type(t) = op else {
                return err(arg.span, format!("`${name}` must be given a type"));
            };
            st.bindings.push((name, Value::Type(t), TypeId::TYPE));
        }
        Ok(())
    }

    /// Binds a `$name` constant parameter from its argument or default.
    fn infer_baked_param<'a>(
        &mut self,
        st: &mut Inference<'a>,
        i: usize,
        param: &'a ast::Param,
        arg_ops: &[&CallArg],
    ) -> Result<()> {
        let def_scope = st.def_scope;
        let span = st.span;
        let args = st.args;
        let slots = st.slots;
        if param.baked && param.variadic {
            let name = param.name.map(|n| n.name).unwrap();
            // `..names` passes on a constant view (another baked variadic) as it is.
            if let Slot::Spread(a) = &slots[i] {
                let (value, ty) = self.baked_spread(def_scope, param, &args[*a])?;
                st.bindings.push((name, value, ty));
                return Ok(());
            }
            let (value, ty) = self.baked_pack(def_scope, param, arg_ops)?;
            st.bindings.push((name, value, ty));
            return Ok(());
        }
        if param.baked {
            let name = param.name.map(|n| n.name).unwrap();
            let Some(arg) = arg_ops.first() else {
                if let Some(d) = &param.default {
                    // `$type: Query = .X`: the default takes the declared type. A type
                    // naming a binding only `null` can make (`is(null)` with
                    // `$cmp: (T, T) -> bool = null`) waits until `null` has bound it.
                    match self.baked_default(def_scope, param, d, &st.bindings, span) {
                        Ok((v, ty)) => {
                            // `$$im: $U = 0.0`: the default's type binds `$U`, which a
                            // `#modify` block reads.
                            if let Some(pattern) = &param.ty
                                && procs::has_poly(pattern)
                            {
                                let value_ty = self.ty.default_untyped(ty, &v);
                                self.match_pattern(pattern, value_ty, &mut st.bindings, def_scope)?;
                            }
                            st.bindings.push((name, v, ty));
                        }
                        Err(_) if !st.null_patterns.is_empty() => {
                            st.deferred_defaults.push((name, param, d));
                        }
                        Err(e) => return Err(e),
                    }
                    return Ok(());
                }
                return err(
                    span,
                    format!("missing argument for baked parameter `{name}`"),
                );
            };
            // `$c: Code` takes the argument expression itself, unevaluated.
            if let Some(t) = &param.ty
                && !procs::has_poly(t)
                && self.eval_type(def_scope, t).ok() == Some(TypeId::CODE)
                && let Some(id) = self.quote_code_arg(arg)
            {
                st.bindings.push((name, Value::Code(id), TypeId::CODE));
                return Ok(());
            }
            let op = if let Some(op) = arg.op.clone() {
                op
            } else {
                // `.{...}` / `.X` for a baked parameter: check against its declared
                // type, or the type of its default (`$info := Info.{}`).
                let declared = match (&param.ty, &param.default) {
                    (Some(t), _) if !procs::has_poly(t) => Some(self.eval_type(def_scope, t)?),
                    (None, Some(d)) => Some(self.eval_const(def_scope, d, None)?.ty()),
                    _ => None,
                };
                // An unfinished argument (the one completion asks about) binds nothing.
                let Some(expr) = arg.expr.clone() else {
                    return Ok(());
                };
                let op = self.eval_const(arg.scope, &expr, declared)?;
                match declared {
                    Some(t) => {
                        let value = self.const_value_of_type(arg.scope, &expr, t)?;
                        Operand::Const {
                            ty: t,
                            value,
                            untyped: false,
                            origin: Origin::Computed,
                        }
                    }
                    None => op,
                }
            };
            let op_ty = op.ty();
            let value_ty = match &op {
                Operand::Const {
                    ty,
                    value,
                    untyped: true,
                    ..
                } => self.ty.default_untyped(*ty, value),
                Operand::Const {
                    ..
                }
                | Operand::Type(_) => op_ty,
                _ => TypeId::VOID,
            };
            // A number-typed constant parameter takes its argument as a normal parameter does:
            // converted to the declared type, or rejected.
            let scalar_declared = match (&param.ty, &param.default) {
                // A view-typed parameter (`$words: [] string`) takes an array argument as a view
                // too, so the bound constant is the `(count, data)` pair and not the array.
                (Some(t), _) if !procs::has_poly(t) => self
                    .eval_type(def_scope, t)
                    .ok()
                    .filter(|&dt| self.is_scalar_constant_type(dt) || self.is_view_type(dt)),
                // `$flag := true` is a `bool`: an untyped parameter has the type of its default.
                (None, Some(d)) => self
                    .eval_const(def_scope, d, None)
                    .ok()
                    .map(|op| match op {
                        Operand::Const {
                            ty,
                            value,
                            untyped: true,
                            ..
                        } => self.ty.default_untyped(ty, &value),
                        other => other.ty(),
                    })
                    .filter(|&dt| self.is_scalar_constant_type(dt)),
                _ => None,
            };
            // Any other declared type rejects an argument of a type it does not accept, so
            // overloads `($c: A)` and `($c: B)` of distinct types tell their arguments apart.
            if scalar_declared.is_none()
                && arg.op.is_some()
                && matches!(
                    &op,
                    Operand::Const {
                        value: Value::Bytes(_) | Value::Int(_) | Value::Float(..),
                        ..
                    }
                )
                && let Some(t) = &param.ty
                && !procs::has_poly(t)
                && let Ok(dt) = self.eval_type(def_scope, t)
                && dt != TypeId::TYPE
                && dt != TypeId::CODE
            {
                self.arg_cost(arg, dt, false)?;
            }
            let op = match (scalar_declared, &op) {
                (
                    Some(dt),
                    Operand::Const {
                        ..
                    },
                ) => {
                    if arg.op.is_some() {
                        self.arg_cost(arg, dt, false)?;
                    }
                    let value = self.const_value_of_operand(arg.scope, op.clone(), dt, arg.span)?;
                    Operand::Const {
                        ty: dt,
                        value,
                        untyped: false,
                        origin: Origin::Computed,
                    }
                }
                _ => op,
            };
            let value = match op {
                Operand::Type(t) => Value::Type(t),
                Operand::Const {
                    value, ..
                } => value,
                Operand::Procs(p) if p.len() == 1 => Value::Proc(p[0]),
                Operand::Procs(p) if !self.proc(p[0]).is_poly => {
                    Value::Proc(self.overload_for_baked(&p, param, def_scope, arg.span)?)
                }
                other => {
                    // A member of a constant (`help.build`) is only known to be one when
                    // evaluated as a constant expression.
                    let folded = match &arg.expr {
                        Some(expr) => match self.eval_const(arg.scope, expr, None) {
                            Ok(Operand::Const {
                                value, ..
                            }) => Some(value),
                            _ => None,
                        },
                        None => None,
                    };
                    let _ = other;
                    match folded {
                        Some(value) => value,
                        None => {
                            return err(
                                arg.span,
                                format!("argument for `${name}` must be a compile-time constant"),
                            );
                        }
                    }
                }
            };
            let value_ty = if value_ty == TypeId::VOID {
                match self.type_of_value(&value) {
                    TypeId::VOID => op_ty,
                    ty => ty,
                }
            } else {
                value_ty
            };
            // The declared type may itself be polymorphic (`$T: Type`), or mention
            // another parameter's binding (`$compare: (T, T) -> bool`).
            let ty = match &param.ty {
                Some(t) if !procs::has_poly(t) => match self.eval_type(def_scope, t) {
                    Ok(ty) => ty,
                    // The declared type names an earlier binding (`$help: Help(T)`).
                    Err(_) => value_ty,
                },
                // Aggregate constants carry no type of their own.
                _ => value_ty,
            };
            if let Some(t) = &param.ty
                && procs::has_poly(t)
            {
                // Aggregate constants (`.[1, 2]`) carry no type of their own.
                let mut vt = value_ty;
                // A constant view (`xs :: #run make()`) gives a `[$N] T` pattern its count.
                if let Value::Bytes(image) = &value
                    && let TypeKind::Array {
                        elem,
                        kind: ArrayKind::View,
                    } = *self.ty.types.kind(vt)
                    && matches!(
                        &t.kind,
                        E::ArrayType {
                            size: ast::ArraySize::Fixed(size),
                            ..
                        } if matches!(size.kind, E::PolyVar { .. })
                    )
                    && let Some(bytes) = image.bytes.get(0..8)
                {
                    let count = u64::from_le_bytes(bytes.try_into().unwrap());
                    vt = self.ty.types.array(elem, ArrayKind::Fixed(count));
                }
                let represented = if let Value::Type(t) = &value {
                    Some(*t)
                } else {
                    None
                };
                self.match_pattern_value(t, vt, &mut st.bindings, def_scope, represented)?;
            }
            st.bindings.push((name, value, ty));
            return Ok(());
        }
        Ok(())
    }

    /// Binds the type variables of a parameter type pattern from its arguments.
    fn infer_pattern_param<'a>(
        &mut self,
        st: &mut Inference<'a>,
        param: &'a ast::Param,
        arg_ops: &[&CallArg],
    ) -> Result<()> {
        let def_scope = st.def_scope;
        let Some(pattern) = &param.ty else {
            return Ok(());
        };
        let implicit =
            !procs::has_poly(pattern) && self.mentions_poly_struct(def_scope, pattern)?;
        if !procs::has_poly(pattern) && !implicit {
            return Ok(());
        }
        for (k, arg) in arg_ops.iter().enumerate() {
            // An untyped `.[...]` argument fixes the count of a `[$N] T` parameter.
            if !param.variadic
                && arg.op.is_none()
                && let Some(e) = &arg.expr
                && Self::bind_literal_count(pattern, e, &mut st.bindings)?
            {
                continue;
            }
            // `ifx` waits for a target type, but its branches can bind `$T` themselves. An
            // argument kept as an expression (the overload set has a macro taking `Code`
            // there) still binds this procedure's type variables.
            let evaluated;
            let op = match (&arg.op, &arg.expr) {
                (Some(op), _) => op,
                (None, _) if let Some(op) = self.ifx_binding_operand(arg) => {
                    evaluated = op;
                    &evaluated
                }
                (None, Some(e)) if !is_deferred(e) => match self.check_expr_no_emit(arg.scope, e) {
                    Ok(op) => {
                        evaluated = op;
                        &evaluated
                    }
                    Err(_) => continue,
                },
                _ => continue,
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
                st.null_patterns.push(pattern.clone());
                continue;
            }
            // An overloaded or polymorphic procedure passed for a procedure-typed
            // parameter is resolved once the other st.bindings are known.
            if let Operand::Procs(ps) = op
                && (ps.len() > 1 || self.proc(ps[0]).is_poly)
                && matches!(pattern.kind, E::ProcType(_))
            {
                st.deferred_procs
                    .push((pattern.clone(), ps.clone(), arg.span));
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
                    ..
                } => self.ty.default_untyped(*ty, value),
                Operand::Type(_) => TypeId::TYPE,
                // An overload set binds the type of its first member.
                Operand::Procs(p) if !self.proc(p[0]).is_poly => self.proc_type(p[0], arg.span)?,
                other => other.ty(),
            };
            if param.variadic
                && arg.spread
                && let TypeKind::Array {
                    elem, ..
                } = self.ty.types.kind(ty)
            {
                ty = *elem;
            }
            let represented = match op.const_value() {
                Some(Value::Type(t)) => Some(t),
                _ => None,
            };
            if k > 0 && param.variadic {
                // Later variadic elements must match the first binding.
                let mut probe = st.bindings.clone();
                if let Err(error) =
                    self.match_pattern_value(pattern, ty, &mut probe, def_scope, represented)
                {
                    if error.kind == DiagnosticKind::InterfaceConstraint {
                        return Err(error);
                    }
                    continue;
                }
            }
            let before = st.bindings.len();
            if let Err(error) =
                self.match_pattern_value(pattern, ty, &mut st.bindings, def_scope, represented)
            {
                return Err(Box::new(self.with_pattern_notes(
                    *error,
                    pattern,
                    arg,
                    ty,
                    &st.binders,
                )));
            }
            for (name, _, _) in &st.bindings[before..] {
                st.binders.push((*name, arg.span, ty));
            }
            if implicit && !param.variadic {
                self.bind_own_instance(st, param, pattern, ty)?;
            }
            // Later parameter types may use this one's instance (`item: array.type`).
            if implicit
                && k == 0
                && let Some(n) = param.name
                && !st.bindings.iter().any(|(b, _, _)| *b == n.name)
            {
                st.bindings.push((n.name, Value::Type(ty), TypeId::TYPE));
            }
        }
        Ok(())
    }

    /// The member of an overload set a baked parameter takes: the one with the parameter's
    /// declared procedure type, or the first when the type is left to `$F`.
    fn overload_for_baked(
        &mut self,
        procs: &[ProcId],
        param: &ast::Param,
        def_scope: ScopeId,
        span: Span,
    ) -> Result<ProcId> {
        let Some(declared) = param.ty.as_ref().filter(|t| !procs::has_poly(t)) else {
            return Ok(procs[0]);
        };
        let want = self.eval_type(def_scope, declared)?;
        for &p in procs {
            if self.proc(p).is_poly {
                continue;
            }
            let ty = self.proc_type(p, span)?;
            if ty == want || self.ty.proc_types_compatible(ty, want) {
                return Ok(p);
            }
        }
        err(
            span,
            format!(
                "procedure does not match parameter type {}",
                self.ty.types.name(want)
            ),
        )
    }

    /// A bare polymorphic struct parameter named `Table` is its own instance, even when an
    /// earlier parameter already bound `Table` to another one: the instance is kept under the
    /// parameter's name for its signature.
    fn bind_own_instance(
        &mut self,
        st: &mut Inference<'_>,
        param: &ast::Param,
        pattern: &ast::Expr,
        arg_ty: TypeId,
    ) -> Result<()> {
        let (Some(param_name), Some((name, ps, ty))) = (param.name, {
            let (mut pattern, mut ty) = (pattern, arg_ty);
            while let E::Unary(ast::UnOp::Star, inner) = &pattern.kind
                && let Some(p) = self.ty.types.pointee(ty)
            {
                pattern = inner;
                ty = p;
            }
            match &pattern.kind {
                E::Ident(name) => self
                    .ident_poly_struct(st.def_scope, *name)?
                    .map(|ps| (*name, ps, ty)),
                _ => None,
            }
        }) else {
            return Ok(());
        };
        let matched = if self.instance_or_as_base(ps, ty).is_some() {
            Some(ty)
        } else {
            self.ty
                .types
                .pointee(ty)
                .filter(|&p| self.instance_or_as_base(ps, p).is_some())
        };
        if let Some(t) = matched
            && st
                .bindings
                .iter()
                .any(|(n, v, _)| *n == name && *v != Value::Type(t))
        {
            st.bindings.push((
                own_instance_key(param_name.name),
                Value::Type(t),
                TypeId::TYPE,
            ));
        }
        Ok(())
    }

    /// Lambdas are checked last: their parameter types come from the other bindings.
    fn infer_lambda_params(&mut self, st: &mut Inference<'_>) -> Result<()> {
        let def_scope = st.def_scope;
        let args = st.args;
        let slots = st.slots;
        let header = st.header;
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
                    &mut st.bindings,
                    *span,
                )?;
            }
        }
        Ok(())
    }

    /// Resolves what waited for the other bindings: procedures, `null` and defaults.
    fn settle_deferred_bindings(&mut self, st: &mut Inference<'_>) -> Result<()> {
        let def_scope = st.def_scope;
        let span = st.span;
        for (pattern, ps, arg_span) in std::mem::take(&mut st.deferred_procs) {
            let E::ProcType(ph) = &pattern.kind else {
                continue;
            };
            let known = self.const_scope(def_scope, st.bindings.clone(), arg_span);
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
            self.match_pattern(&pattern, ty, &mut st.bindings, def_scope)?;
        }
        // A type variable seen only through `null` is `*void`.
        let void_ptr = self.ty.types.pointer(TypeId::VOID);
        for pattern in std::mem::take(&mut st.null_patterns) {
            let mut probe = st.bindings.clone();
            if self
                .match_pattern(&pattern, void_ptr, &mut probe, def_scope)
                .is_ok()
            {
                st.bindings = probe;
            }
        }
        for (name, param, d) in std::mem::take(&mut st.deferred_defaults) {
            let (v, ty) = self.baked_default(def_scope, param, d, &st.bindings, span)?;
            st.bindings.push((name, v, ty));
        }
        Ok(())
    }

    /// Binds what only the default of an omitted argument determines.
    fn bind_omitted_defaults(&mut self, st: &mut Inference<'_>) -> Result<()> {
        let def_scope = st.def_scope;
        let span = st.span;
        let slots = st.slots;
        let header = st.header;
        for (i, param) in header.params.iter().enumerate() {
            let (Slot::Default, Some(pattern), Some(default)) =
                (&slots[i], &param.ty, &param.default)
            else {
                continue;
            };
            if param.baked
                || poly_names(pattern)
                    .iter()
                    .all(|name| st.bindings.iter().any(|(n, _, _)| n == name))
            {
                continue;
            }
            if Self::bind_literal_count(pattern, default, &mut st.bindings)? {
                continue;
            }
            let known = self.const_scope(def_scope, st.bindings.clone(), span);
            let ty = match self.eval_const(known, default, None)? {
                Operand::Const {
                    ty,
                    value,
                    untyped: true,
                    ..
                } => self.ty.default_untyped(ty, &value),
                Operand::Type(_) => TypeId::TYPE,
                other => other.ty(),
            };
            self.match_pattern(pattern, ty, &mut st.bindings, def_scope)?;
        }
        Ok(())
    }

    /// `[$N] T` against an untyped `.[a, b, c]`: binds `$N` to the element count, or fails
    /// when `$N` is already bound to another count. Returns whether the literal was such an
    /// argument for such a pattern.
    fn bind_literal_count(
        pattern: &ast::Expr,
        literal: &ast::Expr,
        bindings: &mut Vec<(Sym, Value, TypeId)>,
    ) -> Result<bool> {
        let (
            E::ArrayType {
                size: ast::ArraySize::Fixed(size),
                ..
            },
            E::ArrayLit {
                ty: None,
                elems,
            },
        ) = (&pattern.kind, &literal.kind)
        else {
            return Ok(false);
        };
        let E::PolyVar {
            name, ..
        } = &size.kind
        else {
            return Ok(false);
        };
        let given = elems.len() as i128;
        match bindings.iter().find(|(bound, _, _)| bound == name) {
            Some((_, Value::Int(wanted), _)) if *wanted != given => err(
                literal.span,
                format!("unequal array lengths (wanted {wanted}, given {given})"),
            ),
            Some(_) => Ok(true),
            None => {
                bindings.push((*name, Value::Int(given), TypeId::S64));
                Ok(true)
            }
        }
    }

    /// A default `.[...]` for a `[$N] T` parameter must have `N` elements once `$N` is
    /// bound, whether or not the call passed that argument.
    fn check_literal_defaults(st: &Inference<'_>) -> Result<()> {
        for param in &st.header.params {
            if let (Some(pattern), Some(default)) = (&param.ty, &param.default) {
                Self::bind_literal_count(pattern, default, &mut st.bindings.clone())?;
            }
        }
        Ok(())
    }

    /// Every type variable of the header must be bound unless a `#modify` block may bind it.
    fn require_all_bound(&mut self, st: &mut Inference<'_>) -> Result<()> {
        let span = st.span;
        let header = st.header;
        // Defaults of the form `$T` without arguments are an error unless bound elsewhere.
        // A `#modify` block may still bind what the arguments did not determine.
        {
            for param in &header.params {
                if let Some(t) = &param.ty {
                    for name in poly_names(t) {
                        if !st.bindings.iter().any(|(n, _, _)| *n == name) {
                            let d = Diagnostic::error(
                                span,
                                format!("could not infer polymorphic type `${name}`"),
                            );
                            return Err(Box::new(
                                self.with_unbound_notes(d, header, st.slots, st.args, name),
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Match a polymorphic type pattern against a concrete type, adding bindings.
    pub fn match_pattern(
        &mut self,
        pattern: &ast::Expr,
        ty: TypeId,
        bindings: &mut Vec<(Sym, Value, TypeId)>,
        scope: ScopeId,
    ) -> Result<()> {
        self.match_pattern_value(pattern, ty, bindings, scope, None)
    }

    /// `Table($K, $V)` against an instance of the same polymorphic struct
    /// (a `*Instance` argument is dereferenced for a by-value parameter).
    fn match_instance_pattern(
        &mut self,
        callee: &ast::Expr,
        args: &[ast::Arg],
        ty: TypeId,
        bindings: &mut Vec<(Sym, Value, TypeId)>,
        scope: ScopeId,
        span: Span,
    ) -> Result<()> {
        let ty = match self.ty.types.pointee(ty) {
            Some(p) if self.ty.types.as_struct(p).is_some() => p,
            _ => ty,
        };
        // The instance itself, or (for a named struct) an `#as` member that is one.
        let (instance, parameters) = match &callee.kind {
            E::Ident(name) => match self.ident_poly_struct(scope, *name)? {
                Some(ps) => (
                    self.instance_or_as_base(ps, ty),
                    Some(self.ty.poly_structs[ps.0 as usize].lit.params.clone()),
                ),
                None => (None, None),
            },
            _ => match self.eval_const(scope, callee, None)? {
                Operand::PolyStruct(ps) => (
                    self.instance_or_as_base(ps, ty),
                    Some(self.ty.poly_structs[ps.0 as usize].lit.params.clone()),
                ),
                _ => (self.ty.types.as_struct(ty).map(|_| ty), None),
            },
        };
        let Some(ty) = instance else {
            return err(
                span,
                format!(
                    "{} is not an instance of this struct",
                    self.ty.types.name(ty)
                ),
            );
        };
        let s = self.ty.types.as_struct(ty).unwrap();
        let parameters = parameters
            .or_else(|| {
                self.ty
                    .struct_asts
                    .get(&s)
                    .map(|src| src.lit.params.clone())
            })
            .unwrap_or_default();
        // Instance keys store values, not their declared types, and may contain
        // inferred type bindings absent from the written parameter list. Match
        // that list by parameter identity against the canonical typed bindings.
        let actual = self.poly_struct_bindings(s);
        let mut position = 0;
        for a in args {
            let parameter = if let Some(name) = a.name {
                parameters
                    .iter()
                    .find(|p| p.name.map(|n| n.name) == Some(name.name))
            } else {
                let parameter = parameters.get(position);
                position += 1;
                parameter
            };
            let Some(name) = parameter.and_then(|p| p.name).map(|n| n.name) else {
                return err(a.value.span, "unknown polymorphic struct parameter");
            };
            let Some((_, value, value_type)) = actual.iter().find(|(n, _, _)| *n == name) else {
                return err(a.value.span, "missing polymorphic struct parameter binding");
            };
            match &a.value.kind {
                E::PolyVar {
                    name, ..
                } => {
                    bind_pattern(
                        span,
                        &self.ty.types,
                        bindings,
                        *name,
                        value.clone(),
                        *value_type,
                    )?;
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

    pub(in crate::sema) fn match_pattern_value(
        &mut self,
        pattern: &ast::Expr,
        ty: TypeId,
        bindings: &mut Vec<(Sym, Value, TypeId)>,
        scope: ScopeId,
        represented: Option<TypeId>,
    ) -> Result<()> {
        let span = pattern.span;
        match &pattern.kind {
            E::PolyVar {
                name, ..
            } => bind_pattern(
                span,
                &self.ty.types,
                bindings,
                *name,
                Value::Type(ty),
                TypeId::TYPE,
            ),
            // A bare polymorphic struct matches any of its instances; binding its
            // name makes the instance signature name the instance.
            E::Ident(name) => match self.ident_poly_struct(scope, *name)? {
                Some(ps) => {
                    // A `*Instance` argument is dereferenced for a by-value parameter.
                    // A struct with an `#as` instance member keeps its own type, so the
                    // body sees its members too (n00bmind/reflector's `r: *BinaryReflector`
                    // takes a `*BinaryWriter` and reaches `r.buffer`).
                    let matched = if self.instance_or_as_base(ps, ty).is_some() {
                        Some(ty)
                    } else {
                        self.ty
                            .types
                            .pointee(ty)
                            .filter(|&p| self.instance_or_as_base(ps, p).is_some())
                    };
                    match matched {
                        // Each bare `Table` parameter is its own instance: the name stays
                        // bound to the first.
                        Some(_) if bindings.iter().any(|b| b.0 == *name) => Ok(()),
                        Some(t) => bind_pattern(
                            span,
                            &self.ty.types,
                            bindings,
                            *name,
                            Value::Type(t),
                            TypeId::TYPE,
                        ),
                        None => err(
                            span,
                            format!("{} is not an instance of `{name}`", self.ty.types.name(ty)),
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
                if *interface {
                    let checked = (|| {
                        let candidate = if ty == TypeId::TYPE {
                            represented.ok_or_else(|| {
                                Box::new(Diagnostic::error(
                                    span,
                                    "interface constraint requires a concrete type binding",
                                ))
                            })?
                        } else {
                            ty
                        };
                        let required = self.eval_type(scope, restriction)?;
                        self.validate_interface(candidate, required, span)
                    })();
                    checked.map_err(|error| {
                        Box::new((*error).with_kind(DiagnosticKind::InterfaceConstraint))
                    })?;
                }
                // `$T/Table`: T must be an instance of the polymorphic struct `Table` (or hold one
                // as an `#as` member).
                let mut ty = ty;
                if !interface
                    && let E::Ident(r) = &restriction.kind
                    && let Some(ps) = self.ident_poly_struct(scope, *r)?
                    && self.instance_or_as_base(ps, ty).is_none()
                {
                    // A `*Instance` argument is dereferenced for a by-value parameter.
                    match self.ty.types.pointee(ty) {
                        Some(p) if self.instance_or_as_base(ps, p).is_some() => ty = p,
                        _ => {
                            return err(
                                span,
                                format!("{} is not an instance of `{r}`", self.ty.types.name(ty)),
                            );
                        }
                    }
                }
                // `$T/Entity`: a struct argument must be `Entity` or have it as an `#as` base.
                if !interface
                    && !self.ty.in_element_pattern
                    && let E::Ident(_) | E::Member(..) = &restriction.kind
                    && self.ident_poly_struct_expr(scope, restriction)?.is_none()
                    && let Ok(want) = self.eval_type(scope, restriction)
                    && self.ty.types.as_struct(want).is_some()
                    && self.ty.types.as_struct(ty).is_some()
                    && !self.has_as_base(ty, want)
                {
                    return err(
                        span,
                        format!(
                            "{} does not satisfy the restriction `{}`",
                            self.ty.types.name(ty),
                            self.ty.types.name(want)
                        ),
                    );
                }
                // `$T/.[A, B]` (or a constant naming such a list): T must be one of the types.
                if !interface
                    && let Some(options) = self.restriction_type_list(scope, restriction, 0)?
                    && !options.contains(&ty)
                {
                    return err(
                        span,
                        format!(
                            "{} is not one of the types the restriction allows",
                            self.ty.types.name(ty)
                        ),
                    );
                }
                bind_pattern(
                    span,
                    &self.ty.types,
                    bindings,
                    *name,
                    Value::Type(ty),
                    TypeId::TYPE,
                )
            }
            E::Unary(ast::UnOp::Star, inner) => match self.ty.types.kind(ty).clone() {
                TypeKind::Pointer(p) => self.match_pattern(inner, p, bindings, scope),
                _ => err(
                    span,
                    format!("expected a pointer, found {}", self.ty.types.name(ty)),
                ),
            },
            E::ArrayType {
                size,
                elem,
            } => {
                let TypeKind::Array {
                    elem: actual,
                    kind,
                } = self.ty.types.kind(ty).clone()
                else {
                    return err(
                        span,
                        format!("expected an array, found {}", self.ty.types.name(ty)),
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
                            bind_pattern(
                                span,
                                &self.ty.types,
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
                            format!("array kind mismatch with {}", self.ty.types.name(ty)),
                        );
                    }
                }
                let outer = std::mem::replace(&mut self.ty.in_element_pattern, true);
                let matched = self.match_pattern(elem, actual, bindings, scope);
                self.ty.in_element_pattern = outer;
                matched
            }
            E::Call {
                callee,
                args,
                ..
            } => self.match_instance_pattern(callee, args, ty, bindings, scope, span),
            E::ProcType(header) => {
                let TypeKind::Proc(pt) = self.ty.types.kind(ty).clone() else {
                    return err(
                        span,
                        format!("expected a procedure, found {}", self.ty.types.name(ty)),
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
}

/// The binding that holds the instance a bare polymorphic struct parameter was given.
pub(in crate::sema) fn own_instance_key(param: Sym) -> Sym {
    Sym::intern(&format!("\u{0}own:{param}"))
}

/// Binds `$name` to `value` while matching a polymorphic pattern; a second binding must agree.
fn bind_pattern(
    span: Span,
    types: &Types,
    bindings: &mut Vec<(Sym, Value, TypeId)>,
    name: Sym,
    value: Value,
    vty: TypeId,
) -> Result<()> {
    if let Some((_, existing, _)) = bindings.iter().find(|(n, _, _)| *n == name) {
        if *existing != value {
            return err(
                span,
                format!(
                    "{CONFLICT}{name}`: {} and {}",
                    existing.render(types, true),
                    value.render(types, true)
                ),
            );
        }
        return Ok(());
    }
    bindings.push((name, value, vty));
    Ok(())
}
