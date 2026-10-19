//! Emitting objects and IR for a program, split into codegen units.
use super::linker::check_sanitizer_target;
use super::lower::Shard;
use super::machine::{set_coverage_options, target_machine, verify_ir};
#[cfg(target_os = "macos")]
use super::malloc_zone;
use super::options::{Codegen, OptLevel, Options};
use super::partition::func_weight;
use super::{debuginfo, green, lower, partition, simd, split};
use inkwell::attributes::{Attribute, AttributeLoc};
use inkwell::context::Context;
use inkwell::passes::PassBuilderOptions;
use inkwell::targets::FileType;
use jaic::ir::{self, Library, Program};
use jaic::target::Target as BuildTarget;
use std::path::{Path, PathBuf};

/// Lower `program` (or one shard of it) to one LLVM module and write it as an object file.
/// With `split_after_opt`, an optimized module large enough is written as several objects
/// in parallel (`split.rs`); the objects written are returned.
fn emit_module(
    program: &Program,
    options: &Options,
    path: &Path,
    shard: Option<Shard>,
    split_after_opt: bool,
    leak: bool,
    no_inline: &[bool],
) -> Result<Vec<PathBuf>, String> {
    // Without +atomics LLVM may lower atomic IR to ordinary wasm accesses.
    // Select the required target feature from the program, not a module name or
    // fixture. An explicit contradictory feature is a configuration error.
    let atomic_options;
    let options = if options.target.is_some_and(jaic::target::Target::is_wasm)
        && program.funcs.iter().flatten().flat_map(|f| &f.blocks).flat_map(|b| &b.insts).any(|i| {
            matches!(i, ir::Inst::Intrinsic(call) if matches!(call.op,
                ir::Intrinsic::CompareAndSwap | ir::Intrinsic::AtomicLoad(_) |
                ir::Intrinsic::AtomicStore(_) | ir::Intrinsic::AtomicRmw(..) | ir::Intrinsic::AtomicCas(..)))
        }) {
        let mut selected = options.clone();
        let features = selected.features.get_or_insert_default();
        if features.split(',').rev().find(|f| matches!(f.trim(), "+atomics" | "-atomics"))
            .is_some_and(|f| f.trim() == "-atomics") {
            return Err("atomic operations require the WebAssembly atomics target feature".into());
        }
        if !features.is_empty() { features.push(','); }
        features.push_str("+atomics");
        atomic_options = selected;
        &atomic_options
    } else { options };
    if options.sanitize.any() {
        check_sanitizer_target(options.target)?;
    }
    let (machine, triple, build_target, arch) = target_machine(options)?;
    let context = Context::create();
    if !wants_ir_text(options) {
        discard_value_names(&context);
    }
    let module = context.create_module("jai");
    module.set_triple(&triple);
    module.set_data_layout(&machine.get_target_data().get_data_layout());
    let debug = options.debug_info.then(|| {
        (
            debuginfo::DebugFormat::for_target(build_target),
            options.opt_level != OptLevel::O0,
        )
    });
    lower::lower_program(
        &context,
        &module,
        program,
        &lower::LowerOptions {
            arch,
            target: build_target,
            shard,
            debug,
            unoptimized: options.opt_level == OptLevel::O0,
            no_inline,
            simd: simd::Target::resolve(
                arch,
                &machine.get_cpu().to_string_lossy(),
                &machine.get_feature_string().to_string_lossy(),
            ),
        },
    )?;
    apply_abi_unwind_attributes(&context, &module, arch)?;
    let unit = shard.map_or(0, |s| s.index);
    if let Some(ir_path) = &options.emit_ir {
        module
            .print_to_file(unit_path(ir_path, unit))
            .map_err(|e| e.to_string())?;
    }
    if verify_ir(options) {
        module
            .verify()
            .map_err(|e| format!("invalid LLVM IR: {e}"))?;
    }
    apply_codegen_attributes(&context, &module, &options.codegen, build_target);
    if let Some(path) = &options.codegen.bitcode_before
        && !module.write_bitcode_to_path(unit_path(path, unit))
    {
        return Err(format!("could not write `{}`", path.display()));
    }
    if options.sanitize.address {
        // ASan instruments only functions with this attribute (Clang adds it to each
        // definition it emits); declarations are left alone.
        let kind = Attribute::get_named_enum_kind_id("sanitize_address");
        for function in module.get_functions() {
            if function.count_basic_blocks() > 0 {
                function.add_attribute(
                    AttributeLoc::Function,
                    context.create_enum_attribute(kind, 0),
                );
            }
        }
    }
    if options.sanitize.coverage || options.sanitize.fuzzer {
        set_coverage_options(options.sanitize.fuzzer);
    }
    let mut passes: Vec<&str> = options.opt_level.pipeline().into_iter().collect();
    passes.extend(options.sanitize.passes(options.opt_level));
    if options.opt_level == OptLevel::O0
        && let Some(p) = &options.codegen.o0_passes
    {
        passes.push(p);
    }
    if !passes.is_empty() {
        let pass_options = PassBuilderOptions::create();
        let c = &options.codegen;
        if let Some(on) = c.loop_unrolling {
            pass_options.set_loop_unrolling(on);
        }
        if let Some(on) = c.loop_vectorization {
            pass_options.set_loop_vectorization(on);
            pass_options.set_loop_interleaving(on);
        }
        if let Some(on) = c.slp_vectorization {
            pass_options.set_loop_slp_vectorization(on);
        }
        if let Some(on) = c.merge_functions {
            pass_options.set_merge_functions(on);
        }
        module
            .run_passes(&passes.join(","), &machine, pass_options)
            .map_err(|e| e.to_string())?;
    }
    // Instrumentation/optimization can introduce definitions. They have the same ABI
    // obligations as source functions, including when a pass ignores synthesis defaults.
    apply_abi_unwind_attributes(&context, &module, arch)?;
    if let Some(path) = &options.codegen.ir_after {
        module
            .print_to_file(unit_path(path, unit))
            .map_err(|e| e.to_string())?;
    }
    if let Some(path) = &options.codegen.bitcode_after
        && !module.write_bitcode_to_path(unit_path(path, unit))
    {
        return Err(format!("could not write `{}`", path.display()));
    }
    if split_after_opt && !options.sanitize.any() {
        let units = split::units_for(&module, options.codegen.split_units);
        if units > 1 {
            let make = || target_machine(options).map(|(machine, ..)| machine);
            return split::emit(&module, units, path, build_target.is_windows(), &make);
        }
    }
    machine
        .write_to_file(&module, FileType::Object, path)
        .map_err(|e| e.to_string())?;
    if leak {
        // The process links and exits next: tearing down a large module takes about as long as
        // writing its object (`docs/compiler/compile-speed.md`).
        std::mem::forget(module);
        std::mem::forget(context);
        std::mem::forget(machine);
    }
    Ok(vec![path.to_path_buf()])
}

