//! Apply sealed storage casts to one captured source image.
use super::*;
use jai_types::StorageBitcast;
#[cfg(test)]
mod tests;

impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    pub(super) fn storage_bitcast_place(
        &mut self,
        source: &Pointer,
        cast: StorageBitcast,
    ) -> Result<Value> {
        self.prepare_layout(cast.source_type())?;
        self.prepare_layout(cast.target_type())?;
        self.prepare_pointer_layouts(source, true)?;
        let work =
            self.memory
                .storage_bitcast_place_work_cost(self.provider.types(), source, cast)?;
        self.charge_work(work)?;
        // Loading a structural source first would discard inactive bytes and
        // reject holes outside the selected destination prefix.
        Ok(self
            .memory
            .storage_bitcast_place(self.provider.types(), source, cast)?)
    }

    pub(super) fn storage_bitcast_value(
        &mut self,
        source: Value,
        cast: StorageBitcast,
        depth: usize,
    ) -> Result<Value> {
        self.prepare_layout(cast.source_type())?;
        self.prepare_layout(cast.target_type())?;
        // Language strings occupy descriptors; immutable literal byte roots
        // are not the storage representation of a string expression.
        let source = self.normalize_storage_value(source, depth + 1)?;
        let work =
            self.memory
                .storage_bitcast_value_work_cost(self.provider.types(), &source, cast)?;
        self.charge_work(work)?;
        Ok(self
            .memory
            .storage_bitcast_value(self.provider.types(), &source, cast)?)
    }
}
