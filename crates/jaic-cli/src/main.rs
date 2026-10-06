//! `jaic` command line: `jaic <run|check|build> <file.jai> [-I dir]... [-o out]`.
use jaic::build::{BuildEnv, BuildSettings, OutputBackend, OutputType, Workspaces};
use jaic::interp::{NativeHost, SandboxHost, SharedHost};
use jaic::sema::{Compiler, FileSystem, NativeFs, Options, ProgramSource, TargetCpu, TargetOs};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::rc::Rc;

/// `--timings`: wall time per compiler phase, printed to stderr when the command ends.
mod timings {
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::{Duration, Instant};

    static ENABLED: AtomicBool = AtomicBool::new(false);
    /// Phases in first-seen order; a phase that runs more than once (one backend call per
    /// workspace) accumulates.
    static PHASES: Mutex<Vec<(&'static str, Duration, u32)>> = Mutex::new(Vec::new());

    pub fn enable() {
        ENABLED.store(true, Ordering::Relaxed);
    }

    /// Run `f`, adding its wall time to `phase`.
    pub fn time<T>(phase: &'static str, f: impl FnOnce() -> T) -> T {
        if !ENABLED.load(Ordering::Relaxed) {
            return f();
        }
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed();
        let mut phases = PHASES.lock().unwrap();
        match phases.iter_mut().find(|(name, ..)| *name == phase) {
            Some((_, total, count)) => {
                *total += elapsed;
                *count += 1;
            }
            None => phases.push((phase, elapsed, 1)),
        }
        result
    }

    /// One `jaic-timing: <phase> <seconds> <calls>` line per phase (tools/compile_bench.py
    /// parses them).
    pub fn report() {
        if !ENABLED.load(Ordering::Relaxed) {
            return;
        }
        for (name, total, count) in PHASES.lock().unwrap().iter() {
            eprintln!("jaic-timing: {name} {:.6} {count}", total.as_secs_f64());
        }
    }
}

fn stdlib_dir() -> PathBuf {
    jaic::stdlib_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"))
}

/// Where `tools/build_native_libs.py` puts third-party libraries for this host, or the
/// `JAIC_NATIVE_LIBS` path list.
fn native_lib_dirs(stdlib: &Path) -> Vec<PathBuf> {
    if let Some(dirs) = std::env::var_os("JAIC_NATIVE_LIBS") {
        return std::env::split_paths(&dirs).collect();
    }
    let os = match std::env::consts::OS {
        "macos" => "macos",
        "linux" => "linux",
        _ => return Vec::new(),
    };
    let arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        "x86_64" => "x64",
        _ => return Vec::new(),
    };
    let dir = stdlib.join(format!("../artifacts/native-libs/{os}-{arch}"));
    dir.canonicalize().map(|d| vec![d]).unwrap_or_default()
}

