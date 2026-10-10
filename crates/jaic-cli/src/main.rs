//! `jaic` command line: `jaic <run|check|build> <file.jai> [-I dir]... [-o out]`.
use jaic::build::{BuildEnv, BuildSettings, OutputBackend, OutputType, Workspaces};
use jaic::interp::{NativeHost, SandboxHost, SharedHost};
use jaic::sema::{
    Compiler, DeadCode, FileSystem, NativeFs, Options, ProgramSource, TargetCpu, TargetOs,
};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::rc::Rc;

/// Every Rust allocation goes through the counter that enforces `JAIC_MEMORY_LIMIT`; unarmed it
/// costs one relaxed load per call.
#[global_allocator]
static ALLOCATOR: jaic::memory_limit::CountingAllocator = jaic::memory_limit::CountingAllocator;

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
        "windows" => "windows",
        _ => return Vec::new(),
    };
    let arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        "x86_64" => "x64",
        _ => return Vec::new(),
    };
    let dir = stdlib.join(format!("../artifacts/native-libs/{os}-{arch}"));
    jaic::canonicalize(&dir)
        .map(|d| vec![d])
        .unwrap_or_default()
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
    /// `-sanitize address,undefined` (repeatable): sanitizer instrumentation for `build`.
    sanitize: Vec<String>,
    /// `-no_dce`: type-check every declaration, modules included
    /// (`Build_Options.dead_code_elimination = .NONE`).
    no_dce: bool,
    /// `-no_workspace_output` (`run`): do not write what workspaces a metaprogram creates ask
    /// for (test sweeps over many programs).
    no_workspace_output: bool,
    /// `--color`: colour in diagnostics.
    color: jaic::render::ColorChoice,
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
            // A WASI command, runnable by node, wasmtime and the like.
            TargetOs::Wasm => Ok(Some(WASI_TRIPLE.into())),
            _ => Err(
                "native cross-compilation is only supported for -os windows and -os wasm; pass -target <triple> for others"
                    .into(),
            ),
        }
    }
}

/// What `jaic build -os wasm` targets: wasm64 with the `Wasi_Runtime` module linked in.
const WASI_TRIPLE: &str = "wasm64-unknown-wasi";

/// `jaic build`'s exit status when the program has no `main` (its work is all `#run`s), so
/// tools can tell "nothing to build" from a failed compile.
const NO_MAIN_STATUS: u8 = 3;

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

