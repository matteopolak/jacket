//! Matching arguments against a candidate and baking automatic variants.
use super::slots::{assign_slots, header_poly_names, pattern_specificity};
use super::{AutoBake, CallArg, Candidate, Slot, const_marker};
use crate::ast::ExprKind as E;
use crate::sema::convert;
use crate::sema::lower::Operand;
use crate::sema::{
    Compiler, Diagnostic, DiagnosticKind, ProcId, Rc, Result, Span, TypeId, Value, ast, err, ir,
};

impl Compiler {
    /// Why the only candidate of a call does not accept its arguments: the problem at the
    /// argument it concerns, and where the procedure is declared.
    pub(super) fn call_mismatch(
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
        let header = self.proc(proc).lit.header.span;
        let within = |outer: Span| {
            error.span.file == outer.file
                && error.span.start >= outer.start
                && error.span.end <= outer.end
        };
        // A problem in the call itself, or in a default value of the procedure.
        let inside = within(span) || (header != Span::NONE && within(header));
        let mut d = Diagnostic {
            span: if inside {
                error.span
            } else {
                span
            },
            message: format!("in call to `{name}`: {}", error.message),
            // Structural binding failures retain their machine-readable contract kind.
            kind: if matches!(
                error.kind,
                DiagnosticKind::InterfaceConstraint | DiagnosticKind::ArgumentCount
            ) {
                error.kind
            } else {
                DiagnosticKind::Other
            },
            ..error
        };
        let overlaps =
            header.file == span.file && header.start < span.end && span.start < header.end;
        if header != Span::NONE && !overlaps {
            d.notes.push((header, format!("`{name}` is declared here")));
        }
        if d.kind == DiagnosticKind::ArgumentCount {
            d = self.with_signature_note(d, name, header);
        }
        d
    }

    /// Check whether `proc` accepts `args`, instantiating polymorphic procedures.
    pub(super) fn match_candidate(
        &mut self,
        proc: ProcId,
        args: &[CallArg],
        span: Span,
    ) -> Result<Candidate> {
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
                .map(|ty| pattern_specificity(ty, &poly_vars))
                .sum();
            let bindings = self.infer_bindings(proc, &header, &slots, args, span)?;
            proc_id = self.instantiate(proc, bindings, span)?;
            let made = self.meta.export.call_instances.entry(span).or_default();
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
        let mut fit = 0;
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
                            .with_kind(DiagnosticKind::ArgumentCount)
                            .with_help(format!(
                                "pass a value for `{name}`, or give the parameter a default value (`{name} := ...`)"
                            )),
                        ));
                    }
                }
                Slot::Arg(a) => {
                    cost += self.arg_cost(&args[*a], param.ty, is_macro)?;
                    fit += self.arg_fit(&args[*a], param.ty)?;
                }
                Slot::Spread(a) => cost += self.arg_cost(&args[*a], param.ty, is_macro)?,
                Slot::Variadic(list) => {
                    let elem = if sig.c_varargs {
                        param.ty
                    } else {
                        self.ty.jai_variadic_element(param.ty, span)?
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
            fit,
        })
    }

    /// The type the chosen candidate gives argument `arg`, when it is a runtime parameter of an
    /// ordinary procedure.
    pub(super) fn argument_param_type(
        &mut self,
        c: &Candidate,
        arg: usize,
        span: Span,
    ) -> Result<Option<TypeId>> {
        if self.proc(c.proc).is_macro {
            return Ok(None);
        }
        let header = self.proc(c.proc).lit.header.clone();
        let sig = self.signature(c.proc, span)?;
        let runtime = (0..header.params.len()).filter(|&i| !header.params[i].baked);
        for (k, i) in runtime.enumerate() {
            let param = &sig.params[k];
            match &c.slots[i] {
                Slot::Arg(a) | Slot::Spread(a) if *a == arg => return Ok(Some(param.ty)),
                Slot::Variadic(list) if list.contains(&arg) && !sig.c_varargs => {
                    return Ok(Some(self.ty.jai_variadic_element(param.ty, span)?));
                }
                _ => {}
            }
        }
        Ok(None)
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
                let Some(p) = self.ty.types.pointee(ty) else {
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
        if let Some(&v) = self.pr.auto_bake_variants.get(&(proc, mask.clone())) {
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
        self.pr.auto_bake_variants.insert((proc, mask), v);
        v
    }
}
