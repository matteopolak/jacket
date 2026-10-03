//! Fixed-width numeric register transfers through validated virtual byte storage.
use super::*;

impl Memory {
    pub(crate) fn simd_read_bytes(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        length: usize,
    ) -> Result<Vec<u8>, Error> {
        validate_width(length)?;
        let range = self.intrinsic_range(types, pointer, length, false)?;
        let image = self.image_for(types, self.allocation(pointer)?)?;
        if image.range_has_provenance(range.start, length)? {
            return Err(Error::UnsupportedPointerOperation(
                "SIMD numeric loads cannot consume virtual address bytes",
            ));
        }
        Ok(image.read_range(range.start, length)?.to_vec())
    }

    #[cfg(test)]
    pub(crate) fn simd_write_bytes(
        &mut self,
        types: &dyn TypeView,
        pointer: &Pointer,
        bytes: &[u8],
    ) -> Result<(), Error> {
        self.simd_write_bytes_reserved(types, pointer, bytes, 0)
    }

    pub(crate) fn simd_write_bytes_reserved(
        &mut self,
        types: &dyn TypeView,
        pointer: &Pointer,
        bytes: &[u8],
        reserved: usize,
    ) -> Result<(), Error> {
        validate_width(bytes.len())?;
        let range = self.intrinsic_range(types, pointer, bytes.len(), true)?;
        let mut image = self.intrinsic_destination(types, pointer, &range)?;
        image.write_range(range.start, bytes)?;
        let allocation = self.allocation(pointer)?;
        self.image_cell_charge(allocation, &image)?
            .1
            .checked_add(reserved)
            .filter(|cells| *cells <= self.limits.value_cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        self.install_intrinsic_image(pointer, image)
    }
}

fn validate_width(length: usize) -> Result<(), Error> {
    if matches!(length, 16 | 32) {
        Ok(())
    } else {
        Err(Error::InvalidIr(
            "SIMD transfer width must be 128 or 256 bits",
        ))
    }
}

#[cfg(test)]
mod tests;
