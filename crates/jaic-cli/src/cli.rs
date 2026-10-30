//! The parsed command line: modes, flavors, plugins and settings.
use super::options::CliError;
use super::{crash_runtime, tuning};
use jaic::builtin_keys::BitcodeOptimization;
use jaic::sema::{Options, TargetCpu, TargetOs};
use jaic::target::Target;
use std::path::{Path, PathBuf};

/// Where `tools/native/build_native_libs.py` (or a release package) puts third-party libraries for this
/// host, or the `JAIC_NATIVE_LIBS` path list.
pub(super) fn native_lib_dirs(stdlib: &Path) -> Vec<PathBuf> {
    // The crash runtime is built into jaic itself (docs/native/crash-runtime-build.md). It comes
    // first so an older copy left in a configured folder never shadows the one this stdlib needs;
    // the folder holds only those runtimes, so no other library is affected.
    let mut dirs: Vec<PathBuf> = crash_runtime::dir().into_iter().collect();
    dirs.extend(configured_native_lib_dirs(stdlib));
    dirs.extend(apple_toolchain_lib_dir());
    dirs
}

/// The Xcode (or Command Line Tools) toolchain's library folder on a Mac, when it holds
/// `libclang.dylib`: the `Bindings_Generator` parses headers with it, and the release archive does
/// not carry a second copy (the `Bindings_Generator` section of docs/tools/releases.md). Only a folder with
/// that library is searched, so the toolchain's other libraries are never picked up by name.
fn apple_toolchain_lib_dir() -> Option<PathBuf> {
    if !cfg!(target_os = "macos") {
        return None;
    }
    let selected = jacket_config::env::DEVELOPER_DIR.os().map(PathBuf::from);
    let candidates = selected
        .iter()
        .flat_map(|dir| {
            [
                dir.join("Toolchains/XcodeDefault.xctoolchain/usr/lib"),
                dir.join("usr/lib"),
            ]
        })
        .chain([
            PathBuf::from("/Library/Developer/CommandLineTools/usr/lib"),
            PathBuf::from(
                "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib",
            ),
        ]);
    candidates
        .into_iter()
        .find(|dir| dir.join("libclang.dylib").is_file())
}

fn configured_native_lib_dirs(stdlib: &Path) -> Vec<PathBuf> {
    if let Some(dirs) = jacket_config::env::NATIVE_LIBS.os() {
        return std::env::split_paths(&dirs).collect();
    }
    let host = Target::host();
    if host.is_wasm() {
        return Vec::new();
    }
    let platform = format!(
        "{}-{}",
        jaic::target::os_name(host.os),
        jaic::target::cpu_name(host.cpu)
    );
    // `JACKET_OUTPUT_DIR` (tools/repo_output.py) relocates the build output directory; a
    // release package carries `native-libs/` beside `stdlib/`, a checkout builds them under
    // `target/`.
    let mut candidates = Vec::new();
    if let Some(output) = jacket_config::env::OUTPUT_DIR.os() {
        candidates.push(PathBuf::from(output).join("native-libs").join(&platform));
    }
    candidates.push(stdlib.join("../native-libs").join(&platform));
    candidates.push(stdlib.join("../target/native-libs").join(&platform));
    // A linked git worktree shares the main checkout's builds, where `build_native_libs.py` puts
    // them (`tools/repo_output.py`).
    candidates.extend(
        main_checkout(&stdlib.join(".."))
            .map(|main| main.join("target/native-libs").join(&platform)),
    );
    candidates
        .iter()
        .find_map(|dir| jaic::canonicalize(dir).ok())
        .map(|d| vec![d])
        .unwrap_or_default()
}

/// The main checkout of the repository whose linked worktree is `root`: `.git` there is a file
/// naming the worktree's git folder, whose `commondir` leads to the main checkout's `.git`.
fn main_checkout(root: &Path) -> Option<PathBuf> {
    let link = std::fs::read_to_string(root.join(".git")).ok()?;
    let git_dir = PathBuf::from(link.trim().strip_prefix("gitdir:")?.trim());
    let git_dir = if git_dir.is_absolute() {
        git_dir
    } else {
        root.join(git_dir)
    };
    let common = std::fs::read_to_string(git_dir.join("commondir")).ok()?;
    let common = jaic::canonicalize(git_dir.join(common.trim())).ok()?;
    common.parent().map(Path::to_path_buf)
}