/// `jaic --help`.
fn usage_text() -> String {
    format!(
        "jaic {version}: compile, check and run Jai programs

usage: jaic run <file.jai> [options] [- metaprogram args...] [-- program args...]
       jaic check <file.jai> [options]
       jaic build <file.jai> [options] [-o output]

commands:
  run      compile the program and run it in the interpreter
  check    compile the program without running or writing anything
  build    compile the program to a native executable (or what its metaprogram asks for)

options:
  -I, -import_dir <dir>     also look for modules in <dir> (repeatable)
  -os <os>                  target OS: linux, windows, macos or wasm
  -cpu <cpu>                target CPU: x64 or arm64
  -target <triple>          build for an LLVM target triple
  -plug <Module>            run a metaprogram plugin (check and build; repeatable)
  -no_dce                   type-check unreferenced module code too
  --color <when>            coloured diagnostics: auto, always or never
  --timings                 print the wall time of each phase on stderr
  -h, --help                print this help
  -V, --version             print the version

build options:
  -o <output>               the output file
  -O0, -O1, -O2, -O3        optimization level
  --emit-ir <file.ll>       also write the LLVM IR
  --no-debug-info           leave out native debug information
  -sanitize <list>          address and/or undefined, comma-separated

run options:
  -no_workspace_output      do not write what a metaprogram's workspaces ask for

environment:
  JAIC_MEMORY_LIMIT=<bytes|nK|nM|nG>  stop with exit status {limit} once that much is allocated
  JAIC_DIAGNOSTICS=plain|ascii|unicode  diagnostic layout (default: by terminal)
  NO_COLOR, FORCE_COLOR, CLICOLOR_FORCE  turn colour off or on

exit status: 0 success, 1 the program failed to compile, an input file cannot be read
or a runtime error stopped it (`run` otherwise exits with the program's own status),
2 a command-line mistake, {no_main} `build` found no `main` to make an executable from,
{limit} the memory limit, {crash} native code crashed under the interpreter, 101 an
internal compiler error",
        version = env!("CARGO_PKG_VERSION"),
        no_main = NO_MAIN_STATUS,
        limit = jaic::memory_limit::EXIT_CODE,
        crash = jaic::interp::CRASH_STATUS,
    )
}

/// A command-line mistake: the message, then help lines.
struct CliError {
    message: String,
    help: Vec<String>,
}

impl CliError {
    fn new(message: impl Into<String>) -> Self {
        CliError {
            message: message.into(),
            help: Vec::new(),
        }
    }

    fn help(mut self, help: impl Into<String>) -> Self {
        self.help.push(help.into());
        self
    }
}

/// What the command line asks for.
enum Request {
    Compile(Box<Cli>),
    Help,
    Version,
}

/// Options that take a value, with what the value is (for "needs a value" errors).
const VALUE_OPTIONS: &[(&str, &str)] = &[
    ("-I", "a directory"),
    ("-import_dir", "a directory"),
    ("-os", "an OS name: linux, windows, macos or wasm"),
    ("-cpu", "a CPU name: x64 or arm64"),
    (
        "-target",
        "an LLVM target triple, such as x86_64-pc-windows-gnu",
    ),
    (
        "--target",
        "an LLVM target triple, such as x86_64-pc-windows-gnu",
    ),
    ("-o", "an output path"),
    ("--emit-ir", "a file to write the LLVM IR to"),
    ("-sanitize", "address, undefined or both, comma-separated"),
    ("--sanitize", "address, undefined or both, comma-separated"),
    ("-plug", "a plugin module name"),
    ("-plugin", "a plugin module name"),
    ("--color", "a value: auto, always or never"),
    ("-color", "a value: auto, always or never"),
];

/// Every option jaic knows, for "did you mean" suggestions.
const KNOWN_OPTIONS: &[&str] = &[
    "-I",
    "-import_dir",
    "-os",
    "-cpu",
    "-target",
    "--target",
    "-o",
    "--emit-ir",
    "--no-debug-info",
    "-sanitize",
    "--sanitize",
    "-O0",
    "-O1",
    "-O2",
    "-O3",
    "-plug",
    "-plugin",
    "--timings",
    "-no_dce",
    "-no_workspace_output",
    "--color",
    "--help",
    "--version",
];

/// Options only `jaic build` takes.
const BUILD_ONLY: &[&str] = &[
    "-o",
    "--emit-ir",
    "--no-debug-info",
    "-sanitize",
    "--sanitize",
    "-O0",
    "-O1",
    "-O2",
    "-O3",
];

fn parse(args: &[String]) -> Result<Request, CliError> {
    let Some(first) = args.first() else {
        return Err(CliError::new("no command given")
            .help("run a program with `jaic run file.jai`; `jaic --help` lists every command"));
    };
    let command = match first.as_str() {
        "check" => Command::Check,
        "run" => Command::Run,
        "build" => Command::Build,
        "-h" | "--help" | "-help" | "help" => return Ok(Request::Help),
        "-V" | "--version" | "-version" | "version" => return Ok(Request::Version),
        other if other.ends_with(".jai") => {
            return Err(CliError::new(format!("`{other}` is not a command"))
                .help(format!("to run it, use `jaic run {other}`"))
                .help("`jaic check` only compiles it, `jaic build` writes an executable"));
        }
        other => {
            let mut error = CliError::new(format!("unknown command `{other}`"));
            if let Some(near) = jaic::suggest::closest(other, ["run", "check", "build", "help"]) {
                error = error.help(format!("did you mean `jaic {near}`?"));
            }
            return Err(error.help("the commands are `run`, `check` and `build`"));
        }
    };
    let name = first.as_str();
    let file = match args.get(1) {
        Some(file) if file == "-h" || file == "--help" => return Ok(Request::Help),
        Some(file) if file.starts_with('-') && file.len() > 1 => {
            return Err(CliError::new(format!(
                "expected a .jai file after `jaic {name}`, found `{file}`"
            ))
            .help(format!(
                "put the file first and options after it: `jaic {name} file.jai {file} ...`"
            )));
        }
        Some(file) => file.clone(),
        None => {
            return Err(CliError::new(format!("`jaic {name}` needs a .jai file"))
                .help(format!("for example: `jaic {name} main.jai`")));
        }
    };
    let mut cli = Cli {
        command,
        file,
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
        sanitize: Vec::new(),
        no_dce: false,
        no_workspace_output: false,
        color: jaic::render::ColorChoice::Auto,
    };
    // Only jaic's own options count: metaprogram and program arguments start at `-`/`--`.
    let has_plugins = args
        .iter()
        .take_while(|a| *a != "-" && *a != "--")
        .any(|a| a == "-plug" || a == "-plugin");
    let mut rest = args[2..].iter();
    while let Some(arg) = rest.next() {
        // `--color=always` and the like.
        let (a, inline) = match arg.split_once('=') {
            Some((option, value))
                if option.starts_with('-') && VALUE_OPTIONS.iter().any(|(o, _)| *o == option) =>
            {
                (option, Some(value.to_string()))
            }
            _ => (arg.as_str(), None),
        };
        let mut value = |option: &str| -> Result<String, CliError> {
            if let Some(v) = inline.clone() {
                return Ok(v);
            }
            match rest.next() {
                Some(v) => Ok(v.clone()),
                None => {
                    let what = VALUE_OPTIONS
                        .iter()
                        .find(|(o, _)| *o == option)
                        .map_or("a value", |(_, what)| what);
                    Err(CliError::new(format!("`{option}` needs {what}")))
                }
            }
        };
        if BUILD_ONLY.contains(&a) && command != Command::Build && !has_plugins {
            return Err(
                CliError::new(format!("`{a}` only applies to `jaic build`")).help(format!(
                    "`jaic {name}` writes no output; use `jaic build {} {a} ...`",
                    cli.file
                )),
            );
        }
        match a {
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
            "-I" | "-import_dir" => cli.imports.push(PathBuf::from(value(a)?)),
            "-os" => {
                let os = value(a)?;
                cli.os = Some(match os.as_str() {
                    "linux" => TargetOs::Linux,
                    "windows" => TargetOs::Windows,
                    "macos" => TargetOs::MacOS,
                    "wasm" => TargetOs::Wasm,
                    _ => {
                        let names = ["linux", "windows", "macos", "wasm"];
                        let mut error = CliError::new(format!("unknown OS `{os}` for `-os`"));
                        if let Some(near) = jaic::suggest::closest(&os, names) {
                            error = error.help(format!("did you mean `-os {near}`?"));
                        }
                        return Err(error.help("the OS names are linux, windows, macos and wasm"));
                    }
                })
            }
            "-cpu" => {
                let cpu = value(a)?;
                cli.cpu = Some(match cpu.as_str() {
                    "x64" | "x86_64" => TargetCpu::X64,
                    "arm64" | "aarch64" => TargetCpu::Arm64,
                    _ => {
                        return Err(CliError::new(format!("unknown CPU `{cpu}` for `-cpu`"))
                            .help("the CPU names are x64 (or x86_64) and arm64 (or aarch64)"));
                    }
                })
            }
            "-target" | "--target" => cli.target = Some(value(a)?),
            "-h" | "--help" | "-help" if !has_plugins => return Ok(Request::Help),
            "--timings" => cli.timings = true,
            "-no_dce" => cli.no_dce = true,
            "-no_workspace_output" if command == Command::Run => cli.no_workspace_output = true,
            "-no_workspace_output" if !has_plugins => {
                return Err(
                    CliError::new("`-no_workspace_output` only applies to `jaic run`").help(
                        if command == Command::Check {
                            "`jaic check` never writes workspace output"
                        } else {
                            "`jaic build` writes what the workspaces ask for; leave the option out"
                        },
                    ),
                );
            }
            "--color" | "-color" => {
                let when = value("--color")?;
                cli.color = jaic::render::ColorChoice::parse(&when).ok_or_else(|| {
                    CliError::new(format!("unknown `--color` value `{when}`"))
                        .help("use auto, always or never")
                })?;
            }
            "-o" => cli.output = Some(PathBuf::from(value(a)?)),
            "--emit-ir" => cli.emit_ir = Some(PathBuf::from(value(a)?)),
            "--no-debug-info" => cli.no_debug_info = true,
            "-sanitize" | "--sanitize" => {
                let list = value(a)?;
                if let Some(bad) = list
                    .split(',')
                    .map(str::trim)
                    .find(|s| !matches!(*s, "address" | "undefined"))
                {
                    return Err(
                        CliError::new(format!("unknown sanitizer `{bad}` for `{a}`"))
                            .help("use address, undefined or both: `-sanitize address,undefined`"),
                    );
                }
                cli.sanitize.push(list)
            }
            "-O0" => cli.opt_level = Some("O0"),
            "-O1" => cli.opt_level = Some("O1"),
            "-O2" => cli.opt_level = Some("O2"),
            "-O3" => cli.opt_level = Some("O3"),
            "-plug" | "-plugin" => cli.plugins.push(value(a)?),
            other if other.starts_with("-O") && command == Command::Build && !has_plugins => {
                return Err(
                    CliError::new(format!("unknown optimization level `{other}`"))
                        .help("use -O0, -O1, -O2 or -O3"),
                );
            }
            // An unknown option and everything after it (its values) go to the plugins.
            other if other.starts_with('-') || !cli.plugin_options.is_empty() => {
                cli.plugin_options.push(other.to_string())
            }
            other => {
                let mut error = CliError::new(format!("unexpected argument `{other}`"));
                if command == Command::Run {
                    error = error.help(format!(
                        "arguments for the program go after `--`: `jaic run {} -- {other}`",
                        cli.file
                    ));
                } else {
                    error = error
                        .help("jaic compiles one file; it can `#load` or `#import` the others");
                }
                return Err(error);
            }
        }
    }
    // Plugins compile the program in a workspace of their own, which `run` cannot start.
    if !cli.plugins.is_empty() && command == Command::Run {
        return Err(CliError::new(
            "`-plug` works with `jaic check` and `jaic build`, not `jaic run`",
        )
        .help(format!("use `jaic build {} -plug ...`", cli.file)));
    }
    if let Some(unknown) = cli
        .plugin_options
        .first()
        .filter(|_| cli.plugins.is_empty())
    {
        let mut error = CliError::new(format!("unknown option `{unknown}`"));
        if let Some(near) = jaic::suggest::closest(unknown, KNOWN_OPTIONS.iter().copied()) {
            error = error.help(format!("did you mean `{near}`?"));
        }
        return Err(error.help(
            "options jaic does not know are handed to `-plug` plugins, but none were given; `jaic --help` lists the options",
        ));
    }
    Ok(Request::Compile(Box::new(cli)))
}

/// Why `file` cannot be compiled, if it cannot: missing, a directory, unreadable.
fn check_input(file: &str) -> Result<(), CliError> {
    let path = Path::new(file);
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    // The `.jai` files of a directory, as paths joined to `dir` the way the user wrote it.
    let jai_files = |dir: &Path| -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".jai"))
            .collect();
        names.sort();
        names
    };
    match std::fs::metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let mut error = CliError::new(format!("file `{file}` does not exist"));
            if let Some(near) = jaic::suggest::similar_sibling(path, |n| n.ends_with(".jai")) {
                error = error.help(format!("did you mean `{}`?", near.display()));
            } else if !parent.is_dir() {
                error = error.help(format!(
                    "the directory `{}` does not exist either",
                    parent.display()
                ));
            } else if let Ok(cwd) = std::env::current_dir()
                && path.is_relative()
            {
                error = error.help(format!(
                    "relative paths start from the current directory, {}",
                    cwd.display()
                ));
            }
            Err(error)
        }
        Err(e) => Err(CliError::new(format!(
            "cannot read `{file}`: {}",
            jaic::io_reason(&e)
        ))),
        Ok(meta) if meta.is_dir() => {
            let mut error = CliError::new(format!("`{file}` is a directory, not a .jai file"));
            let files = jai_files(path);
            let entry = ["first.jai", "build.jai", "main.jai", "module.jai"]
                .into_iter()
                .find(|n| files.iter().any(|f| f == n));
            match (entry, files.len()) {
                (Some(entry), _) => {
                    error = error.help(format!("did you mean `{}`?", path.join(entry).display()))
                }
                (None, 0) => error = error.help("it holds no .jai files"),
                (None, _) => {
                    let list: Vec<String> =
                        files.iter().take(5).map(|f| format!("`{f}`")).collect();
                    error = error.help(format!("pass one of its files: {}", list.join(", ")));
                }
            }
            Err(error)
        }
        Ok(_) => match std::fs::File::open(path) {
            Ok(_) => Ok(()),
            Err(e) => Err(CliError::new(format!(
                "cannot read `{file}`: {}",
                jaic::io_reason(&e)
            ))),
        },
    }
}

