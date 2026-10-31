//! `#bake_arguments f(a = 1)`: a new procedure (or polymorphic struct) with
//! some parameters fixed to compile-time values.
//!
//! The baked parameters are removed from a copy of the header and declared
//! as constants in a scope between the original defining scope and the new
//! procedure, so the body and the remaining parameter types see them by name.
use super::lower::Operand;
use super::{
    Compiler, ConstArg, Diagnostic, Rc, Result, ScopeId, Span, Sym, TypeId, Value, ast, calls, err,
    procs, value,
};
use crate::diagnostics::Code;

impl Compiler {
    pub fn check_bake(
        &mut self,
        scope: ScopeId,
        callee: &ast::Expr,
        args: &[ast::Arg],
        span: Span,
    ) -> Result<Operand> {
        match self.eval_const(scope, callee, None)? {
            Operand::Procs(procs) if procs.len() == 1 => {
                let proc = procs[0];
                let (name, lit, def_scope) = {
                    let p = self.proc(proc);
                    (p.name, p.lit.clone(), p.bindings.unwrap_or(p.scope))
                };
                let params = lit.header.params.clone();
                let (context_args, plain_args): (Vec<&ast::Arg>, Vec<&ast::Arg>) =
                    args.iter().partition(|a| a.context);
                // `T = float` binds a polymorphic type variable rather than a parameter.
                let poly_vars = calls::header_poly_names(&lit.header);
                let (poly_args, param_args): (Vec<&ast::Arg>, Vec<&ast::Arg>) =
                    plain_args.iter().copied().partition(|a| {
                        a.name.is_some_and(|n| {
                            poly_vars.contains(&n.name)
                                && !params
                                    .iter()
                                    .any(|p| p.name.map(|pn| pn.name) == Some(n.name))
                        })
                    });
                self.ide_note_bake_proc_args(def_scope, &lit.header, args);
                let (kept, mut consts) =
                    self.bake_params(scope, def_scope, &params, &param_args, span)?;
                let body = if context_args.is_empty() {
                    lit.body.clone()
                } else {
                    Some(self.bake_context_overrides(
                        scope,
                        lit.body.as_ref(),
                        &context_args,
                        &mut consts,
                        span,
                    )?)
                };
                let mut header = (*lit.header).clone();
                header.params = kept;
                // `#bake_constants f(T = float)` on `f :: (m: Matrix($T, $M, $N))`: the copy is
                // still polymorphic over `M` and `N`, with `T` a constant its header names.
                let bound: Vec<Sym> = poly_args
                    .iter()
                    .filter_map(|a| a.name)
                    .map(|n| n.name)
                    .collect();
                let partial = !poly_args.is_empty() && poly_vars.iter().any(|v| !bound.contains(v));
                let binders = self.poly_binders(&lit.header);
                if partial {
                    for arg in &poly_args {
                        let ty = self.eval_type(scope, &arg.value)?;
                        consts.push((arg.name.unwrap().name, Value::Type(ty), TypeId::TYPE));
                    }
                    for t in header
                        .params
                        .iter_mut()
                        .filter_map(|p| p.ty.as_mut())
                        .chain(header.returns.iter_mut().filter_map(|r| r.ty.as_mut()))
                    {
                        unbind_poly(t, &bound);
                    }
                }
                header.baked = true;
                // The copy is a new procedure, not another export of the original.
                header.flags.program_export = None;
                let lit = Rc::new(ast::ProcLit {
                    header: Rc::new(header),
                    body,
                });
                let baked_scope = self.const_scope(def_scope, consts, span);
                if partial {
                    // A bound `$T` is still declared where the header writes it.
                    for (n, at) in binders {
                        let ids = self.scope(baked_scope).names.get(&n).cloned();
                        for id in ids.into_iter().flatten().filter(|_| bound.contains(&n)) {
                            self.entity_mut(id).span = at;
                            if self.ed.is_live() {
                                self.ide_note_entity(id);
                            }
                        }
                    }
                }
                let baked = self.new_proc(name, lit, baked_scope, span);
                if poly_args.is_empty() || partial {
                    return Ok(Operand::Procs(vec![baked]));
                }
                let mut bindings = Vec::new();
                for arg in poly_args {
                    let ty = self.eval_type(scope, &arg.value)?;
                    bindings.push((arg.name.unwrap().name, Value::Type(ty), TypeId::TYPE));
                }
                Ok(Operand::Procs(vec![
                    self.instantiate(baked, bindings, span)?,
                ]))
            }
            Operand::PolyStruct(ps) => {
                let (name, lit) = {
                    let p = &self.ty.poly_structs[ps.0 as usize];
                    (p.name, p.lit.clone())
                };
                if let Some(arg) = args.iter().find(|a| a.context) {
                    return err(
                        Code::CallBakeArguments,
                        arg.value.span,
                        "context arguments can only be baked into a procedure",
                    );
                }
                let def_scope = self.ty.poly_structs[ps.0 as usize].scope;
                self.ide_note_bake_struct_args(ps, &lit.params, args);
                let args: Vec<&ast::Arg> = args.iter().collect();
                let (kept, consts) =
                    self.bake_params(scope, def_scope, &lit.params, &args, span)?;
                // Constants baked earlier stay; instances are the origin's instances.
                let origin = self.ty.poly_structs[ps.0 as usize].origin.unwrap_or(ps);
                let consts: Vec<_> = self.ty.poly_structs[ps.0 as usize]
                    .baked
                    .iter()
                    .cloned()
                    .chain(consts)
                    .collect();
                let mut lit = (*lit).clone();
                lit.params = kept;
                let baked = self.ty.new_poly_struct(name, Rc::new(lit), def_scope);
                self.ty.poly_structs[baked.0 as usize].baked = consts;
                self.ty.poly_structs[baked.0 as usize].origin = Some(origin);
                if self.ty.poly_structs[baked.0 as usize].lit.params.is_empty() {
                    return Ok(Operand::Type(self.instantiate_struct(
                        baked,
                        Vec::new(),
                        span,
                    )?));
                }
                Ok(Operand::PolyStruct(baked))
            }
            _ => err(
                Code::CallBakeArguments,
                callee.span,
                "#bake_arguments needs a single procedure or a polymorphic struct",
            ),
        }
    }

