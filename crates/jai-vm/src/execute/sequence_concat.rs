//! Immediate snapshots and caller-owned storage for ordered variadic packs.
use super::*;
use crate::ByteImage;
use std::collections::HashSet;

impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    pub(super) fn sequence_concat(
        &mut self,
        ty: TypeId,
        parts: &[SequencePackPart],
        depth: usize,
    ) -> Result<Value> {
        self.step(depth)?;
        let TypeKind::Slice(element) = *self.provider.types().kind(ty)? else {
            return Err(Error::InvalidIr("sequence pack requires a slice type").into());
        };
        self.prepare_layout(element)?;
        let layout = self
            .memory
            .prepared_layout(self.provider.types(), element)?;
        if parts.len() > self.limits.value_cells {
            return Err(Error::Limit(LimitKind::ValueCells).into());
        }
        let mut snapshots = Vec::with_capacity(parts.len());
        let mut total = 0usize;
        let mut metadata = 0usize;
        for part in parts {
            self.step(depth + 1)?;
            let (count, snapshot) = match part {
                SequencePackPart::Element(expression) => {
                    let (value, storage) = if let ValueExpr::Load(place) = expression {
                        self.step(depth + 1)?;
                        let pointer = self.place(*place, depth + 2)?;
                        let value = self.load_sequence_value(&pointer)?;
                        let storage = (!matches!(value, Value::String(_))).then_some(pointer);
                        (value, storage)
                    } else {
                        (self.value(expression, depth + 1)?, None)
                    };
                    let value = self.normalize_storage_value(value, depth + 1)?;
                    value.validate(
                        self.provider.types(),
                        element,
                        self.limits.evaluation_depth.min(256),
                    )?;
                    self.sequence_pack_bytes(
                        total.checked_add(1).ok_or(Error::CheckedCast)?,
                        layout.size,
                    )?;
                    self.charge_sequence_work(layout.size)?;
                    let snapshot = if let Some(pointer) = storage {
                        let bytes = usize::try_from(layout.size).map_err(|_| Error::CheckedCast)?;
                        self.prepare_pointer_layouts(&pointer, true)?;
                        self.charge_work(self.memory.sequence_snapshot_work_cost(
                            self.provider.types(),
                            &pointer,
                            bytes,
                        )?)?;
                        // A loaded aggregate may contain inactive union bytes or
                        // padding that its structural value cannot describe.
                        self.memory
                            .sequence_snapshot(self.provider.types(), &pointer, bytes)?
                    } else {
                        ByteImage::encode(
                            self.provider.types(),
                            self.memory.target(),
                            element,
                            &value,
                            self.limits.value_cells,
                        )?
                    };
                    (1, snapshot)
                }
                SequencePackPart::Spread(expression) => {
                    let value = self.value(expression, depth + 1)?;
                    value.validate(
                        self.provider.types(),
                        ty,
                        self.limits.evaluation_depth.min(256),
                    )?;
                    let Value::Slice { pointer, count, .. } = value else {
                        return Err(Error::InvalidIr(
                            "sequence spread requires a slice descriptor",
                        )
                        .into());
                    };
                    let count = crate::checked_sequence_count(count)?;
                    if count != 0 && pointer.is_null() {
                        return Err(Error::NullPointer.into());
                    }
                    if count != 0 {
                        // Zero-sized copies still require a live, local allocation.
                        self.prepare_pointer_layouts(&pointer, false)?;
                        self.memory.cast_pointer(
                            self.provider.types(),
                            &pointer,
                            element,
                            CastMode::Checked,
                        )?;
                    }
                    let bytes = self.sequence_pack_bytes(count, layout.size)?;
                    self.sequence_pack_bytes(
                        total.checked_add(count).ok_or(Error::CheckedCast)?,
                        layout.size,
                    )?;
                    self.charge_sequence_allocation(count, bytes, layout.alignment)?;
                    self.charge_sequence_work(bytes)?;
                    let snapshot_bytes = usize::try_from(bytes).map_err(|_| Error::CheckedCast)?;
                    if count != 0 {
                        self.prepare_pointer_layouts(&pointer, true)?;
                    }
                    self.charge_work(self.memory.sequence_snapshot_work_cost(
                        self.provider.types(),
                        &pointer,
                        snapshot_bytes,
                    )?)?;
                    // Snapshot before evaluating the next part, preserving raw union,
                    // initialization, pointer, procedure and address-integer metadata.
                    let snapshot = self.memory.sequence_snapshot(
                        self.provider.types(),
                        &pointer,
                        snapshot_bytes,
                    )?;
                    (count, snapshot)
                }
            };
            total = total
                .checked_add(count)
                .filter(|total| u64::try_from(*total).is_ok_and(|total| total <= i64::MAX as u64))
                .ok_or(Error::CheckedCast)?;
            let snapshot_metadata = snapshot.metadata_cells();
            metadata = metadata
                .checked_add(snapshot_metadata)
                .filter(|cells| *cells <= self.limits.value_cells)
                .ok_or(Error::Limit(LimitKind::ValueCells))?;
            // Charge origins and opaque metadata before the next argument runs
            // or a later assembly clones this snapshot.
            self.charge_sequence_work(
                u64::try_from(snapshot_metadata).map_err(|_| Error::Limit(LimitKind::Fuel))?,
            )?;
            snapshots.push(snapshot);
        }
        let bytes = self.sequence_pack_bytes(total, layout.size)?;
        self.charge_sequence_allocation(total, bytes, layout.alignment)?;
        self.charge_sequence_work(bytes)?;
        if total == 0 {
            return Ok(Value::Slice {
                ty,
                pointer: Pointer::null(element),
                count: 0,
            });
        }
        if bytes != 0 {
            // Concatenation retains every snapshot metadata record. Bound its
            // cloning work before assembling or installing the final image.
            self.charge_sequence_work(
                u64::try_from(metadata).map_err(|_| Error::Limit(LimitKind::Fuel))?,
            )?;
        }
        let image = if bytes == 0 {
            ByteImage::from_bytes(self.memory.target(), vec![0], self.limits.value_cells)?
        } else {
            ByteImage::concatenate(&snapshots, self.memory.target(), self.limits.value_cells)?
        };
        if metadata != 0 {
            self.charge_work(self.memory.retokenize_work_cost())?;
        }
        self.prepare_layout(element)?;
        let root = self
            .memory
            .allocate_sequence_buffer(self.provider.types(), element, total)?;
        if let Some(frame) = self.frames.last_mut() {
            frame.temporaries.push(root.clone());
            frame.sequence_temp_roots.push(root.clone());
        } else {
            self.root_temporaries.push(root.clone());
        }
        self.memory
            .install_sequence_buffer(self.provider.types(), &root, image)?;
        self.prepare_pointer_layouts(&root, false)?;
        let pointer =
            self.memory
                .cast_pointer(self.provider.types(), &root, element, CastMode::Unchecked)?;
        Ok(Value::Slice {
            ty,
            pointer,
            count: i64::try_from(total).map_err(|_| Error::CheckedCast)?,
        })
    }

    pub(super) fn sequence_pack_bytes(&self, count: usize, element_size: u64) -> Result<u64> {
        u64::try_from(count)
            .ok()
            .and_then(|count| count.checked_mul(element_size))
            .filter(|bytes| *bytes <= MAX_SEQUENCE_TEMP_BYTES)
            .ok_or_else(|| Error::Limit(LimitKind::SequenceTemporaryBytes).into())
    }
    pub(super) fn charge_sequence_allocation(
        &mut self,
        count: usize,
        bytes: u64,
        alignment: u32,
    ) -> Result<()> {
        if count == 0 {
            return Ok(());
        }
        let charge = sequence_temp_allocation_charge(bytes, alignment)
            .ok_or(Error::Limit(LimitKind::SequenceTemporaryBytes))?;
        let used = match self.frames.last_mut() {
            Some(frame) => &mut frame.sequence_temp_bytes,
            None => &mut self.root_sequence_temp_bytes,
        };
        *used = used
            .checked_add(charge)
            .filter(|used| *used <= MAX_SEQUENCE_TEMP_BYTES)
            .ok_or(Error::Limit(LimitKind::SequenceTemporaryBytes))?;
        Ok(())
    }
    pub(super) fn charge_sequence_work(&mut self, bytes: u64) -> Result<()> {
        self.statistics.steps = self
            .statistics
            .steps
            .checked_add(bytes)
            .filter(|steps| *steps <= self.limits.fuel)
            .ok_or(Error::Limit(LimitKind::Fuel))?;
        Ok(())
    }

    pub(super) fn validate_sequence_escape(
        &self,
        values: &[Value],
        roots: &[Pointer],
    ) -> std::result::Result<(), Error> {
        if roots.is_empty() {
            return Ok(());
        }
        if roots.len() > self.limits.value_cells || values.len() > self.limits.value_cells {
            return Err(Error::Limit(LimitKind::ValueCells));
        }
        let mut work = self
            .limits
            .fuel
            .checked_sub(u64::try_from(roots.len()).map_err(|_| Error::Limit(LimitKind::Fuel))?)
            .ok_or(Error::Limit(LimitKind::Fuel))?;
        let keys: HashSet<_> = roots
            .iter()
            .filter_map(Pointer::data_allocation_key)
            .collect();
        let mut pending: Vec<_> = values.iter().map(|value| (value, 0usize)).collect();
        let mut remaining = self.limits.value_cells;
        while let Some((value, depth)) = pending.pop() {
            remaining = remaining
                .checked_sub(1)
                .ok_or(Error::Limit(LimitKind::ValueCells))?;
            work = work.checked_sub(1).ok_or(Error::Limit(LimitKind::Fuel))?;
            if depth > self.limits.evaluation_depth.min(256) {
                return Err(Error::Limit(LimitKind::EvaluationDepth));
            }
            let key = match value {
                Value::Pointer(pointer)
                | Value::Slice { pointer, .. }
                | Value::DynamicArray { pointer, .. }
                | Value::StringView { pointer, .. }
                | Value::Type {
                    descriptor: Some(pointer),
                } => pointer.data_allocation_key(),
                _ => None,
            };
            if key.is_some_and(|key| keys.contains(&key)) {
                return Err(Error::SequenceTemporaryEscape);
            }
            if let Value::AddressInteger(number) = value {
                let origins = number.origin_count();
                remaining = remaining
                    .checked_sub(origins)
                    .ok_or(Error::Limit(LimitKind::ValueCells))?;
                work = work
                    .checked_sub(u64::try_from(origins).map_err(|_| Error::Limit(LimitKind::Fuel))?)
                    .ok_or(Error::Limit(LimitKind::Fuel))?;
                if let Some(memory) = number.memory_identity() {
                    if memory != roots[0].memory_identity() {
                        return Err(Error::ForeignPointer);
                    }
                    if number
                        .allocation_ids_iter()
                        .any(|allocation| keys.contains(&(memory, allocation)))
                    {
                        return Err(Error::SequenceTemporaryEscape);
                    }
                }
            }
            match value {
                Value::StoredAggregate(snapshot) => {
                    remaining = remaining
                        .checked_sub(snapshot.storage_cells())
                        .ok_or(Error::Limit(LimitKind::ValueCells))?;
                    work = work
                        .checked_sub(
                            u64::try_from(snapshot.image().metadata_cells())
                                .map_err(|_| Error::Limit(LimitKind::Fuel))?,
                        )
                        .ok_or(Error::Limit(LimitKind::Fuel))?;
                    snapshot
                        .image()
                        .visit_address_origins(|memory, allocation| {
                            if memory != roots[0].memory_identity() {
                                return Err(Error::ForeignPointer);
                            }
                            if keys.contains(&(memory, allocation)) {
                                return Err(Error::SequenceTemporaryEscape);
                            }
                            Ok(())
                        })?;
                    if let Some(semantic) = snapshot.decoded_semantic() {
                        pending.push((semantic, depth + 1));
                    }
                }
                Value::Record { fields, .. }
                | Value::Array {
                    elements: fields, ..
                } => {
                    if fields.len() > remaining.saturating_sub(pending.len()) {
                        return Err(Error::Limit(LimitKind::ValueCells));
                    }
                    pending.extend(fields.iter().map(|value| (value, depth + 1)));
                }
                Value::Distinct { value, .. } | Value::Union { value, .. } => {
                    pending.push((value, depth + 1))
                }
                Value::DynamicArray {
                    allocator: Some(allocator),
                    ..
                } => {
                    pending.push((allocator.as_ref(), depth + 1));
                }
                _ => {}
            }
        }
        Ok(())
    }
}