/// `path` as the user would write it: relative to where jaic was started when inside it.
fn shown(path: &Path) -> String {
    jaic::display_path(path)
}

/// Print an error given as text, its `help: `/`note: ` lines rendered as such.
fn print_error(message: &str) {
    let report = jaic::render::Report::from_text(jaic::render::Severity::Error, message);
    eprint!("{}", report.render());
}

/// Print a command-line mistake and return the usage exit status.
fn report_cli_error(error: CliError) -> ExitCode {
    print_cli_error(error);
    ExitCode::from(2)
}

fn print_cli_error(error: CliError) {
    let mut report = jaic::render::Report::new(jaic::render::Severity::Error, error.message);
    for help in error.help {
        report = report.help(help);
    }
    eprint!("{}", report.render());
}

fn main() -> ExitCode {
    install_panic_hook();
    let args: Vec<String> = std::env::args().skip(1).collect();
    // Colour and layout first, so even command-line mistakes use them.
    let color = jaic::render::ColorChoice::from_args(&args, &["--color", "-color"]);
    jaic::render::set_style(jaic::render::detect(color));
    if let Err(message) = jaic::memory_limit::arm_from_env() {
        eprint!(
            "{}",
            jaic::render::Report::new(jaic::render::Severity::Error, message).render()
        );
        return ExitCode::from(2);
    }
    let cli = match parse(&args) {
        Ok(Request::Compile(cli)) => *cli,
        Ok(Request::Help) => {
            println!("{}", usage_text());
            return ExitCode::SUCCESS;
        }
        Ok(Request::Version) => {
            println!("jaic {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        Err(error) => return report_cli_error(error),
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

/// A panic is a bug in jaic, not in the program: say so, and how to report it, instead of
/// Rust's bare panic message. The exit status stays 101.
fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let message = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "unknown panic".into());
        let at = info
            .location()
            .map(|l| format!(" (at {}:{})", l.file(), l.line()))
            .unwrap_or_default();
        let report = jaic::render::Report::new(
            jaic::render::Severity::Error,
            format!("internal compiler error: {message}"),
        )
        .note(format!("this is a bug in jaic{at}, not in your program"))
        .help(format!(
            "please report it at {}/issues with the program that triggers it; set RUST_BACKTRACE=1 for a backtrace",
            env!("CARGO_PKG_REPOSITORY")
        ));
        eprint!("{}", report.render());
        if std::env::var_os("RUST_BACKTRACE").is_some() {
            default(info);
        }
    }));
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
    jaic::interp::profile::write_coverage();
    code
}

