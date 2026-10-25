//! Running a request: the compile, interpret and write steps.
use super::backend::{build, native_backend};
use super::cli::{Cli, Flavor, Mode, native_lib_dirs};
use super::leak::{LeakOnExit, flush_sandbox};
use super::options::NO_MAIN_STATUS;
use super::parse::{check_input, print_cli_error, print_error, shown};
use super::{timings, tuning};
use jaic::build::{BuildEnv, BuildSettings, OutputBackend, OutputType, Workspaces};
use jaic::builtin_keys::BitcodeOptimization;
use jaic::interp::{NativeHost, SandboxHost, SharedHost};
use jaic::sema::{Compiler, DeadCode, FileSystem, NativeFs, Options, ProgramSource, TargetOs};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::rc::Rc;

pub(super) fn run(cli: Cli) -> ExitCode {
    jaic::knobs::set(tuning::knobs(&cli));
    if cli.timings {
        timings::enable();
    }
    start_progress(&cli);
    let code = timings::time("total", || compile_and_run(cli));
    // A failed compile ends without a `Finished` line; a finished one already printed it.
    jaic::progress::abandon();
    timings::report();
    // Interpreters flush their `-profile` counts when dropped, which has happened by now.
    if let Some(report) = jaic::interp::profile::report(40) {
        eprint!("{report}");
    }
    jaic::interp::profile::write_coverage();
    code
}

fn compile_and_run(mut cli: Cli) -> ExitCode {
    let stdlib = jaic::default_stdlib_dir();
    if let Some(message) = jaic::missing_stdlib(&stdlib) {
        print_error(&message);
        return ExitCode::from(1);
    }
    jaic::interp::set_library_dirs(native_lib_dirs(&stdlib));
    if let Err(error) = check_input(&cli.file) {
        // An input that cannot be read is a failure of the command (1), not a mistake in
        // its arguments (2); see docs/compiler/diagnostics.md.
        print_cli_error(error);
        return ExitCode::from(1);
    }
    let (path, main_dir) = match enter_main_directory(&mut cli) {
        Ok(located) => located,
        Err(code) => return code,
    };
    let options = match compiler_options(&cli, &stdlib, &main_dir) {
        Ok(options) => options,
        Err(code) => return code,
    };
    let fs: Rc<dyn FileSystem> = Rc::new(NativeFs);
    if cli.no_check_bindings {
        disable_check_bindings(&mut cli, &options, &stdlib);
    }
    // `OS == .WASM` code is written for the interpreter's sandbox host (virtual clock and files,
    // cooperative threads), so compile-time code of a wasm target runs there whatever the
    // command. `jaic -interp -os wasm` runs the program there too, the way the browser does, with
    // its output printed when the run ends; a build for wasm compiles it to a wasm module.
    let cwd = std::env::current_dir().unwrap_or_default();
    let shared_sandbox = Rc::new(RefCell::new(SandboxHost::with_files(
        fs.clone(),
        &cwd.to_string_lossy(),
    )));
    let sandbox = (options.os == TargetOs::Wasm).then(|| shared_sandbox.clone());
    let workspaces = create_workspaces(&cli, &options, &fs, &shared_sandbox);
    // jaic exits right after: tearing down the syntax trees and the program takes about a tenth
    // of a large check, and nothing observes it. After `-interp` the program's threads are left
    // to the exit too, as a native program's are; joining one blocked in C would never return.
    let mut compiler = LeakOnExit::new(Compiler::new(options, fs), |c| {
        c.ct.interp.stop_for_exit();
    });
    if let Some(host) = &sandbox {
        compiler.ct.interp.host = Box::new(SharedHost(host.clone()));
    }
    compiler.attach_workspaces(workspaces.clone());
    // `-os wasm` / `-target wasm64-unknown-wasi`: a WASI command, with its runtime added.
    let wasi = cli.mode == Mode::Build
        && jaic::build::wants_wasi_runtime(cli.build_target().ok().flatten());
    if cli.verbose {
        announce(&cli, &path);
    }
    if !compile_front_end(&mut compiler, &cli, &path, wasi, &shared_sandbox) {
        return ExitCode::from(1);
    }
    let finished = timings::time("workspaces", || {
        jaic::build::finish_all(&workspaces, &mut compiler.ct.interp.block_budget)
    });
    if cli.mode == Mode::Build {
        flush_sandbox(&shared_sandbox);
    }
    if let Err(message) = finished {
        print_error(&message);
        return ExitCode::from(1);
    }
    if workspaces.borrow().any_failed() {
        return ExitCode::from(1);
    }
    if cli.verbose {
        let created = workspaces.borrow().created_workspaces();
        if created > 0 {
            eprintln!("jaic: the metaprogram built {created} workspace(s)");
        }
    }
    let settings = workspaces.borrow().top_level_settings();
    if let Err(diagnostic) = compiler.finalize_runtime_configuration() {
        eprintln!("{}", compiler.render(&diagnostic));
        return ExitCode::from(1);
    }
    match cli.mode {
        Mode::Check => {
            jaic::progress::finish("check");
            ExitCode::SUCCESS
        }
        Mode::Interp => {
            jaic::progress::finish("interp");
            jaic::progress::announce("Running", &shown(&path));
            interpret(&mut compiler, &cli, &path, &main_dir, sandbox.as_ref())
        }
        Mode::Build => {
            let optimized = !matches!(
                cli.opt_level
                    .or(cli.flavor.map(Flavor::level))
                    .unwrap_or(settings.optimization),
                BitcodeOptimization::Unset | BitcodeOptimization::O0
            );
            jaic::progress::set_optimized(optimized);
            let code = write_program(&mut compiler, &cli, &path, settings);
            if code == ExitCode::SUCCESS {
                jaic::progress::finish(if optimized {
                    "optimized"
                } else {
                    "debug [unoptimized]"
                });
            }
            code
        }
    }
}

