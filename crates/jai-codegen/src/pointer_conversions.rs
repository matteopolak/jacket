//! Native address conversions obey the selected LLVM pointer width.
use super::*;

impl<'ctx> Generator<'ctx, '_, '_> {
    pub(super) fn numeric_pointer_mode(mode: CastMode) -> Result<(), Error> {
        match mode {
            CastMode::Checked | CastMode::Unchecked | CastMode::Truncate => Ok(()),
            CastMode::Force(_) => Err(Error::UnsupportedPointerCast(mode)),
        }
    }

    fn pointer_width(&self) -> Result<u32, Error> {
        let target = self.lowerer.target_data().ok_or(Error::Invariant)?;
        Ok(target.get_pointer_byte_size(None) * 8)
    }

    fn pointer_integer_type(&self, width: u32) -> Result<IntType<'ctx>, Error> {
        let width = std::num::NonZeroU32::new(width).ok_or(Error::Invariant)?;
        self.context
            .custom_width_int_type(width)
            .map_err(|_| Error::Invariant)
    }

    pub(super) fn pointer_to_integer(
        &mut self,
        value: &ValueExpr,
        ty: IntegerType,
        mode: CastMode,
    ) -> Result<IntValue<'ctx>, Error> {
        Self::numeric_pointer_mode(mode)?;
        if !matches!(
            self.types.kind(value.type_id(self.types))?,
            TypeKind::Pointer(_)
        ) {
            return Err(Error::Invariant);
        }
        let pointer = self.value(value)?.into_pointer_value();
        let pointer_width = self.pointer_width()?;
        let result = self.builder.build_ptr_to_int(
            pointer,
            integer_type(self.context, ty),
            "pointer.integer",
        )?;
        if mode == CastMode::Checked && ty.bits() < pointer_width {
            let address_type = self.pointer_integer_type(pointer_width)?;
            let address =
                self.builder
                    .build_ptr_to_int(pointer, address_type, "pointer.address")?;
            let restored = self.builder.build_int_z_extend(
                result,
                address_type,
                "pointer.integer.restored",
            )?;
            let fits = self.builder.build_int_compare(
                IntPredicate::EQ,
                address,
                restored,
                "pointer.integer.fits",
            )?;
            let fits = if ty.signed() {
                let nonnegative = self.builder.build_int_compare(
                    IntPredicate::SGE,
                    result,
                    result.get_type().const_zero(),
                    "pointer.integer.nonnegative",
                )?;
                self.builder
                    .build_and(fits, nonnegative, "pointer.integer.signed.fits")?
            } else {
                fits
            };
            self.check_cast(Bit(fits))?;
        }
        Ok(result)
    }

    pub(super) fn integer_to_pointer(
        &mut self,
        value: &IntExpr,
        ty: TypeId,
        mode: CastMode,
    ) -> Result<BasicValueEnum<'ctx>, Error> {
        Self::numeric_pointer_mode(mode)?;
        if !matches!(self.types.kind(ty)?, TypeKind::Pointer(_)) {
            return Err(Error::Invariant);
        }
        let pointer_width = self.pointer_width()?;
        let address_type = self.pointer_integer_type(pointer_width)?;
        let integer = self.int(value)?.0;
        let address = if value.ty().bits() < pointer_width {
            if value.ty().signed() {
                self.builder
                    .build_int_s_extend(integer, address_type, "pointer.integer.signed")?
            } else {
                self.builder.build_int_z_extend(
                    integer,
                    address_type,
                    "pointer.integer.unsigned",
                )?
            }
        } else if value.ty().bits() > pointer_width {
            if mode == CastMode::Checked {
                let maximum = integer
                    .get_type()
                    .const_int(u64::MAX >> (64 - pointer_width), false);
                let fits = self.builder.build_int_compare(
                    IntPredicate::ULE,
                    integer,
                    maximum,
                    "integer.pointer.fits",
                )?;
                self.check_cast(Bit(fits))?;
            }
            self.builder
                .build_int_truncate(integer, address_type, "pointer.integer.truncated")?
        } else {
            integer
        };
        Ok(self
            .builder
            .build_int_to_ptr(
                address,
                self.lowerer.basic(ty)?.into_pointer_type(),
                "integer.pointer",
            )?
            .into())
    }

    pub(super) fn offset_pointer_left(
        &mut self,
        offset: &IntExpr,
        value: &ValueExpr,
        ty: TypeId,
    ) -> Result<BasicValueEnum<'ctx>, Error> {
        let TypeKind::Pointer(element) = *self.types.kind(ty)? else {
            return Err(Error::Invariant);
        };
        if value.type_id(self.types) != ty || offset.ty() != IntegerType::S64 {
            return Err(Error::Invariant);
        }
        // Keep the written operand order, including side effects in both calls.
        let offset = self.int(offset)?.0;
        let pointer = self.value(value)?.into_pointer_value();
        Ok(jai_llvm::gep(
            &self.builder,
            self.lowerer.basic(element)?,
            pointer,
            &[offset],
            "integer.pointer.offset",
        )?
        .into())
    }
}
