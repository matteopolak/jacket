//! Assignments: indexed, operator and plain targets.
use crate::ast::ExprKind as E;
use crate::sema::calls::CallArg;
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::scope::EntityKind;
use crate::sema::*;

impl Compiler {
    /// `x[i] = v` / `x[i] op= v` through `operator []=` (and `operator []` to
    /// read the old value). Evaluation order: base, index, (get), value, set.
    fn try_index_assign(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        op: ast::AssignOp,
        target: &ast::Expr,
        rhs: &ast::Expr,
        span: Span,
    ) -> Result<bool> {
        let E::Index(base, index) = &target.kind else {
            return Ok(false);
        };
        let Ok(probe) = self.check_expr_no_emit(scope, base) else {
            return Ok(false);
        };
        let bt = probe.ty();
        let st = self.types.pointee(bt).unwrap_or(bt);
        if self.types.as_struct(self.types.repr_struct(st)).is_none() {
            return Ok(false);
        }
        let mut setters = self.operator_candidates(scope, "[]=")?;
        // Only an operator taking this struct applies. Otherwise `p[i] = v` through a raw
        // pointer writes memory, and a struct with only `operator *[]` assigns through it.
        setters.retain(|&p| self.first_param_accepts(p, st));
        if setters.is_empty() {
            return Ok(false);
        }
        let base_op = self.check_expr(f, scope, base, None)?;
        let ptr_ty = self.types.pointer(st);
        let ptr = if self.types.is_pointer(bt) {
            let (_, v) = self.rvalue(f, base_op, base.span)?;
            v
        } else {
            let (_, addr) = self.address_of(f, base_op, base.span)?;
            addr
        };
        let ptr_op = Operand::Value {
            ty: ptr_ty,
            val: ptr,
        };
        let index_op = self.check_expr(f, scope, index, None)?;
        let index_op = if index_op.is_const() {
            index_op
        } else {
            let (ty, val) = self.rvalue(f, index_op, index.span)?;
            Operand::Value {
                ty,
                val,
            }
        };
        let arg = |op: Operand| CallArg {
            name: None,
            spread: false,
            expr: None,
            op: Some(op),
            span,
            scope,
        };
        let value = match op {
            ast::AssignOp::Assign => self.check_expr(f, scope, rhs, None)?,
            ast::AssignOp::Op(bin) => {
                let getters = self.operator_candidates(scope, "[]")?;
                let by_ptr = vec![arg(ptr_op.clone()), arg(index_op.clone())];
                let old = if let Ok(v) = self.try_call(f, scope, &getters, by_ptr, span) {
                    v
                } else {
                    let place = Operand::Place {
                        ty: st,
                        addr: ptr,
                    };
                    let by_value = vec![arg(place), arg(index_op.clone())];
                    self.call_procs(f, scope, &getters, by_value, None, span)?
                };
                let (old_ty, old_val) = self.rvalue(f, old, span)?;
                let inner = self.new_block_scope(scope);
                let slot = self.spill(f, old_ty, old_val, span)?;
                let tmp = Sym::intern("\u{0}old");
                let depth = self.scope(inner).proc_depth;
                self.add_entity(
                    inner,
                    tmp,
                    span,
                    EntityKind::Local {
                        ty: old_ty,
                        addr: slot,
                        depth,
                    },
                    false,
                );
                let expr = ast::Expr {
                    kind: E::Binary(
                        bin,
                        Box::new(ast::Expr {
                            kind: E::Ident(tmp),
                            span: target.span,
                        }),
                        Box::new(rhs.clone()),
                    ),
                    span,
                };
                self.check_expr(f, inner, &expr, Some(old_ty))?
            }
        };
        let args = vec![arg(ptr_op), arg(index_op), arg(value)];
        self.call_procs(f, scope, &setters, args, None, span)?;
        Ok(true)
    }