/// The build status (`jaic::progress`): `   Compiling hello (hello.jai)` now, a status line on
/// a terminal while the compile runs, and `    Finished ...` at the end. The name is the main
/// file's, or the project's when jacket passes one in `JACKET_PROJECT`.
fn start_progress(cli: &Cli) {
    let mode = jaic::progress::mode(cli.quiet, cli.no_color || cli.msvc_format);
    if mode == jaic::progress::Mode::Off {
        return;
    }
    let path = Path::new(&cli.file);
    let name = std::env::var("JACKET_PROJECT")
        .ok()
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| {
            path.file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        });
    let verb = if cli.mode == Mode::Check {
        "Checking"
    } else {
        "Compiling"
    };
    // jacket passes the entry file as an absolute path; show it as the user would type it.
    let shown = if path.is_absolute() {
        jaic::set_display_base(std::env::current_dir().unwrap_or_default());
        jaic::display_path(path)
    } else {
        cli.file.clone()
    };
    jaic::progress::start(mode, verb, &name, &shown);
}

/// Resolve the main file and move into its directory. Like `jai`, the program runs from the
/// main file's directory (so its meaning does not depend on where the compiler was started);
/// paths given on the command line stay relative to the original directory, so they are made
/// absolute first. Returns the main file and its directory.
fn enter_main_directory(cli: &mut Cli) -> Result<(PathBuf, PathBuf), ExitCode> {
    let path = jaic::canonicalize(&cli.file).unwrap_or_else(|_| PathBuf::from(&cli.file));
    let absolute = |p: &PathBuf| std::path::absolute(p).unwrap_or_else(|_| p.clone());
    cli.imports = cli.imports.iter().map(absolute).collect();
    cli.output_path = cli.output_path.as_ref().map(absolute);
    // `-exe` with a directory is taken from where jaic was started, like the paths above, and
    // splits into the output path and the executable's name; a bare name goes into the
    // directory jaic was started in (or into `-output_path`).
    if let Some(exe) = cli.exe.clone()
        && cli.output_path.is_none()
    {
        let full = absolute(&PathBuf::from(&exe));
        if let (Some(dir), Some(name)) = (full.parent(), full.file_name()) {
            cli.output_path = Some(dir.to_path_buf());
            cli.exe = Some(name.to_string_lossy().into_owned());
        }
    }
    cli.emit_ir = cli.emit_ir.as_ref().map(absolute);
    let main_dir = path.parent().map(PathBuf::from).unwrap_or_default();
    let started_in = std::env::current_dir().unwrap_or_default();
    jaic::set_display_base(started_in);
    if !main_dir.as_os_str().is_empty() && std::env::set_current_dir(&main_dir).is_err() {
        eprintln!("error: cannot change directory to {}", main_dir.display());
        return Err(ExitCode::from(1));
    }
    Ok((path, main_dir))
}

