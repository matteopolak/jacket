//! Prove selected source identities before accepting a canonical runtime-info schema.
use super::*;

pub(in crate::modules::compiler_intrinsics) fn projection(
    ty: TypeId,
    graph: &ModuleGraph,
    declarations: &ScopedDeclarations<'_>,
    types: &TypeRegistry,
    context: &CompilerBindingContext,
    meta: &crate::reflection::MetaContext,
) -> Result<jai_types::RuntimeInfoSchema, String> {
    let info = record(ty, "Runtime_Info", graph, declarations, context)?;
    if info.fields.len() != 2
        || graph.symbols().name(info.fields[0].name) != "type_table"
        || graph.symbols().name(info.fields[1].name) != "global_data_info"
    {
        return Err(
            "Runtime_Info requires its exact type_table and global_data_info fields".into(),
        );
    }
    let TypeKind::Slice(element) = types.kind(info.fields[0].ty).map_err(|e| e.to_string())? else {
        return Err("Runtime_Info.type_table requires [] *Type_Info".into());
    };
    let TypeKind::Pointer(header) = types.kind(*element).map_err(|e| e.to_string())? else {
        return Err("Runtime_Info.type_table requires [] *Type_Info".into());
    };
    record(*header, "Type_Info", graph, declarations, context)?;
    if types.runtime_type_header() != Some(*header) {
        return Err("Runtime_Info requires the actually adopted Type_Info header identity".into());
    }
    let TypeKind::Pointer(global) = types.kind(info.fields[1].ty).map_err(|e| e.to_string())?
    else {
        return Err("Runtime_Info.global_data_info requires *Global_Data_Info".into());
    };
    let data = record(*global, "Global_Data_Info", graph, declarations, context)?;
    if data.fields.len() != 2
        || graph.symbols().name(data.fields[0].name) != "version_stamp"
        || graph.symbols().name(data.fields[1].name) != "segment_info"
        || !matches!(
            types.kind(data.fields[0].ty),
            Ok(TypeKind::Integer(IntegerType::U64))
        )
    {
        return Err("Global_Data_Info requires u64 version_stamp and segment_info fields".into());
    }
    let TypeKind::Slice(segment) = types.kind(data.fields[1].ty).map_err(|e| e.to_string())? else {
        return Err("Global_Data_Info.segment_info requires [] Global_Data_Segment_Info".into());
    };
    let segments = record(
        *segment,
        "Global_Data_Segment_Info",
        graph,
        declarations,
        context,
    )?;
    if segments.fields.len() != 2
        || graph.symbols().name(segments.fields[0].name) != "segment_tag"
        || graph.symbols().name(segments.fields[1].name) != "data"
        || !matches!(types.kind(segments.fields[1].ty), Ok(TypeKind::Slice(element)) if matches!(types.kind(*element), Ok(TypeKind::Integer(IntegerType::U8))))
    {
        return Err(
            "Global_Data_Segment_Info requires exact segment_tag and []u8 data fields".into(),
        );
    }
    super::messages::enumeration(
        segments.fields[0].ty,
        (
            IntegerType::U16,
            false,
            &[
                ("BSS", 0),
                ("DATA", 1),
                ("RDATA", 2),
                ("NO_RESET", 3),
                ("USER", 5),
            ],
        ),
        graph,
        declarations,
        types,
        meta,
    )?;
    let module = graph
        .file(info.file)
        .expect("source record belongs to graph")
        .module();
    for (record_ty, shape) in [(ty, info), (*global, data), (*segment, segments)] {
        if graph
            .file(shape.file)
            .expect("source record belongs to graph")
            .module()
            != module
        {
            return Err(
                "Runtime_Info and global-data schemas require one selected source module".into(),
            );
        }
        for field in &shape.fields {
            if types
                .validate_field(record_ty, field.id)
                .map_err(|e| e.to_string())?
                != field.ty
            {
                return Err("runtime-info source field differs from its canonical owner".into());
            }
        }
    }
    jai_types::RuntimeInfoSchema::validate(types, ty, *global).map_err(|e| e.to_string())
}