fn usage() -> ExitCode {
    eprintln!(
        "usage: jaic <run|check> <file.jai> [-I|-import_dir dir]... [-os linux|windows|macos|wasm] [- metaprogram args...] [-- program args...]"
    );
    eprintln!(
        "       jaic build <file.jai> [-I dir]... [-o output] [-O0|-O1|-O2|-O3] [--emit-ir file.ll] [--no-debug-info] [-os windows] [-cpu x64|arm64] [-target triple]"
    );
    eprintln!(
        "       jaic <check|build> <file.jai> -plug Module [-plug Module]... [plugin options...]"
    );
    eprintln!("       --timings (any command): wall time per phase on stderr");
    ExitCode::from(2)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Command {
    Check,
    Run,
    Build,
}

/// Parsed command line.
struct Cli {
    command: Command,
    file: String,
    imports: Vec<PathBuf>,
    output: Option<PathBuf>,
    /// `-O0`..`-O3`; `None`: what the metaprogram chose (default: unoptimized).
    opt_level: Option<&'static str>,
    emit_ir: Option<PathBuf>,
    /// `--no-debug-info`: omit native debug information (on by default, like `jai`).
    no_debug_info: bool,
    /// Arguments after `-`, for the metaprogram (`compiler_get_command_line`).
    command_line: Vec<String>,
    /// `run` only: arguments after `--`, for the program (`get_command_line_arguments`).
    program_args: Vec<String>,
    /// `-os`: the target `OS` when it is not the host (checking code for another platform).
    os: Option<TargetOs>,
    /// `-cpu`: the target `CPU` when it is not the host's (`-os windows -cpu arm64`).
    cpu: Option<TargetCpu>,
    /// `-plug Name`: metaprogram plugin modules (`Name` may carry module parameters,
    /// `Check(CHECK_BINDINGS=false)`).
    plugins: Vec<String>,
    /// Arguments jaic does not know; they are the plugins' options (an error without plugins).
    plugin_options: Vec<String>,
    /// `-target`: an explicit LLVM target triple (`x86_64-pc-windows-msvc`...).
    target: Option<String>,
    /// `--timings`: report the wall time of each phase on stderr.
    timings: bool,
}

impl Cli {
    /// The LLVM triple to build for, `None` for the host. `-os windows` on another host
    /// cross-compiles with MinGW-w64 (x64 unless `-cpu arm64`); other cross targets need an
    /// explicit `-target`.
    fn target_triple(&self) -> Result<Option<String>, String> {
        self.target_triple_from(Options::host().os, Options::host().cpu)
    }

    fn target_triple_from(
        &self,
        host_os: TargetOs,
        host_cpu: TargetCpu,
    ) -> Result<Option<String>, String> {
        if let Some(triple) = &self.target {
            return Ok(Some(triple.clone()));
        }
        let os = self.os.unwrap_or(host_os);
        let cpu = self.cpu.unwrap_or(host_cpu);
        if os == host_os && cpu == host_cpu {
            return Ok(None);
        }
        match os {
            // A Windows host keeps its own toolchain (MSVC) for the other CPU. Elsewhere x64
            // is the default: it is what most Windows machines run, and Windows on arm64
            // runs x64 programs too.
            TargetOs::Windows => Ok(Some(
                windows_triple(
                    self.cpu.unwrap_or(TargetCpu::X64),
                    host_os == TargetOs::Windows,
                )
                .to_string(),
            )),
            _ if self.command != Command::Build => Ok(None),
            _ => Err(
                "native cross-compilation is only supported for -os windows; pass -target <triple> for others"
                    .into(),
            ),
        }
    }
}

/// The triple a Windows build for `cpu` targets: the MSVC environment on a Windows host,
/// MinGW-w64 (whose cross toolchains exist for macOS and Linux) elsewhere.
fn windows_triple(cpu: TargetCpu, msvc: bool) -> &'static str {
    match (cpu, msvc) {
        (TargetCpu::Arm64, true) => "aarch64-pc-windows-msvc",
        (TargetCpu::Arm64, false) => "aarch64-pc-windows-gnu",
        (_, true) => "x86_64-pc-windows-msvc",
        (_, false) => "x86_64-pc-windows-gnu",
    }
}

/// `OS` and `CPU` as a target triple implies them.
fn os_and_cpu(triple: &str) -> (TargetOs, TargetCpu) {
    let os = if triple.contains("windows") || triple.contains("mingw") {
        TargetOs::Windows
    } else if triple.contains("apple") || triple.contains("darwin") || triple.contains("macos") {
        TargetOs::MacOS
    } else if triple.starts_with("wasm") {
        TargetOs::Wasm
    } else {
        TargetOs::Linux
    };
    let cpu = if triple.starts_with("aarch64") || triple.starts_with("arm64") {
        TargetCpu::Arm64
    } else if triple.starts_with("wasm") {
        TargetCpu::Wasm
    } else {
        TargetCpu::X64
    };
    (os, cpu)
}

