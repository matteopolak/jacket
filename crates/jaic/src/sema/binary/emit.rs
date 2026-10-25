//! Unifying operand types and emitting the operation, with its overflow check.
use super::{Binary, Unified, arith_op, cmp_op, is_comparison, is_shift};
use crate::ast::BinOp;
use crate::ir::{CmpOp, Ty};
use crate::sema::calls::binop_text;
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::scope::Resolved;
use crate::sema::{Compiler, Diagnostic, Rc, Result, Span, Sym, TypeId, Value, ast, err, ir};
use crate::types::TypeKind;

impl Compiler {
    /// Convert both operands to the operation's type and load them. Returns that type, the
    /// right operand's type (a shift amount keeps its own) and the two values.
    pub(super) fn unify_binary_operands(
        &mut self,
        f: &mut FnCtx,
        node: Binary<'_>,
        (mut lhs, mut rhs): (Operand, Operand),
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Unified> {
        let Binary {
            op,
            a,
            b,
        } = node;
        let is_shift = is_shift(op);
        // An integer variable meeting an untyped float constant (`n * 0.5`) is converted to float.
        let is_float_lit = |o: &Operand| {
            matches!(
                o,
                Operand::Const {
                    value: Value::Float(_, _),
                    untyped: true,
                    ..
                }
            )
        };
        let is_int_value = |s: &Self, o: &Operand| {
            !matches!(
                o,
                Operand::Const {
                    untyped: true,
                    ..
                }
            ) && s.ty.types.is_integer(o.ty())
                && !matches!(s.ty.types.kind(o.ty()), TypeKind::Enum(_))
        };
        if !is_shift && is_float_lit(&rhs) && is_int_value(self, &lhs) {
            let ty = self
                .ty
                .default_untyped(rhs.ty(), &rhs.const_value().unwrap());
            lhs = self.explicit_cast(f, lhs, ty, ast::CastFlags::default(), span)?;
        } else if !is_shift && is_float_lit(&lhs) && is_int_value(self, &rhs) {
            let ty = self
                .ty
                .default_untyped(lhs.ty(), &lhs.const_value().unwrap());
            rhs = self.explicit_cast(f, rhs, ty, ast::CastFlags::default(), span)?;
        }
        // Comparing integers compares their values: a distinct integer type meets another
        // integer type as the type it is made from.
        if is_comparison(op)
            && lhs.ty() != rhs.ty()
            && !matches!(
                lhs,
                Operand::Const {
                    untyped: true,
                    ..
                }
            )
            && !matches!(
                rhs,
                Operand::Const {
                    untyped: true,
                    ..
                }
            )
        {
            for operand in [&mut lhs, &mut rhs] {
                let ty = operand.ty();
                let base = self.ty.types.repr(ty);
                if base != ty && is_int_value(self, operand) {
                    *operand = self.explicit_cast(
                        f,
                        operand.clone(),
                        base,
                        ast::CastFlags::default(),
                        span,
                    )?;
                }
            }
        }
        // Unify operand types.
        let ty =
            self.binary_operand_type(&lhs, &rhs, expected, is_shift, (a.span, b.span), span)?;
        // A negative constant cannot meet a `u64`: nothing wider takes its place.
        if !is_shift {
            for (constant, other) in [(&rhs, &lhs), (&lhs, &rhs)] {
                if let Operand::Const {
                    value: Value::Int(v),
                    untyped: true,
                    ..
                } = constant
                    && *v < 0
                    && !matches!(
                        other,
                        Operand::Const {
                            untyped: true,
                            ..
                        }
                    )
                    && self.ty.types.int_info(other.ty()) == Some((64, false))
                    && !matches!(self.ty.types.kind(other.ty()), TypeKind::Enum(_))
                {
                    return err(
                        span,
                        format!(
                            "signedness mismatch: the constant {v} does not fit in `{}`",
                            self.ty.types.name(other.ty())
                        ),
                    );
                }
            }
        }
        if self.ty.types.repr(ty) == TypeId::BOOL
            && matches!(
                op,
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem
            )
        {
            let symbol = binop_text(op);
            return err(span, format!("operator `{symbol}` is not defined for bool"));
        }
        // A literal that met the other operand's type takes it as a bit pattern
        // (`mask & ~0x7` with a `u32` mask).
        if !is_shift {
            lhs = self.ty.literal_as_bits(lhs, ty);
            rhs = self.ty.literal_as_bits(rhs, ty);
        }
        let rhs_ty = if is_shift {
            self.ty.shift_amount_type(&rhs, ty)
        } else {
            ty
        };
        // An integer value meeting a float converts like `cast(float) x`.
        let int_to_float = |s: &Self, o: &Operand| {
            !is_shift && s.ty.types.is_float(ty) && s.ty.types.is_integer(o.ty())
        };
        if int_to_float(self, &lhs) {
            lhs = self.explicit_cast(f, lhs, ty, ast::CastFlags::default(), span)?;
        }
        if int_to_float(self, &rhs) {
            rhs = self.explicit_cast(f, rhs, ty, ast::CastFlags::default(), span)?;
        }
        if ty == TypeId::VOID_PTR && matches!(self.ty.types.kind(lhs.ty()), TypeKind::Proc(_)) {
            lhs = self.explicit_cast(f, lhs, ty, ast::CastFlags::default(), span)?;
        }
        if ty == TypeId::VOID_PTR && matches!(self.ty.types.kind(rhs.ty()), TypeKind::Proc(_)) {
            rhs = self.explicit_cast(f, rhs, ty, ast::CastFlags::default(), span)?;
        }
        let lhs = self.convert(f, lhs, ty, a.span)?;
        let rhs = self.convert(f, rhs, rhs_ty, b.span)?;
        let (_, x) = self.rvalue(f, lhs, span)?;
        let (_, y) = self.rvalue(f, rhs, span)?;
        Ok(Unified {
            ty,
            rhs_ty,
            x,
            y,
        })
    }

    /// Emit `x op y` on values of type `ty` (`rhs_ty` for a shift amount).
    pub(super) fn emit_binary(
        &mut self,
        f: &mut FnCtx,
        op: BinOp,
        values: Unified,
        span: Span,
    ) -> Result<Operand> {
        let Unified {
            ty,
            rhs_ty,
            x,
            y,
        } = values;
        let (is_cmp, is_shift) = (is_comparison(op), is_shift(op));
        if ty == TypeId::STRING && is_cmp {
            return self.string_compare(f, op, x, y, span);
        }
        if self.ty.types.wide_float(ty).is_some() && !is_shift {
            return self.wide_binary(f, ty, op, x, y, span);
        }
        let Some(t) = self.ir_ty(ty) else {
            if is_cmp && matches!(op, BinOp::Eq | BinOp::Ne) {
                // Aggregate equality: bytewise.
                let size = self.size_of(ty, span)?;
                let n = f.b.iconst(Ty::I64, size);
                let r =
                    f.b.intrinsic(ir::Intrinsic::Memcmp, vec![x, y, n], &[Ty::I16]);
                let zero = f.b.iconst(Ty::I16, 0);
                let cmp = if op == BinOp::Eq {
                    CmpOp::Eq
                } else {
                    CmpOp::Ne
                };
                return Ok(Operand::Value {
                    ty: TypeId::BOOL,
                    val: f.b.cmp(cmp, Ty::I16, r[0], zero),
                });
            }
            return err(
                span,
                format!(
                    "operator {} is not defined for {}",
                    crate::sema::calls::binop_text(op),
                    self.ty.types.name(ty)
                ),
            );
        };
        let signed = self.ty.types.int_info(ty).is_some_and(|(_, s)| s);
        if is_cmp {
            let cmp = cmp_op(op, t.is_float(), signed);
            return Ok(Operand::Value {
                ty: TypeId::BOOL,
                val: f.b.cmp(cmp, t, x, y),
            });
        }
        if matches!(op, BinOp::Div | BinOp::Rem) && !t.is_float() && !f.type_only {
            // Native code reports a division by zero through Runtime_Support.
            self.note_check_handler(span)?;
        }
        let ir_op = arith_op(op, t.is_float(), signed).ok_or_else(|| {
            Box::new(Diagnostic::error(
                span,
                format!(
                    "operator {} is not defined for {}",
                    crate::sema::calls::binop_text(op),
                    self.ty.types.name(ty)
                ),
            ))
        })?;
        if t == Ty::Ptr {
            return err(
                span,
                format!(
                    "arithmetic operator {} is not defined for {}",
                    crate::sema::calls::binop_text(op),
                    self.ty.types.name(ty)
                ),
            );
        }
        let y = if is_shift {
            let rt = self.ir_ty(rhs_ty).unwrap();
            if rt.size() == t.size() {
                y
            } else {
                let conv = if rt.size() > t.size() {
                    ir::ConvOp::Trunc
                } else {
                    ir::ConvOp::ZExt
                };
                f.b.conv(conv, rt, t, y)
            }
        } else {
            y
        };
        let val = f.b.bin(ir_op, t, x, y);
        if self.options.arithmetic_overflow_check != 0
            && !f.checks_off.no_aoc()
            && !f.type_only
            && matches!(ir_op, ir::BinOp::Add | ir::BinOp::Sub | ir::BinOp::Mul)
            && self.ty.types.is_integer(ty)
            && !matches!(self.ty.types.kind(ty), TypeKind::Enum(_))
        {
            self.emit_overflow_check(f, (ir_op, signed, t), (x, y), span)?;
        }
        Ok(Operand::Value {
            ty,
            val,
        })
    }

    /// After `x op y` on integers of IR type `t`: call the runtime's `__arithmetic_overflow`
    /// when the result did not fit (`Build_Options.arithmetic_overflow_check`).
    pub(in crate::sema) fn emit_overflow_check(
        &mut self,
        f: &mut FnCtx,
        (op, signed, t): (ir::BinOp, bool, Ty),
        (x, y): (ir::Val, ir::Val),
        span: Span,
    ) -> Result<()> {
        use ir::Intrinsic as I;
        let (intrinsic, operator) = match (op, signed) {
            (ir::BinOp::Add, true) => (I::SAddOverflow, 1u64),
            (ir::BinOp::Add, false) => (I::UAddOverflow, 1),
            (ir::BinOp::Sub, true) => (I::SSubOverflow, 2),
            (ir::BinOp::Sub, false) => (I::USubOverflow, 2),
            (ir::BinOp::Mul, true) => (I::SMulOverflow, 3),
            _ => (I::UMulOverflow, 3),
        };
        let Some(runtime) = self.ns.runtime_support else {
            return Ok(());
        };
        let handler = self.module_declarations(runtime, Sym::intern("__arithmetic_overflow"))?;
        let entity = match handler[..] {
            [entity] => entity,
            [] => {
                return err(
                    span,
                    "Runtime_Support does not define `__arithmetic_overflow`",
                );
            }
            _ => {
                return err(
                    span,
                    "Runtime_Support defines `__arithmetic_overflow` more than once",
                );
            }
        };
        let Resolved::Proc(proc) = self.resolve_entity(entity)? else {
            return err(span, "`__arithmetic_overflow` is not a procedure");
        };
        let crate::sema::procs::ProcTarget::Func(func) = self.proc_func(proc, span)? else {
            return err(span, "`__arithmetic_overflow` must be defined in Jai");
        };
        let width = f.b.iconst(Ty::I64, t.size());
        let overflowed = f.b.intrinsic(intrinsic, vec![x, y, width], &[Ty::I8])[0];
        let fail = f.b.new_block();
        let cont = f.b.new_block();
        f.b.branch(overflowed, fail, cont);
        f.b.switch_to(fail);
        let widen = if signed {
            ir::ConvOp::SExt
        } else {
            ir::ConvOp::ZExt
        };
        let (left, right) = if t == Ty::I64 {
            (x, y)
        } else {
            (
                f.b.conv(widen, t, Ty::I64, x),
                f.b.conv(widen, t, Ty::I64, y),
            )
        };
        // Same layout as the reference: bit 15 fatal, bit 14 signed, bits 7-8 operator, low bits size.
        let code = (u64::from(self.options.arithmetic_overflow_check == 2) << 15)
            | (u64::from(signed) << 14)
            | (operator << 7)
            | t.size();
        let code = f.b.iconst(Ty::I16, code);
        let file = self.src.sources.get(span.file);
        let (line, _) = file.line_col(span.start);
        let path: Rc<[u8]> = Rc::from(file.path.to_string().as_bytes());
        let line = f.b.iconst(Ty::I64, line as u64);
        let global = self.out.string_global(&path);
        let filename = f.b.global_addr(global);
        f.b.call(
            ir::Callee::Func(func),
            vec![left, right, code, line, filename],
            &[],
        );
        f.b.jump(cont);
        f.b.switch_to(cont);
        Ok(())
    }
}
