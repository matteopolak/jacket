//! Apply only fields projected from a checked source Build_Options schema.
use super::*;
use jai_types::{Integer, IntegerType, TypeView};

pub(super) fn invoke_get_options(
    projection: BuildOptionsProjection,
    result: TypeId,
    arguments: &[Value],
    current_workspace: WorkspaceId,
    effects: &mut impl CompilerEffects,
    types: &dyn TypeView,
) -> Result<Vec<Value>, EffectError> {
    let fail = |message| EffectError::Failed(Error::InvalidIr(message));
    if projection == BuildOptionsProjection::default() {
        return Err(fail(
            "get_build_options requires at least one checked option field",
        ));
    }
    let [workspace] = arguments else {
        return Err(fail("get_build_options requires a source s64 workspace"));
    };
    let workspace = super::source::workspace_argument(workspace, current_workspace)?;
    let snapshot = match effects.request(CompilerRequest::GetBuildOptions {
        workspace,
    }) {
        EffectOutcome::Ready(CompilerResponse::BuildOptions(snapshot)) => snapshot,
        EffectOutcome::Ready(_) => {
            return Err(EffectError::Failed(Error::EffectResponse(
                "get_build_options expected a settings snapshot",
            )));
        }
        EffectOutcome::Pending(key) => return Err(EffectError::Pending(Dependency::Effect(key))),
        EffectOutcome::Rejected(reason) => {
            return Err(EffectError::Failed(Error::EffectRejected(reason)));
        }
    };
    let mut record = empty_options(result, types, 0)?;
    if let Some(path) = projection.output_kind {
        assign_enum(
            &mut record,
            path,
            match snapshot.output_kind {
                BuildOutputKind::None => 0,
                BuildOutputKind::Executable => 1,
                BuildOutputKind::DynamicLibrary => 2,
                BuildOutputKind::StaticLibrary => 3,
                BuildOutputKind::Object => 4,
            },
            types,
        )?;
    }
    if let Some(path) = projection.runtime_support {
        assign_enum(
            &mut record,
            path,
            match snapshot.runtime_support {
                RuntimeSupportMode::Auto => 0,
                RuntimeSupportMode::EntryPointAndInitialization => 1,
                RuntimeSupportMode::InitializationOnly => 2,
                RuntimeSupportMode::Omit => 3,
            },
            types,
        )?;
    }
    if let Some(path) = projection.temporary_storage_size {
        assign(
            &mut record,
            path,
            Value::Int(
                Integer::checked(IntegerType::S32, snapshot.temporary_storage_size as i128)
                    .expect("retained temporary-storage setting fits source s32"),
            ),
            types,
        )?;
    }
    if let Some(path) = projection.backtrace_on_crash {
        assign_enum(
            &mut record,
            path,
            match snapshot.backtrace_on_crash {
                BacktraceOnCrash::Off => 0,
                BacktraceOnCrash::On => 1,
            },
            types,
        )?;
    }
    if let Some(path) = projection.output_path {
        let text = match snapshot.output_path {
            Some(path) => path
                .to_str()
                .ok_or_else(|| {
                    fail("compiler output path cannot be represented as UTF-8 source text")
                })?
                .to_owned(),
            None => String::new(),
        };
        assign(&mut record, path, Value::String(text.into_bytes()), types)?;
    }
    if let Some(path) = projection.target {
        let text = snapshot
            .target
            .map(|target| target.as_str().to_owned())
            .unwrap_or_default();
        assign(&mut record, path, Value::String(text.into_bytes()), types)?;
    }
    if let Some(path) = projection.bitcode {
        let level = match snapshot.bitcode {
            BitcodeOptimization::Unset => 0,
            BitcodeOptimization::O0 => 1,
            BitcodeOptimization::O1 => 2,
            BitcodeOptimization::O2 => 3,
            BitcodeOptimization::O3 => 4,
            BitcodeOptimization::Os => 5,
            BitcodeOptimization::Oz => 6,
        };
        assign_enum(&mut record, path, level, types)?;
    }
    if let Some(path) = projection.machine {
        let level = match snapshot.machine {
            MachineOptimization::Unset => 0,
            MachineOptimization::None => 1,
            MachineOptimization::Less => 2,
            MachineOptimization::Default => 3,
            MachineOptimization::Aggressive => 4,
        };
        assign_enum(&mut record, path, level, types)?;
    }
    record
        .validate(types, result, 4)
        .map_err(EffectError::Failed)?;
    Ok(vec![record])
}