fn parse(args: &[String]) -> Option<Cli> {
    let command = match args.first()?.as_str() {
        "check" => Command::Check,
        "run" => Command::Run,
        "build" => Command::Build,
        _ => return None,
    };
    let mut cli = Cli {
        command,
        file: args.get(1)?.clone(),
        imports: Vec::new(),
        output: None,
        opt_level: None,
        emit_ir: None,
        no_debug_info: false,
        command_line: Vec::new(),
        program_args: Vec::new(),
        os: None,
        cpu: None,
        target: None,
        plugins: Vec::new(),
        plugin_options: Vec::new(),
        timings: false,
    };
    let mut rest = args[2..].iter();
    while let Some(a) = rest.next() {
        match a.as_str() {
            "-" => {
                // Metaprogram arguments run up to a `--` (if any).
                for arg in rest.by_ref() {
                    if arg == "--" && command == Command::Run {
                        cli.program_args.push(cli.file.clone());
                        break;
                    }
                    cli.command_line.push(arg.clone());
                }
                cli.program_args.extend(rest.by_ref().cloned());
            }
            "--" if command == Command::Run => {
                // argv[0] is the source file, like a built program's executable path.
                cli.program_args.push(cli.file.clone());
                cli.program_args.extend(rest.by_ref().cloned());
            }
            "-I" | "-import_dir" => cli.imports.extend(rest.next().map(PathBuf::from)),
            "-os" => {
                cli.os = Some(match rest.next()?.as_str() {
                    "linux" => TargetOs::Linux,
                    "windows" => TargetOs::Windows,
                    "macos" => TargetOs::MacOS,
                    "wasm" => TargetOs::Wasm,
                    _ => return None,
                })
            }
            "-cpu" => {
                cli.cpu = Some(match rest.next()?.as_str() {
                    "x64" | "x86_64" => TargetCpu::X64,
                    "arm64" | "aarch64" => TargetCpu::Arm64,
                    _ => return None,
                })
            }
            "-target" | "--target" => cli.target = Some(rest.next()?.clone()),
            "--timings" => cli.timings = true,
            "-o" if command == Command::Build => cli.output = Some(PathBuf::from(rest.next()?)),
            "--emit-ir" if command == Command::Build => {
                cli.emit_ir = Some(PathBuf::from(rest.next()?))
            }
            "--no-debug-info" if command == Command::Build => cli.no_debug_info = true,
            "-O0" if command == Command::Build => cli.opt_level = Some("O0"),
            "-O1" if command == Command::Build => cli.opt_level = Some("O1"),
            "-O2" if command == Command::Build => cli.opt_level = Some("O2"),
            "-O3" if command == Command::Build => cli.opt_level = Some("O3"),
            "-plug" | "-plugin" => cli.plugins.push(rest.next()?.clone()),
            // An unknown option and everything after it (its values) go to the plugins.
            other if other.starts_with('-') || !cli.plugin_options.is_empty() => {
                cli.plugin_options.push(other.to_string())
            }
            _ => return None,
        }
    }
    // Plugins compile the program in a workspace of their own, which `run` cannot start.
    if !cli.plugin_options.is_empty() && cli.plugins.is_empty()
        || !cli.plugins.is_empty() && command == Command::Run
    {
        return None;
    }
    Some(cli)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cli) = parse(&args) else {
        return usage();
    };
    // Deeply recursive programs and checking need a large stack.
    // On macOS the main thread stays free to run the program's foreign calls (AppKit only
    // works there); see `jaic::interp::main_thread`.
    #[cfg(target_os = "macos")]
    {
        jaic::interp::main_thread::serve(1 << 30, move || run(cli)).unwrap_or(ExitCode::from(101))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let worker = std::thread::Builder::new()
            .stack_size(1 << 30)
            .spawn(move || run(cli));
        match worker.map(|h| h.join()) {
            Ok(Ok(code)) => code,
            _ => ExitCode::from(101),
        }
    }
}

fn run(cli: Cli) -> ExitCode {
    if cli.timings {
        timings::enable();
    }
    let code = timings::time("total", || compile_and_run(cli));
    timings::report();
    // Interpreters flush their `JAIC_PROFILE` counts when dropped, which has happened by now.
    if let Some(report) = jaic::interp::profile::report(40) {
        eprint!("{report}");
    }
    code
}

