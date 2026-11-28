//! Parsed compiler settings, with no execution or backend initialization.
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use argdef::{Args, Shell};
use jaic::builtin_keys::BitcodeOptimization;
use jaic::sema::Options;
use jaic::target::{Target, TargetCpu, TargetOs};

use crate::options::{
    CliError, FLAVOR_CONFLICT, add_code, add_run, add_sanitizer, read_backend, read_cpu, read_os,
    read_package_import, set_color, store_level,
};
use crate::tuning;

/// What jaic does with the program: build it (the default), only compile it, or interpret it.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    Check,
    Interp,
    #[default]
    Build,
}

impl Mode {
    /// Resolve mode switches with the same conflicts for the compiler and its consumers.
    pub fn from_flags(check: bool, interp: bool, deps: bool) -> Result<Self, CliError> {
        if check && interp {
            return Err(CliError::new("`-check` and `-interp` cannot be combined")
                .help("`-check` only compiles the program; `-interp` runs it in the interpreter"));
        }

        if deps && interp {
            return Err(CliError::new("`-deps` and `-interp` cannot be combined")
                .help("`-deps` only compiles the program and lists the files it read"));
        }

        Ok(match (check || deps, interp) {
            (true, _) => Self::Check,
            (_, true) => Self::Interp,
            _ => Self::Build,
        })
    }
}

/// The optimization flavor `-o`, `-od` and `-very_debug` stand for.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Flavor {
    Optimized,
    OptimizedDebug,
    VeryDebug,
}

impl Flavor {
    /// The `llvm_options.bitcode_optimization_setting` this flavor builds with.
    pub fn level(self) -> BitcodeOptimization {
        match self {
            Flavor::Optimized => BitcodeOptimization::O2,
            Flavor::OptimizedDebug => BitcodeOptimization::O1,
            Flavor::VeryDebug => BitcodeOptimization::O0,
        }
    }
}

/// The code generator `-backend` picks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CodegenBackend {
    #[default]
    Llvm,
    /// Cranelift: unoptimized native builds only (`docs/native/cranelift-backend.md`).
    Cranelift,
}

/// A `-add` or `-run` string, in command-line order.
pub enum Added {
    /// `-add`: more code for the target program.
    Code(String),
    /// `-run`: code to run at compile time.
    Run(String),
}

