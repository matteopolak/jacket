//! LLVM backend for the jaic compiler core.
//!
//! [`emit_object`] lowers an `ir::Program` to LLVM IR and writes a native
//! object file ([`emit_objects`] splits large unoptimized builds across threads); [`link`] turns object files into an executable with the
//! system C compiler driver.
mod debuginfo;
mod lower;
mod split;
mod wasm;

pub use wasm::{WASM_TRIPLE, WasmLink, find_wasm_ld, is_wasm_target, link_wasm};

use lower::Shard;

use inkwell::OptimizationLevel;
use inkwell::attributes::{Attribute, AttributeLoc};
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
    /// Sanitizer instrumentation (`docs/native/sanitizers.md`).
    pub sanitize: Sanitize,
    /// LLVM CPU name for a cross target (`llvm_options.target_system_cpu`); `None`: generic.
    pub cpu: Option<String>,
    /// LLVM feature string for a cross target (`llvm_options.target_system_features`, such as
    /// `+simd128`); `None`: the target's default. wasm always gets `+bulk-memory` added.
    pub features: Option<String>,
}

/// Which sanitizers instrument a native build. [`link`] links the runtime they call.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Sanitize {
    /// AddressSanitizer: heap, stack and global out-of-bounds, use-after-free, use-after-return.
    pub address: bool,
    /// The UBSan checks that exist at the LLVM IR level and match Jai semantics: accesses
    /// outside an object of known size (`bounds-checking`, Clang's `local-bounds`).
    pub undefined: bool,
}

impl Sanitize {
    pub fn any(self) -> bool {
        self.address || self.undefined
    }

    /// Parse a `-sanitize` value: a comma-separated list of `address` and `undefined`.
    pub fn parse(list: &str) -> Result<Sanitize, String> {
        let mut sanitize = Sanitize::default();
        for name in list.split(',') {
            match name.trim() {
                "address" => sanitize.address = true,
                "undefined" => sanitize.undefined = true,
                other => {
                    return Err(format!(
                        "unknown sanitizer `{other}`\nhelp: `-sanitize` takes address, undefined or both: `-sanitize address,undefined`"
                    ));
                }
            }
        }
        Ok(sanitize)
    }

    /// The Clang driver's `-fsanitize=` value; the driver then links the matching runtime.
    fn driver_flag(self) -> String {
        let names: Vec<&str> = [(self.address, "address"), (self.undefined, "undefined")]
            .into_iter()
            .filter_map(|(on, name)| on.then_some(name))
            .collect();
        format!("-fsanitize={}", names.join(","))
    }

    /// Passes appended to the optimization pipeline, where Clang runs its sanitizer passes.
    /// The bounds checks come first so that ASan does not instrument them.
    fn passes(self, opt_level: OptLevel) -> Vec<&'static str> {
        let mut passes = Vec::new();
        if self.undefined {
            // `rt-abort`: report through the UBSan runtime, then stop the program. The check
            // needs to see which object a pointer came from; unoptimized code keeps every
            // value in a stack slot, so promote those first (`sroa`) or it finds almost none.
            passes.push(if opt_level == OptLevel::O0 {
                "function(sroa,bounds-checking<rt-abort>)"
            } else {
                "function(bounds-checking<rt-abort>)"
            });
        }
        if self.address {
            passes.push("asan");
        }
        passes
    }
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
    // Target registration writes process-wide tables: once, not from every codegen thread.
    static TARGETS: std::sync::Once = std::sync::Once::new();
    TARGETS.call_once(|| Target::initialize_all(&InitializationConfig::default()));
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
        let mut features = options.features.clone().unwrap_or_default();
        // wasm: `memory.copy`/`memory.fill` for memcpy and memset. Without them LLVM calls
        // `memmove`, which Wasi_Runtime implements with that same intrinsic. Only an explicit
        // `-bulk-memory` turns them off.
        if arch.is_wasm() && !features.contains("bulk-memory") {
            if !features.is_empty() {
                features.push(',');
            }
            features.push_str("+bulk-memory");
        }
        (
            options.cpu.clone().unwrap_or_else(|| "generic".into()),
            features,
        )
    };
    // A wasm module is linked statically; PIC would ask for Emscripten-style dynamic linking.
    let reloc = if arch.is_wasm() {
        RelocMode::Static
    } else {
        RelocMode::PIC
    };
    let machine = target
        .create_target_machine(
            &triple,
            &cpu,
            &features,
            options.opt_level.llvm(),
            reloc,
            CodeModel::Default,
        )
        .ok_or("could not create a target machine")?;
    Ok((machine, triple, arch))
}

