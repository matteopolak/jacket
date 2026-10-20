//! `ifx` expressions: branch agreement and constant folding.
use super::implicit_then_expr;
use crate::ast::ExprKind as E;
use crate::sema::convert::int_fits;
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::{Compiler, Diagnostic, Result, ScopeId, Span, TypeId, Value, ast};

/// An arm whose type only an expected type can give it: `.B`, `.{...}`, `.[...]`.
fn needs_expected_type(e: &ast::Expr) -> bool {
    matches!(
        e.kind,
        E::InferredMember(_)
            | E::StructLit {
                ty: None,
                ..
            }
            | E::ArrayLit {
                ty: None,
                ..
            }
    )
}

impl Compiler {
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

    /// With no target type, an untyped integer constant that does not fit the other arm's
    /// integer type makes the result `s64` when that holds both (`ifx c then 300 else a_u8`).
    fn ifx_overflow_widening(&mut self, (a, b): (&Operand, &Operand)) -> Option<TypeId> {
        let overflows = |c: &Self, constant: &Operand, other: &Operand| {
            let Operand::Const {
                value: Value::Int(v),
                untyped: true,
                ..
            } = constant
            else {
                return false;
            };
            if matches!(
                other,
                Operand::Const {
                    untyped: true,
                    ..
                }
            ) {
                return false;
            }
            let repr = c.ty.types.repr(other.ty());
            c.ty.types
                .int_info(repr)
                .is_some_and(|(bits, signed)| !int_fits(*v, bits, signed))
        };
        let other = if overflows(self, a, b) {
            b
        } else if overflows(self, b, a) {
            a
        } else {
            return None;
        };
        self.implicit_cost(other.ty(), false, TypeId::S64)
            .map(|_| TypeId::S64)
    }

    /// Both arms of an `ifx` must meet in one type: `ifx c then 1 else "a"` has none.
    fn ifx_branch_agreement(
        &mut self,
        (then_op, else_op): (&Operand, &Operand),
        (then_e, else_e): (&ast::Expr, &ast::Expr),
        (expected, operand): (Option<TypeId>, bool),
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
        // Two untyped integer arms make an `s64`, which an unsigned target does not take. An
        // operand of a binary operator is not a target: `c - (ifx up then 32 else 0)` with a
        // `u16` `c` takes the arms as that type.
        let untyped_int = |o: &Operand| {
            matches!(
                o,
                Operand::Const {
                    value: Value::Int(_),
                    untyped: true,
                    ..
                }
            )
        };
        if !operand
            && untyped_int(then_op)
            && untyped_int(else_op)
            && let Some(want) = expected
            && let Some((_, false)) = self.ty.types.int_info(self.ty.types.repr(want))
        {
            let wanted = self.ty.types.name(want).clone();
            return Err(Box::new(Diagnostic::error(
                span,
                format!(
                    "Number signedness mismatch. Type wanted: {wanted}; type given: {}.",
                    self.ty.types.name(TypeId::S64)
                ),
            )));
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
            ) && !c.ty.types.is_pointer(o.ty())
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
            (None, Some(b)) => !(number(self, then_op) && self.ty.types.repr(b) == TypeId::STRING),
            (Some(a), None) => !(number(self, else_op) && self.ty.types.repr(a) == TypeId::STRING),
            _ => true,
        };
        if agree {
            return Ok(());
        }
        let describe = |c: &mut Self, o: &Operand| {
            if let Some(t) = Self::ifx_branch_type(o) {
                c.ty.types.name(t).clone()
            } else {
                let d = c.settle_untyped(o.clone(), None).ty();
                format!("{} (untyped constant)", c.ty.types.name(d))
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
    fn fold_constant_ifx(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        cond_holds: bool,
        (then_e, else_e): (&ast::Expr, &ast::Expr),
        (expected, operand): (Option<TypeId>, bool),
        span: Span,
    ) -> Result<Operand> {
        let (taken_e, other_e) = if cond_holds {
            (then_e, else_e)
        } else {
            (else_e, then_e)
        };
        // An arm that only an expected type can type takes the other arm's type.
        let mut scratch = self.scratch_ctx(scope);
        let (taken, other);
        if expected.is_none() && needs_expected_type(taken_e) && !needs_expected_type(other_e) {
            other = self.check_expr(&mut scratch, scope, other_e, None)?;
            let hint = Self::ifx_branch_type(&other);
            taken = self.check_expr(f, scope, taken_e, hint)?;
        } else {
            taken = self.check_expr(f, scope, taken_e, expected)?;
            // The other arm is checked for its type and diagnostics but emits no code. It
            // still sees the expected type, so `.ARM64` there names the target's enum.
            let hint = match Self::ifx_branch_type(&taken) {
                Some(t) if expected.is_none() && needs_expected_type(other_e) => Some(t),
                _ => expected,
            };
            other = self.check_expr(&mut scratch, scope, other_e, hint)?;
        }
        let (then_op, else_op) = if cond_holds {
            (&taken, &other)
        } else {
            (&other, &taken)
        };
        self.ifx_branch_agreement(
            (then_op, else_op),
            (then_e, else_e),
            (expected, operand),
            span,
        )?;
        // A one-character string only becomes a byte where a string is converted directly,
        // not through `ifx`.
        if let (
            Operand::Const {
                value: Value::String(_),
                ..
            },
            Some(e),
        ) = (&taken, expected)
            && self.ty.types.is_integer(self.ty.types.repr(e))
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
        let operand = std::mem::take(&mut f.ifx_operand);
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
                    return self.fold_constant_ifx(
                        f,
                        scope,
                        t,
                        (then_e, else_e),
                        (expected, operand),
                        span,
                    );
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
        // A then-value that only an expected type can type (`.B`, `.{}`, `.[1, 2]`) takes the
        // else-value's type, so the else-value is checked first.
        let mut early_else = None;
        let mut then_hint = expected;
        if expected.is_none()
            && then_value.is_some_and(needs_expected_type)
            && let Some(else_e) = else_value
            && !needs_expected_type(else_e)
        {
            f.b.switch_to(else_block);
            let op = self.check_expr(f, scope, else_e, None)?;
            then_hint = Self::ifx_branch_type(&op);
            early_else = Some(op);
            f.b.switch_to(then_block);
        }
        let then_op = match (then_value, implicit_then) {
            (Some(e), _) | (None, Some(e)) => self.check_expr(f, scope, e, then_hint)?,
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
            let r = c.ty.types.repr(t);
            c.ty.types.is_integer(r) || c.ty.types.is_float(r)
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
        let else_op = match (early_else, else_value) {
            (Some(op), _) => Some(op),
            (None, Some(e)) => Some(self.check_expr(f, scope, e, result_ty.or(expected))?),
            (None, None) => None,
        };
        if let (Some(then_e), Some(else_e), Some(else_op)) = (then_value, else_value, &else_op) {
            self.ifx_branch_agreement(
                (&then_op, else_op),
                (then_e, else_e),
                (expected, operand),
                span,
            )?;
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
        let overflow = match &else_op {
            Some(op) if expected.is_none() => self.ifx_overflow_widening((&then_op, op)),
            _ => None,
        };
        let ty = match (overflow.or(else_wider).or(result_ty), &else_op) {
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
