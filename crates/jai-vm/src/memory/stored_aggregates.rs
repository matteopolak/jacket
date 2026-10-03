//! Ordinary aggregate copies retain inactive bytes, holes and address provenance.
use super::*;
use crate::StoredAggregate;

impl Memory {
    pub(super) fn load_stored_aggregate(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
    ) -> Result<Value, Error> {
        let allocation = self.allocation(pointer)?;
        self.ensure_image(types, allocation)?;
        let image = allocation.image.borrow();
        let image = image
            .as_ref()
            .ok_or(Error::InvalidIr("byte image initialization failed"))?;
        let offset =
            usize::try_from(self.byte_offset(types, pointer)?).map_err(|_| Error::CheckedCast)?;
        if allocation.has_stored_aggregate {
            let mut representation = pointer.pointee;
            for _ in 0..self.limits.evaluation_depth.min(256) {
                match types.kind(representation)? {
                    TypeKind::Distinct(id) => representation = types.distinct(*id)?.representation,
                    TypeKind::Record(_)
                    | TypeKind::Any(_)
                    | TypeKind::FixedArray {
                        ..
                    } => {
                        let layout = self.layout(types, pointer.pointee)?;
                        let length = usize::try_from(layout.size)
                            .map_err(|_| Error::Limit(LimitKind::ValueCells))?;
                        // A genuine carrier copy preserves even fully initialized
                        // padding; semantic reconstruction would discard those bytes.
                        let image = image.extract_range(offset, length)?;
                        return Ok(Value::StoredAggregate(StoredAggregate::opaque(
                            types,
                            pointer.pointee,
                            image,
                            &layout,
                            self.limits.value_cells,
                        )?));
                    }
                    _ => break,
                }
            }
        }
        if crate::value::allocator_schema(types)?
            .is_some_and(|schema| schema.ty() == pointer.pointee)
        {
            return image.read_partial_preserving_with_limit(
                types,
                self.target,
                offset,
                pointer.pointee,
                self.limits.value_cells,
            );
        }
        image.read_preserving(types, self.target, offset, pointer.pointee)
    }

    pub(super) fn validate_stored_aggregate(
        &self,
        types: &dyn TypeView,
        snapshot: &StoredAggregate,
    ) -> Result<(), Error> {
        if snapshot.image().target() != self.target {
            return Err(Error::InvalidIr(
                "aggregate snapshot uses a different target layout",
            ));
        }
        snapshot.image().validate_memory_provenance(self.identity)?;
        snapshot
            .image()
            .validate_complete_handles(|value| match value {
                Value::Pointer(pointer) => self.validate_pointer(types, pointer),
                Value::Procedure {
                    signature, ..
                } => {
                    types.procedure_definition(*signature)?;
                    Ok(())
                }
                _ => Err(Error::InvalidIr(
                    "aggregate snapshot contains an invalid handle",
                )),
            })
    }

    pub(super) fn store_stored_aggregate(
        &mut self,
        types: &dyn TypeView,
        pointer: &Pointer,
        snapshot: &StoredAggregate,
    ) -> Result<(), Error> {
        self.validate_stored_aggregate(types, snapshot)?;
        let allocation = self.allocation(pointer)?;
        let mut image = if allocation.value.is_none() && allocation.image.borrow().is_none() {
            let length = usize::try_from(self.storage_length(types, pointer)?)
                .map_err(|_| Error::Limit(LimitKind::ValueCells))?;
            ByteImage::uninitialized(self.target, length, self.limits.value_cells)?
        } else {
            self.image_for(types, allocation)?
        };
        image.copy_range_from(
            snapshot.image(),
            0,
            usize::try_from(self.byte_offset(types, pointer)?).map_err(|_| Error::CheckedCast)?,
            snapshot.image().len(),
        )?;
        for (offset, ty, field) in self.union_projections(types, pointer)? {
            image.note_union_field(types, offset, ty, field)?;
        }
        self.retokenize_image(types, &mut image)?;
        self.install_intrinsic_image(pointer, image)?;
        self.allocations
            .get_mut(&pointer.allocation_id())
            .ok_or(Error::DanglingPointer)?
            .has_stored_aggregate = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests;

impl Memory {
    /// Complete typed handles acquire this domain's bits before another write
    /// can split their provenance or remove their complete relocation.
    pub(crate) fn write_ordered_record_patch(
        &self,
        types: &dyn TypeView,
        image: &mut ByteImage,
        offset: usize,
        ty: TypeId,
        value: &Value,
    ) -> Result<(), Error> {
        image.write_normalized(types, self.target, offset, ty, value, |patch| {
            patch.validate_memory_provenance(self.identity)?;
            self.retokenize_image(types, patch)
        })
    }

    /// Normalize handles in the actual owning domain before publishing the rvalue.
    pub(crate) fn finish_ordered_record(
        &self,
        types: &dyn TypeView,
        ty: TypeId,
        layout: std::sync::Arc<jai_types::Layout>,
        mut image: ByteImage,
    ) -> std::result::Result<Value, Error> {
        image.validate_memory_provenance(self.identity)?;
        self.retokenize_image(types, &mut image)?;
        Ok(Value::StoredAggregate(StoredAggregate::opaque(
            types,
            ty,
            image,
            &layout,
            self.limits.value_cells,
        )?))
    }
}
