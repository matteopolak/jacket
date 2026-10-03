//! Typed swaps stage both reads and roll back failed stores as one operation.
use super::*;

impl Memory {
    pub(crate) fn swap_work_cost(
        &self,
        types: &dyn TypeView,
        left: &Pointer,
        right: &Pointer,
    ) -> Result<u64, Error> {
        self.validate_pointer(types, left)?;
        self.validate_pointer(types, right)?;
        if left.pointee != right.pointee {
            return Err(Error::TypeMismatch {
                expected: left.pointee,
            });
        }
        // The VM admitted this fact with the target layout before asking for
        // work. A warm operation must not rewalk an uncharged aggregate graph.
        let shape = u64::try_from(self.decoded_cells(types, left.pointee)?)
            .map_err(|_| Error::Limit(LimitKind::Fuel))?;
        let mut work = u64::try_from(self.value_cells())
            .map_err(|_| Error::Limit(LimitKind::Fuel))?
            .checked_mul(2)
            .ok_or(Error::Limit(LimitKind::Fuel))?;
        // Snapshot copies these tables even when aliases have left their images.
        // Hash-table capacity bounds its scan work; tree entries have fixed size.
        for entries in [
            self.allocations.capacity(),
            self.handle_tokens.borrow().values.capacity(),
            self.virtual_regions.len(),
            self.runtime_types.capacity(),
            self.pool_ledger.table_capacity(),
        ] {
            work = work
                .checked_add(u64::try_from(entries).map_err(|_| Error::Limit(LimitKind::Fuel))?)
                .ok_or(Error::Limit(LimitKind::Fuel))?;
        }
        for pointer in [left, right] {
            let allocation = self.allocation(pointer)?;
            let bytes = self.layout(types, allocation.ty)?.size;
            let cells =
                u64::try_from(allocation.cells.get()).map_err(|_| Error::Limit(LimitKind::Fuel))?;
            work = work
                .checked_add(bytes)
                .and_then(|work| work.checked_add(cells))
                .and_then(|work| work.checked_add(shape))
                .ok_or(Error::Limit(LimitKind::Fuel))?;
        }
        Ok(work)
    }

    /// Matching addresses are valid; distinct nonempty regions must be disjoint.
    pub fn swap_values(
        &mut self,
        types: &dyn TypeView,
        left: &Pointer,
        right: &Pointer,
    ) -> Result<(), Error> {
        if left.pointee != right.pointee {
            return Err(Error::TypeMismatch {
                expected: left.pointee,
            });
        }
        let size = self.layout(types, left.pointee)?.size;
        let size = usize::try_from(size).map_err(|_| Error::Limit(LimitKind::ValueCells))?;
        let left_range = self.intrinsic_range(types, left, size, true)?;
        let right_range = self.intrinsic_range(types, right, size, true)?;
        let same_address = self.same_address(types, left, right)?;
        if !same_address
            && left.allocation_id() == right.allocation_id()
            && left_range.start < right_range.end
            && right_range.start < left_range.end
        {
            return Err(Error::InvalidIr(
                "swap requires identical or nonoverlapping regions",
            ));
        }
        let first = self.load(types, left)?;
        let second = self.load(types, right)?;
        if same_address {
            return Ok(());
        }
        let snapshot = self.snapshot();
        let result = self
            .store(types, left, second)
            .and_then(|()| self.store(types, right, first));
        if result.is_err() {
            self.restore(snapshot);
        }
        result
    }
}

#[cfg(test)]
#[path = "runtime_swap_tests.rs"]
mod tests;
