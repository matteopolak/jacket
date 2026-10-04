//! `jaic` command line: `jaic <run|check|build> <file.jai> [-I dir]... [-o out]`.
use jaic::build::{BuildEnv, BuildSettings, OutputBackend, OutputType, Workspaces};
use jaic::interp::NativeHost;
use jaic::sema::{Compiler, FileSystem, NativeFs, Options};
use jaic_llvm::OptLevel;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::rc::Rc;

fn stdlib_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("JAIC_STDLIB") {
        return PathBuf::from(dir);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib")
}

fn usage() -> ExitCode {
    eprintln!("usage: jaic <run|check> <file.jai> [-I|-import_dir dir]... [- metaprogram args...]");
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
    /// `None`: what the metaprogram chose (default: unoptimized).
    opt_level: Option<OptLevel>,
    emit_ir: Option<PathBuf>,
    /// Arguments after `-`, for the metaprogram (`compiler_get_command_line`).
    command_line: Vec<String>,
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
    };
    let mut rest = args[2..].iter();
    while let Some(a) = rest.next() {
        match a.as_str() {
            "-" => {
                cli.command_line.extend(rest.by_ref().cloned());
            }
            "-I" | "-import_dir" => cli.imports.extend(rest.next().map(PathBuf::from)),
            "-o" if command == Command::Build => cli.output = Some(PathBuf::from(rest.next()?)),
            "--emit-ir" if command == Command::Build => {
                cli.emit_ir = Some(PathBuf::from(rest.next()?))
            }
            "-O0" if command == Command::Build => cli.opt_level = Some(OptLevel::O0),
            "-O1" if command == Command::Build => cli.opt_level = Some(OptLevel::O1),
            "-O2" if command == Command::Build => cli.opt_level = Some(OptLevel::O2),
            "-O3" if command == Command::Build => cli.opt_level = Some(OptLevel::O3),
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
    let worker = std::thread::Builder::new()
        .stack_size(1 << 30)
        .spawn(move || run(cli));
    match worker.map(|h| h.join()) {
        Ok(Ok(code)) => code,
        _ => ExitCode::from(101),
    }
}

fn run(mut cli: Cli) -> ExitCode {
    let stdlib = stdlib_dir();
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
    // The local `modules` folder is searched first, then `-import_dir`s, then the stdlib.
    options.import_paths = vec![main_dir.join("modules")];
    options.import_paths.extend(cli.imports.iter().cloned());
    options.import_paths.push(stdlib.clone());
    options.preload = Some(stdlib.join("prelude/module.jai"));
    let fs: Rc<dyn FileSystem> = Rc::new(NativeFs);
    // Workspaces created by metaprograms are written only by `build`.
    let backend: Option<Box<dyn OutputBackend>> = (cli.command == Command::Build).then(|| {
        Box::new(LlvmBackend {
            emit_ir: cli.emit_ir.clone(),
        }) as Box<dyn OutputBackend>
    });
    let workspaces = Workspaces::new(BuildEnv {
        fs: fs.clone(),
        options: options.clone(),
        backend,
        command_line: cli.command_line.clone(),
        make_host: Box::new(|| Box::new(NativeHost)),
        report: Box::new(|text| eprintln!("{text}")),
    });
    let mut compiler = Compiler::new(options, fs);
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
        Command::Run => match compiler.run_program() {
            Ok(code) => ExitCode::from(code as u8),
            Err(d) => {
                eprintln!("{}", compiler.render(&d));
                ExitCode::from(1)
            }
        },
        // A metaprogram that turned its own output off has nothing to write.
        Command::Build if !settings.do_output || settings.output_type == OutputType::NoOutput => {
            ExitCode::SUCCESS
        }
        Command::Build => match build(&compiler, &cli, &path, settings) {
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
    compiler: &Compiler,
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
        settings.optimization = match level {
            OptLevel::O0 => "O0",
            OptLevel::O1 => "O1",
            OptLevel::O2 => "O2",
            OptLevel::O3 => "O3",
        }
        .into();
    }
    LlvmBackend {
        emit_ir: cli.emit_ir.clone(),
    }
    .write_output(&compiler.program, &settings, &output)
}

/// Native output through `jaic-llvm` and the system linker.
struct LlvmBackend {
    emit_ir: Option<PathBuf>,
}

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
