//! Inferring `$T` bindings of a polymorphic procedure from its arguments, and matching type
//! patterns.

use super::*;

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
        let mut bindings: Vec<(Sym, Value, TypeId)> = Vec::new();
        let mut deferred_procs = Vec::new();
        let mut null_patterns: Vec<ast::Expr> = Vec::new();
        let mut deferred_defaults: Vec<(Sym, &ast::Param, &ast::Expr)> = Vec::new();
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
                return err(arg.span, format!("`${name}` must be given a type"));
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
                        // naming a binding only `null` can make (`is(null)` with
                        // `$cmp: (T, T) -> bool = null`) waits until `null` has bound it.
                        match self.baked_default(def_scope, param, d, &bindings, span) {
                            Ok((v, ty)) => bindings.push((name, v, ty)),
                            Err(_) if !null_patterns.is_empty() => {
                                deferred_defaults.push((name, param, d));
                            }
                            Err(e) => return Err(e),
                        }
                        continue;
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
                let value_ty = match &op {
                    Operand::Const {
                        ty,
                        value,
                        untyped: true,
                    } => self.default_untyped(*ty, value),
                    Operand::Const {
                        ..
                    }
                    | Operand::Type(_) => op_ty,
                    _ => TypeId::VOID,
                };
                let value = match op {
                    Operand::Type(t) => Value::Type(t),
                    Operand::Const {
                        value, ..
                    } => value,
                    Operand::Procs(p) if p.len() == 1 => Value::Proc(p[0]),
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
                                    format!(
                                        "argument for `${name}` must be a compile-time constant"
                                    ),
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
                    let vt = value_ty;
                    let represented = if let Value::Type(t) = &value {
                        Some(*t)
                    } else {
                        None
                    };
                    self.match_pattern_value(t, vt, &mut bindings, def_scope, represented)?;
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
                    (None, Some(e)) if !is_deferred(e) => {
                        match self.check_expr_no_emit(arg.scope, e) {
                            Ok(op) => {
                                evaluated = op;
                                &evaluated
                            }
                            Err(_) => continue,
                        }
                    }
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
                    && arg.spread
                    && let TypeKind::Array {
                        elem, ..
                    } = self.types.kind(ty)
                {
                    ty = *elem;
                }
                let represented = match op.const_value() {
                    Some(Value::Type(t)) => Some(t),
                    _ => None,
                };
                if k > 0 && param.variadic {
                    // Later variadic elements must match the first binding.
                    let mut probe = bindings.clone();
                    if let Err(error) =
                        self.match_pattern_value(pattern, ty, &mut probe, def_scope, represented)
                    {
                        if error.kind == DiagnosticKind::InterfaceConstraint {
                            return Err(error);
                        }
                        continue;
                    }
                }
                self.match_pattern_value(pattern, ty, &mut bindings, def_scope, represented)?;
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
        for (name, param, d) in deferred_defaults {
            let (v, ty) = self.baked_default(def_scope, param, d, &bindings, span)?;
            bindings.push((name, v, ty));
        }
        // An omitted argument binds what only its default determines
        // (`platform_code: $T = 0` called without it: `T` is `s64`).
        for (i, param) in header.params.iter().enumerate() {
            let (Slot::Default, Some(pattern), Some(default)) =
                (&slots[i], &param.ty, &param.default)
            else {
                continue;
            };
            if param.baked
                || poly_names(pattern)
                    .iter()
                    .all(|name| bindings.iter().any(|(n, _, _)| n == name))
            {
                continue;
            }
            let known = self.const_scope(def_scope, bindings.clone(), span);
            let ty = match self.eval_const(known, default, None)? {
                Operand::Const {
                    ty,
                    value,
                    untyped: true,
                } => self.default_untyped(ty, &value),
                Operand::Type(_) => TypeId::TYPE,
                other => other.ty(),
            };
            self.match_pattern(pattern, ty, &mut bindings, def_scope)?;
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
                                format!("could not infer polymorphic type `${name}`"),
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

    /// Match a polymorphic type pattern against a concrete type, adding bindings.
    /// `ty` if it is an instance of `ps`, else the first `#as` member (searched
    /// depth-first) that is one.
    /// Whether struct `ty` is `want` or reaches it through `#as` members.
    pub(super) fn has_as_base(&mut self, ty: TypeId, want: TypeId) -> bool {
        if ty == want {
            return true;
        }
        let Some(s) = self.types.as_struct(ty) else {
            return false;
        };
        if self.layout_struct(s, Span::NONE).is_err() {
            return false;
        }
        let fields = self.types.struct_info(s).fields.clone();
        fields
            .iter()
            .filter(|f| f.as_)
            .any(|f| self.has_as_base(f.ty, want))
    }

    /// The polymorphic struct a type expression names, if it is a bare identifier for one.
    pub(super) fn ident_poly_struct_expr(
        &mut self,
        scope: ScopeId,
        expr: &ast::Expr,
    ) -> Result<Option<value::PolyStructId>> {
        match &expr.kind {
            E::Ident(name) => self.ident_poly_struct(scope, *name),
            _ => Ok(None),
        }
    }

    pub(super) fn instance_or_as_base(
        &mut self,
        ps: value::PolyStructId,
        ty: TypeId,
    ) -> Option<TypeId> {
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
        self.layout_struct(s, Span::NONE).ok()?;
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
        self.match_pattern_value(pattern, ty, bindings, scope, None)
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
                            "conflicting types for `${name}`: {} and {}",
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
                    // A `*Instance` argument is dereferenced for a by-value parameter.
                    // A struct with an `#as` instance member keeps its own type, so the
                    // body sees its members too (n00bmind/reflector's `r: *BinaryReflector`
                    // takes a `*BinaryWriter` and reaches `r.buffer`).
                    let matched = if self.instance_or_as_base(ps, ty).is_some() {
                        Some(ty)
                    } else {
                        self.types
                            .pointee(ty)
                            .filter(|&p| self.instance_or_as_base(ps, p).is_some())
                    };
                    match matched {
                        Some(t) => bind(self, bindings, *name, Value::Type(t), TypeId::TYPE),
                        None => err(
                            span,
                            format!("{} is not an instance of `{name}`", self.types.name(ty)),
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
                    match self.types.pointee(ty) {
                        Some(p) if self.instance_or_as_base(ps, p).is_some() => ty = p,
                        _ => {
                            return err(
                                span,
                                format!("{} is not an instance of `{r}`", self.types.name(ty)),
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
                            "{} does not satisfy the restriction `{}`",
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
                let (instance, parameters) = match &callee.kind {
                    E::Ident(name) => match self.ident_poly_struct(scope, *name)? {
                        Some(ps) => (
                            self.instance_or_as_base(ps, ty),
                            Some(self.poly_structs[ps.0 as usize].lit.params.clone()),
                        ),
                        None => (None, None),
                    },
                    _ => match self.eval_const(scope, callee, None)? {
                        Operand::PolyStruct(ps) => (
                            self.instance_or_as_base(ps, ty),
                            Some(self.poly_structs[ps.0 as usize].lit.params.clone()),
                        ),
                        _ => (self.types.as_struct(ty).map(|_| ty), None),
                    },
                };
                let Some(ty) = instance else {
                    return err(
                        span,
                        format!("{} is not an instance of this struct", self.types.name(ty)),
                    );
                };
                let s = self.types.as_struct(ty).unwrap();
                let parameters = parameters
                    .or_else(|| self.struct_asts.get(&s).map(|src| src.lit.params.clone()))
                    .unwrap_or_default();
                // Instance keys store values, not their declared types, and may contain
                // inferred type bindings absent from the written parameter list. Match
                // that list by parameter identity against the canonical typed bindings.
                let actual = self.poly_struct_bindings(s);
                let mut position = 0;
                for a in args {
                    let parameter = match a.name {
                        Some(name) => parameters
                            .iter()
                            .find(|p| p.name.map(|n| n.name) == Some(name.name)),
                        None => {
                            let parameter = parameters.get(position);
                            position += 1;
                            parameter
                        }
                    };
                    let Some(name) = parameter.and_then(|p| p.name).map(|n| n.name) else {
                        return err(a.value.span, "unknown polymorphic struct parameter");
                    };
                    let Some((_, value, value_type)) = actual.iter().find(|(n, _, _)| *n == name)
                    else {
                        return err(a.value.span, "missing polymorphic struct parameter binding");
                    };
                    match &a.value.kind {
                        E::PolyVar {
                            name, ..
                        } => {
                            bind(self, bindings, *name, value.clone(), *value_type)?;
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
}
