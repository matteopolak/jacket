//! Explicit access alignment follows pointer provenance, not loaded type width.
use super::*;
use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValue;

pub(super) fn load<'ctx>(
    builder: &Builder<'ctx>,
    ty: BasicTypeEnum<'ctx>,
    pointer: PointerValue<'ctx>,
    name: &str,
    alignment: u32,
) -> Result<BasicValueEnum<'ctx>, Error> {
    let value = builder.build_load(ty, pointer, name)?;
    value
        .as_instruction_value()
        .ok_or(Error::Invariant)?
        .set_alignment(alignment)
        .map_err(|_| Error::Invariant)?;
    Ok(value)
}

pub(super) fn store(
    builder: &Builder<'_>,
    pointer: PointerValue<'_>,
    value: BasicValueEnum<'_>,
    alignment: u32,
) -> Result<(), Error> {
    builder
        .build_store(pointer, value)?
        .set_alignment(alignment)
        .map_err(|_| Error::Invariant)
}

pub(super) fn offset_alignment(base: u32, offset: u64) -> u32 {
    if offset == 0 {
        base
    } else {
        u64::from(base).min(1u64 << offset.trailing_zeros()) as u32
    }
}
