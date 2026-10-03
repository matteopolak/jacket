//! Bounded byte operations over virtual allocation images.
use super::*;

#[path = "runtime_swap.rs"]
mod swap;
#[path = "runtime_work.rs"]
mod work;

impl Memory {
    pub(super) fn intrinsic_range(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        count: usize,
        writable: bool,
    ) -> Result<std::ops::Range<usize>, Error> {
        if count > self.limits.value_cells {
            return Err(Error::Limit(LimitKind::ValueCells));
        }
        self.validate_pointer(types, pointer)?;
        let allocation = self.allocation(pointer)?;
        if writable && allocation.readonly {
            return Err(Error::ReadOnlyStorage);
        }
        let offset = self.byte_offset(types, pointer)?;
        let (start, extent) = self.region(types, pointer)?;
        let end = offset
            .checked_add(u64::try_from(count).map_err(|_| Error::CheckedCast)?)
            .ok_or(Error::CheckedCast)?;
        if offset < start || end > extent {
            return Err(Error::OutOfBounds {
                index: usize::try_from(end).unwrap_or(usize::MAX),
                length: usize::try_from(extent.saturating_sub(start)).unwrap_or(usize::MAX),
            });
        }
        Ok(usize::try_from(offset).map_err(|_| Error::CheckedCast)?
            ..usize::try_from(end).map_err(|_| Error::CheckedCast)?)
    }

    pub(super) fn intrinsic_destination(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        range: &std::ops::Range<usize>,
    ) -> Result<ByteImage, Error> {
        let allocation = self.allocation(pointer)?;
        if allocation.value.is_some() || allocation.image.borrow().is_some() {
            return self.image_for(types, allocation);
        }
        let length = usize::try_from(self.storage_length(types, pointer)?)
            .map_err(|_| Error::Limit(LimitKind::ValueCells))?;
        // A full write initializes all bytes. Partial writes must not silently
        // manufacture initialized bytes for the rest of an uninitialized object.
        if range.start != 0 || range.end != length {
            return Err(Error::Uninitialized);
        }
        if length > self.limits.value_cells {
            return Err(Error::Limit(LimitKind::ValueCells));
        }
        ByteImage::from_bytes(self.target, vec![0; length], self.limits.value_cells)
    }

    pub(super) fn install_intrinsic_image(
        &mut self,
        pointer: &Pointer,
        image: ByteImage,
    ) -> Result<(), Error> {
        let allocation = self.allocation(pointer)?;
        let (cells, total) = self.image_cell_charge(allocation, &image)?;
        let allocation = self
            .allocations
            .get_mut(&pointer.allocation_id())
            .ok_or(Error::DanglingPointer)?;
        *allocation.image.borrow_mut() = Some(image);
        allocation.cells.set(cells);
        self.cells.set(total);
        Ok(())
    }

    /// `memcpy` requires disjoint regions; rejected operations leave storage unchanged.
    /// A zero-byte operation does not inspect the pointers.
    pub fn byte_copy(
        &mut self,
        types: &dyn TypeView,
        destination: &Pointer,
        source: &Pointer,
        count: usize,
    ) -> Result<(), Error> {
        if count == 0 {
            return Ok(());
        }
        let destination_range = self.intrinsic_range(types, destination, count, true)?;
        let source_range = self.intrinsic_range(types, source, count, false)?;
        if destination.allocation_id() == source.allocation_id()
            && destination_range.start < source_range.end
            && source_range.start < destination_range.end
        {
            return Err(Error::InvalidIr(
                "memcpy requires nonoverlapping byte ranges",
            ));
        }
        let source_image = self.image_for(types, self.allocation(source)?)?;
        let mut image = self.intrinsic_destination(types, destination, &destination_range)?;
        image.copy_range_from(
            &source_image,
            source_range.start,
            destination_range.start,
            count,
        )?;
        self.install_intrinsic_image(destination, image)
    }