/// Lower `program` (or one shard of it) to one LLVM module and write it as an object file.
/// With `split_after_opt`, an optimized module large enough is written as several objects
/// in parallel (`split.rs`); the objects written are returned.
fn emit_module(
    program: &Program,
    options: &Options,
    path: &Path,
    shard: Option<Shard>,
    split_after_opt: bool,
) -> Result<Vec<PathBuf>, String> {
    if options.sanitize.any() {
        check_sanitizer_target(options.target.as_deref())?;
    }
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
    let mut passes: Vec<&str> = options.opt_level.pipeline().into_iter().collect();
    passes.extend(options.sanitize.passes(options.opt_level));
    if !passes.is_empty() {
        module
            .run_passes(&passes.join(","), &machine, PassBuilderOptions::create())
            .map_err(|e| e.to_string())?;
    }
    if split_after_opt && !options.sanitize.any() {
        let units = split::units_for(&module);
        if units > 1 {
            let make = || target_machine(options).map(|(machine, ..)| machine);
            return split::emit(&module, units, path, &make);
        }
    }
    machine
        .write_to_file(&module, FileType::Object, path)
        .map_err(|e| e.to_string())?;
    Ok(vec![path.to_path_buf()])
}

/// Translate `program` to a single native object file at `path`.
pub fn emit_object(program: &Program, options: &Options, path: &Path) -> Result<(), String> {
    emit_module(program, options, path, None, false).map(drop)
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
        // Optimized builds keep one module through the optimizer and split only machine
        // code generation, unless `JAIC_CODEGEN_UNITS` asked for exactly one unit.
        let split = options.opt_level != OptLevel::O0
            && options.emit_ir.is_none()
            && std::env::var_os("JAIC_CODEGEN_UNITS").is_none();
        return emit_module(program, options, path, None, split);
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
                    emit_module(program, options, p, Some(shard), false).map(drop)
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

/// The linker family a target uses.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LinkFlavor {
    /// macOS and Linux: the system `cc` driver (Clang or GCC).
    Unix,
    /// Windows with the MinGW-w64 runtime (`x86_64-pc-windows-gnu`): a MinGW GCC or Clang
    /// driver. This is what cross builds from macOS and Linux use.
    MinGw,
    /// Windows with the Microsoft toolchain (`x86_64-pc-windows-msvc`, the default on a
    /// Windows host): Clang's driver, `lld-link` or `link.exe`.
    Msvc,
}

impl LinkFlavor {
    /// The flavor for a target triple (`None`: the host).
    pub fn for_target(target: Option<&str>) -> LinkFlavor {
        let triple = match target {
            Some(t) => t.to_string(),
            None => TargetMachine::get_default_triple()
                .as_str()
                .to_string_lossy()
                .into_owned(),
        };
        if triple.contains("windows-gnu") || triple.contains("mingw") {
            LinkFlavor::MinGw
        } else if triple.contains("windows") || triple.contains("win32") {
            LinkFlavor::Msvc
        } else {
            LinkFlavor::Unix
        }
    }

    pub fn is_windows(self) -> bool {
        self != LinkFlavor::Unix
    }
}

