//! Private exact source proof for inverse runtime descriptor lookup.
use super::*;

pub(in crate::modules::compiler_intrinsics) fn projection(
    signature: &jai_types::ProcedureType,
    graph: &ModuleGraph,
    declarations: &ScopedDeclarations<'_>,
    types: &TypeRegistry,
    context: &CompilerBindingContext,
) -> Result<jai_types::RuntimeTypeSchema, String> {
    let ([pointer], [result]) = (signature.parameters.as_ref(), signature.results.as_ref()) else {
        return Err("get_type requires one Type_Info pointer and one Type result".into());
    };
    if *result != types.meta_type() || !matches!(types.kind(*result), Ok(TypeKind::Type)) {
        return Err("get_type requires the canonical runtime Type result".into());
    }
    let TypeKind::Pointer(header) = types.kind(*pointer).map_err(|error| error.to_string())? else {
        return Err("get_type requires *Type_Info".into());
    };
    record(*header, "Type_Info", graph, declarations, context)?;
    let schema =
        jai_types::RuntimeTypeSchema::from_view(types).map_err(|error| error.to_string())?;
    if schema.header_type() != *header || schema.descriptor_type() != *pointer {
        return Err("get_type requires the actually adopted Type_Info header and pointer".into());
    }
    schema.validate(types).map_err(|error| error.to_string())?;
    Ok(schema)
}
