//! Memory-owned storage casts retain canonical addresses and byte initialization.
use super::*;
use jai_types::StorageBitcast;

impl Memory {
    fn storage_cast_receipt(
        &self,
        types: &dyn TypeView,
        cast: StorageBitcast,
    ) -> Result<(), Error> {
        if cast.layout_policy() != self.target.policy {
            return Err(Error::InvalidIr(
                "storage cast belongs to another target layout",
            ));
        }
        // VM callers prepare both roots with charged cold traversal. Reuse that
        // cache here rather than hiding a new LayoutEngine behind a warm cast.
        let source = self.prepared_layout(types, cast.source_type())?;
        let target = self.prepared_layout(types, cast.target_type())?;
        if source.size != cast.source_size()
            || target.size != cast.target_size()
            || source.alignment.max(target.alignment) != cast.scratch_alignment()
        {
            return Err(Error::InvalidIr(
                "storage cast proof differs from prepared layouts",
            ));
        }
        Ok(())
    }

    pub(crate) fn storage_bitcast_place_work_cost(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        cast: StorageBitcast,
    ) -> Result<usize, Error> {
        self.storage_cast_receipt(types, cast)?;
        if pointer.pointee != cast.source_type() {
            return Err(Error::TypeMismatch {
                expected: cast.source_type(),
            });
        }
        let bytes = storage_bytes(cast.source_size(), self.limits.value_cells)?;
        let range = self.intrinsic_range(types, pointer, bytes, false)?;
        let allocation = self.allocation(pointer)?;
        let target_nodes = self.prepared_decoded_cells(types, cast.target_type())?;
        let mut work = add_work(pointer.metadata_cells(), bytes)?;
        work = add_work(
            work,
            self.prepared_codec_layout_work(types, cast.target_type())?
                .saturating_mul(3),
        )?;
        // Destination traversal validates representations and then decodes them.
        work = add_work(work, target_nodes.saturating_mul(2))?;
        let image = allocation.image.borrow();
        if let Some(image) = image.as_ref() {
            work = add_work(work, image.range_metadata_work(range.start, bytes)?)?;
        } else {
            work = add_work(
                work,
                storage_bytes(allocation.virtual_extent, self.limits.value_cells)?,
            )?;
            work = add_work(work, allocation.cells.get())?;
            work = add_work(work, self.prepared_codec_layout_work(types, allocation.ty)?)?;
            work = add_work(work, self.retokenize_work_cost())?;
        }
        Ok(work)
    }

    pub(crate) fn storage_bitcast_value_work_cost(
        &self,
        types: &dyn TypeView,
        value: &Value,
        cast: StorageBitcast,
    ) -> Result<usize, Error> {
        self.storage_cast_receipt(types, cast)?;
        let cells = value.cells(self.limits.value_cells)?;
        let bytes = storage_bytes(cast.source_size(), self.limits.value_cells)?;
        let target_nodes = self.prepared_decoded_cells(types, cast.target_type())?;
        let work = add_work(bytes, cells.saturating_mul(3))?;
        let work = add_work(
            work,
            self.prepared_codec_layout_work(types, cast.source_type())?,
        )?;
        let work = add_work(
            work,
            self.prepared_codec_layout_work(types, cast.target_type())?
                .saturating_mul(3),
        )?;
        let work = add_work(work, target_nodes.saturating_mul(2))?;
        add_work(work, self.retokenize_work_cost())
    }

