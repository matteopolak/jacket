//! Immediate byte snapshots retained while later variadic arguments suspend.
use super::super::*;
use super::{Operand, plan::PackPartMode};
use crate::ByteImage;
#[cfg(test)]
mod tests;

#[derive(Clone)]
pub(super) struct PackState {
    ty: TypeId,
    element: TypeId,
    size: u64,
    alignment: u32,
    snapshots: Vec<ByteImage>,
    count: usize,
    metadata: usize,
    cells: usize,
}

impl PackState {
    pub(super) fn new<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        vm: &mut Vm<'_, P, E>,
        ty: TypeId,
    ) -> Result<Self> {
        let TypeKind::Slice(element) = *vm.provider.types().kind(ty)? else {
            return Err(Error::InvalidIr("sequence pack requires a slice type").into());
        };
        vm.prepare_layout(element)?;
        let layout = vm.memory.prepared_layout(vm.provider.types(), element)?;
        Ok(Self {
            ty,
            element,
            size: layout.size,
            alignment: layout.alignment,
            snapshots: Vec::new(),
            count: 0,
            metadata: 0,
            cells: 0,
        })
    }

    /// Cached retained image bytes, metadata, and snapshot headers for task admission.
    pub(super) fn cells(&self) -> usize {
        self.cells.saturating_add(self.snapshots.capacity())
    }

    /// Capture only a computed operand. No source expression is retained or rerun.
    pub(super) fn capture<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        mode: PackPartMode,
        operand: Operand,
        depth: usize,
    ) -> Result<()> {
        vm.step(depth)?;
        if self.snapshots.len() >= vm.limits.value_cells {
            return Err(Error::Limit(LimitKind::ValueCells).into());
        }
        let (count, snapshot) = match mode {
            PackPartMode::ElementValue | PackPartMode::ElementPlace => {
                let (value, storage) = match (mode, operand) {
                    (PackPartMode::ElementValue, Operand::Value(value)) => (value, None),
                    (PackPartMode::ElementPlace, Operand::Place(pointer)) => {
                        let value = vm.load_sequence_value(&pointer)?;
                        let storage = (!matches!(value, Value::String(_))).then_some(pointer);
                        (value, storage)
                    }
                    _ => return Err(Error::InvalidIr("pack element has wrong operand kind").into()),
                };
                let value = vm.normalize_storage_value(value, depth + 1)?;
                value.validate(
                    vm.provider.types(),
                    self.element,
                    vm.limits.evaluation_depth.min(256),
                )?;
                vm.sequence_pack_bytes(
                    self.count.checked_add(1).ok_or(Error::CheckedCast)?,
                    self.size,
                )?;
                vm.charge_sequence_work(self.size)?;
                let snapshot = if let Some(pointer) = storage {
                    let bytes = usize::try_from(self.size).map_err(|_| Error::CheckedCast)?;
                    vm.prepare_pointer_layouts(&pointer, true)?;
                    vm.charge_work(vm.memory.sequence_snapshot_work_cost(
                        vm.provider.types(),
                        &pointer,
                        bytes,
                    )?)?;
                    vm.memory
                        .sequence_snapshot(vm.provider.types(), &pointer, bytes)?
                } else {
                    ByteImage::encode(
                        vm.provider.types(),
                        vm.memory.target(),
                        self.element,
                        &value,
                        vm.limits.value_cells,
                    )?
                };
                (1, snapshot)
            }
            PackPartMode::Spread => {
                let value = operand.into_value()?;
                value.validate(
                    vm.provider.types(),
                    self.ty,
                    vm.limits.evaluation_depth.min(256),
                )?;
                let Value::Slice {
                    pointer,
                    count,
                    ..
                } = value
                else {
                    return Err(
                        Error::InvalidIr("sequence spread requires a slice descriptor").into(),
                    );
                };
                let count = crate::checked_sequence_count(count)?;
                if count != 0 && pointer.is_null() {
                    return Err(Error::NullPointer.into());
                }
                if count != 0 {
                    vm.prepare_pointer_layouts(&pointer, false)?;
                    vm.memory.cast_pointer(
                        vm.provider.types(),
                        &pointer,
                        self.element,
                        CastMode::Checked,
                    )?;
                }
                let bytes = vm.sequence_pack_bytes(count, self.size)?;
                vm.sequence_pack_bytes(
                    self.count.checked_add(count).ok_or(Error::CheckedCast)?,
                    self.size,
                )?;
                vm.charge_sequence_allocation(count, bytes, self.alignment)?;
                vm.charge_sequence_work(bytes)?;
                let bytes = usize::try_from(bytes).map_err(|_| Error::CheckedCast)?;
                if count != 0 {
                    vm.prepare_pointer_layouts(&pointer, true)?;
                }
                vm.charge_work(vm.memory.sequence_snapshot_work_cost(
                    vm.provider.types(),
                    &pointer,
                    bytes,
                )?)?;
                (
                    count,
                    vm.memory
                        .sequence_snapshot(vm.provider.types(), &pointer, bytes)?,
                )
            }
        };
        let count = self
            .count
            .checked_add(count)
            .filter(|count| i64::try_from(*count).is_ok())
            .ok_or(Error::CheckedCast)?;
        let metadata = self
            .metadata
            .checked_add(snapshot.metadata_cells())
            .filter(|cells| *cells <= vm.limits.value_cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let cells = self
            .cells
            .checked_add(snapshot.len())
            .and_then(|cells| cells.checked_add(snapshot.metadata_cells()))
            .and_then(|cells| cells.checked_add(1))
            .filter(|cells| *cells <= vm.limits.value_cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        vm.charge_work(snapshot.metadata_cells())?;
        if !self.snapshots.is_empty() && self.snapshots.len() == self.snapshots.capacity() {
            // Growing this vector moves headers only; image payloads stay owned.
            vm.charge_work(self.snapshots.len())?;
        }
        self.snapshots.push(snapshot);
        self.count = count;
        self.metadata = metadata;
        self.cells = cells;
        Ok(())
    }

    pub(super) fn finish<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        self,
        vm: &mut Vm<'_, P, E>,
    ) -> Result<Value> {
        let bytes = vm.sequence_pack_bytes(self.count, self.size)?;
        vm.charge_sequence_allocation(self.count, bytes, self.alignment)?;
        vm.charge_sequence_work(bytes)?;
        if self.count == 0 {
            return Ok(Value::Slice {
                ty: self.ty,
                pointer: Pointer::null(self.element),
                count: 0,
            });
        }
        if bytes != 0 {
            vm.charge_work(self.metadata)?;
        }
        let image = if bytes == 0 {
            ByteImage::from_bytes(vm.memory.target(), vec![0], vm.limits.value_cells)?
        } else {
            ByteImage::concatenate(&self.snapshots, vm.memory.target(), vm.limits.value_cells)?
        };
        if self.metadata != 0 {
            vm.charge_work(vm.memory.retokenize_work_cost())?;
        }
        vm.prepare_layout(self.element)?;
        let root =
            vm.memory
                .allocate_sequence_buffer(vm.provider.types(), self.element, self.count)?;
        if let Some(frame) = vm.frames.last_mut() {
            frame.temporaries.push(root.clone());
            frame.sequence_temp_roots.push(root.clone());
        } else {
            vm.root_temporaries.push(root.clone());
        }
        vm.memory
            .install_sequence_buffer(vm.provider.types(), &root, image)?;
        vm.prepare_pointer_layouts(&root, false)?;
        let pointer = vm.memory.cast_pointer(
            vm.provider.types(),
            &root,
            self.element,
            CastMode::Unchecked,
        )?;
        Ok(Value::Slice {
            ty: self.ty,
            pointer,
            count: i64::try_from(self.count).map_err(|_| Error::CheckedCast)?,
        })
    }
}
