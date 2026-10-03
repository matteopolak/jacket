//! Opaque FILE storage lifecycle. Only a retired ledger proof authorizes release.
use super::*;
use crate::file_tokens::RetiredFileToken;
impl Memory {
    pub(crate) fn allocate_opaque_host_token(
        &mut self,
        types: &dyn TypeView,
        file_type: TypeId,
    ) -> Result<Pointer, Error> {
        if !matches!(types.kind(file_type)?, TypeKind::Record(_))
            || types.record_definition(file_type)?.kind != jai_types::RecordKind::Struct
        {
            return Err(Error::InvalidIr(
                "opaque FILE storage requires the actual nominal struct",
            ));
        }
        let pointer = self.allocate(types, file_type, None)?;
        if let Err(error) = self.freeze(&pointer) {
            let _ = self.release(&pointer);
            return Err(error);
        }
        Ok(pointer)
    }
    pub(crate) fn release_opaque_host_token(
        &mut self,
        proof: RetiredFileToken,
    ) -> Result<(), Error> {
        let (pointer, _) = proof.into_parts();
        let allocation = self.allocation(&pointer)?;
        if !pointer.data()?.path.is_empty()
            || pointer.data()?.region.is_some()
            || pointer.pointee != allocation.ty
            || !allocation.readonly
            || allocation.value.is_some()
            || allocation.image.borrow().is_some()
        {
            return Err(Error::InvalidIr(
                "retired FILE proof does not own an opaque allocation root",
            ));
        }
        let allocation = self
            .allocations
            .remove(&pointer.allocation_id())
            .ok_or(Error::DanglingPointer)?;
        self.cells.set(self.cells.get() - allocation.cells.get());
        self.virtual_regions.remove(&allocation.virtual_base);
        self.runtime_types
            .retain(|(id, _), _| *id != pointer.allocation_id());
        Ok(())
    }
}
impl Memory {
    pub(crate) fn host_read_bytes(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        count: usize,
    ) -> Result<Vec<u8>, Error> {
        if count == 0 {
            return Ok(Vec::new());
        }
        let range = self.intrinsic_range(types, pointer, count, false)?;
        let image = self.image_for(types, self.allocation(pointer)?)?;
        if image.range_has_provenance(range.start, count)? {
            return Err(Error::UnsupportedPointerOperation(
                "native file bytes cannot encode virtual handles",
            ));
        }
        Ok(image.read_range(range.start, count)?.to_vec())
    }
    pub(crate) fn host_write_bytes(
        &mut self,
        types: &dyn TypeView,
        pointer: &Pointer,
        bytes: &[u8],
    ) -> Result<(), Error> {
        if bytes.is_empty() {
            return Ok(());
        }
        let range = self.intrinsic_range(types, pointer, bytes.len(), true)?;
        let allocation = self.allocation(pointer)?;
        let mut image = if allocation.value.is_some() || allocation.image.borrow().is_some() {
            self.image_for(types, allocation)?
        } else {
            let length = usize::try_from(self.storage_length(types, pointer)?)
                .map_err(|_| Error::Limit(LimitKind::ValueCells))?;
            ByteImage::uninitialized(self.target, length, self.limits.value_cells)?
        };
        image.write_range(range.start, bytes)?;
        self.install_intrinsic_image(pointer, image)
    }
    pub(crate) fn host_c_string_cost(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        maximum: usize,
    ) -> Result<u64, Error> {
        self.validate_pointer(types, pointer)?;
        let offset = self.byte_offset(types, pointer)?;
        let (_, end) = self.region(types, pointer)?;
        let count = usize::try_from(end.saturating_sub(offset))
            .map_err(|_| Error::CheckedCast)?
            .min(maximum);
        if count == 0 {
            return Err(Error::InvalidIr("C string has no readable terminator"));
        }
        self.intrinsic_work_cost(types, &[(pointer, false)], count)
    }
    pub(crate) fn host_c_string(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        maximum: usize,
    ) -> Result<Vec<u8>, Error> {
        self.validate_pointer(types, pointer)?;
        if !matches!(
            types.kind(pointer.pointee())?,
            TypeKind::Integer(IntegerType::U8)
        ) {
            return Err(Error::InvalidIr("C string requires a byte pointer"));
        }
        let offset = self.byte_offset(types, pointer)?;
        let (_, end) = self.region(types, pointer)?;
        let count = usize::try_from(end.saturating_sub(offset))
            .map_err(|_| Error::CheckedCast)?
            .min(maximum);
        let range = self.intrinsic_range(types, pointer, count, false)?;
        let image = self.image_for(types, self.allocation(pointer)?)?;
        let mut result = Vec::new();
        for index in range {
            if image.range_has_provenance(index, 1)? {
                return Err(Error::UnsupportedPointerOperation(
                    "virtual handle in C string",
                ));
            }
            let byte = image.read_range(index, 1)?[0];
            if byte == 0 {
                return Ok(result);
            }
            result.push(byte);
        }
        Err(Error::InvalidIr("unterminated or over-budget C string"))
    }
}
