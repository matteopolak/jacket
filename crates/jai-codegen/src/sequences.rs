//! Ordered array snapshots and target-shaped count/data descriptors.
use super::*;
use inkwell::{module::Linkage, types::BasicTypeEnum, values::BasicValue};
use jai_ir::{ConstantKind, ConstantValue, SequenceField};

pub(super) fn array_constant<'ctx>(
    element: BasicTypeEnum<'ctx>,
    elements: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, Error> {
    // Inkwell's constant arrays expose one constructor per physical element type.
    macro_rules! array {
        ($ty:expr, $variant:ident) => {{
            let elements = elements
                .iter()
                .map(|element| match element {
                    BasicValueEnum::$variant(value) => Ok(*value),
                    _ => Err(Error::Invariant),
                })
                .collect::<Result<Vec<_>, _>>()?;
            $ty.const_array(&elements).into()
        }};
    }
    Ok(match element {
        BasicTypeEnum::IntType(ty) => array!(ty, IntValue),
        BasicTypeEnum::FloatType(ty) => array!(ty, FloatValue),
        BasicTypeEnum::PointerType(ty) => array!(ty, PointerValue),
        BasicTypeEnum::StructType(ty) => array!(ty, StructValue),
        BasicTypeEnum::ArrayType(ty) => array!(ty, ArrayValue),
        BasicTypeEnum::VectorType(ty) => array!(ty, VectorValue),
        BasicTypeEnum::ScalableVectorType(_) => return Err(Error::Invariant),
    })
}

pub(super) fn constant_string<'ctx>(
    lowerer: &mut types::TypeLowerer<'ctx, '_>,
    value: &ConstantValue,
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> Result<BasicValueEnum<'ctx>, Error> {
    let ConstantKind::StringBytes(bytes) = &value.kind else {
        return Err(Error::Invariant);
    };
    string_descriptor(
        context,
        module,
        lowerer.basic(value.ty)?.into_struct_type(),
        bytes,
    )
}

fn string_descriptor<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    descriptor: inkwell::types::StructType<'ctx>,
    bytes: &[u8],
) -> Result<BasicValueEnum<'ctx>, Error> {
    let count = i64::try_from(bytes.len()).map_err(|_| Error::Invariant)?;
    let data = if bytes.is_empty() {
        context
            .ptr_type(inkwell::AddressSpace::default())
            .const_null()
    } else {
        let storage = context.const_string(bytes, false);
        literal_global(module, storage.into(), "jai.string.bytes")
    };
    Ok(descriptor
        .const_named_struct(&[
            context.i64_type().const_int(count as u64, false).into(),
            data.into(),
        ])
        .into())
}

