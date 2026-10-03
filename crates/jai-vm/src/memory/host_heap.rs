//! Byte-backed heap storage and proof-consuming release, private to VM adapters.
use super::*;
use crate::virtual_heap::RetiredHeapAllocation;
impl Memory {
    pub(crate) fn preflight_host_heap_allocation(&self, size: usize) -> Result<(), Error> {
        if self.allocations.len() >= self.limits.allocations {
            return Err(Error::Limit(LimitKind::Allocations));
        }
        // The immutable backing-length value contributes one cell in addition
        // to bytes. Reject before creating either zero backing or byte image.
        self.cells
            .get()
            .checked_add(size)
            .and_then(|n| n.checked_add(1))
            .filter(|n| *n <= self.limits.value_cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        Ok(())
    }
    pub(crate) fn validate_host_heap_root(
        &self,
        types: &dyn TypeView,
        root: &Pointer,
    ) -> Result<(), Error> {
        let allocation = self.allocation(root)?;
        if !root.data()?.path.is_empty()
            || root.data()?.region.is_some()
            || allocation.readonly
            || root.pointee != allocation.ty
            || !matches!(types.kind(allocation.ty)?, TypeKind::String)
            || !matches!(&allocation.value, Some(Value::String(_)))
            || allocation.image.borrow().is_none()
        {
            return Err(Error::InvalidIr(
                "heap proof requires mutable canonical byte backing",
            ));
        }
        Ok(())
    }
    pub(crate) fn allocate_host_heap(
        &mut self,
        types: &dyn TypeView,
        size: usize,
    ) -> Result<Pointer, Error> {
        self.preflight_host_heap_allocation(size)?;
        let image = ByteImage::uninitialized(self.target, size, self.limits.value_cells)?;
        let byte = types.scalar(ScalarType::Int(IntegerType::U8));
        let root = self.allocate_sequence_buffer(types, byte, size)?;
        if let Err(error) = self.install_intrinsic_image(&root, image) {
            self.release(&root)?;
            return Err(error);
        }
        Ok(root)
    }
    pub(crate) fn host_heap_work_cost(&self, root: &Pointer) -> Result<u64, Error> {
        let allocation = self.allocation(root)?;
        allocation
            .virtual_extent
            .checked_add(
                u64::try_from(allocation.cells.get()).map_err(|_| Error::Limit(LimitKind::Fuel))?,
            )
            .and_then(|n| n.checked_mul(4))
            .and_then(|n| n.checked_add(1))
            .ok_or(Error::Limit(LimitKind::Fuel))
    }
    pub(crate) fn copy_host_heap(
        &mut self,
        types: &dyn TypeView,
        source: &Pointer,
        destination: &Pointer,
        count: usize,
    ) -> Result<(), Error> {
        self.validate_host_heap_root(types, source)?;
        self.validate_host_heap_root(types, destination)?;
        // Borrow metadata before any full image clone or range capture. Copying
        // a complete handle can add relocation/provenance cells to the new root.
        let source_allocation = self.allocation(source)?;
        let destination_allocation = self.allocation(destination)?;
        let metadata = source_allocation
            .image
            .borrow()
            .as_ref()
            .ok_or(Error::Uninitialized)?
            .range_metadata_work(0, count)?;
        let image_cells = usize::try_from(destination_allocation.virtual_extent)
            .ok()
            .and_then(|n| n.checked_add(metadata))
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let replacement_cells = destination_allocation.cells.get().max(image_cells);
        self.cells
            .get()
            .checked_sub(destination_allocation.cells.get())
            .and_then(|n| n.checked_add(replacement_cells))
            .filter(|n| *n <= self.limits.value_cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let source_image = self.image_for(types, self.allocation(source)?)?;
        let mut image = self.image_for(types, self.allocation(destination)?)?;
        image.copy_range_from(&source_image, 0, 0, count)?;
        self.install_intrinsic_image(destination, image)
    }
    pub(crate) fn release_host_heap(&mut self, proof: RetiredHeapAllocation) -> Result<(), Error> {
        let root = proof.into_pointer();
        // The proof constructor is private to the heap ledger; public release
        // retains its existing restrictions for every other allocation kind.
        self.release(&root)
    }
}
