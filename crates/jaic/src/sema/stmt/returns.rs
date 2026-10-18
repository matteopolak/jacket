//! `return` statements and emitting deferred code.
use crate::ir::Ty;
use crate::sema::lower::{DeferAction, FnCtx, Operand};
use crate::sema::*;

impl Compiler {
    pub(super) fn check_return(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        values: &[ast::Arg],
        backtick: bool,
        span: Span,
    ) -> Result<()> {
        if let Some(frame) = f.macros.last().cloned()
            && !backtick
        {
            // Return from a macro: store results and leave the expansion.
            let mut ops = Vec::new();
            for (i, v) in values.iter().enumerate() {
                let expected = frame.result_slots.get(i).map(|s| s.0);
                ops.push(self.check_expr(f, scope, &v.value, expected)?);
            }
            for (op, (ty, slot)) in ops.into_iter().zip(&frame.result_slots) {
                let op = self.convert(f, op, *ty, span)?;
                let (_, v) = self.rvalue(f, op, span)?;
                self.store_value(f, *ty, *slot, v, span)?;
            }
            self.emit_defers(f, frame.defer_depth.position(), span)?;
            f.b.jump(frame.exit_block);
            return Ok(());
        }
        // `` `return `x, y; ``: returns from the caller; `` `x `` in the values still means
        // the caller's x.
        let caller = f.macros.last().map(|m| m.caller_scope);
        let saved_macros = if backtick {
            std::mem::take(&mut f.macros)
        } else {
            Vec::new()
        };
        let saved_backtick = f.backtick_scope;
        if backtick && caller.is_some() {
            f.backtick_scope = caller;
        }
        let result = self.check_proc_return(f, scope, values, span);
        f.backtick_scope = saved_backtick;
        if backtick {
            f.macros = saved_macros;
        }
        result
    }

    /// `return second = 2, first = 1;`: values are evaluated in source order
    /// and stored into their named results.
    fn check_named_return(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        values: &[ast::Arg],
        span: Span,
    ) -> Result<()> {
        let Some(proc) = f.proc else {
            return err(span, "named return values need a procedure");
        };
        let names = self.signature(proc, span)?.return_names.clone();
        let types = f.return_types.clone();
        let mut positional = 0;
        for v in values {
            let index = if let Some(n) = v.name {
                names
                    .iter()
                    .position(|r| *r == Some(n.name))
                    .ok_or_else(|| {
                        Box::new(Diagnostic::error(
                            n.span,
                            format!("no return value named `{}`", n.name),
                        ))
                    })?
            } else {
                positional += 1;
                positional - 1
            };
            let Some(&ty) = types.get(index) else {
                return err(v.value.span, "too many return values");
            };
            let Some(addr) = f.named_results[index] else {
                return err(v.value.span, "return value has no name to assign");
            };
            let op = self.check_expr(f, scope, &v.value, Some(ty))?;
            let op = self.convert(f, op, ty, v.value.span)?;
            let (_, val) = self.rvalue(f, op, v.value.span)?;
            self.store_value(f, ty, addr, val, v.value.span)?;
        }
        self.emit_fallthrough_return(f, span)
    }

