//! LLVM backend for the jaic compiler core.
//!
//! [`emit_object`] lowers an `ir::Program` to LLVM IR and writes a native
//! object file ([`emit_objects`] splits large unoptimized builds across threads); [`link`] turns object files into an executable with the
//! system C compiler driver.
mod debuginfo;
mod lower;

use lower::Shard;

use inkwell::OptimizationLevel;
use inkwell::context::Context;
use inkwell::passes::PassBuilderOptions;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine, TargetTriple,
};
use jaic::ir::{Library, Program};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Optimization pipeline selection.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum OptLevel {
    #[default]
    O0,
    O1,
    O2,
    O3,
}

impl OptLevel {
    fn llvm(self) -> OptimizationLevel {
        match self {
            OptLevel::O0 => OptimizationLevel::None,
            OptLevel::O1 => OptimizationLevel::Less,
            OptLevel::O2 => OptimizationLevel::Default,
            OptLevel::O3 => OptimizationLevel::Aggressive,
        }
    }
    /// New-pass-manager pipeline string, `None` when no optimization runs.
    fn pipeline(self) -> Option<&'static str> {
        match self {
            OptLevel::O0 => None,
            OptLevel::O1 => Some("default<O1>"),
            OptLevel::O2 => Some("default<O2>"),
            OptLevel::O3 => Some("default<O3>"),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Options {
    pub opt_level: OptLevel,
    /// Target triple; `None` selects the host.
    pub target: Option<String>,
    /// Also write the textual LLVM IR (before optimization) to this path.
    pub emit_ir: Option<PathBuf>,
    /// Emit native debug information (DWARF; see `docs/native/debug-info.md`).
    pub debug_info: bool,
}

/// The host triple. On macOS LLVM's default names the Darwin kernel version, which it maps to
/// a newer macOS than the SDK the linker targets (a warning per link); objects are built for a
/// deployment target instead (`MACOSX_DEPLOYMENT_TARGET`, default 11.0, the first arm64 macOS).
fn host_triple() -> TargetTriple {
    let default = TargetMachine::get_default_triple();
    let text = default.as_str().to_string_lossy().into_owned();
    match text.split_once("-apple-darwin") {
        Some((arch, _)) => {
            let version =
                std::env::var("MACOSX_DEPLOYMENT_TARGET").unwrap_or_else(|_| "11.0".into());
            TargetTriple::create(&format!("{arch}-apple-macosx{version}"))
        }
        None => default,
    }
}

/// The target machine for `options`, and the architecture it targets.
fn target_machine(
    options: &Options,
) -> Result<(TargetMachine, TargetTriple, jaic::abi::Arch), String> {
    Target::initialize_all(&InitializationConfig::default());
    let host = options.target.is_none();
    let triple = match &options.target {
        Some(t) => TargetTriple::create(t),
        None => host_triple(),
    };
    let triple_str = triple.as_str().to_string_lossy().into_owned();
    let arch = jaic::abi::Arch::from_triple(&triple_str)
        .ok_or_else(|| format!("unsupported target architecture in '{triple_str}'"))?;
    let target = Target::from_triple(&triple).map_err(|e| e.to_string())?;
    let (cpu, features) = if host {
        (
            TargetMachine::get_host_cpu_name().to_string(),
            TargetMachine::get_host_cpu_features().to_string(),
        )
    } else {
        ("generic".to_string(), String::new())
    };
    let machine = target
        .create_target_machine(
            &triple,
            &cpu,
            &features,
            options.opt_level.llvm(),
            RelocMode::PIC,
            CodeModel::Default,
        )
        .ok_or("could not create a target machine")?;
    Ok((machine, triple, arch))
}

/// Lower `program` (or one shard of it) to one LLVM module and write it as an object file.
fn emit_module(
    program: &Program,
    options: &Options,
    path: &Path,
    shard: Option<Shard>,
) -> Result<(), String> {
    let (machine, triple, arch) = target_machine(options)?;
    let context = Context::create();
    let module = context.create_module("jai");
    module.set_triple(&triple);
    module.set_data_layout(&machine.get_target_data().get_data_layout());
    let debug = options.debug_info.then(|| {
        let triple = triple.as_str().to_string_lossy();
        (
            debuginfo::DebugFormat::for_triple(&triple),
            options.opt_level != OptLevel::O0,
        )
    });
    lower::lower_program(&context, &module, program, arch, shard, debug)?;
    if let Some(ir_path) = &options.emit_ir {
        module.print_to_file(ir_path).map_err(|e| e.to_string())?;
    }
    module
        .verify()
        .map_err(|e| format!("invalid LLVM IR: {e}"))?;
    if let Some(pipeline) = options.opt_level.pipeline() {
        module
            .run_passes(pipeline, &machine, PassBuilderOptions::create())
            .map_err(|e| e.to_string())?;
    }
    machine
        .write_to_file(&module, FileType::Object, path)
        .map_err(|e| e.to_string())
}

/// Translate `program` to a single native object file at `path`.
pub fn emit_object(program: &Program, options: &Options, path: &Path) -> Result<(), String> {
    emit_module(program, options, path, None)
}

/// IR instructions per codegen unit below which splitting does not pay for itself.
const INSTS_PER_UNIT: usize = 20_000;

/// How many modules to split codegen into: `JAIC_CODEGEN_UNITS` when set, otherwise one
/// per core for large unoptimized builds. Optimized builds stay whole so LLVM can inline
/// across the program.
fn codegen_units(program: &Program, options: &Options) -> usize {
    if let Some(n) = std::env::var("JAIC_CODEGEN_UNITS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
    {
        return n.max(1);
    }
    if options.opt_level != OptLevel::O0 || options.emit_ir.is_some() {
        return 1;
    }
    let insts: usize = program.funcs.iter().flatten().map(func_weight).sum();
    let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
    (insts / INSTS_PER_UNIT).clamp(1, cores)
}

fn func_weight(func: &jaic::ir::Func) -> usize {
    func.blocks.iter().map(|b| b.insts.len() + 1).sum()
}

/// Translate `program` to native object files, splitting codegen across threads when it
/// is large. Returns the objects written: `path` itself, then `path.1.o`, `path.2.o`...
pub fn emit_objects(
    program: &Program,
    options: &Options,
    path: &Path,
) -> Result<Vec<PathBuf>, String> {
    let units = codegen_units(program, options);
    if units == 1 {
        emit_object(program, options, path)?;
        return Ok(vec![path.to_path_buf()]);
    }
    // Largest functions first, each to the lightest unit. Unit 0 also holds the globals.
    let mut order: Vec<usize> = (0..program.funcs.len()).collect();
    let weight = |i: usize| program.funcs[i].as_ref().map_or(0, func_weight);
    order.sort_by_key(|&i| std::cmp::Reverse(weight(i)));
    let mut load = vec![0usize; units];
    load[0] = program
        .globals
        .iter()
        .map(|g| g.init.len() / 64 + g.relocs.len())
        .sum();
    let mut owner = vec![0u32; program.funcs.len()];
    for i in order {
        let unit = (0..units).min_by_key(|&u| load[u]).unwrap_or(0);
        owner[i] = unit as u32;
        load[unit] += weight(i);
    }
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
    std::thread::scope(|scope| {
        let handles: Vec<_> = paths
            .iter()
            .enumerate()
            .map(|(u, p)| {
                let owner = &owner;
                scope.spawn(move || {
                    let shard = Shard {
                        owner,
                        index: u as u32,
                    };
                    emit_module(program, options, p, Some(shard))
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| {
                h.join()
                    .unwrap_or_else(|_| Err("codegen thread panicked".into()))
            })
            .collect::<Result<Vec<()>, String>>()
    })?;
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

/// Link object files into an executable with the system `cc`.
pub fn link(
    objects: &[PathBuf],
    libraries: &[Library],
    output: &Path,
    dynamic_library: bool,
    extra_args: &[String],
) -> Result<(), String> {
    let mut cmd = Command::new("cc");
    if dynamic_library {
        cmd.arg("-shared");
    }
    cmd.args(objects).arg("-o").arg(output);
    // Each library's argument group is added once (`-framework X` is two arguments).
    let mut seen: Vec<Vec<String>> = Vec::new();
    for lib in libraries {
        let args = library_args(lib)?;
        if !args.is_empty() && !seen.contains(&args) {
            seen.push(args);
        }
    }
    cmd.args(seen.concat()).args(extra_args);
    let out = cmd
        .output()
        .map_err(|e| format!("could not run the system linker 'cc': {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "linking failed ({}):\n{}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

/// Collect the DWARF of a linked macOS executable or library into `output.dSYM`.
///
/// Apple's linker does not copy debug information into its output: the binary only
/// records which object files hold it, and the CLI deletes those objects. `dsymutil`
/// gathers it into a bundle next to the binary, where lldb finds it by UUID.
pub fn write_dsym(output: &Path) -> Result<(), String> {
    let mut bundle = output.to_path_buf().into_os_string();
    bundle.push(".dSYM");
    let out = Command::new("dsymutil")
        .arg(output)
        .arg("-o")
        .arg(&bundle)
        .output()
        .map_err(|e| format!("could not run 'dsymutil' for debug information: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "dsymutil failed ({}):\n{}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

/// Linker arguments for one Jai library reference.
fn library_args(lib: &Library) -> Result<Vec<String>, String> {
    let name = lib.name.as_str();
    // libc and friends are always linked implicitly.
    if matches!(name, "c" | "libc") {
        return Ok(Vec::new());
    }
    if !lib.system {
        // A library shipped next to the source: link it by path when found.
        let path = Path::new(name);
        let file = path
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_default();
        let dir = Path::new(&lib.base_dir).join(path.parent().unwrap_or(Path::new("")));
        let ext = if cfg!(target_os = "macos") {
            "dylib"
        } else {
            "so"
        };
        // Like `jai`, a static archive wins over a shared library.
        let candidates = [
            format!("{file}.a"),
            format!("lib{file}.a"),
            format!("{file}.{ext}"),
            format!("lib{file}.{ext}"),
        ];
        for candidate in &candidates {
            let full = dir.join(candidate);
            if !full.exists() {
                continue;
            }
            if candidate.ends_with(".a") {
                return Ok(vec![full.display().to_string()]);
            }
            let parent = full.parent().unwrap_or(Path::new(".")).display();
            return Ok(vec![
                full.display().to_string(),
                format!("-Wl,-rpath,{parent}"),
            ]);
        }
        // A path is never a system library name: report it instead of a confusing `-l`.
        if name.contains('/') {
            return Err(format!(
                "library '{name}' not found: looked for {} in {}",
                candidates.join(", "),
                dir.display()
            ));
        }
    }
    // Apple frameworks (`AppKit`, `Metal`...) link with `-framework`; their directories exist on
    // disk even though the binaries live in the shared cache.
    if cfg!(target_os = "macos")
        && Path::new(&format!("/System/Library/Frameworks/{name}.framework")).exists()
    {
        return Ok(vec!["-framework".to_string(), name.to_string()]);
    }
    // Jai names libraries either way (`"libobjc"` / `"objc"`); `-l` wants the bare name.
    let name = name.strip_prefix("lib").unwrap_or(name);
    // Built third-party libraries link statically, so the executable is self-contained.
    for dir in jaic::interp::library_dirs() {
        let archive = dir.join(format!("lib{name}.a"));
        if archive.exists() {
            return Ok(vec![archive.display().to_string()]);
        }
    }
    let mut args = Vec::new();
    if cfg!(target_os = "macos") && Path::new("/opt/homebrew/lib").exists() {
        args.push("-L/opt/homebrew/lib".to_string());
    }
    args.push(format!("-l{name}"));
    Ok(args)
}