    /// Can procedure `p`'s first parameter take a `ty` (or `*ty`)? Polymorphic patterns
    /// are matched; plain types compared after resolution.
    pub(in crate::sema) fn first_param_accepts(&mut self, p: ProcId, ty: TypeId) -> bool {
        let info = self.proc(p);
        let (scope, lit) = (info.scope, info.lit.clone());
        let Some(param_ty) = lit.header.params.first().and_then(|p| p.ty.clone()) else {
            return false;
        };
        let pointer = self.types.pointer(ty);
        if procs::has_poly(&param_ty)
            || self.mentions_poly_struct(scope, &param_ty).unwrap_or(false)
        {
            [ty, pointer].into_iter().any(|t| {
                let mut bindings = Vec::new();
                self.match_pattern(&param_ty, t, &mut bindings, scope)
                    .is_ok()
            })
        } else {
            self.eval_type(scope, &param_ty).is_ok_and(|target| {
                [ty, pointer]
                    .into_iter()
                    .any(|from| self.implicit_cost(from, false, target).is_some())
            })
        }
    }

    /// Does some `operator op=` take this target (by pointer or by value) and value? Without one,
    /// `a op= b` is `a = a op b`. The operands are checked in a scratch context, so nothing is
    /// emitted twice when the answer is no.
    fn operator_assign_applies(
        &mut self,
        f: &FnCtx,
        scope: ScopeId,
        candidates: &[ProcId],
        target: &ast::Expr,
        rhs: &ast::Expr,
        span: Span,
    ) -> bool {
        let mut scratch = self.scratch_ctx(scope);
        scratch.backtick_scope = f
            .backtick_scope
            .or_else(|| f.macros.last().map(|frame| frame.caller_scope));
        let Ok(Operand::Place { ty, addr }) = self.check_expr(&mut scratch, scope, target, None) else {
            return true;
        };
        let Ok(value) = self.check_expr(&mut scratch, scope, rhs, None) else {
            return true;
        };
        let arg = |op: Operand| CallArg {
            name: None,
            spread: false,
            expr: None,
            op: Some(op),
            span,
            scope,
        };
        let ptr = Operand::Value {
            ty: self.types.pointer(ty),
            val: addr,
        };
        let by_ptr = vec![arg(ptr), arg(value.clone())];
        let by_value = vec![arg(Operand::Place { ty, addr }), arg(value)];
        self.any_accepts(candidates, &by_ptr, span) || self.any_accepts(candidates, &by_value, span)
    }

    /// `a op= b` through a user-defined `operator op=` (called with `*a`, or `a` by value).
    fn try_operator_assign(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        bin: ast::BinOp,
        target: &ast::Expr,
        rhs: &ast::Expr,
        span: Span,
    ) -> Result<bool> {
        let Ok(probe) = self.check_expr_no_emit(scope, target) else {
            return Ok(false);
        };
        if !self.overloadable(probe.ty()) {
            return Ok(false);
        }
        let text = format!("{}=", calls::binop_text(bin));
        let candidates = self.operator_candidates(scope, &text)?;
        if candidates.is_empty() || !self.operator_assign_applies(f, scope, &candidates, target, rhs, span) {
            return Ok(false);
        }
        let Operand::Place {
            ty,
            addr,
        } = self.check_expr(f, scope, target, None)?
        else {
            return err(target.span, "cannot assign to this expression");
        };
        let value = self.check_expr(f, scope, rhs, None)?;
        let arg = |op: Operand| CallArg {
            name: None,
            spread: false,
            expr: None,
            op: Some(op),
            span,
            scope,
        };
        let ptr = Operand::Value {
            ty: self.types.pointer(ty),
            val: addr,
        };
        let by_ptr = vec![arg(ptr), arg(value.clone())];
        if self.try_call(f, scope, &candidates, by_ptr, span).is_err() {
            let place = Operand::Place {
                ty,
                addr,
            };
            let by_value = vec![arg(place), arg(value)];
            self.call_procs(f, scope, &candidates, by_value, None, span)?;
        }
        Ok(true)
    }

    /// A member/index write must not materialize a constant container as a
    /// writable temporary. Qualifiers may themselves be modules or types;
    /// pointer indirection instead selects separate pointee storage.
    pub(super) fn reject_constant_target(
        &mut self,
        f: &FnCtx,
        scope: ScopeId,
        target: &ast::Expr,
    ) -> Result<()> {
        let mut container = target;
        while let E::Index(base, _) | E::Member(base, _) = &container.kind {
            let mut scratch = self.scratch_ctx(scope);
            scratch.backtick_scope = f
                .backtick_scope
                .or_else(|| f.macros.last().map(|frame| frame.caller_scope));
            let operand = self.check_expr(&mut scratch, scope, base, None)?;
            if self.types.is_pointer(operand.ty()) {
                return Ok(());
            }
            if let Operand::Const {
                value, ..
            } = operand
                && !matches!(value, Value::Type(_) | Value::Null)
            {
                return err(
                    target.span,
                    format!(
                        "cannot assign into the constant `{}`",
                        self.sources.snippet(base.span).trim()
                    ),
                );
            }
            container = base;
        }
        Ok(())
    }

