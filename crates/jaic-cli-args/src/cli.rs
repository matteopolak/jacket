//! Parsed compiler settings, with no execution or backend initialization.
use crate::options::CliError;
use crate::tuning;
use jaic::builtin_keys::BitcodeOptimization;
use jaic::sema::Options;
use jaic::target::{Target, TargetCpu, TargetOs};
use std::path::{Path, PathBuf};

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
    pub options: Vec<String>,
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

/// Parsed command line.
#[derive(Default)]
pub struct Cli {
    pub mode: Mode,
    pub file: String,
    pub imports: Vec<PathBuf>,
    /// `-exe`: the target workspace's `output_executable_name`. It may carry a directory and an
    /// extension.
    pub exe: Option<String>,
    /// `-output_path`: the target workspace's `output_path`.
    pub output_path: Option<PathBuf>,
    /// `-o`, `-od` or `-very_debug`.
    pub flavor: Option<Flavor>,
    /// `-O0`..`-O3`; `None`: what the metaprogram chose (default: unoptimized).
    pub opt_level: Option<BitcodeOptimization>,
    pub emit_ir: Option<PathBuf>,
    /// `-no_debug_info`: omit native debug information (on by default).
    pub no_debug_info: bool,
    /// `-no_inline`: every function not marked `inline` gets `noinline`.
    pub no_inline: bool,
    /// `-no_split`: generate code in one module instead of several.
    pub no_split: bool,
    /// `-codegen_units`, `-parse_threads`, `-profile` and the `-dev_*` flags (`tuning.rs`).
    pub tuning: tuning::Tuning,
    /// `-no_backtrace_on_crash`: no crash handler in the target program.
    pub no_backtrace_on_crash: bool,
    /// `-backtrace_on_crash`: install the crash handler without a metaprogram asking for it.
    pub backtrace_on_crash: bool,
    /// `-add` and `-run` strings.
    pub added: Vec<Added>,
    /// Arguments after `-`, for the metaprogram (`compiler_get_command_line`).
    pub command_line: Vec<String>,
    /// `-interp` only: arguments after `--`, for the program (`get_command_line_arguments`).
    pub program_args: Vec<String>,
    /// `-os`: the target `OS` when it is not the host (checking code for another platform).
    pub os: Option<TargetOs>,
    /// `-cpu`: the target `CPU` when it is not the host's (`-os windows -cpu arm64`).
    pub cpu: Option<TargetCpu>,
    /// `+Name`: metaprogram plugin modules, each with the arguments up to the next `+Name`.
    pub plugins: Vec<Plugin>,
    /// `-target`: an explicit LLVM target triple (`x86_64-pc-windows-msvc`...).
    pub target: Option<String>,
    /// `-timings`: report the wall time of each phase on stderr.
    pub timings: bool,
    /// `-sanitize address,undefined` (repeatable): sanitizer instrumentation for builds.
    pub sanitize: Vec<String>,
    /// `-coverage`: line and function coverage instrumentation (docs/tools/coverage.md).
    pub coverage: bool,
    /// `-deps`: check the program, then print every source file the compile read.
    pub deps: bool,
    /// `-no_dce`: type-check every declaration, modules included
    /// (`Build_Options.dead_code_elimination = .NONE`).
    pub no_dce: bool,
    /// `-no_check_bindings`: `CHECK_BINDINGS=false` for a `+Check` plugin.
    pub no_check_bindings: bool,
    /// `-no_workspace_output` (`-interp`): do not write what workspaces a metaprogram creates
    /// ask for (test sweeps over many programs).
    pub no_workspace_output: bool,
    /// `-no_color`: no colour in diagnostics.
    pub no_color: bool,
    /// `-msvc_format`: diagnostics as `path(line,col): error: message` lines.
    pub msvc_format: bool,
    /// `-A`, `-W` and `-D` as written, in order.
    pub level_flags: Vec<(String, jaic::diagnostics::Level)>,
    /// What those flags make of each warning.
    pub levels: jaic::diagnostics::Levels,
    /// `-quiet`: print nothing but errors.
    pub quiet: bool,
    /// `-verbose`: say what is being done.
    pub verbose: bool,
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
        Ok(())
    }

    /// Fold a flag shared with the other tools into the command line.
    pub fn take_shared(&mut self, flag: jaic::flags::Shared) {
        use jaic::flags::Shared;
        match flag {
            Shared::Os(os) => self.os = Some(os),
            Shared::Cpu(cpu) => self.cpu = Some(cpu),
            Shared::Triple(triple) => self.target = Some(triple),
            Shared::ImportDir(dir) => self.imports.push(dir),
            Shared::Add(code) => self.added.push(Added::Code(code)),
            // 0 leaves the pool at one thread per core, as for no flag.
            Shared::Jobs(n) => self.tuning.threads = (n > 0).then_some(n),
            Shared::Color(choice) => self.no_color = choice == jaic::render::ColorChoice::Never,
        }
    }

    /// What to build for, `None` for the host. `-os windows` on another host cross-compiles
    /// with MinGW-w64 (x64 unless `-cpu arm64`); other cross targets need an explicit
    /// `-target`.
    pub fn build_target(&self) -> Result<Option<Target>, String> {
        self.build_target_from(Options::host().os, Options::host().cpu)
    }

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
