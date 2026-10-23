//! Public option records are decoded by their resolved field schema.
use super::abi::{Need, bytes, string, view, write};
use super::operations::{Call, put_strings};
use super::{Kind, Operation};
use crate::{
    build::{Snapshot, engine},
    builtin_keys::{
        WorkspaceFlag as Flag, WorkspaceList as List, WorkspaceNumber as Number,
        WorkspaceText as Text,
    },
    interp::{Interp, Rets, Trap},
    ir,
    types::TypeId,
};

const NUMBERS: &[(Number, &str)] = &[
    (Number::OutputType, "output_type"),
    (Number::OsTarget, "os_target"),
    (Number::CpuTarget, "cpu_target"),
    (Number::TemporaryStorageSize, "temporary_storage_size"),
    (Number::ArrayBoundsCheck, "array_bounds_check"),
    (Number::CastBoundsCheck, "cast_bounds_check"),
    (Number::NullPointerCheck, "null_pointer_check"),
    (Number::ArithmeticOverflowCheck, "arithmetic_overflow_check"),
    (Number::DeadCodeElimination, "dead_code_elimination"),
    (
        Number::RuntimeSupportDefinitions,
        "runtime_support_definitions",
    ),
    (Number::BacktraceOnCrash, "backtrace_on_crash"),
    (Number::EmitDebugInfo, "emit_debug_info"),
    (
        Number::BitcodeOptimization,
        "llvm_options.bitcode_optimization_setting",
    ),
    (
        Number::MachineCodeOptimization,
        "llvm_options.machine_code_optimization_setting",
    ),
];

const FLAGS: &[(Flag, &str)] = &[
    (Flag::StackTrace, "stack_trace"),
    (Flag::DisableRedzone, "disable_redzone"),
    (Flag::EnableFramePointers, "enable_frame_pointers"),
    (
        Flag::AppendExecutableFilenameExtension,
        "append_executable_filename_extension",
    ),
    (Flag::UseCustomLinkCommand, "use_custom_link_command"),
    (Flag::LlvmEnableTailCalls, "llvm_options.enable_tail_calls"),
    (
        Flag::LlvmEnableLoopUnrolling,
        "llvm_options.enable_loop_unrolling",
    ),
    (
        Flag::LlvmEnableSlpVectorization,
        "llvm_options.enable_slp_vectorization",
    ),
    (
        Flag::LlvmEnableLoopVectorization,
        "llvm_options.enable_loop_vectorization",
    ),
    (
        Flag::LlvmPreserveDebugInfo,
        "llvm_options.preserve_debug_info",
    ),
    (Flag::LlvmMergeFunctions, "llvm_options.merge_functions"),
    (Flag::LlvmDisableInlining, "llvm_options.disable_inlining"),
    (Flag::LlvmDisableMem2reg, "llvm_options.disable_mem2reg"),
    (
        Flag::LlvmEnableSplitModules,
        "llvm_options.enable_split_modules",
    ),
    (Flag::LlvmFunctionSections, "llvm_options.function_sections"),
    (Flag::OutputBitcode, "llvm_options.output_bitcode"),
    (Flag::OutputLlvmIr, "llvm_options.output_llvm_ir"),
    (
        Flag::OutputBitcodeBeforeOptimizations,
        "llvm_options.output_bitcode_before_optimizations",
    ),
    (
        Flag::OutputLlvmIrBeforeOptimizations,
        "llvm_options.output_llvm_ir_before_optimizations",
    ),
];

const TEXTS: &[(Text, &str)] = &[
    (Text::OutputExecutableName, "output_executable_name"),
    (Text::OutputPath, "output_path"),
    (Text::IntermediatePath, "intermediate_path"),
    (Text::EntryPointName, "entry_point_name"),
    (
        Text::LlvmTargetSystemTriple,
        "llvm_options.target_system_triple",
    ),
    (Text::LlvmTargetSystemCpu, "llvm_options.target_system_cpu"),
    (
        Text::LlvmTargetSystemFeatures,
        "llvm_options.target_system_features",
    ),
];

