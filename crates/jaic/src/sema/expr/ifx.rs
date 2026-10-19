//! `ifx` expressions: branch agreement and constant folding.
use super::implicit_then_expr;
use crate::ir::Ty;
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::{Compiler, Diagnostic, Result, ScopeId, Span, TypeId, Value, ast};

impl Compiler {
    #[allow(clippy::too_many_arguments)]
    /// The type an `ifx` branch contributes, or `None` when it adapts to the other branch
    /// (an untyped constant) or is not a plain value.
    fn ifx_branch_type(op: &Operand) -> Option<TypeId> {
        match op {
            Operand::Const {
                untyped: true, ..
            }
            | Operand::Procs(_)
            | Operand::Void => None,
            other => Some(other.ty()),
        }
    }

    /// Both arms of an `ifx` must meet in one type: `ifx c then 1 else "a"` has none.
    fn ifx_branch_agreement(
        &mut self,
        then_op: &Operand,
        else_op: &Operand,
        then_e: &ast::Expr,
        else_e: &ast::Expr,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<()> {
        if matches!(
            then_op,
            Operand::Procs(_) | Operand::Void | Operand::Type(_)
        ) || matches!(
            else_op,
            Operand::Procs(_) | Operand::Void | Operand::Type(_)
        ) {
            return Ok(());
        }
        let (tt, et) = (
            Self::ifx_branch_type(then_op),
            Self::ifx_branch_type(else_op),
        );
        let number = |c: &Self, o: &Operand| {
            matches!(
                o,
                Operand::Const {
                    value: Value::Int(_) | Value::Float(_, _),
                    untyped: true,
                    ..
                }
            ) && !c.types.is_pointer(o.ty())
        };
        let agree = match (tt, et) {
            (Some(a), Some(b)) if a != b => {
                let to = |c: &mut Self, from: TypeId, into: TypeId| {
                    c.implicit_cost(from, false, into).is_some()
                };
                to(self, a, b)
                    || to(self, b, a)
                    || expected.is_some_and(|e| to(self, a, e) && to(self, b, e))
            }
            (None, Some(b)) => !(number(self, then_op) && self.types.repr(b) == TypeId::STRING),
            (Some(a), None) => !(number(self, else_op) && self.types.repr(a) == TypeId::STRING),
            _ => true,
        };
        if agree {
            return Ok(());
        }
        let describe = |c: &mut Self, o: &Operand| {
            if let Some(t) = Self::ifx_branch_type(o) {
                c.types.name(t).clone()
            } else {
                let d = c.settle_untyped(o.clone(), None).ty();
                format!("{} (untyped constant)", c.types.name(d))
            }
        };
        let (a, b) = (describe(self, then_op), describe(self, else_op));
        Err(Box::new(
            Diagnostic::error(span, "the branches of `ifx` have different types")
                .with_note(then_e.span, format!("the `then` value is `{a}`"))
                .with_note(else_e.span, format!("the `else` value is `{b}`")),
        ))
    }

    /// `ifx` with a constant condition: the taken branch is the value, so it stays constant,
    /// but the other branch is still checked, and the arms must still agree on a type.
    #[allow(clippy::too_many_arguments)]
    fn fold_constant_ifx(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        cond_holds: bool,
        then_e: &ast::Expr,
        else_e: &ast::Expr,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        let (taken_e, other_e) = if cond_holds {
            (then_e, else_e)
        } else {
            (else_e, then_e)
        };
        let taken = self.check_expr(f, scope, taken_e, expected)?;
        // The other arm is lowered into a block nothing reaches.
        let dead = f.b.new_block();
        let after = f.b.new_block();
        let never = f.b.iconst(Ty::I8, 0);
        f.b.branch(never, dead, after);
        f.b.switch_to(dead);
        let other = self.check_expr(f, scope, other_e, expected);
        f.b.jump(after);
        f.b.switch_to(after);
        let other = other?;
        let (then_op, else_op) = if cond_holds {
            (&taken, &other)
        } else {
            (&other, &taken)
        };
        self.ifx_branch_agreement(then_op, else_op, then_e, else_e, expected, span)?;
        // A one-character string only becomes a byte where a string is converted directly,
        // not through `ifx`.
        if let (
            Operand::Const {
                value: Value::String(_),
                ..
            },
            Some(e),
        ) = (&taken, expected)
            && self.types.is_integer(self.types.repr(e))
        {
            let (ty, val) = self.rvalue(f, taken.clone(), span)?;
            self.convert(
                f,
                Operand::Value {
                    ty,
                    val,
                },
                e,
                span,
            )?;
        }
        Ok(taken)
    }

    pub(super) fn check_ifx(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        cond: &ast::Expr,
        (then_value, else_value): (Option<&ast::Expr>, Option<&ast::Expr>),
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        let cond_op = self.check_expr(
            f,
            scope,
            cond,
            then_value.map_or(expected, |_| Some(TypeId::BOOL)),
        )?;
        self.ide_note_ifx(cond, then_value, else_value);
        // Compile-time condition: only check the taken branch.
        if let Operand::Const {
            value, ..
        } = &cond_op
            && then_value.is_some()
        {
            let taken = match value {
                Value::Bool(b) => Some(*b),
                Value::Int(i) => Some(*i != 0),
                Value::Null => Some(false),
                _ => None,
            };
            if let Some(t) = taken {
                if let (Some(then_e), Some(else_e)) = (then_value, else_value) {
                    return self.fold_constant_ifx(f, scope, t, then_e, else_e, expected, span);
                }
                if t {
                    return self.check_expr(f, scope, then_value.unwrap(), expected);
                } else if let Some(else_value) = else_value {
                    return self.check_expr(f, scope, else_value, expected);
                }
                // A false condition without else still needs the then type.
                // Lower it into the untaken block below, so discovering that
                // type does not execute the arm's effects.
            }
        }
        let cond_op = self.settle_untyped(cond_op, None);
        let (cty, cv) = self.rvalue(f, cond_op, span)?;
        let c = self.truthy(f, cty, cv, span)?;
        let then_block = f.b.new_block();
        let else_block = f.b.new_block();
        let done = f.b.new_block();
        f.b.branch(c, then_block, else_block);
        f.b.switch_to(then_block);
        let implicit_then = match then_value {
            None => implicit_then_expr(cond),
            Some(_) => None,
        };
        let then_op = match (then_value, implicit_then) {
            (Some(e), _) | (None, Some(e)) => self.check_expr(f, scope, e, expected)?,
            (None, None) => Operand::Value {
                ty: cty,
                val: cv,
            },
        };
        let then_ty = match &then_op {
            Operand::Const {
                untyped: true, ..
            } => None,
            // An overload set takes the expected procedure type (picked by `convert`), or the
            // else-value's.
            Operand::Procs(p) if p.len() > 1 || expected.is_some() => None,
            Operand::Procs(p) => Some(self.proc_type(p[0], span)?),
            other => Some(other.ty()),
        };
        // A numeric then-value widens to an expected numeric type, so the else-value may be
        // wider (`ifx c then small_u8 else big_s32` into an `s32`).
        let numeric = |c: &Compiler, t: TypeId| {
            let r = c.types.repr(t);
            c.types.is_integer(r) || c.types.is_float(r)
        };
        let widen = match (then_ty, expected) {
            (Some(t), Some(e)) if t != e && numeric(self, t) && numeric(self, e) => {
                self.implicit_cost(t, false, e).map(|_| e)
            }
            _ => None,
        };
        let result_ty = widen.or(then_ty).or(expected);
        let then_end = f.b.current;
        // Check else first to learn its type when then is untyped.
        f.b.switch_to(else_block);
        let else_op = match else_value {
            Some(e) => Some(self.check_expr(f, scope, e, result_ty.or(expected))?),
            None => None,
        };
        if let (Some(then_e), Some(else_e), Some(else_op)) = (then_value, else_value, &else_op) {
            self.ifx_branch_agreement(&then_op, else_op, then_e, else_e, expected, span)?;
        }
        // Without a target type, a narrower numeric then-value widens to the else-value's
        // type (`ifx hit then entry.score_s16 else evaluate()` is an `s64`).
        let else_wider = match (then_ty, expected, &else_op) {
            (Some(t), None, Some(op))
                if numeric(self, t)
                    && !matches!(
                        op,
                        Operand::Const {
                            untyped: true,
                            ..
                        }
                    )
                    && numeric(self, op.ty())
                    && op.ty() != t
                    && self.implicit_cost(op.ty(), false, t).is_none()
                    && self.implicit_cost(t, false, op.ty()).is_some() =>
            {
                Some(op.ty())
            }
            _ => None,
        };
        let ty = match (else_wider.or(result_ty), &else_op) {
            (Some(t), _) => t,
            (None, Some(Operand::Procs(p))) if p.len() == 1 => self.proc_type(p[0], span)?,
            (None, Some(op)) => self.settle_untyped(op.clone(), None).ty(),
            // `ifx c then 5`: an untyped then-value takes its default type.
            (None, None) => self.settle_untyped(then_op.clone(), None).ty(),
        };
        if ty == TypeId::VOID {
            // A discarded conditional can join void arms. Their effects are
            // already emitted in their own blocks; there is no result storage
            // to load. Conversion still rejects an incompatible value arm.
            if let Some(else_op) = else_op {
                self.convert(f, else_op, ty, span)?;
            }
            f.b.jump(done);
            f.b.switch_to(then_end);
            self.convert(f, then_op, ty, span)?;
            f.b.jump(done);
            f.b.switch_to(done);
            return Ok(Operand::Void);
        }
        let size = self.size_of(ty, span)?;
        let align = self.align_of(ty, span)?;
        // The result slot must be allocated before both branches use it; slots are function-wide.
        let slot = f.b.slot(size.max(1), align);
        let addr = f.b.slot_addr(slot);
        let mut callable_parameters = None;
        match (else_op, else_value) {
            (Some(else_op), Some(else_value)) => {
                let else_converted = self.convert(f, else_op, ty, else_value.span)?;
                // Runtime selection changes the callee, not its lexical call
                // header: inference retains the else arm's names/defaults.
                callable_parameters = self.callable_operand_parameters(&else_converted)?;
                let (_, ev) = self.rvalue(f, else_converted, span)?;
                self.store_value(f, ty, addr, ev, span)?;
            }
            _ => self.init_default(f, ty, addr, span)?,
        }
        f.b.jump(done);
        f.b.switch_to(then_end);
        let then_converted = self.convert(f, then_op, ty, span)?;
        let (_, tv) = self.rvalue(f, then_converted, span)?;
        let addr = f.b.slot_addr(slot);
        self.store_value(f, ty, addr, tv, span)?;
        f.b.jump(done);
        f.b.switch_to(done);
        let addr = f.b.slot_addr(slot);
        match self.ir_ty(ty) {
            Some(t) => Ok(Operand::Value {
                ty,
                val: f.b.load(t, addr),
            }),
            None => Ok(Operand::Value {
                ty,
                val: addr,
            }),
        }
        .map(|operand| operand.with_callable(callable_parameters))
    }
}