/// Where codegen unit `unit` writes its copy of an IR or bitcode output: the path itself for the
/// first unit, `x.1.ll`, `x.2.ll`... for the others. Units are normally one when IR is asked for;
/// `-codegen_units` can force several, and each then keeps its own file instead of the units
/// overwriting one another.
pub(super) fn unit_path(path: &Path, unit: u32) -> PathBuf {
    if unit == 0 {
        return path.to_path_buf();
    }
    let mut name = path.file_stem().unwrap_or_default().to_os_string();
    name.push(format!(".{unit}"));
    if let Some(extension) = path.extension() {
        name.push(".");
        name.push(extension);
    }
    path.with_file_name(name)
}

/// Whether the caller reads the LLVM IR text (`-emit_ir`, `output_llvm_ir`, bitcode), which is
/// easier to follow with the value and block names lowering gives.
fn wants_ir_text(options: &Options) -> bool {
    let c = &options.codegen;
    options.emit_ir.is_some()
        || c.ir_after.is_some()
        || c.bitcode_after.is_some()
        || c.bitcode_before.is_some()
}

/// Do not keep names on instructions, arguments and blocks: nothing reads them, and at `-O0`
/// uniquing and storing them was a visible share of lowering.
#[allow(unsafe_code)]
fn discard_value_names(context: &Context) {
    // SAFETY: `context` is live; the setting only affects values created afterwards.
    unsafe {
        inkwell::llvm_sys::core::LLVMContextSetDiscardValueNames(
            inkwell::context::AsContextRef::as_ctx_ref(&context),
            1,
        );
    };
}

