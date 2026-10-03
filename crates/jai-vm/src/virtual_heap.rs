//! VM-owned malloc storage. Foreign heap calls never reach a native allocator.
use crate::{Error, LimitKind, Memory, Pointer};
use jai_types::{CastMode, TypeKind, TypeView};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct HeapLimits {
    pub allocations: usize,
    pub bytes: usize,
}
impl Default for HeapLimits {
    fn default() -> Self {
        Self {
            allocations: 1024,
            bytes: 16 * 1024 * 1024,
        }
    }
}
#[derive(Clone, Debug)]
pub struct VirtualHeap {
    roots: HashMap<Pointer, usize>,
    bytes: usize,
    limits: HeapLimits,
}
impl Default for VirtualHeap {
    fn default() -> Self {
        Self::new(HeapLimits::default())
    }
}
/// Only the live heap ledger can mint this proof; ordinary allocation roots,
/// including FILE tokens and source frames, cannot authorize a foreign free.
#[derive(Debug)]
pub(crate) struct RetiredHeapAllocation {
    pointer: Pointer,
}
impl RetiredHeapAllocation {
    pub(crate) fn into_pointer(self) -> Pointer {
        self.pointer
    }
}
impl VirtualHeap {
    pub fn new(limits: HeapLimits) -> Self {
        Self {
            roots: HashMap::new(),
            bytes: 0,
            limits,
        }
    }
    pub fn allocation_count(&self) -> usize {
        self.roots.len()
    }
    pub fn live_bytes(&self) -> usize {
        self.bytes
    }
    /// Admit only the retained heap ledger for a scheduler-private branch.
    /// Allocation bytes and initialization/provenance images belong to Memory.
    pub(crate) fn fork_bounds(
        &self,
        charge: &mut impl FnMut(u64) -> Result<(), Error>,
    ) -> Result<usize, Error> {
        let mut cells = self
            .roots
            .capacity()
            .checked_add(1) // VirtualHeap header, including scalar size/limit state
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        // HashMap iteration visits retained buckets even after all entries have
        // been removed. Reject low fuel before inspecting any stored pointer.
        charge(u64::try_from(cells).map_err(|_| Error::Limit(LimitKind::Fuel))?)?;
        for root in self.roots.keys() {
            cells = cells
                .checked_add(root.metadata_cells())
                .ok_or(Error::Limit(LimitKind::ValueCells))?;
        }
        Ok(cells)
    }
    fn canonical<'a>(
        &'a self,
        memory: &Memory,
        types: &dyn TypeView,
        pointer: &Pointer,
    ) -> Result<&'a Pointer, Error> {
        memory.cast_pointer(types, pointer, pointer.pointee(), CastMode::Checked)?;
        // Identity first bounds the comparison to one allocation. Address equality
        // accepts provenance-preserving casts while rejecting shifted pointers.
        let root = self
            .roots
            .keys()
            .find(|root| {
                pointer.data_allocation_key().is_some()
                    && root.data_allocation_key() == pointer.data_allocation_key()
            })
            .ok_or(Error::InvalidIr("pointer is not owned by the virtual heap"))?;
        if !memory.same_address(types, root, pointer)? {
            return Err(Error::InvalidIr(
                "heap operation requires the allocation base",
            ));
        }
        memory.validate_host_heap_root(types, root)?;
        Ok(root)
    }
    fn check_size(&self, old: Option<usize>, size: usize) -> Result<(), Error> {
        if old.is_none() && self.roots.len() >= self.limits.allocations {
            return Err(Error::Limit(LimitKind::Allocations));
        }
        self.bytes
            .checked_sub(old.unwrap_or(0))
            .and_then(|n| n.checked_add(size))
            .filter(|n| *n <= self.limits.bytes)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        Ok(())
    }
    pub fn malloc_work_cost(&self, size: usize) -> Result<u64, Error> {
        self.check_size(None, size)?;
        u64::try_from(size)
            .ok()
            .and_then(|n| n.checked_mul(4))
            .and_then(|n| n.checked_add(1 + self.roots.len() as u64))
            .ok_or(Error::Limit(LimitKind::Fuel))
    }
    pub fn realloc_work_cost(
        &self,
        memory: &Memory,
        types: &dyn TypeView,
        pointer: &Pointer,
        size: usize,
    ) -> Result<u64, Error> {
        if pointer.is_null() {
            memory.preflight_host_heap_allocation(size)?;
            return self.malloc_work_cost(size);
        }
        let root = self.canonical(memory, types, pointer)?;
        self.check_size(Some(self.roots[root]), size)?;
        if size != 0 {
            memory.preflight_host_heap_allocation(size)?;
        }
        memory
            .host_heap_work_cost(root)?
            .checked_add(self.roots.len() as u64 + pointer.metadata_cells() as u64)
            .ok_or(Error::Limit(LimitKind::Fuel))?
            .checked_add(
                u64::try_from(size)
                    .map_err(|_| Error::Limit(LimitKind::Fuel))?
                    .checked_mul(4)
                    .ok_or(Error::Limit(LimitKind::Fuel))?,
            )
            .ok_or(Error::Limit(LimitKind::Fuel))
    }
    pub fn free_work_cost(
        &self,
        memory: &Memory,
        types: &dyn TypeView,
        pointer: &Pointer,
    ) -> Result<u64, Error> {
        if pointer.is_null() {
            return Ok(0);
        }
        let root = self.canonical(memory, types, pointer)?;
        memory
            .host_heap_work_cost(root)?
            .checked_add(self.roots.len() as u64 + pointer.metadata_cells() as u64)
            .ok_or(Error::Limit(LimitKind::Fuel))
    }
    pub fn malloc(
        &mut self,
        memory: &mut Memory,
        types: &dyn TypeView,
        size: usize,
    ) -> Result<Pointer, Error> {
        self.check_size(None, size)?;
        if let Some(root) = self.roots.keys().next() {
            memory.validate_host_heap_root(types, root)?;
        }
        let void = types
            .lookup(&TypeKind::Void)
            .ok_or(Error::InvalidIr("heap ABI void type is unavailable"))?;
        types.kind(void)?;
        let root = memory.allocate_host_heap(types, size)?;
        let result = memory.cast_pointer(types, &root, void, CastMode::Checked)?;
        self.roots.insert(root, size);
        self.bytes += size;
        Ok(result)
    }
    pub fn free(
        &mut self,
        memory: &mut Memory,
        types: &dyn TypeView,
        pointer: &Pointer,
    ) -> Result<(), Error> {
        if pointer.is_null() {
            return Ok(());
        }
        let root = self.canonical(memory, types, pointer)?.clone();
        let size = self.roots[&root];
        memory.release_host_heap(RetiredHeapAllocation {
            pointer: root.clone(),
        })?;
        self.roots.remove(&root);
        self.bytes -= size;
        Ok(())
    }
    pub fn realloc(
        &mut self,
        memory: &mut Memory,
        types: &dyn TypeView,
        pointer: &Pointer,
        size: usize,
    ) -> Result<Pointer, Error> {
        if pointer.is_null() {
            return self.malloc(memory, types, size);
        }
        let root = self.canonical(memory, types, pointer)?.clone();
        let void = types
            .lookup(&TypeKind::Void)
            .ok_or(Error::InvalidIr("heap ABI void type is unavailable"))?;
        types.kind(void)?;
        if size == 0 {
            self.free(memory, types, pointer)?;
            return Ok(Pointer::null(void));
        }
        let old = self.roots[&root];
        self.check_size(Some(old), size)?;
        let replacement = memory.allocate_host_heap(types, size)?;
        if let Err(error) = memory.copy_host_heap(types, &root, &replacement, old.min(size)) {
            // This fresh canonical root was never published or admitted to the
            // heap ledger. Internal staging cleanup uses ordinary root release;
            // a retired heap proof is reserved for exact live ledger members.
            memory.release(&replacement)?;
            return Err(error);
        }
        let result = memory.cast_pointer(types, &replacement, void, CastMode::Checked)?;
        memory.release_host_heap(RetiredHeapAllocation {
            pointer: root.clone(),
        })?;
        self.roots.remove(&root);
        self.roots.insert(replacement, size);
        self.bytes = self.bytes - old + size;
        Ok(result)
    }
}
#[cfg(test)]
#[path = "virtual_heap/tests.rs"]
mod tests;
