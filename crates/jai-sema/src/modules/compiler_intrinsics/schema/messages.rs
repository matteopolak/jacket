//! Exact source event schemas preserve inline enums and embedded base identities.
use super::*;
use jai_vm::CompilerMessageSchema;

pub(super) fn enumeration(
    ty: TypeId,
    expected: (IntegerType, bool, &[(&str, i128)]),
    graph: &ModuleGraph,
    declarations: &ScopedDeclarations<'_>,
    types: &TypeRegistry,
    meta: &crate::reflection::MetaContext,
) -> Result<(), String> {
    let (representation, flags, members) = expected;
    let (actual_representation, actual_flags, values) =
        if let Some(info) = declarations.nominals.enums.get(&ty) {
            (
                info.representation,
                info.flags,
                info.members
                    .iter()
                    .map(|(&name, &value)| (name, value))
                    .collect::<Vec<_>>(),
            )
        } else if let Some(info) = meta.record_specializations.member_enum(ty) {
            (info.representation, info.flags, info.values.clone())
        } else {
            return Err("compiler message field requires its checked source enum".into());
        };
    if actual_representation != representation
        || actual_flags != flags
        || values.len() != members.len()
        || types.enum_definition(ty).is_err()
    {
        return Err(
            "compiler message enum representation or members differ from its source schema".into(),
        );
    }
    for &(name, expected) in members {
        if !values.iter().any(|&(symbol, value)| {
            graph.symbols().name(symbol) == name && value.value() == expected
        }) {
            return Err(format!(
                "compiler message enum has an incompatible `{name}` value"
            ));
        }
    }
    Ok(())
}

pub(in crate::modules::compiler_intrinsics) fn intercept_flags(
    ty: TypeId,
    graph: &ModuleGraph,
    declarations: &ScopedDeclarations<'_>,
    types: &TypeRegistry,
    context: &CompilerBindingContext,
    meta: &crate::reflection::MetaContext,
) -> Result<(), String> {
    let declaration = declarations
        .nominals
        .declarations
        .iter()
        .find_map(|(&id, &actual)| {
            (actual == ty)
                .then(|| graph.declaration(id))
                .flatten()
                .filter(|declaration| graph.symbols().name(declaration.name()) == "Intercept_Flags")
        })
        .ok_or_else(|| "requires source Intercept_Flags enum_flags u32".to_owned())?;
    let module = graph
        .file(declaration.file())
        .expect("source declaration belongs to graph")
        .module();
    if context.origins.origin(module).is_none() {
        return Err("Intercept_Flags does not belong to a selected compiler API origin".into());
    }
    enumeration(
        ty,
        (
            IntegerType::U32,
            true,
            &[
                ("SKIP_EXPRESSIONS_WITHOUT_NOTES", 1),
                ("SKIP_DECLARATIONS", 2),
                ("SKIP_PROCEDURE_HEADERS", 4),
                ("SKIP_PROCEDURE_BODIES", 8),
                ("SKIP_STRUCTS", 16),
                ("SKIP_OTHERS", 32),
                ("SKIP_ALL", 63),
                ("DO_PERFORMANCE_REPORT_POLYMORPHS", 0x1000),
                ("DO_PERFORMANCE_REPORT_RUNS", 0x2000),
            ],
        ),
        graph,
        declarations,
        types,
        meta,
    )
}

fn sibling(
    message: TypeId,
    name: &str,
    graph: &ModuleGraph,
    declarations: &ScopedDeclarations<'_>,
) -> Result<TypeId, String> {
    let base = declarations
        .nominals
        .records
        .get(&message)
        .expect("checked source Message");
    let module = graph
        .file(base.file)
        .expect("source record belongs to graph")
        .module();
    let mut matches = declarations
        .nominals
        .records
        .iter()
        .filter(|(_, record)| {
            graph
                .file(record.file)
                .is_some_and(|file| file.module() == module)
                && graph
                    .declaration(record.declaration)
                    .is_some_and(|declaration| graph.symbols().name(declaration.name()) == name)
        })
        .map(|(&ty, _)| ty);
    let ty = matches
        .next()
        .ok_or_else(|| format!("requires source `{name}` in the Message module"))?;
    if matches.next().is_some() {
        return Err(format!(
            "source `{name}` is ambiguous in the Message module"
        ));
    }
    Ok(ty)
}

