//! Bind the actual nominal source flags enum before a VM call is admitted.
use super::*;

pub(in crate::modules::compiler_intrinsics) fn projection(
    signature: &jai_types::ProcedureType,
    graph: &ModuleGraph,
    declarations: &ScopedDeclarations<'_>,
    types: &TypeRegistry,
    context: &CompilerBindingContext,
) -> Result<TypeId, String> {
    let ([represented, flags], []) = (signature.parameters.as_ref(), signature.results.as_ref())
    else {
        return Err("compiler_set_type_info_flags requires (Type, Type_Info_Flags) -> void".into());
    };
    if *represented != types.meta_type() || !matches!(types.kind(*represented), Ok(TypeKind::Type))
    {
        return Err(
            "compiler_set_type_info_flags requires the canonical runtime Type operand".into(),
        );
    }
    let (&id, _) = declarations
        .nominals
        .declarations
        .iter()
        .find(|(id, actual)| {
            **actual == *flags
                && graph.declaration(**id).is_some_and(|declaration| {
                    graph.symbols().name(declaration.name()) == "Type_Info_Flags"
                        && matches!(declaration.syntax().kind, FileDeclarationKind::Enum(_))
                })
        })
        .ok_or("compiler_set_type_info_flags requires the source Type_Info_Flags enum")?;
    let declaration = graph.declaration(id).expect("checked nominal declaration");
    let module = graph
        .file(declaration.file())
        .expect("checked declaration file")
        .module();
    if context.origins.origin(module).is_none() {
        return Err("Type_Info_Flags does not belong to a selected compiler API origin".into());
    }
    let info = declarations
        .nominals
        .enums
        .get(flags)
        .ok_or("compiler_set_type_info_flags requires a checked source enum")?;
    let definition = types
        .enum_definition(*flags)
        .map_err(|error| error.to_string())?;
    if info.flags
        || info.representation != IntegerType::U32
        || definition.representation != IntegerType::U32
        || info.members.len() != 3
        || definition.values.len() != 3
    {
        return Err("Type_Info_Flags requires exactly the three source u32 enum members".into());
    }
    for (name, value) in [
        ("NO_TYPE_INFO", 1),
        ("PROCEDURES_ARE_VOID_POINTERS", 2),
        ("NO_SIZE_COMPLAINT", 4),
    ] {
        if !graph
            .symbols()
            .find(name)
            .and_then(|name| info.members.get(&name))
            .is_some_and(|actual| actual.value() == value)
        {
            return Err(format!(
                "Type_Info_Flags.{name} has an incompatible source value"
            ));
        }
    }
    Ok(*flags)
}
