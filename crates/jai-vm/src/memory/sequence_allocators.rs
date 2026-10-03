//! Certified allocator subobjects retain descriptor storage and pointer regions.
use super::*;

impl Memory {
    /// The execution layer prepares both descriptor and allocator before this call.
    pub(crate) fn sequence_allocator(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
    ) -> Result<Pointer, Error> {
        if !matches!(types.kind(pointer.pointee)?, TypeKind::DynamicArray(_)) {
            return Err(Error::UnsupportedType(pointer.pointee));
        }
        let schema = crate::value::allocator_schema(types)?.ok_or(
            Error::UnsupportedPointerOperation("dynamic array has no certified allocator field"),
        )?;
        let pointer = self.cast_pointer(types, pointer, pointer.pointee, CastMode::Checked)?;
        self.validate_pointer(types, &pointer)?;
        let descriptor = self.layout(types, pointer.pointee)?;
        let allocator = self.layout(types, schema.ty())?;
        let target = self.target.policy.pointer();
        let tail = *descriptor.field_offsets.get(3).ok_or(Error::InvalidIr(
            "dynamic array layout lacks allocator storage",
        ))?;
        if allocator.size != target.size.checked_mul(2).ok_or(Error::CheckedCast)?
            || allocator.alignment != target.alignment
            || allocator.field_offsets.as_ref() != [0, target.size]
            || tail.checked_add(allocator.size) != Some(descriptor.size)
        {
            return Err(Error::InvalidIr(
                "certified allocator layout differs from descriptor storage",
            ));
        }
        let start = self
            .byte_offset(types, &pointer)?
            .checked_add(tail)
            .ok_or(Error::CheckedCast)?;
        let end = start
            .checked_add(allocator.size)
            .ok_or(Error::CheckedCast)?;
        let parent = self.region(types, &pointer)?;
        if start < parent.0 || end > parent.1 {
            return Err(Error::OutOfBounds {
                index: usize::try_from(end).unwrap_or(usize::MAX),
                length: usize::try_from(parent.1 - parent.0).unwrap_or(usize::MAX),
            });
        }
        let mut result = pointer;
        result.data_mut()?.path = vec![Projection::Bytes {
            offset: start,
            ty: schema.ty(),
        }];
        result.pointee = schema.ty();
        result.data_mut()?.region = Some((start, end));
        Ok(result)
    }
}

#[cfg(test)]
mod tests;
