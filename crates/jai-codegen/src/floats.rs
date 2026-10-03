//! IEEE instruction lowering and raw-bit constants, without fast-math flags.
use super::*;
use inkwell::{FloatPredicate, values::FloatValue as LlvmFloatValue};
use jai_ir::{FloatExpr, FloatExprKind};
use jai_types::{FloatOp, FloatType, FloatValue};

mod constants;
use constants::llvm_type;

pub(super) fn constant(context: &Context, value: FloatValue) -> Result<LlvmFloatValue<'_>, Error> {
    Ok(constants::constant(context, value)?)
}

impl<'ctx> Generator<'ctx, '_, '_> {
    pub(super) fn float(&mut self, expression: &FloatExpr) -> Result<LlvmFloatValue<'ctx>, Error> {
        let ty = llvm_type(self.context, expression.ty());
        let result = match expression.kind() {
            FloatExprKind::Constant(value) => constant(self.context, *value)?,
            FloatExprKind::Value(value) => self.value(value)?.into_float_value(),
            FloatExprKind::Load(place) => self.load(*place)?.into_float_value(),
            FloatExprKind::Call(call) => self
                .call(call)?
                .value
                .ok_or(Error::Invariant)?
                .into_float_value(),
            FloatExprKind::Negate(source) => {
                let source = self.float(source)?;
                self.builder.build_float_neg(source, "float.negate")?
            }
            FloatExprKind::Binary(op, left, right) => {
                let left = self.float(left)?;
                let right = self.float(right)?;
                match op {
                    FloatOp::Add => self.builder.build_float_add(left, right, "float.add")?,
                    FloatOp::Subtract => {
                        self.builder
                            .build_float_sub(left, right, "float.subtract")?
                    }
                    FloatOp::Multiply => {
                        self.builder
                            .build_float_mul(left, right, "float.multiply")?
                    }
                    FloatOp::Divide => self.builder.build_float_div(left, right, "float.divide")?,
                    FloatOp::Remainder => {
                        self.builder
                            .build_float_rem(left, right, "float.remainder")?
                    }
                }
            }
            FloatExprKind::Cast(source) => {
                let source = self.float(source)?;
                self.builder.build_float_cast(source, ty, "float.cast")?
            }
            FloatExprKind::FromInt(source) => {
                let value = self.int(source)?.0;
                if source.ty().signed() {
                    self.builder
                        .build_signed_int_to_float(value, ty, "float.from.int")?
                } else {
                    self.builder
                        .build_unsigned_int_to_float(value, ty, "float.from.int")?
                }
            }
            FloatExprKind::Conditional(conditional) => {
                if let Some(selected) = self.native_condition(&conditional.condition) {
                    self.boolean(&conditional.condition)?;
                    return self.float(if selected {
                        &conditional.then_value
                    } else {
                        &conditional.else_value
                    });
                }
                let [(yes, yes_end), (no, no_end)] =
                    self.conditional_values(conditional, Self::float)?;
                let phi = self.builder.build_phi(ty, "ifx.float")?;
                phi.add_incoming(&[(&yes, yes_end), (&no, no_end)]);
                phi.as_basic_value().into_float_value()
            }
        };
        if result.get_type() != ty {
            return Err(Error::Invariant);
        }
        Ok(result)
    }

    pub(super) fn compare_floats(
        &mut self,
        relation: Relation,
        left: &FloatExpr,
        right: &FloatExpr,
    ) -> Result<IntValue<'ctx>, Error> {
        let left = self.float(left)?;
        let right = self.float(right)?;
        let predicate = match relation {
            Relation::Equal => FloatPredicate::OEQ,
            Relation::NotEqual => FloatPredicate::UNE,
            Relation::Less => FloatPredicate::OLT,
            Relation::LessEqual => FloatPredicate::OLE,
            Relation::Greater => FloatPredicate::OGT,
            Relation::GreaterEqual => FloatPredicate::OGE,
        };
        Ok(self
            .builder
            .build_float_compare(predicate, left, right, "float.compare")?)
    }

    pub(super) fn float_to_integer(
        &mut self,
        source: &FloatExpr,
        target: IntegerType,
        mode: CastMode,
    ) -> Result<Number<'ctx>, Error> {
        if mode != CastMode::Checked {
            return Err(Error::Invariant);
        }
        let value = self.float(source)?;
        // Widen F32 before comparisons: integer boundaries such as -129 are
        // exactly represented in F64, avoiding a rounded-down validity interval.
        let wide = if source.ty() == FloatType::F32 {
            self.builder
                .build_float_ext(value, self.context.f64_type(), "cast.float.wide")?
        } else {
            value
        };
        // Range checks apply after truncation. Exact powers of two delimit the
        // upper end; for narrow widths, the open lower bound admits fractions
        // truncating to MIN (or zero for unsigned targets).
        let lower = if target.signed() {
            target.min() as f64 - 1.0
        } else {
            -1.0
        };
        let upper = 2.0_f64.powi((target.bits() - u32::from(target.signed())) as i32);
        let lower_predicate = if target.bits() == 64 && target.signed() {
            FloatPredicate::OGE
        } else {
            FloatPredicate::OGT
        };
        let lower = self.context.f64_type().const_float(lower);
        let upper = self.context.f64_type().const_float(upper);
        let valid_lower =
            self.builder
                .build_float_compare(lower_predicate, wide, lower, "cast.float.lower")?;
        let valid_upper = self.builder.build_float_compare(
            FloatPredicate::OLT,
            wide,
            upper,
            "cast.float.upper",
        )?;
        let valid = self
            .builder
            .build_and(valid_lower, valid_upper, "cast.float.valid")?;
        self.check_cast(Bit(valid))?;
        let signed = target.signed();
        let target = integer_type(self.context, target);
        // The trap branch dominates the conversion, so NaNs/overflow cannot
        // reach LLVM's poison-producing fptosi/fptoui instructions.
        let integer = if signed {
            self.builder
                .build_float_to_signed_int(value, target, "cast.float.int")?
        } else {
            self.builder
                .build_float_to_unsigned_int(value, target, "cast.float.int")?
        };
        Ok(Number(integer))
    }
}