fn compile_and_run(mut cli: Cli) -> ExitCode {
    let stdlib = stdlib_dir();
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
    let path = jaic::canonicalize(&cli.file).unwrap_or_else(|_| PathBuf::from(&cli.file));
    // Like `jai`, run from the main file's directory (so the program's meaning does not
    // depend on where the compiler was started); paths given on the command line stay
    // relative to the original directory.
    let absolute = |p: &PathBuf| std::path::absolute(p).unwrap_or_else(|_| p.clone());
    cli.imports = cli.imports.iter().map(absolute).collect();
    cli.output = cli.output.as_ref().map(absolute);
    cli.emit_ir = cli.emit_ir.as_ref().map(absolute);
    let main_dir = path.parent().map(PathBuf::from).unwrap_or_default();
    let started_in = std::env::current_dir().unwrap_or_default();
    jaic::set_display_base(started_in.clone());
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
    if cli.no_dce {
        options.dead_code = DeadCode::None;
    }
    // Only native output has a use for variable and type descriptions.
    options.debug_info = cli.command == Command::Build && !cli.no_debug_info;
    match cli.target_triple() {
        // The interpreter models wasm32 (the browser engine's target); LLVM output cannot.
        Ok(Some(triple)) if triple.starts_with("wasm32") && cli.command == Command::Build => {
            eprintln!(
                "error: jaic builds wasm64 (Memory64) only; Jai needs 8-byte pointers (use -target wasm64-unknown-wasi)"
            );
            return ExitCode::from(2);
        }
        Ok(Some(triple)) => {
            let (os, cpu) = os_and_cpu(&triple);
            options.os = os;
            options.cpu = cpu;
            let gnu = triple.contains("gnu") || triple.contains("mingw");
            options.long_double = jaic::sema::long_double_for(os, cpu, gnu);
        }
        Ok(None) if cli.os.is_some() || cli.cpu.is_some() => {
            // `-os`/`-cpu` for checking only: that OS and CPU, with its usual C compiler. A wasm
            // target has its own CPU (`CUSTOM`), as `jaic build -os wasm` and the browser give it.
            if options.os == TargetOs::Wasm && cli.cpu.is_none() {
                options.cpu = TargetCpu::Wasm;
            }
            options.long_double = jaic::sema::long_double_for(options.os, options.cpu, false);
        }
        Ok(None) => {}
        Err(message) => {
            print_error(&message);
            return ExitCode::from(2);
        }
    }
    // The local `modules` folder is searched first, then `-import_dir`s, then the stdlib.
    options.import_paths = vec![main_dir.join("modules")];
    options.import_paths.extend(cli.imports.iter().cloned());
    options.import_paths.push(stdlib.clone());
    options.preload = Some(stdlib.join("prelude/module.jai"));
    let fs: Rc<dyn FileSystem> = Rc::new(NativeFs);
    // What workspaces created by metaprograms ask for is written by `build` and `run` alike:
    // `run` differs only in interpreting the top-level program instead of compiling it.
    // `check` only checks.
    let writes_workspaces = match cli.command {
        Command::Build => true,
        Command::Run => !cli.no_workspace_output,
        Command::Check => false,
    };
    let backend: Option<Box<dyn OutputBackend>> =
        writes_workspaces.then(|| Box::new(native_backend(&cli)) as Box<dyn OutputBackend>);
    // `OS == .WASM` code is written for the interpreter's sandbox host (virtual clock and files,
    // cooperative threads), so compile-time code of a wasm target runs there whatever the
    // command. `jaic run -os wasm` runs the program there too, the way the browser does, with its
    // output printed when the run ends; `jaic build` for wasm compiles it to a wasm module.
    let cwd = std::env::current_dir().unwrap_or_default();
    let shared_sandbox = Rc::new(RefCell::new(SandboxHost::with_files(
        fs.clone(),
        &cwd.to_string_lossy(),
    )));
    let sandbox = (options.os == TargetOs::Wasm).then(|| shared_sandbox.clone());
    let workspace_sandbox = shared_sandbox.clone();
    let workspaces = Workspaces::new(BuildEnv {
        fs: fs.clone(),
        options: options.clone(),
        backend,
        unwritten_output_hint: (cli.command == Command::Check).then(|| {
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
        report: Box::new(|text| eprintln!("{text}")),
        observer: None,
    });
    workspaces.borrow_mut().main_file = path.display().to_string();
    // `check` and `build` exit right after: tearing down the syntax trees and the program takes
    // about a tenth of a large `check`, and nothing observes it.
    let mut compiler = LeakOnExit::new(Compiler::new(options, fs), cli.command != Command::Run);
    if let Some(host) = &sandbox {
        compiler.interp.host = Box::new(SharedHost(host.clone()));
    }
    compiler.attach_workspaces(workspaces.clone());
    // `-os wasm` / `-target wasm64-unknown-wasi`: a WASI command, with its runtime added.
    let wasi = cli.command == Command::Build
        && cli
            .target_triple()
            .ok()
            .flatten()
            .is_some_and(|t| jaic::build::wants_wasi_runtime(compiler.options.os, &t));
    let compiled = if wasi && cli.plugins.is_empty() {
        let sources = [
            ProgramSource::File(path.clone()),
            ProgramSource::String(jaic::build::WASI_RUNTIME_IMPORT.into()),
        ];
        timings::time("front end", || compiler.compile_sources(&sources))
    } else if cli.plugins.is_empty() {
        timings::time("front end", || compiler.compile_program(&path))
    } else {
        let source = plugin_metaprogram(&cli, &path);
        timings::time("front end", || {
            compiler.compile_sources(&[ProgramSource::String(source)])
        })
    };
    if cli.command == Command::Build {
        // What compile-time code of a wasm target printed (it runs in the sandbox).
        flush_sandbox(&shared_sandbox);
    }
    for warning in &compiler.warnings {
        eprintln!("{}", compiler.render(warning));
    }
    if let Err(d) = compiled {
        eprintln!("{}", compiler.render(&d));
        return ExitCode::from(1);
    }
    let finished = timings::time("workspaces", || {
        jaic::build::finish_all(&workspaces, &mut compiler.interp.block_budget)
    });
    if cli.command == Command::Build {
        flush_sandbox(&shared_sandbox);
    }
    if let Err(message) = finished {
        print_error(&message);
        return ExitCode::from(1);
    }
    if workspaces.borrow().any_failed() {
        return ExitCode::from(1);
    }
    let settings = workspaces.borrow().top_level_settings();
    match cli.command {
        Command::Check => ExitCode::SUCCESS,
        Command::Run => {
            // The program sees itself as the executable `jaic build` would write next to its
            // main file (not `jaic`), so data found relative to the executable is found.
            if sandbox.is_none() {
                let stem = path.file_stem().unwrap_or_default().to_string_lossy();
                let name = if cfg!(windows) {
                    format!("{stem}.exe")
                } else {
                    stem.into_owned()
                };
                compiler.interp.run_executable = Some(main_dir.join(name).display().to_string());
            }
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
        Command::Build
            if settings.output_type == OutputType::Executable
                && compiler.exported_func("main").is_none() =>
        {
            let file = shown(&path);
            print_error(&format!(
                "`{file}` has no `main` procedure, so there is no program to write\n\
                 help: add `main :: () {{ ... }}` as the program's starting point, or use `jaic check {file}` to only check the code"
            ));
            ExitCode::from(NO_MAIN_STATUS)
        }
        Command::Build => match build(&mut compiler, &cli, &path, settings) {
            Ok(()) => ExitCode::SUCCESS,
            Err(message) => {
                print_error(&message);
                ExitCode::from(1)
            }
        },
    }
}

/// A value the process never tears down when `leak` is set: it exits right after, and freeing a
/// large compiler piece by piece takes longer than the exit itself.
struct LeakOnExit<T> {
    value: Option<T>,
    leak: bool,
}

impl<T> LeakOnExit<T> {
    fn new(value: T, leak: bool) -> Self {
        LeakOnExit {
            value: Some(value),
            leak,
        }
    }
}

impl<T> std::ops::Deref for LeakOnExit<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.value.as_ref().expect("present until dropped")
    }
}

impl<T> std::ops::DerefMut for LeakOnExit<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.value.as_mut().expect("present until dropped")
    }
}