fn empty_options(ty: TypeId, types: &dyn TypeView, depth: usize) -> Result<Value, EffectError> {
    let fail = |message| EffectError::Failed(Error::InvalidIr(message));
    if depth > 3 {
        return Err(fail("get_build_options schema exceeds supported nesting"));
    }
    match types
        .kind(ty)
        .map_err(|error| EffectError::Failed(Error::Type(error)))?
    {
        jai_types::TypeKind::String => Ok(Value::String(vec![])),
        jai_types::TypeKind::Integer(integer) => Ok(Value::Int(
            Integer::checked(*integer, 0).expect("zero fits every source integer type"),
        )),
        jai_types::TypeKind::Bool => Ok(Value::Bool(false)),
        jai_types::TypeKind::Enum(id) => {
            let definition = types
                .enumeration(*id)
                .map_err(|error| EffectError::Failed(Error::Type(error)))?;
            if definition.representation != IntegerType::U8 {
                return Err(fail(
                    "get_build_options enum requires source u8 representation",
                ));
            }
            Ok(Value::Enum {
                ty,
                value: Integer::checked(IntegerType::U8, 0).expect("zero fits u8"),
            })
        }
        jai_types::TypeKind::Record(id) => {
            let definition = types
                .record(*id)
                .map_err(|error| EffectError::Failed(Error::Type(error)))?;
            if definition.kind != jai_types::RecordKind::Struct {
                return Err(fail("get_build_options requires source struct records"));
            }
            let fields = definition
                .fields
                .iter()
                .map(|&field| empty_options(field, types, depth + 1))
                .collect::<Result<_, _>>()?;
            Ok(Value::Record {
                ty,
                fields,
            })
        }
        _ => Err(fail("get_build_options has an unsupported schema field")),
    }
}

fn assign_enum(
    record: &mut Value,
    path: RecordFieldPath,
    level: i128,
    types: &dyn TypeView,
) -> Result<(), EffectError> {
    let Value::Enum {
        ty, ..
    } = field(record, path, types)?
    else {
        return Err(EffectError::Failed(Error::InvalidIr(
            "get_build_options optimization projection requires an enum",
        )));
    };
    let value = Value::Enum {
        ty: *ty,
        value: Integer::checked(IntegerType::U8, level)
            .expect("checked source optimization values fit u8"),
    };
    assign(record, path, value, types)
}

fn assign(
    record: &mut Value,
    path: RecordFieldPath,
    value: Value,
    types: &dyn TypeView,
) -> Result<(), EffectError> {
    fn extract<'a>(
        record: &'a mut Value,
        field: jai_types::FieldId,
        types: &dyn TypeView,
    ) -> Result<(&'a mut Value, TypeId), EffectError> {
        let Value::Record {
            ty,
            fields,
        } = record
        else {
            return Err(EffectError::Failed(Error::InvalidIr(
                "get_build_options projection requires a struct record",
            )));
        };
        let expected = types
            .validate_field(*ty, field)
            .map_err(|error| EffectError::Failed(Error::Type(error)))?;
        let field = fields.get_mut(field.index()).ok_or({
            EffectError::Failed(Error::InvalidIr(
                "get_build_options record is missing a checked field",
            ))
        })?;
        Ok((field, expected))
    }
    let (mut destination, mut expected) = extract(record, path.outer, types)?;
    if let Some(inner) = path.inner {
        (destination, expected) = extract(destination, inner, types)?;
    }
    if let Some(leaf) = path.leaf {
        if path.inner.is_none() {
            return Err(EffectError::Failed(Error::InvalidIr(
                "build option leaf requires a checked middle field",
            )));
        }
        (destination, expected) = extract(destination, leaf, types)?;
    }
    value
        .validate(types, expected, 4)
        .map_err(EffectError::Failed)?;
    *destination = value;
    Ok(())
}