    /// A name, or fields of a name that is not a pointer: evaluating it twice has no effect and
    /// finds the same address.
    fn is_plain_variable_target(&mut self, scope: ScopeId, target: &ast::Expr) -> bool {
        let mut root = target;
        while let E::Member(base, _) = &root.kind {
            root = base;
        }
        if !matches!(root.kind, E::Ident(_)) {
            return false;
        }
        if std::ptr::eq(root, target) {
            return true;
        }
        match self.check_expr_no_emit(scope, root) {
            Ok(op) => !self.types.is_pointer(op.ty()),
            Err(_) => false,
        }
    }

    pub(super) fn check_assign(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        op: ast::AssignOp,
        lhs: &[ast::Expr],
        rhs: &[ast::Expr],
        span: Span,
    ) -> Result<()> {
        for target in lhs {
            self.reject_constant_target(f, scope, target)?;
        }
        if lhs.len() == 1
            && rhs.len() == 1
            && self.try_index_assign(f, scope, op, &lhs[0], &rhs[0], span)?
        {
            return Ok(());
        }
        if let ast::AssignOp::Op(bin) = op {
            if lhs.len() > 1 && rhs.len() == 1 {
                // `a, b += 1;` applies the operation to every target.
                for target in lhs {
                    self.check_assign(f, scope, op, std::slice::from_ref(target), rhs, span)?;
                }
                return Ok(());
            }
            if lhs.len() != 1 || rhs.len() != 1 {
                return err(span, "compound assignment takes one target and one value");
            }
            if self.try_operator_assign(f, scope, bin, &lhs[0], &rhs[0], span)? {
                return Ok(());
            }
            // Evaluate the target once, then `*tmp = *tmp op rhs`.
            let place = self.check_target(f, scope, &lhs[0])?.without_callable();
            let Operand::Place {
                ty,
                addr,
            } = place
            else {
                return err(lhs[0].span, "cannot assign to this expression");
            };
            if self.is_plain_variable_target(scope, &lhs[0]) {
                // A variable (or a field of one) has the same address every time it is named,
                // so `x += y` is `x = x + y` without a hidden pointer to it.
                let value = ast::Expr {
                    kind: E::Binary(bin, Box::new(lhs[0].clone()), Box::new(rhs[0].clone())),
                    span,
                };
                let result = self.check_expr(f, scope, &value, Some(ty))?;
                let result = self.convert(f, result, ty, span)?;
                let (_, v) = self.rvalue(f, result, span)?;
                self.store_value(f, ty, addr, v, span)?;
                return Ok(());
            }
            let inner = self.new_block_scope(scope);
            let ptr_ty = self.types.pointer(ty);
            let slot = self.spill(f, ptr_ty, addr, span)?;
            let tmp = Sym::intern("\u{0}lhs");
            let depth = self.scope(inner).proc_depth;
            self.add_entity(
                inner,
                tmp,
                span,
                EntityKind::Local {
                    ty: ptr_ty,
                    addr: slot,
                    depth,
                },
                false,
            );
            // The hidden pointer gets an empty span at the target's start, so what is recorded
            // for the target's own span (its type, the names it resolves to) stays the user's.
            let at = lhs[0].span;
            let target = ast::Expr {
                kind: E::Unary(
                    ast::UnOp::Deref,
                    Box::new(ast::Expr {
                        kind: E::Ident(tmp),
                        span: Span::new(at.file, at.start as usize, at.start as usize),
                    }),
                ),
                span: at,
            };
            let value = ast::Expr {
                kind: E::Binary(bin, Box::new(target), Box::new(rhs[0].clone())),
                span,
            };
            let result = self.check_expr(f, inner, &value, Some(ty))?;
            let result = self.convert(f, result, ty, span)?;
            let (_, v) = self.rvalue(f, result, span)?;
            self.store_value(f, ty, addr, v, span)?;
            return Ok(());
        }
        if lhs.len() == 1 && rhs.len() == 1 {
            let place = self.check_target(f, scope, &lhs[0])?.without_callable();
            let Operand::Place {
                ty,
                addr,
            } = place
            else {
                return err(
                    lhs[0].span,
                    format!("cannot assign to {}", self.describe(&place)),
                );
            };
            let value = self.check_expr(f, scope, &rhs[0], Some(ty))?;
            let value = self.convert(f, value, ty, rhs[0].span)?;
            // A call's result is a temporary of its own and cannot overlap the target.
            let fresh =
                matches!(rhs[0].kind, E::Call { .. }) && matches!(value, Operand::Value { .. });
            let (_, v) = self.rvalue(f, value, span)?;
            // Copy through a temporary when the source may overlap the target.
            if self.is_memory_type(ty) && !fresh {
                let size = self.size_of(ty, span)?;
                let align = self.align_of(ty, span)?;
                let tmp = f.b.alloca(size.max(1), align);
                f.b.copy(tmp, v, size);
                f.b.copy(addr, tmp, size);
            } else {
                self.store_value(f, ty, addr, v, span)?;
            }
            return Ok(());
        }
        // Multiple assignment: evaluate every value first.
        let mut values: Vec<(TypeId, ir::Val)> = Vec::new();
        if rhs.len() == 1 {
            match self.check_expr(f, scope, &rhs[0], None)? {
                Operand::Multi(v) => {
                    for operand in v {
                        values.push(self.rvalue(f, operand, rhs[0].span)?);
                    }
                }
                other => {
                    // `a, b = value;` assigns the one value to every target.
                    if matches!(
                        other,
                        Operand::Callable { .. }
                            | Operand::Value { .. }
                            | Operand::Place { .. }
                            | Operand::Const { .. }
                            | Operand::Type(_)
                    ) {
                        for l in lhs {
                            let place = self.check_target(f, scope, l)?.without_callable();
                            let Operand::Place {
                                ty,
                                addr,
                            } = place
                            else {
                                return err(l.span, "cannot assign to this expression");
                            };
                            let op = self.convert(f, other.clone(), ty, rhs[0].span)?;
                            let (_, v) = self.rvalue(f, op, l.span)?;
                            self.store_value(f, ty, addr, v, l.span)?;
                        }
                        return Ok(());
                    }
                    return err(
                        rhs[0].span,
                        format!(
                            "expected {} values, found {}",
                            lhs.len(),
                            self.describe(&other)
                        ),
                    );
                }
            }
        } else {
            for (r, l) in rhs.iter().zip(lhs) {
                let op = self.check_expr(f, scope, r, None)?;
                // Untyped constants take the type of their own target (`v.x, v.y = 42, 108;`).
                let target = match &l.kind {
                    E::Ident(name) if name.as_str() == "_" => None,
                    _ => match self.check_expr(f, scope, l, None)? {
                        Operand::Place {
                            ty, ..
                        } => Some(ty),
                        _ => None,
                    },
                };
                let op = match (&op, target) {
                    (
                        Operand::Const {
                            untyped: true, ..
                        },
                        Some(ty),
                    ) => self.convert(f, op, ty, r.span)?,
                    _ => op,
                };
                let op = self.settle_untyped(op, None);
                let (ty, v) = self.rvalue(f, op, r.span)?;
                let tmp = self.spill(f, ty, v, r.span)?;
                let v = match self.ir_ty(ty) {
                    Some(t) => f.b.load(t, tmp),
                    None => tmp,
                };
                values.push((ty, v));
            }
        }
        if values.len() < lhs.len() {
            return err(
                span,
                format!("expected {} values, found {}", lhs.len(), values.len()),
            );
        }
        for (l, (vty, v)) in lhs.iter().zip(values) {
            if let E::Ident(name) = &l.kind
                && name.as_str() == "_"
            {
                continue;
            }
            let place = self.check_target(f, scope, l)?.without_callable();
            let Operand::Place {
                ty,
                addr,
            } = place
            else {
                return err(l.span, "cannot assign to this expression");
            };
            let op = self.convert(
                f,
                Operand::Value {
                    ty: vty,
                    val: v,
                },
                ty,
                l.span,
            )?;
            let (_, v) = self.rvalue(f, op, l.span)?;
            self.store_value(f, ty, addr, v, l.span)?;
        }
        Ok(())
    }
}
