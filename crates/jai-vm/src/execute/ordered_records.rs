//! Physical, ordered field writes; skipped defaults never create parent values.
use super::*;
use crate::ByteImage;
use jai_types::{FieldId, Layout, RecordKind};
use std::sync::Arc;

#[derive(Clone)]
pub(super) struct OrderedRecordState {
    ty: TypeId,
    layout: Arc<Layout>,
    image: ByteImage,
    writes: Arc<[PreparedWrite]>,
}

#[derive(Clone, Copy)]
struct PreparedWrite {
    ty: TypeId,
    offset: usize,
    extent: usize,
}

impl OrderedRecordState {
    /// Include both the bytes and their initialization mask in scheduler admission.
    pub(super) fn cells(&self, limit: usize) -> std::result::Result<usize, Error> {
        self.image
            .len()
            .checked_mul(2)
            .and_then(|n| n.checked_add(self.image.metadata_cells()))
            .and_then(|n| n.checked_add(self.layout.field_offsets.len()))
            .and_then(|n| n.checked_add(self.writes.len().saturating_mul(3)))
            .and_then(|n| n.checked_add(2))
            .filter(|n| *n <= limit)
            .ok_or(Error::Limit(LimitKind::ValueCells))
    }
}

impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    /// Layout and path readiness runs before consuming an initializer or an operand.
    pub(super) fn prepare_ordered_record<'a>(
        &mut self,
        ty: TypeId,
        paths: impl IntoIterator<Item = &'a [FieldId]>,
    ) -> Result<()> {
        self.prepare_layout(ty)?;
        if self.provider.types().record_storage_definition(ty)?.kind != RecordKind::Struct {
            return Err(Error::InvalidIr("ordered record requires a struct owner").into());
        }
        for path in paths {
            if path.is_empty() {
                return Err(Error::InvalidIr("ordered record write has an empty path").into());
            }
            if path.len() > self.limits.evaluation_depth.min(128) {
                return Err(Error::Limit(LimitKind::EvaluationDepth).into());
            }
            let mut current = ty;
            for &field in path {
                self.charge_work(1)?;
                if self
                    .provider
                    .types()
                    .record_storage_definition(current)?
                    .kind
                    != RecordKind::Struct
                {
                    return Err(
                        Error::InvalidIr("ordered record path requires struct fields").into(),
                    );
                }
                self.prepare_layout(current)?;
                current = self.provider.types().validate_field(current, field)?;
            }
            self.prepare_layout(current)?;
        }
        Ok(())
    }

    pub(super) fn start_ordered_record<'a>(
        &mut self,
        ty: TypeId,
        backing: OrderedRecordBacking,
        paths: impl IntoIterator<Item = &'a [FieldId]>,
        retained: usize,
    ) -> Result<OrderedRecordState> {
        let layout = self.memory.prepared_layout(self.provider.types(), ty)?;
        let length =
            usize::try_from(layout.size).map_err(|_| Error::Limit(LimitKind::ValueCells))?;
        let mut writes = Vec::new();
        let base_cells = length
            .checked_mul(2)
            .and_then(|n| n.checked_add(layout.field_offsets.len()))
            .and_then(|n| n.checked_add(2))
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        self.admit_ordered_cells(base_cells, retained)?;
        for path in paths {
            let mut current = ty;
            let mut offset = 0u64;
            for &field in path {
                let parent = self
                    .memory
                    .prepared_layout(self.provider.types(), current)?;
                offset = offset
                    .checked_add(
                        *parent
                            .field_offsets
                            .get(field.index())
                            .ok_or(Error::InvalidIr("ordered record field offset is absent"))?,
                    )
                    .ok_or(Error::Limit(LimitKind::ValueCells))?;
                current = self.provider.types().validate_field(current, field)?;
            }
            let terminal = self
                .memory
                .prepared_layout(self.provider.types(), current)?;
            let offset =
                usize::try_from(offset).map_err(|_| Error::Limit(LimitKind::ValueCells))?;
            let extent =
                usize::try_from(terminal.size).map_err(|_| Error::Limit(LimitKind::ValueCells))?;
            if offset.checked_add(extent).is_none_or(|end| end > length) {
                return Err(
                    Error::InvalidIr("ordered record path escapes its backing extent").into(),
                );
            }
            self.admit_ordered_cells(
                base_cells.saturating_add((writes.len() + 1).saturating_mul(6)),
                retained,
            )?;
            self.charge_work(path.len().saturating_add(3))?;
            writes.push(PreparedWrite {
                ty: current,
                offset,
                extent,
            });
        }
        self.charge_work(length.saturating_mul(2))?;
        let image = match backing {
            OrderedRecordBacking::Zeroed => ByteImage::from_bytes(
                self.memory.target(),
                vec![0; length],
                self.limits.value_cells,
            )?,
            OrderedRecordBacking::Uninitialized => {
                ByteImage::uninitialized(self.memory.target(), length, self.limits.value_cells)?
            }
        };
        Ok(OrderedRecordState {
            ty,
            layout,
            image,
            writes: writes.into(),
        })
    }

    /// Ordinary evaluation retains this backing outside the expression stack.
    pub(super) fn ordered_record_initializer(
        &mut self,
        state: &OrderedRecordState,
        expression: &ValueExpr,
        depth: usize,
    ) -> Result<Value> {
        let cells = state.cells(self.limits.value_cells)?;
        let original = self.limits.value_cells;
        let narrowed = original
            .checked_sub(cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let memory_original = self.memory.value_cell_limit();
        let memory_narrowed = memory_original
            .checked_sub(cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        self.memory.replace_value_cell_limit(memory_narrowed)?;
        self.limits.value_cells = narrowed;
        let result = self.value(expression, depth);
        self.limits.value_cells = original;
        self.memory.replace_value_cell_limit(memory_original)?;
        result
    }

    fn admit_ordered_cells(&self, cells: usize, retained: usize) -> Result<()> {
        cells
            .checked_add(retained)
            .and_then(|n| n.checked_add(self.memory.value_cells()))
            .and_then(|n| n.checked_add(self.expression_bindings.cells()))
            .and_then(|n| {
                n.checked_add(
                    self.processes
                        .as_ref()
                        .map_or(0, process::ProcessState::cells),
                )
            })
            .filter(|n| *n <= self.limits.value_cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        Ok(())
    }

    pub(super) fn write_ordered_record(
        &mut self,
        state: &mut OrderedRecordState,
        index: usize,
        value: &Value,
        retained: usize,
    ) -> Result<()> {
        let write = *state
            .writes
            .get(index)
            .ok_or(Error::InvalidIr("ordered record write index is absent"))?;
        value.validate(
            self.provider.types(),
            write.ty,
            self.limits.evaluation_depth.min(128),
        )?;
        let value_cells = value.cells(self.limits.value_cells)?;
        // Splitting existing address provenance temporarily retains the old
        // receipt and its fragments, including projected-pointer paths. Three
        // backing copies bound those clones as well as the old metadata vectors;
        // incoming value receipts and the encoded patch are admitted separately.
        let transient = state
            .cells(self.limits.value_cells)?
            .checked_mul(3)
            .and_then(|n| n.checked_add(value_cells.saturating_mul(3)))
            .and_then(|n| n.checked_add(write.extent.saturating_mul(2)))
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        self.admit_ordered_cells(transient, retained)?;
        let codec = self
            .memory
            .prepared_codec_layout_work(self.provider.types(), write.ty)?;
        self.charge_work(transient.saturating_add(codec))?;
        self.memory.write_ordered_record_patch(
            self.provider.types(),
            &mut state.image,
            write.offset,
            write.ty,
            value,
        )?;
        self.admit_ordered_cells(state.cells(self.limits.value_cells)?, retained)?;
        Ok(())
    }

    pub(super) fn finish_ordered_record(
        &mut self,
        state: OrderedRecordState,
        retained: usize,
    ) -> Result<Value> {
        self.admit_ordered_cells(
            state.cells(self.limits.value_cells)?.saturating_mul(2),
            retained,
        )?;
        self.charge_work(state.cells(self.limits.value_cells)?.saturating_mul(2))?;
        Ok(self.memory.finish_ordered_record(
            self.provider.types(),
            state.ty,
            state.layout,
            state.image,
        )?)
    }
}

#[cfg(test)]
mod tests;