impl Added {
    /// The source text this adds to the program.
    pub fn source(&self) -> String {
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
pub struct Plugin {
    /// The module name, without its module parameters.
    pub module: String,
    /// The text inside `Name(...)`: module parameters, `A=1,B=2`.
    parameters: Option<String>,
    /// The arguments after the plugin's name, up to the next `+Plugin` or boundary, as written.
    pub options: Vec<String>,
}

impl argdef::Group for Plugin {
    fn start(name: &str) -> Self {
        Plugin::parse(name)
    }

    fn push(&mut self, arg: &str) {
        self.options.push(arg.to_string());
    }
}

impl Plugin {
    /// Split a `Name` or `Name(A=1,B=2)` plugin argument.
    pub fn parse(spec: &str) -> Self {
        let (module, parameters) = match spec.split_once('(') {
            Some((module, rest)) => (module, Some(rest.strip_suffix(')').unwrap_or(rest))),
            None => (spec, None),
        };

        Plugin {
            module: module.to_string(),
            parameters: parameters.map(str::to_string),
            options: Vec::new(),
        }
    }

    /// Give module parameter `name` the value `value` unless the plugin already sets it.
    pub fn default_parameter(&mut self, name: &str, value: &str) {
        let set = self.parameters.as_deref().is_some_and(|parameters| {
            parameters.split(',').any(|assignment| {
                assignment
                    .split('=')
                    .next()
                    .is_some_and(|k| k.trim() == name)
            })
        });

        if !set {
            let assignment = format!("{name}={value}");

            match &mut self.parameters {
                Some(parameters) if !parameters.is_empty() => {
                    parameters.push(',');
                    parameters.push_str(&assignment);
                }
                parameters => *parameters = Some(assignment),
            }
        }
    }

    /// The module parameter list as it follows `#import "Name"`.
    pub fn import_parameters(&self) -> String {
        self.parameters
            .as_ref()
            .map(|parameters| format!("({parameters})"))
            .unwrap_or_default()
    }
}

impl Cli {
    /// The metaprogram `+Plugin` stands for, compiling `path`.
    pub fn plugin_metaprogram(&self, path: &Path, wasi: bool) -> String {
        let plugins: Vec<jaic::build::PluginImport> = self
            .plugins
            .iter()
            .map(|p| jaic::build::PluginImport {
                module: p.module.clone(),
                import_parameters: p.import_parameters(),
                options: p.options.clone(),
            })
            .collect();
        let added: Vec<String> = self.added.iter().map(Added::source).collect();

        jaic::build::plugin_metaprogram(&jaic::build::PluginProgram {
            plugins: &plugins,
            main_file: path,
            exe: self.exe.as_deref(),
            output_path: self.output_path.as_deref(),
            added_sources: &added,
            wasi,
        })
    }
}

/// compile, check and run Jai programs
///
/// By default jaic builds a native executable. Options may come before or after the file.
// The doc comments are `-help` text: names and variables stay unquoted.
#[allow(clippy::doc_markdown, clippy::duplicated_attributes)]
#[derive(Default, Args)]
#[args(
    name = "jaic",
    version = env!("CARGO_PKG_VERSION"),
    usage = "jaic [options] file.jai [options] [+Plugin [plugin options...]]... [- metaprogram args...] [-- program args...]",
    sections = ["Modes", "Program", "Output (building only)", "Plugins", "Diagnostics and output"],
)]
pub struct Cli {
    // `-check`, `-interp` and `-deps` are read into `mode` by `Mode::from_flags`.
    /// compile the program without running or writing anything
    #[arg(name = "-check", section = "Modes", tag = "check")]
    pub check: bool,
    /// run `main` in the interpreter (arguments after `--` go to the program)
    #[arg(name = "-interp", tag = "interp")]
    pub interp: bool,
    /// like -check, then print every source file the compile read, one per line
    #[arg(name = "-deps", tag = "deps")]
    pub deps: bool,
    /// What `-check`, `-interp` and `-deps` resolve to; the caller sets it after parsing, with
    /// [`Mode::from_flags`].
    #[arg(skip)]
    pub mode: Mode,
    /// The program's source file; the caller sets it after parsing.
    #[arg(skip)]
    pub file: String,
    /// also look for modules in <dir> (repeatable; short form -I)
    #[arg(
        name = "-import_dir",
        name = "-I",
        synopsis = "-import_dir <dir>",
        value = "dir",
        needs = "a directory",
        hint = "dir",
        tag = "import",
        section = "Program"
    )]
    pub imports: Vec<PathBuf>,
    /// also look for modules in <dir>, for the files under <package> only (repeatable)
    #[arg(name = "-package_import_dir", synopsis = "-package_import_dir <package>=<dir>", needs = "a package folder and a directory, as <package>=<dir>", parse = read_package_import)]
    pub package_imports: Vec<(PathBuf, PathBuf)>,
    // `-exe` is the target workspace's `output_executable_name`: it may carry a directory and
    // an extension.
    /// the executable's name; it may include a directory and extension
    /// (`-exe build/app.exe`). A bare name goes in the current directory
    #[arg(
        name = "-exe",
        value = "name",
        needs = "an executable name",
        hint = "file",
        section = "Output (building only)"
    )]
    pub exe: Option<String>,
    // The target workspace's `output_path`.
    /// the directory the executable is written to
    #[arg(
        name = "-output_path",
        value = "dir",
        needs = "a directory",
        hint = "dir"
    )]
    pub output_path: Option<PathBuf>,
    // `-o`, `-od` and `-very_debug` each pick a flavor, so they exclude one another.
    /// optimized build (LLVM O2) without stack traces or runtime checks
    #[arg(name = "-o", name = "-optimized", set = Flavor::Optimized, conflict = FLAVOR_CONFLICT)]
    #[arg(name = "-od", name = "-optimized_debug", set = Flavor::OptimizedDebug, conflict = FLAVOR_CONFLICT, help = "lightly optimized build (LLVM O1) that keeps stack traces and checks")]
    #[arg(name = "-very_debug", set = Flavor::VeryDebug, conflict = FLAVOR_CONFLICT, help = "unoptimized build with every runtime check on")]
    pub flavor: Option<Flavor>,
    // `-O0`..`-O3`; `None`: what the metaprogram chose (default: unoptimized).
    /// pick the LLVM optimization level directly
    #[arg(name = "-O0", set = BitcodeOptimization::O0, synopsis = "-O0, -O1, -O2, -O3")]
    #[arg(name = "-O1", set = BitcodeOptimization::O1)]
    #[arg(name = "-O2", set = BitcodeOptimization::O2)]
    #[arg(name = "-O3", set = BitcodeOptimization::O3)]
    pub opt_level: Option<BitcodeOptimization>,
    /// also write the LLVM IR
    #[arg(
        name = "-emit_ir",
        value = "file.ll",
        needs = "a file to write the LLVM IR to",
        hint = "file"
    )]
    pub emit_ir: Option<PathBuf>,
    /// leave out native debug information
    #[arg(name = "-no_debug_info")]
    pub no_debug_info: bool,
    /// do not inline procedures
    #[arg(name = "-no_inline")]
    pub no_inline: bool,
    /// generate code as one module instead of several
    #[arg(name = "-no_split")]
    pub no_split: bool,
    /// the code generator: llvm (the default) or cranelift, an experimental one for faster
    /// unoptimized builds; what it cannot build goes to llvm (docs/native/cranelift-backend.md)
    #[arg(
        name = "-backend",
        value = "name",
        needs = "a code generator: llvm or cranelift",
        parse = read_backend
    )]
    pub backend: Option<CodegenBackend>,
    /// `-codegen_units`, `-parse_threads`, `-profile` and the `-dev_*` flags (`tuning.rs`).
    #[arg(flatten)]
    pub tuning: tuning::Tuning,
    /// leave the crash handler out of the program
    #[arg(name = "-no_backtrace_on_crash")]
    pub no_backtrace_on_crash: bool,
    /// install the crash handler (a metaprogram's setting wins; -no_backtrace_on_crash overrides)
    #[arg(name = "-backtrace_on_crash")]
    pub backtrace_on_crash: bool,
    /// add the code to the target program (repeatable)
    #[arg(name = "-add", alias = "--add", synopsis = "-add \"code\"", needs = "the code to add to the program", with = add_code, section = "Program")]
    #[arg(name = "-run", synopsis = "-run \"code\"", needs = "the code to run at compile time", with = add_run, help = "run the code at compile time, as a `#run` (repeatable)")]
    pub added: Vec<Added>,
    #[arg(skip)]
    /// Arguments after `-`, for the metaprogram (`compiler_get_command_line`).
    pub command_line: Vec<String>,
    #[arg(skip)]
    /// `-interp` only: arguments after `--`, for the program (`get_command_line_arguments`).
    pub program_args: Vec<String>,
    /// target OS: linux, windows, macos or wasm
    #[arg(name = "-os", value = "os", needs = "an OS name: linux, windows, macos or wasm", parse = read_os, tag = "target")]
    pub os: Option<TargetOs>,
    /// target CPU: x64 or arm64
    #[arg(name = "-cpu", value = "cpu", needs = "a CPU name: x64 or arm64", parse = read_cpu, tag = "target")]
    pub cpu: Option<TargetCpu>,
    /// build for an LLVM target triple (overrides -os and -cpu)
    #[arg(
        name = "-target",
        value = "triple",
        needs = "an LLVM target triple, such as x86_64-pc-windows-gnu",
        tag = "target"
    )]
    pub target: Option<String>,
    /// What `-os`, `-cpu` and `-target` select, checked by [`Cli::finish`]; `None`: the host.
    #[arg(skip)]
    pub build_for: Option<Target>,
    /// run the metaprogram plugin module Name. Every argument after it, up
    /// to the next +Name, `-` or `--`, is that plugin's own option.
    /// Module parameters go in the name: '+Check(CHECK_BINDINGS=false)'
    /// (quote it in a shell). With -interp the program they compile is run.
    #[arg(group = "+", synopsis = "+Name", section = "Plugins")]
    pub plugins: Vec<Plugin>,
    /// the rest is for the metaprogram (`compiler_get_command_line`)
    #[arg(rest = "-", until = "--", synopsis = "-")]
    pub after_dash: Option<Vec<String>>,
    /// the rest is for the program (-interp only)
    #[arg(rest = "--", synopsis = "--")]
    pub after_dashes: Option<Vec<String>>,
    /// report the wall time of each phase on stderr
    #[arg(name = "-timings", section = "Diagnostics and output")]
    pub timings: bool,
    /// link with this program instead of the one found for the target (also JAIC_LINKER)
    #[arg(
        name = "-linker",
        value = "program",
        needs = "a program",
        hint = "file",
        os,
        section = "Output (building only)"
    )]
    pub linker: Option<OsString>,
    /// the C library folder a cross link uses (also JAIC_SYSROOT)
    #[arg(
        name = "-sysroot",
        value = "dir",
        needs = "a directory",
        hint = "dir",
        os
    )]
    pub sysroot: Option<PathBuf>,
    /// the compiler builtins archive a freestanding link uses (also JAIC_COMPILER_RT)
    #[arg(
        name = "-compiler_rt",
        value = "file",
        needs = "a file",
        hint = "file",
        os
    )]
    pub compiler_rt: Option<PathBuf>,
    /// an argument for the linker, in `ld.lld`'s spelling; repeat it for each argument.
    /// It is translated for the linker in use (`--gc-sections` is `/OPT:REF` for `link.exe`)
    #[arg(
        name = "-link_arg",
        value = "argument",
        needs = "a linker argument",
        os
    )]
    pub link_args: Vec<OsString>,
    /// address, undefined and/or fuzzer, comma-separated
    #[arg(name = "-sanitize", value = "list", needs = "address, undefined and/or fuzzer, comma-separated", with = add_sanitizer)]
    pub sanitize: Vec<String>,
    /// record line and function coverage (docs/tools/coverage.md)
    #[arg(name = "-coverage")]
    pub coverage: bool,
    /// after the compile, write the source files it read to the file, as -deps lists them
    #[arg(
        name = "-deps_file",
        value = "file",
        needs = "a file name",
        hint = "file"
    )]
    pub deps_file: Option<PathBuf>,
    /// type-check unreferenced module code too
    #[arg(name = "-no_dce", section = "Program")]
    pub no_dce: bool,
    // Check is not run unless asked for, so there is nothing to turn off.
    /// accepted and ignored: jaic does not run the Check plugin unless asked
    #[arg(name = "-no_check")]
    pub no_check: bool,
    /// set CHECK_BINDINGS=false on a +Check plugin, if there is one
    #[arg(name = "-no_check_bindings")]
    pub no_check_bindings: bool,
    /// (-interp only) do not write what a metaprogram's workspaces ask for
    #[arg(name = "-no_workspace_output", section = "Output (building only)")]
    pub no_workspace_output: bool,
    /// no colour in diagnostics
    #[arg(name = "-no_color", section = "Diagnostics and output")]
    #[arg(name = "--color", value = "when", needs = "auto, always or never", with = set_color, help = "colour in diagnostics: auto, always or never")]
    pub no_color: bool,
    /// one `path(line,col): error: message` line per diagnostic
    #[arg(name = "-msvc_format")]
    pub msvc_format: bool,
    // `-A`, `-W` and `-D` as written, in order; `levels` is what they make of each warning.
    /// allow a warning: print nothing (`warnings` for all, `leniencies` for the forms the
    /// official compiler rejects; `jacket explain` lists the codes)
    #[arg(name = "-A", value = "code", needs = "a warning code, `leniencies` or `warnings`", with = store_level, attached)]
    #[arg(name = "-W", value = "code", needs = "a warning code, `leniencies` or `warnings`", with = store_level, attached, help = "warn about a code (the default)")]
    #[arg(name = "-D", value = "code", needs = "a warning code, `leniencies` or `warnings`", with = store_level, attached, help = "deny a warning: print it as an error and fail (`-D warnings` for all)")]
    pub level_flags: Vec<(String, jaic::diagnostics::Level)>,
    /// What `level_flags` make of each warning; the caller sets it after parsing.
    #[arg(skip)]
    pub levels: jaic::diagnostics::Levels,
    /// print nothing but errors (no build status either)
    #[arg(name = "-quiet", alias = "--quiet")]
    pub quiet: bool,
    /// say what is being done
    #[arg(name = "-verbose")]
    pub verbose: bool,
    /// print this help
    #[arg(
        name = "-help",
        alias = "-h",
        alias = "--help",
        synopsis = "-h, -help, --help",
        exit
    )]
    pub help: bool,
    /// print the version
    #[arg(
        name = "-version",
        alias = "-V",
        alias = "--version",
        synopsis = "-V, -version, --version",
        exit
    )]
    pub version: bool,
    /// print the standard library folder jaic uses and exit
    #[arg(name = "-print_stdlib", exit)]
    pub print_stdlib: bool,
    /// print a completion script for bash, zsh or fish, and exit
    #[arg(
        name = "-completions",
        value = "shell",
        needs = "bash, zsh or fish",
        exit
    )]
    pub completions: Option<Shell>,
    /// print the doc comments of the library in <dir> as records, for `jacket test`; -import_dir folders are searched for the modules it imports
    #[arg(name = "-doc_comments", synopsis = "-doc_comments <dir>")]
    pub doc_comments: bool,
    /// print which public declarations of the library in <dir> have docs and an example; -import_dir folders are searched for the modules it imports
    #[arg(name = "-doc_gaps", synopsis = "-doc_gaps <dir>")]
    pub doc_gaps: bool,
    /// print the documentation of the library in <dir> (items, signatures, docs) as JSON, with what its public top-level `#insert`s declare
    /// (compiled once with default module parameters), for `jacket doc`
    #[arg(name = "-doc_model", synopsis = "-doc_model <dir>")]
    pub doc_model: bool,
    /// print every procedure with a body below the folders <dirs> (a path list) as records,
    /// for `jacket test --coverage`; `-os` and `-cpu` pick the `#if` branches
    #[arg(name = "-procedures", synopsis = "-procedures <dirs>")]
    pub procedures: bool,
    #[arg(positional)]
    pub positional: Vec<String>,
}

