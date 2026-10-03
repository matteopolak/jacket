//! Numeric pointer values have a target-width bit domain and no storage issuer.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum OpaqueAddress {
    Bits32(u32),
    Bits64(u64),
}
impl OpaqueAddress {
    fn bits(self) -> (u64, u32) {
        match self {
            Self::Bits32(bits) => (u64::from(bits), 32),
            Self::Bits64(bits) => (bits, 64),
        }
    }
}
impl Pointer {
    /// Numeric bits carry no allocation or code issuer.
    pub fn opaque_address_bits(&self) -> Option<(u64, u32)> {
        match self.origin {
            PointerOrigin::Opaque(address) => Some(address.bits()),
            _ => None,
        }
    }
    pub fn is_opaque(&self) -> bool {
        matches!(self.origin, PointerOrigin::Opaque(_))
    }
    pub(crate) fn opaque(bits: u64, width: u32, pointee: TypeId) -> Result<Self, Error> {
        let address = match width {
            32 => OpaqueAddress::Bits32(u32::try_from(bits).map_err(|_| Error::CheckedCast)?),
            64 => OpaqueAddress::Bits64(bits),
            _ => {
                return Err(Error::UnsupportedPointerOperation(
                    "numeric pointer requires a 32-bit or 64-bit language target",
                ));
            }
        };
        let mut pointer = Self::null(pointee);
        if bits != 0 {
            pointer.origin = PointerOrigin::Opaque(address);
        }
        Ok(pointer)
    }
}
impl Memory {
    pub(super) fn validate_opaque_address(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
    ) -> Result<(), Error> {
        let (_, width) = pointer
            .opaque_address_bits()
            .ok_or(Error::InvalidIr("expected opaque numeric pointer"))?;
        types.kind(pointer.pointee)?;
        if u64::from(width) != self.target.policy.pointer().size * 8 {
            return Err(Error::UnsupportedPointerOperation(
                "numeric pointer belongs to another target width",
            ));
        }
        Ok(())
    }
    pub(super) fn comparable_pointer_bits(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
    ) -> Result<u64, Error> {
        if let Some((bits, _)) = pointer.opaque_address_bits() {
            self.validate_opaque_address(types, pointer)?;
            return Ok(bits);
        }
        Ok(self
            .pointer_to_integer(types, pointer, IntegerType::U64, CastMode::Unchecked)?
            .bits())
    }
}
#[cfg(test)]
mod tests;