/// The compiler options the command line asks for: checks, target and where imports are found.
fn compiler_options(cli: &Cli, stdlib: &Path, main_dir: &Path) -> Result<Options, ExitCode> {
    let mut options = Options::host();
    if cli.no_dce {
        options.dead_code = DeadCode::None;
    }
    // Only native output has a use for variable and type descriptions.
    options.debug_info = cli.mode == Mode::Build && !cli.no_debug_info;
    if cli.no_backtrace_on_crash {
        options.backtrace_on_crash = false;
    }
    if cli.flavor == Some(Flavor::Optimized) {
        // What `set_optimization(*o, .OPTIMIZED)` does besides the LLVM levels, and no stack trace.
        options.stack_trace = false;
        options.array_bounds_check = false;
        options.null_pointer_check = false;
        options.arithmetic_overflow_check = 0;
        options.cast_bounds_check = 0;
    }
    match cli.build_target() {
        // The interpreter models wasm32 (the browser engine's target); LLVM output cannot.
        Ok(Some(target)) if target.is_wasm() && target.memory32 && cli.mode == Mode::Build => {
            eprintln!(
                "error: jaic builds wasm64 (Memory64) only; Jai needs 8-byte pointers (use -target wasm64-unknown-wasi)"
            );
            return Err(ExitCode::from(2));
        }
        Ok(Some(target)) => target.apply(&mut options),
        // `-os`/`-cpu` for checking only.
        Ok(None) => jaic::target::select_os_cpu(&mut options, cli.os, cli.cpu),
        Err(message) => {
            print_error(&message);
            return Err(ExitCode::from(2));
        }
    }
    // The local `modules` folder is searched first, then `-import_dir`s, then the stdlib.
    options.import_paths = vec![main_dir.join("modules")];
    options.import_paths.extend(cli.imports.iter().cloned());
    options.import_paths.push(stdlib.to_path_buf());
    options.set_stdlib(&NativeFs, stdlib);
    Ok(options)
}

/// The registry of workspaces a metaprogram creates.
fn create_workspaces(
    cli: &Cli,
    options: &Options,
    fs: &Rc<dyn FileSystem>,
    shared_sandbox: &Rc<RefCell<SandboxHost>>,
) -> jaic::build::SharedWorkspaces {
    // What workspaces created by metaprograms ask for is written by a build and `-interp` alike:
    // `-interp` differs only in interpreting the top-level program instead of compiling it.
    // `-check` only checks.
    let writes_workspaces = match cli.mode {
        Mode::Build => true,
        Mode::Interp => !cli.no_workspace_output,
        Mode::Check => false,
    };
    let backend: Option<Box<dyn OutputBackend>> =
        writes_workspaces.then(|| Box::new(native_backend(cli)) as Box<dyn OutputBackend>);
    let workspace_sandbox = shared_sandbox.clone();
    let quiet = cli.quiet;
    Workspaces::new(BuildEnv {
        fs: fs.clone(),
        options: options.clone(),
        backend,
        unwritten_output_hint: (cli.mode == Mode::Check && !cli.quiet).then(|| {
            jaic::build::UnwrittenOutputHint {
                main_file: cli.file.clone(),
            }
        }),
        command_line: cli.command_line.clone(),
        make_host: Box::new(move |os| {
            if os == TargetOs::Wasm {
                Box::new(SharedHost(workspace_sandbox.clone()))
            } else {
                Box::new(NativeHost)
            }
        }),
        report: Box::new(move |severity, text| {
            if !(quiet && severity.is_chatter()) {
                eprintln!("{text}");
            }
        }),
        observer: None,
    })
}