pub(super) fn invoke_options(
    projection: BuildOptionsProjection,
    arguments: &[Value],
    current_workspace: WorkspaceId,
    effects: &mut impl CompilerEffects,
    types: &dyn TypeView,
) -> Result<Vec<Value>, EffectError> {
    invoke_options_with_location(
        projection,
        arguments,
        current_workspace,
        effects,
        types,
        None,
    )
}

pub(super) fn invoke_options_at(
    projection: BuildOptionsProjection,
    arguments: &[Value],
    current_workspace: WorkspaceId,
    effects: &mut impl CompilerEffects,
    types: &dyn TypeView,
) -> Result<Vec<Value>, EffectError> {
    let [_, _, location] = arguments else {
        return Err(EffectError::Failed(Error::InvalidIr(
            "set_build_options requires options, workspace and caller location",
        )));
    };
    let location = super::source_location::location(location)?;
    invoke_options_with_location(
        projection,
        &arguments[..2],
        current_workspace,
        effects,
        types,
        Some(location),
    )
}

fn invoke_options_with_location(
    projection: BuildOptionsProjection,
    arguments: &[Value],
    current_workspace: WorkspaceId,
    effects: &mut impl CompilerEffects,
    types: &dyn TypeView,
    location: Option<SourceLocation>,
) -> Result<Vec<Value>, EffectError> {
    let fail = |message| EffectError::Failed(Error::InvalidIr(message));
    if projection == BuildOptionsProjection::default() {
        return Err(fail(
            "set_build_options requires at least one checked option field",
        ));
    }
    let [record, Value::Int(workspace)] = arguments else {
        return Err(fail(
            "set_build_options requires a record and source s64 workspace",
        ));
    };
    if workspace.ty() != IntegerType::S64 {
        return Err(fail("set_build_options requires a source s64 workspace"));
    }
    let workspace = match workspace.value() {
        -1 => current_workspace,
        value if value > 0 => {
            WorkspaceId::from_raw(value as u64).ok_or_else(|| fail("invalid workspace identity"))?
        }
        _ => {
            return Err(fail(
                "source workspace identity must be positive s64 or current-workspace sentinel -1",
            ));
        }
    };
    let mut options = Vec::new();
    if let Some(path) = projection.output_kind {
        let kind = match enumeration(field(record, path, types)?)? {
            0 => BuildOutputKind::None,
            1 => BuildOutputKind::Executable,
            2 => BuildOutputKind::DynamicLibrary,
            3 => BuildOutputKind::StaticLibrary,
            4 => BuildOutputKind::Object,
            _ => return Err(fail("unknown Build_Options output_type value")),
        };
        options.push(BuildOption::OutputKind(kind));
    }
    if let Some(path) = projection.runtime_support {
        let mode = match enumeration(field(record, path, types)?)? {
            0 => RuntimeSupportMode::Auto,
            1 => RuntimeSupportMode::EntryPointAndInitialization,
            2 => RuntimeSupportMode::InitializationOnly,
            3 => RuntimeSupportMode::Omit,
            _ => {
                return Err(fail(
                    "unknown Build_Options runtime_support_definitions value",
                ));
            }
        };
        options.push(BuildOption::RuntimeSupport(mode));
    }
    if let Some(path) = projection.temporary_storage_size {
        let Value::Int(value) = field(record, path, types)? else {
            return Err(fail(
                "Build_Options temporary_storage_size requires an s32 integer",
            ));
        };
        if value.ty() != IntegerType::S32 || value.value() < 0 {
            return Err(fail(
                "Build_Options temporary_storage_size must be a nonnegative source s32",
            ));
        }
        options.push(BuildOption::TemporaryStorageSize(value.value() as i32));
    }
    if let Some(path) = projection.backtrace_on_crash {
        let mode = match enumeration(field(record, path, types)?)? {
            0 => BacktraceOnCrash::Off,
            1 => BacktraceOnCrash::On,
            _ => return Err(fail("unknown Build_Options backtrace_on_crash value")),
        };
        options.push(BuildOption::BacktraceOnCrash(mode));
    }
    if let Some(path) = projection.output_path {
        let text = string(field(record, path, types)?)?;
        if !text.is_empty() {
            options.push(BuildOption::OutputPath(PathBuf::from(text)));
        }
    }
    if let Some(path) = projection.target {
        let text = string(field(record, path, types)?)?;
        if !text.is_empty() {
            options.push(BuildOption::Target(TargetTriple::parse(&text).map_err(
                |_| fail("source target_system_triple has invalid structure"),
            )?));
        }
    }
    if let Some(path) = projection.bitcode {
        let level = match enumeration(field(record, path, types)?)? {
            0 => BitcodeOptimization::Unset,
            1 => BitcodeOptimization::O0,
            2 => BitcodeOptimization::O1,
            3 => BitcodeOptimization::O2,
            4 => BitcodeOptimization::O3,
            5 => BitcodeOptimization::Os,
            6 => BitcodeOptimization::Oz,
            _ => return Err(fail("unknown Llvm_Bitcode_Optimization_Setting value")),
        };
        options.push(BuildOption::BitcodeOptimization(level));
    }
    if let Some(path) = projection.machine {
        let level = match enumeration(field(record, path, types)?)? {
            0 => MachineOptimization::Unset,
            1 => MachineOptimization::None,
            2 => MachineOptimization::Less,
            3 => MachineOptimization::Default,
            4 => MachineOptimization::Aggressive,
            _ => return Err(fail("unknown Llvm_Machine_Code_Optimization_Setting value")),
        };
        options.push(BuildOption::MachineOptimization(level));
    }
    for option in options {
        let request = if let Some(location) = &location {
            CompilerRequest::SetBuildOptionAt {
                workspace,
                option,
                location: location.clone(),
            }
        } else {
            CompilerRequest::SetBuildOption {
                workspace,
                option,
            }
        };
        match effects.request(request) {
            EffectOutcome::Ready(CompilerResponse::Unit) => {}
            EffectOutcome::Ready(_) => {
                return Err(EffectError::Failed(Error::EffectResponse(
                    "set_build_options expected a unit response",
                )));
            }
            EffectOutcome::Pending(key) => {
                return Err(EffectError::Pending(Dependency::Effect(key)));
            }
            EffectOutcome::Rejected(reason) => {
                return Err(EffectError::Failed(Error::EffectRejected(reason)));
            }
        }
    }
    Ok(vec![])
}

