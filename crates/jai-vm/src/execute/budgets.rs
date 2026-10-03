//! Incremental admission keeps aggregate children within one result budget.
use super::*;

impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    /// Admit cold target-layout work before an operation can calculate or retain it.
    /// Failed readiness attempts consume their bounded traversal work as well.
    pub(super) fn prepare_layout(&mut self, ty: TypeId) -> Result<()> {
        let available = usize::try_from(self.limits.fuel.saturating_sub(self.statistics.steps))
            .unwrap_or(usize::MAX);
        let (work, result) = self
            .memory
            .prepare_layout(self.provider.types(), ty, available);
        self.charge_work(work)?;
        Ok(result?)
    }

    /// Address construction needs enclosing layouts; access additionally needs
    /// the final view layout. A null or pending pointee remains valid as an address.
    pub(super) fn prepare_pointer_layouts(
        &mut self,
        pointer: &Pointer,
        include_pointee: bool,
    ) -> Result<()> {
        let available = usize::try_from(self.limits.fuel.saturating_sub(self.statistics.steps))
            .unwrap_or(usize::MAX);
        let (work, result) = self.memory.prepare_pointer_layouts(
            self.provider.types(),
            pointer,
            include_pointee,
            available,
        );
        self.charge_work(work)?;
        Ok(result?)
    }

    pub(super) fn prepare_number_address(&mut self, number: &Number) -> Result<()> {
        if let Some(crate::AddressProvenance::Pointer(pointer)) = number.provenance() {
            self.prepare_pointer_layouts(pointer, false)?;
        }
        Ok(())
    }

    pub(super) fn prepare_field_layouts(&mut self, pointer: &Pointer, field: usize) -> Result<()> {
        let available = usize::try_from(self.limits.fuel.saturating_sub(self.statistics.steps))
            .unwrap_or(usize::MAX);
        let (work, result) =
            self.memory
                .prepare_field_layouts(self.provider.types(), pointer, field, available);
        self.charge_work(work)?;
        Ok(result?)
    }

    /// Charge the expanded value shape before allocating default aggregate nodes.
    pub(super) fn zero_value(&mut self, ty: TypeId) -> Result<Value> {
        let work = self
            .memory
            .zero_value_work_cost(self.provider.types(), ty)?;
        self.charge_work(work)?;
        Ok(crate::constants::zero(
            self.provider.types(),
            ty,
            self.limits,
        )?)
    }

    pub(super) fn push_value(
        &mut self,
        values: &mut Vec<Value>,
        cells: &mut usize,
        value: Value,
    ) -> Result<()> {
        self.admit_value(cells, &value)?;
        values.push(value);
        Ok(())
    }

    pub(super) fn admit_value(&mut self, cells: &mut usize, value: &Value) -> Result<()> {
        let child = value.cells(self.limits.value_cells)?;
        *cells = cells
            .checked_add(child)
            .filter(|total| *total <= self.limits.value_cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        self.charge_work(child)?;
        Ok(())
    }
}
