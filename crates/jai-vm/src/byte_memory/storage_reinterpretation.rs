//! Force decoding validates actual target values and retains their storage image.
use super::*;

impl ByteImage {
    pub(crate) fn read_storage_bitcast(
        &self,
        types: &dyn TypeView,
        target: ByteTarget,
        offset: usize,
        ty: TypeId,
        limit: usize,
    ) -> Result<Value, Error> {
        self.check_target(target)?;
        let mut layouts = LayoutEngine::new(types, target.policy);
        let mut remaining = limit;
        self.storage_cast_booleans(types, &mut layouts, offset, ty, 0, &mut remaining)?;
        let semantic = self
            .read(types, target, offset, ty)
            .map_err(|error| match error {
                Error::InvalidIr("byte storage cannot forge pointer provenance") => {
                    Error::UnsupportedPointerOperation(
                        "storage cast cannot forge pointer provenance",
                    )
                }
                Error::InvalidIr("byte storage cannot forge procedure provenance") => {
                    Error::UnsupportedPointerOperation(
                        "storage cast cannot forge procedure provenance",
                    )
                }
                error => error,
            })?;
        // Scalar decoded values already preserve address-derived integer receipts.
        // Aggregate carriers additionally retain padding, inactive bytes, and holes.
        let mut represented = ty;
        for _ in 0..MAX_DEPTH {
            match types.kind(represented)? {
                TypeKind::Distinct(id) => represented = types.distinct(*id)?.representation,
                TypeKind::Record(_)
                | TypeKind::Any(_)
                | TypeKind::FixedArray {
                    ..
                } => {
                    let length = size(layout(&mut layouts, ty)?.size, limit)?;
                    let range = self.range(offset, length)?;
                    let storage = length
                        .checked_add(self.range_metadata_cells(&range))
                        .ok_or(Error::Limit(LimitKind::ValueCells))?;
                    semantic
                        .cells(limit)?
                        .checked_add(storage)
                        .and_then(|n| n.checked_add(1))
                        .filter(|n| *n <= limit)
                        .ok_or(Error::Limit(LimitKind::ValueCells))?;
                    let mut image = self.extract_range(offset, length)?;
                    image.limit = limit;
                    return Ok(Value::StoredAggregate(StoredAggregate::new(
                        ty, semantic, image, limit,
                    )?));
                }
                _ => return Ok(semantic),
            }
        }
        Err(Error::Limit(LimitKind::EvaluationDepth))
    }

    fn storage_cast_booleans(
        &self,
        types: &dyn TypeView,
        layouts: &mut LayoutEngine<'_>,
        offset: usize,
        ty: TypeId,
        depth: usize,
        remaining: &mut usize,
    ) -> Result<(), Error> {
        depth_check(depth)?;
        *remaining = remaining
            .checked_sub(1)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let storage = layout(layouts, ty)?.clone();
        let length = size(storage.size, self.limit)?;
        self.range(offset, length)?;
        match types.kind(ty)? {
            TypeKind::Bool => {
                self.reject_address_view(
                    offset,
                    length,
                    "address bytes cannot be interpreted as booleans",
                )?;
                if self.bits(offset, length)? > 1 {
                    return Err(Error::UnsupportedPointerOperation(
                        "storage cast contains an invalid boolean representation",
                    ));
                }
            }
            TypeKind::Type => {
                jai_types::RuntimeTypeSchema::from_view(types)?;
            }
            TypeKind::Distinct(id) => self.storage_cast_booleans(
                types,
                layouts,
                offset,
                types.distinct(*id)?.representation,
                depth + 1,
                remaining,
            )?,
            kind if kind.record_storage_id().is_some() => {
                let record = types.record_storage_definition(ty)?;
                if record.kind == RecordKind::Union {
                    // Check only the selected semantic member. Other alias views
                    // remain byte storage and are checked when actually projected.
                    let field = self.union_field(offset, ty).unwrap_or(0);
                    let selected = *record.fields.get(field).ok_or(Error::UnsupportedType(ty))?;
                    self.storage_cast_booleans(
                        types,
                        layouts,
                        offset,
                        selected,
                        depth + 1,
                        remaining,
                    )?;
                } else {
                    if record.fields.len() > *remaining {
                        return Err(Error::Limit(LimitKind::ValueCells));
                    }
                    for (index, field) in record.fields.iter().enumerate() {
                        self.storage_cast_booleans(
                            types,
                            layouts,
                            at(offset, storage.field_offsets[index])?,
                            *field,
                            depth + 1,
                            remaining,
                        )?;
                    }
                }
            }
            TypeKind::FixedArray {
                element,
                count,
            } => {
                let count = size(*count, *remaining)?;
                let stride = storage
                    .array_stride
                    .ok_or(Error::InvalidIr("array layout missing stride"))?;
                for index in 0..count {
                    let delta = stride.checked_mul(index as u64).ok_or(Error::CheckedCast)?;
                    self.storage_cast_booleans(
                        types,
                        layouts,
                        at(offset, delta)?,
                        *element,
                        depth + 1,
                        remaining,
                    )?;
                }
            }
            TypeKind::Void | TypeKind::Code => return Err(Error::UnsupportedType(ty)),
            _ => {}
        }
        Ok(())
    }
}