impl Cli {
    /// Check the parsed options against each other and the mode, and apply the ones that
    /// change plugins. `extra` is the first argument after the file. Options that shape the
    /// output (`-exe`, `-O2`, `-emit_ir`, ...) are accepted and ignored by `-check` and
    /// `-interp`, which write none, so one command line serves all three.
    pub fn finish(&mut self, extra: Option<String>) -> Result<(), CliError> {
        if let Some(word) = extra {
            let mut error = if word.ends_with(".jai") {
                CliError::new(format!(
                    "jaic compiles one file, but got `{}` and `{word}`",
                    self.file
                ))
                .help("`#load` or `#import` the others from the first")
            } else {
                CliError::new(format!("unexpected argument `{word}`"))
            };

            if self.mode == Mode::Interp {
                error = error.help(format!(
                    "arguments for the program go after `--`: `jaic {} -interp -- {word}`",
                    self.file
                ));
            }

            return Err(error);
        }

        if self.no_workspace_output && self.mode != Mode::Interp {
            return Err(
                CliError::new("`-no_workspace_output` only applies to `-interp`").help(
                    if self.mode == Mode::Check {
                        "`-check` never writes workspace output"
                    } else {
                        "a build writes what the workspaces ask for; leave the option out"
                    },
                ),
            );
        }

        if self.opt_level.is_some() && self.flavor.is_some() {
            return Err(CliError::new(
                "`-O0`..`-O3` and `-o`, `-od` or `-very_debug` both set the optimization",
            )
            .help("use one of them"));
        }

        if self.quiet && self.verbose {
            return Err(CliError::new(
                "`-quiet` and `-verbose` contradict each other",
            ));
        }

        self.build_for =
            self.build_target()
                .map_err(|message| match message.split_once("\nhelp: ") {
                    Some((message, help)) => CliError::new(message).help(help),
                    None => CliError::new(message),
                })?;
        Ok(())
    }

    /// What to build for, `None` for the host. `-os windows` on another host cross-compiles
    /// with MinGW-w64 (x64 unless `-cpu arm64`); other cross targets need an explicit
    /// `-target`.
    pub fn build_target(&self) -> Result<Option<Target>, String> {
        self.build_target_from(Options::host().os, Options::host().cpu)
    }

    /// Like [`build_target`](Cli::build_target) for a given host, instead of the running one.
    pub fn build_target_from(
        &self,
        host_os: TargetOs,
        host_cpu: TargetCpu,
    ) -> Result<Option<Target>, String> {
        jaic::target::Selection {
            os: self.os,
            cpu: self.cpu,
            triple: self.target.clone(),
        }
        .build_target(host_os, host_cpu, self.mode == Mode::Build)
    }
}