/// Scalars `set_build_options_dc` copies from its argument into the workspace's options.
const COPIED: &[&str] = &[
    "write_added_strings",
    "append_executable_filename_extension",
    "interactive_bytecode_debugger",
];

/// Texts `set_build_options_dc` sets when its argument has them.
const RENAMED: &[(Text, &str)] = &[
    (Text::OutputExecutableName, "output_executable_name"),
    (Text::OutputPath, "output_path"),
];

/// The fields an options record must have, checked when an operation is bound.
pub(super) fn fields(operation: Operation) -> Vec<(&'static str, Need)> {
    let renamed = RENAMED.iter().map(|&(_, name)| (name, Need::Text));
    let copied = COPIED.iter().map(|&name| (name, Need::Scalar));
    match operation {
        Operation::GetOptions => vec![("compile_time_command_line", Need::Texts)],
        Operation::SetOptions => NUMBERS
            .iter()
            .map(|&(_, name)| (name, Need::Scalar))
            .chain(FLAGS.iter().map(|&(_, name)| (name, Need::Scalar)))
            .chain(TEXTS.iter().map(|&(_, name)| (name, Need::Text)))
            .chain([
                ("interactive_bytecode_debugger", Need::Scalar),
                ("llvm_options.command_line", Need::Texts),
                ("minimum_os_version.major", Need::Scalar),
                ("minimum_os_version.minor", Need::Scalar),
                ("import_path", Need::Texts),
                ("additional_linker_arguments", Need::Texts),
            ])
            .collect(),
        // The workspace's options this operation updates.
        Operation::SetOptionsDuringCompile => copied
            .chain(renamed)
            .chain([("additional_linker_arguments", Need::Texts)])
            .collect(),
        _ => Vec::new(),
    }
}

/// The fields `set_build_options_dc` reads from its argument.
pub(super) fn during_compile() -> Vec<(&'static str, Need)> {
    COPIED
        .iter()
        .map(|&name| (name, Need::Scalar))
        .chain(RENAMED.iter().map(|&(_, name)| (name, Need::Text)))
        .chain([
            ("append_linker_arguments", Need::Texts),
            ("do_output", Need::Scalar),
        ])
        .collect()
}

pub(super) fn call(cx: &Call, interp: &mut Interp) -> Result<Rets, Trap> {
    let ws = cx.workspace(usize::from(cx.binding.operation != Operation::GetOptions))?;
    match cx.binding.operation {
        Operation::GetOptions => {
            get(cx, interp, ws, cx.result(0)?.0)?;
        }
        Operation::SetOptions => set(cx, interp, ws, cx.arg(0), cx.binding.parameters[0])?,
        Operation::SetOptionsDuringCompile => {
            let layout = cx.schema().get(options_record(cx)?.0)?;
            let options = interp.register_data_global(
                &vec![0; layout.size as usize],
                layout.align,
                false,
                1,
            )?;
            let ty = get(cx, interp, ws, options)?;
            let dc = cx.binding.parameters[0];
            let input = cx.arg(0);
            if cx.number(interp, dc, input, "interactive_bytecode_debugger")? != 0 {
                return Err(Trap::new("interactive bytecode debugging is unavailable"));
            }
            for &name in COPIED {
                cx.set_number(
                    interp,
                    ty,
                    options,
                    name,
                    cx.number(interp, dc, input, name)?,
                )?;
            }
            for &(key, name) in RENAMED {
                let value = cx.text(interp, dc, input, name)?;
                if !value.is_empty() {
                    super::abi::put_string(
                        interp,
                        cx.field(ty, options, name)?.0,
                        value.as_bytes(),
                    )?;
                }
                let value = cx.text(interp, ty, options, name)?;
                cx.registry().set_text(ws, key, &value).map_err(Trap::new)?;
            }
            let added = read_strings(interp, cx.field(dc, input, "append_linker_arguments")?.0)?;
            if !added.is_empty() {
                let destination = cx.field(ty, options, "additional_linker_arguments")?.0;
                let mut all = read_strings(interp, destination)?;
                for value in &added {
                    append(
                        cx,
                        interp,
                        ws,
                        List::AdditionalLinkerArgument,
                        value.clone(),
                    )?;
                }
                all.extend(added);
                put_strings(interp, destination, &all)?;
            }
            for (flag, name) in [
                (Flag::DoOutput, "do_output"),
                (
                    Flag::AppendExecutableFilenameExtension,
                    "append_executable_filename_extension",
                ),
            ] {
                let value = cx.number(interp, dc, input, name)? & 1 != 0;
                cx.registry().set_flag(ws, flag, value).map_err(Trap::new)?;
            }
            snapshot(cx, interp, ws, options, ty, true)?;
        }
        _ => unreachable!(),
    }
    Ok(Rets::default())
}

