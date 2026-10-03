//! Metered private copies of unfinished continuation progress, with shared frozen code.
use super::*;

impl Machine {
    /// Cached complete task, operand and frozen-plan admission; no value walk.
    pub(in crate::execute::resumable) fn retained_cells(&self) -> usize {
        self.snapshot_cells().unwrap_or(usize::MAX)
    }

    pub(in crate::execute::resumable) fn accounted_cells(&self) -> usize {
        self.retained.saturating_add(self.plan_cells)
    }

    pub(in crate::execute::resumable) fn fork_result_scratch_cells(
        &self,
    ) -> std::result::Result<usize, Error> {
        self.operands
            .capacity()
            .max(self.operands.len().saturating_add(1))
            .checked_mul(4)
            .and_then(|cells| cells.checked_add(16))
            .ok_or(Error::Limit(LimitKind::ValueCells))
    }

    pub(in crate::execute::resumable) fn snapshot_cells(
        &self,
    ) -> std::result::Result<usize, Error> {
        if self.source_retired {
            return Ok(0);
        }
        let mut cells = self
            .accounted_cells()
            .checked_mul(3)
            .and_then(|cells| cells.checked_add(1))
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        for capacity in [
            self.plans.capacity(),
            self.tasks.capacity(),
            self.operands.capacity(),
        ] {
            cells = cells
                .checked_add(capacity)
                .ok_or(Error::Limit(LimitKind::ValueCells))?;
        }
        if let Some(values) = &self.result {
            cells = cells
                .checked_add(values.capacity())
                .ok_or(Error::Limit(LimitKind::ValueCells))?;
        }
        Ok(cells)
    }

    /// Constant-time conservative bound, including sparse outer collection tables.
    /// Pack masks and stored image metadata are covered by the threefold payload charge.
    pub(in crate::execute::resumable) fn fork_work_cost(
        &self,
    ) -> std::result::Result<usize, Error> {
        if self.source_retired {
            return Err(Error::InvalidIr(
                "cannot fork a retired source continuation",
            ));
        }
        if self.result.is_some() {
            return Err(Error::InvalidIr(
                "cannot fork a completed continuation result",
            ));
        }
        self.snapshot_cells()
            .map_err(|_| Error::Limit(LimitKind::Fuel))
    }

    /// The scheduler separately admits both branches and forks their Memory and bindings.
    /// Same-lineage pointers may remain inside each branch, never in branch transports.
    pub(in crate::execute::resumable) fn fork_private(
        &self,
        available_work: usize,
    ) -> std::result::Result<Self, Error> {
        if self.fork_work_cost()? > available_work {
            return Err(Error::Limit(LimitKind::Fuel));
        }
        Ok(self.clone())
    }
}

#[cfg(test)]
mod tests;
