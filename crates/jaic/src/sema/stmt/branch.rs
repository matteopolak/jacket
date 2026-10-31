//! `if` and `switch` statements and the comparisons they make.
use crate::diagnostics::Code;
use crate::ir::{CmpOp, Ty};
use crate::sema::TypeState;
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::{Compiler, Diagnostic, Result, ScopeId, Span, TypeId, Value, ast, err, ir};
use crate::types::TypeKind;

/// A switch on a constant: its value type, cases and the indices of the cases that run.
struct Switch<'a> {
    vty: TypeId,
    cases: &'a [ast::Case],
    taken: &'a [usize],
    complete: bool,
}

impl Compiler {
    fn const_truth(op: &Operand) -> Option<bool> {
        match op {
            Operand::Const {
                value: Value::Bool(b),
                ..
            } => Some(*b),
            Operand::Const {
                value: Value::Int(i),
                ..
            } => Some(*i != 0),
            Operand::Const {
                value: Value::Null,
                ..
            } => Some(false),
            _ => None,
        }
    }

    /// An editor analysis checks the branch a constant condition rules out, so hover, definition
    /// and tokens work in it. It is lowered into a block nothing reaches, and its errors and
    /// warnings are dropped: the batch compiler never looks at it.
    fn check_dead_for_facts(&mut self, f: &mut FnCtx, scope: ScopeId, branch: &ast::Stmt) {
        if !self.ed.is_live() || self.ed.sketching() || !self.ide_wants_pub(branch.span.file) {
            return;
        }
        let warned = self.diag.warnings.len();
        let errors = self.ed.error_count();
        let dead = f.b.new_block();
        let after = f.b.new_block();
        let never = f.b.iconst(Ty::I8, 0);
        f.b.branch(never, dead, after);
        f.b.switch_to(dead);
        let _ = self.check_scoped(f, scope, branch);
        if !f.b.is_terminated() {
            f.b.jump(after);
        }
        f.b.switch_to(after);
        self.diag.warnings.truncate(warned);
        self.ed.forget_errors(errors);
    }

    pub(super) fn check_if(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        cond: &ast::Expr,
        then_branch: &ast::Stmt,
        else_branch: Option<&ast::Stmt>,
    ) -> Result<()> {
        let c = match self.check_expr(f, scope, cond, Some(TypeId::BOOL)) {
            Ok(c) => c,
            Err(e) => {
                // The branches still name and color what they use while the condition is wrong.
                self.check_dead_for_facts(f, scope, then_branch);
                if let Some(b) = else_branch {
                    self.check_dead_for_facts(f, scope, b);
                }
                return Err(e);
            }
        };
        if let Some(t) = Self::const_truth(&c) {
            if t {
                if let Some(e) = else_branch {
                    self.check_dead_for_facts(f, scope, e);
                }
                return self.check_scoped(f, scope, then_branch);
            }
            self.check_dead_for_facts(f, scope, then_branch);
            if let Some(e) = else_branch {
                return self.check_scoped(f, scope, e);
            }
            return Ok(());
        }
        let c = self.settle_untyped(c, None);
        let (ty, v) = self.rvalue(f, c, cond.span)?;
        let c = self.truthy(f, ty, v, cond.span)?;
        let then_block = f.b.new_block();
        let done = f.b.new_block();
        let else_block = if else_branch.is_some() {
            f.b.new_block()
        } else {
            done
        };
        f.b.branch(c, then_block, else_block);
        f.b.switch_to(then_block);
        self.check_scoped(f, scope, then_branch)?;
        f.b.jump(done);
        if let Some(e) = else_branch {
            f.b.switch_to(else_block);
            self.check_scoped(f, scope, e)?;
            f.b.jump(done);
        }
        f.b.switch_to(done);
        Ok(())
    }

