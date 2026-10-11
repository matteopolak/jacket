//! `#bake_arguments f(a = 1)`: a new procedure (or polymorphic struct) with
//! some parameters fixed to compile-time values.
//!
//! The baked parameters are removed from a copy of the header and declared
//! as constants in a scope between the original defining scope and the new
//! procedure, so the body and the remaining parameter types see them by name.
use super::lower::Operand;
use super::*;

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
                // `T = float` binds a polymorphic type variable rather than a parameter.
                let poly_vars = calls::header_poly_names(&lit.header);
                let (poly_args, param_args): (Vec<&ast::Arg>, Vec<&ast::Arg>) =
                    args.iter().partition(|a| {
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
                    body: lit.body.clone(),
                });
                let baked_scope = self.const_scope(def_scope, consts, span);
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
                    let p = &self.poly_structs[ps.0 as usize];
                    (p.name, p.lit.clone())
                };
                let def_scope = self.poly_structs[ps.0 as usize].scope;
                self.ide_note_bake_struct_args(ps, &lit.params, args);
                let args: Vec<&ast::Arg> = args.iter().collect();
                let (kept, consts) =
                    self.bake_params(scope, def_scope, &lit.params, &args, span)?;
                // Constants baked earlier stay; instances are the origin's instances.
                let origin = self.poly_structs[ps.0 as usize].origin.unwrap_or(ps);
                let consts: Vec<_> = self.poly_structs[ps.0 as usize]
                    .baked
                    .iter()
                    .cloned()
                    .chain(consts)
                    .collect();
                let mut lit = (*lit).clone();
                lit.params = kept;
                let baked = self.new_poly_struct(name, Rc::new(lit), def_scope);
                self.poly_structs[baked.0 as usize].baked = consts;
                self.poly_structs[baked.0 as usize].origin = Some(origin);
                if self.poly_structs[baked.0 as usize].lit.params.is_empty() {
                    return Ok(Operand::Type(self.instantiate_struct(
                        baked,
                        Vec::new(),
                        span,
                    )?));
                }
                Ok(Operand::PolyStruct(baked))
            }
            _ => err(
                callee.span,
                "#bake_arguments needs a single procedure or a polymorphic struct",
            ),
        }
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
                        Box::new(Diagnostic::error(
                            n.span,
                            format!("no parameter named `{}` to bake", n.name),
                        ))
                    })?,
                None if i < params.len() => i,
                None => return err(arg.value.span, "too many arguments to #bake_arguments"),
            };
            baked[index] = Some(arg);
        }
        let mut kept = Vec::new();
        let mut consts = Vec::new();
        for (param, arg) in params.iter().zip(baked) {
            let Some(arg) = arg else {
                kept.push(param.clone());
                continue;
            };
            let Some(name) = param.name else {
                return err(span, "cannot bake an unnamed parameter");
            };
            let declared = match &param.ty {
                Some(t) if !procs::has_poly(t) => Some(self.eval_type(def_scope, t)?),
                _ => None,
            };
            let (value, ty) = match declared {
                Some(TypeId::TYPE) | None => match self.eval_const(scope, &arg.value, declared)? {
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
                    } => {
                        let ty = if untyped {
                            self.default_untyped(ty, &value)
                        } else {
                            ty
                        };
                        (value, ty)
                    }
                    _ => {
                        return err(
                            arg.value.span,
                            "baked argument must be a compile-time constant",
                        );
                    }
                },
                Some(t) => (self.const_value_of_type(scope, &arg.value, t)?, t),
            };
            consts.push((name.name, value, ty));
        }
        Ok((kept, consts))
    }

    pub(super) fn const_scope(
        &mut self,
        parent: ScopeId,
        consts: Vec<(Sym, Value, TypeId)>,
        span: Span,
    ) -> ScopeId {
        let module = self.scope(parent).module;
        let scope = self.new_scope(scope::ScopeKind::Block, Some(parent), module, None);
        for (name, value, ty) in consts {
            self.add_const(scope, name, span, value, ty);
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
        if self.ide.is_none() {
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
        if self.ide.is_none() {
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
        _ => {}
    }
}