/// The triple `jaic build -os windows` targets from a non-Windows host: the MinGW-w64
/// environment, whose cross toolchains are packaged for macOS and Linux.
pub const WINDOWS_CROSS_TRIPLE: &str = "x86_64-pc-windows-gnu";

/// One linker input, rendered per linker style.
#[derive(Clone, PartialEq, Eq, Debug)]
enum LinkArg {
    /// A library by name (`-lname`, `name.lib`).
    Lib(String),
    /// A library file by path.
    File(String),
    /// An Apple framework.
    Framework(String),
    /// Runtime search directory for a shared library next to the source.
    Rpath(String),
    /// Link-time search directory.
    SearchDir(String),
}

/// Link object files into an executable (or shared library) for `target` (`None`: the
/// host). macOS and Linux use the system `cc`; Windows uses a MinGW or MSVC toolchain,
/// see [`LinkFlavor`] and `docs/native/windows.md`. With `debug_info`, MSVC targets also get
/// a PDB next to the output (`foo.exe` -> `foo.pdb`).
#[allow(clippy::too_many_arguments)]
pub fn link(
    objects: &[PathBuf],
    libraries: &[Library],
    output: &Path,
    dynamic_library: bool,
    extra_args: &[String],
    target: Option<&str>,
    sanitize: Sanitize,
    debug_info: bool,
) -> Result<(), String> {
    let flavor = LinkFlavor::for_target(target);
    let cross = target.is_some() && flavor != LinkFlavor::for_target(None);
    if sanitize.any() {
        check_sanitizer_target(target)?;
    }
    // Each library's argument group is added once (`-framework X` is two arguments).
    let mut seen: Vec<Vec<LinkArg>> = Vec::new();
    for lib in libraries {
        let args = library_args(lib, flavor, cross)?;
        if !args.is_empty() && !seen.contains(&args) {
            seen.push(args);
        }
    }
    let (program, mut cmd) = if sanitize.any() {
        let program = sanitizer_driver();
        let mut cmd = Command::new(&program);
        cmd.arg(sanitize.driver_flag());
        (program, cmd)
    } else {
        linker_command(flavor, target)?
    };
    let msvc_style = is_msvc_linker(&program);
    if msvc_style {
        cmd.arg("/NOLOGO")
            .arg(format!("/OUT:{}", output.display()))
            .args(objects);
        cmd.arg(if dynamic_library {
            "/DLL"
        } else {
            "/SUBSYSTEM:CONSOLE"
        });
        // The dynamic CRT; `oldnames` maps POSIX names (`write`) to the CRT's underscored
        // ones, `legacy_stdio_definitions` keeps `printf` and friends linkable as functions.
        cmd.args([
            "/DEFAULTLIB:msvcrt",
            "/DEFAULTLIB:oldnames",
            "/DEFAULTLIB:legacy_stdio_definitions",
        ]);
    } else {
        if dynamic_library {
            cmd.arg("-shared");
        }
        cmd.args(objects).arg("-o").arg(output);
        if flavor == LinkFlavor::Msvc {
            // Clang's MSVC driver: the dynamic CRT, as above.
            cmd.args(["-fms-runtime-lib=dll", "-llegacy_stdio_definitions"]);
        }
    }
    // Windows reserves 1 MiB for the main thread's stack (and threads created without a
    // size); reserve the 8 MiB macOS and Linux give, so deep recursion behaves the same.
    if flavor.is_windows() && !dynamic_library {
        cmd.arg(match (msvc_style, flavor) {
            (true, _) => "/STACK:8388608",
            (false, LinkFlavor::Msvc) => "-Wl,/STACK:8388608",
            _ => "-Wl,--stack,8388608",
        });
    }
    if debug_info && flavor == LinkFlavor::Msvc {
        let prefix = if msvc_style {
            ""
        } else {
            "-Wl,"
        };
        cmd.args(pdb_args(output).iter().map(|a| format!("{prefix}{a}")));
    }
    for arg in seen.concat() {
        render_link_arg(&mut cmd, &arg, msvc_style);
    }
    cmd.args(extra_args);
    let install = if cfg!(target_os = "macos") {
        "install the Xcode command line tools (`xcode-select --install`), or name a linker with the JAIC_LINKER environment variable"
    } else if cfg!(windows) {
        "install LLVM (clang) or the Visual Studio build tools, or name a linker with the JAIC_LINKER environment variable"
    } else {
        "install a C compiler (clang or gcc, e.g. `apt install clang`), or name a linker with the JAIC_LINKER environment variable"
    };
    let out = cmd
        .output()
        .map_err(|e| linker_not_run(&program, &e, install))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(link_failure(&program, &out))
    }
}

