use super::*;

impl Memory {
    /// Apply the scheduler's parked-branch quota without altering target policy
    /// or any other memory limits. Return the previous quota for restoration.
    pub(crate) fn replace_value_cell_limit(&mut self, limit: usize) -> Result<usize, Error> {
        if self.value_cells() > limit {
            return Err(Error::Limit(LimitKind::ValueCells));
        }
        Ok(std::mem::replace(&mut self.limits.value_cells, limit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_types::TypeRegistry;

    #[test]
    fn parked_quota_rejects_live_usage_then_restores_original_limit() {
        let types = TypeRegistry::new();
        let word = types.scalar(ScalarType::Int(IntegerType::U64));
        let limits = Limits::default();
        let mut memory = Memory::new(limits);
        let pointer = memory.allocate(&types, word, None).unwrap();
        let live = memory.value_cells();
        let target = memory.target();
        assert!(live > 0);
        assert_eq!(
            memory.replace_value_cell_limit(live - 1),
            Err(Error::Limit(LimitKind::ValueCells))
        );
        assert_eq!(memory.value_cell_limit(), limits.value_cells);
        assert_eq!(
            memory.replace_value_cell_limit(live),
            Ok(limits.value_cells)
        );
        assert!(matches!(
            memory.allocate(&types, word, None),
            Err(Error::Limit(LimitKind::ValueCells))
        ));
        assert_eq!(
            memory.replace_value_cell_limit(limits.value_cells),
            Ok(live)
        );
        memory.release(&pointer).unwrap();
        memory.allocate(&types, word, None).unwrap();
        assert_eq!(memory.target(), target);
        assert_eq!(memory.limits.fuel, limits.fuel);
        assert_eq!(memory.limits.allocations, limits.allocations);
    }
}