fn compile_and_run(mut cli: Cli) -> ExitCode {
    let stdlib = stdlib_dir();
    jaic::interp::set_library_dirs(native_lib_dirs(&stdlib));
    let path = std::fs::canonicalize(&cli.file).unwrap_or_else(|_| PathBuf::from(&cli.file));
    // Like `jai`, run from the main file's directory (so the program's meaning does not
    // depend on where the compiler was started); paths given on the command line stay
    // relative to the original directory.
    let absolute = |p: &PathBuf| std::path::absolute(p).unwrap_or_else(|_| p.clone());
    cli.imports = cli.imports.iter().map(absolute).collect();
    cli.output = cli.output.as_ref().map(absolute);
    cli.emit_ir = cli.emit_ir.as_ref().map(absolute);
    let main_dir = path.parent().map(PathBuf::from).unwrap_or_default();
    if !main_dir.as_os_str().is_empty() && std::env::set_current_dir(&main_dir).is_err() {
        eprintln!("error: cannot change directory to {}", main_dir.display());
        return ExitCode::from(1);
    }
    let mut options = Options::host();
    if let Some(os) = cli.os {
        options.os = os;
    }
    if let Some(cpu) = cli.cpu {
        options.cpu = cpu;
    }
    // Only native output has a use for variable and type descriptions.
    options.debug_info = cli.command == Command::Build && !cli.no_debug_info;
    match cli.target_triple() {
        Ok(Some(triple)) => {
            let (os, cpu) = os_and_cpu(&triple);
            options.os = os;
            options.cpu = cpu;
        }
        Ok(None) => {}
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::from(2);
        }
    }
    // The local `modules` folder is searched first, then `-import_dir`s, then the stdlib.
    options.import_paths = vec![main_dir.join("modules")];
    options.import_paths.extend(cli.imports.iter().cloned());
    options.import_paths.push(stdlib.clone());
    options.preload = Some(stdlib.join("prelude/module.jai"));
    let fs: Rc<dyn FileSystem> = Rc::new(NativeFs);
    // Workspaces created by metaprograms are written only by `build`.
    let backend: Option<Box<dyn OutputBackend>> = (cli.command == Command::Build)
        .then(|| Box::new(native_backend(&cli)) as Box<dyn OutputBackend>);
    // `-os wasm` runs the program the way the browser does: in the sandbox host (virtual clock and
    // files, cooperative threads), with its output printed when the run ends.
    let sandbox = (options.os == TargetOs::Wasm).then(|| {
        let cwd = std::env::current_dir().unwrap_or_default();
        Rc::new(RefCell::new(SandboxHost::with_files(
            fs.clone(),
            &cwd.to_string_lossy(),
        )))
    });
    let workspace_sandbox = sandbox.clone();
    let workspaces = Workspaces::new(BuildEnv {
        fs: fs.clone(),
        options: options.clone(),
        backend,
        command_line: cli.command_line.clone(),
        make_host: Box::new(move || match &workspace_sandbox {
            Some(host) => Box::new(SharedHost(host.clone())),
            None => Box::new(NativeHost),
        }),
        report: Box::new(|text| eprintln!("{text}")),
    });
    let mut compiler = Compiler::new(options, fs);
    if let Some(host) = &sandbox {
        compiler.interp.host = Box::new(SharedHost(host.clone()));
    }
    compiler.attach_workspaces(workspaces.clone());
    let compiled = if cli.plugins.is_empty() {
        timings::time("front end", || compiler.compile_program(&path))
    } else {
        let source = plugin_metaprogram(&cli, &path);
        timings::time("front end", || {
            compiler.compile_sources(&[ProgramSource::String(source)])
        })
    };
    if let Err(d) = compiled {
        eprintln!("{}", compiler.render(&d));
        return ExitCode::from(1);
    }
    if let Err(message) = timings::time("workspaces", || jaic::build::finish_all(&workspaces)) {
        eprintln!("error: {message}");
        return ExitCode::from(1);
    }
    if workspaces.borrow().any_failed() {
        return ExitCode::from(1);
    }
    let settings = workspaces.borrow().top_level_settings();
    match cli.command {
        Command::Check => ExitCode::SUCCESS,
        Command::Run => {
            // The sandbox's memory is virtual; host-allocated argv strings are not visible there.
            let outcome = timings::time("run", || {
                if sandbox.is_some() {
                    compiler.run_program()
                } else {
                    compiler.run_program_with_args(&cli.program_args)
                }
            });
            if let Some(host) = &sandbox {
                use std::io::Write;
                let host = host.borrow();
                let _ = std::io::stdout().write_all(&host.stdout);
                let _ = std::io::stderr().write_all(&host.stderr);
            }
            match outcome {
                Ok(code) => ExitCode::from(code as u8),
                Err(d) => {
                    eprintln!("{}", compiler.render(&d));
                    ExitCode::from(1)
                }
            }
        }
        // A metaprogram that turned its own output off has nothing to write.
        Command::Build if !settings.do_output || settings.output_type == OutputType::NoOutput => {
            ExitCode::SUCCESS
        }
        Command::Build => match build(&mut compiler, &cli, &path, settings) {
            Ok(()) => ExitCode::SUCCESS,
            Err(message) => {
                eprintln!("error: {message}");
                ExitCode::from(1)
            }
        },
    }
}