    pub(super) fn check_switch(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        value: &ast::Expr,
        cases: &[ast::Case],
        complete: bool,
        (header, span): (Span, Span),
    ) -> Result<()> {
        let v = self.check_expr(f, scope, value, None)?;
        // Constant switch (types, constants): pick the case at compile time.
        if v.is_const() {
            let mut matched = None;
            let mut default = None;
            'outer: for (i, case) in cases.iter().enumerate() {
                if case.values.is_empty() {
                    default = Some(i);
                    continue;
                }
                for cv in &case.values {
                    let c = self.check_expr(f, scope, cv, Some(v.ty()))?;
                    if !c.is_const() {
                        matched = None;
                        default = None;
                        break 'outer;
                    }
                    if self.const_equal(&v, &c) {
                        matched = Some(i);
                        break 'outer;
                    }
                }
            }
            if let Some(start) = matched.or(default) {
                let mut taken = Vec::new();
                let mut i = start;
                loop {
                    taken.push(i);
                    let inner = self.new_block_scope(scope, cases[i].span);
                    let depth = f.mark_defers();
                    self.check_block_stmts(f, inner, &cases[i].body)?;
                    if !f.b.is_terminated() {
                        self.emit_defers(f, depth.position(), span)?;
                    }
                    f.truncate_defers(depth.position());
                    if !cases[i].through || i + 1 >= cases.len() {
                        break;
                    }
                    i += 1;
                }
                let switch = Switch {
                    vty: v.ty(),
                    cases,
                    taken: &taken,
                    complete,
                };
                return self.check_untaken_cases(f, scope, switch, header, span);
            }
            if cases.iter().all(|c| !c.values.is_empty()) {
                let switch = Switch {
                    vty: v.ty(),
                    cases,
                    taken: &[],
                    complete,
                };
                return self.check_untaken_cases(f, scope, switch, header, span);
            }
        }
        let v = self.settle_untyped(v, None);
        let vty = v.ty();
        if vty != TypeId::STRING && self.is_memory_type(vty) {
            return err(
                Code::StatementsNotSupport,
                value.span,
                "'if ==' statements do not support struct or array types at this time.",
            );
        }
        let (_, val) = self.rvalue(f, v, value.span)?;
        let done = f.b.new_block();
        let bodies: Vec<ir::BlockId> = cases.iter().map(|_| f.b.new_block()).collect();
        let mut default = done;
        // Enum values named by a case, for the `#complete` check below.
        let mut covered = Vec::new();
        let mut seen = Vec::new();
        for (i, case) in cases.iter().enumerate() {
            if case.values.is_empty() {
                default = bodies[i];
                continue;
            }
            for cv in &case.values {
                let raw = self.check_expr(f, scope, cv, Some(vty))?;
                let c = self.convert(f, raw.clone(), vty, cv)?;
                Self::check_case_value(f, &raw, &c, cv.span, &mut seen)?;
                if let Some(Value::Int(n)) = c.const_value() {
                    covered.push(n);
                }
                let (_, cval) = self.rvalue(f, c, cv.span)?;
                let eq = self.runtime_equal(f, vty, val, cval, cv.span)?;
                let next = f.b.new_block();
                f.b.branch(eq, bodies[i], next);
                f.b.switch_to(next);
            }
        }
        // A missing case is reported after the bodies are checked, so they still have types.
        let missing = if complete {
            self.ty.check_switch_complete(vty, &covered, header)
        } else {
            Ok(())
        };
        // Every member has a case, but the value may still be none of them (a cast, or
        // uninitialized memory): stop rather than fall through silently.
        if complete
            && missing.is_ok()
            && default == done
            && !f.type_only
            && let Some(fail) = self.unmatched_switch_check(f, vty, val, value.span)?
        {
            default = fail;
        }
        f.b.jump(default);
        for (i, case) in cases.iter().enumerate() {
            f.b.switch_to(bodies[i]);
            let inner = self.new_block_scope(scope, case.span);
            let depth = f.mark_defers();
            self.check_block_stmts(f, inner, &case.body)?;
            if !f.b.is_terminated() {
                self.emit_defers(f, depth.position(), span)?;
            }
            f.truncate_defers(depth.position());
            let next = if case.through && i + 1 < cases.len() {
                bodies[i + 1]
            } else {
                done
            };
            f.b.jump(next);
        }
        f.b.switch_to(done);
        missing
    }

    /// A case value must be a constant, and no two cases may name the same one. A constant
    /// that only converts at run time (`#char "+"` against an enum) still counts as one, as
    /// does the address of a program-wide object (`type_info(T)`).
    fn check_case_value(
        f: &FnCtx,
        raw: &Operand,
        c: &Operand,
        span: Span,
        seen: &mut Vec<(Value, Span)>,
    ) -> Result<()> {
        let value = match c {
            Operand::Procs(p) if p.len() == 1 => Value::Proc(p[0]),
            _ => match c.const_value().or_else(|| raw.const_value()) {
                Some(value) => value,
                None if Self::is_fixed_address(f, c) => return Ok(()),
                None => {
                    return err(
                        Code::ConditionsListedCase,
                        span,
                        "Conditions listed by 'case' must be constant.",
                    );
                }
            },
        };
        if let Some((_, earlier)) = seen.iter().find(|(v, _)| *v == value) {
            return Err(Box::new(
                Diagnostic::new(Code::DuplicateCase, span, "Duplicate case.")
                    .with_note(*earlier, "The earlier case is here."),
            ));
        }
        seen.push((value, span));
        Ok(())
    }

    /// Whether the operand is a value the program fixes before it runs: the address of a
    /// global or a procedure, or a literal the converter materialised.
    fn is_fixed_address(f: &FnCtx, c: &Operand) -> bool {
        let Operand::Value {
            val, ..
        } = c
        else {
            return false;
        };
        f.b.func.blocks.iter().flat_map(|b| &b.insts).any(|inst| {
            matches!(
                inst,
                ir::Inst::GlobalAddr { dst, .. }
                    | ir::Inst::FuncAddr { dst, .. }
                    | ir::Inst::IConst { dst, .. }
                    | ir::Inst::FConst { dst, .. }
                    if dst == val
            )
        })
    }

    /// A switch on a constant runs one case, but every case is still type-checked (only `#if`
    /// skips code), as is `#complete`. The bodies not taken are checked into a block nothing
    /// jumps to.
    fn check_untaken_cases(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        switch: Switch<'_>,
        header: Span,
        span: Span,
    ) -> Result<()> {
        let Switch {
            vty,
            cases,
            taken,
            complete,
        } = switch;
        let mut covered = Vec::new();
        let mut seen = Vec::new();
        for case in cases {
            for cv in &case.values {
                let raw = self.check_expr(f, scope, cv, Some(vty))?;
                let c = self.convert(f, raw.clone(), vty, cv)?;
                Self::check_case_value(f, &raw, &c, cv.span, &mut seen)?;
                if let Some(Value::Int(n)) = c.const_value() {
                    covered.push(n);
                }
            }
        }
        let here = f.b.current;
        let dead = f.b.new_block();
        f.b.switch_to(dead);
        for (i, case) in cases.iter().enumerate() {
            if taken.contains(&i) {
                continue;
            }
            let inner = self.new_block_scope(scope, case.span);
            let depth = f.mark_defers();
            self.check_block_stmts(f, inner, &case.body)?;
            if !f.b.is_terminated() {
                self.emit_defers(f, depth.position(), span)?;
            }
            f.truncate_defers(depth.position());
            if f.b.is_terminated() {
                let next = f.b.new_block();
                f.b.switch_to(next);
            }
        }
        if !f.b.is_terminated() {
            f.b.terminate(ir::Term::Unreachable);
        }
        f.b.switch_to(here);
        if complete {
            self.ty.check_switch_complete(vty, &covered, header)?;
        }
        Ok(())
    }

    /// The block a `#complete` switch on a (non-flags) enum without a default case jumps to when
    /// no case matched: it reports the value (`ir::CheckReason::SwitchUnmatched`) and stops. `None`
    /// for other switches. Leaves the builder where it was.
    fn unmatched_switch_check(
        &mut self,
        f: &mut FnCtx,
        ty: TypeId,
        val: ir::Val,
        span: Span,
    ) -> Result<Option<ir::BlockId>> {
        let TypeKind::Enum(id) = *self.ty.types.kind(ty) else {
            return Ok(None);
        };
        if self.ty.types.enums[id.0 as usize].is_flags {
            return Ok(None);
        }
        let Some(t) = self.ir_ty(ty) else {
            return Ok(None);
        };
        let Some((_, signed)) = self.ty.types.int_info(self.ty.types.repr(ty)) else {
            return Ok(None);
        };
        let here = f.b.current;
        let fail = f.b.new_block();
        f.b.switch_to(fail);
        let wide = if t == Ty::I64 {
            val
        } else {
            let op = if signed {
                ir::ConvOp::SExt
            } else {
                ir::ConvOp::ZExt
            };
            f.b.conv(op, t, Ty::I64, val)
        };
        let zero = f.b.iconst(Ty::I64, 0);
        self.emit_check_failed(f, ir::CheckReason::SwitchUnmatched, wide, zero, true, span)?;
        f.b.terminate(ir::Term::Unreachable);
        f.b.switch_to(here);
        Ok(Some(fail))
    }

    #[expect(
        clippy::float_cmp,
        reason = "constant folding of `==` between an integer and a float is exact, as at run time"
    )]
    pub(super) fn const_equal(&self, a: &Operand, b: &Operand) -> bool {
        match (a.const_value(), b.const_value()) {
            (Some(Value::Int(x)), Some(Value::Float(y, _)))
            | (Some(Value::Float(y, _)), Some(Value::Int(x))) => x as f64 == y,
            (Some(x), Some(y)) => x == y,
            _ => false,
        }
    }

    /// Runtime `a == b` for switch cases.
    fn runtime_equal(
        &mut self,
        f: &mut FnCtx,
        ty: TypeId,
        a: ir::Val,
        b: ir::Val,
        span: Span,
    ) -> Result<ir::Val> {
        match self.ir_ty(ty) {
            Some(t) if t.is_float() => Ok(f.b.cmp(CmpOp::FEq, t, a, b)),
            Some(t) => Ok(f.b.cmp(CmpOp::Eq, t, a, b)),
            None if ty == TypeId::STRING => {
                let r = self.string_equal(f, a, b);
                Ok(r)
            }
            None => {
                let size = self.size_of(ty, span)?;
                let n = f.b.iconst(Ty::I64, size);
                let r =
                    f.b.intrinsic(ir::Intrinsic::Memcmp, vec![a, b, n], &[Ty::I16]);
                let zero = f.b.iconst(Ty::I16, 0);
                Ok(f.b.cmp(CmpOp::Eq, Ty::I16, r[0], zero))
            }
        }
    }

    fn string_equal(&mut self, f: &mut FnCtx, x: ir::Val, y: ir::Val) -> ir::Val {
        let result = f.b.alloca(1, 1);
        let lc = f.b.load(Ty::I64, x);
        let rc = f.b.load(Ty::I64, y);
        let same = f.b.cmp(CmpOp::Eq, Ty::I64, lc, rc);
        f.b.store(Ty::I8, result, same);
        let cmp_block = f.b.new_block();
        let done = f.b.new_block();
        f.b.branch(same, cmp_block, done);
        f.b.switch_to(cmp_block);
        let xp = f.b.ptr_offset(x, 8);
        let xd = f.b.load(Ty::Ptr, xp);
        let yp = f.b.ptr_offset(y, 8);
        let yd = f.b.load(Ty::Ptr, yp);
        let r =
            f.b.intrinsic(ir::Intrinsic::Memcmp, vec![xd, yd, lc], &[Ty::I16]);
        let zero = f.b.iconst(Ty::I16, 0);
        let eq = f.b.cmp(CmpOp::Eq, Ty::I16, r[0], zero);
        f.b.store(Ty::I8, result, eq);
        f.b.jump(done);
        f.b.switch_to(done);
        f.b.load(Ty::I8, result)
    }
}

impl TypeState {
    /// `if #complete x == {`: every member of x's enum needs a case of its own, default or
    /// not. Flag enums and non-enum values are not checked.
    fn check_switch_complete(&self, ty: TypeId, covered: &[i128], span: Span) -> Result<()> {
        let TypeKind::Enum(id) = *self.types.kind(ty) else {
            return err(
                Code::CompleteOnlyApplies,
                span,
                "#complete only applies to 'if' on enum types.",
            );
        };
        let info = &self.types.enums[id.0 as usize];
        if info.is_flags {
            return Ok(());
        }
        let missing: Vec<String> = info
            .members
            .iter()
            .filter(|(_, value)| !covered.contains(value))
            .map(|(name, _)| format!(".{name}"))
            .collect();
        if missing.is_empty() {
            return Ok(());
        }
        Err(Box::new(Diagnostic::new(
            Code::IncompleteSwitch,
            span,
            format!(
                "#complete switch on {} has no case for {}",
                self.types.name(ty),
                missing.join(", ")
            ),
        )))
    }
}
