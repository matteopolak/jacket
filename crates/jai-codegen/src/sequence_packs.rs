//! Caller-frame storage for ordered scalar/spread Jai argument packs.
use super::*;
use inkwell::values::BasicValue;
use jai_ir::{MAX_SEQUENCE_TEMP_BYTES, SequencePackPart, sequence_temp_allocation_charge};

enum Snapshot<'ctx> {
    Element(BasicValueEnum<'ctx>),
    Spread {
        count: IntValue<'ctx>,
        bytes: IntValue<'ctx>,
        data: PointerValue<'ctx>,
    },
}

impl<'ctx> Generator<'ctx, '_, '_> {
    pub(super) fn sequence_concat(
        &mut self,
        ty: TypeId,
        parts: &[SequencePackPart],
    ) -> Result<BasicValueEnum<'ctx>, Error> {
        let TypeKind::Slice(element) = *self.types.kind(ty)? else {
            return Err(Error::Invariant);
        };
        let storage = self.lowerer.basic(element)?;
        let layout = self.lowerer.semantic_layout(element)?;
        let integer = self.context.i64_type();
        let mut total = integer.const_zero();
        let mut snapshots = Vec::with_capacity(parts.len());
        for part in parts {
            let (count, snapshot) = match part {
                SequencePackPart::Element(value) => (
                    integer.const_int(1, false),
                    Snapshot::Element(self.value(value)?),
                ),
                SequencePackPart::Spread(value) => {
                    let descriptor = self.value(value)?.into_struct_value();
                    let count = self
                        .builder
                        .build_extract_value(descriptor, 0, "pack.spread.count")?
                        .into_int_value();
                    let source = self
                        .builder
                        .build_extract_value(descriptor, 1, "pack.spread.data")?
                        .into_pointer_value();
                    let nonnegative = self.builder.build_int_compare(
                        IntPredicate::SGE,
                        count,
                        integer.const_zero(),
                        "pack.spread.nonnegative",
                    )?;
                    self.check_cast(Bit(nonnegative))?;
                    let empty = self.builder.build_int_compare(
                        IntPredicate::EQ,
                        count,
                        integer.const_zero(),
                        "pack.spread.empty",
                    )?;
                    let nonnull = self
                        .builder
                        .build_is_not_null(source, "pack.spread.nonnull")?;
                    let valid = self.builder.build_or(empty, nonnull, "pack.spread.valid")?;
                    self.check_cast(Bit(valid))?;
                    let bytes = self.sequence_pack_bytes(count, layout.size)?;
                    let data = self.sequence_pack_allocation(count, bytes, layout.alignment)?;
                    // Copy now: later argument expressions may mutate source backing.
                    self.builder
                        .build_memcpy(data, layout.alignment, source, 1, bytes)?;
                    (
                        count,
                        Snapshot::Spread {
                            count,
                            bytes,
                            data,
                        },
                    )
                }
            };
            let available = self.builder.build_int_sub(
                integer.const_int(i64::MAX as u64, false),
                total,
                "pack.count.remaining",
            )?;
            let valid = self.builder.build_int_compare(
                IntPredicate::ULE,
                count,
                available,
                "pack.count.valid",
            )?;
            self.check_cast(Bit(valid))?;
            total = self.builder.build_int_add(total, count, "pack.count")?;
            snapshots.push(snapshot);
        }
        let bytes = self.sequence_pack_bytes(total, layout.size)?;
        let data = self.sequence_pack_allocation(total, bytes, layout.alignment)?;
        let mut offset = integer.const_zero();
        for snapshot in snapshots {
            let destination = jai_llvm::gep(
                &self.builder,
                storage,
                data,
                &[offset],
                "pack.element.address",
            )?;
            let count = match snapshot {
                Snapshot::Element(value) => {
                    memory::store(&self.builder, destination, value, layout.alignment)?;
                    integer.const_int(1, false)
                }
                Snapshot::Spread {
                    count,
                    bytes,
                    data: source,
                } => {
                    self.builder.build_memcpy(
                        destination,
                        layout.alignment,
                        source,
                        layout.alignment,
                        bytes,
                    )?;
                    count
                }
            };
            offset = self
                .builder
                .build_int_add(offset, count, "pack.element.offset")?;
        }
        let descriptor = self.lowerer.basic(ty)?.into_struct_type();
        let value = self
            .builder
            .build_insert_value(descriptor.const_zero(), total, 0, "pack.count")?
            .into_struct_value();
        Ok(self
            .builder
            .build_insert_value(value, data, 1, "pack.data")?
            .into_struct_value()
            .into())
    }

    fn sequence_pack_bytes(
        &mut self,
        count: IntValue<'ctx>,
        element_size: u64,
    ) -> Result<IntValue<'ctx>, Error> {
        let integer = self.context.i64_type();
        if element_size == 0 {
            return Ok(integer.const_zero());
        }
        // This proves multiplication fits both the resource cap and the target
        // pointer width before LLVM sees an allocation or memcpy byte count.
        let maximum = MAX_SEQUENCE_TEMP_BYTES / element_size;
        let valid = self.builder.build_int_compare(
            IntPredicate::ULE,
            count,
            integer.const_int(maximum, false),
            "pack.bytes.valid",
        )?;
        self.check_cast(Bit(valid))?;
        Ok(self.builder.build_int_mul(
            count,
            integer.const_int(element_size, false),
            "pack.bytes",
        )?)
    }

    fn sequence_pack_allocation(
        &mut self,
        count: IntValue<'ctx>,
        bytes: IntValue<'ctx>,
        alignment: u32,
    ) -> Result<PointerValue<'ctx>, Error> {
        let integer = self.context.i64_type();
        let empty = self.builder.build_int_compare(
            IntPredicate::EQ,
            count,
            integer.const_zero(),
            "pack.empty",
        )?;
        let skip = self.label("pack.empty");
        let allocate = self.label("pack.allocate");
        let merge = self.label("pack.storage");
        self.builder
            .build_conditional_branch(empty, skip, allocate)?;
        self.builder.position_at_end(skip);
        let null = self
            .context
            .ptr_type(inkwell::AddressSpace::default())
            .const_null();
        self.builder.build_unconditional_branch(merge)?;
        self.builder.position_at_end(allocate);
        let zero_bytes = self.builder.build_int_compare(
            IntPredicate::EQ,
            bytes,
            integer.const_zero(),
            "pack.zero.bytes",
        )?;
        // Nonempty zero-sized element arrays still need a nonnull data address.
        let allocated_bytes = self
            .builder
            .build_select(
                zero_bytes,
                integer.const_int(1, false),
                bytes,
                "pack.allocated.bytes",
            )?
            .into_int_value();
        let charged = self.builder.build_int_add(
            allocated_bytes,
            integer.const_int(
                sequence_temp_allocation_charge(1, alignment).ok_or(Error::Invariant)? - 1,
                false,
            ),
            "pack.charged.bytes",
        )?;
        let counter = self.sequence_pack_counter()?;
        let used = memory::load(&self.builder, integer.into(), counter, "pack.used.bytes", 8)?
            .into_int_value();
        let available = self.builder.build_int_sub(
            integer.const_int(MAX_SEQUENCE_TEMP_BYTES, false),
            used,
            "pack.available.bytes",
        )?;
        let valid = self.builder.build_int_compare(
            IntPredicate::ULE,
            charged,
            available,
            "pack.frame.valid",
        )?;
        self.check_cast(Bit(valid))?;
        let used = self
            .builder
            .build_int_add(used, charged, "pack.used.bytes")?;
        memory::store(&self.builder, counter, used.into(), 8)?;
        let data = self.builder.build_array_alloca(
            self.context.i8_type(),
            allocated_bytes,
            "pack.temporary",
        )?;
        data.as_instruction_value()
            .ok_or(Error::Invariant)?
            .set_alignment(alignment)
            .map_err(|_| Error::Invariant)?;
        self.record_sequence_pack_allocation(data, allocated_bytes)?;
        let allocated = self.builder.get_insert_block().ok_or(Error::Invariant)?;
        self.builder.build_unconditional_branch(merge)?;
        self.builder.position_at_end(merge);
        let result = self.builder.build_phi(null.get_type(), "pack.data")?;
        result.add_incoming(&[(&null, skip), (&data, allocated)]);
        Ok(result.as_basic_value().into_pointer_value())
    }

    fn sequence_pack_counter(&mut self) -> Result<PointerValue<'ctx>, Error> {
        if let Some(counter) = self.sequence_temp_bytes {
            return Ok(counter);
        }
        let entry = self
            .function
            .get_first_basic_block()
            .ok_or(Error::Invariant)?;
        let builder = self.context.create_builder();
        if let Some(first) = entry.get_first_instruction() {
            builder.position_before(&first)
        } else {
            builder.position_at_end(entry)
        }
        let counter = builder.build_alloca(self.context.i64_type(), "pack.frame.bytes")?;
        counter
            .as_instruction_value()
            .ok_or(Error::Invariant)?
            .set_alignment(8)
            .map_err(|_| Error::Invariant)?;
        memory::store(
            &builder,
            counter,
            self.context.i64_type().const_zero().into(),
            8,
        )?;
        self.sequence_temp_bytes = Some(counter);
        Ok(counter)
    }
}