/// The metaprogram `-plug` stands for: import each plugin module, then compile `path` in a
/// workspace with their hooks (`build_with_plugins` in `Metaprogram_Plugins`).
fn plugin_metaprogram(cli: &Cli, path: &Path) -> String {
    let quote = |s: &str| format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""));
    let mut text = String::from(
        "Compiler :: #import \"Compiler\";\nPlugins :: #import \"Metaprogram_Plugins\";\n#import \"Basic\";\n",
    );
    for (i, plugin) in cli.plugins.iter().enumerate() {
        // `Name(PARAM=value)` passes module parameters along.
        let (name, params) = plugin.split_at(plugin.find('(').unwrap_or(plugin.len()));
        text += &format!("__plugin_{i} :: #import {}{params};\n", quote(name));
    }
    text += "#run,stallable {\n    plugins: [..] *Compiler.Metaprogram_Plugin;\n";
    for i in 0..cli.plugins.len() {
        text += &format!("    array_add(*plugins, __plugin_{i}.get_plugin());\n");
    }
    text += "    options: [..] string;\n";
    for option in &cli.plugin_options {
        text += &format!("    array_add(*options, {});\n", quote(option));
    }
    // `-o` names the executable; otherwise it is named after the file, next to it.
    let lossy = |p: Option<&std::ffi::OsStr>| p.map(|n| n.to_string_lossy().into_owned());
    let (name, dir) = match &cli.output {
        Some(out) => (
            lossy(out.file_name()),
            lossy(out.parent().map(|d| d.as_os_str())),
        ),
        None => (lossy(path.file_stem()), None),
    };
    text += &format!(
        "    Plugins.build_with_plugins({}, plugins, options, {}, {});\n}}\n",
        quote(&path.to_string_lossy()),
        quote(&name.unwrap_or_default()),
        quote(&dir.unwrap_or_default()),
    );
    text
}

/// Write the top-level program. `-o`/`-O` override what the metaprogram set.
fn build(
    compiler: &mut Compiler,
    cli: &Cli,
    source: &Path,
    mut settings: BuildSettings,
) -> Result<(), String> {
    if settings.output_type == OutputType::Executable && compiler.exported_func("main").is_none() {
        return Err("no exported 'main' (is Runtime_Support loaded?)".into());
    }
    let output = cli.output.clone().unwrap_or_else(|| {
        let name = if settings.output_executable_name.is_empty() {
            source
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        } else {
            settings.output_executable_name.clone()
        };
        PathBuf::from(&settings.output_path).join(name)
    });
    if let Some(level) = cli.opt_level {
        settings.optimization = level.into();
    }
    timings::time("prepare output", || compiler.prepare_compiled_output());
    native_backend(cli).write_output(&compiler.program, &settings, &output)
}

#[cfg(feature = "llvm")]
fn native_backend(cli: &Cli) -> LlvmBackend {
    LlvmBackend {
        emit_ir: cli.emit_ir.clone(),
        debug_info: !cli.no_debug_info,
        target: cli.target_triple().ok().flatten(),
    }
}

/// Without the `llvm` feature (an interpreter-only build, e.g. for a host without LLVM
/// libraries) `build` reports that it cannot write native output.
#[cfg(not(feature = "llvm"))]
fn native_backend(_cli: &Cli) -> NoBackend {
    NoBackend
}

#[cfg(not(feature = "llvm"))]
struct NoBackend;

#[cfg(not(feature = "llvm"))]
impl OutputBackend for NoBackend {
    fn write_output(
        &mut self,
        _program: &jaic::ir::Program,
        _settings: &BuildSettings,
        _output: &Path,
    ) -> Result<(), String> {
        Err("this jaic was built without the `llvm` feature and cannot write native output".into())
    }
}

/// Native output through `jaic-llvm` and the system linker.
#[cfg(feature = "llvm")]
struct LlvmBackend {
    emit_ir: Option<PathBuf>,
    debug_info: bool,
    /// Target triple; `None` for the host.
    target: Option<String>,
}