/// Windows stack walks and asynchronous exceptions need tables even for inferred `nounwind`
/// functions. Frame pointers and language exception propagation are independent of this ABI.
pub(super) fn apply_abi_unwind_attributes<'ctx>(
    context: &'ctx Context,
    module: &inkwell::module::Module<'ctx>,
    arch: jaic::abi::Arch,
) -> Result<(), String> {
    if !arch.is_windows() {
        return Ok(());
    }
    // LLVM UWTableKind::Async = 2 (llvm/Support/CodeGen.h). A module default also
    // covers functions synthesized by instrumentation and outlining passes.
    if module.get_flag("uwtable").is_none() {
        // LLVM’s C API exposes no Max flag behavior; construct its documented
        // metadata record directly (LangRef: Module Flags Metadata, behavior 7).
        let flag = context.metadata_node(&[
            context.i32_type().const_int(7, false).into(),
            context.metadata_string("uwtable").into(),
            context.i32_type().const_int(2, false).into(),
        ]);
        module
            .add_global_metadata("llvm.module.flags", &flag)
            .map_err(|error| error.to_string())?;
    }
    let kind = Attribute::get_named_enum_kind_id("uwtable");
    for function in module.get_functions() {
        if function.count_basic_blocks() > 0 {
            function.add_attribute(
                AttributeLoc::Function,
                context.create_enum_attribute(kind, 2),
            );
        }
    }
    Ok(())
}

/// The function attributes `Build_Options` asks for, on every function with a body.
pub(super) fn apply_codegen_attributes(
    context: &Context,
    module: &inkwell::module::Module,
    codegen: &Codegen,
    target: BuildTarget,
) {
    let apple = target.is_apple();
    let frame_pointer = match codegen.frame_pointers {
        Some(true) => Some("all"),
        // Apple's ABIs require frame records (`docs/metaprogramming/build-options.md`).
        Some(false) if !apple => Some("none"),
        _ => None,
    };
    let enum_attr =
        |name: &str| context.create_enum_attribute(Attribute::get_named_enum_kind_id(name), 0);
    let always_inline = Attribute::get_named_enum_kind_id("alwaysinline");
    for function in module.get_functions() {
        if function.count_basic_blocks() == 0 {
            continue;
        }
        if let Some(value) = frame_pointer {
            function.remove_string_attribute(AttributeLoc::Function, "frame-pointer");
            function.add_attribute(
                AttributeLoc::Function,
                context.create_string_attribute("frame-pointer", value),
            );
        }
        if codegen.no_red_zone {
            function.add_attribute(AttributeLoc::Function, enum_attr("noredzone"));
        }
        if codegen.disable_inlining == Some(true)
            && function
                .get_enum_attribute(AttributeLoc::Function, always_inline)
                .is_none()
        {
            function.add_attribute(AttributeLoc::Function, enum_attr("noinline"));
        }
        if codegen.tail_calls == Some(false) {
            function.add_attribute(
                AttributeLoc::Function,
                context.create_string_attribute("disable-tail-calls", "true"),
            );
        }
    }
}

/// Translate `program` to a single native object file at `path`.
pub fn emit_object(program: &Program, options: &Options, path: &Path) -> Result<(), String> {
    let instrumented = green::instrument(program);
    let program = instrumented.as_ref().unwrap_or(program);
    emit_module(
        program,
        options,
        path,
        None,
        false,
        false,
        &inline_cuts(program, options),
    )
    .map(drop)
}

/// IR instructions per codegen unit below which splitting does not pay for itself.
pub(super) const INSTS_PER_UNIT: usize = 5_000;

/// The same for optimized builds, whose modules also run the optimizer.
const OPT_INSTS_PER_UNIT: usize = 20_000;

/// Most modules an optimized build is divided into before the optimizer.
const MAX_OPT_UNITS: usize = 8;