    /// The constant a baked argument stands for, of the parameter's `declared` type when it has
    /// one.
    fn bake_value(
        &mut self,
        scope: ScopeId,
        arg: &ast::Expr,
        declared: Option<TypeId>,
    ) -> Result<(Value, TypeId)> {
        Ok(match declared {
            Some(TypeId::TYPE) | None => match self.eval_const(scope, arg, declared)? {
                Operand::Type(t) => (Value::Type(t), TypeId::TYPE),
                Operand::Procs(p) if p.len() == 1 => {
                    let v = Value::Proc(p[0]);
                    let ty = self.type_of_value(&v);
                    (v, ty)
                }
                Operand::Const {
                    value,
                    ty,
                    untyped,
                    ..
                } => {
                    let ty = if untyped {
                        self.ty.default_untyped(ty, &value)
                    } else {
                        ty
                    };
                    (value, ty)
                }
                _ => {
                    return err(
                        Code::CallBakeArguments,
                        arg.span,
                        "baked argument must be a compile-time constant",
                    );
                }
            },
            Some(t) => (self.const_value_of_type(scope, arg, t)?, t),
        })
    }

    /// `#bake_arguments f(,, allocator = a)`: the copy runs `f`'s body with those context
    /// fields set, as a call written with `,,` does. Each value is a constant of the copy.
    fn bake_context_overrides(
        &mut self,
        scope: ScopeId,
        body: Option<&ast::Block>,
        overrides: &[&ast::Arg],
        consts: &mut Vec<ConstArg>,
        span: Span,
    ) -> Result<ast::Block> {
        use ast::ExprKind as E;
        use ast::StmtKind as S;
        let Some(body) = body else {
            return err(
                Code::CallBakeArguments,
                span,
                "a procedure without a body cannot bake context arguments",
            );
        };
        let at = |kind, span| ast::Expr {
            kind,
            span,
            parens: None,
        };
        let stmt = |kind, span| ast::Stmt {
            kind,
            span,
            notes: ast::Notes::default(),
        };
        let mut stmts = Vec::new();
        for (i, arg) in overrides.iter().enumerate() {
            let field = arg.name.unwrap_or(ast::Ident {
                name: Sym::intern("allocator"),
                span: arg.value.span,
            });
            let (value, ty) = self.bake_value(scope, &arg.value, None)?;
            let held = Sym::intern(&format!("\u{0}context_argument_{i}"));
            consts.push((held, value, ty));
            stmts.push(stmt(
                S::Assign {
                    op: ast::AssignOp::Assign,
                    lhs: vec![at(
                        E::Member(Box::new(at(E::Context, field.span)), field),
                        field.span,
                    )],
                    rhs: vec![at(E::Ident(held), arg.value.span)],
                },
                arg.value.span,
            ));
        }
        stmts.push(stmt(S::Block(body.clone()), body.span));
        let pushed = ast::PushContextStmt {
            backtick: false,
            context: at(E::Context, span),
            body: Box::new(stmt(
                S::Block(ast::Block {
                    stmts,
                    span: body.span,
                    checks_off: body.checks_off,
                }),
                body.span,
            )),
            bare: true,
        };
        Ok(ast::Block {
            stmts: vec![stmt(S::PushContext(Box::new(pushed)), body.span)],
            span: body.span,
            checks_off: body.checks_off,
        })
    }

