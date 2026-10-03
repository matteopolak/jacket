//! Account for complete root image and metadata work before range operations.
use super::*;

impl Memory {
    pub(crate) fn atomic_work_cost(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        value: TypeId,
    ) -> Result<u64, Error> {
        if pointer.pointee != value {
            return Err(Error::TypeMismatch { expected: value });
        }
        let size = self.layout(types, value)?.size;
        if !matches!(size, 1 | 2 | 4 | 8) {
            return Err(Error::InvalidIr(
                "compare_and_swap requires a 1, 2, 4 or 8 byte scalar",
            ));
        }
        self.intrinsic_work_cost(
            types,
            &[(pointer, true)],
            usize::try_from(size).map_err(|_| Error::Limit(LimitKind::Fuel))?,
        )
    }

    pub(crate) fn intrinsic_work_cost(
        &self,
        types: &dyn TypeView,
        pointers: &[(&Pointer, bool)],
        count: usize,
    ) -> Result<u64, Error> {
        if count == 0 {
            return Ok(0);
        }
        let mut work = u64::try_from(count)
            .map_err(|_| Error::Limit(LimitKind::Fuel))?
            .checked_add(
                u64::try_from(self.handle_tokens.borrow().values.capacity())
                    .map_err(|_| Error::Limit(LimitKind::Fuel))?,
            )
            .ok_or(Error::Limit(LimitKind::Fuel))?;
        for &(pointer, writable) in pointers {
            self.intrinsic_range(types, pointer, count, writable)?;
            let allocation = self.allocation(pointer)?;
            // Allocation certified this extent against the selected target. Its
            // cached cell charge covers initialized values and byte metadata.
            let cells =
                u64::try_from(allocation.cells.get()).map_err(|_| Error::Limit(LimitKind::Fuel))?;
            let root = allocation
                .virtual_extent
                .checked_add(cells)
                .and_then(|root| root.checked_mul(2))
                .ok_or(Error::Limit(LimitKind::Fuel))?;
            work = work
                .checked_add(root)
                .ok_or(Error::Limit(LimitKind::Fuel))?;
        }
        Ok(work)
    }
}
