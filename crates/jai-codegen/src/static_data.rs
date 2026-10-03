//! Lazily emitted private globals for immutable compiler-owned object graphs.
use super::*;
use inkwell::{
    module::Linkage,
    values::{GlobalValue, PointerValue},
};
use jai_ir::{
    StaticAddress, StaticData, StaticObjectId, StaticProjection, StaticValue, StaticValueKind,
};
use jai_types::TypeKind;

struct StaticEmitter<'ctx, 'types, 'storage> {
    types: &'types Types,
    lowerer: &'storage mut types::TypeLowerer<'ctx, 'types>,
    context: &'ctx Context,
    module: &'storage Module<'ctx>,
    functions: &'storage HashMap<jai_ir::ProcedureId, FunctionValue<'ctx>>,
    signatures: &'storage HashMap<jai_ir::ProcedureId, TypeId>,
}

pub(super) fn address<'ctx>(
    generator: &mut Generator<'ctx, '_, '_>,
    data: &StaticData,
    address: &StaticAddress,
    ty: TypeId,
) -> Result<BasicValueEnum<'ctx>, Error> {
    let mut emitter = StaticEmitter {
        types: generator.types,
        lowerer: generator.lowerer,
        context: generator.context,
        module: generator.module,
        functions: generator.functions,
        signatures: generator.signatures,
    };
    emit_address(&mut emitter, data, address, ty)
}

/// Descriptor relocations are LLVM constants and also initialize global Type cells.
pub(super) fn runtime_type_constant<'ctx>(
    lowerer: &mut types::TypeLowerer<'ctx, '_>,
    value: &jai_ir::RuntimeTypeConstant,
    context: &'ctx Context,
    module: &Module<'ctx>,
    functions: &HashMap<jai_ir::ProcedureId, FunctionValue<'ctx>>,
    signatures: &HashMap<jai_ir::ProcedureId, TypeId>,
) -> Result<BasicValueEnum<'ctx>, Error> {
    value
        .validate(lowerer.registry())
        .map_err(|_| Error::Invariant)?;
    let mut emitter = StaticEmitter {
        types: lowerer.registry(),
        lowerer,
        context,
        module,
        functions,
        signatures,
    };
    emit_address(
        &mut emitter,
        value.data(),
        value.address(),
        value.descriptor_type(),
    )
}

/// Reuse the certified table's real descriptor relocations for native runtime
/// publication. Its compile-time global-data null is deliberately not copied.
pub(super) fn runtime_info_table_constant<'ctx>(
    lowerer: &mut types::TypeLowerer<'ctx, '_>,
    snapshot: &jai_ir::RuntimeInfoSnapshot,
    context: &'ctx Context,
    module: &Module<'ctx>,
    functions: &HashMap<jai_ir::ProcedureId, FunctionValue<'ctx>>,
    signatures: &HashMap<jai_ir::ProcedureId, TypeId>,
) -> Result<BasicValueEnum<'ctx>, Error> {
    let target = lowerer.target_data().ok_or(Error::Invariant)?;
    snapshot
        .revalidate(lowerer.registry(), types::layout_policy(context, target)?)
        .map_err(|_| Error::Invariant)?;
    let data = snapshot.data();
    let object = data
        .object(snapshot.address().object())
        .map_err(|_| Error::Invariant)?;
    let StaticValueKind::Record(fields) = &object.value().kind else {
        return Err(Error::Invariant);
    };
    let table = fields.first().ok_or(Error::Invariant)?;
    let mut emitter = StaticEmitter {
        types: lowerer.registry(),
        lowerer,
        context,
        module,
        functions,
        signatures,
    };
    let globals = reserve(&mut emitter, data)?;
    for object in data.objects() {
        let global = globals[&object.id()];
        if global.get_initializer().is_none() {
            let value = constant(&mut emitter, data, &globals, object.value())?;
            global.set_initializer(&value);
        }
    }
    constant(&mut emitter, data, &globals, table)
}

fn emit_address<'ctx>(
    generator: &mut StaticEmitter<'ctx, '_, '_>,
    data: &StaticData,
    address: &StaticAddress,
    ty: TypeId,
) -> Result<BasicValueEnum<'ctx>, Error> {
    data.validate(generator.types)
        .map_err(|_| Error::Invariant)?;
    if let Some(first) = data
        .objects()
        .iter()
        .find_map(|object| object.runtime_type_identity())
    {
        let target = generator
            .lowerer
            .target_data()
            .ok_or(types::Error::MissingDescriptorTarget(first.ty()))?;
        let policy = types::layout_policy(generator.context, target)?;
        if let Some(identity) = data
            .objects()
            .iter()
            .filter_map(|object| object.runtime_type_identity())
            .find(|identity| identity.policy() != policy)
        {
            return Err(types::Error::DescriptorTargetMismatch {
                ty: identity.ty(),
                descriptor: Box::new(identity.policy()),
                target: Box::new(policy),
            }
            .into());
        }
    }
    let TypeKind::Pointer(pointee) = *generator.types.kind(ty)? else {
        return Err(Error::Invariant);
    };
    if data
        .address_type(address, generator.types)
        .map_err(|_| Error::Invariant)?
        != pointee
    {
        return Err(Error::Invariant);
    }
    let globals = reserve(generator, data)?;
    for object in data.objects() {
        let global = globals[&object.id()];
        if global.get_initializer().is_none() {
            let value = constant(generator, data, &globals, object.value())?;
            if value.get_type() != generator.lowerer.basic(object.ty())? {
                return Err(Error::Invariant);
            }
            global.set_initializer(&value);
        }
    }
    Ok(project(generator, data, &globals, address)?.into())
}

