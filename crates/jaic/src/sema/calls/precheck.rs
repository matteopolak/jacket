//! Checking arguments before a candidate is chosen, and ranking candidates.
use super::{CallArg, Candidate, is_deferred};
use crate::sema::lower::FnCtx;
use crate::sema::*;

impl Compiler {
    /// A copy of the current context with fields replaced; an unnamed
    /// override sets the allocator.
    pub(super) fn context_with_overrides(
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

    pub(in crate::sema) fn precheck_args(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        args: &[ast::Arg],
    ) -> Result<Vec<CallArg>> {
        self.precheck_args_deferring(f, scope, args, &[], &[])
    }

    /// Arguments passed to a `#discard` parameter of one of `procs`: they are typechecked
    /// but generate no code.
    pub(super) fn discarded_args(&self, procs: &[ProcId], args: &[ast::Arg]) -> Vec<bool> {
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
    pub(super) fn macro_code_args(&mut self, procs: &[ProcId], args: &[ast::Arg]) -> Vec<bool> {
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
    pub(super) fn precheck_args_deferring(
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
    pub(super) fn rank_candidates(
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
}