#[cfg(feature = "llvm")]
impl OutputBackend for LlvmBackend {
    fn write_output(
        &mut self,
        program: &jaic::ir::Program,
        settings: &BuildSettings,
        output: &Path,
    ) -> Result<(), String> {
        let target = self.target.as_deref();
        // Windows wants `.exe`/`.dll`/`.lib`; a name without an extension gets the platform's.
        use jaic_llvm::OutputKind;
        let kind = match settings.output_type {
            OutputType::Executable => Some(OutputKind::Executable),
            OutputType::DynamicLibrary => Some(OutputKind::DynamicLibrary),
            OutputType::StaticLibrary => Some(OutputKind::StaticLibrary),
            OutputType::ObjectFile | OutputType::NoOutput => None,
        };
        let mut output = output.to_path_buf();
        if output.extension().is_none()
            && let Some(ext) = kind.and_then(|k| jaic_llvm::output_extension(target, k))
        {
            output.set_extension(ext);
        }
        let output = output.as_path();
        if let Some(dir) = output.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir)
                .map_err(|e| format!("could not create {}: {e}", dir.display()))?;
        }
        let with_ext = |ext: &str| {
            let mut name = output.to_path_buf().into_os_string();
            name.push(ext);
            PathBuf::from(name)
        };
        let object = if settings.output_type == OutputType::ObjectFile {
            output.to_path_buf()
        } else {
            with_ext(".o")
        };
        use jaic_llvm::OptLevel;
        let opt_level = match settings.optimization.as_str() {
            // `llvm_options.bitcode_optimization_setting` member names.
            "O1" => OptLevel::O1,
            "O2" | "OS" | "OZ" => OptLevel::O2,
            "O3" => OptLevel::O3,
            _ => OptLevel::O0,
        };
        // `Build_Options.emit_debug_info = .NONE` (or `set_optimization(..., false)`) turns it off.
        let debug_info = self.debug_info && settings.emit_debug_info != Some(false);
        let options = jaic_llvm::Options {
            opt_level,
            target: self.target.clone(),
            emit_ir: self.emit_ir.clone(),
            debug_info,
        };
        if matches!(
            settings.output_type,
            OutputType::ObjectFile | OutputType::NoOutput
        ) {
            return timings::time("codegen", || {
                jaic_llvm::emit_object(program, &options, &object)
            });
        }
        let objects = timings::time("codegen", || {
            jaic_llvm::emit_objects(program, &options, &object)
        })?;
        let libraries = jaic_llvm::used_libraries(program);
        let linked = timings::time("link", || match settings.output_type {
            OutputType::ObjectFile | OutputType::NoOutput => unreachable!(),
            OutputType::Executable | OutputType::DynamicLibrary => jaic_llvm::link(
                &objects,
                &libraries,
                output,
                settings.output_type == OutputType::DynamicLibrary,
                &settings.additional_linker_arguments,
                target,
            ),
            OutputType::StaticLibrary => jaic_llvm::archive(&objects, output, target),
        });
        // macOS linkers leave DWARF in the objects; collect it into `output.dSYM` before they go.
        if debug_info
            && linked.is_ok()
            && target.map_or(cfg!(target_os = "macos"), |t| t.contains("apple"))
            && settings.output_type != OutputType::StaticLibrary
            && let Err(message) = timings::time("debug info", || jaic_llvm::write_dsym(output))
        {
            eprintln!("warning: {message}");
        }
        for object in &objects {
            let _ = std::fs::remove_file(object);
        }
        linked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triple(args: &[&str], host_os: TargetOs, host_cpu: TargetCpu) -> Option<String> {
        let mut argv = vec!["build".to_string(), "main.jai".to_string()];
        argv.extend(args.iter().map(|a| a.to_string()));
        let cli = parse(&argv).expect("valid command line");
        cli.target_triple_from(host_os, host_cpu).expect("a target")
    }

    #[test]
    fn windows_cross_builds_default_to_x64_mingw() {
        let mac = (TargetOs::MacOS, TargetCpu::Arm64);
        assert_eq!(
            triple(&["-os", "windows"], mac.0, mac.1).as_deref(),
            Some("x86_64-pc-windows-gnu")
        );
        assert_eq!(
            triple(&["-os", "windows", "-cpu", "arm64"], mac.0, mac.1).as_deref(),
            Some("aarch64-pc-windows-gnu")
        );
        assert_eq!(triple(&[], mac.0, mac.1), None);
    }

    #[test]
    fn windows_hosts_build_for_themselves_or_the_other_cpu_with_msvc() {
        let arm = (TargetOs::Windows, TargetCpu::Arm64);
        assert_eq!(triple(&["-os", "windows"], arm.0, arm.1), None);
        assert_eq!(
            triple(&["-cpu", "x64"], arm.0, arm.1).as_deref(),
            Some("x86_64-pc-windows-msvc")
        );
        let x64 = (TargetOs::Windows, TargetCpu::X64);
        assert_eq!(
            triple(&["-cpu", "arm64"], x64.0, x64.1).as_deref(),
            Some("aarch64-pc-windows-msvc")
        );
        assert_eq!(
            triple(&["-target", "aarch64-w64-mingw32"], x64.0, x64.1).as_deref(),
            Some("aarch64-w64-mingw32")
        );
        assert!(matches!(
            os_and_cpu("aarch64-pc-windows-msvc"),
            (TargetOs::Windows, TargetCpu::Arm64)
        ));
    }
}