    /// Lexicographic comparison of unsigned bytes; only the sign is part of the contract.
    pub fn byte_compare(
        &self,
        types: &dyn TypeView,
        left: &Pointer,
        right: &Pointer,
        count: usize,
    ) -> Result<i16, Error> {
        if count == 0 {
            return Ok(0);
        }
        let left_range = self.intrinsic_range(types, left, count, false)?;
        let right_range = self.intrinsic_range(types, right, count, false)?;
        let left = self.image_for(types, self.allocation(left)?)?;
        let right = self.image_for(types, self.allocation(right)?)?;
        if left.range_has_provenance(left_range.start, count)?
            || right.range_has_provenance(right_range.start, count)?
        {
            let equal = left.range_provenance_equivalent(
                &right,
                left_range.start,
                right_range.start,
                count,
                |left, right| atomic_equal(self, types, left, right),
            )?;
            return if equal {
                Ok(0)
            } else {
                Err(Error::UnsupportedPointerOperation(
                    "memcmp of distinct address-dependent bytes has no target-independent result",
                ))
            };
        }
        for (&left, &right) in left
            .read_range(left_range.start, count)?
            .iter()
            .zip(right.read_range(right_range.start, count)?)
        {
            if left != right {
                return Ok(i16::from(left) - i16::from(right));
            }
        }
        Ok(0)
    }

    /// Filling bytes discards overlapping virtual-handle provenance.
    pub fn byte_set(
        &mut self,
        types: &dyn TypeView,
        destination: &Pointer,
        byte: u8,
        count: usize,
    ) -> Result<(), Error> {
        self.byte_set_number(
            types,
            destination,
            &crate::Number::plain(Integer::wrapping(IntegerType::U8, i128::from(byte))),
            count,
        )
    }

    /// Address-derived bytes retain their provenance through filling and later copies.
    pub fn byte_set_number(
        &mut self,
        types: &dyn TypeView,
        destination: &Pointer,
        byte: &crate::Number,
        count: usize,
    ) -> Result<(), Error> {
        if byte.ty() != IntegerType::U8 {
            return Err(Error::InvalidIr("memset requires a u8 byte"));
        }
        if count == 0 {
            return Ok(());
        }
        let range = self.intrinsic_range(types, destination, count, true)?;
        let mut image = self.intrinsic_destination(types, destination, &range)?;
        image.fill_range_number(range.start, count, byte.clone())?;
        self.install_intrinsic_image(destination, image)
    }

    /// The isolated VM is single threaded. The read/compare/store is one VM operation;
    /// native lowering supplies sequentially consistent atomicity between threads.
    pub fn compare_and_swap(
        &mut self,
        types: &dyn TypeView,
        pointer: &Pointer,
        expected: &Value,
        replacement: &Value,
    ) -> Result<(bool, Value), Error> {
        jai_ir::atomic_scalar(types, self.target.policy, pointer.pointee).map_err(Error::from)?;
        expected.validate(types, pointer.pointee, self.limits.evaluation_depth)?;
        replacement.validate(types, pointer.pointee, self.limits.evaluation_depth)?;
        validate_atomic_number(expected)?;
        validate_atomic_number(replacement)?;
        let layout = self.layout(types, pointer.pointee)?;
        let range = self.intrinsic_range(
            types,
            pointer,
            usize::try_from(layout.size).map_err(|_| Error::CheckedCast)?,
            true,
        )?;
        let root_alignment = self.storage_alignment(pointer)?;
        if !(range.start as u128).is_multiple_of(u128::from(layout.size))
            || u64::from(root_alignment) < layout.size
        {
            return Err(Error::InvalidIr(
                "compare_and_swap requires natural atomic alignment",
            ));
        }
        let observed = self.load(types, pointer)?;
        validate_atomic_number(&observed)?;
        let success = atomic_equal(self, types, &observed, expected)?;
        if success {
            self.store(types, pointer, replacement.clone())?;
        }
        Ok((success, observed))
    }
}

fn validate_atomic_number(value: &Value) -> Result<(), Error> {
    match value {
        Value::AddressInteger(_) => Err(Error::UnsupportedPointerOperation(
            "compare_and_swap on address-derived integers has no target-independent comparison",
        )),
        Value::Distinct {
            value, ..
        } => validate_atomic_number(value),
        _ => Ok(()),
    }
}

fn atomic_equal(
    memory: &Memory,
    types: &dyn TypeView,
    left: &Value,
    right: &Value,
) -> Result<bool, Error> {
    match (left, right) {
        (Value::Pointer(left), Value::Pointer(right)) => memory.same_address(types, left, right),
        (
            Value::Distinct {
                ty: left_ty,
                value: left,
            },
            Value::Distinct {
                ty: right_ty,
                value: right,
            },
        ) if left_ty == right_ty => atomic_equal(memory, types, left, right),
        _ => Ok(left == right),
    }
}

#[cfg(test)]
#[path = "runtime_intrinsics_tests.rs"]
mod tests;
