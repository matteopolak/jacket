use super::*;
impl Memory {
    pub(crate) fn sequence_snapshot(
        &self,
        types: &dyn TypeView,
        source: &Pointer,
        bytes: usize,
    ) -> Result<ByteImage, Error> {
        if bytes == 0 {
            return ByteImage::from_bytes(self.target, vec![], self.limits.value_cells);
        }
        let range = self.intrinsic_range(types, source, bytes, false)?;
        let allocation = self.allocation(source)?;
        self.ensure_image(types, allocation)?;
        let image = allocation.image.borrow();
        image
            .as_ref()
            .ok_or(Error::Uninitialized)?
            .extract_range(range.start, bytes)
    }
    pub(crate) fn install_sequence_buffer(
        &mut self,
        types: &dyn TypeView,
        root: &Pointer,
        mut image: ByteImage,
    ) -> Result<(), Error> {
        let allocation = self.allocation(root)?;
        if !root.data()?.path.is_empty() || allocation.readonly || image.target() != self.target {
            return Err(Error::InvalidIr(
                "sequence buffer installation requires mutable target storage",
            ));
        }
        if image.len() as u64 > allocation.virtual_extent {
            return Err(Error::OutOfBounds {
                index: image.len(),
                length: allocation.virtual_extent as usize,
            });
        }
        self.retokenize_image(types, &mut image)?;
        self.install_intrinsic_image(root, image)
    }
}