/// The error for a linker that could not be started, with how to get one.
/// The error for a linker that could not be started; `install` says how to get one.
pub fn linker_not_run(program: &str, e: &std::io::Error, install: &str) -> String {
    if e.kind() != std::io::ErrorKind::NotFound {
        return format!("could not run the linker `{program}`: {e}");
    }
    format!(
        "could not find the linker `{program}`, which `jaic build` uses to write executables\n\
         help: {install}\n\
         help: `jaic run` needs no linker: it runs the program in the interpreter"
    )
}

/// The error for a link that failed: the linker's own output, then what usually causes it.
pub fn link_failure(program: &str, out: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let log = format!("{stdout}{stderr}");
    let mut text = format!(
        "linking failed: `{program}` stopped with {}\n{}",
        out.status,
        log.trim_end()
    );
    let missing_symbol = [
        "Undefined symbols",
        "undefined reference",
        "unresolved external",
    ]
    .iter()
    .any(|m| log.contains(m));
    let missing_library = [
        "library not found",
        "cannot find -l",
        "unable to find library",
    ]
    .iter()
    .any(|m| log.contains(m));
    if missing_library {
        text += "\nhelp: a `#library` or `#system_library` names a library the linker cannot find: \
                 check its path, or install the library";
    } else if missing_symbol {
        text += "\nhelp: a `#foreign` procedure is not in any linked library: check its name and \
                 that its `#library` is the one that defines it";
    }
    text
}

