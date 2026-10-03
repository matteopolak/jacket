//! Check target byte displacement before narrowing a canonical sequence index.
use super::*;

impl<'ctx> Generator<'ctx, '_, '_> {
    pub(super) fn index_displacement(
        &mut self,
        index: IntValue<'ctx>,
        domain: IntegerType,
        element: TypeId,
    ) -> Result<IntValue<'ctx>, Error> {
        let pointer_integer = self.context.ptr_sized_int_type(&self.target.data, None);
        let bits = pointer_integer.get_bit_width();
        if bits > 64 || index.get_type().get_bit_width() != 64 {
            return Err(Error::Invariant);
        }
        let layout = self.lowerer.semantic_layout(element)?;
        let alignment = u64::from(layout.alignment);
        let expected_stride = layout
            .size
            .checked_add(alignment - 1)
            .ok_or(Error::Invariant)?
            & !(alignment - 1);
        let stride = self.target.data.get_abi_size(&self.lowerer.basic(element)?);
        if stride != expected_stride {
            return Err(Error::Invariant);
        }
        if stride == 0 {
            // The original logical index was bounds-checked already. A ZST
            // always addresses the same backing byte on every target width.
            return Ok(pointer_integer.const_zero());
        }
        let pointer_max = u64::MAX >> (64 - bits);
        let unsigned_limit = pointer_max / stride;
        let valid = if domain.signed() {
            let negative = self.builder.build_int_compare(
                IntPredicate::SLT,
                index,
                index.get_type().const_zero(),
                "index.negative",
            )?;
            let magnitude = self
                .builder
                .build_int_neg(index, "index.negative.magnitude")?;
            let negative_limit = (1u64 << (bits - 1)) / stride;
            let valid_negative = self.builder.build_int_compare(
                IntPredicate::ULE,
                magnitude,
                index.get_type().const_int(negative_limit, false),
                "index.negative.displacement.fits",
            )?;
            let valid_positive = self.builder.build_int_compare(
                IntPredicate::ULE,
                index,
                index.get_type().const_int(unsigned_limit, false),
                "index.positive.displacement.fits",
            )?;
            self.builder
                .build_select(
                    negative,
                    valid_negative,
                    valid_positive,
                    "index.displacement.fits",
                )?
                .into_int_value()
        } else {
            self.builder.build_int_compare(
                IntPredicate::ULE,
                index,
                index.get_type().const_int(unsigned_limit, false),
                "index.unsigned.displacement.fits",
            )?
        };
        self.check_cast(Bit(valid))?;
        if bits == 64 {
            Ok(index)
        } else {
            Ok(self
                .builder
                .build_int_truncate(index, pointer_integer, "index.target.width")?)
        }
    }
}