    pub(crate) fn storage_bitcast_place(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        cast: StorageBitcast,
    ) -> Result<Value, Error> {
        self.storage_cast_receipt(types, cast)?;
        if pointer.pointee != cast.source_type() {
            return Err(Error::TypeMismatch {
                expected: cast.source_type(),
            });
        }
        let bytes = storage_bytes(cast.source_size(), self.limits.value_cells)?;
        let range = self.intrinsic_range(types, pointer, bytes, false)?;
        let allocation = self.allocation(pointer)?;
        if allocation.image.borrow().is_none() && allocation.value.is_none() {
            // Reading an empty destination is valid even over unwritten source
            // storage. The scratch mask remains unwritten; no allocation changes.
            let length = storage_bytes(allocation.virtual_extent, self.limits.value_cells)?;
            let image = ByteImage::uninitialized(self.target, length, self.limits.value_cells)?;
            return self.decode_storage_cast(types, &image, range.start, cast.target_type());
        }
        self.ensure_image(types, allocation)?;
        let image = allocation.image.borrow();
        self.decode_storage_cast(
            types,
            image
                .as_ref()
                .ok_or(Error::InvalidIr("byte image initialization failed"))?,
            range.start,
            cast.target_type(),
        )
    }

    pub(crate) fn storage_bitcast_value(
        &self,
        types: &dyn TypeView,
        value: &Value,
        cast: StorageBitcast,
    ) -> Result<Value, Error> {
        self.storage_cast_receipt(types, cast)?;
        if let Value::StoredAggregate(snapshot) = value {
            self.validate_stored_aggregate(types, snapshot)?;
        }
        let mut image = ByteImage::encode(
            types,
            self.target,
            cast.source_type(),
            value,
            self.limits.value_cells,
        )?;
        // Standalone encoding cannot certify code addresses. Only this Memory
        // issues canonical relocation tokens and the corresponding receipts.
        self.retokenize_image(types, &mut image)?;
        self.decode_storage_cast(types, &image, 0, cast.target_type())
    }

    fn decode_storage_cast(
        &self,
        types: &dyn TypeView,
        image: &ByteImage,
        offset: usize,
        ty: TypeId,
    ) -> Result<Value, Error> {
        let result =
            image.read_storage_bitcast(types, self.target, offset, ty, self.limits.value_cells);
        // A complete integer address receipt has no ByteImage relocation. An
        // explicit storage cast may recover it only through Memory's existing
        // inverse proof of bits, issuer, lifetime, region, and code signature.
        match (result, types.kind(ty)?) {
            (
                Err(Error::UnsupportedPointerOperation(
                    "tagged integer bytes require an explicit integer-to-pointer cast",
                )),
                TypeKind::Pointer(pointee),
            ) => {
                let number = self.storage_address_number(types, image, offset)?;
                Ok(Value::Pointer(self.integer_to_pointer(
                    types,
                    number,
                    *pointee,
                    CastMode::Checked,
                )?))
            }
            (
                Err(Error::UnsupportedPointerOperation(
                    "tagged address bytes cannot construct procedure values",
                )),
                TypeKind::Procedure(_),
            ) => {
                let number = self.storage_address_number(types, image, offset)?;
                let pointer = self.integer_to_pointer(types, number, ty, CastMode::Checked)?;
                let code = pointer
                    .code_pointer()
                    .ok_or(Error::UnsupportedPointerOperation(
                        "data address cannot construct a procedure",
                    ))?;
                self.code_pointer_value(types, code, ty)
            }
            (result, _) => result,
        }
    }

    fn storage_address_number(
        &self,
        types: &dyn TypeView,
        image: &ByteImage,
        offset: usize,
    ) -> Result<crate::Number, Error> {
        let integer = match self.target.policy.pointer().size {
            1 => IntegerType::U8,
            2 => IntegerType::U16,
            4 => IntegerType::U32,
            8 => IntegerType::U64,
            _ => {
                return Err(Error::UnsupportedPointerOperation(
                    "target pointer width exceeds the virtual address representation",
                ));
            }
        };
        image
            .read(
                types,
                self.target,
                offset,
                types.scalar(ScalarType::Int(integer)),
            )?
            .number()
    }
}

fn storage_bytes(bytes: u64, limit: usize) -> Result<usize, Error> {
    usize::try_from(bytes)
        .ok()
        .filter(|bytes| *bytes <= limit)
        .ok_or(Error::Limit(LimitKind::ValueCells))
}
fn add_work(left: usize, right: usize) -> Result<usize, Error> {
    left.checked_add(right).ok_or(Error::Limit(LimitKind::Fuel))
}