/// How many modules to split codegen into: `-codegen_units` when set, otherwise one
/// per 5,000 IR instructions (at most one per core) for unoptimized builds and one per 20,000
/// for optimized ones, which are divided by who calls whom and keep copies of small callees for
/// inlining (`partition.rs`). Sanitized builds stay whole, even under `-codegen_units`: on
/// Intel macOS the linker records only the first sanitized unit in the debug map, and the reports
/// lose the other units' lines.
fn codegen_units(program: &Program, options: &Options) -> usize {
    if options.sanitize.any() || options.codegen.single_unit {
        return 1;
    }
    if let Some(n) = options.codegen.units {
        return n.max(1);
    }
    if options.emit_ir.is_some()
        || options.codegen.ir_after.is_some()
        || options.codegen.bitcode_after.is_some()
        || options.codegen.bitcode_before.is_some()
    {
        return 1;
    }
    let insts: usize = program.funcs.iter().flatten().map(func_weight).sum();
    let cores = jaic::pool::threads();
    if options.opt_level == OptLevel::O0 {
        if options.codegen.split_modules == Some(false) {
            return 1;
        }
        return (insts / INSTS_PER_UNIT).clamp(1, cores);
    }
    // Optimized builds split before the optimizer only when it is worth the code it loses
    // (`partition.rs`); wasm modules stay whole.
    let wasm = options.target.is_some_and(jaic::target::Target::is_wasm);
    if wasm {
        return 1;
    }
    (insts / OPT_INSTS_PER_UNIT).clamp(1, cores.min(MAX_OPT_UNITS))
}


/// The functions kept out of their callers in an optimized build (`partition::inline_cuts`).
pub(super) fn inline_cuts(program: &Program, options: &Options) -> Vec<bool> {
    if options.opt_level == OptLevel::O0 {
        return Vec::new();
    }
    partition::inline_cuts(program, partition::INLINE_CAP)
}

/// Keep `fork` working when the process's C allocator is mimalloc's macOS zone
/// (the `override` feature of mimalloc); call once at startup. Does nothing elsewhere.
pub fn fix_malloc_zone_for_fork() {
    #[cfg(target_os = "macos")]
    malloc_zone::fix_fork();
}

/// Translate `program` to native object files, splitting codegen across threads when it
/// is large. Returns the objects written: `path` itself, then `path.1.o`, `path.2.o`...
pub fn emit_objects(
    program: &Program,
    options: &Options,
    path: &Path,
) -> Result<Vec<PathBuf>, String> {
    // WASI programs that start threads run them as green threads (`green.rs`).
    let instrumented = green::instrument(program);
    let program = instrumented.as_ref().unwrap_or(program);
    let units = codegen_units(program, options);
    let cuts = inline_cuts(program, options);
    if units == 1 {
        // A program too small to divide before the optimizer is optimized whole and only
        // machine code generation is split, unless `-codegen_units` asked for exactly one
        // unit. The objects then differ from one module's only in how many there are.
        let split = options.opt_level != OptLevel::O0
            && options.emit_ir.is_none()
            && !options.codegen.single_unit
            && options.codegen.units.is_none();
        return emit_module(program, options, path, None, split, true, &cuts);
    }
    let plan = if options.opt_level == OptLevel::O0 {
        partition::flat(program, units)
    } else {
        partition::plan(program, units)
    };
    let paths: Vec<PathBuf> = (0..units)
        .map(|u| {
            if u == 0 {
                return path.to_path_buf();
            }
            let mut name = path.to_path_buf().into_os_string();
            name.push(format!(".{u}.o"));
            PathBuf::from(name)
        })
        .collect();
    jaic::pool::units(
        units,
        |u| {
            let shard = Shard {
                plan: &plan,
                index: u as u32,
            };
            emit_module(program, options, &paths[u], Some(shard), false, true, &cuts).map(drop)
        },
        || {},
    )
    .into_iter()
    .collect::<Result<Vec<()>, String>>()?;
    Ok(paths)
}

/// The libraries from `program.libraries` that some foreign symbol uses, or that are
/// `link_always`.
pub fn used_libraries(program: &Program) -> Vec<Library> {
    let mut used: Vec<bool> = program.libraries.iter().map(|l| l.link_always).collect();
    for foreign in &program.foreigns {
        if let Some(i) = foreign.library.filter(|&i| i < used.len()) {
            used[i] = true;
        }
    }
    program
        .libraries
        .iter()
        .zip(used)
        .filter(|&(_, u)| u)
        .map(|(l, _)| l.clone())
        .collect()
}