pub(in crate::modules::compiler_intrinsics) fn projection(
    message: TypeId,
    graph: &ModuleGraph,
    declarations: &ScopedDeclarations<'_>,
    types: &TypeRegistry,
    context: &CompilerBindingContext,
    meta: &crate::reflection::MetaContext,
) -> Result<CompilerMessageSchema, String> {
    let base = record(message, "Message", graph, declarations, context)?;
    if base.fields.len() != 2
        || graph.symbols().name(base.fields[0].name) != "kind"
        || graph.symbols().name(base.fields[1].name) != "workspace"
        || !matches!(
            types.kind(base.fields[1].ty),
            Ok(TypeKind::Integer(IntegerType::S64))
        )
    {
        return Err("Message requires exact kind and s64 workspace source fields".into());
    }
    enumeration(
        base.fields[0].ty,
        (
            IntegerType::U8,
            false,
            &[
                ("UNINITIALIZED", 0),
                ("FILE", 1),
                ("IMPORT", 2),
                ("FAILED_IMPORT", 3),
                ("PHASE", 4),
                ("TYPECHECKED", 5),
                ("COMPLETE", 6),
                ("DEBUG_DUMP", 7),
                ("ERROR", 8),
                ("PERFORMANCE_REPORT", 9),
            ],
        ),
        graph,
        declarations,
        types,
        meta,
    )?;
    let phase = sibling(message, "Message_Phase", graph, declarations)?;
    let complete = sibling(message, "Message_Complete", graph, declarations)?;
    let phase_record = record(phase, "Message_Phase", graph, declarations, context)?;
    let complete_record = record(complete, "Message_Complete", graph, declarations, context)?;
    let expected_phase = [
        "m",
        "phase",
        "executable_name",
        "executable_write_failed",
        "linker_exit_code",
        "num_items_waiting_to_typecheck",
        "compiler_generated_object_files",
        "support_object_files",
        "system_libraries",
        "user_libraries",
    ];
    for (fields, names) in [
        (&phase_record.fields, expected_phase.as_slice()),
        (&complete_record.fields, ["m", "error_code"].as_slice()),
    ] {
        if fields.len() != names.len()
            || fields
                .iter()
                .zip(names)
                .any(|(field, name)| graph.symbols().name(field.name) != *name)
        {
            return Err("compiler event fields differ from their source schema".into());
        }
        if fields[0].ty != message
            || !fields[0].syntax.using
            || fields[0].syntax.conversion != syntax::FieldConversion::Implicit
        {
            return Err("compiler event requires its actual #as using Message base field".into());
        }
    }
    enumeration(
        phase_record.fields[1].ty,
        (
            IntegerType::U32,
            false,
            &[
                ("ALL_SOURCE_CODE_PARSED", 0),
                ("TYPECHECKED_ALL_WE_CAN", 1),
                ("ALL_TARGET_CODE_BUILT", 2),
                ("PRE_WRITE_EXECUTABLE", 3),
                ("POST_WRITE_EXECUTABLE", 4),
                ("READY_FOR_CUSTOM_LINK_COMMAND", 5),
            ],
        ),
        graph,
        declarations,
        types,
        meta,
    )?;
    enumeration(
        complete_record.fields[1].ty,
        (
            IntegerType::U8,
            false,
            &[
                ("NONE", 0),
                ("COMPILATION_FAILED", 1),
                ("COMPILER_SHUTDOWN", 2),
            ],
        ),
        graph,
        declarations,
        types,
        meta,
    )?;
    if !matches!(types.kind(phase_record.fields[2].ty), Ok(TypeKind::String))
        || !matches!(types.kind(phase_record.fields[3].ty), Ok(TypeKind::Bool))
        || [4,5].iter().any(|&index| !matches!(types.kind(phase_record.fields[index].ty), Ok(TypeKind::Integer(IntegerType::S32))))
        || phase_record.fields[6..].iter().any(|field| !matches!(types.kind(field.ty), Ok(TypeKind::Slice(element)) if matches!(types.kind(*element), Ok(TypeKind::String)))) {
        return Err("Message_Phase scalar or string-array fields differ from their source schema".into());
    }
    for (ty, record) in [
        (message, base),
        (phase, phase_record),
        (complete, complete_record),
    ] {
        for field in &record.fields {
            types
                .validate_field(ty, field.id)
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(CompilerMessageSchema {
        message,
        kind: base.fields[0].id,
        workspace: base.fields[1].id,
        phase,
        phase_base: phase_record.fields[0].id,
        phase_kind: phase_record.fields[1].id,
        pending_count: phase_record.fields[5].id,
        complete,
        complete_base: complete_record.fields[0].id,
        completion_error: complete_record.fields[1].id,
    })
}
