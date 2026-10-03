//! Native address constants use LLVM's selected width, not host addresses.
use super::*;

pub(super) fn constant<'ctx>(
    lowerer: &mut types::TypeLowerer<'ctx, '_>,
    value: &jai_ir::NativePointerConstant,
) -> Result<BasicValueEnum<'ctx>, Error> {
    value.validate(lowerer.registry())?;
    let pointer = lowerer.basic(value.type_id())?.into_pointer_type();
    let target = lowerer.target_data().ok_or(Error::Invariant)?;
    let integer =
        value.address(target.get_pointer_byte_size(Some(pointer.get_address_space())) * 8)?;
    Ok(integer_type(lowerer.context(), integer.ty())
        .const_int(integer.bits(), false)
        .const_to_pointer(pointer)
        .into())
}