    fn check_proc_return(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        values: &[ast::Arg],
        span: Span,
    ) -> Result<()> {
        let types = f.return_types.clone();
        if values.iter().any(|v| v.name.is_some()) {
            return self.check_named_return(f, scope, values, span);
        }
        let mut ops: Vec<Operand> = Vec::new();
        // Where each value was written, for errors.
        let mut spans: Vec<Span> = Vec::new();
        for (i, v) in values.iter().enumerate() {
            let op = self.check_expr(f, scope, &v.value, types.get(i).copied())?;
            match op {
                // Extra results of a returned call are dropped, like in a declaration.
                Operand::Multi(vals) if values.len() == 1 => {
                    ops.extend(vals.into_iter().take(types.len()));
                }
                // `return f();` of a procedure without results.
                Operand::Void if values.len() == 1 => {}
                other => ops.push(other),
            }
            spans.resize(ops.len(), v.value.span);
        }
        if ops.is_empty() && !types.is_empty() && f.named_results.iter().all(Option::is_some) {
            self.emit_fallthrough_return(f, span)?;
            return Ok(());
        }
        if ops.len() > types.len() {
            return err(
                span,
                format!("too many return values (procedure returns {})", types.len()),
            );
        }
        if ops.len() < types.len() && !ops.is_empty() {
            // Remaining results take their named defaults.
            if let Some(i) = (ops.len()..types.len()).find(|&i| f.named_results[i].is_none()) {
                return err(
                    span,
                    format!("missing return value {} of {}", i + 1, types.len()),
                );
            }
        } else if ops.is_empty() && !types.is_empty() {
            return err(span, "missing return value");
        }
        let mut scalars = Vec::new();
        for (i, &ty) in types.iter().enumerate() {
            let v = if let Some(op) = ops.get(i) {
                let at = spans.get(i).copied().unwrap_or(span);
                let op = self
                    .convert(f, op.clone(), ty, at)
                    .map_err(|e| self.return_type_mismatch(e, f.proc, i))?;
                let (_, v) = self.rvalue(f, op, at)?;
                v
            } else {
                let addr = f.named_results[i].unwrap();
                match self.ir_ty(ty) {
                    Some(t) => f.b.load(t, addr),
                    None => addr,
                }
            };
            match f.return_outs[i] {
                Some(out) => self.store_value(f, ty, out, v, span)?,
                None if f.defers.is_empty() => scalars.push((v, None)),
                None => {
                    // Keep the value safe from defers that modify locals.
                    let t = self.ir_ty(ty).unwrap();
                    let slot = f.b.alloca(t.size(), t.size());
                    f.b.store(t, slot, v);
                    scalars.push((v, Some((t, slot))));
                }
            }
        }
        self.emit_defers(f, 0, span)?;
        let mut rets = Vec::new();
        for (v, saved) in scalars {
            rets.push(match saved {
                Some((t, slot)) => f.b.load(t, slot),
                None => v,
            });
        }
        f.b.ret(rets);
        Ok(())
    }

    /// Emit deferred statements registered above `depth`, innermost first.
    pub fn emit_defers(&mut self, f: &mut FnCtx, depth: usize, _span: Span) -> Result<()> {
        if f.defers.len() <= depth {
            return Ok(());
        }
        let saved = f.defers.clone();
        let saved_context = f.context_binding();
        let result = (|| {
            for i in (depth..saved.len()).rev() {
                // Keep only earlier cleanups visible to a return in deferred code.
                f.defers.truncate(i);
                let entry = &saved[i];
                let after = entry.active.map(|active| {
                    let run = f.b.new_block();
                    let after = f.b.new_block();
                    let enabled = f.b.load(Ty::I8, active);
                    f.b.branch(enabled, run, after);
                    f.b.switch_to(run);
                    let no = f.b.iconst(Ty::I8, 0);
                    f.b.store(Ty::I8, active, no);
                    after
                });
                match &entry.action {
                    DeferAction::RestoreContext {
                        previous,
                        saved_slot,
                    } => {
                        let previous_ptr = f.b.load(Ty::Ptr, *saved_slot);
                        f.set_context(Some(previous_ptr));
                        f.restore_context_binding(*previous);
                    }
                    DeferAction::Statement(stmt) => {
                        let inner = self.new_block_scope(entry.scope);
                        let saved_backtick =
                            std::mem::replace(&mut f.backtick_scope, entry.caller_scope);
                        let result = self.check_stmt(f, inner, stmt);
                        f.backtick_scope = saved_backtick;
                        result?;
                    }
                }
                if let Some(after) = after {
                    if !f.b.is_terminated() {
                        f.b.jump(after);
                    }
                    f.b.switch_to(after);
                }
            }
            Ok(())
        })();
        f.defers = saved;
        // Exit branches do not change the context used to compile other paths.
        f.restore_context_binding(saved_context);
        result
    }
}
