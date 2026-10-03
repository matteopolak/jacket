//! Owned immutable captures, with exact lexical restoration and cached copy costs.
use crate::{Error, LimitKind, Value};
use jai_ir::ExpressionBindingId;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
mod fork;

static NEXT_ENVIRONMENT: AtomicU64 = AtomicU64::new(1);

/// A token belongs to one live scope in one environment; fields stay private.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ScopeToken {
    environment: u64,
    depth: usize,
    serial: u64,
}
#[derive(Clone)]
struct Scope {
    token: ScopeToken,
    bindings: Vec<ExpressionBindingId>,
}
#[derive(Clone)]
struct Capture {
    scope: u64,
    value: Value,
    cells: usize,
}

/// Values are moved into this ledger unchanged. Copies belong to the metered caller.
#[derive(Clone)]
pub(super) struct BindingEnvironment {
    identity: u64,
    next_scope: u64,
    scopes: Vec<Scope>,
    values: HashMap<ExpressionBindingId, Vec<Capture>>,
    cells: usize,
}
impl Default for BindingEnvironment {
    fn default() -> Self {
        Self::new()
    }
}
impl BindingEnvironment {
    pub(super) fn new() -> Self {
        let identity = NEXT_ENVIRONMENT
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |identity| {
                identity.checked_add(1)
            })
            .expect("expression binding environment identities exhausted");
        Self {
            identity,
            next_scope: 1,
            scopes: vec![],
            values: HashMap::new(),
            cells: 0,
        }
    }
    pub(super) fn begin(&mut self) -> ScopeToken {
        let serial = self.next_scope;
        self.next_scope = serial
            .checked_add(1)
            .expect("expression binding scope identities exhausted");
        let token = ScopeToken {
            environment: self.identity,
            depth: self.scopes.len() + 1,
            serial,
        };
        self.scopes.push(Scope {
            token,
            bindings: vec![],
        });
        self.cells = self
            .cells
            .checked_add(1)
            .expect("binding scope header was admitted by the caller");
        token
    }
    /// `available_cells` is additional capacity, after existing retained captures.
    /// Return the complete value plus entry charge; later clones charge only the value.
    pub(super) fn insert(
        &mut self,
        id: ExpressionBindingId,
        value: Value,
        available_cells: usize,
    ) -> Result<usize, Error> {
        let scope = self
            .scopes
            .last()
            .ok_or(Error::InvalidIr("expression binding without a scope"))?;
        if self
            .values
            .get(&id)
            .and_then(|values| values.last())
            .is_some_and(|value| value.scope == scope.token.serial)
        {
            return Err(Error::InvalidIr(
                "duplicate expression binding in one scope",
            ));
        }
        // Admission happens before map/vector growth or moving the value into storage.
        let value_limit = available_cells
            .checked_sub(1)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let cells = value.cells(value_limit)?;
        let entry_cells = cells
            .checked_add(1)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let total = self
            .cells
            .checked_add(entry_cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        self.values.entry(id).or_default().push(Capture {
            scope: scope.token.serial,
            value,
            cells,
        });
        self.scopes
            .last_mut()
            .expect("scope was checked")
            .bindings
            .push(id);
        self.cells = total;
        Ok(entry_cells)
    }
    pub(super) fn lookup(&self, id: ExpressionBindingId) -> Result<&Value, Error> {
        Ok(&self.capture(id)?.value)
    }
    /// This cached charge includes bytes, projections, Number origins and storage carriers.
    pub(super) fn clone_charge(&self, id: ExpressionBindingId) -> Result<usize, Error> {
        Ok(self.capture(id)?.cells)
    }
    fn capture(&self, id: ExpressionBindingId) -> Result<&Capture, Error> {
        self.values
            .get(&id)
            .and_then(|values| values.last())
            .ok_or(Error::InvalidIr("unbound expression capture"))
    }
    pub(super) fn end(&mut self, token: ScopeToken) -> Result<(), Error> {
        if self.scopes.last().is_none_or(|scope| scope.token != token) {
            return Err(Error::InvalidIr(
                "expression binding scopes must end in exact LIFO order",
            ));
        }
        let scope = self.scopes.pop().expect("scope was checked");
        for id in scope.bindings.into_iter().rev() {
            let values = self
                .values
                .get_mut(&id)
                .expect("scope owns its installed capture");
            let capture = values.pop().expect("scope owns its installed capture");
            self.cells -= capture.cells + 1;
            if values.is_empty() {
                self.values.remove(&id);
            }
        }
        self.cells -= 1;
        Ok(())
    }
    pub(super) fn depth(&self) -> usize {
        self.scopes.len()
    }
    pub(super) fn cells(&self) -> usize {
        self.cells
    }
    pub(super) fn clear(&mut self) {
        self.scopes.clear();
        self.values.clear();
        self.cells = 0;
        // Keep scope identities monotonic so a token from a cancelled scope stays invalid.
    }
}

#[cfg(test)]
mod tests;
