//! Ownership proof for source *FILE values. No integer becomes a file capability.
use crate::{Error, Memory, Pointer, virtual_files::VirtualFileHandle};
use jai_types::{CastMode, RecordKind, TypeId, TypeKind, TypeView};
use std::collections::HashMap;
/// A ledger is scoped to one VM/evaluation. Every token owns distinct readonly,
/// uninitialized storage of the actual FILE type; its address is virtual only.
#[derive(Clone, Debug)]
pub struct FileTokens {
    file_type: TypeId,
    tokens: HashMap<Pointer, VirtualFileHandle>,
}
/// A closed token proof minted only by removing a live ledger member. Its private
/// constructor prevents the memory helper from accepting an arbitrary heap pointer.
#[derive(Debug)]
pub struct RetiredFileToken {
    pointer: Pointer,
    handle: VirtualFileHandle,
}
impl RetiredFileToken {
    pub fn pointer(&self) -> &Pointer {
        &self.pointer
    }
    pub fn handle(&self) -> VirtualFileHandle {
        self.handle
    }
    pub fn into_parts(self) -> (Pointer, VirtualFileHandle) {
        (self.pointer, self.handle)
    }
}
impl FileTokens {
    pub fn new(types: &dyn TypeView, file_type: TypeId) -> Result<Self, Error> {
        if !matches!(types.kind(file_type)?, TypeKind::Record(_))
            || types.record_definition(file_type)?.kind != RecordKind::Struct
        {
            return Err(Error::InvalidIr(
                "FILE token requires its nominal struct identity",
            ));
        }
        Ok(Self {
            file_type,
            tokens: HashMap::new(),
        })
    }
    /// Only the trusted ABI adapter calls mint after a real host open observation.
    pub fn mint(
        &mut self,
        types: &dyn TypeView,
        memory: &mut Memory,
        handle: VirtualFileHandle,
    ) -> Result<Pointer, Error> {
        if let Some(existing) = self.tokens.keys().next() {
            memory.cast_pointer(types, existing, self.file_type, CastMode::Checked)?;
        }
        let token = memory.allocate_opaque_host_token(types, self.file_type)?;
        self.tokens.insert(token.clone(), handle);
        Ok(token)
    }
    fn canonical(
        &self,
        types: &dyn TypeView,
        memory: &Memory,
        pointer: &Pointer,
    ) -> Result<&Pointer, Error> {
        if pointer.pointee() != self.file_type {
            return Err(Error::TypeMismatch {
                expected: self.file_type,
            });
        }
        // Validates the virtual owner and allocation lifetime even on exact key hits.
        memory.cast_pointer(types, pointer, self.file_type, CastMode::Checked)?;
        if let Some((token, _)) = self.tokens.get_key_value(pointer) {
            return Ok(token);
        }
        // Equivalent provenance-preserving casts may use another projection path.
        // This bounded ledger comparison never compares guessed numeric addresses.
        for token in self.tokens.keys() {
            if memory.same_address(types, token, pointer)? {
                return Ok(token);
            }
        }
        Err(Error::InvalidIr("pointer is not a live FILE capability"))
    }
    pub fn lookup(
        &self,
        types: &dyn TypeView,
        memory: &Memory,
        pointer: &Pointer,
    ) -> Result<VirtualFileHandle, Error> {
        let token = self.canonical(types, memory, pointer)?;
        Ok(self.tokens[token])
    }
    /// Remove only the proven token and return its canonical root for the adapter's
    /// trusted opaque-storage release. Ordinary source free remains prohibited.
    pub fn remove(
        &mut self,
        types: &dyn TypeView,
        memory: &Memory,
        pointer: &Pointer,
    ) -> Result<RetiredFileToken, Error> {
        let token = self.canonical(types, memory, pointer)?.clone();
        let handle = self.tokens.remove(&token).unwrap();
        Ok(RetiredFileToken {
            pointer: token,
            handle,
        })
    }
    pub fn live_tokens(&self) -> usize {
        self.tokens.len()
    }
    /// Live token ownership is copied together with Memory by a rollback checkpoint.
    pub(crate) fn snapshot_bounds(
        &self,
        charge: &mut impl FnMut(u64) -> Result<(), Error>,
    ) -> Result<usize, Error> {
        let mut cells = self
            .tokens
            .capacity()
            .checked_add(1)
            .ok_or(Error::Limit(crate::LimitKind::ValueCells))?;
        charge(u64::try_from(cells).map_err(|_| Error::Limit(crate::LimitKind::Fuel))?)?;
        for pointer in self.tokens.keys() {
            cells = cells
                .checked_add(1)
                .and_then(|cells| cells.checked_add(pointer.metadata_cells()))
                .ok_or(Error::Limit(crate::LimitKind::ValueCells))?;
        }
        Ok(cells)
    }
    pub(crate) fn closed_table_capacity(&self) -> Result<usize, Error> {
        if !self.tokens.is_empty() {
            return Err(Error::InvalidIr("unclosed FILE capability"));
        }
        Ok(self.tokens.capacity())
    }
    pub fn reset(&mut self) {
        self.tokens.clear();
    }
}
#[cfg(test)]
#[path = "file_tokens/tests.rs"]
mod tests;