fn reserve<'ctx>(
    generator: &mut StaticEmitter<'ctx, '_, '_>,
    data: &StaticData,
) -> Result<HashMap<StaticObjectId, GlobalValue<'ctx>>, Error> {
    let mut globals = HashMap::with_capacity(data.objects().len());
    for object in data.objects() {
        let name = format!("jai.static.{}.{}", data.identity(), object.id().index());
        let ty = generator.lowerer.basic(object.ty())?;
        let global = match generator.module.get_global(&name) {
            Some(global) => global,
            None => {
                let global = generator.module.add_global(ty, None, &name);
                global.set_constant(true);
                global.set_linkage(Linkage::Private);
                global
            }
        };
        globals.insert(object.id(), global);
    }
    Ok(globals)
}

fn constant<'ctx>(
    generator: &mut StaticEmitter<'ctx, '_, '_>,
    data: &StaticData,
    globals: &HashMap<StaticObjectId, GlobalValue<'ctx>>,
    value: &StaticValue,
) -> Result<BasicValueEnum<'ctx>, Error> {
    let ty = generator.lowerer.basic(value.ty)?;
    Ok(match &value.kind {
        StaticValueKind::Constant(value) => aggregates::constant(
            generator.lowerer,
            value,
            generator.context,
            generator.module,
            generator.functions,
            generator.signatures,
        )?,
        StaticValueKind::Address(address) => project(generator, data, globals, address)?.into(),
        StaticValueKind::Record(fields) => {
            let fields = fields
                .iter()
                .map(|field| constant(generator, data, globals, field))
                .collect::<Result<Vec<_>, _>>()?;
            let layout = generator.lowerer.semantic_layout(value.ty)?;
            records::constant(
                generator.context,
                generator.lowerer.target_data().ok_or(Error::Invariant)?,
                ty.into_struct_type(),
                &fields,
                &layout,
            )?
        }
        StaticValueKind::Array(elements) => {
            let elements = elements
                .iter()
                .map(|element| constant(generator, data, globals, element))
                .collect::<Result<Vec<_>, _>>()?;
            sequences::array_constant(ty.into_array_type().get_element_type(), &elements)?
        }
        StaticValueKind::Slice {
            data: address,
            count,
        } => {
            let pointer = match address {
                Some(address) => project(generator, data, globals, address)?,
                None => generator
                    .context
                    .ptr_type(inkwell::AddressSpace::default())
                    .const_null(),
            };
            ty.into_struct_type()
                .const_named_struct(&[
                    generator.context.i64_type().const_int(*count, false).into(),
                    pointer.into(),
                ])
                .into()
        }
    })
}

fn project<'ctx>(
    generator: &mut StaticEmitter<'ctx, '_, '_>,
    data: &StaticData,
    globals: &HashMap<StaticObjectId, GlobalValue<'ctx>>,
    address: &StaticAddress,
) -> Result<PointerValue<'ctx>, Error> {
    let object = data
        .object(address.object())
        .map_err(|_| Error::Invariant)?;
    let mut ty = object.ty();
    let mut pointer = globals
        .get(&object.id())
        .ok_or(Error::Invariant)?
        .as_pointer_value();
    for projection in address.path() {
        let physical = generator.lowerer.basic(ty)?;
        let index = match projection {
            StaticProjection::Field(field) => {
                let next = generator.types.validate_field(ty, *field)?;
                let layout = generator.lowerer.semantic_layout(ty)?;
                let offset = *layout
                    .field_offsets
                    .get(field.index())
                    .ok_or(Error::Invariant)?;
                pointer = jai_llvm::const_gep(
                    generator.context.i8_type().into(),
                    pointer,
                    &[generator.context.i64_type().const_int(offset, false)],
                )
                .map_err(|_| Error::Invariant)?;
                ty = next;
                continue;
            }
            StaticProjection::ByteView(view) => {
                let target = generator.lowerer.target_data().ok_or(Error::Invariant)?;
                view.validate_target(types::layout_policy(generator.context, target)?)
                    .map_err(Error::StaticByteView)?;
                pointer = jai_llvm::const_gep(
                    generator.context.i8_type().into(),
                    pointer,
                    &[generator.context.i64_type().const_int(view.offset(), false)],
                )
                .map_err(|_| Error::Invariant)?;
                ty = generator
                    .types
                    .scalar(jai_types::ScalarType::Int(jai_types::IntegerType::U8));
                continue;
            }
            StaticProjection::Index(index) => {
                let TypeKind::FixedArray { element, count } = *generator.types.kind(ty)? else {
                    return Err(Error::Invariant);
                };
                if *index >= count {
                    return Err(Error::Invariant);
                }
                ty = element;
                generator.context.i64_type().const_int(*index, false)
            }
        };
        pointer = jai_llvm::const_gep(
            physical,
            pointer,
            &[generator.context.i64_type().const_zero(), index],
        )
        .map_err(|_| Error::Invariant)?;
    }
    Ok(pointer)
}