impl<T> Drop for LeakOnExit<T> {
    fn drop(&mut self) {
        if self.leak {
            std::mem::forget(self.value.take());
        }
    }
}

/// Print and drop what the sandbox host has buffered.
fn flush_sandbox(host: &Rc<RefCell<SandboxHost>>) {
    use std::io::Write;
    let mut host = host.borrow_mut();
    let _ = std::io::stdout().write_all(&std::mem::take(&mut host.stdout));
    let _ = std::io::stderr().write_all(&std::mem::take(&mut host.stderr));
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
        // `-O` stands for a whole flavor: the metaprogram's code generation switches go too.
        settings.optimization = level.into();
        settings.machine_code_optimization.clear();
        settings.llvm_switches.clear();
        settings.frame_pointers = None;
    }
    if settings.use_custom_link_command {
        // Nothing waits for the top-level program's messages, so nothing could link it.
        return Err(
            "`use_custom_link_command` is set for the top-level program, which no \
             metaprogram links\nhelp: set it on a workspace the metaprogram creates and link \
             on READY_FOR_CUSTOM_LINK_COMMAND"
                .into(),
        );
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
        sanitize: cli.sanitize.join(","),
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
    /// The `-sanitize` lists, comma-joined (empty: no sanitizers).
    sanitize: String,
}