/// What jaic does with the program: build it (the default), only compile it, or interpret it.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum Mode {
    Check,
    Interp,
    #[default]
    Build,
}

/// The optimization flavor `-o`, `-od` and `-very_debug` stand for.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Flavor {
    Optimized,
    OptimizedDebug,
    VeryDebug,
}

impl Flavor {
    /// The `llvm_options.bitcode_optimization_setting` this flavor builds with.
    pub(super) fn level(self) -> BitcodeOptimization {
        match self {
            Flavor::Optimized => BitcodeOptimization::O2,
            Flavor::OptimizedDebug => BitcodeOptimization::O1,
            Flavor::VeryDebug => BitcodeOptimization::O0,
        }
    }
}

/// A `-add` or `-run` string, in command-line order.
pub(super) enum Added {
    /// `-add`: more code for the target program.
    Code(String),
    /// `-run`: code to run at compile time.
    Run(String),
}

impl Added {
    /// The source text this adds to the program.
    pub(super) fn source(&self) -> String {
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
pub(super) struct Plugin {
    /// The module name, without its module parameters.
    pub(super) module: String,
    /// The text inside `Name(...)`: module parameters, `A=1,B=2`.
    parameters: Option<String>,
    pub(super) options: Vec<String>,
}

impl Plugin {
    /// Split a `Name` or `Name(A=1,B=2)` plugin argument.
    pub(super) fn parse(spec: &str) -> Self {
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
    pub(super) fn default_parameter(&mut self, name: &str, value: &str) {
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
    pub(super) fn import_parameters(&self) -> String {
        self.parameters
            .as_ref()
            .map(|parameters| format!("({parameters})"))
            .unwrap_or_default()
    }
}

impl Cli {
    /// The metaprogram `+Plugin` stands for, compiling `path`.
    pub(super) fn plugin_metaprogram(&self, path: &Path, wasi: bool) -> String {
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
pub(super) struct Cli {
    pub(super) mode: Mode,
    pub(super) file: String,
    pub(super) imports: Vec<PathBuf>,
    /// `-exe`: the target workspace's `output_executable_name`. It may carry a directory and an
    /// extension.
    pub(super) exe: Option<String>,
    /// `-output_path`: the target workspace's `output_path`.
    pub(super) output_path: Option<PathBuf>,
    /// `-o`, `-od` or `-very_debug`.
    pub(super) flavor: Option<Flavor>,
    /// `-O0`..`-O3`; `None`: what the metaprogram chose (default: unoptimized).
    pub(super) opt_level: Option<BitcodeOptimization>,
    pub(super) emit_ir: Option<PathBuf>,
    /// `-no_debug_info`: omit native debug information (on by default).
    pub(super) no_debug_info: bool,
    /// `-no_inline`: every function not marked `inline` gets `noinline`.
    pub(super) no_inline: bool,
    /// `-no_split`: generate code in one module instead of several.
    pub(super) no_split: bool,
    /// `-codegen_units`, `-parse_threads`, `-profile` and the `-dev_*` flags (`tuning.rs`).
    pub(super) tuning: tuning::Tuning,
    /// `-no_backtrace_on_crash`: no crash handler in the target program.
    pub(super) no_backtrace_on_crash: bool,
    /// `-add` and `-run` strings.
    pub(super) added: Vec<Added>,
    /// Arguments after `-`, for the metaprogram (`compiler_get_command_line`).
    pub(super) command_line: Vec<String>,
    /// `-interp` only: arguments after `--`, for the program (`get_command_line_arguments`).
    pub(super) program_args: Vec<String>,
    /// `-os`: the target `OS` when it is not the host (checking code for another platform).
    pub(super) os: Option<TargetOs>,
    /// `-cpu`: the target `CPU` when it is not the host's (`-os windows -cpu arm64`).
    pub(super) cpu: Option<TargetCpu>,
    /// `+Name`: metaprogram plugin modules, each with the arguments up to the next `+Name`.
    pub(super) plugins: Vec<Plugin>,
    /// `-target`: an explicit LLVM target triple (`x86_64-pc-windows-msvc`...).
    pub(super) target: Option<String>,
    /// `-timings`: report the wall time of each phase on stderr.
    pub(super) timings: bool,
    /// `-sanitize address,undefined` (repeatable): sanitizer instrumentation for builds.
    pub(super) sanitize: Vec<String>,
    /// `-coverage`: line and function coverage instrumentation (docs/tools/coverage.md).
    pub(super) coverage: bool,
    /// `-deps`: check the program, then print every source file the compile read.
    pub(super) deps: bool,
    /// `-no_dce`: type-check every declaration, modules included
    /// (`Build_Options.dead_code_elimination = .NONE`).
    pub(super) no_dce: bool,
    /// `-no_check_bindings`: `CHECK_BINDINGS=false` for a `+Check` plugin.
    pub(super) no_check_bindings: bool,
    /// `-no_workspace_output` (`-interp`): do not write what workspaces a metaprogram creates
    /// ask for (test sweeps over many programs).
    pub(super) no_workspace_output: bool,
    /// `-no_color`: no colour in diagnostics.
    pub(super) no_color: bool,
    /// `-msvc_format`: diagnostics as `path(line,col): error: message` lines.
    pub(super) msvc_format: bool,
    /// `-quiet`: print nothing but errors.
    pub(super) quiet: bool,
    /// `-verbose`: say what is being done.
    pub(super) verbose: bool,
}

impl Cli {
    /// Check the parsed options against each other and the mode, and apply the ones that
    /// change plugins. `extra` is the first argument after the file. Options that shape the
    /// output (`-exe`, `-O2`, `-emit_ir`, ...) are accepted and ignored by `-check` and
    /// `-interp`, which write none, so one command line serves all three.
    pub(super) fn finish(&mut self, extra: Option<String>) -> Result<(), CliError> {
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

    /// What to build for, `None` for the host. `-os windows` on another host cross-compiles
    /// with MinGW-w64 (x64 unless `-cpu arm64`); other cross targets need an explicit
    /// `-target`.
    /// Fold a flag shared with the other tools into the command line.
    pub(super) fn take_shared(&mut self, flag: jaic::flags::Shared) {
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

    pub(super) fn build_target(&self) -> Result<Option<Target>, String> {
        self.build_target_from(Options::host().os, Options::host().cpu)
    }

    pub(super) fn build_target_from(
        &self,
        host_os: TargetOs,
        host_cpu: TargetCpu,
    ) -> Result<Option<Target>, String> {
        if let Some(triple) = &self.target {
            return Target::parse(triple).map(Some);
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
            TargetOs::Windows => Ok(Some(Target::windows(
                self.cpu.unwrap_or(TargetCpu::X64),
                host_os == TargetOs::Windows,
            ))),
            _ if self.mode != Mode::Build => Ok(None),
            // A WASI command, runnable by node, wasmtime and the like.
            TargetOs::Wasm => Ok(Some(Target::wasi())),
            _ => Err(
                "native cross-compilation is only supported for -os windows and -os wasm; pass -target <triple> for others"
                    .into(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_linked_worktree_finds_the_main_checkout() {
        let base = std::env::temp_dir().join(format!("jaic-worktree-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let git = base.join("main/.git");
        let admin = git.join("worktrees/w");
        std::fs::create_dir_all(&admin).unwrap();
        std::fs::create_dir_all(base.join("main/.claude/worktrees/w")).unwrap();
        std::fs::write(admin.join("commondir"), "../..\n").unwrap();
        let worktree = base.join("main/.claude/worktrees/w");
        std::fs::write(
            worktree.join(".git"),
            format!("gitdir: {}\n", admin.display()),
        )
        .unwrap();
        let main = main_checkout(&worktree).unwrap();
        assert_eq!(main, jaic::canonicalize(base.join("main")).unwrap());
        // The main checkout's `.git` is a folder, and a plain folder has none.
        assert_eq!(main_checkout(&base.join("main")), None);
        assert_eq!(main_checkout(&base), None);
        let _ = std::fs::remove_dir_all(&base);
    }
}