/// `-no_check_bindings`: sets `CHECK_BINDINGS=false` on the plugins that resolve, on the import
/// path, to the standard library's `Check` module (not on any module that merely has the name).
pub(super) fn disable_check_bindings(cli: &mut Cli, options: &jaic::sema::Options, stdlib: &Path) {
    let check = jaic::canonicalize(stdlib.join("Check").join("module.jai")).ok();
    for plugin in &mut cli.plugins {
        let entry = jaic::sema::find_module_in(
            &NativeFs,
            &options.import_paths,
            &plugin.module,
            Path::new(""),
        )
        .and_then(|entry| jaic::canonicalize(&entry).ok());
        if entry.is_some() && entry == check {
            plugin.default_parameter("CHECK_BINDINGS", "false");
        }
    }
}

/// `-verbose`: what is about to be done.
fn announce(cli: &Cli, path: &Path) {
    let what = match cli.mode {
        Mode::Build => "building",
        Mode::Check => "checking",
        Mode::Interp => "interpreting",
    };
    eprintln!("jaic: {what} {}", shown(path));
    if !cli.plugins.is_empty() {
        let names: Vec<&str> = cli.plugins.iter().map(|p| p.module.as_str()).collect();
        eprintln!("jaic: plugins: {}", names.join(", "));
    }
}

/// Parse and check the program (or the plugin metaprogram that compiles it), printing warnings
/// and the error that stops it. Returns whether it compiled.
fn compile_front_end(
    compiler: &mut Compiler,
    cli: &Cli,
    path: &Path,
    wasi: bool,
    shared_sandbox: &Rc<RefCell<SandboxHost>>,
) -> bool {
    let compiled = if cli.plugins.is_empty() {
        let mut sources = vec![ProgramSource::File(path.to_path_buf())];
        sources.extend(cli.added.iter().map(|a| ProgramSource::String(a.source())));
        if wasi {
            sources.push(ProgramSource::String(
                jaic::build::WASI_RUNTIME_IMPORT.into(),
            ));
        }
        timings::time("front end", || compiler.compile_sources(&sources))
    } else {
        let source = cli.plugin_metaprogram(path, wasi);
        timings::time("front end", || {
            compiler.compile_sources(&[ProgramSource::String(source)])
        })
    };
    if cli.mode == Mode::Build {
        // What compile-time code of a wasm target printed (it runs in the sandbox).
        flush_sandbox(shared_sandbox);
    }
    if !cli.quiet {
        for warning in &compiler.diag.warnings {
            eprintln!("{}", compiler.render(warning));
        }
    }
    if let Err(d) = compiled {
        eprintln!("{}", compiler.render(&d));
        return false;
    }
    true
}

/// `-interp`: run the program in the interpreter and return its exit status.
fn interpret(
    compiler: &mut Compiler,
    cli: &Cli,
    path: &Path,
    main_dir: &Path,
    sandbox: Option<&Rc<RefCell<SandboxHost>>>,
) -> ExitCode {
    // The program sees itself as the executable a build would write next to its
    // main file (not `jaic`), so data found relative to the executable is found.
    if sandbox.is_none() {
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let name = if cfg!(windows) {
            format!("{stem}.exe")
        } else {
            stem.into_owned()
        };
        compiler.ct.interp.run_executable = Some(main_dir.join(name).display().to_string());
    }
    let outcome = timings::time("run", || compiler.run_program_with_args(&cli.program_args));
    if let Some(host) = sandbox {
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

/// A build: write the top-level program unless the metaprogram turned its output off.
fn write_program(
    compiler: &mut Compiler,
    cli: &Cli,
    path: &Path,
    settings: BuildSettings,
) -> ExitCode {
    // A metaprogram that turned its own output off has nothing to write.
    if !settings.do_output || settings.output_type == OutputType::NoOutput {
        return ExitCode::SUCCESS;
    }
    if settings.output_type == OutputType::Executable && compiler.exported_func("main").is_none() {
        let file = shown(path);
        print_error(&format!(
            "`{file}` has no `main` procedure, so there is no program to write\n\
             help: add `main :: () {{ ... }}` as the program's starting point, or use `jaic {file} -check` to only check the code"
        ));
        return ExitCode::from(NO_MAIN_STATUS);
    }
    match build(compiler, cli, path, settings) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            print_error(&message);
            ExitCode::from(1)
        }
    }
}