    /// Split `params` into those left open and `(name, value, type)` constants
    /// for the baked ones. Arguments are named, or positional from the start.
    fn bake_params(
        &mut self,
        scope: ScopeId,
        def_scope: ScopeId,
        params: &[ast::Param],
        args: &[&ast::Arg],
        span: Span,
    ) -> Result<(Vec<ast::Param>, Vec<ConstArg>)> {
        let mut baked: Vec<Option<&ast::Arg>> = vec![None; params.len()];
        for (i, &arg) in args.iter().enumerate() {
            let index = match arg.name {
                Some(n) => params
                    .iter()
                    .position(|p| p.name.map(|pn| pn.name) == Some(n.name))
                    .ok_or_else(|| {
                        Box::new(Diagnostic::new(
                            Code::CallBakeArguments,
                            n.span,
                            format!("no parameter named `{}` to bake", n.name),
                        ))
                    })?,
                None if i < params.len() => i,
                None => {
                    return err(
                        Code::CallBakeArguments,
                        arg.value.span,
                        "too many arguments to #bake_arguments",
                    );
                }
            };
            baked[index] = Some(arg);
        }
        let mut kept = Vec::new();
        let mut consts = Vec::new();
        // A name some parameter declares as `$T`; baked parameters may mention it as plain `T`.
        let mut binders = Vec::new();
        for ty in params.iter().filter_map(|p| p.ty.as_ref()) {
            procs::for_poly_binders(ty, &mut |n| binders.push(n));
        }
        let mut inferred = Vec::new();
        for (param, arg) in params.iter().zip(baked) {
            let Some(arg) = arg else {
                kept.push(param.clone());
                continue;
            };
            let Some(name) = param.name else {
                return err(
                    Code::CallBakeArguments,
                    span,
                    "cannot bake an unnamed parameter",
                );
            };
            let pattern = param.ty.as_ref().and_then(|t| {
                let mut t = t.clone();
                polyfy(&mut t, &binders);
                procs::has_poly(&t).then_some(t)
            });
            let declared = match (&param.ty, &pattern) {
                (Some(t), None) => Some(self.eval_type(def_scope, t)?),
                // `mode := Mode.FAST` is a `Mode`: the baked value is checked against it.
                (None, _) => param
                    .default
                    .as_ref()
                    .and_then(|d| self.eval_const(def_scope, d, None).ok())
                    .map(|op| match op {
                        Operand::Const {
                            ty,
                            value,
                            untyped: true,
                            ..
                        } => self.ty.default_untyped(ty, &value),
                        other => other.ty(),
                    })
                    .filter(|&t| t != TypeId::TYPE && t != TypeId::CODE),
                _ => None,
            };
            let (value, ty) = self.bake_value(scope, &arg.value, declared)?;
            // `($v: $T)` baked with `v = 3`: `T` is what the value's type says, and the body and
            // the other parameter types still name it.
            if let Some(pattern) = &pattern {
                let mut found = Vec::new();
                self.match_pattern(pattern, ty, &mut found, def_scope)?;
                for bound in found {
                    if !consts.iter().any(|c: &ConstArg| c.0 == bound.0) {
                        inferred.push(bound.0);
                        consts.push(bound);
                    }
                }
            }
            consts.push((name.name, value, ty));
        }
        // What a baked value fixed is no longer a variable of the parameters left open.
        for ty in kept
            .iter_mut()
            .filter_map(|p: &mut ast::Param| p.ty.as_mut())
        {
            unbind_poly(ty, &inferred);
        }
        Ok((kept, consts))
    }

    pub(super) fn const_scope(
        &mut self,
        parent: ScopeId,
        consts: Vec<(Sym, Value, TypeId)>,
        span: Span,
    ) -> ScopeId {
        let scope = self.new_hidden_scope(parent);
        // Each name is written as an argument, recorded as naming the parameter it bakes.
        for (name, value, ty) in consts {
            self.new_const(scope, name, span, value, ty);
        }
        scope
    }

