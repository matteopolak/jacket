//! `jaic` command line: `jaic [options] file.jai [options] [+Plugin ...] [- metaprogram args] [-- program args]`.
use jaic::build::{BuildEnv, BuildSettings, MainOverrides, OutputBackend, OutputType, Workspaces};
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

/// `-timings`: wall time per compiler phase, printed to stderr when the command ends.
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

/// What jaic does with the program: build it (the default), only compile it, or interpret it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Check,
    Interp,
    Build,
}

/// The optimization flavor `-o`, `-od` and `-very_debug` stand for.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Flavor {
    Optimized,
    OptimizedDebug,
    VeryDebug,
}

impl Flavor {
    /// The `llvm_options.bitcode_optimization_setting` name this flavor builds with.
    fn level(self) -> &'static str {
        match self {
            Flavor::Optimized => "O2",
            Flavor::OptimizedDebug => "O1",
            Flavor::VeryDebug => "O0",
        }
    }
}

/// A `-add` or `-run` string, in command-line order.
enum Added {
    /// `-add`: more code for the target program.
    Code(String),
    /// `-run`: code to run at compile time.
    Run(String),
}

impl Added {
    /// The source text this adds to the program.
    fn source(&self) -> String {
        match self {
            Added::Code(code) => code.clone(),
            Added::Run(code) => {
                let end = if code.trim_end().ends_with([';', '}']) {
                    ""
                } else {
                    ";"
                };
                format!("#run {{\n{code}{end}\n}};\n")
            }
        }
    }
}

/// One `+Name` plugin with the arguments that follow it.
struct Plugin {
    /// The module name; it may carry module parameters, `Check(CHECK_BINDINGS=false)`.
    name: String,
    options: Vec<String>,
}

/// Parsed command line.
struct Cli {
    mode: Mode,
    file: String,
    imports: Vec<PathBuf>,
    /// `-exe`: the target workspace's `output_executable_name`. It may carry a directory and an
    /// extension.
    exe: Option<String>,
    /// `-output_path`: the target workspace's `output_path`.
    output_path: Option<PathBuf>,
    /// `-o`, `-od` or `-very_debug`.
    flavor: Option<Flavor>,
    /// `-O0`..`-O3`; `None`: what the metaprogram chose (default: unoptimized).
    opt_level: Option<&'static str>,
    emit_ir: Option<PathBuf>,
    /// `-no_debug_info`: omit native debug information (on by default).
    no_debug_info: bool,
    /// `-no_inline`: every function not marked `inline` gets `noinline`.
    no_inline: bool,
    /// `-no_split`: generate code in one module instead of several.
    no_split: bool,
    /// `-no_backtrace_on_crash`: no crash handler in the target program.
    no_backtrace_on_crash: bool,
    /// `-add` and `-run` strings.
    added: Vec<Added>,
    /// Arguments after `-`, for the metaprogram (`compiler_get_command_line`).
    command_line: Vec<String>,
    /// `-interp` only: arguments after `--`, for the program (`get_command_line_arguments`).
    program_args: Vec<String>,
    /// `-os`: the target `OS` when it is not the host (checking code for another platform).
    os: Option<TargetOs>,
    /// `-cpu`: the target `CPU` when it is not the host's (`-os windows -cpu arm64`).
    cpu: Option<TargetCpu>,
    /// `+Name`: metaprogram plugin modules, each with the arguments up to the next `+Name`.
    plugins: Vec<Plugin>,
    /// `-target`: an explicit LLVM target triple (`x86_64-pc-windows-msvc`...).
    target: Option<String>,
    /// `-timings`: report the wall time of each phase on stderr.
    timings: bool,
    /// `-sanitize address,undefined` (repeatable): sanitizer instrumentation for builds.
    sanitize: Vec<String>,
    /// `-no_dce`: type-check every declaration, modules included
    /// (`Build_Options.dead_code_elimination = .NONE`).
    no_dce: bool,
    /// `-no_check_bindings`: `CHECK_BINDINGS=false` for a `+Check` plugin.
    no_check_bindings: bool,
    /// `-no_workspace_output` (`-interp`): do not write what workspaces a metaprogram creates
    /// ask for (test sweeps over many programs).
    no_workspace_output: bool,
    /// `-no_color`: no colour in diagnostics.
    no_color: bool,
    /// `-msvc_format`: diagnostics as `path(line,col): error: message` lines.
    msvc_format: bool,
    /// `-quiet`: print nothing but errors.
    quiet: bool,
    /// `-verbose`: say what is being done.
    verbose: bool,
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
            _ if self.mode != Mode::Build => Ok(None),
            // A WASI command, runnable by node, wasmtime and the like.
            TargetOs::Wasm => Ok(Some(WASI_TRIPLE.into())),
            _ => Err(
                "native cross-compilation is only supported for -os windows and -os wasm; pass -target <triple> for others"
                    .into(),
            ),
        }
    }
}

/// What `jaic -os wasm` targets: wasm64 with the `Wasi_Runtime` module linked in.
const WASI_TRIPLE: &str = "wasm64-unknown-wasi";

/// A build's exit status when the program has no `main` (its work is all `#run`s), so
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