impl<'ctx> Generator<'ctx, '_, '_> {
    pub(super) fn sequence_array(
        &mut self,
        ty: TypeId,
        elements: &[ValueExpr],
    ) -> Result<BasicValueEnum<'ctx>, Error> {
        let array = self.lowerer.basic(ty)?.into_array_type();
        let mut snapshot = array.const_zero();
        for (index, element) in elements.iter().enumerate() {
            let element = self.value(element)?;
            snapshot = self
                .builder
                .build_insert_value(
                    snapshot,
                    element,
                    u32::try_from(index).map_err(|_| Error::Invariant)?,
                    "array.element",
                )?
                .into_array_value();
        }
        Ok(snapshot.into())
    }

    pub(super) fn sequence_string(
        &mut self,
        ty: TypeId,
        bytes: &[u8],
    ) -> Result<BasicValueEnum<'ctx>, Error> {
        let descriptor = self.lowerer.basic(ty)?.into_struct_type();
        string_descriptor(self.context, self.module, descriptor, bytes)
    }

    pub(super) fn sequence_field(
        &mut self,
        base: &ValueExpr,
        field: SequenceField,
        _ty: TypeId,
    ) -> Result<BasicValueEnum<'ctx>, Error> {
        let base_ty = base.type_id(self.types);
        if let TypeKind::FixedArray { count, .. } = self.types.kind(base_ty)? {
            let count = *count;
            return match field {
                SequenceField::Count => {
                    self.value(base)?; // Preserve calls and initializer evaluation.
                    Ok(self.context.i64_type().const_int(count, false).into())
                }
                SequenceField::Data => {
                    let pointer = match base {
                        ValueExpr::Load(place) => self.slot(*place)?.pointer,
                        _ => {
                            let snapshot = self.value(base)?;
                            self.sequence_array_address(snapshot, jai_ir::is_static_value(base))?
                        }
                    };
                    // Opaque pointers use the same address for an array and its first element.
                    let pointer = if count == 0 {
                        self.context
                            .ptr_type(inkwell::AddressSpace::default())
                            .const_null()
                    } else {
                        pointer
                    };
                    Ok(pointer.into())
                }
                SequenceField::Allocated => Err(Error::Invariant),
            };
        }
        let snapshot = self.value(base)?.into_struct_value();
        let index = match field {
            SequenceField::Count => 0,
            SequenceField::Data => 1,
            SequenceField::Allocated => 2,
        };
        Ok(self
            .builder
            .build_extract_value(snapshot, index, "sequence.field")?)
    }

    pub(super) fn array_to_slice(
        &mut self,
        array: Place,
        ty: TypeId,
    ) -> Result<BasicValueEnum<'ctx>, Error> {
        let TypeKind::FixedArray { count, .. } = self.types.kind(array.ty())? else {
            return Err(Error::Invariant);
        };
        let count = *count;
        let pointer = self.slot(array)?.pointer;
        let pointer = if count == 0 {
            self.context
                .ptr_type(inkwell::AddressSpace::default())
                .const_null()
        } else {
            pointer
        };
        let descriptor = self.lowerer.basic(ty)?.into_struct_type();
        let snapshot = self
            .builder
            .build_insert_value(
                descriptor.const_zero(),
                self.context.i64_type().const_int(count, false),
                0,
                "slice.count",
            )?
            .into_struct_value();
        Ok(self
            .builder
            .build_insert_value(snapshot, pointer, 1, "slice.data")?
            .into_struct_value()
            .into())
    }
    pub(super) fn sequence_build(
        &mut self,
        ty: TypeId,
        initializers: &[(SequenceField, ValueExpr)],
    ) -> Result<BasicValueEnum<'ctx>, Error> {
        let descriptor = self.lowerer.basic(ty)?.into_struct_type();
        let mut snapshot = descriptor.const_zero();
        for (field, initializer) in initializers {
            let value = self.value(initializer)?;
            let index = sequence_ordinal(*field);
            snapshot = self
                .builder
                .build_insert_value(snapshot, value, index, "sequence.field")?
                .into_struct_value();
        }
        Ok(snapshot.into())
    }

    pub(super) fn array_view(
        &mut self,
        array: &ValueExpr,
        ty: TypeId,
    ) -> Result<BasicValueEnum<'ctx>, Error> {
        let TypeKind::FixedArray { count, .. } = self.types.kind(array.type_id(self.types))? else {
            return Err(Error::Invariant);
        };
        let count = *count;
        let snapshot = self.value(array)?;
        let storage = self.sequence_array_address(snapshot, jai_ir::is_static_value(array))?;
        let descriptor = self.lowerer.basic(ty)?.into_struct_type();
        let snapshot = self
            .builder
            .build_insert_value(
                descriptor.const_zero(),
                self.context.i64_type().const_int(count, false),
                0,
                "slice.count",
            )?
            .into_struct_value();
        Ok(self
            .builder
            .build_insert_value(snapshot, storage, 1, "slice.data")?
            .into_struct_value()
            .into())
    }

    pub(super) fn sequence_projection(
        &mut self,
        base: Place,
        field: SequenceField,
        ty: TypeId,
    ) -> Result<Slot<'ctx>, Error> {
        let source = self.slot(base)?;
        let structure = self.lowerer.basic(base.ty())?.into_struct_type();
        let pointer = self.builder.build_struct_gep(
            structure,
            source.pointer,
            sequence_ordinal(field),
            "sequence.field.address",
        )?;
        let offset = self
            .target
            .data
            .offset_of_element(&structure, sequence_ordinal(field))
            .ok_or(Error::Invariant)?;
        Ok(Slot {
            pointer,
            ty,
            alignment: memory::offset_alignment(source.alignment, offset),
        })
    }
    pub(super) fn sequence_view(
        &mut self,
        sequence: &ValueExpr,
        ty: TypeId,
    ) -> Result<BasicValueEnum<'ctx>, Error> {
        let source = self.value(sequence)?.into_struct_value();
        let count = self.builder.build_extract_value(source, 0, "slice.count")?;
        let data = self.builder.build_extract_value(source, 1, "slice.data")?;
        let descriptor = self.lowerer.basic(ty)?.into_struct_type();
        let snapshot = self
            .builder
            .build_insert_value(descriptor.const_zero(), count, 0, "slice.count")?
            .into_struct_value();
        Ok(self
            .builder
            .build_insert_value(snapshot, data, 1, "slice.data")?
            .into_struct_value()
            .into())
    }

    fn sequence_array_address(
        &mut self,
        snapshot: BasicValueEnum<'ctx>,
        require_static: bool,
    ) -> Result<PointerValue<'ctx>, Error> {
        let array = snapshot.into_array_value();
        if array.get_type().is_empty() {
            return Ok(self
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .const_null());
        }
        if snapshot.is_const() {
            return Ok(literal_global(self.module, snapshot, "jai.array.literal"));
        }
        // Source return checking relies on this classification to promise static
        // storage. A new lowering must preserve that promise before it can run.
        if require_static {
            return Err(Error::Invariant);
        }
        let storage = self
            .builder
            .build_alloca(snapshot.get_type(), "array.temporary")?;
        self.builder.build_store(storage, snapshot)?;
        Ok(storage)
    }
}

fn sequence_ordinal(field: SequenceField) -> u32 {
    match field {
        SequenceField::Count => 0,
        SequenceField::Data => 1,
        SequenceField::Allocated => 2,
    }
}

fn literal_global<'ctx>(
    module: &Module<'ctx>,
    value: BasicValueEnum<'ctx>,
    name: &str,
) -> PointerValue<'ctx> {
    if let Some(global) = module.get_globals().find(|global| {
        global.is_constant()
            && global.get_linkage() == Linkage::Private
            && global.get_initializer() == Some(value)
    }) {
        return global.as_pointer_value();
    }
    let global = module.add_global(value.get_type(), None, name);
    global.set_initializer(&value);
    global.set_constant(true);
    global.set_linkage(Linkage::Private);
    global.as_pointer_value()
}
