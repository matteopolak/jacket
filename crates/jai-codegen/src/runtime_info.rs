//! Publish genuine compiler-owned native tables and target-sized global ranges.
use super::*;
use inkwell::{module::Linkage, types::BasicType, values::GlobalValue};
use std::collections::HashSet;

const GLOBAL_DATA_VERSION: u64 = 1;

struct Pending<'ctx, 'a> {
    publication: &'a jai_ir::NativeRuntimeInfoPublication,
    binding: GlobalValue<'ctx>,
    table: BasicValueEnum<'ctx>,
    metadata: GlobalValue<'ctx>,
    segments: GlobalValue<'ctx>,
}

/// Run after every procedure and entry bridge has emitted its lazy static data.
/// Reserve the catalog's own globals before measuring the complete owned set.
pub(super) fn publish<'ctx>(
    library: &jai_ir::Library,
    reachable: &native_reachability::Reachable,
    lowerer: &mut types::TypeLowerer<'ctx, '_>,
    context: &'ctx Context,
    module: &Module<'ctx>,
    target: &target::NativeTarget,
    functions: &HashMap<jai_ir::ProcedureId, FunctionValue<'ctx>>,
) -> Result<(), Error> {
    let publications = library
        .native_runtime_info()
        .iter()
        .filter(|publication| reachable.contains(publication.procedure()))
        .collect::<Vec<_>>();
    if publications.is_empty() {
        return Ok(());
    }
    let mut prepared = Vec::new();
    for publication in publications {
        publication
            .snapshot()
            .revalidate(library.types(), target.layout_policy()?)
            .map_err(|error| Error::RuntimeInfo(error.to_string()))?;
        let table = static_data::runtime_info_table_constant(
            lowerer,
            publication.snapshot(),
            context,
            module,
            functions,
            library.signatures(),
        )?;
        let binding = module
            .get_global(publication.data().symbol())
            .ok_or(Error::Invariant)?;
        if binding.get_initializer().is_some() {
            return Err(Error::RuntimeInfo(
                "native runtime-info external already has owned storage".into(),
            ));
        }
        binding.set_constant(true);
        let schema = publication.schema();
        let metadata = owned_global(
            module,
            lowerer.basic(schema.global_data_type())?,
            &format!("jai.runtime.global-data.{}", publication.global().index()),
            target,
        )?;
        prepared.push((publication, binding, table, metadata));
    }
    // Each program external becomes a genuine definition. Include its bytes
    // even though the initializer is completed after computing these ranges.
    let known = prepared
        .iter()
        .flat_map(|(_, binding, _, metadata)| [*binding, *metadata])
        .collect::<HashSet<_>>();
    let count = module
        .get_globals()
        .filter(|global| global.get_initializer().is_some() || known.contains(global))
        .count()
        .checked_add(prepared.len())
        .ok_or(Error::Invariant)?;
    let count_u32 = u32::try_from(count).map_err(|_| Error::Invariant)?;
    let mut pending = Vec::new();
    for (publication, binding, table, metadata) in prepared {
        let segment_type = lowerer.basic(publication.schema().global_data_segment_type())?;
        let segments = owned_global(
            module,
            segment_type.array_type(count_u32).into(),
            &format!(
                "jai.runtime.global-segments.{}",
                publication.global().index()
            ),
            target,
        )?;
        pending.push(Pending {
            publication,
            binding,
            table,
            metadata,
            segments,
        });
    }
    let reserved = pending
        .iter()
        .flat_map(|item| [item.binding, item.metadata, item.segments])
        .collect::<HashSet<_>>();
    let owned = module
        .get_globals()
        .filter(|global| global.get_initializer().is_some() || reserved.contains(global))
        .collect::<Vec<_>>();
    if owned.len() != count {
        return Err(Error::Invariant);
    }
    for item in pending {
        let schema = item.publication.schema();
        let segment_type = schema.global_data_segment_type();
        let byte_slice = library.types().record_definition(segment_type)?.fields[1];
        let rows = owned
            .iter()
            .map(|global| {
                // Extern declarations have no storage and never enter `owned`.
                let bytes = target.data.get_abi_size(&global.get_value_type());
                let tag = if global.is_constant() {
                    2
                } else if global.get_initializer().is_some_and(is_zero) {
                    0
                } else {
                    1
                };
                let data = lowerer
                    .basic(byte_slice)?
                    .into_struct_type()
                    .const_named_struct(&[
                        context.i64_type().const_int(bytes, false).into(),
                        global.as_pointer_value().into(),
                    ]);
                record_constant(
                    lowerer,
                    context,
                    target,
                    segment_type,
                    &[context.i16_type().const_int(tag, false).into(), data.into()],
                )
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let rows = sequences::array_constant(lowerer.basic(segment_type)?, &rows)?;
        item.segments.set_initializer(&rows);
        let segment_slice_type = library
            .types()
            .record_definition(schema.global_data_type())?
            .fields[1];
        let segment_slice = lowerer
            .basic(segment_slice_type)?
            .into_struct_type()
            .const_named_struct(&[
                context.i64_type().const_int(count as u64, false).into(),
                item.segments.as_pointer_value().into(),
            ]);
        let metadata = record_constant(
            lowerer,
            context,
            target,
            schema.global_data_type(),
            &[
                context
                    .i64_type()
                    .const_int(GLOBAL_DATA_VERSION, false)
                    .into(),
                segment_slice.into(),
            ],
        )?;
        item.metadata.set_initializer(&metadata);
        let runtime = record_constant(
            lowerer,
            context,
            target,
            schema.ty(),
            &[item.table, item.metadata.as_pointer_value().into()],
        )?;
        item.binding.set_initializer(&runtime);
    }
    Ok(())
}

fn owned_global<'ctx>(
    module: &Module<'ctx>,
    ty: inkwell::types::BasicTypeEnum<'ctx>,
    name: &str,
    target: &target::NativeTarget,
) -> Result<GlobalValue<'ctx>, Error> {
    if module.get_global(name).is_some() || module.get_function(name).is_some() {
        return Err(Error::RuntimeInfo(format!(
            "native runtime-info storage symbol {name:?} conflicts"
        )));
    }
    let global = module.add_global(ty, None, name);
    global.set_constant(true);
    global.set_linkage(Linkage::Private);
    global.set_alignment(target.data.get_abi_alignment(&ty));
    Ok(global)
}

fn record_constant<'ctx>(
    lowerer: &mut types::TypeLowerer<'ctx, '_>,
    context: &'ctx Context,
    target: &target::NativeTarget,
    ty: TypeId,
    fields: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, Error> {
    let storage = lowerer.basic(ty)?.into_struct_type();
    let layout = lowerer.semantic_layout(ty)?;
    records::constant(context, &target.data, storage, fields, &layout)
}

fn is_zero(value: BasicValueEnum<'_>) -> bool {
    match value {
        BasicValueEnum::IntValue(value) => value.is_null(),
        BasicValueEnum::FloatValue(value) => value.is_null(),
        BasicValueEnum::PointerValue(value) => value.is_null(),
        BasicValueEnum::StructValue(value) => value.is_null(),
        BasicValueEnum::ArrayValue(value) => value.is_null(),
        BasicValueEnum::VectorValue(value) => value.is_null(),
        BasicValueEnum::ScalableVectorValue(_) => false,
    }
}
