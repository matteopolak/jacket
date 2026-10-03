//! Integer overflow checks and guards against undefined LLVM arithmetic.
use super::*;
use jai_ir::CheckMode;

impl<'ctx> Generator<'ctx, '_, '_> {
    pub(super) fn negate_integer(
        &mut self,
        value: IntValue<'ctx>,
        domain: IntegerType,
        check: CheckMode,
    ) -> Result<IntValue<'ctx>, Error> {
        self.integer_operation(
            IntOp::Subtract,
            value.get_type().const_zero(),
            value,
            domain,
            check,
        )
    }

    pub(super) fn integer_operation(
        &mut self,
        operation: IntOp,
        left: IntValue<'ctx>,
        right: IntValue<'ctx>,
        domain: IntegerType,
        check: CheckMode,
    ) -> Result<IntValue<'ctx>, Error> {
        if check.enabled() && matches!(operation, IntOp::Add | IntOp::Subtract | IntOp::Multiply) {
            let width = std::num::NonZeroU32::new(domain.bits() * 2).ok_or(Error::Invariant)?;
            let wide = self
                .context
                .custom_width_int_type(width)
                .map_err(|_| Error::Invariant)?;
            let extend = |value, name| {
                if domain.signed() {
                    self.builder.build_int_s_extend(value, wide, name)
                } else {
                    self.builder.build_int_z_extend(value, wide, name)
                }
            };
            let wide_left = extend(left, "checked.left")?;
            let wide_right = extend(right, "checked.right")?;
            let exact = match operation {
                IntOp::Add => self
                    .builder
                    .build_int_add(wide_left, wide_right, "checked.add")?,
                IntOp::Subtract => {
                    self.builder
                        .build_int_sub(wide_left, wide_right, "checked.subtract")?
                }
                IntOp::Multiply => {
                    self.builder
                        .build_int_mul(wide_left, wide_right, "checked.multiply")?
                }
                _ => return Err(Error::Invariant),
            };
            let narrowed =
                self.builder
                    .build_int_truncate(exact, left.get_type(), "checked.result")?;
            let restored = extend(narrowed, "checked.restored")?;
            let valid = self.builder.build_int_compare(
                IntPredicate::EQ,
                exact,
                restored,
                "checked.fits",
            )?;
            self.check_cast(Bit(valid))?;
            return Ok(narrowed);
        }
        match operation {
            IntOp::Divide | IntOp::Remainder => {
                let nonzero = self.builder.build_int_compare(
                    IntPredicate::NE,
                    right,
                    right.get_type().const_zero(),
                    "division.nonzero",
                )?;
                self.check_cast(Bit(nonzero))?;
                if domain.signed() {
                    let minimum = left
                        .get_type()
                        .const_int(1u64 << (domain.bits() - 1), false);
                    let minus_one = right.get_type().const_all_ones();
                    let at_minimum = self.builder.build_int_compare(
                        IntPredicate::EQ,
                        left,
                        minimum,
                        "division.minimum",
                    )?;
                    let at_minus_one = self.builder.build_int_compare(
                        IntPredicate::EQ,
                        right,
                        minus_one,
                        "division.minus.one",
                    )?;
                    let overflow =
                        self.builder
                            .build_and(at_minimum, at_minus_one, "division.overflow")?;
                    if check.enabled() {
                        let valid = self.builder.build_not(overflow, "division.valid")?;
                        self.check_cast(Bit(valid))?;
                    } else {
                        let safe_divisor = self
                            .builder
                            .build_select(
                                overflow,
                                right.get_type().const_int(1, false),
                                right,
                                "division.safe.divisor",
                            )?
                            .into_int_value();
                        let (result, wrapped) = match operation {
                            IntOp::Divide => (
                                self.builder
                                    .build_int_signed_div(left, safe_divisor, "divide")?,
                                minimum,
                            ),
                            IntOp::Remainder => (
                                self.builder.build_int_signed_rem(
                                    left,
                                    safe_divisor,
                                    "remainder",
                                )?,
                                left.get_type().const_zero(),
                            ),
                            _ => return Err(Error::Invariant),
                        };
                        return Ok(self
                            .builder
                            .build_select(overflow, wrapped, result, "division.wrapped")?
                            .into_int_value());
                    }
                }
            }
            IntOp::ShiftLeft | IntOp::ShiftRight => {
                let width = right.get_type().const_int(u64::from(domain.bits()), false);
                let valid = self.builder.build_int_compare(
                    IntPredicate::ULT,
                    right,
                    width,
                    "shift.valid",
                )?;
                self.check_cast(Bit(valid))?;
            }
            _ => {}
        }
        Ok(match operation {
            IntOp::Add => self.builder.build_int_add(left, right, "add")?,
            IntOp::Subtract => self.builder.build_int_sub(left, right, "subtract")?,
            IntOp::Multiply => self.builder.build_int_mul(left, right, "multiply")?,
            IntOp::Divide if domain.signed() => {
                self.builder.build_int_signed_div(left, right, "divide")?
            }
            IntOp::Divide => self.builder.build_int_unsigned_div(left, right, "divide")?,
            IntOp::Remainder if domain.signed() => {
                self.builder
                    .build_int_signed_rem(left, right, "remainder")?
            }
            IntOp::Remainder => self
                .builder
                .build_int_unsigned_rem(left, right, "remainder")?,
            IntOp::BitAnd => self.builder.build_and(left, right, "and")?,
            IntOp::BitOr => self.builder.build_or(left, right, "or")?,
            IntOp::BitXor => self.builder.build_xor(left, right, "xor")?,
            IntOp::ShiftLeft => self.builder.build_left_shift(left, right, "shift.left")?,
            IntOp::ShiftRight => {
                self.builder
                    .build_right_shift(left, right, domain.signed(), "shift.right")?
            }
        })
    }
}
