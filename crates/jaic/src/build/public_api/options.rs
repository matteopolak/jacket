//! Public option records are decoded by their resolved field schema.
use super::abi::{bytes, string, view, write};
use super::operations::{Call, put_strings, temporary_string};
use super::{Kind, Operation};
use crate::{
    build::MetaOp,
    builtin_keys::{
        WorkspaceFlag as Flag, WorkspaceList as List, WorkspaceNumber as Number,
        WorkspaceText as Text,
    },
    interp::{Interp, Rets, Trap},
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

pub(super) fn call(cx: &Call, interp: &mut Interp) -> Result<Rets, Trap> {
    let ws = cx.workspace(usize::from(cx.binding.operation != Operation::GetOptions))?;
    match cx.binding.operation {
        Operation::GetOptions => {
            let (at, ty) = cx.result(0)?;
            get(cx, interp, ws, at, ty)?;
        }
        Operation::SetOptions => set(cx, interp, ws, cx.arg(0), cx.binding.parameters[0])?,
        Operation::SetOptionsDuringCompile => {
            let ty = cx.schema().named("Build_Options")?;
            let options = interp.register_data_global(
                &vec![0; cx.schema().get(ty).size as usize],
                cx.schema().get(ty).align,
                false,
                1,
            )?;
            get(cx, interp, ws, options, ty)?;
            let dc = cx.binding.parameters[0];
            let input = cx.arg(0);
            if cx.number(interp, dc, input, "interactive_bytecode_debugger")? != 0 {
                return Err(Trap::new("interactive bytecode debugging is unavailable"));
            }
            for name in [
                "write_added_strings",
                "append_executable_filename_extension",
                "interactive_bytecode_debugger",
            ] {
                cx.set_number(
                    interp,
                    ty,
                    options,
                    name,
                    cx.number(interp, dc, input, name)?,
                )?;
            }
            for (key, name) in [
                (Text::OutputExecutableName, "output_executable_name"),
                (Text::OutputPath, "output_path"),
            ] {
                let value = cx.text(interp, dc, input, name)?;
                if !value.is_empty() {
                    super::abi::put_string(
                        interp,
                        cx.field(ty, options, name)?.0,
                        value.as_bytes(),
                    )?;
                }
                cx.invoke(
                    interp,
                    MetaOp::SetText,
                    &[ws as u64, key as u64, cx.field(ty, options, name)?.0],
                )?;
            }
            let added = read_strings(interp, cx.field(dc, input, "append_linker_arguments")?.0)?;
            if !added.is_empty() {
                let destination = cx.field(ty, options, "additional_linker_arguments")?.0;
                let mut all = read_strings(interp, destination)?;
                for value in &added {
                    let slot = temporary_string(interp, value.as_bytes())?;
                    cx.invoke(
                        interp,
                        MetaOp::AppendList,
                        &[ws as u64, List::AdditionalLinkerArgument as u64, slot],
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
                cx.invoke(
                    interp,
                    MetaOp::SetFlag,
                    &[ws as u64, flag as u64, cx.number(interp, dc, input, name)?],
                )?;
            }
            snapshot(cx, interp, ws, options, ty, true)?;
        }
        _ => unreachable!(),
    }
    Ok(Rets::default())
}

fn get(cx: &Call, interp: &mut Interp, ws: i64, at: u64, ty: TypeId) -> Result<(), Trap> {
    let layout = cx.schema().get(ty);
    let initial = if let Some(global) = layout.default {
        let address = interp.global_addr(cx.program, global)?;
        bytes(interp, address, layout.size as usize)?
    } else {
        vec![0; layout.size as usize]
    };
    write(interp, at, &initial)?;
    if !snapshot(cx, interp, ws, at, ty, false)? && ws == cx.shared.borrow().current_id() {
        let arguments = cx.shared.borrow().env.command_line.clone();
        put_strings(
            interp,
            cx.field(ty, at, "compile_time_command_line")?.0,
            &arguments,
        )?;
    }
    Ok(())
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
        cx.invoke(
            interp,
            MetaOp::SetNumber,
            &[ws as u64, option as u64, value],
        )?;
    }
    for &(option, name) in FLAGS {
        cx.invoke(
            interp,
            MetaOp::SetFlag,
            &[ws as u64, option as u64, cx.number(interp, ty, at, name)?],
        )?;
    }
    for &(option, name) in TEXTS {
        cx.invoke(
            interp,
            MetaOp::SetText,
            &[ws as u64, option as u64, cx.field(ty, at, name)?.0],
        )?;
    }
    let major = cx.number(interp, ty, at, "minimum_os_version.major")?;
    let minor = cx.number(interp, ty, at, "minimum_os_version.minor")?;
    let version = if major == 0 && minor == 0 {
        String::new()
    } else {
        format!("{major}.{minor}")
    };
    let slot = temporary_string(interp, version.as_bytes())?;
    cx.invoke(
        interp,
        MetaOp::SetText,
        &[ws as u64, Text::MinimumOsVersion as u64, slot],
    )?;
    for (list, name) in [
        (List::ImportPath, "import_path"),
        (
            List::AdditionalLinkerArgument,
            "additional_linker_arguments",
        ),
    ] {
        cx.invoke(interp, MetaOp::ClearList, &[ws as u64, list as u64])?;
        for value in read_strings(interp, cx.field(ty, at, name)?.0)? {
            let value = if list == List::ImportPath {
                cx.location_path(interp, 2, value)?
            } else {
                value
            };
            let slot = temporary_string(interp, value.as_bytes())?;
            cx.invoke(interp, MetaOp::AppendList, &[ws as u64, list as u64, slot])?;
        }
    }
    snapshot(cx, interp, ws, at, ty, true)?;
    Ok(())
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
        match &cx.schema().get(ty).kind {
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
                        base + i * cx.schema().get(*element).size,
                        strings,
                        arrays,
                    )?;
                }
            }
            Kind::Array {
                element,
                count: None,
                resizable: false,
            } if matches!(cx.schema().get(*element).kind, Kind::String) => arrays.push(base),
            Kind::Array {
                count: None, ..
            } => return Err(Trap::new("unsupported build option view schema")),
            _ => {}
        }
        Ok(())
    }
    fn list(interp: &mut Interp, values: &[u64]) -> Result<u64, Trap> {
        let bytes: Vec<u8> = values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect();
        let data = interp.register_data_global(&bytes, 8, false, 1)?;
        let slot = interp.register_data_global(&[0; 16], 8, false, 1)?;
        super::abi::put_view(interp, slot, values.len() as u64, data)?;
        Ok(slot)
    }
    let (mut strings, mut arrays) = (Vec::new(), Vec::new());
    offsets(cx, ty, 0, &mut strings, &mut arrays)?;
    let strings = list(interp, &strings)?;
    let arrays = list(interp, &arrays)?;
    Ok(cx.invoke(
        interp,
        MetaOp::OptionsSnapshot,
        &[
            ws as u64,
            at,
            cx.schema().get(ty).size,
            strings,
            arrays,
            u64::from(save),
        ],
    )?[0]
        != 0)
}
