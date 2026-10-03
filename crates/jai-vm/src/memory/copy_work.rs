//! Preflight copying work without loading values or creating byte images.
use super::*;

impl Memory {
    /// Count zero-value nodes from type shape before allocating any aggregate.
    /// Fixed-array counts multiply memoized child shapes; union zero selects
    /// only the first field, matching `constants::zero`.
    pub(crate) fn zero_value_work_cost(
        &self,
        types: &dyn TypeView,
        ty: TypeId,
    ) -> Result<usize, Error> {
        Ok(zero_shape(types, ty, self.limits, 0, &mut HashMap::new())?.cells)
    }

    /// Bound the selected value clone and any first conversion of its backing.
    /// Cached scalar reads do not pay for unrelated allocation elements.
    pub(crate) fn load_work_cost(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
    ) -> Result<usize, Error> {
        self.validate_pointer(types, pointer)?;
        self.validate_access(types, pointer)?;
        let allocation = self.allocation(pointer)?;
        let path = pointer.metadata_cells();
        if pointer.data()?.path.is_empty()
            && let Some(Value::String(bytes)) = &allocation.value
        {
            return add_work(path, 1usize.saturating_add(bytes.len()));
        }
        let mut ty = allocation.ty;
        let mut union = false;
        for projection in &pointer.data()?.path {
            if let (Projection::Field(_), kind) = (projection, types.kind(ty)?)
                && kind.record_storage_id().is_some()
                && types.record_storage_definition(ty)?.kind == jai_types::RecordKind::Union
            {
                union = true;
            }
            ty = match projection {
                Projection::Bytes {
                    ty, ..
                } => *ty,
                Projection::Field(index) => *types
                    .record_storage_definition(ty)?
                    .fields
                    .get(*index)
                    .ok_or(Error::InvalidIr("invalid copy field projection"))?,
                Projection::Index(_) => match types.kind(ty)? {
                    TypeKind::FixedArray {
                        element, ..
                    } => *element,
                    TypeKind::String => types.scalar(ScalarType::Int(IntegerType::U8)),
                    _ => return Err(Error::InvalidIr("invalid copy index projection")),
                },
                Projection::Sequence(field) => sequence_field_type(types, ty, *field)?,
            };
        }
        let image = allocation.image.borrow();
        let decoded = image.is_some()
            || allocation.has_stored_aggregate
            || union
            || ty != pointer.pointee
            || pointer
                .data()?
                .path
                .iter()
                .any(|p| matches!(p, Projection::Bytes { .. }));
        if decoded {
            let bytes = usize::try_from(self.layout(types, pointer.pointee)?.size)
                .map_err(|_| Error::Limit(LimitKind::Fuel))?;
            let mut work = add_work(path, bytes)?;
            // A zero-stride aggregate can decode many values from no bytes.
            // Compute its node count from types rather than expanding it first.
            let cells = self.decoded_cells(types, pointer.pointee)?;
            work = add_work(work, cells)?;
            if let Some(image) = image.as_ref() {
                let offset = usize::try_from(self.byte_offset(types, pointer)?)
                    .map_err(|_| Error::Limit(LimitKind::Fuel))?;
                work = add_work(work, image.range_metadata_work(offset, bytes)?)?;
            } else {
                work = add_work(work, self.cold_image_work(allocation)?)?;
            }
            return Ok(work);
        }
        let mut value = allocation.value.as_ref().ok_or(Error::Uninitialized)?;
        for (ordinal, projection) in pointer.data()?.path.iter().enumerate() {
            match (projection, value) {
                (Projection::Sequence(field), _) if ordinal + 1 == pointer.data()?.path.len() => {
                    let metadata = match (field, value) {
                        (
                            jai_ir::SequenceField::Data,
                            Value::Slice {
                                pointer, ..
                            },
                        )
                        | (
                            jai_ir::SequenceField::Data,
                            Value::DynamicArray {
                                pointer, ..
                            },
                        )
                        | (
                            jai_ir::SequenceField::Data,
                            Value::StringView {
                                pointer, ..
                            },
                        ) => pointer.metadata_cells(),
                        _ => 0,
                    };
                    return add_work(path, add_work(1, metadata)?);
                }
                (Projection::Index(_), Value::String(_))
                    if ordinal + 1 == pointer.data()?.path.len() =>
                {
                    return add_work(path, 1);
                }
                (
                    Projection::Field(index),
                    Value::Record {
                        fields, ..
                    },
                ) => {
                    value = fields.get(*index).ok_or(Error::OutOfBounds {
                        index: *index,
                        length: fields.len(),
                    })?;
                }
                (
                    Projection::Index(index),
                    Value::Array {
                        elements, ..
                    },
                ) => {
                    value = elements.get(*index).ok_or(Error::OutOfBounds {
                        index: *index,
                        length: elements.len(),
                    })?;
                }
                _ => return Err(Error::InvalidIr("invalid structural copy projection")),
            }
        }
        add_work(path, value.cells(self.limits.value_cells)?)
    }