/// What one write of a workspace's output needs, worked out from its settings.
#[cfg(feature = "llvm")]
struct Prepared {
    output: PathBuf,
    /// The first object file (`emit_objects` numbers the others after it).
    object: PathBuf,
    /// The objects stay after linking, for the debugger.
    keep_objects: bool,
    target: Option<String>,
    options: jaic_llvm::Options,
    /// Linker arguments beyond the metaprogram's (`-mmacosx-version-min`).
    extra_link_args: Vec<String>,
}

#[cfg(feature = "llvm")]
impl LlvmBackend {
    fn prepare(&self, settings: &BuildSettings, output: &Path) -> Result<Prepared, String> {
        // `-target`/`-os` win; a workspace built for `os_target = .WASM` names its triple in
        // `llvm_options.target_system_triple` (default: bare wasm64, its runtime up to the program).
        let wasm_settings = settings.os == Some(TargetOs::Wasm);
        let target = self.target.clone().or_else(|| {
            wasm_settings.then(|| {
                if settings.llvm_triple.is_empty() {
                    jaic_llvm::WASM_TRIPLE.to_string()
                } else {
                    settings.llvm_triple.clone()
                }
            })
        });
        let non_empty = |s: &String| (!s.is_empty()).then(|| s.clone());
        // Windows wants `.exe`/`.dll`/`.lib`; a name without an extension gets the platform's,
        // unless the metaprogram turned `append_executable_filename_extension` off.
        use jaic_llvm::OutputKind;
        let kind = match settings.output_type {
            OutputType::Executable => Some(OutputKind::Executable),
            OutputType::DynamicLibrary => Some(OutputKind::DynamicLibrary),
            OutputType::StaticLibrary => Some(OutputKind::StaticLibrary),
            OutputType::ObjectFile | OutputType::NoOutput => None,
        };
        let mut output = output.to_path_buf();
        if settings.append_extension
            && output.extension().is_none()
            && let Some(ext) = kind.and_then(|k| jaic_llvm::output_extension(target.as_deref(), k))
        {
            output.set_extension(ext);
        }
        if output.is_dir() {
            let name = output.join("program");
            return Err(format!(
                "the output path `{}` is a directory\nhelp: name the file to write inside it, as in `-o {}`",
                shown(&output),
                shown(&name)
            ));
        }
        let create_dir = |dir: &Path, what: &str| {
            std::fs::create_dir_all(dir).map_err(|e| {
                format!(
                    "could not create the {what} directory `{}`: {}\nhelp: choose another place with `-o`",
                    shown(dir),
                    jaic::io_reason(&e)
                )
            })
        };
        if let Some(dir) = output.parent().filter(|d| !d.as_os_str().is_empty()) {
            create_dir(dir, "output")?;
        }
        // Objects, IR and bitcode go to `intermediate_path` when it is set, else next to the
        // output.
        let file_name = output.file_name().unwrap_or_default().to_os_string();
        let intermediate = if settings.intermediate_path.is_empty() {
            output.parent().map(Path::to_path_buf).unwrap_or_default()
        } else {
            let dir = PathBuf::from(&settings.intermediate_path);
            create_dir(&dir, "intermediate")?;
            dir
        };
        let beside = |suffix: &str| {
            let mut name = file_name.clone();
            name.push(suffix);
            intermediate.join(name)
        };
        use jaic_llvm::OptLevel;
        let opt_level = match settings.optimization.as_str() {
            // `llvm_options.bitcode_optimization_setting` member names.
            "O1" => OptLevel::O1,
            "O2" => OptLevel::O2,
            "OS" => OptLevel::Os,
            "OZ" => OptLevel::Oz,
            "O3" => OptLevel::O3,
            _ => OptLevel::O0,
        };
        // `Build_Options.emit_debug_info = .NONE`, `llvm_options.preserve_debug_info = false`
        // (or `set_optimization(..., false)`) turn it off.
        let debug_info = self.debug_info
            && settings.emit_debug_info != Some(false)
            && settings.llvm_switch("preserve_debug_info") != Some(false);
        let sanitize = if self.sanitize.is_empty() {
            jaic_llvm::Sanitize::default()
        } else {
            jaic_llvm::Sanitize::parse(&self.sanitize)?
        };
        let macos = target
            .as_deref()
            .map_or(cfg!(target_os = "macos"), |t| t.contains("apple"));
        let macos_version = (macos && !settings.minimum_os_version.is_empty())
            .then(|| settings.minimum_os_version.clone());
        let mut extra_link_args = Vec::new();
        if let Some(version) = &macos_version {
            extra_link_args.push(format!("-mmacosx-version-min={version}"));
        }
        let switch = |name: &str| settings.llvm_switch(name);
        let wanted = |on: bool, suffix: &str| on.then(|| beside(suffix));
        let codegen = jaic_llvm::Codegen {
            machine_level: match settings.machine_code_optimization.as_str() {
                "NONE" => Some(OptLevel::O0),
                "LESS" => Some(OptLevel::O1),
                "DEFAULT" => Some(OptLevel::O2),
                "AGGRESSIVE" => Some(OptLevel::O3),
                _ => None,
            },
            loop_unrolling: switch("enable_loop_unrolling"),
            loop_vectorization: switch("enable_loop_vectorization"),
            slp_vectorization: switch("enable_slp_vectorization"),
            merge_functions: switch("merge_functions"),
            disable_inlining: switch("disable_inlining"),
            tail_calls: switch("enable_tail_calls"),
            split_modules: switch("enable_split_modules"),
            frame_pointers: settings.frame_pointers,
            no_red_zone: settings.disable_redzone,
            ir_after: wanted(settings.output_llvm_ir, ".ll"),
            bitcode_after: wanted(settings.output_bitcode, ".bc"),
            bitcode_before: wanted(
                settings.output_bitcode_before_optimizations,
                ".unoptimized.bc",
            ),
            macos_version,
        };
        // `-emit-ir` wins over `output_llvm_ir_before_optimizations`.
        let emit_ir = self.emit_ir.clone().or_else(|| {
            wanted(
                settings.output_llvm_ir_before_optimizations,
                ".unoptimized.ll",
            )
        });
        let options = jaic_llvm::Options {
            opt_level,
            target: target.clone(),
            emit_ir,
            debug_info,
            sanitize,
            cpu: non_empty(&settings.llvm_cpu),
            features: non_empty(&settings.llvm_features),
            codegen,
        };
        // An unoptimized macOS executable or library keeps its objects, in `.build/` beside the
        // output (or in the intermediate directory): the debugger reads the DWARF from them
        // (`docs/native/debug-info.md`). Every other build deletes its objects after linking. A
        // sanitized build still writes the dSYM: the sanitizer's symbolizer does not read the debug
        // map on every macOS, and its reports need file and line.
        let keep_objects = debug_info
            && macos
            && opt_level == OptLevel::O0
            && self.sanitize.is_empty()
            && matches!(
                settings.output_type,
                OutputType::Executable | OutputType::DynamicLibrary
            )
            && std::env::var_os("JAIC_DSYM").is_none();
        let kept_dir = if settings.intermediate_path.is_empty() {
            intermediate.join(".build")
        } else {
            intermediate.clone()
        };
        if settings.output_type != OutputType::ObjectFile {
            remove_stale_objects(&kept_dir, &file_name);
        }
        let object = if settings.output_type == OutputType::ObjectFile {
            output.clone()
        } else if keep_objects {
            kept_dir.join(beside(".o").file_name().unwrap_or_default())
        } else {
            beside(".o")
        };
        // Creating the object file now finds out, with the system's reason, when that place
        // cannot be written, rather than from LLVM or the linker after code generation.
        if keep_objects {
            std::fs::create_dir_all(&kept_dir).map_err(|e| {
                format!(
                    "could not create the object directory `{}`: {}",
                    shown(&kept_dir),
                    jaic::io_reason(&e)
                )
            })?;
        }
        std::fs::File::create(&object).map_err(|e| {
            format!(
                "cannot write `{}`: {}\nhelp: choose a directory you can write to with `-o`",
                shown(&object),
                jaic::io_reason(&e)
            )
        })?;
        Ok(Prepared {
            output,
            object,
            keep_objects,
            target,
            options,
            extra_link_args,
        })
    }
}

