//! Custom records use typed packed payloads with explicit padding and alignment.
use super::*;
use inkwell::{
    targets::TargetData,
    types::{BasicType, BasicTypeEnum, StructType},
    values::BasicValue,
};
use jai_types::Layout;

/// A zero-sized vector carrier imposes alignment without adding storage bytes.
/// Check the selected target rather than assuming a host/vector ABI rule.
pub(super) fn alignment_carrier<'ctx>(
    context: &'ctx Context,
    target: &TargetData,
    alignment: u32,
) -> Result<BasicTypeEnum<'ctx>, types::Error> {
    let candidate: BasicTypeEnum<'ctx> = if alignment == 1 {
        context.i8_type().into()
    } else {
        context.i8_type().vec_type(alignment).into()
    };
    let actual = target.get_abi_alignment(&candidate);
    if actual != alignment {
        return Err(types::Error::UnsupportedAlignment {
            requested: alignment,
            actual,
        });
    }
    Ok(candidate.array_type(0).into())
}

/// Preserve typed fields (and therefore relocations in constants). Padding
/// occupies separate byte arrays; all payload members use packed LLVM offsets.
pub(super) fn payload<'ctx>(
    context: &'ctx Context,
    target: &TargetData,
    fields: &[BasicTypeEnum<'ctx>],
    layout: &Layout,
) -> Result<(StructType<'ctx>, Vec<u32>), types::Error> {
    if fields.len() != layout.field_offsets.len() {
        return Err(types::Error::InvalidCustomLayout);
    }
    let mut physical = Vec::new();
    let mut ordinals = Vec::new();
    let mut end = 0;
    for (field, &offset) in fields.iter().zip(&layout.field_offsets) {
        append_padding(
            context,
            &mut physical,
            offset
                .checked_sub(end)
                .ok_or(types::Error::InvalidCustomLayout)?,
        )?;
        ordinals
            .push(u32::try_from(physical.len()).map_err(|_| types::Error::InvalidCustomLayout)?);
        physical.push(*field);
        end = offset
            .checked_add(target.get_abi_size(field))
            .ok_or(types::Error::InvalidCustomLayout)?;
    }
    append_padding(
        context,
        &mut physical,
        layout
            .size
            .checked_sub(end)
            .ok_or(types::Error::InvalidCustomLayout)?,
    )?;
    let payload = context.struct_type(&physical, true);
    if target.get_abi_size(&payload) != layout.size {
        return Err(types::Error::InvalidCustomLayout);
    }
    for (&ordinal, &offset) in ordinals.iter().zip(&layout.field_offsets) {
        if target.offset_of_element(&payload, ordinal) != Some(offset) {
            return Err(types::Error::InvalidCustomLayout);
        }
    }
    Ok((payload, ordinals))
}

fn append_padding<'ctx>(
    context: &'ctx Context,
    fields: &mut Vec<BasicTypeEnum<'ctx>>,
    count: u64,
) -> Result<(), types::Error> {
    if count > 0 {
        let count = u32::try_from(count).map_err(|_| types::Error::InvalidCustomLayout)?;
        fields.push(context.i8_type().array_type(count).into());
    }
    Ok(())
}

pub(super) fn body<'ctx>(
    context: &'ctx Context,
    target: &TargetData,
    fields: &[BasicTypeEnum<'ctx>],
    layout: &Layout,
) -> Result<Vec<BasicTypeEnum<'ctx>>, types::Error> {
    let carrier = alignment_carrier(context, target, layout.alignment)?;
    let (payload, _) = payload(context, target, fields, layout)?;
    Ok(vec![carrier, payload.into()])
}

pub(super) fn field_pointer<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    pointer: PointerValue<'ctx>,
    offset: u64,
) -> Result<PointerValue<'ctx>, Error> {
    Ok(jai_llvm::gep(
        builder,
        context.i8_type().into(),
        pointer,
        &[context.i64_type().const_int(offset, false)],
        "record.field.address",
    )?)
}

