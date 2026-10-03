//! Scoped immutable values shared by both execution engines.
use super::*;

impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    fn binding_capacity(&self, retained: usize) -> Result<usize> {
        self.limits
            .value_cells
            .checked_sub(
                self.memory
                    .value_cells()
                    .checked_add(retained)
                    .and_then(|cells| cells.checked_add(self.expression_bindings.cells()))
                    .and_then(|cells| {
                        cells.checked_add(
                            self.processes
                                .as_ref()
                                .map_or(0, process::ProcessState::cells),
                        )
                    })
                    .ok_or(Error::Limit(LimitKind::ValueCells))?,
            )
            .ok_or_else(|| Error::Limit(LimitKind::ValueCells).into())
    }
    pub(super) fn begin_bindings(&mut self, retained: usize) -> Result<bindings::ScopeToken> {
        if self.binding_capacity(retained)? == 0 {
            return Err(Error::Limit(LimitKind::ValueCells).into());
        }
        self.charge_work(1)?;
        Ok(self.expression_bindings.begin())
    }
    pub(super) fn capture_binding(
        &mut self,
        binding: ExpressionBindingId,
        value: Value,
        retained: usize,
    ) -> Result<()> {
        let available = self.binding_capacity(retained)?;
        let cells = value.cells(available.saturating_sub(1))?;
        self.charge_work(cells.saturating_add(1))?;
        self.memory
            .validate_runtime_type_values(self.provider.types(), &value)?;
        self.expression_bindings.insert(binding, value, available)?;
        Ok(())
    }
    pub(super) fn bound_value(
        &mut self,
        binding: ExpressionBindingId,
        retained: usize,
    ) -> Result<Value> {
        let cells = self.expression_bindings.clone_charge(binding)?;
        if cells > self.binding_capacity(retained)? {
            return Err(Error::Limit(LimitKind::ValueCells).into());
        }
        self.charge_work(cells.saturating_add(self.expression_bindings.depth()))?;
        let value = self.expression_bindings.lookup(binding)?;
        self.memory
            .validate_runtime_type_values(self.provider.types(), value)?;
        Ok(value.clone())
    }
    pub(super) fn end_bindings(&mut self, scope: bindings::ScopeToken) -> Result<()> {
        Ok(self.expression_bindings.end(scope)?)
    }
}