/// `jaic -help`.
fn usage_text() -> String {
    format!(
        "jaic {version}: compile, check and run Jai programs

usage: jaic [options] file.jai [options] [+Plugin [plugin options...]]... [- metaprogram args...] [-- program args...]

By default jaic builds a native executable. Options may come before or after the file.

modes:
  -check                    compile the program without running or writing anything
  -interp                   run `main` in the interpreter (arguments after `--` go to the program)

program:
  -add \"code\"               add the code to the target program (repeatable)
  -run \"code\"               run the code at compile time, as a `#run` (repeatable)
  -import_dir <dir>         also look for modules in <dir> (repeatable; short form -I)
  -os <os>                  target OS: linux, windows, macos or wasm
  -cpu <cpu>                target CPU: x64 or arm64
  -target <triple>          build for an LLVM target triple
  -no_dce                   type-check unreferenced module code too
  -no_check                 accepted and ignored: jaic does not run the Check plugin unless asked
  -no_check_bindings        set CHECK_BINDINGS=false on a +Check plugin, if there is one

output (building only):
  -exe <name>               the executable's name; it may include a directory and extension
                            (`-exe build/app.exe`). A bare name goes next to the source file
  -output_path <dir>        the directory the executable is written to
  -o, -optimized            optimized build (LLVM O2) without stack traces or runtime checks
  -od, -optimized_debug     lightly optimized build (LLVM O1) that keeps stack traces and checks
  -very_debug               unoptimized build with every runtime check on
  -O0, -O1, -O2, -O3        pick the LLVM optimization level directly
  -no_inline                do not inline procedures
  -no_split                 generate code as one module instead of several
  -no_backtrace_on_crash    leave the crash handler out of the program
  -emit_ir <file.ll>        also write the LLVM IR
  -no_debug_info            leave out native debug information
  -sanitize <list>          address and/or undefined, comma-separated
  -no_workspace_output      (-interp only) do not write what a metaprogram's workspaces ask for

plugins:
  +Name                     run the metaprogram plugin module Name. Every argument after it, up
                            to the next +Name, `-` or `--`, is that plugin's own option.
                            Module parameters go in the name: '+Check(CHECK_BINDINGS=false)'
                            (quote it in a shell). Plugins do not combine with -interp.
  -                         the rest is for the metaprogram (`compiler_get_command_line`)
  --                        the rest is for the program (-interp only)

diagnostics and output:
  -no_color                 no colour in diagnostics
  -msvc_format              one `path(line,col): error: message` line per diagnostic
  -quiet                    print nothing but errors
  -verbose                  say what is being done
  -timings                  print the wall time of each phase on stderr
  -h, -help, --help         print this help
  -V, -version, --version   print the version

environment:
  JAIC_MEMORY_LIMIT=<bytes|nK|nM|nG>  stop with exit status {limit} once that much is allocated
  JAIC_DIAGNOSTICS=plain|ascii|unicode  diagnostic layout (default: by terminal)
  NO_COLOR, FORCE_COLOR, CLICOLOR_FORCE  turn colour off or on

exit status: 0 success, 1 the program failed to compile, an input file cannot be read
or a runtime error stopped it (`-interp` otherwise exits with the program's own status),
2 a command-line mistake, {no_main} a build found no `main` to make an executable from,
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
    ("-exe", "an executable name"),
    ("-output_path", "a directory"),
    ("-emit_ir", "a file to write the LLVM IR to"),
    ("-sanitize", "address, undefined or both, comma-separated"),
    ("-add", "the code to add to the program"),
    ("-run", "the code to run at compile time"),
];

/// Every option jaic knows, for "did you mean" suggestions.
const KNOWN_OPTIONS: &[&str] = &[
    "-check",
    "-interp",
    "-add",
    "-run",
    "-I",
    "-import_dir",
    "-os",
    "-cpu",
    "-target",
    "-exe",
    "-output_path",
    "-o",
    "-optimized",
    "-od",
    "-optimized_debug",
    "-very_debug",
    "-O0",
    "-O1",
    "-O2",
    "-O3",
    "-no_inline",
    "-no_split",
    "-no_dce",
    "-no_backtrace_on_crash",
    "-no_check",
    "-no_check_bindings",
    "-emit_ir",
    "-no_debug_info",
    "-sanitize",
    "-no_workspace_output",
    "-no_color",
    "-msvc_format",
    "-quiet",
    "-verbose",
    "-timings",
    "-help",
    "-version",
];

/// Options only a build takes: `-check` and `-interp` write no output.
const BUILD_ONLY: &[&str] = &[
    "-exe",
    "-output_path",
    "-o",
    "-optimized",
    "-od",
    "-optimized_debug",
    "-very_debug",
    "-O0",
    "-O1",
    "-O2",
    "-O3",
    "-no_inline",
    "-no_split",
    "-emit_ir",
    "-no_debug_info",
    "-sanitize",
];

/// Spellings from before 0.17.0 and what replaces them.
const RENAMED: &[(&str, &str)] = &[
    ("--emit-ir", "-emit_ir <file.ll>"),
    ("--no-debug-info", "-no_debug_info"),
    ("--timings", "-timings"),
    ("--sanitize", "-sanitize <list>"),
    ("--target", "-target <triple>"),
    (
        "--color",
        "-no_color to turn colour off; FORCE_COLOR=1 turns it on",
    ),
    (
        "-color",
        "-no_color to turn colour off; FORCE_COLOR=1 turns it on",
    ),
    ("-plug", "+Name after the file, as in `jaic file.jai +Name`"),
    (
        "-plugin",
        "+Name after the file, as in `jaic file.jai +Name`",
    ),
];

/// `arg` as a shell would need it written.
fn shell_word(arg: &str) -> String {
    let plain = !arg.is_empty()
        && arg
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./:=+,@%".contains(c));
    if plain {
        arg.to_string()
    } else {
        format!("'{}'", arg.replace('\'', "'\\''"))
    }
}

/// The error for `jaic run|check|build ...`: the same command in the single-command form, with
/// the other arguments carried over (renamed options included).
fn subcommand_error(args: &[String]) -> CliError {
    let old = args[0].as_str();
    let mode_flag = match old {
        "run" => Some("-interp"),
        "check" => Some("-check"),
        _ => None,
    };
    let mut file = None;
    let mut options: Vec<String> = Vec::new();
    let mut plugins: Vec<String> = Vec::new();
    let mut plugin_options: Vec<String> = Vec::new();
    let mut tail: Vec<String> = Vec::new();
    let mut rest = args[1..].iter();
    while let Some(arg) = rest.next() {
        let arg = arg.as_str();
        match arg {
            "-" | "--" => {
                tail.push(arg.to_string());
                tail.extend(rest.by_ref().cloned());
            }
            "-plug" | "-plugin" => plugins.extend(rest.next().map(|name| format!("+{name}"))),
            "--color" | "-color" => {
                if rest.next().is_some_and(|when| when == "never") {
                    options.push("-no_color".into());
                }
            }
            "-o" => {
                if let Some(path) = rest.next() {
                    options.extend(["-exe".into(), path.clone()]);
                }
            }
            "--emit-ir" | "--sanitize" | "--target" => {
                options.push(arg[1..].replace("emit-ir", "emit_ir"));
                options.extend(rest.next().cloned());
            }
            "--no-debug-info" | "--timings" => {
                options.push(arg[1..].replace("no-debug-info", "no_debug_info"));
            }
            _ if VALUE_OPTIONS.iter().any(|(option, _)| *option == arg) => {
                options.push(arg.to_string());
                options.extend(rest.next().cloned());
            }
            _ if arg.starts_with('-') || !plugin_options.is_empty() => {
                plugin_options.push(arg.to_string())
            }
            _ if file.is_none() => file = Some(arg.to_string()),
            _ => plugin_options.push(arg.to_string()),
        }
    }
    let mut words = vec![file.unwrap_or_else(|| "file.jai".into())];
    words.extend(options);
    words.extend(mode_flag.map(String::from));
    words.extend(plugins);
    words.extend(plugin_options);
    words.extend(tail);
    let words: Vec<String> = words.iter().map(|w| shell_word(w)).collect();
    CliError::new("jaic has no subcommands since 0.17.0")
        .help(format!("use `jaic {}`", words.join(" ")))
        .help(
            "`-check` only compiles, `-interp` runs the program in the interpreter, and with \
             neither jaic builds an executable",
        )
}

/// What is wrong with `--`, which only `-interp` takes.
fn program_args_error(file: &str) -> CliError {
    CliError::new("`--` hands arguments to the program, which only `-interp` runs")
        .help(format!("use `jaic {file} -interp -- ...`"))
        .help("arguments for the metaprogram go after a lone `-`")
}

fn parse(args: &[String]) -> Result<Request, CliError> {
    let Some(first) = args.first() else {
        return Err(CliError::new("no file to compile")
            .help("usage: jaic [options] file.jai [options]")
            .help("`jaic -help` lists the options"));
    };
    if matches!(first.as_str(), "run" | "check" | "build") && !Path::new(first).exists() {
        return Err(subcommand_error(args));
    }
    let mut cli = Cli {
        mode: Mode::Build,
        file: String::new(),
        imports: Vec::new(),
        exe: None,
        output_path: None,
        flavor: None,
        opt_level: None,
        emit_ir: None,
        no_debug_info: false,
        no_inline: false,
        no_split: false,
        no_backtrace_on_crash: false,
        added: Vec::new(),
        command_line: Vec::new(),
        program_args: Vec::new(),
        os: None,
        cpu: None,
        target: None,
        plugins: Vec::new(),
        timings: false,
        sanitize: Vec::new(),
        no_dce: false,
        no_check_bindings: false,
        no_workspace_output: false,
        no_color: false,
        msvc_format: false,
        quiet: false,
        verbose: false,
    };
    let mut file: Option<String> = None;
    let mut extra: Option<String> = None;
    let (mut check, mut interp) = (false, false);
    // The first option that only builds take, for the `-check`/`-interp` error.
    let mut build_only: Option<String> = None;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "-" => {
                // Metaprogram arguments run up to a `--` (if any).
                for arg in rest.by_ref() {
                    if arg == "--" {
                        if !interp {
                            return Err(program_args_error(file.as_deref().unwrap_or("file.jai")));
                        }
                        cli.program_args.push(file.clone().unwrap_or_default());
                        break;
                    }
                    cli.command_line.push(arg.clone());
                }
                cli.program_args.extend(rest.by_ref().cloned());
                continue;
            }
            "--" => {
                if !interp {
                    return Err(program_args_error(file.as_deref().unwrap_or("file.jai")));
                }
                // argv[0] is the source file, like a built program's executable path.
                cli.program_args.push(file.clone().unwrap_or_default());
                cli.program_args.extend(rest.by_ref().cloned());
                continue;
            }
            _ => {}
        }
        // A plugin takes every argument up to the next `+Name`, `-` or `--`.
        if let Some(name) = arg.strip_prefix('+').filter(|name| !name.is_empty()) {
            cli.plugins.push(Plugin {
                name: name.to_string(),
                options: Vec::new(),
            });
            continue;
        }
        if let Some(plugin) = cli.plugins.last_mut() {
            plugin.options.push(arg.clone());
            continue;
        }
        // `-os=linux` and the like.
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
        if BUILD_ONLY.contains(&a) {
            build_only.get_or_insert_with(|| a.to_string());
        }
        match a {
            "-h" | "-help" | "--help" => return Ok(Request::Help),
            "-V" | "-version" | "--version" => return Ok(Request::Version),
            "-check" => check = true,
            "-interp" => interp = true,
            "-I" | "-import_dir" => cli.imports.push(PathBuf::from(value(a)?)),
            "-add" => cli.added.push(Added::Code(value(a)?)),
            "-run" => cli.added.push(Added::Run(value(a)?)),
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
            "-target" => cli.target = Some(value(a)?),
            "-exe" => cli.exe = Some(value(a)?),
            "-output_path" => cli.output_path = Some(PathBuf::from(value(a)?)),
            "-o" | "-optimized" | "-od" | "-optimized_debug" | "-very_debug" => {
                // Before 0.17.0 `-o` named the output.
                if a == "-o"
                    && let Some(word) = rest.clone().next()
                    && !word.starts_with(['-', '+'])
                    && !word.ends_with(".jai")
                {
                    return Err(CliError::new("`-o` takes no value since 0.17.0")
                        .help("it asks for an optimized build; name the output with `-exe name` and `-output_path dir`")
                        .help(format!("to keep the old meaning of `-o {word}`, use `-exe {word}`")));
                }
                let flavor = match a {
                    "-o" | "-optimized" => Flavor::Optimized,
                    "-od" | "-optimized_debug" => Flavor::OptimizedDebug,
                    _ => Flavor::VeryDebug,
                };
                if cli.flavor.is_some_and(|f| f != flavor) {
                    return Err(CliError::new(
                        "`-o`, `-od` and `-very_debug` each pick a build flavor; use one",
                    ));
                }
                cli.flavor = Some(flavor);
            }
            "-O0" => cli.opt_level = Some("O0"),
            "-O1" => cli.opt_level = Some("O1"),
            "-O2" => cli.opt_level = Some("O2"),
            "-O3" => cli.opt_level = Some("O3"),
            "-no_inline" => cli.no_inline = true,
            "-no_split" => cli.no_split = true,
            "-no_dce" => cli.no_dce = true,
            "-no_backtrace_on_crash" => cli.no_backtrace_on_crash = true,
            // Check is not run unless asked for, so there is nothing to turn off.
            "-no_check" => {}
            "-no_check_bindings" => cli.no_check_bindings = true,
            "-emit_ir" => cli.emit_ir = Some(PathBuf::from(value(a)?)),
            "-no_debug_info" => cli.no_debug_info = true,
            "-sanitize" => {
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
            "-no_workspace_output" => cli.no_workspace_output = true,
            "-no_color" => cli.no_color = true,
            "-msvc_format" => cli.msvc_format = true,
            "-quiet" => cli.quiet = true,
            "-verbose" => cli.verbose = true,
            "-timings" => cli.timings = true,
            other if other.starts_with('-') => {
                let name = other.split_once('=').map_or(other, |(name, _)| name);
                if let Some((_, instead)) = RENAMED.iter().find(|(old, _)| *old == name) {
                    return Err(CliError::new(format!(
                        "`{name}` is spelled differently since 0.17.0"
                    ))
                    .help(format!("use {instead}")));
                }
                if other.starts_with("-O") {
                    return Err(
                        CliError::new(format!("unknown optimization level `{other}`"))
                            .help("use -O0, -O1, -O2 or -O3"),
                    );
                }
                let mut error = CliError::new(format!("unknown option `{other}`"));
                if let Some(near) = jaic::suggest::closest(other, KNOWN_OPTIONS.iter().copied()) {
                    error = error.help(format!("did you mean `{near}`?"));
                }
                return Err(error.help(
                    "options after a `+Plugin` belong to that plugin; `jaic -help` lists jaic's own",
                ));
            }
            other if file.is_none() => file = Some(other.to_string()),
            other => {
                extra.get_or_insert_with(|| other.to_string());
            }
        }
    }
    let Some(file) = file else {
        return Err(CliError::new("no file to compile")
            .help("usage: jaic [options] file.jai [options]")
            .help("`jaic -help` lists the options"));
    };
    cli.file = file;
    if check && interp {
        return Err(CliError::new("`-check` and `-interp` cannot be combined")
            .help("`-check` only compiles the program; `-interp` runs it in the interpreter"));
    }
    cli.mode = match (check, interp) {
        (true, _) => Mode::Check,
        (_, true) => Mode::Interp,
        _ => Mode::Build,
    };
    if let Some(word) = extra {
        let mut error = if word.ends_with(".jai") {
            CliError::new(format!(
                "jaic compiles one file, but got `{}` and `{word}`",
                cli.file
            ))
            .help("`#load` or `#import` the others from the first")
        } else {
            CliError::new(format!("unexpected argument `{word}`"))
        };
        if cli.mode == Mode::Interp {
            error = error.help(format!(
                "arguments for the program go after `--`: `jaic {} -interp -- {word}`",
                cli.file
            ));
        }
        return Err(error);
    }
    if let Some(option) = build_only.filter(|_| cli.mode != Mode::Build) {
        let (mode, why) = if cli.mode == Mode::Check {
            ("-check", "only compiles and writes no output")
        } else {
            (
                "-interp",
                "runs the program in the interpreter and writes no output",
            )
        };
        return Err(
            CliError::new(format!("`{option}` only applies to a build, not `{mode}`"))
                .help(format!("`{mode}` {why}; leave `{mode}` out to build")),
        );
    }
    if cli.no_workspace_output && cli.mode != Mode::Interp {
        return Err(
            CliError::new("`-no_workspace_output` only applies to `-interp`").help(
                if cli.mode == Mode::Check {
                    "`-check` never writes workspace output"
                } else {
                    "a build writes what the workspaces ask for; leave the option out"
                },
            ),
        );
    }
    if cli.opt_level.is_some() && cli.flavor.is_some() {
        return Err(CliError::new(
            "`-O0`..`-O3` and `-o`, `-od` or `-very_debug` both set the optimization",
        )
        .help("use one of them"));
    }
    if cli.quiet && cli.verbose {
        return Err(CliError::new(
            "`-quiet` and `-verbose` contradict each other",
        ));
    }
    // Plugins compile the program in a workspace of their own, which `-interp` cannot start.
    if !cli.plugins.is_empty() && cli.mode == Mode::Interp {
        return Err(
            CliError::new("plugins work with a build or `-check`, not `-interp`").help(format!(
                "use `jaic {} +{} ...` and run the executable",
                cli.file, cli.plugins[0].name
            )),
        );
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

/// Whether `-no_color` is among jaic's own arguments, read before parsing so that even
/// command-line mistakes are printed without colour. Plugin, metaprogram and program
/// arguments (from the first `+Name`, `-` or `--`) are not jaic's.
fn wants_no_color(args: &[String]) -> bool {
    args.iter()
        .take_while(|a| *a != "-" && *a != "--" && !a.starts_with('+'))
        .any(|a| a == "-no_color")
}

fn main() -> ExitCode {
    install_panic_hook();
    let args: Vec<String> = std::env::args().skip(1).collect();
    // Colour and layout first, so even command-line mistakes use them.
    let color = if wants_no_color(&args) {
        jaic::render::ColorChoice::Never
    } else {
        jaic::render::ColorChoice::Auto
    };
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
    if cli.msvc_format {
        jaic::render::set_style(jaic::render::Style {
            layout: jaic::render::Layout::Msvc,
            color: false,
        });
    }

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

/// Whether a reported line is a warning or an informational note, which `-quiet` leaves out.
fn is_chatter(text: &str) -> bool {
    let first = text.lines().next().unwrap_or_default();
    ["warning: ", "info: "]
        .iter()
        .any(|kind| first.starts_with(kind) || first.contains(&format!(": {kind}")))
}

/// Whether `exe` names a place (`build/app`, `/tmp/app`), not just a file name.
fn has_directory(exe: &str) -> bool {
    Path::new(exe).components().count() > 1 || Path::new(exe).is_absolute()
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
    cli.output_path = cli.output_path.as_ref().map(absolute);
    // `-exe` with a directory is taken from where jaic was started, like the paths above, and
    // splits into the output path and the executable's name; a bare name goes beside the main
    // file (or into `-output_path`).
    if let Some(exe) = cli.exe.clone()
        && cli.output_path.is_none()
        && has_directory(&exe)
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
    match cli.target_triple() {
        // The interpreter models wasm32 (the browser engine's target); LLVM output cannot.
        Ok(Some(triple)) if triple.starts_with("wasm32") && cli.mode == Mode::Build => {
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
            // target has its own CPU (`CUSTOM`), as a `-os wasm` build and the browser give it.
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
    // What workspaces created by metaprograms ask for is written by a build and `-interp` alike:
    // `-interp` differs only in interpreting the top-level program instead of compiling it.
    // `-check` only checks.
    let writes_workspaces = match cli.mode {
        Mode::Build => true,
        Mode::Interp => !cli.no_workspace_output,
        Mode::Check => false,
    };
    let backend: Option<Box<dyn OutputBackend>> =
        writes_workspaces.then(|| Box::new(native_backend(&cli)) as Box<dyn OutputBackend>);
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
    let workspace_sandbox = shared_sandbox.clone();
    let quiet = cli.quiet;
    let workspaces = Workspaces::new(BuildEnv {
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
        report: Box::new(move |text| {
            if !(quiet && is_chatter(text)) {
                eprintln!("{text}");
            }
        }),
        observer: None,
    });
    {
        let mut registry = workspaces.borrow_mut();
        registry.main_file = path.display().to_string();
        // What the target program's `get_build_options` reports before a metaprogram changes it.
        registry.main_overrides = MainOverrides {
            exe: cli.exe.clone().unwrap_or_default(),
            output_path: cli
                .output_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            no_backtrace_on_crash: cli.no_backtrace_on_crash,
        };
    }
    // `-check` and builds exit right after: tearing down the syntax trees and the program takes
    // about a tenth of a large check, and nothing observes it.
    let mut compiler = LeakOnExit::new(Compiler::new(options, fs), cli.mode != Mode::Interp);
    if let Some(host) = &sandbox {
        compiler.interp.host = Box::new(SharedHost(host.clone()));
    }
    compiler.attach_workspaces(workspaces.clone());
    // `-os wasm` / `-target wasm64-unknown-wasi`: a WASI command, with its runtime added.
    let wasi = cli.mode == Mode::Build
        && cli
            .target_triple()
            .ok()
            .flatten()
            .is_some_and(|t| jaic::build::wants_wasi_runtime(compiler.options.os, &t));
    if cli.verbose {
        let what = match cli.mode {
            Mode::Build => "building",
            Mode::Check => "checking",
            Mode::Interp => "interpreting",
        };
        eprintln!("jaic: {what} {}", shown(&path));
        if !cli.plugins.is_empty() {
            let names: Vec<&str> = cli.plugins.iter().map(|p| p.name.as_str()).collect();
            eprintln!("jaic: plugins: {}", names.join(", "));
        }
    }
    let added: Vec<ProgramSource> = cli
        .added
        .iter()
        .map(|a| ProgramSource::String(a.source()))
        .collect();
    let compiled = if cli.plugins.is_empty() {
        let mut sources = vec![ProgramSource::File(path.clone())];
        sources.extend(added);
        if wasi {
            sources.push(ProgramSource::String(
                jaic::build::WASI_RUNTIME_IMPORT.into(),
            ));
        }
        timings::time("front end", || compiler.compile_sources(&sources))
    } else {
        let source = plugin_metaprogram(&cli, &path);
        timings::time("front end", || {
            compiler.compile_sources(&[ProgramSource::String(source)])
        })
    };
    if cli.mode == Mode::Build {
        // What compile-time code of a wasm target printed (it runs in the sandbox).
        flush_sandbox(&shared_sandbox);
    }
    if !cli.quiet {
        for warning in &compiler.warnings {
            eprintln!("{}", compiler.render(warning));
        }
    }
    if let Err(d) = compiled {
        eprintln!("{}", compiler.render(&d));
        return ExitCode::from(1);
    }
    let finished = timings::time("workspaces", || {
        jaic::build::finish_all(&workspaces, &mut compiler.interp.block_budget)
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
    match cli.mode {
        Mode::Check => ExitCode::SUCCESS,
        Mode::Interp => {
            // The program sees itself as the executable a build would write next to its
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
        Mode::Build if !settings.do_output || settings.output_type == OutputType::NoOutput => {
            ExitCode::SUCCESS
        }
        Mode::Build
            if settings.output_type == OutputType::Executable
                && compiler.exported_func("main").is_none() =>
        {
            let file = shown(&path);
            print_error(&format!(
                "`{file}` has no `main` procedure, so there is no program to write\n\
                 help: add `main :: () {{ ... }}` as the program's starting point, or use `jaic {file} -check` to only check the code"
            ));
            ExitCode::from(NO_MAIN_STATUS)
        }
        Mode::Build => match build(&mut compiler, &cli, &path, settings) {
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

/// The metaprogram `+Plugin` stands for: import each plugin module, then compile `path` in a
/// workspace with their hooks (`build_with_plugin_groups` in `Metaprogram_Plugins`).
fn plugin_metaprogram(cli: &Cli, path: &Path) -> String {
    let quote = |s: &str| {
        let escaped = s
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r");
        format!("\"{escaped}\"")
    };
    let mut text = String::from(
        "Compiler :: #import \"Compiler\";\nPlugins :: #import \"Metaprogram_Plugins\";\n#import \"Basic\";\n",
    );
    for (i, plugin) in cli.plugins.iter().enumerate() {
        // `Name(PARAM=value)` passes module parameters along.
        let (name, params) = plugin
            .name
            .split_at(plugin.name.find('(').unwrap_or(plugin.name.len()));
        let params = if cli.no_check_bindings && name == "Check" {
            check_bindings_off(params)
        } else {
            params.to_string()
        };
        text += &format!("__plugin_{i} :: #import {}{params};\n", quote(name));
    }
    text += "#run,stallable {\n    plugins: [..] *Compiler.Metaprogram_Plugin;\n";
    for i in 0..cli.plugins.len() {
        text += &format!("    array_add(*plugins, __plugin_{i}.get_plugin());\n");
    }
    // Each plugin's own options, one after the other, with how many belong to each.
    text += "    options: [..] string;\n    counts: [..] s64;\n";
    for plugin in &cli.plugins {
        for option in &plugin.options {
            text += &format!("    array_add(*options, {});\n", quote(option));
        }
        text += &format!("    array_add(*counts, {});\n", plugin.options.len());
    }
    text += "    added: [..] string;\n";
    for added in &cli.added {
        text += &format!("    array_add(*added, {});\n", quote(&added.source()));
    }
    // `-exe` and `-output_path` name the executable; otherwise it is named after the file, next
    // to it.
    let lossy = |p: Option<&std::ffi::OsStr>| p.map(|n| n.to_string_lossy().into_owned());
    let target = match (&cli.exe, &cli.output_path) {
        (Some(exe), Some(dir)) => Some(dir.join(exe)),
        (Some(exe), None) => Some(PathBuf::from(exe)),
        (None, Some(dir)) => Some(dir.join(path.file_stem().unwrap_or_default())),
        (None, None) => None,
    };
    let (name, dir) = match &target {
        Some(out) => (
            lossy(out.file_name()),
            lossy(out.parent().map(|d| d.as_os_str())),
        ),
        None => (lossy(path.file_stem()), None),
    };
    text += &format!(
        "    Plugins.build_with_plugin_groups({}, plugins, options, counts, added, {}, {});\n}}\n",
        quote(&path.to_string_lossy()),
        quote(&name.unwrap_or_default()),
        quote(&dir.unwrap_or_default()),
    );
    text
}

/// Module parameters `params` (`""` or `(A=1,B=2)`) with `CHECK_BINDINGS=false` added, unless
/// they already say what `CHECK_BINDINGS` is.
fn check_bindings_off(params: &str) -> String {
    match params.strip_suffix(')') {
        _ if params.contains("CHECK_BINDINGS") => params.to_string(),
        Some(open) => format!("{open},CHECK_BINDINGS=false)"),
        None => "(CHECK_BINDINGS=false)".into(),
    }
}

/// Write the top-level program. `-exe`, `-output_path` and the optimization options override
/// what the metaprogram set.
fn build(
    compiler: &mut Compiler,
    cli: &Cli,
    source: &Path,
    mut settings: BuildSettings,
) -> Result<(), String> {
    let name = cli.exe.clone().unwrap_or_else(|| {
        if settings.output_executable_name.is_empty() {
            source
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        } else {
            settings.output_executable_name.clone()
        }
    });
    let dir = cli
        .output_path
        .clone()
        .unwrap_or_else(|| PathBuf::from(&settings.output_path));
    let output = dir.join(name);
    if let Some(level) = cli.opt_level.or(cli.flavor.map(Flavor::level)) {
        // `-O` and the flavors stand for a whole build: the metaprogram's code generation
        // switches go too.
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
    native_backend(cli).write_output(&compiler.program, &settings, &output)?;
    if cli.verbose {
        eprintln!("jaic: wrote {}", shown(&output));
    }
    Ok(())
}

#[cfg(feature = "llvm")]
fn native_backend(cli: &Cli) -> LlvmBackend {
    LlvmBackend {
        emit_ir: cli.emit_ir.clone(),
        debug_info: !cli.no_debug_info,
        target: cli.target_triple().ok().flatten(),
        sanitize: cli.sanitize.join(","),
        no_inline: cli.no_inline,
        no_split: cli.no_split,
    }
}

/// Without the `llvm` feature (an interpreter-only build, e.g. for a host without LLVM
/// libraries) a build reports that it cannot write native output.
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
    /// `-no_inline` and `-no_split`: they hold for every workspace this backend writes.
    no_inline: bool,
    no_split: bool,
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
                "the output path `{}` is a directory\nhelp: name the file to write inside it, as in `-exe {}`",
                shown(&output),
                shown(&name)
            ));
        }
        let create_dir = |dir: &Path, what: &str| {
            std::fs::create_dir_all(dir).map_err(|e| {
                format!(
                    "could not create the {what} directory `{}`: {}\nhelp: choose another place with `-output_path`",
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
            disable_inlining: if self.no_inline {
                Some(true)
            } else {
                switch("disable_inlining")
            },
            tail_calls: switch("enable_tail_calls"),
            split_modules: switch("enable_split_modules"),
            single_unit: self.no_split,
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
        // `-emit_ir` wins over `output_llvm_ir_before_optimizations`.
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
                "cannot write `{}`: {}\nhelp: choose a directory you can write to with `-output_path`",
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
        let mut argv = vec!["main.jai".to_string()];
        argv.extend(args.iter().map(|a| a.to_string()));
        let Ok(Request::Compile(cli)) = parse(&argv) else {
            panic!("invalid command line {argv:?}");
        };
        cli.target_triple_from(host_os, host_cpu).expect("a target")
    }

    fn args(words: &[&str]) -> Vec<String> {
        words.iter().map(|w| w.to_string()).collect()
    }

    fn cli(words: &[&str]) -> Cli {
        match parse(&args(words)) {
            Ok(Request::Compile(cli)) => *cli,
            Ok(_) => panic!("{words:?} asked for help or the version"),
            Err(e) => panic!("{words:?}: {} {:?}", e.message, e.help),
        }
    }

    fn error(words: &[&str]) -> CliError {
        match parse(&args(words)) {
            Err(e) => e,
            Ok(_) => panic!("{words:?} should be a command-line mistake"),
        }
    }

    #[test]
    fn the_mode_is_an_option_and_the_file_may_come_anywhere() {
        assert!(cli(&["a.jai"]).mode == Mode::Build);
        assert!(cli(&["a.jai", "-check"]).mode == Mode::Check);
        let interp = cli(&["-interp", "a.jai", "-no_dce"]);
        assert!(interp.mode == Mode::Interp);
        assert_eq!(interp.file, "a.jai");
        assert!(parse(&args(&["a.jai", "-check", "-interp"])).is_err());
        assert!(error(&["a.jai", "b.jai"]).message.contains("one file"));
    }

    #[test]
    fn dashes_split_metaprogram_and_program_arguments() {
        let c = cli(&["a.jai", "-interp", "-", "m1", "m2", "--", "p1"]);
        assert_eq!(c.command_line, ["m1", "m2"]);
        assert_eq!(c.program_args, ["a.jai", "p1"]);
        let c = cli(&["a.jai", "-", "-release", "-x"]);
        assert_eq!(c.command_line, ["-release", "-x"]);
        assert!(
            error(&["a.jai", "--", "p"])
                .message
                .contains("only `-interp`")
        );
    }

    #[test]
    fn plugins_take_the_arguments_up_to_the_next_plugin() {
        let c = cli(&[
            "a.jai",
            "-I",
            "d",
            "+One",
            "-x",
            "1",
            "+Two(A=1)",
            "-",
            "meta",
        ]);
        assert_eq!(c.imports, [PathBuf::from("d")]);
        assert_eq!(c.plugins.len(), 2);
        assert_eq!(c.plugins[0].name, "One");
        assert_eq!(c.plugins[0].options, ["-x", "1"]);
        assert_eq!(c.plugins[1].name, "Two(A=1)");
        assert!(c.plugins[1].options.is_empty());
        assert_eq!(c.command_line, ["meta"]);
        assert!(
            error(&["a.jai", "-interp", "+One"])
                .message
                .contains("not `-interp`")
        );
        // Before the first plugin an unknown option is jaic's mistake.
        assert!(
            error(&["a.jai", "-bogus", "+One"])
                .message
                .contains("unknown option")
        );
    }

    #[test]
    fn check_bindings_off_edits_module_parameters() {
        assert_eq!(check_bindings_off(""), "(CHECK_BINDINGS=false)");
        assert_eq!(check_bindings_off("(A=1)"), "(A=1,CHECK_BINDINGS=false)");
        assert_eq!(
            check_bindings_off("(CHECK_BINDINGS=true)"),
            "(CHECK_BINDINGS=true)"
        );
    }

    #[test]
    fn optimized_is_a_flavor_that_takes_no_value() {
        assert!(cli(&["a.jai", "-o"]).flavor == Some(Flavor::Optimized));
        assert!(cli(&["-od", "a.jai"]).flavor == Some(Flavor::OptimizedDebug));
        assert!(cli(&["-o", "a.jai"]).flavor == Some(Flavor::Optimized));
        assert!(
            error(&["a.jai", "-o", "out"])
                .message
                .contains("takes no value")
        );
        assert!(
            error(&["a.jai", "-o", "-very_debug"])
                .message
                .contains("flavor")
        );
        let c = cli(&["a.jai", "-exe", "build/app.exe", "-output_path", "out"]);
        assert_eq!(c.exe.as_deref(), Some("build/app.exe"));
        assert_eq!(c.output_path, Some(PathBuf::from("out")));
    }

    #[test]
    fn old_subcommands_get_the_new_command_back() {
        let e = error(&["run", "a.jai", "-I", "d", "--", "x"]);
        assert!(
            e.help[0].contains("`jaic a.jai -I d -interp -- x`"),
            "{:?}",
            e.help
        );
        let e = error(&["build", "a.jai", "-o", "out", "-plug", "Check"]);
        assert!(
            e.help[0].contains("`jaic a.jai -exe out +Check`"),
            "{:?}",
            e.help
        );
    }

    #[test]
    fn add_and_run_become_source_text() {
        let c = cli(&["a.jai", "-add", "x :: 1;", "-run", "f()", "-run", "g();"]);
        let text: Vec<String> = c.added.iter().map(Added::source).collect();
        assert_eq!(text[0], "x :: 1;");
        assert_eq!(text[1], "#run {\nf();\n};\n");
        assert_eq!(text[2], "#run {\ng();\n};\n");
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
