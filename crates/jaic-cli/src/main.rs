//! `jaic` command line: `jaic <run|check|build> <file.jai> [-I dir]... [-o out]`.
use jaic::build::{BuildEnv, BuildSettings, OutputBackend, OutputType, Workspaces};
use jaic::interp::{NativeHost, SandboxHost, SharedHost};
use jaic::sema::{Compiler, FileSystem, NativeFs, Options, TargetOs};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::rc::Rc;

fn stdlib_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("JAIC_STDLIB") {
        return PathBuf::from(dir);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib")
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
        "usage: jaic <run|check> <file.jai> [-I|-import_dir dir]... [-os linux|windows|macos|wasm] [- metaprogram args...]"
    );
    eprintln!(
        "       jaic build <file.jai> [-I dir]... [-o output] [-O0|-O1|-O2|-O3] [--emit-ir file.ll]"
    );
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
    /// Arguments after `-`, for the metaprogram (`compiler_get_command_line`).
    command_line: Vec<String>,
    /// `-os`: the target `OS` when it is not the host (checking code for another platform).
    os: Option<TargetOs>,
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
        command_line: Vec::new(),
        os: None,
    };
    let mut rest = args[2..].iter();
    while let Some(a) = rest.next() {
        match a.as_str() {
            "-" => {
                cli.command_line.extend(rest.by_ref().cloned());
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
            "-o" if command == Command::Build => cli.output = Some(PathBuf::from(rest.next()?)),
            "--emit-ir" if command == Command::Build => {
                cli.emit_ir = Some(PathBuf::from(rest.next()?))
            }
            "-O0" if command == Command::Build => cli.opt_level = Some("O0"),
            "-O1" if command == Command::Build => cli.opt_level = Some("O1"),
            "-O2" if command == Command::Build => cli.opt_level = Some("O2"),
            "-O3" if command == Command::Build => cli.opt_level = Some("O3"),
            _ => return None,
        }
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

fn run(mut cli: Cli) -> ExitCode {
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
    if let Err(d) = compiler.compile_program(&path) {
        eprintln!("{}", compiler.render(&d));
        return ExitCode::from(1);
    }
    if let Err(message) = jaic::build::finish_all(&workspaces) {
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
            let outcome = compiler.run_program();
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
    compiler.prepare_compiled_output();
    native_backend(cli).write_output(&compiler.program, &settings, &output)
}

#[cfg(feature = "llvm")]
fn native_backend(cli: &Cli) -> LlvmBackend {
    LlvmBackend {
        emit_ir: cli.emit_ir.clone(),
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
}

#[cfg(feature = "llvm")]
impl OutputBackend for LlvmBackend {
    fn write_output(
        &mut self,
        program: &jaic::ir::Program,
        settings: &BuildSettings,
        output: &Path,
    ) -> Result<(), String> {
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
        let options = jaic_llvm::Options {
            opt_level,
            target: None,
            emit_ir: self.emit_ir.clone(),
        };
        jaic_llvm::emit_object(program, &options, &object)?;
        let libraries = jaic_llvm::used_libraries(program);
        let objects = std::slice::from_ref(&object);
        let linked = match settings.output_type {
            OutputType::ObjectFile | OutputType::NoOutput => return Ok(()),
            OutputType::Executable | OutputType::DynamicLibrary => jaic_llvm::link(
                objects,
                &libraries,
                output,
                settings.output_type == OutputType::DynamicLibrary,
                &settings.additional_linker_arguments,
            ),
            OutputType::StaticLibrary => std::process::Command::new("ar")
                .arg("rcs")
                .arg(output)
                .arg(&object)
                .status()
                .map_err(|e| format!("could not run 'ar': {e}"))
                .and_then(|s| s.success().then_some(()).ok_or(format!("ar failed ({s})"))),
        };
        let _ = std::fs::remove_file(&object);
        linked
    }
}
