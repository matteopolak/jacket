use super::*;
use crate::{AddressProvenance, Number};

impl Memory {
    fn pointer_bits(&self) -> Result<u32, Error> {
        let size = self.target.policy.pointer().size;
        if !matches!(size, 1 | 2 | 4 | 8) {
            return Err(Error::UnsupportedPointerOperation(
                "target pointer width exceeds the virtual address representation",
            ));
        }
        Ok(size as u32 * 8)
    }
    pub(super) fn reserve_virtual_region(&self, extent: u64, alignment: u64) -> Result<u64, Error> {
        let bits = self.pointer_bits()?;
        let max = u64::MAX >> (64 - bits);
        let base = self
            .next_virtual_address
            .get()
            .checked_add(alignment - 1)
            .map(|value| value & !(alignment - 1))
            .ok_or(Error::Limit(LimitKind::Allocations))?;
        // A gap keeps an allocation's one-past address distinct from the next root.
        let next = base
            .checked_add(extent.max(1))
            .and_then(|end| end.checked_add(alignment))
            .filter(|next| *next <= max)
            .ok_or(Error::Limit(LimitKind::Allocations))?;
        self.next_virtual_address.set(next);
        Ok(base)
    }
    /// Target-width virtual bytes, independent of the Rust host and never recycled.
    pub fn pointer_to_integer(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        target: IntegerType,
        mode: CastMode,
    ) -> Result<Number, Error> {
        if matches!(mode, CastMode::Force(_)) {
            return Err(Error::InvalidIr("force requires a checked storage bitcast"));
        }
        let bits = self.pointer_bits()?;
        if pointer.is_null() {
            types.kind(pointer.pointee)?;
            return Ok(Number::plain(Integer::wrapping(target, 0)));
        }
        if let Some((address, width)) = pointer.opaque_address_bits() {
            self.validate_opaque_address(types, pointer)?;
            let result = Integer::wrapping(target, i128::from(address));
            if mode == CastMode::Checked
                && target.bits() < width
                && (result.bits() != address || (target.signed() && result.value() < 0))
            {
                return Err(Error::CheckedCast);
            }
            return Ok(Number::plain(result));
        }
        if let Some(code) = pointer.code_pointer() {
            return Ok(Number::address(
                self.code_pointer_integer(types, code, target)?,
                AddressProvenance::Pointer(pointer.clone()),
            ));
        }
        if target.bits() < bits {
            return Err(Error::UnsupportedPointerOperation(
                "nonnull pointer cast to a narrower integer cannot model native address range",
            ));
        }
        self.validate_pointer(types, pointer)?;
        let allocation = self.allocation(pointer)?;
        let offset = self.byte_offset(types, pointer)?;
        if offset > allocation.virtual_extent {
            return Err(Error::OutOfBounds {
                index: usize::try_from(offset).unwrap_or(usize::MAX),
                length: usize::try_from(allocation.virtual_extent).unwrap_or(usize::MAX),
            });
        }
        let address = allocation
            .virtual_base
            .checked_add(offset)
            .ok_or(Error::CheckedCast)?;
        Ok(Number::address(
            Integer::wrapping(target, i128::from(address)),
            AddressProvenance::Pointer(pointer.clone()),
        ))
    }
    /// Inverse casts only recover allocation provenance already exposed by a cast.
    pub fn integer_to_pointer(
        &self,
        types: &dyn TypeView,
        integer: Number,
        pointee: TypeId,
        mode: CastMode,
    ) -> Result<Pointer, Error> {
        if matches!(mode, CastMode::Force(_)) {
            return Err(Error::InvalidIr("force requires a checked storage bitcast"));
        }
        types.kind(pointee)?;
        let bits = self.pointer_bits()?;
        let max = u64::MAX >> (64 - bits);
        let address = if integer.ty().bits() == bits {
            integer.bits()
        } else if integer.ty().bits() < bits {
            // The source's signedness controls extension, as it does in native code.
            integer.value() as u64 & max
        } else {
            if mode == CastMode::Checked
                && (integer.value() < 0 || integer.value() > i128::from(max))
            {
                return Err(Error::CheckedCast);
            }
            integer.bits() & max
        };
        let origin = match integer.provenance() {
            None if address == 0 => return Ok(Pointer::null(pointee)),
            Some(AddressProvenance::Pointer(pointer)) => pointer,
            Some(AddressProvenance::Derived {
                ..
            }) => {
                return Err(Error::UnsupportedPointerOperation(
                    "transformed address integer has no proven affine pointer identity",
                ));
            }
            None => return Pointer::opaque(address, bits, pointee),
        };
        if let Some(code) = origin.code_pointer() {
            let code = self.code_pointer_from_integer(types, code, integer.integer(), mode)?;
            return Ok(Pointer::from_code(code, pointee));
        }
        let allocation = self.allocation(origin)?;
        let expected = allocation
            .virtual_base
            .checked_add(self.byte_offset(types, origin)?)
            .ok_or(Error::CheckedCast)?;
        if address != expected {
            return Err(Error::UnsupportedPointerOperation(
                "integer bits no longer match their proven pointer identity",
            ));
        }
        self.cast_pointer(types, origin, pointee, mode)
    }
    pub(crate) fn affine_address_number(
        &self,
        types: &dyn TypeView,
        number: &Number,
        origin: &Pointer,
        delta: i128,
    ) -> Result<Option<Number>, Error> {
        if let Some(code) = origin.code_pointer() {
            self.validate_code_pointer(types, code)?;
            return if delta == 0 && number.bits() == code.token() {
                Ok(Some(Number::address(
                    number.integer(),
                    AddressProvenance::Pointer(origin.clone()),
                )))
            } else {
                Ok(None)
            };
        }
        if number.ty().bits() < self.pointer_bits()? {
            return Ok(None);
        }
        let allocation = self.allocation(origin)?;
        let offset = i128::from(self.byte_offset(types, origin)?)
            .checked_add(delta)
            .and_then(|offset| u64::try_from(offset).ok());
        let Some(offset) = offset else {
            return Ok(None);
        };
        let (start, end) = self.region(types, origin)?;
        if offset < start || offset > end {
            return Ok(None);
        }
        let address = allocation
            .virtual_base
            .checked_add(offset)
            .ok_or(Error::CheckedCast)?;
        if number.bits() != address {
            return Ok(None);
        }
        let mut pointer = origin.clone();
        pointer.data_mut()?.path = vec![Projection::Bytes {
            offset,
            ty: pointer.pointee,
        }];
        Ok(Some(Number::address(
            number.integer(),
            AddressProvenance::Pointer(pointer),
        )))
    }
    pub(crate) fn independent_address_difference(
        &self,
        types: &dyn TypeView,
        left: &Pointer,
        right: &Pointer,
    ) -> Result<Option<i128>, Error> {
        if left.code_pointer().is_some() || right.code_pointer().is_some() {
            if let Some(code) = left.code_pointer() {
                self.validate_code_pointer(types, code)?;
            }
            if let Some(code) = right.code_pointer() {
                self.validate_code_pointer(types, code)?;
            }
            return Ok((left.code_pointer().is_some()
                && left.code_pointer() == right.code_pointer())
            .then_some(0));
        }
        self.allocation(left)?;
        self.allocation(right)?;
        if left.memory_identity() != right.memory_identity()
            || left.allocation_id() != right.allocation_id()
        {
            return Ok(None);
        }
        Ok(Some(
            i128::from(self.byte_offset(types, left)?)
                - i128::from(self.byte_offset(types, right)?),
        ))
    }
    pub(crate) fn independent_address_alignment(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        modulus: u64,
    ) -> Result<Option<u64>, Error> {
        if let Some(code) = pointer.code_pointer() {
            self.validate_code_pointer(types, code)?;
            return Ok(None);
        }
        let allocation = self.allocation(pointer)?;
        if !modulus.is_power_of_two() || u64::from(allocation.virtual_alignment) < modulus {
            return Ok(None);
        }
        Ok(Some(self.byte_offset(types, pointer)? % modulus))
    }
}

#[cfg(test)]
#[path = "addresses_tests.rs"]
mod tests;