pub(super) fn constant<'ctx>(
    context: &'ctx Context,
    target: &TargetData,
    storage: StructType<'ctx>,
    fields: &[BasicValueEnum<'ctx>],
    layout: &Layout,
) -> Result<BasicValueEnum<'ctx>, Error> {
    let actual = fields
        .iter()
        .map(|field| field.get_type())
        .collect::<Vec<_>>();
    let (payload, ordinals) = payload(context, target, &actual, layout)?;
    let mut values = payload
        .get_field_types()
        .iter()
        .map(|ty| ty.const_zero())
        .collect::<Vec<_>>();
    for (&ordinal, &value) in ordinals.iter().zip(fields) {
        values[ordinal as usize] = value;
    }
    let carrier = storage.get_field_type_at_index(0).ok_or(Error::Invariant)?;
    let physical = if storage.get_field_type_at_index(1) == Some(payload.into()) {
        storage
    } else {
        context.struct_type(&[carrier, payload.into()], false)
    };
    if target.get_abi_size(&physical) != layout.size
        || target.get_abi_alignment(&physical) != layout.alignment
    {
        return Err(Error::Invariant);
    }
    Ok(physical
        .const_named_struct(&[
            carrier.const_zero(),
            payload.const_named_struct(&values).into(),
        ])
        .into())
}

