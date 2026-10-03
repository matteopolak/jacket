//! Private branch copies admit sparse tables as well as active capture payloads.
use super::*;

impl BindingEnvironment {
    /// Report conservative retained cells and clone work after a metered header walk.
    /// Active values already have cached shape charges. Spare nested vector slots
    /// are inspected without walking or cloning any captured value.
    pub(in crate::execute) fn fork_bounds(
        &self,
        charge: &mut impl FnMut(u64) -> Result<(), Error>,
    ) -> Result<(usize, usize), Error> {
        let headers = self
            .scopes
            .capacity()
            .checked_add(self.values.capacity())
            .and_then(|cells| cells.checked_add(1))
            .ok_or(Error::Limit(LimitKind::Fuel))?;
        charge(u64::try_from(headers).map_err(|_| Error::Limit(LimitKind::Fuel))?)?;
        let mut cells = self
            .cells
            .checked_add(headers)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        for scope in &self.scopes {
            cells = cells
                .checked_add(scope.bindings.capacity())
                .ok_or(Error::Limit(LimitKind::ValueCells))?;
        }
        for captures in self.values.values() {
            cells = cells
                .checked_add(captures.capacity())
                .ok_or(Error::Limit(LimitKind::ValueCells))?;
        }
        Ok((cells, self.fork_clone_work()?))
    }

    fn fork_clone_work(&self) -> Result<usize, Error> {
        // Each active capture is represented once in values, once in its scope's
        // ID vector, and once in the cached payload. Vector clones copy live
        // elements; sparse HashMap cloning additionally touches its capacity.
        self.cells
            .checked_mul(3)
            .and_then(|work| work.checked_add(self.scopes.capacity()))
            .and_then(|work| work.checked_add(self.values.capacity()))
            .and_then(|work| work.checked_add(1))
            .ok_or(Error::Limit(LimitKind::Fuel))
    }

    pub(in crate::execute) fn fork_private(&self, admitted_work: usize) -> Result<Self, Error> {
        if self.fork_clone_work()? > admitted_work {
            return Err(Error::Limit(LimitKind::Fuel));
        }
        // Token identity and scope serials intentionally survive in the isolated
        // copy: already-scheduled BindEnd tasks must still close their own scope.
        Ok(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_ir::ProcedureId;

    #[test]
    fn fork_preserves_scope_tokens_but_restoration_does_not_change_the_parent() {
        let mut parent = BindingEnvironment::new();
        let id = ExpressionBindingId::new(ProcedureId::new(0), 0);
        let outer = parent.begin();
        parent.insert(id, Value::Bool(false), 100).unwrap();
        let inner = parent.begin();
        parent
            .insert(id, Value::String(vec![1, 2, 3]), 100)
            .unwrap();
        let mut charged = 0;
        let (cells, work) = parent
            .fork_bounds(&mut |count| {
                charged += count;
                Ok(())
            })
            .unwrap();
        assert!(cells > parent.cells());
        assert!(charged > 0);
        assert!(matches!(
            parent.fork_private(work - 1),
            Err(Error::Limit(LimitKind::Fuel))
        ));
        let mut child = parent.fork_private(work).unwrap();
        child.end(inner).unwrap();
        assert_eq!(child.lookup(id), Ok(&Value::Bool(false)));
        assert_eq!(parent.lookup(id), Ok(&Value::String(vec![1, 2, 3])));
        child.end(outer).unwrap();
        assert_eq!(child.cells(), 0);
        assert!(parent.cells() > 0);
        parent.end(inner).unwrap();
        parent.end(outer).unwrap();
    }

    #[test]
    fn sparse_scope_capacity_is_retained_and_inspection_is_charged_first() {
        let mut environment = BindingEnvironment::new();
        let mut scopes = vec![];
        for _ in 0..16 {
            scopes.push(environment.begin());
        }
        for scope in scopes.into_iter().rev() {
            environment.end(scope).unwrap();
        }
        assert_eq!(environment.cells(), 0);
        assert!(matches!(
            environment.fork_bounds(&mut |_| Err(Error::Limit(LimitKind::Fuel))),
            Err(Error::Limit(LimitKind::Fuel))
        ));
        let (cells, _) = environment.fork_bounds(&mut |_| Ok(())).unwrap();
        assert!(cells >= 17);
    }
}