fn options_record(cx: &Call) -> Result<(TypeId, ir::GlobalId), Trap> {
    cx.binding
        .options
        .ok_or_else(|| Trap::new("compiler operation has no build options record"))
}

/// Fill the options record at `at` with the workspace's options, returning its type.
fn get(cx: &Call, interp: &mut Interp, ws: i64, at: u64) -> Result<TypeId, Trap> {
    let (ty, defaults) = options_record(cx)?;
    let address = interp.global_addr(cx.program, defaults)?;
    let initial = bytes(interp, address, cx.schema().get(ty)?.size as usize)?;
    write(interp, at, &initial)?;
    if !snapshot(cx, interp, ws, at, ty, false)? && ws == cx.shared.borrow().current_id() {
        let arguments = cx.shared.borrow().env.command_line.clone();
        put_strings(
            interp,
            cx.field(ty, at, "compile_time_command_line")?.0,
            &arguments,
        )?;
    }
    Ok(ty)
}

fn set(cx: &Call, interp: &mut Interp, ws: i64, at: u64, ty: TypeId) -> Result<(), Trap> {
    if cx.number(interp, ty, at, "interactive_bytecode_debugger")? != 0 {
        return Err(Trap::new("interactive bytecode debugging is unavailable"));
    }
    let command = cx.field(ty, at, "llvm_options.command_line")?.0;
    if view(interp, command)?.0 != 0 {
        return Err(Trap::new(
            "raw LLVM command-line options are unavailable; use explicit build settings",
        ));
    }
    if cx.number(interp, ty, at, "llvm_options.function_sections")? != 0
        || cx.number(interp, ty, at, "llvm_options.disable_mem2reg")? != 0
    {
        return Err(Trap::new(
            "these LLVM pass switches are not implemented by the current backend",
        ));
    }
    for &(option, name) in NUMBERS {
        let mut value = cx.number(interp, ty, at, name)?;
        if matches!(option, Number::OsTarget | Number::CpuTarget) {
            let field = cx.field(ty, at, name)?.1;
            let member = cx
                .schema()
                .enums
                .get(&field)
                .and_then(|members| members.iter().find(|(_, v)| *v as u64 == value))
                .map(|(n, _)| n.as_str());
            value = match (option, member) {
                (Number::OsTarget, Some("WINDOWS")) => crate::builtin_keys::OsTag::Windows as u64,
                (Number::OsTarget, Some("LINUX")) => crate::builtin_keys::OsTag::Linux as u64,
                (Number::OsTarget, Some("MACOS")) => crate::builtin_keys::OsTag::MacOS as u64,
                (Number::OsTarget, Some("WASM")) => crate::builtin_keys::OsTag::Wasm as u64,
                (Number::CpuTarget, Some("X64")) => crate::builtin_keys::CpuTag::X64 as u64,
                (Number::CpuTarget, Some("ARM64")) => crate::builtin_keys::CpuTag::Arm64 as u64,
                (Number::CpuTarget, Some("CUSTOM")) => crate::builtin_keys::CpuTag::Custom as u64,
                (_, member) => {
                    return Err(Trap::new(format!(
                        "jaic cannot build for `{name} = .{}`",
                        member.map_or_else(|| value.to_string(), str::to_owned)
                    )));
                }
            };
        }
        cx.registry()
            .set_number(ws, option, value)
            .map_err(Trap::new)?;
    }
    for &(option, name) in FLAGS {
        let value = cx.number(interp, ty, at, name)? & 1 != 0;
        cx.registry()
            .set_flag(ws, option, value)
            .map_err(Trap::new)?;
    }
    for &(option, name) in TEXTS {
        let value = cx.text(interp, ty, at, name)?;
        cx.registry()
            .set_text(ws, option, &value)
            .map_err(Trap::new)?;
    }
    let major = cx.number(interp, ty, at, "minimum_os_version.major")?;
    let minor = cx.number(interp, ty, at, "minimum_os_version.minor")?;
    let version = if major == 0 && minor == 0 {
        String::new()
    } else {
        format!("{major}.{minor}")
    };
    cx.registry()
        .set_text(ws, Text::MinimumOsVersion, &version)
        .map_err(Trap::new)?;
    for (list, name) in [
        (List::ImportPath, "import_path"),
        (
            List::AdditionalLinkerArgument,
            "additional_linker_arguments",
        ),
    ] {
        cx.registry().clear_list(ws, list).map_err(Trap::new)?;
        for value in read_strings(interp, cx.field(ty, at, name)?.0)? {
            let value = if list == List::ImportPath {
                cx.location_path(interp, 2, value)?
            } else {
                value
            };
            append(cx, interp, ws, list, value)?;
        }
    }
    snapshot(cx, interp, ws, at, ty, true)?;
    Ok(())
}