impl<'ctx> Generator<'ctx, '_, '_> {
    pub(super) fn custom_record_build<'a>(
        &mut self,
        ty: TypeId,
        initializers: impl IntoIterator<Item = (jai_types::FieldId, &'a ValueExpr)>,
    ) -> Result<BasicValueEnum<'ctx>, Error> {
        let storage = self.lowerer.basic(ty)?;
        let layout = self.lowerer.semantic_layout(ty)?;
        let definition = self.types.record_storage_definition(ty)?;
        let field_types = definition.fields.to_vec();
        let placed = definition
            .layout
            .field_placements
            .iter()
            .any(Option::is_some);
        let mut fields = if placed {
            Vec::new()
        } else {
            field_types
                .iter()
                .map(|&ty| self.lowerer.basic(ty).map(|ty| ty.const_zero()))
                .collect::<Result<Vec<_>, _>>()?
        };
        let mut writes = Vec::new();
        for (field, initializer) in initializers {
            if self.types.validate_field(ty, field)? != initializer.type_id(self.types) {
                return Err(Error::Invariant);
            }
            if placed {
                let offset = *layout
                    .field_offsets
                    .get(field.index())
                    .ok_or(Error::Invariant)?;
                let field_type = self.lowerer.basic(initializer.type_id(self.types))?;
                let extent = self.target.data.get_abi_size(&field_type);
                if offset
                    .checked_add(extent)
                    .is_none_or(|end| end > layout.size)
                {
                    return Err(Error::Invariant);
                }
                writes.push((offset, initializer));
            } else {
                fields[field.index()] = self.value(initializer)?;
            }
        }
        if !placed && fields.iter().all(|field| field.is_const()) {
            let constant = constant(
                self.context,
                &self.target.data,
                storage.into_struct_type(),
                &fields,
                &layout,
            )?;
            if constant.get_type() == storage {
                return Ok(constant);
            }
        }
        let temporary =
            unions::entry_alloca(self.context, &self.builder, storage, "record.snapshot")?;
        memory::store(
            &self.builder,
            temporary,
            storage.const_zero(),
            layout.alignment,
        )?;
        if placed {
            // Placed fields may overlap. Keep initializer order so later
            // writes replace earlier bytes at the selected storage offset.
            for (offset, initializer) in writes {
                let value = self.value(initializer)?;
                let pointer = field_pointer(self.context, &self.builder, temporary, offset)?;
                memory::store(
                    &self.builder,
                    pointer,
                    value,
                    memory::offset_alignment(layout.alignment, offset),
                )?;
            }
        } else {
            for (index, value) in fields.into_iter().enumerate() {
                let offset = *layout.field_offsets.get(index).ok_or(Error::Invariant)?;
                let pointer = field_pointer(self.context, &self.builder, temporary, offset)?;
                memory::store(
                    &self.builder,
                    pointer,
                    value,
                    memory::offset_alignment(layout.alignment, offset),
                )?;
            }
        }
        memory::load(
            &self.builder,
            storage,
            temporary,
            "record.value",
            layout.alignment,
        )
    }

    pub(super) fn ordered_record_build(
        &mut self,
        ty: TypeId,
        backing: jai_ir::OrderedRecordBacking,
        initializers: &[(Box<[jai_types::FieldId]>, ValueExpr)],
    ) -> Result<BasicValueEnum<'ctx>, Error> {
        if self.types.record_storage_definition(ty)?.kind != jai_types::RecordKind::Struct {
            return Err(Error::Invariant);
        }
        let storage = self.lowerer.basic(ty)?;
        let layout = self.lowerer.semantic_layout(ty)?;
        // All paths, leaf types, offsets, and extents are checked before any
        // initializer runs. Overlap is intentional; writes are never grouped.
        let mut writes = Vec::with_capacity(initializers.len());
        for (path, value) in initializers {
            let leaf = jai_ir::ordered_record_path_type(self.types, ty, path)?;
            if value.type_id(self.types) != leaf {
                return Err(Error::Invariant);
            }
            let mut owner = ty;
            let mut offset = 0u64;
            for &field in path.iter() {
                let owner_layout = self.lowerer.semantic_layout(owner)?;
                offset = offset
                    .checked_add(
                        *owner_layout
                            .field_offsets
                            .get(field.index())
                            .ok_or(Error::Invariant)?,
                    )
                    .ok_or(Error::Invariant)?;
                owner = self.types.validate_field(owner, field)?;
            }
            let leaf_storage = self.lowerer.basic(leaf)?;
            if offset
                .checked_add(self.target.data.get_abi_size(&leaf_storage))
                .is_none_or(|end| end > layout.size)
            {
                return Err(Error::Invariant);
            }
            writes.push((offset, leaf_storage));
        }
        let temporary =
            unions::entry_alloca(self.context, &self.builder, storage, "record.ordered")?;
        if backing == jai_ir::OrderedRecordBacking::Zeroed {
            memory::store(
                &self.builder,
                temporary,
                storage.const_zero(),
                layout.alignment,
            )?;
        }
        for ((_, initializer), (offset, leaf_storage)) in initializers.iter().zip(writes) {
            let value = self.value(initializer)?;
            if value.get_type() != leaf_storage {
                return Err(Error::Invariant);
            }
            let pointer = field_pointer(self.context, &self.builder, temporary, offset)?;
            memory::store(
                &self.builder,
                pointer,
                value,
                memory::offset_alignment(layout.alignment, offset),
            )?;
        }
        memory::load(
            &self.builder,
            storage,
            temporary,
            "record.value",
            layout.alignment,
        )
    }

    pub(super) fn custom_record_extract(
        &mut self,
        base_ty: TypeId,
        snapshot: BasicValueEnum<'ctx>,
        field: jai_types::FieldId,
        ty: TypeId,
    ) -> Result<BasicValueEnum<'ctx>, Error> {
        let layout = self.lowerer.semantic_layout(base_ty)?;
        let offset = *layout
            .field_offsets
            .get(field.index())
            .ok_or(Error::Invariant)?;
        if snapshot.is_const()
            && !self
                .types
                .record_storage_definition(base_ty)?
                .layout
                .field_placements
                .iter()
                .any(Option::is_some)
        {
            let field_types = self
                .types
                .record_storage_definition(base_ty)?
                .fields
                .to_vec();
            let fields = field_types
                .iter()
                .map(|&ty| self.lowerer.basic(ty))
                .collect::<Result<Vec<_>, _>>()?;
            let (_, ordinals) = payload(self.context, &self.target.data, &fields, &layout)?;
            let packed = snapshot
                .into_struct_value()
                .get_field_at_index(1)
                .ok_or(Error::Invariant)?
                .into_struct_value();
            return packed
                .get_field_at_index(ordinals[field.index()])
                .ok_or(Error::Invariant);
        }
        let temporary = unions::entry_alloca(
            self.context,
            &self.builder,
            snapshot.get_type(),
            "record.snapshot",
        )?;
        memory::store(&self.builder, temporary, snapshot, layout.alignment)?;
        let pointer = field_pointer(self.context, &self.builder, temporary, offset)?;
        memory::load(
            &self.builder,
            self.lowerer.basic(ty)?,
            pointer,
            "record.field",
            memory::offset_alignment(layout.alignment, offset),
        )
    }
}