/// `link.exe`/`lld-link` arguments that keep the objects' CodeView in a PDB named after the
/// output. The objects are deleted after linking, so without `/DEBUG` the debug information is
/// lost. `/DEBUG` would otherwise also turn on incremental linking (an `.ilk` file and padded
/// code) and keep unreferenced functions; both are turned back off to match a build without it.
fn pdb_args(output: &Path) -> [String; 4] {
    [
        "/DEBUG".to_string(),
        format!("/PDB:{}", output.with_extension("pdb").display()),
        "/INCREMENTAL:NO".to_string(),
        "/OPT:REF".to_string(),
    ]
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

/// Archive object files into a static library for `target`: the system `ar` on macOS and
/// Linux; for Windows a MinGW `ar`, `llvm-ar` or `lib.exe` (`JAIC_AR` overrides the choice).
pub fn archive(objects: &[PathBuf], output: &Path, target: Option<&str>) -> Result<(), String> {
    let program = match std::env::var("JAIC_AR") {
        Ok(program) => program,
        // The system `ar` on macOS writes no symbol index for wasm objects.
        Err(_) if target.is_some_and(is_wasm_target) => wasm::find_llvm_tool("llvm-ar")
            .ok_or("no llvm-ar found for a WebAssembly archive; set JAIC_AR")?,
        Err(_) if LinkFlavor::for_target(target).is_windows() => {
            let mingw_ar = format!("{}-w64-mingw32-ar", mingw_cpu(target));
            find_program(&[&mingw_ar, "llvm-ar", "llvm-lib", "lib"])
                .ok_or("no archiver for Windows libraries found: install LLVM or mingw-w64")?
        }
        Err(_) => "ar".to_string(),
    };
    let stem = Path::new(&program)
        .file_stem()
        .map(|s| s.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    let mut cmd = Command::new(&program);
    if matches!(stem.as_str(), "lib" | "llvm-lib") {
        cmd.arg("/NOLOGO").arg(format!("/OUT:{}", output.display()));
    } else {
        // `ar` appends to an existing archive; start from an empty one.
        let _ = std::fs::remove_file(output);
        cmd.arg("rcs").arg(output);
    }
    let status = cmd
        .args(objects)
        .status()
        .map_err(|e| format!("could not run '{program}': {e}"))?;
    status
        .success()
        .then_some(())
        .ok_or(format!("{program} failed ({status})"))
}

fn render_link_arg(cmd: &mut Command, arg: &LinkArg, msvc_style: bool) {
    match (arg, msvc_style) {
        (LinkArg::Lib(name), false) => cmd.arg(format!("-l{name}")),
        (LinkArg::Lib(name), true) => cmd.arg(format!("{name}.lib")),
        (LinkArg::File(path), _) => cmd.arg(path),
        (LinkArg::Framework(name), _) => cmd.arg("-framework").arg(name),
        (LinkArg::Rpath(dir), _) => cmd.arg(format!("-Wl,-rpath,{dir}")),
        (LinkArg::SearchDir(dir), false) => cmd.arg(format!("-L{dir}")),
        (LinkArg::SearchDir(dir), true) => cmd.arg(format!("/LIBPATH:{dir}")),
    };
}

/// Whether `program` takes `link.exe`-style arguments rather than a C compiler driver's.
fn is_msvc_linker(program: &str) -> bool {
    let stem = Path::new(program)
        .file_stem()
        .map(|s| s.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    matches!(stem.as_str(), "link" | "lld-link")
}

/// Sanitized builds run only on the host, and only on macOS and Linux: the runtimes come from
/// the host's LLVM install, and jaic does not drive Clang's Windows or wasm sanitizer setups.
pub fn check_sanitizer_target(target: Option<&str>) -> Result<(), String> {
    if target.is_some() {
        return Err("-sanitize supports only native builds for the host, not cross builds".into());
    }
    if !cfg!(any(target_os = "macos", target_os = "linux")) {
        return Err("-sanitize is supported on macOS and Linux hosts only".into());
    }
    Ok(())
}

/// The Clang driver that links a sanitized build. The instrumentation comes from the LLVM jaic
/// is built with and calls into a runtime of the same version, so prefer that install's
/// `clang` over the system `cc` (Apple's Clang ships an older runtime; GCC's `libasan` is a
/// different one). `JAIC_SANITIZER_CC` overrides the choice.
fn sanitizer_driver() -> String {
    if let Ok(program) = std::env::var("JAIC_SANITIZER_CC") {
        return program;
    }
    let prefixes = [
        std::env::var("LLVM_SYS_231_PREFIX").ok(),
        option_env!("LLVM_SYS_231_PREFIX").map(str::to_string),
    ];
    for prefix in prefixes.into_iter().flatten() {
        let clang = Path::new(&prefix).join("bin/clang");
        if clang.is_file() {
            return clang.to_string_lossy().into_owned();
        }
    }
    for config in ["llvm-config-23", "llvm-config"] {
        let Ok(out) = Command::new(config).arg("--bindir").output() else {
            continue;
        };
        let clang = Path::new(String::from_utf8_lossy(&out.stdout).trim()).join("clang");
        if out.status.success() && clang.is_file() {
            return clang.to_string_lossy().into_owned();
        }
    }
    find_program(&["clang-23", "clang"]).unwrap_or_else(|| "clang".into())
}

/// The first of `names` found on `PATH`.
fn find_program(names: &[&str]) -> Option<String> {
    let path = std::env::var_os("PATH")?;
    for name in names {
        for dir in std::env::split_paths(&path) {
            if dir.join(name).is_file() || dir.join(format!("{name}.exe")).is_file() {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// The linker program and its base command. `JAIC_LINKER` overrides the choice: a program
/// named `link` or `lld-link` gets MSVC-style arguments, anything else C-driver arguments.
fn linker_command(flavor: LinkFlavor, target: Option<&str>) -> Result<(String, Command), String> {
    let msvc_triple = match target {
        Some(t) => t.to_string(),
        None => TargetMachine::get_default_triple()
            .as_str()
            .to_string_lossy()
            .into_owned(),
    };
    if let Some(program) = std::env::var_os("JAIC_LINKER") {
        let program = program.to_string_lossy().into_owned();
        let mut cmd = Command::new(&program);
        if flavor == LinkFlavor::Msvc && !is_msvc_linker(&program) {
            cmd.arg(format!("--target={msvc_triple}"));
        }
        return Ok((program, cmd));
    }
    match flavor {
        LinkFlavor::Unix => {
            let mut cmd = Command::new("cc");
            // The other architecture of a Mac (`x86_64-apple-darwin` on arm64, run through
            // Rosetta, and the reverse): Apple's `cc` is clang and links either.
            if let Some(triple) = target
                && cfg!(target_os = "macos")
                && (triple.contains("apple") || triple.contains("darwin"))
            {
                cmd.arg(format!("--target={triple}"));
            }
            Ok(("cc".into(), cmd))
        }
        LinkFlavor::MinGw => {
            // GCC only targets x64; llvm-mingw provides `-clang` (and a `-gcc` alias of it)
            // for both CPUs.
            let cpu = mingw_cpu(target);
            let gcc = format!("{cpu}-w64-mingw32-gcc");
            let clang = format!("{cpu}-w64-mingw32-clang");
            let mut names = vec![gcc.as_str(), clang.as_str()];
            if cfg!(windows) {
                names.extend(["gcc", "clang"]);
            }
            let program = find_program(&names).ok_or_else(|| {
                let install = if cpu == "aarch64" {
                    "install llvm-mingw (it provides aarch64-w64-mingw32-clang)"
                } else {
                    "install mingw-w64 (it provides x86_64-w64-mingw32-gcc)"
                };
                format!(
                    "no MinGW-w64 linker found to link for Windows\nhelp: {install}, or name a linker with JAIC_LINKER"
                )
            })?;
            let mut cmd = Command::new(&program);
            if program.ends_with("clang") {
                cmd.arg(format!(
                    "--target={}",
                    target.unwrap_or(WINDOWS_CROSS_TRIPLE)
                ));
            }
            Ok((program, cmd))
        }
        LinkFlavor::Msvc => {
            // Clang's driver finds the MSVC and Windows SDK libraries without a developer
            // prompt; `lld-link`/`link.exe` need one (the `LIB` environment variable).
            if let Some(program) = find_program(&["clang"]) {
                let mut cmd = Command::new(&program);
                cmd.arg(format!("--target={msvc_triple}"));
                return Ok((program, cmd));
            }
            let program = find_program(&["lld-link", "link"]).ok_or(
                "no MSVC linker found to link for Windows\n\
                 help: install LLVM (clang) or the Visual Studio build tools, or name a linker with JAIC_LINKER",
            )?;
            let cmd = Command::new(&program);
            Ok((program, cmd))
        }
    }
}

/// The CPU prefix of the MinGW-w64 tool names for `target` (`None`: the host).
fn mingw_cpu(target: Option<&str>) -> &'static str {
    let arm = match target {
        Some(t) => t.starts_with("aarch64") || t.starts_with("arm64"),
        None => cfg!(target_arch = "aarch64"),
    };
    if arm {
        "aarch64"
    } else {
        "x86_64"
    }
}

/// What a native build writes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OutputKind {
    Executable,
    DynamicLibrary,
    StaticLibrary,
}

/// The file name extension an output gets on `target` when its name has none, where the
/// platform expects one: `exe`, `dll` and `lib` on Windows, `wasm` (or `a`) for WebAssembly.
pub fn output_extension(target: Option<&str>, kind: OutputKind) -> Option<&'static str> {
    if target.is_some_and(is_wasm_target) {
        return Some(match kind {
            OutputKind::StaticLibrary => "a",
            _ => "wasm",
        });
    }
    if !LinkFlavor::for_target(target).is_windows() {
        return None;
    }
    Some(match kind {
        OutputKind::Executable => "exe",
        OutputKind::DynamicLibrary => "dll",
        OutputKind::StaticLibrary => "lib",
    })
}

/// Linker inputs for one Jai library reference. `cross`: the target is not the host, so the
/// host's library directories (Homebrew, native-libs builds, frameworks) do not apply.
fn library_args(lib: &Library, flavor: LinkFlavor, cross: bool) -> Result<Vec<LinkArg>, String> {
    let name = lib.name.as_str();
    // libc and friends are always linked implicitly.
    if matches!(name, "c" | "libc") {
        return Ok(Vec::new());
    }
    // On Windows the toolchain picks the C runtime (see `link`); POSIX-only names are not
    // libraries there.
    if flavor.is_windows()
        && matches!(
            name.to_ascii_lowercase().as_str(),
            "msvcrt"
                | "ucrt"
                | "ucrtbase"
                | "vcruntime"
                | "libcmt"
                | "m"
                | "libm"
                | "pthread"
                | "libpthread"
                | "dl"
                | "libdl"
                | "rt"
                | "librt"
        )
    {
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
        // Like `jai`, a static archive wins over a shared library.
        let candidates: Vec<String> = match flavor {
            LinkFlavor::Unix => {
                let ext = if cfg!(target_os = "macos") {
                    "dylib"
                } else {
                    "so"
                };
                vec![
                    format!("{file}.a"),
                    format!("lib{file}.a"),
                    format!("{file}.{ext}"),
                    format!("lib{file}.{ext}"),
                ]
            }
            // `name.lib` is a static library or a DLL's import library; MinGW's `ld` also
            // reads GNU archives and links against a DLL directly.
            LinkFlavor::MinGw => vec![
                format!("{file}.lib"),
                format!("{file}.a"),
                format!("lib{file}.a"),
                format!("{file}.dll.a"),
                format!("lib{file}.dll.a"),
                format!("{file}.dll"),
            ],
            LinkFlavor::Msvc => vec![format!("{file}.lib"), format!("lib{file}.lib")],
        };
        for candidate in &candidates {
            let full = dir.join(candidate);
            if !full.exists() {
                continue;
            }
            let mut args = vec![LinkArg::File(full.display().to_string())];
            if candidate.ends_with(".dylib") || candidate.ends_with(".so") {
                let parent = full.parent().unwrap_or(Path::new("."));
                args.push(LinkArg::Rpath(parent.display().to_string()));
            }
            return Ok(args);
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
    if flavor.is_windows() {
        // Import libraries of system DLLs (`kernel32`, `user32`...) come with the toolchain.
        return Ok(vec![LinkArg::Lib(name.to_string())]);
    }
    // Apple frameworks (`AppKit`, `Metal`...) link with `-framework`; their directories exist on
    // disk even though the binaries live in the shared cache.
    if cfg!(target_os = "macos")
        && !cross
        && Path::new(&format!("/System/Library/Frameworks/{name}.framework")).exists()
    {
        return Ok(vec![LinkArg::Framework(name.to_string())]);
    }
    // Jai names libraries either way (`"libobjc"` / `"objc"`); `-l` wants the bare name.
    let name = name.strip_prefix("lib").unwrap_or(name);
    // Built third-party libraries link statically, so the executable is self-contained.
    if !cross {
        for dir in jaic::interp::library_dirs() {
            let archive = dir.join(format!("lib{name}.a"));
            if archive.exists() {
                return Ok(vec![LinkArg::File(archive.display().to_string())]);
            }
        }
    }
    let mut args = Vec::new();
    if cfg!(target_os = "macos") && !cross && Path::new("/opt/homebrew/lib").exists() {
        args.push(LinkArg::SearchDir("/opt/homebrew/lib".to_string()));
    }
    args.push(LinkArg::Lib(name.to_string()));
    Ok(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizer_lists_parse_into_flags_and_passes() {
        let both = Sanitize::parse("address,undefined").unwrap();
        assert!(both.address && both.undefined);
        assert_eq!(both.driver_flag(), "-fsanitize=address,undefined");
        assert_eq!(
            both.passes(OptLevel::O2),
            ["function(bounds-checking<rt-abort>)", "asan"]
        );
        // Unoptimized code needs its slots promoted before bounds checks see any object.
        assert_eq!(
            Sanitize::parse("undefined").unwrap().passes(OptLevel::O0),
            ["function(sroa,bounds-checking<rt-abort>)"]
        );
        assert_eq!(
            Sanitize::parse("address").unwrap().driver_flag(),
            "-fsanitize=address"
        );
        assert!(Sanitize::parse("thread").is_err());
        assert!(check_sanitizer_target(Some(WINDOWS_CROSS_TRIPLE)).is_err());
        assert!(check_sanitizer_target(Some("wasm32-unknown-unknown")).is_err());
    }

    #[test]
    fn windows_triples_pick_windows_linkers() {
        assert_eq!(
            LinkFlavor::for_target(Some("x86_64-pc-windows-gnu")),
            LinkFlavor::MinGw
        );
        assert_eq!(
            LinkFlavor::for_target(Some("x86_64-pc-windows-msvc")),
            LinkFlavor::Msvc
        );
        assert_eq!(
            LinkFlavor::for_target(Some("x86_64-unknown-linux-gnu")),
            LinkFlavor::Unix
        );
        assert_eq!(
            output_extension(Some(WINDOWS_CROSS_TRIPLE), OutputKind::Executable),
            Some("exe")
        );
        assert_eq!(
            output_extension(Some(WINDOWS_CROSS_TRIPLE), OutputKind::DynamicLibrary),
            Some("dll")
        );
    }

    #[test]
    fn windows_arm64_triples_pick_windows_linkers() {
        assert_eq!(
            LinkFlavor::for_target(Some("aarch64-pc-windows-msvc")),
            LinkFlavor::Msvc
        );
        for mingw in ["aarch64-pc-windows-gnu", "aarch64-w64-mingw32"] {
            assert_eq!(LinkFlavor::for_target(Some(mingw)), LinkFlavor::MinGw);
            assert_eq!(mingw_cpu(Some(mingw)), "aarch64");
        }
        assert_eq!(mingw_cpu(Some(WINDOWS_CROSS_TRIPLE)), "x86_64");
        assert_eq!(
            output_extension(Some("aarch64-pc-windows-gnu"), OutputKind::Executable),
            Some("exe")
        );
    }

    #[test]
    fn windows_system_libraries_link_by_name() {
        let lib = |name: &str| Library {
            name: name.into(),
            system: true,
            link_always: false,
            base_dir: String::new(),
        };
        let args = |name: &str, flavor| library_args(&lib(name), flavor, true).unwrap();
        assert_eq!(
            args("kernel32", LinkFlavor::MinGw),
            vec![LinkArg::Lib("kernel32".into())]
        );
        assert!(args("msvcrt", LinkFlavor::Msvc).is_empty());
        assert!(args("libc", LinkFlavor::MinGw).is_empty());
        let mut cmd = Command::new("link");
        render_link_arg(&mut cmd, &LinkArg::Lib("user32".into()), true);
        assert_eq!(cmd.get_args().next().unwrap(), "user32.lib");
    }
}
