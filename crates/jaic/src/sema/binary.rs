//! Binary operators: operand checking and typing, constant folding, pointer arithmetic and
//! the lowered operation.
use super::calls::binop_text;
use super::expr::wrap_int;
use super::lower::{FnCtx, Operand};
use super::scope::Resolved;
use super::*;
use crate::ast::{BinOp, ExprKind as E};
use crate::ir::{CmpOp, Ty};
use crate::types::TypeKind;

impl Compiler {
    /// `a op b` for every operator but `&&` and `||`: check the operands, then fold, overload,
    /// do pointer arithmetic, or convert both sides to one type and emit the operation.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn check_binary(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        op: BinOp,
        a: &ast::Expr,
        b: &ast::Expr,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        if matches!(op, BinOp::And | BinOp::Or) {
            return self.check_logical(f, scope, op, a, b, span);
        }
        // `cast(float32) ((ifx c then a else b) & mask)`: a float target cannot be the type of an
        // operand of an operator that floats lack, so it does not reach the operands (an `ifx`
        // would widen to it, and the operator would then fail on a float).
        let expected = if is_bitwise(op) || op == BinOp::Rem {
            expected.filter(|&t| !self.types.is_float(self.types.repr(t)))
        } else {
            expected
        };
        let (lhs, rhs) = self.binary_operands(f, scope, op, a, b, expected)?;
        // Type comparisons fold at compile time.
        if let (Operand::Type(x), Operand::Type(y)) = (&lhs, &rhs) {
            return match op {
                BinOp::Eq => Ok(Operand::bool(x == y)),
                BinOp::Ne => Ok(Operand::bool(x != y)),
                _ => err(span, "types can only be compared with == and !="),
            };
        }
        if let Some(result) = self.try_binary_operator_overload(f, scope, op, &lhs, &rhs, span)? {
            return Ok(result);
        }
        if let (
            Operand::Const {
                ..
            },
            Operand::Const {
                ..
            },
        ) = (&lhs, &rhs)
            && let Some(folded) = self.fold_binary(op, &lhs, &rhs, expected, span)?
        {
            return Ok(folded);
        }
        let (lhs, rhs) = self.procedure_operands(f, lhs, rhs, span)?;
        let (lty, rty) = (lhs.ty(), rhs.ty());
        if matches!(op, BinOp::Eq | BinOp::Ne) {
            self.reject_array_comparison(op, lty, rty, span)?;
        }
        if self.types.is_pointer(lty) && matches!(op, BinOp::Add | BinOp::Sub) {
            return self.pointer_offset(f, op, lhs, rhs, span);
        }
        if self.types.is_pointer(rty) && op == BinOp::Add && self.types.is_integer(lty) {
            return self.check_binary(f, scope, op, b, a, expected, span);
        }
        let pointer_and_int = (self.types.is_pointer(lty) && self.types.is_integer(rty))
            || (self.types.is_pointer(rty) && self.types.is_integer(lty));
        if is_bitwise(op) && pointer_and_int {
            return self.mask_pointer(f, op, lhs, rhs, span);
        }
        let (ty, rhs_ty, x, y) =
            self.unify_binary_operands(f, op, (a, b), lhs, rhs, expected, span)?;
        self.emit_binary(f, op, ty, rhs_ty, x, y, span)
    }

    /// Check both operands of `a op b`, each with the type the other or `expected` gives it.
    fn binary_operands(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        op: BinOp,
        a: &ast::Expr,
        b: &ast::Expr,
        expected: Option<TypeId>,
    ) -> Result<(Operand, Operand)> {
        let is_cmp = is_comparison(op);
        let is_shift = is_shift(op);
        let lhs_expected = if is_cmp {
            None
        } else if is_shift {
            // `cast(float) (1 << n)` shifts an integer.
            expected.filter(|&t| self.types.is_integer(self.types.repr(t)))
        } else {
            expected
        };
        // `.FIRST == x`: an inferred member on the left takes the right operand's type, so
        // the right side is checked first.
        let mut early_rhs = None;
        let is_xx = |e: &ast::Expr| {
            matches!(
                e.kind,
                E::Cast {
                    ty: None,
                    ..
                }
            )
        };
        // `.FLAG & x.flags` too: the member takes the flags type, not an expected `int`.
        // An expected enum type already names the member (`.A & ~.B` as a flags argument).
        let bitwise = matches!(op, BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor)
            && !lhs_expected.is_some_and(|t| {
                matches!(
                    self.types.kind(self.types.repr_struct(t)),
                    TypeKind::Enum(_)
                )
            });
        let mut lhs = if ((is_cmp
            && (matches!(a.kind, E::InferredMember(_)) || (is_xx(a) && !is_xx(b))))
            || (bitwise && matches!(a.kind, E::InferredMember(_))))
            && !matches!(b.kind, E::InferredMember(_))
        {
            // `xx err == GL_FALSE` casts to the other operand's type too.
            let r = self.check_expr(f, scope, b, None)?;
            let l = self.check_expr(f, scope, a, Some(r.ty()))?;
            early_rhs = Some(r);
            l
        } else if lhs_expected.is_some() && is_number_literal(a) {
            // `x: float32 = 0 - w`: the literal is matched with `w`, not with the declaration, so
            // the subtraction is in `u16` and only its result converts. It stays untyped here and
            // takes the right operand's type below; with an untyped right operand, both fold
            // under the expected type as before {#num.16}.
            self.check_expr(f, scope, a, None)?
        } else {
            let own = is_complement_literal(a) && !is_shift && !is_cmp;
            self.check_expr(
                f,
                scope,
                a,
                if own {
                    None
                } else {
                    lhs_expected
                },
            )?
        };
        if let Operand::PolyStruct(ps) = lhs {
            lhs = Operand::Type(self.poly_struct_type(ps));
        }
        // Inferred enum members on the right take the left operand's type.
        let pointer_lhs = self.types.is_pointer(self.types.repr(lhs.ty()));
        let pointer_offset = matches!(op, BinOp::Add | BinOp::Sub) && pointer_lhs;
        let pointer_bits =
            matches!(op, BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor) && pointer_lhs;
        let rhs_expected = if is_shift || pointer_bits {
            None
        } else if pointer_offset {
            // `p += ifx c then 3 else 1`: the offset is an integer.
            Some(TypeId::S64)
        } else {
            (!matches!(
                lhs,
                Operand::Const {
                    untyped: true,
                    ..
                }
            ))
            .then(|| lhs.ty())
        };
        let mut rhs = match early_rhs {
            Some(r) => r,
            None => self.check_expr(
                f,
                scope,
                b,
                if is_complement_literal(b) && !is_shift && !is_cmp {
                    None
                } else {
                    rhs_expected.or(if is_cmp || is_shift {
                        None
                    } else {
                        expected
                    })
                },
            )?,
        };
        if let Operand::PolyStruct(ps) = rhs {
            rhs = Operand::Type(self.poly_struct_type(ps));
        }
        // `s[0] == "-"`, `c - "0"`: a one-byte string constant next to a `u8` compares and
        // offsets as that byte. Wider integers do not take the conversion.
        if is_cmp || matches!(op, BinOp::Add | BinOp::Sub) {
            let byte_of = |c: &Compiler, op: &Operand, other: &Operand| match op {
                Operand::Const {
                    value: Value::String(bytes),
                    ..
                } if bytes.len() == 1 && c.types.repr(other.ty()) == TypeId::U8 => {
                    Some(Operand::untyped_int(bytes[0] as i128))
                }
                _ => None,
            };
            if let Some(b) = byte_of(self, &rhs, &lhs) {
                rhs = b;
            } else if let Some(b) = byte_of(self, &lhs, &rhs) {
                lhs = b;
            }
        }
        if matches!(
            lhs,
            Operand::Const {
                untyped: true,
                ..
            }
        ) && !matches!(
            rhs,
            Operand::Const {
                untyped: true,
                ..
            }
        ) && !is_shift
        {
            // Re-check the left side with the right side's type for inferred members.
            if matches!(a.kind, E::InferredMember(_)) {
                lhs = self.check_expr(f, scope, a, Some(rhs.ty()))?;
            }
        }
        if matches!(a.kind, E::InferredMember(_)) && !matches!(b.kind, E::InferredMember(_)) {
            lhs = self.check_expr(f, scope, a, Some(rhs.ty()))?;
        }
        if matches!(b.kind, E::InferredMember(_)) {
            rhs = self.check_expr(f, scope, b, Some(lhs.ty()))?;
        }
        Ok((lhs, rhs))
    }

    /// Two procedure names compare as their addresses, and a procedure name compared with a
    /// procedure value takes the value's type.
    fn procedure_operands(
        &mut self,
        f: &mut FnCtx,
        lhs: Operand,
        rhs: Operand,
        span: Span,
    ) -> Result<(Operand, Operand)> {
        match (&lhs, &rhs) {
            (Operand::Procs(l), Operand::Procs(r)) if l.len() == 1 && r.len() == 1 => {
                let (lt, rt) = (self.proc_type(l[0], span)?, self.proc_type(r[0], span)?);
                Ok((
                    self.value_as(f, lhs, lt, span)?,
                    self.value_as(f, rhs, rt, span)?,
                ))
            }
            (Operand::Procs(_), Operand::Procs(_)) => Ok((lhs, rhs)),
            (_, Operand::Procs(_)) => {
                let ty = lhs.ty();
                Ok((lhs, self.value_as(f, rhs, ty, span)?))
            }
            (Operand::Procs(_), _) => {
                let ty = rhs.ty();
                Ok((self.value_as(f, lhs, ty, span)?, rhs))
            }
            _ => Ok((lhs, rhs)),
        }
    }

    /// `op` converted to `ty` and loaded as a value.
    fn value_as(&mut self, f: &mut FnCtx, op: Operand, ty: TypeId, span: Span) -> Result<Operand> {
        let converted = self.convert(f, op, ty, span)?;
        let (ty, val) = self.rvalue(f, converted, span)?;
        Ok(Operand::Value {
            ty,
            val,
        })
    }

    /// Arrays (`[]T`, `[N]T`, `[..]T`) have no `==`; strings do.
    fn reject_array_comparison(
        &self,
        op: BinOp,
        lty: TypeId,
        rty: TypeId,
        span: Span,
    ) -> Result<()> {
        for t in [lty, rty] {
            if matches!(self.types.kind(self.types.repr(t)), TypeKind::Array { .. }) {
                let symbol = binop_text(op);
                return Err(Box::new(
                    Diagnostic::error(span, format!("arrays cannot be compared with `{symbol}`"))
                        .with_note(
                            span,
                            format!("the compared type is `{}`", self.types.name(t)),
                        )
                        .with_help("compare the lengths and elements in a loop"),
                ));
            }
        }
        Ok(())
    }

    /// `p + n`, `p - n` and `p - q` on a pointer `p`.
    fn pointer_offset(
        &mut self,
        f: &mut FnCtx,
        op: BinOp,
        lhs: Operand,
        rhs: Operand,
        span: Span,
    ) -> Result<Operand> {
        let (lty, rty) = (lhs.ty(), rhs.ty());
        let (_, base) = self.rvalue(f, lhs, span)?;
        if op == BinOp::Sub && self.types.is_pointer(rty) {
            let (_, other) = self.rvalue(f, rhs, span)?;
            let pointee = self.types.pointee(lty).unwrap();
            let size = if pointee == TypeId::VOID {
                1
            } else {
                self.size_of(pointee, span)?.max(1)
            };
            let x = f.b.conv(ir::ConvOp::Bitcast, Ty::Ptr, Ty::I64, base);
            let y = f.b.conv(ir::ConvOp::Bitcast, Ty::Ptr, Ty::I64, other);
            let diff = f.b.bin(ir::BinOp::Sub, Ty::I64, x, y);
            let s = f.b.iconst(Ty::I64, size);
            return Ok(Operand::Value {
                ty: TypeId::S64,
                val: f.b.bin(ir::BinOp::SDiv, Ty::I64, diff, s),
            });
        }
        // Any runtime integer offsets a pointer (`data + total_read` with a u64).
        let rhs = if self.types.is_integer(rty) && !matches!(rhs, Operand::Const { .. }) {
            self.explicit_cast(f, rhs, TypeId::S64, ast::CastFlags::default(), span)?
        } else {
            self.convert(f, rhs, TypeId::S64, span)?
        };
        let (_, idx) = self.rvalue(f, rhs, span)?;
        let pointee = self.types.pointee(lty).unwrap();
        let size = if pointee == TypeId::VOID {
            1
        } else {
            self.size_of(pointee, span)?
        };
        let s = f.b.iconst(Ty::I64, size);
        let mut off = f.b.bin(ir::BinOp::Mul, Ty::I64, idx, s);
        if op == BinOp::Sub {
            off = f.b.un(ir::UnOp::Neg, Ty::I64, off);
        }
        Ok(Operand::Value {
            ty: lty,
            val: f.b.ptr_add(base, off),
        })
    }

    /// Masking an address (`p & (alignment - 1)`; Jai reads `cast(u64) p & MASK` as
    /// `cast(u64) (p & MASK)`): the integer meets the pointer's bits, and the result keeps the
    /// pointer's type.
    fn mask_pointer(
        &mut self,
        f: &mut FnCtx,
        op: BinOp,
        lhs: Operand,
        rhs: Operand,
        span: Span,
    ) -> Result<Operand> {
        let (ptr, int) = if self.types.is_pointer(lhs.ty()) {
            (lhs, rhs)
        } else {
            (rhs, lhs)
        };
        let ptr_ty = ptr.ty();
        let (_, address) = self.rvalue(f, ptr, span)?;
        let bits = ast::CastFlags {
            no_check: true,
            ..Default::default()
        };
        let int = self.explicit_cast(f, int, TypeId::U64, bits, span)?;
        let (_, mask) = self.rvalue(f, int, span)?;
        let address = f.b.conv(ir::ConvOp::Bitcast, Ty::Ptr, Ty::I64, address);
        let ir_op = match op {
            BinOp::BitAnd => ir::BinOp::And,
            BinOp::BitOr => ir::BinOp::Or,
            _ => ir::BinOp::Xor,
        };
        let combined = f.b.bin(ir_op, Ty::I64, address, mask);
        Ok(Operand::Value {
            ty: ptr_ty,
            val: f.b.conv(ir::ConvOp::Bitcast, Ty::I64, Ty::Ptr, combined),
        })
    }

    /// Convert both operands to the operation's type and load them. Returns that type, the
    /// right operand's type (a shift amount keeps its own) and the two values.
    #[allow(clippy::too_many_arguments)]
    fn unify_binary_operands(
        &mut self,
        f: &mut FnCtx,
        op: BinOp,
        (a, b): (&ast::Expr, &ast::Expr),
        mut lhs: Operand,
        mut rhs: Operand,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<(TypeId, TypeId, ir::Val, ir::Val)> {
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
            ) && s.types.is_integer(o.ty())
                && !matches!(s.types.kind(o.ty()), TypeKind::Enum(_))
        };
        if !is_shift && is_float_lit(&rhs) && is_int_value(self, &lhs) {
            let ty = self.default_untyped(rhs.ty(), &rhs.const_value().unwrap());
            lhs = self.explicit_cast(f, lhs, ty, ast::CastFlags::default(), span)?;
        } else if !is_shift && is_float_lit(&lhs) && is_int_value(self, &rhs) {
            let ty = self.default_untyped(lhs.ty(), &lhs.const_value().unwrap());
            rhs = self.explicit_cast(f, rhs, ty, ast::CastFlags::default(), span)?;
        }
        // Unify operand types.
        let patterns = (self.bit_pattern(a.span), self.bit_pattern(b.span));
        let ty = self.binary_operand_type(&lhs, &rhs, expected, is_shift, patterns, span)?;
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
                    && self.types.int_info(other.ty()) == Some((64, false))
                    && !matches!(self.types.kind(other.ty()), TypeKind::Enum(_))
                {
                    return err(
                        span,
                        format!(
                            "signedness mismatch: the constant {v} does not fit in `{}`",
                            self.types.name(other.ty())
                        ),
                    );
                }
            }
        }
        if self.types.repr(ty) == TypeId::BOOL
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
            lhs = self.literal_as_bits(lhs, ty);
            rhs = self.literal_as_bits(rhs, ty);
        }
        let rhs_ty = if is_shift {
            self.shift_amount_type(&rhs, ty)
        } else {
            ty
        };
        // An integer value meeting a float converts like `cast(float) x`.
        let int_to_float =
            |s: &Self, o: &Operand| !is_shift && s.types.is_float(ty) && s.types.is_integer(o.ty());
        if int_to_float(self, &lhs) {
            lhs = self.explicit_cast(f, lhs, ty, ast::CastFlags::default(), span)?;
        }
        if int_to_float(self, &rhs) {
            rhs = self.explicit_cast(f, rhs, ty, ast::CastFlags::default(), span)?;
        }
        if ty == TypeId::VOID_PTR && matches!(self.types.kind(lhs.ty()), TypeKind::Proc(_)) {
            lhs = self.explicit_cast(f, lhs, ty, ast::CastFlags::default(), span)?;
        }
        if ty == TypeId::VOID_PTR && matches!(self.types.kind(rhs.ty()), TypeKind::Proc(_)) {
            rhs = self.explicit_cast(f, rhs, ty, ast::CastFlags::default(), span)?;
        }
        let lhs = self.convert(f, lhs, ty, span)?;
        let rhs = self.convert(f, rhs, rhs_ty, span)?;
        let (_, x) = self.rvalue(f, lhs, span)?;
        let (_, y) = self.rvalue(f, rhs, span)?;
        Ok((ty, rhs_ty, x, y))
    }

    /// Emit `x op y` on values of type `ty` (`rhs_ty` for a shift amount).
    #[allow(clippy::too_many_arguments)]
    fn emit_binary(
        &mut self,
        f: &mut FnCtx,
        op: BinOp,
        ty: TypeId,
        rhs_ty: TypeId,
        x: ir::Val,
        y: ir::Val,
        span: Span,
    ) -> Result<Operand> {
        let (is_cmp, is_shift) = (is_comparison(op), is_shift(op));
        if ty == TypeId::STRING && is_cmp {
            return self.string_compare(f, op, x, y, span);
        }
        if self.wide_float(ty).is_some() && !is_shift {
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
                    "operator {:?} is not defined for {}",
                    op,
                    self.types.name(ty)
                ),
            );
        };
        let signed = self.types.int_info(ty).is_some_and(|(_, s)| s);
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
                    "operator {:?} is not defined for {}",
                    op,
                    self.types.name(ty)
                ),
            ))
        })?;
        if t == Ty::Ptr {
            return err(
                span,
                format!(
                    "arithmetic operator {:?} is not defined for {}",
                    op,
                    self.types.name(ty)
                ),
            );
        }
        let y = if is_shift {
            let rt = self.ir_ty(rhs_ty).unwrap();
            if rt.size() != t.size() {
                let conv = if rt.size() > t.size() {
                    ir::ConvOp::Trunc
                } else {
                    ir::ConvOp::ZExt
                };
                f.b.conv(conv, rt, t, y)
            } else {
                y
            }
        } else {
            y
        };
        let val = f.b.bin(ir_op, t, x, y);
        if self.options.arithmetic_overflow_check != 0
            && !f.no_aoc
            && !f.type_only
            && matches!(ir_op, ir::BinOp::Add | ir::BinOp::Sub | ir::BinOp::Mul)
            && self.types.is_integer(ty)
            && !matches!(self.types.kind(ty), TypeKind::Enum(_))
        {
            self.emit_overflow_check(f, ir_op, signed, t, x, y, span)?;
        }
        Ok(Operand::Value {
            ty,
            val,
        })
    }

    /// After `x op y` on integers of IR type `t`: call the runtime's `__arithmetic_overflow`
    /// when the result did not fit (`Build_Options.arithmetic_overflow_check`).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_overflow_check(
        &mut self,
        f: &mut FnCtx,
        op: ir::BinOp,
        signed: bool,
        t: Ty,
        x: ir::Val,
        y: ir::Val,
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
        let Some(runtime) = self.runtime_support else {
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
        let super::procs::ProcTarget::Func(func) = self.proc_func(proc, span)? else {
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
        let file = self.sources.get(span.file);
        let (line, _) = file.line_col(span.start);
        let path: Rc<[u8]> = Rc::from(file.path.to_string().as_bytes());
        let line = f.b.iconst(Ty::I64, line as u64);
        let global = self.string_global(&path);
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

    /// The type an untyped literal takes next to a typed operand: the operand's
    /// type, or a 64-bit integer when the literal does not fit (`small + -1` and
    /// `small == -1` with a `u8` work in `s64`). A hex, binary or `~` literal
    /// (`bit_pattern`) may still fill the operand's bits.
    pub(super) fn literal_meets(&self, lit: &Operand, ty: TypeId, bit_pattern: bool) -> TypeId {
        if let Operand::Const {
            value: Value::Int(v),
            ..
        } = lit
            && let Some((bits, signed)) = self.types.int_info(ty)
            && !super::convert::int_constant_fits(*v, bits, signed, bit_pattern)
            && !(bit_pattern && super::convert::int_fits(*v, bits, signed))
            && !matches!(self.types.kind(ty), TypeKind::Enum(_))
        {
            return if super::convert::int_fits(*v, 64, true) {
                TypeId::S64
            } else {
                TypeId::U64
            };
        }
        ty
    }

    /// An untyped integer literal operand of integer type `ty`, wrapped to `ty`'s bits when
    /// it only fits as a bit pattern.
    pub(super) fn literal_as_bits(&self, op: Operand, ty: TypeId) -> Operand {
        match op {
            Operand::Const {
                value: Value::Int(v),
                untyped: true,
                ..
            } if !matches!(self.types.kind(ty), TypeKind::Enum(_))
                && let Some((bits, signed)) = self.types.int_info(ty)
                && bits <= 64
                && super::convert::int_fits(v, bits, signed) =>
            {
                Operand::Const {
                    ty,
                    value: Value::Int(wrap_int(v, bits, signed)),
                    untyped: false,
                }
            }
            op => op,
        }
    }

    pub(super) fn shift_amount_type(&self, rhs: &Operand, lhs_ty: TypeId) -> TypeId {
        match rhs {
            Operand::Const {
                untyped: true, ..
            } => lhs_ty,
            other => {
                let t = other.ty();
                if self.types.is_integer(t) {
                    t
                } else {
                    lhs_ty
                }
            }
        }
    }

    pub(super) fn binary_operand_type(
        &mut self,
        lhs: &Operand,
        rhs: &Operand,
        expected: Option<TypeId>,
        is_shift: bool,
        patterns: (bool, bool),
        span: Span,
    ) -> Result<TypeId> {
        let lu = matches!(
            lhs,
            Operand::Const {
                untyped: true,
                ..
            }
        );
        let ru = matches!(
            rhs,
            Operand::Const {
                untyped: true,
                ..
            }
        );
        let (lt, rt) = (lhs.ty(), rhs.ty());
        if is_shift {
            if lu {
                return Ok(expected
                    .filter(|&e| self.types.is_integer(e))
                    .unwrap_or(TypeId::S64));
            }
            return Ok(lt);
        }
        match (lu, ru) {
            (true, true) => {
                let lf = matches!(
                    lhs,
                    Operand::Const {
                        value: Value::Float(_, _),
                        ..
                    }
                );
                let rf = matches!(
                    rhs,
                    Operand::Const {
                        value: Value::Float(_, _),
                        ..
                    }
                );
                if lt == TypeId::NULL || rt == TypeId::NULL {
                    return Ok(TypeId::VOID_PTR);
                }
                Ok(if lf || rf {
                    let natural = if [lhs, rhs].iter().any(|o| {
                        o.const_value()
                            .is_some_and(|v| self.default_untyped(o.ty(), &v) == TypeId::F64)
                    }) {
                        TypeId::F64
                    } else {
                        TypeId::F32
                    };
                    expected
                        .filter(|&e| self.types.is_float(e))
                        .unwrap_or(natural)
                } else {
                    expected
                        .filter(|&e| self.types.is_integer(e) || self.types.is_float(e))
                        .unwrap_or(TypeId::S64)
                })
            }
            (true, false) => Ok(self.literal_meets(lhs, rt, patterns.0)),
            (false, true) => Ok(self.literal_meets(rhs, lt, patterns.1)),
            (false, false) => {
                if lt == rt {
                    return Ok(lt);
                }
                // Widening within the same class.
                if self.implicit_cost(rt, false, lt).is_some() {
                    return Ok(lt);
                }
                if self.implicit_cost(lt, false, rt).is_some() {
                    return Ok(rt);
                }
                // An integer meeting a float in arithmetic becomes that float.
                if self.types.is_float(lt) && self.types.is_integer(rt) {
                    return Ok(lt);
                }
                if self.types.is_integer(lt) && self.types.is_float(rt) {
                    return Ok(rt);
                }
                let (li, ri) = (self.types.int_info(lt), self.types.int_info(rt));
                if let (Some((lb, _)), Some((rb, _))) = (li, ri) {
                    // Mixed signedness: Jai requires a cast, but comparisons with
                    // literals are common; pick the wider type.
                    return Ok(if lb >= rb {
                        lt
                    } else {
                        rt
                    });
                }
                // A procedure value meets a `*void` as an address (`proc == get_caller_address()`).
                let is_proc = |c: &Self, t| matches!(c.types.kind(t), TypeKind::Proc(_));
                if (is_proc(self, lt) && rt == TypeId::VOID_PTR)
                    || (is_proc(self, rt) && lt == TypeId::VOID_PTR)
                {
                    return Ok(TypeId::VOID_PTR);
                }
                let mut d = Diagnostic::error(
                    span,
                    format!(
                        "type mismatch: `{}` and `{}` cannot be combined",
                        self.types.name(lt),
                        self.types.name(rt)
                    ),
                );
                // `c * "2"`: a one-character string where a character code was meant (`+` and `-`
                // take it as the byte).
                let one_char = |o: &Operand| match o {
                    Operand::Const {
                        value: Value::String(s),
                        ..
                    } if s.len() == 1 => Some(s[0] as char),
                    _ => None,
                };
                let (int_side, text) = match (one_char(lhs), one_char(rhs)) {
                    (_, Some(c)) => (lt, Some(c)),
                    (Some(c), _) => (rt, Some(c)),
                    _ => (lt, None),
                };
                if let Some(c) = text
                    && self.types.is_integer(int_side)
                {
                    d = d.with_help(format!(
                        "`\"{c}\"` is a string; for the character's code write `#char \"{c}\"`"
                    ));
                } else if self.types.is_integer(lt) && self.types.is_float(rt)
                    || self.types.is_float(lt) && self.types.is_integer(rt)
                {
                    d = d.with_help("convert one side with `cast(float)` or `cast(int)` first");
                } else if self.types.is_integer(lt) && self.types.is_integer(rt) {
                    d = d.with_help(
                        "integers of different sizes or signedness need a `cast` to one type",
                    );
                }
                Err(Box::new(d))
            }
        }
    }

    pub(super) fn fold_binary(
        &mut self,
        op: BinOp,
        lhs: &Operand,
        rhs: &Operand,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Option<Operand>> {
        let (
            Operand::Const {
                ty: lt,
                value: lv,
                untyped: lu,
            },
            Operand::Const {
                ty: rt,
                value: rv,
                untyped: ru,
            },
        ) = (lhs, rhs)
        else {
            return Ok(None);
        };
        if self.wide_float(*lt).is_some() || self.wide_float(*rt).is_some() {
            return Ok(self.wide_fold(op, (*lt, lv), (*rt, rv)));
        }
        let untyped = *lu && *ru;
        let is_shift = matches!(op, BinOp::Shl | BinOp::Shr | BinOp::Rotl | BinOp::Rotr);
        let plain_int = |c: &Self, t: TypeId| {
            c.types.int_info(t).is_some() && !matches!(c.types.kind(t), TypeKind::Enum(_))
        };
        let ty = if *lu && !*ru && !is_shift {
            *rt
        } else if !*lu
            && !*ru
            && !is_shift
            && lt != rt
            && plain_int(self, *lt)
            && plain_int(self, *rt)
        {
            // Two typed integer constants meet in the wider type, as variables do
            // (`cast(s16) 1 + cast(s32) 2` is `s32`).
            if self.implicit_cost(*rt, false, *lt).is_some() {
                *lt
            } else if self.implicit_cost(*lt, false, *rt).is_some() {
                *rt
            } else if self.types.int_info(*lt).unwrap().0 >= self.types.int_info(*rt).unwrap().0 {
                *lt
            } else {
                *rt
            }
        } else {
            *lt
        };
        let cmp = |o: std::cmp::Ordering| -> Option<Value> {
            use std::cmp::Ordering::*;
            Some(Value::Bool(match op {
                BinOp::Eq => o == Equal,
                BinOp::Ne => o != Equal,
                BinOp::Lt => o == Less,
                BinOp::Le => o != Greater,
                BinOp::Gt => o == Greater,
                BinOp::Ge => o != Less,
                _ => return None,
            }))
        };
        let result = match (lv, rv) {
            (Value::Int(a), Value::Int(b)) if !self.types.is_float(ty) => {
                let (a, b) = (*a, *b);
                let v = match op {
                    BinOp::Add => Some(Value::Int(a.wrapping_add(b))),
                    BinOp::Sub => Some(Value::Int(a.wrapping_sub(b))),
                    BinOp::Mul => Some(Value::Int(a.wrapping_mul(b))),
                    BinOp::Div => {
                        if b == 0 {
                            return err(span, "division by zero in constant expression");
                        }
                        Some(Value::Int(a.wrapping_div(b)))
                    }
                    BinOp::Rem => {
                        if b == 0 {
                            return err(span, "division by zero in constant expression");
                        }
                        Some(Value::Int(a.wrapping_rem(b)))
                    }
                    BinOp::BitAnd => Some(Value::Int(a & b)),
                    BinOp::BitOr => Some(Value::Int(a | b)),
                    BinOp::BitXor => Some(Value::Int(a ^ b)),
                    BinOp::Shl | BinOp::Shr if b < 0 => {
                        return err(
                            span,
                            format!("negative shift amount {b} in constant expression"),
                        );
                    }
                    BinOp::Shl => Some(Value::Int(if b >= 128 {
                        0
                    } else {
                        a << b
                    })),
                    BinOp::Shr => Some(Value::Int(if b >= 128 {
                        0
                    } else {
                        a >> b
                    })),
                    BinOp::Rotl | BinOp::Rotr => None,
                    _ => cmp(a.cmp(&b)),
                };
                // Wrap typed results to their width.
                match v {
                    Some(Value::Int(x)) if !untyped => {
                        let (bits, signed) = self.types.int_info(ty).unwrap_or((64, true));
                        Some(Value::Int(wrap_int(x, bits, signed)))
                    }
                    Some(Value::Int(x)) => {
                        if let Some((bits, signed)) = expected.and_then(|e| self.types.int_info(e))
                        {
                            let _ = (bits, signed);
                        }
                        Some(Value::Int(x))
                    }
                    other => other,
                }
            }
            (Value::Int(_) | Value::Float(_, _), Value::Int(_) | Value::Float(_, _)) => {
                let mut precision = crate::lexer::FloatLiteralPrecision::default();
                for value in [lv, rv] {
                    if let Value::Float(_, p) = value {
                        precision.defaults_to_float64 |= p.defaults_to_float64;
                        precision.requires_float64 |= p.requires_float64;
                        precision.requires_float64_due_to_significant_digits |=
                            p.requires_float64_due_to_significant_digits;
                    }
                }
                if self.types.repr(ty) == TypeId::F32 && !untyped && precision.requires_float64 {
                    return err(
                        span,
                        "this floating literal requires float64 precision (use an explicit cast to round it)",
                    );
                }
                let a = num(lv);
                let b = num(rv);
                let float = |value| {
                    let mut p = precision;
                    p.requires_float64 |= crate::lexer::has_big_float32_exponent(value);
                    Value::Float(value, p)
                };
                match op {
                    BinOp::Add => Some(float(a + b)),
                    BinOp::Sub => Some(float(a - b)),
                    BinOp::Mul => Some(float(a * b)),
                    BinOp::Div => {
                        if b == 0.0 {
                            return err(span, "division by zero in constant expression");
                        }
                        Some(float(a / b))
                    }
                    _ => a.partial_cmp(&b).and_then(cmp),
                }
            }
            (Value::Bool(a), Value::Bool(b)) => match op {
                BinOp::Eq => Some(Value::Bool(a == b)),
                BinOp::Ne => Some(Value::Bool(a != b)),
                BinOp::BitAnd => Some(Value::Bool(*a & *b)),
                BinOp::BitOr => Some(Value::Bool(*a | *b)),
                BinOp::BitXor => Some(Value::Bool(*a ^ *b)),
                _ => None,
            },
            (Value::String(a), Value::String(b)) => match op {
                BinOp::Eq => Some(Value::Bool(a == b)),
                BinOp::Ne => Some(Value::Bool(a != b)),
                _ => None,
            },
            (Value::Null, Value::Null) => cmp(std::cmp::Ordering::Equal),
            (Value::Proc(a), Value::Proc(b)) => match op {
                BinOp::Eq => Some(Value::Bool(a == b)),
                BinOp::Ne => Some(Value::Bool(a != b)),
                _ => None,
            },
            _ => None,
        };
        let Some(value) = result else {
            return Ok(None);
        };
        let is_bool = matches!(value, Value::Bool(_));
        let rty = if is_bool
            && !matches!((lv, rv), (Value::Bool(_), Value::Bool(_)) if !matches!(op, BinOp::Eq | BinOp::Ne))
        {
            TypeId::BOOL
        } else if matches!(value, Value::Float(_, _)) && !self.types.is_float(ty) {
            if self.types.is_float(*rt) {
                *rt
            } else {
                TypeId::F32
            }
        } else {
            ty
        };
        Ok(Some(Operand::Const {
            ty: rty,
            value,
            untyped: untyped && !is_bool,
        }))
    }

    pub(super) fn check_logical(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        op: BinOp,
        a: &ast::Expr,
        b: &ast::Expr,
        span: Span,
    ) -> Result<Operand> {
        let lhs = self.check_expr(f, scope, a, Some(TypeId::BOOL))?;
        if let Operand::Const {
            value, ..
        } = &lhs
        {
            let truth = match value {
                Value::Bool(v) => Some(*v),
                Value::Int(v) => Some(*v != 0),
                Value::Null => Some(false),
                _ => None,
            };
            if let Some(t) = truth {
                // Short-circuit at compile time.
                if (op == BinOp::And && !t) || (op == BinOp::Or && t) {
                    return Ok(Operand::bool(t));
                }
                let rhs = self.check_expr(f, scope, b, Some(TypeId::BOOL))?;
                if let Operand::Const {
                    value, ..
                } = &rhs
                {
                    let r = match value {
                        Value::Bool(v) => *v,
                        Value::Int(v) => *v != 0,
                        Value::Null => false,
                        _ => true,
                    };
                    return Ok(Operand::bool(r));
                }
                let rhs = self.settle_untyped(rhs, None);
                let (ty, v) = self.rvalue(f, rhs, span)?;
                let v = self.truthy(f, ty, v, span)?;
                return Ok(Operand::Value {
                    ty: TypeId::BOOL,
                    val: v,
                });
            }
        }
        let lhs = self.settle_untyped(lhs, None);
        let (lty, lv) = self.rvalue(f, lhs, span)?;
        let lc = self.truthy(f, lty, lv, span)?;
        let result = f.b.alloca(1, 1);
        f.b.store(Ty::I8, result, lc);
        let rhs_block = f.b.new_block();
        let done = f.b.new_block();
        if op == BinOp::And {
            f.b.branch(lc, rhs_block, done);
        } else {
            f.b.branch(lc, done, rhs_block);
        }
        f.b.switch_to(rhs_block);
        let rhs = self.check_expr(f, scope, b, Some(TypeId::BOOL))?;
        let rhs = self.settle_untyped(rhs, None);
        let (rty, rv) = self.rvalue(f, rhs, span)?;
        let rc = self.truthy(f, rty, rv, span)?;
        f.b.store(Ty::I8, result, rc);
        f.b.jump(done);
        f.b.switch_to(done);
        Ok(Operand::Value {
            ty: TypeId::BOOL,
            val: f.b.load(Ty::I8, result),
        })
    }

    pub(super) fn string_compare(
        &mut self,
        f: &mut FnCtx,
        op: BinOp,
        x: ir::Val,
        y: ir::Val,
        span: Span,
    ) -> Result<Operand> {
        if !matches!(op, BinOp::Eq | BinOp::Ne) {
            return err(span, "strings can only be compared with == and !=");
        }
        let result = f.b.alloca(1, 1);
        let lc = f.b.load(Ty::I64, x);
        let rc = f.b.load(Ty::I64, y);
        let same_len = f.b.cmp(CmpOp::Eq, Ty::I64, lc, rc);
        f.b.store(Ty::I8, result, same_len);
        let cmp_block = f.b.new_block();
        let done = f.b.new_block();
        f.b.branch(same_len, cmp_block, done);
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
        let mut v = f.b.load(Ty::I8, result);
        if op == BinOp::Ne {
            let one = f.b.iconst(Ty::I8, 1);
            v = f.b.bin(ir::BinOp::Xor, Ty::I8, v, one);
        }
        Ok(Operand::Value {
            ty: TypeId::BOOL,
            val: v,
        })
    }
}

fn num(v: &Value) -> f64 {
    match v {
        Value::Int(i) => *i as f64,
        Value::Float(f, _) => *f,
        _ => 0.0,
    }
}

fn cmp_op(op: BinOp, float: bool, signed: bool) -> CmpOp {
    match (op, float, signed) {
        (BinOp::Eq, true, _) => CmpOp::FEq,
        (BinOp::Ne, true, _) => CmpOp::FNe,
        (BinOp::Lt, true, _) => CmpOp::FLt,
        (BinOp::Le, true, _) => CmpOp::FLe,
        (BinOp::Gt, true, _) => CmpOp::FGt,
        (BinOp::Ge, true, _) => CmpOp::FGe,
        (BinOp::Eq, _, _) => CmpOp::Eq,
        (BinOp::Ne, _, _) => CmpOp::Ne,
        (BinOp::Lt, _, true) => CmpOp::SLt,
        (BinOp::Le, _, true) => CmpOp::SLe,
        (BinOp::Gt, _, true) => CmpOp::SGt,
        (BinOp::Ge, _, true) => CmpOp::SGe,
        (BinOp::Lt, _, false) => CmpOp::ULt,
        (BinOp::Le, _, false) => CmpOp::ULe,
        (BinOp::Gt, _, false) => CmpOp::UGt,
        _ => CmpOp::UGe,
    }
}

fn arith_op(op: BinOp, float: bool, signed: bool) -> Option<ir::BinOp> {
    use ir::BinOp as I;
    Some(match (op, float) {
        (BinOp::Add, true) => I::FAdd,
        (BinOp::Sub, true) => I::FSub,
        (BinOp::Mul, true) => I::FMul,
        (BinOp::Div, true) => I::FDiv,
        (_, true) => return None,
        (BinOp::Add, _) => I::Add,
        (BinOp::Sub, _) => I::Sub,
        (BinOp::Mul, _) => I::Mul,
        (BinOp::Div, _) => {
            if signed {
                I::SDiv
            } else {
                I::UDiv
            }
        }
        (BinOp::Rem, _) => {
            if signed {
                I::SRem
            } else {
                I::URem
            }
        }
        (BinOp::BitAnd, _) => I::And,
        (BinOp::BitOr, _) => I::Or,
        (BinOp::BitXor, _) => I::Xor,
        (BinOp::Shl, _) => I::Shl,
        (BinOp::Shr, _) => {
            if signed {
                I::AShr
            } else {
                I::LShr
            }
        }
        (BinOp::Rotl, _) => I::Rotl,
        (BinOp::Rotr, _) => I::Rotr,
        _ => return None,
    })
}

/// `~7`, `~0`: the complement of a constant. It is a 64-bit pattern whatever the other operand
/// is, so it does not adapt to the width of that operand.
fn is_complement_literal(e: &ast::Expr) -> bool {
    matches!(&e.kind, E::Unary(ast::UnOp::BitNot, x) if is_number_literal(x))
}

/// A numeric literal, possibly signed or parenthesised arithmetic of literals (`-1`, `(2 * 3)`):
/// an expression that is an untyped constant whatever the context.
fn is_number_literal(e: &ast::Expr) -> bool {
    match &e.kind {
        E::Int(_) | E::Float(_) => true,
        E::Unary(ast::UnOp::Neg | ast::UnOp::Plus | ast::UnOp::BitNot, x) => is_number_literal(x),
        E::Binary(_, a, b) => is_number_literal(a) && is_number_literal(b),
        _ => false,
    }
}

fn is_comparison(op: BinOp) -> bool {
    matches!(
        op,
        BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge
    )
}

fn is_shift(op: BinOp) -> bool {
    matches!(op, BinOp::Shl | BinOp::Shr | BinOp::Rotl | BinOp::Rotr)
}

fn is_bitwise(op: BinOp) -> bool {
    matches!(op, BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor)
}