fn field<'a>(
    record: &'a Value,
    path: RecordFieldPath,
    types: &dyn TypeView,
) -> Result<&'a Value, EffectError> {
    fn extract<'a>(
        record: &'a Value,
        field: jai_types::FieldId,
        types: &dyn TypeView,
    ) -> Result<&'a Value, EffectError> {
        let Value::Record {
            ty,
            fields,
        } = record
        else {
            return Err(EffectError::Failed(Error::InvalidIr(
                "build option projection requires a struct record",
            )));
        };
        let expected = types
            .validate_field(*ty, field)
            .map_err(|error| EffectError::Failed(Error::Type(error)))?;
        let value = fields.get(field.index()).ok_or({
            EffectError::Failed(Error::InvalidIr(
                "build option record is missing a checked field",
            ))
        })?;
        value
            .validate(types, expected, 256)
            .map_err(EffectError::Failed)?;
        Ok(value)
    }
    let mut value = extract(record, path.outer, types)?;
    if let Some(inner) = path.inner {
        value = extract(value, inner, types)?;
    }
    if let Some(leaf) = path.leaf {
        if path.inner.is_none() {
            return Err(EffectError::Failed(Error::InvalidIr(
                "build option leaf requires a checked middle field",
            )));
        }
        value = extract(value, leaf, types)?;
    }
    Ok(value)
}
fn string(value: &Value) -> Result<String, EffectError> {
    let Value::String(bytes) = value else {
        return Err(EffectError::Failed(Error::InvalidIr(
            "build option field requires a string",
        )));
    };
    String::from_utf8(bytes.clone()).map_err(|_| {
        EffectError::Failed(Error::InvalidIr("build option field requires UTF-8 text"))
    })
}
fn enumeration(value: &Value) -> Result<u64, EffectError> {
    let Value::Enum {
        value, ..
    } = value
    else {
        return Err(EffectError::Failed(Error::InvalidIr(
            "optimization field requires its checked enum",
        )));
    };
    if value.ty() != IntegerType::U8 {
        return Err(EffectError::Failed(Error::InvalidIr(
            "optimization enum must use the source u8 representation",
        )));
    }
    Ok(value.bits())
}

#[cfg(test)]
mod tests;
