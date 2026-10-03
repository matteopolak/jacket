//! A descriptor projection must retain its allocator proof's nominal owner.
use super::*;

pub(super) fn allocator_type(types: &dyn TypeView, base: TypeId) -> Result<TypeId, IrError> {
    if !matches!(types.kind(base)?, TypeKind::DynamicArray(_)) {
        return Err(IrError::InvalidValue(base));
    }
    let schema = types
        .allocator_schema()
        .ok_or(IrError::InvalidValue(base))?;
    let checked = jai_types::AllocatorSchema::validate(types, schema.ty(), schema.mode_type())
        .map_err(|_| IrError::InvalidValue(base))?;
    if checked != schema {
        return Err(IrError::InvalidValue(base));
    }
    Ok(schema.ty())
}

#[cfg(test)]
mod tests;
