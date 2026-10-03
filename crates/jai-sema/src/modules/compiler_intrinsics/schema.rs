//! Validate nominal source API schemas before projecting typed fields.
use super::*;

fn record<'a, 'g>(
    ty: TypeId,
    name: &str,
    graph: &ModuleGraph,
    declarations: &'a ScopedDeclarations<'g>,
    context: &CompilerBindingContext,
) -> Result<&'a aggregates::types::RecordInfo<'g>, String> {
    let record = declarations
        .nominals
        .records
        .get(&ty)
        .ok_or_else(|| format!("requires the source `{name}` struct"))?;
    let declaration = graph
        .declaration(record.declaration)
        .expect("nominal declaration belongs to graph");
    if graph.symbols().name(declaration.name()) != name
        || record.kind != jai_types::RecordKind::Struct
    {
        return Err(format!("requires the source `{name}` struct"));
    }
    let module = graph
        .file(record.file)
        .expect("record belongs to graph")
        .module();
    if context.origins.origin(module).is_none() {
        return Err(format!(
            "`{name}` does not belong to a selected compiler API origin"
        ));
    }
    Ok(record)
}

pub(super) fn validate_location(
    ty: TypeId,
    graph: &ModuleGraph,
    declarations: &ScopedDeclarations<'_>,
    types: &TypeRegistry,
    context: &CompilerBindingContext,
) -> Result<(), String> {
    record(ty, "Source_Code_Location", graph, declarations, context)?;
    crate::caller_locations::validate_target(
        graph,
        &declarations.nominals,
        types,
        ty,
        Span::default(),
    )
    .map_err(|error| error.message)
}

pub(super) fn validate_version(
    ty: TypeId,
    graph: &ModuleGraph,
    declarations: &ScopedDeclarations<'_>,
    types: &TypeRegistry,
    context: &CompilerBindingContext,
) -> Result<(), String> {
    let record = record(ty, "Version_Info", graph, declarations, context)?;
    if record.fields.len() != 3 {
        return Err("Version_Info requires exactly s32 major, minor, and micro fields".into());
    }
    for (field, name) in record.fields.iter().zip(["major", "minor", "micro"]) {
        if graph.symbols().name(field.name) != name
            || !matches!(types.validate_field(ty, field.id), Ok(actual) if actual == field.ty)
            || !matches!(
                types.kind(field.ty),
                Ok(TypeKind::Integer(IntegerType::S32))
            )
        {
            return Err(
                "Version_Info requires exactly s32 major, minor, and micro fields in source order"
                    .into(),
            );
        }
    }
    Ok(())
}

pub(super) fn validate_enum(
    ty: TypeId,
    name: &str,
    members: &[&str],
    graph: &ModuleGraph,
    declarations: &ScopedDeclarations<'_>,
    types: &TypeRegistry,
    context: &CompilerBindingContext,
) -> Result<(), String> {
    let (&id, _) = declarations
        .nominals
        .declarations
        .iter()
        .find(|(id, actual)| {
            **actual == ty
                && graph.declaration(**id).is_some_and(|declaration| {
                    graph.symbols().name(declaration.name()) == name
                        && matches!(declaration.syntax().kind, FileDeclarationKind::Enum(_))
                })
        })
        .ok_or_else(|| format!("requires source `{name}` enum"))?;
    let declaration = graph.declaration(id).expect("nominal belongs to graph");
    let module = graph
        .file(declaration.file())
        .expect("declaration belongs to graph")
        .module();
    if graph.symbols().name(declaration.name()) != name || context.origins.origin(module).is_none()
    {
        return Err(format!(
            "requires source `{name}` enum from a selected compiler API origin"
        ));
    }
    let info = declarations
        .nominals
        .enums
        .get(&ty)
        .ok_or_else(|| format!("requires source `{name}` enum"))?;
    if info.flags
        || info.representation != IntegerType::U8
        || info.members.len() != members.len()
        || types.enum_definition(ty).is_err()
    {
        return Err(format!("`{name}` must use the source u8 enum definition"));
    }
    for (value, name) in members.iter().enumerate() {
        if !graph
            .symbols()
            .find(name)
            .and_then(|name| info.members.get(&name))
            .is_some_and(|integer| integer.value() == value as i128)
        {
            return Err(format!("`{name}` has an incompatible source enum value"));
        }
    }
    Ok(())
}

mod build_options;
pub(super) use build_options::build_options_projection;
pub(super) mod messages;
pub(super) mod runtime_info;