    /// Additional work before extracting a range. The caller charges the
    /// selected bytes and metadata; only a cold image touches the whole root.
    pub(crate) fn sequence_snapshot_work_cost(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        bytes: usize,
    ) -> Result<usize, Error> {
        if bytes == 0 {
            return Ok(0);
        }
        self.intrinsic_range(types, pointer, bytes, false)?;
        let allocation = self.allocation(pointer)?;
        let cold = if allocation.image.borrow().is_none() {
            self.cold_image_work(allocation)?
        } else {
            0
        };
        add_work(pointer.metadata_cells(), cold)
    }

    /// A new procedure identity can rehash the canonical code-token table.
    pub(crate) fn retokenize_work_cost(&self) -> usize {
        self.handle_tokens.borrow().values.capacity()
    }

    /// Partial writes stage a complete root clone before committing. They may
    /// also create a byte image for an incoming address-derived integer, so the
    /// preflight includes the root byte extent without inspecting that value.
    pub(crate) fn store_work_cost(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
    ) -> Result<usize, Error> {
        self.validate_pointer(types, pointer)?;
        self.validate_access(types, pointer)?;
        let allocation = self.allocation(pointer)?;
        if allocation.readonly {
            return Err(Error::ReadOnlyStorage);
        }
        let mut work = add_work(pointer.metadata_cells(), allocation.cells.get())?;
        if !pointer.data()?.path.is_empty() {
            work = add_work(
                work,
                usize::try_from(allocation.virtual_extent)
                    .map_err(|_| Error::Limit(LimitKind::Fuel))?,
            )?;
            work = add_work(work, self.retokenize_work_cost())?;
        }
        Ok(work)
    }

    fn cold_image_work(&self, allocation: &Allocation) -> Result<usize, Error> {
        if allocation.value.is_none() {
            return Err(Error::Uninitialized);
        }
        let bytes = usize::try_from(allocation.virtual_extent)
            .map_err(|_| Error::Limit(LimitKind::Fuel))?;
        add_work(
            add_work(bytes, allocation.cells.get())?,
            self.retokenize_work_cost(),
        )
    }
}

fn add_work(left: usize, right: usize) -> Result<usize, Error> {
    left.checked_add(right).ok_or(Error::Limit(LimitKind::Fuel))
}

#[derive(Clone, Copy)]
struct ZeroShape {
    cells: usize,
    height: usize,
}

fn zero_shape(
    types: &dyn TypeView,
    ty: TypeId,
    limits: Limits,
    depth: usize,
    cache: &mut HashMap<TypeId, ZeroShape>,
) -> Result<ZeroShape, Error> {
    let maximum_depth = limits.evaluation_depth.min(256);
    if depth > maximum_depth {
        return Err(Error::Limit(LimitKind::EvaluationDepth));
    }
    if let Some(shape) = cache.get(&ty) {
        if shape.height > maximum_depth - depth {
            return Err(Error::Limit(LimitKind::EvaluationDepth));
        }
        return Ok(*shape);
    }
    let mut shape = ZeroShape {
        cells: 1,
        height: 0,
    };
    let mut include = |child: ZeroShape, count: usize| -> Result<(), Error> {
        shape.cells = child
            .cells
            .checked_mul(count)
            .and_then(|cells| shape.cells.checked_add(cells))
            .filter(|cells| *cells <= limits.value_cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        shape.height = shape.height.max(child.height + 1);
        Ok(())
    };
    match types.kind(ty)? {
        TypeKind::DynamicArray(_) => {
            if let Some(schema) = crate::value::allocator_schema(types)? {
                include(zero_shape(types, schema.ty(), limits, depth + 1, cache)?, 1)?;
            }
        }
        TypeKind::FixedArray {
            element,
            count,
        } => {
            let count = usize::try_from(*count).map_err(|_| Error::Limit(LimitKind::ValueCells))?;
            if count != 0 {
                include(
                    zero_shape(types, *element, limits, depth + 1, cache)?,
                    count,
                )?;
            }
        }
        TypeKind::Distinct(id) => include(
            zero_shape(
                types,
                types.distinct(*id)?.representation,
                limits,
                depth + 1,
                cache,
            )?,
            1,
        )?,
        kind if kind.record_storage_id().is_some() => {
            let record = types.record_storage_definition(ty)?;
            if record.kind == jai_types::RecordKind::Union {
                let first = record.fields.first().ok_or(Error::UnsupportedType(ty))?;
                include(zero_shape(types, *first, limits, depth + 1, cache)?, 1)?;
            } else {
                if record.fields.len() >= limits.value_cells {
                    return Err(Error::Limit(LimitKind::ValueCells));
                }
                for field in &record.fields {
                    include(zero_shape(types, *field, limits, depth + 1, cache)?, 1)?;
                }
            }
        }
        TypeKind::Void | TypeKind::Code => return Err(Error::UnsupportedType(ty)),
        _ => {}
    }
    if shape.cells > limits.value_cells {
        return Err(Error::Limit(LimitKind::ValueCells));
    }
    cache.insert(ty, shape);
    Ok(shape)
}

#[cfg(test)]
mod tests;
