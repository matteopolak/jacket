//! The physical descriptor tail is the certified nominal allocator record.
use super::*;

impl ByteImage {
    pub(super) fn allocator_extent(
        &self,
        _types: &dyn TypeView,
        layouts: &mut LayoutEngine<'_>,
        schema: jai_types::AllocatorSchema,
        descriptor: &Layout,
    ) -> Result<usize, Error> {
        let allocator = layout(layouts, schema.ty())?;
        let pointer = self.target.policy.pointer();
        let tail = descriptor
            .field_offsets
            .get(3)
            .copied()
            .ok_or(Error::InvalidIr(
                "dynamic array layout lacks allocator storage",
            ))?;
        if allocator.size != pointer.size.checked_mul(2).ok_or(Error::CheckedCast)?
            || allocator.alignment != pointer.alignment
            || allocator.field_offsets.as_ref() != [0, pointer.size]
            || tail.checked_add(allocator.size) != Some(descriptor.size)
        {
            return Err(Error::InvalidIr(
                "certified allocator layout differs from descriptor storage",
            ));
        }
        size(allocator.size, self.limit)
    }

    pub(super) fn decode_allocator(
        &self,
        types: &dyn TypeView,
        layouts: &mut LayoutEngine<'_>,
        offset: usize,
        descriptor: &Layout,
        depth: usize,
        remaining: &mut usize,
    ) -> Result<Option<Box<Value>>, Error> {
        depth_check(depth)?;
        let start = at(offset, descriptor.field_offsets[3])?;
        if let Some(schema) = crate::value::allocator_schema(types)? {
            self.allocator_extent(types, layouts, schema, descriptor)?;
            // A descriptor copy can carry a partially initialized allocator.
            // Preserve its mask; field access still rejects each unreadable slot.
            let payload = if let Some(snapshot) =
                self.partial_snapshot(types, layouts, start, schema.ty(), *remaining, true)?
            {
                *remaining = remaining
                    .checked_sub(snapshot.cells(*remaining)?)
                    .ok_or(Error::Limit(LimitKind::ValueCells))?;
                snapshot
            } else {
                self.decode_at(types, layouts, start, schema.ty(), depth, remaining)?
            };
            return Ok(Some(Box::new(payload)));
        }
        let length = size(descriptor.size, self.limit)?
            .checked_sub(size(descriptor.field_offsets[3], self.limit)?)
            .ok_or(Error::InvalidIr(
                "allocator offset exceeds descriptor storage",
            ))?;
        self.reject_address_view(
            start,
            length,
            "address bytes cannot form allocator internals",
        )?;
        if self
            .read_range(start, length)?
            .iter()
            .any(|byte| *byte != 0)
        {
            return Err(Error::UnsupportedPointerOperation(
                "dynamic array allocator requires the certified allocator role",
            ));
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests;