fn append(cx: &Call, interp: &Interp, ws: i64, list: List, value: String) -> Result<(), Trap> {
    let value = if list == List::ImportPath {
        engine::import_folder(interp, value)
    } else {
        value
    };
    cx.registry()
        .append_list(ws, list, &value)
        .map_err(Trap::new)
}

pub(super) fn read_strings(interp: &Interp, at: u64) -> Result<Vec<String>, Trap> {
    let (count, data) = view(interp, at)?;
    if count > crate::sema::value::MAX_IMAGE / 16 {
        return Err(Trap::new("invalid string array count"));
    }
    (0..count).map(|i| string(interp, data + i * 16)).collect()
}

fn snapshot(
    cx: &Call,
    interp: &mut Interp,
    ws: i64,
    at: u64,
    ty: TypeId,
    save: bool,
) -> Result<bool, Trap> {
    fn offsets(
        cx: &Call,
        ty: TypeId,
        base: u64,
        strings: &mut Vec<u64>,
        arrays: &mut Vec<u64>,
    ) -> Result<(), Trap> {
        match &cx.schema().get(ty)?.kind {
            Kind::String => strings.push(base),
            Kind::Record(fields) => {
                for field in fields {
                    offsets(cx, field.ty, base + field.offset, strings, arrays)?;
                }
            }
            Kind::Array {
                element,
                count: Some(count),
                ..
            } => {
                for i in 0..*count {
                    offsets(
                        cx,
                        *element,
                        base + i * cx.schema().get(*element)?.size,
                        strings,
                        arrays,
                    )?;
                }
            }
            Kind::Array {
                element,
                count: None,
                resizable: false,
            } if matches!(cx.schema().get(*element)?.kind, Kind::String) => arrays.push(base),
            Kind::Array {
                count: None, ..
            } => return Err(Trap::new("unsupported build option view schema")),
            _ => {}
        }
        Ok(())
    }
    let (mut strings, mut arrays) = (Vec::new(), Vec::new());
    offsets(cx, ty, 0, &mut strings, &mut arrays)?;
    let size = cx.schema().get(ty)?.size as usize;
    if save {
        let snapshot = Snapshot::capture(interp, at, size, &strings, &arrays)?;
        cx.registry().save_options(ws, snapshot)?;
        return Ok(true);
    }
    let Some(snapshot) = cx.registry().options_snapshot(ws)? else {
        return Ok(false);
    };
    snapshot.restore(interp, at, size)?;
    Ok(true)
}
