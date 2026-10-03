//! Track caller-frame pack backing and reject returned references into that frame.
use super::*;

impl<'ctx> Generator<'ctx, '_, '_> {
    fn sequence_pack_node_type(&self) -> inkwell::types::StructType<'ctx> {
        let pointer = self.context.ptr_type(inkwell::AddressSpace::default());
        self.context.struct_type(
            &[
                pointer.into(),
                self.context.i64_type().into(),
                pointer.into(),
            ],
            false,
        )
    }

    fn sequence_pack_head(&mut self) -> Result<PointerValue<'ctx>, Error> {
        if let Some(head) = self.sequence_temp_allocations {
            return Ok(head);
        }
        let entry = self
            .function
            .get_first_basic_block()
            .ok_or(Error::Invariant)?;
        let builder = self.context.create_builder();
        if let Some(first) = entry.get_first_instruction() {
            builder.position_before(&first);
        } else {
            builder.position_at_end(entry);
        }
        let pointer = self.context.ptr_type(inkwell::AddressSpace::default());
        let head = builder.build_alloca(pointer, "pack.frame.allocations")?;
        memory::store(&builder, head, pointer.const_null().into(), 1)?;
        self.sequence_temp_allocations = Some(head);
        Ok(head)
    }

    pub(super) fn record_sequence_pack_allocation(
        &mut self,
        data: PointerValue<'ctx>,
        bytes: IntValue<'ctx>,
    ) -> Result<(), Error> {
        let head = self.sequence_pack_head()?;
        let pointer = data.get_type();
        let previous = memory::load(&self.builder, pointer.into(), head, "pack.previous", 1)?;
        let node_type = self.sequence_pack_node_type();
        // Each execution owns a distinct node; the cumulative pack budget bounds
        // both these nodes and backing allocations throughout loops.
        let node = self.builder.build_alloca(node_type, "pack.allocation")?;
        for (index, value) in [data.into(), bytes.into(), previous]
            .into_iter()
            .enumerate()
        {
            let field = self.builder.build_struct_gep(
                node_type,
                node,
                index as u32,
                "pack.allocation.field",
            )?;
            memory::store(&self.builder, field, value, 1)?;
        }
        memory::store(&self.builder, head, node.into(), 1)?;
        Ok(())
    }

    pub(super) fn guard_sequence_pack_return(
        &mut self,
        value: Option<BasicValueEnum<'ctx>>,
    ) -> Result<(), Error> {
        let Some(value) = value else {
            return Ok(());
        };
        let signature = *self
            .signatures
            .get(&self.procedure)
            .ok_or(Error::Invariant)?;
        let TypeKind::Procedure(id) = *self.types.kind(signature)? else {
            return Err(Error::Invariant);
        };
        let results = self.types.procedure(id)?.results.to_vec();
        let has_references = results
            .iter()
            .copied()
            .map(|ty| self.sequence_type_contains_pointer(ty))
            .collect::<Result<Vec<_>, _>>()?;
        if !has_references.iter().any(|value| *value) {
            return Ok(());
        }
        let head = self.sequence_pack_head()?;
        let allocations = memory::load(
            &self.builder,
            head.get_type().into(),
            head,
            "pack.return.frame",
            1,
        )?
        .into_pointer_value();
        let active = self
            .builder
            .build_is_not_null(allocations, "pack.return.frame.active")?;
        let check = self.label("pack.return.frame.check");
        let done = self.label("pack.return.frame.done");
        self.builder.build_conditional_branch(active, check, done)?;
        self.builder.position_at_end(check);
        for (index, ty) in results.iter().copied().enumerate() {
            if !self.sequence_type_contains_pointer(ty)? {
                continue;
            }
            let snapshot = if results.len() == 1 {
                value
            } else {
                self.builder.build_extract_value(
                    value.into_struct_value(),
                    index as u32,
                    "pack.return.result",
                )?
            };
            let storage = unions::entry_alloca(
                self.context,
                &self.builder,
                snapshot.get_type(),
                "pack.return.snapshot",
            )?;
            memory::store(&self.builder, storage, snapshot, 1)?;
            self.guard_sequence_pack_storage(storage, ty)?;
        }
        self.builder.build_unconditional_branch(done)?;
        self.builder.position_at_end(done);
        Ok(())
    }

    fn sequence_type_contains_pointer(&self, ty: TypeId) -> Result<bool, Error> {
        let mut pending = vec![ty];
        let mut seen = std::collections::HashSet::new();
        while let Some(ty) = pending.pop() {
            if !seen.insert(ty) {
                continue;
            }
            match *self.types.kind(ty)? {
                TypeKind::Pointer(_)
                | TypeKind::String
                | TypeKind::Slice(_)
                | TypeKind::DynamicArray(_) => return Ok(true),
                TypeKind::FixedArray {
                    element,
                    count,
                } if count != 0 => pending.push(element),
                TypeKind::Record(_) | TypeKind::Any(_) => pending.extend(
                    self.types
                        .record_storage_definition(ty)?
                        .fields
                        .iter()
                        .copied(),
                ),
                TypeKind::Distinct(id) => pending.push(self.types.distinct(id)?.representation),
                _ => {}
            }
        }
        Ok(false)
    }

    fn guard_sequence_pack_storage(
        &mut self,
        storage: PointerValue<'ctx>,
        ty: TypeId,
    ) -> Result<(), Error> {
        match *self.types.kind(ty)? {
            TypeKind::Pointer(_) => {
                let pointer = self.context.ptr_type(inkwell::AddressSpace::default());
                let value = memory::load(
                    &self.builder,
                    pointer.into(),
                    storage,
                    "pack.return.pointer",
                    1,
                )?
                .into_pointer_value();
                self.guard_sequence_pack_pointer(value)?;
            }
            TypeKind::String | TypeKind::Slice(_) | TypeKind::DynamicArray(_) => {
                let layout = self.lowerer.semantic_layout(ty)?;
                let offset = *layout.field_offsets.get(1).ok_or(Error::Invariant)?;
                let field = self.sequence_pack_byte_offset(storage, offset)?;
                let pointer = self.context.ptr_type(inkwell::AddressSpace::default());
                let value =
                    memory::load(&self.builder, pointer.into(), field, "pack.return.data", 1)?
                        .into_pointer_value();
                self.guard_sequence_pack_pointer(value)?;
                if matches!(self.types.kind(ty)?, TypeKind::DynamicArray(_))
                    && let Some(schema) = self.types.allocator_schema()
                {
                    let checked = jai_types::AllocatorSchema::validate(
                        self.types,
                        schema.ty(),
                        schema.mode_type(),
                    )
                    .map_err(|_| Error::Invariant)?;
                    if checked != schema {
                        return Err(Error::Invariant);
                    }
                    let offset = *layout.field_offsets.get(3).ok_or(Error::Invariant)?;
                    let allocator = self.sequence_pack_byte_offset(storage, offset)?;
                    self.guard_sequence_pack_storage(allocator, schema.ty())?;
                }
            }
            TypeKind::Record(_) | TypeKind::Any(_) => {
                let fields = self.types.record_storage_definition(ty)?.fields.to_vec();
                let layout = self.lowerer.semantic_layout(ty)?;
                for (index, field) in fields.into_iter().enumerate() {
                    if self.sequence_type_contains_pointer(field)? {
                        let address =
                            self.sequence_pack_byte_offset(storage, layout.field_offsets[index])?;
                        self.guard_sequence_pack_storage(address, field)?;
                    }
                }
            }
            TypeKind::Distinct(id) => {
                self.guard_sequence_pack_storage(storage, self.types.distinct(id)?.representation)?
            }
            TypeKind::FixedArray {
                element,
                count,
            } if count != 0 && self.sequence_type_contains_pointer(element)? => {
                let integer = self.context.i64_type();
                let origin = self.builder.get_insert_block().ok_or(Error::Invariant)?;
                let test = self.label("pack.return.array.test");
                let body = self.label("pack.return.array.element");
                let done = self.label("pack.return.array.done");
                self.builder.build_unconditional_branch(test)?;
                self.builder.position_at_end(test);
                let index = self.builder.build_phi(integer, "pack.return.array.index")?;
                index.add_incoming(&[(&integer.const_zero(), origin)]);
                let active = self.builder.build_int_compare(
                    IntPredicate::ULT,
                    index.as_basic_value().into_int_value(),
                    integer.const_int(count, false),
                    "pack.return.array.active",
                )?;
                self.builder.build_conditional_branch(active, body, done)?;
                self.builder.position_at_end(body);
                let element_type = self.lowerer.basic(element)?;
                let address = jai_llvm::gep(
                    &self.builder,
                    element_type,
                    storage,
                    &[index.as_basic_value().into_int_value()],
                    "pack.return.array.address",
                )?;
                self.guard_sequence_pack_storage(address, element)?;
                let next = self.builder.build_int_add(
                    index.as_basic_value().into_int_value(),
                    integer.const_int(1, false),
                    "pack.return.array.next",
                )?;
                let backedge = self.builder.get_insert_block().ok_or(Error::Invariant)?;
                self.builder.build_unconditional_branch(test)?;
                index.add_incoming(&[(&next, backedge)]);
                self.builder.position_at_end(done);
            }
            _ => {}
        }
        Ok(())
    }

    fn sequence_pack_byte_offset(
        &self,
        pointer: PointerValue<'ctx>,
        offset: u64,
    ) -> Result<PointerValue<'ctx>, Error> {
        Ok(jai_llvm::gep(
            &self.builder,
            self.context.i8_type().into(),
            pointer,
            &[self.context.i64_type().const_int(offset, false)],
            "pack.return.field",
        )?)
    }

    fn guard_sequence_pack_pointer(&mut self, pointer: PointerValue<'ctx>) -> Result<(), Error> {
        // Initialize even when a return precedes concat in source: an earlier
        // branch may execute after a later allocation on another loop iteration.
        let head = self.sequence_pack_head()?;
        let initial = memory::load(
            &self.builder,
            head.get_type().into(),
            head,
            "pack.return.allocations",
            1,
        )?
        .into_pointer_value();
        let origin = self.builder.get_insert_block().ok_or(Error::Invariant)?;
        let test = self.label("pack.return.lifetime.test");
        let body = self.label("pack.return.lifetime.check");
        let done = self.label("pack.return.lifetime.done");
        self.builder.build_unconditional_branch(test)?;
        self.builder.position_at_end(test);
        let node = self
            .builder
            .build_phi(head.get_type(), "pack.return.allocation")?;
        node.add_incoming(&[(&initial, origin)]);
        let node_pointer = node.as_basic_value().into_pointer_value();
        let active = self
            .builder
            .build_is_not_null(node_pointer, "pack.return.allocation.active")?;
        self.builder.build_conditional_branch(active, body, done)?;
        self.builder.position_at_end(body);
        let node_type = self.sequence_pack_node_type();
        let data_field = self.builder.build_struct_gep(
            node_type,
            node_pointer,
            0,
            "pack.return.allocation.data",
        )?;
        let data = memory::load(
            &self.builder,
            head.get_type().into(),
            data_field,
            "pack.return.data",
            1,
        )?
        .into_pointer_value();
        let bytes_field = self.builder.build_struct_gep(
            node_type,
            node_pointer,
            1,
            "pack.return.allocation.bytes",
        )?;
        let bytes = memory::load(
            &self.builder,
            self.context.i64_type().into(),
            bytes_field,
            "pack.return.bytes",
            1,
        )?
        .into_int_value();
        let integer = self.context.ptr_sized_int_type(&self.target.data, None);
        let address = self
            .builder
            .build_ptr_to_int(pointer, integer, "pack.return.address")?;
        let start = self
            .builder
            .build_ptr_to_int(data, integer, "pack.return.start")?;
        let size = self
            .builder
            .build_int_cast(bytes, integer, "pack.return.size")?;
        let offset = self
            .builder
            .build_int_sub(address, start, "pack.return.offset")?;
        let after = self.builder.build_int_compare(
            IntPredicate::UGE,
            address,
            start,
            "pack.return.after.start",
        )?;
        // Include one-past pointers: they also carry the frame's lifetime.
        let within = self.builder.build_int_compare(
            IntPredicate::ULE,
            offset,
            size,
            "pack.return.within",
        )?;
        let escaping = self
            .builder
            .build_and(after, within, "pack.return.escaping")?;
        let valid = self.builder.build_not(escaping, "pack.return.valid")?;
        self.check_cast(Bit(valid))?;
        let next_field = self.builder.build_struct_gep(
            node_type,
            node_pointer,
            2,
            "pack.return.allocation.next",
        )?;
        let next = memory::load(
            &self.builder,
            head.get_type().into(),
            next_field,
            "pack.return.next",
            1,
        )?
        .into_pointer_value();
        let backedge = self.builder.get_insert_block().ok_or(Error::Invariant)?;
        self.builder.build_unconditional_branch(test)?;
        node.add_incoming(&[(&next, backedge)]);
        self.builder.position_at_end(done);
        Ok(())
    }
}