    /// `#bake_arguments f(n = 1)`, `#bake_constants f(T = float)`: each `name` names a
    /// parameter of `f` or one of its `$T`.
    fn ide_note_bake_proc_args(
        &mut self,
        def_scope: ScopeId,
        header: &ast::ProcHeader,
        args: &[ast::Arg],
    ) {
        if !self.ed.is_live() {
            return;
        }
        let binders = self.poly_binders(header);
        for arg in args {
            let Some(name) = arg.name else {
                continue;
            };
            let param = header
                .params
                .iter()
                .find(|p| p.name.is_some_and(|n| n.name == name.name));
            let (decl, ty) = match param {
                Some(p) => {
                    let ty =
                        p.ty.as_ref()
                            .filter(|t| !procs::has_poly(t))
                            .and_then(|t| self.eval_type(def_scope, t).ok())
                            .unwrap_or(TypeId::COMPILE_TIME);
                    (p.name.unwrap().span, ty)
                }
                None => match binders.iter().find(|(n, _)| *n == name.name) {
                    Some(&(_, at)) => (at, TypeId::TYPE),
                    None => continue,
                },
            };
            self.ide_note_named_field(name.span, name.name, decl, ty);
        }
    }

    /// `#bake_arguments Matrix(M = 2)`: each `name` names a parameter of the struct.
    fn ide_note_bake_struct_args(
        &mut self,
        ps: value::PolyStructId,
        params: &[ast::Param],
        args: &[ast::Arg],
    ) {
        if !self.ed.is_live() {
            return;
        }
        for arg in args {
            let Some(name) = arg.name else {
                continue;
            };
            let Some((i, decl)) = params
                .iter()
                .enumerate()
                .find_map(|(i, p)| p.name.filter(|n| n.name == name.name).map(|n| (i, n.span)))
            else {
                continue;
            };
            let ty = self
                .poly_struct_param_type(ps, i, None)
                .unwrap_or(TypeId::COMPILE_TIME);
            self.ide_note_named_field(name.span, name.name, decl, ty);
        }
    }
}

/// Plain `T` in a type expression becomes `$T` for the given names.
fn polyfy(expr: &mut ast::Expr, names: &[Sym]) {
    use ast::ExprKind as E;
    match &mut expr.kind {
        E::Ident(name) if names.contains(name) => {
            expr.kind = E::PolyVar {
                name: *name,
                baked: false,
            };
        }
        E::Unary(_, e)
        | E::TypeDirective {
            ty: e, ..
        } => polyfy(e, names),
        E::ArrayType {
            size,
            elem,
        } => {
            if let ast::ArraySize::Fixed(n) = size {
                polyfy(n, names);
            }
            polyfy(elem, names);
        }
        E::Call {
            callee,
            args,
            ..
        } => {
            polyfy(callee, names);
            for a in args {
                polyfy(&mut a.value, names);
            }
        }
        E::ProcType(header) => {
            let header = Rc::make_mut(header);
            for t in header
                .params
                .iter_mut()
                .filter_map(|p| p.ty.as_mut())
                .chain(header.returns.iter_mut().filter_map(|r| r.ty.as_mut()))
            {
                polyfy(t, names);
            }
        }
        _ => {}
    }
}

/// `$T` (and `$T/R`) in a type expression become plain `T`: `T` is bound elsewhere.
fn unbind_poly(expr: &mut ast::Expr, names: &[Sym]) {
    use ast::ExprKind as E;
    match &mut expr.kind {
        E::PolyVar {
            name, ..
        }
        | E::PolyRestricted {
            name, ..
        } if names.contains(name) => expr.kind = E::Ident(*name),
        E::Unary(_, e)
        | E::TypeDirective {
            ty: e, ..
        } => unbind_poly(e, names),
        E::ArrayType {
            size,
            elem,
        } => {
            if let ast::ArraySize::Fixed(n) = size {
                unbind_poly(n, names);
            }
            unbind_poly(elem, names);
        }
        E::Call {
            callee,
            args,
            ..
        } => {
            unbind_poly(callee, names);
            for a in args {
                unbind_poly(&mut a.value, names);
            }
        }
        E::ProcType(header) => {
            let header = Rc::make_mut(header);
            for t in header
                .params
                .iter_mut()
                .filter_map(|p| p.ty.as_mut())
                .chain(header.returns.iter_mut().filter_map(|r| r.ty.as_mut()))
            {
                unbind_poly(t, names);
            }
        }
        _ => {}
    }
}
