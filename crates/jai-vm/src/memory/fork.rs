//! Deep private branch snapshots. Same-lineage handles never cross branch boundaries.
use super::*;

impl Memory {
    /// Constant-time conservative work bound; no allocation or value is traversed.
    /// An allocation may retain both semantic storage and an image, charged by their
    /// maximum. Three times that charge covers the value, image and initialization mask.
    pub(crate) fn snapshot_work_cost(&self) -> Result<usize, Error> {
        let mut work = self
            .cells
            .get()
            .checked_mul(3)
            .and_then(|work| work.checked_add(1))
            .ok_or(Error::Limit(LimitKind::Fuel))?;
        for capacity in [
            self.allocations.capacity(),
            self.handle_tokens.borrow().values.capacity(),
            self.virtual_regions.len(),
            self.runtime_types.capacity(),
            self.pool_ledger.table_capacity(),
            self.layout_cache.borrow().table_capacity(),
        ] {
            work = work
                .checked_add(capacity)
                .ok_or(Error::Limit(LimitKind::Fuel))?;
        }
        Ok(work)
    }

    /// Fork only after the caller admits combined branch retention and charges work.
    /// This is not an independent public Memory: inherited handles keep their original
    /// lineage, while future identities may coincide in isolated sibling states.
    /// Branch transports must reject all pointer/address/code provenance.
    pub(crate) fn fork_private_branch(&self, available_work: usize) -> Result<Self, Error> {
        if self.snapshot_work_cost()? > available_work {
            return Err(Error::Limit(LimitKind::Fuel));
        }
        Ok(Self {
            identity: self.identity,
            next_allocation: self.next_allocation,
            next_virtual_address: Cell::new(self.next_virtual_address.get()),
            virtual_regions: self.virtual_regions.clone(),
            allocations: self.allocations.clone(),
            cells: Cell::new(self.cells.get()),
            limits: self.limits,
            target: self.target,
            handle_tokens: RefCell::new(HandleTokens {
                values: self.handle_tokens.borrow().values.clone(),
            }),
            runtime_types: self.runtime_types.clone(),
            pool_ledger: self.pool_ledger.clone(),
            layout_cache: RefCell::new(self.layout_cache.borrow().clone()),
        })
    }
}

#[cfg(test)]
mod tests;