/// Delete the objects an earlier build of `file_name` left in `dir` (`<name>.o`, `<name>.o.N.o`),
/// so a build with fewer codegen units leaves none behind. The directory itself stays: concurrent
/// builds into the same output directory share it, and removing it would race their writes.
#[cfg(feature = "llvm")]
fn remove_stale_objects(dir: &Path, file_name: &std::ffi::OsStr) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let prefix = file_name.to_string_lossy().into_owned();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        // `<name>.o`, then `<name>.o.N.o` for the other codegen units.
        let stale = name.strip_prefix(&prefix).is_some_and(|rest| {
            rest == ".o"
                || rest
                    .strip_prefix(".o.")
                    .and_then(|r| r.strip_suffix(".o"))
                    .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        });
        if stale {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

#[cfg(feature = "llvm")]
impl OutputBackend for LlvmBackend {
    fn write_output(
        &mut self,
        program: &jaic::ir::Program,
        settings: &BuildSettings,
        output: &Path,
    ) -> Result<(), String> {
        let Prepared {
            output,
            object,
            keep_objects,
            target,
            options,
            extra_link_args,
        } = self.prepare(settings, output)?;
        let (output, target) = (output.as_path(), target.as_deref());
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
        let wasm = target.is_some_and(jaic_llvm::is_wasm_target);
        let mut link_args = settings.additional_linker_arguments.clone();
        link_args.extend(extra_link_args);
        let debug_info = options.debug_info;
        let linked = timings::time("link", || match settings.output_type {
            OutputType::ObjectFile | OutputType::NoOutput => unreachable!(),
            OutputType::Executable | OutputType::DynamicLibrary if wasm => {
                let start = jaic::ir::Linkage::Export("_start".into());
                jaic_llvm::link_wasm(&jaic_llvm::WasmLink {
                    objects: &objects,
                    libraries: &libraries,
                    output,
                    has_start: settings.output_type == OutputType::Executable
                        && program.funcs.iter().flatten().any(|f| f.linkage == start),
                    strip_debug: !debug_info,
                    extra_args: &settings.additional_linker_arguments,
                })
            }
            OutputType::Executable | OutputType::DynamicLibrary => jaic_llvm::link(
                &objects,
                &libraries,
                output,
                settings.output_type == OutputType::DynamicLibrary,
                &link_args,
                target,
                options.sanitize,
                debug_info,
            ),
            OutputType::StaticLibrary => jaic_llvm::archive(&objects, output, target),
        });
        // macOS linkers leave DWARF in the objects and record where they are. An unoptimized
        // build keeps its objects for the debugger to read (the `dsymutil` pass would be a
        // tenth of the build); any other collects the DWARF into `output.dSYM` first.
        let macho = debug_info
            && linked.is_ok()
            && target.map_or(cfg!(target_os = "macos"), |t| t.contains("apple"))
            && settings.output_type != OutputType::StaticLibrary;
        if macho
            && !keep_objects
            && let Err(message) = timings::time("debug info", || jaic_llvm::write_dsym(output))
        {
            eprintln!("warning: {message}");
        }
        if !(keep_objects && linked.is_ok()) {
            for object in &objects {
                let _ = std::fs::remove_file(object);
            }
        }
        linked
    }

    fn write_objects(
        &mut self,
        program: &jaic::ir::Program,
        settings: &BuildSettings,
        output: &Path,
    ) -> Result<jaic::build::LinkObjects, String> {
        let prepared = self.prepare(settings, output)?;
        let objects = timings::time("codegen", || {
            jaic_llvm::emit_objects(program, &prepared.options, &prepared.object)
        })?;
        let libraries = jaic_llvm::used_libraries(program);
        let inputs = jaic_llvm::link_inputs(&libraries, prepared.target.as_deref())?;
        Ok(jaic::build::LinkObjects {
            objects: objects.iter().map(|o| o.display().to_string()).collect(),
            system_libraries: inputs.system_libraries,
            user_libraries: inputs.user_libraries,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triple(args: &[&str], host_os: TargetOs, host_cpu: TargetCpu) -> Option<String> {
        let mut argv = vec!["build".to_string(), "main.jai".to_string()];
        argv.extend(args.iter().map(|a| a.to_string()));
        let Ok(Request::Compile(cli)) = parse(&argv) else {
            panic!("invalid command line {argv:?}");
        };
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
