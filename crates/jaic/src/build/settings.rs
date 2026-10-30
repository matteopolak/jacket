//! Build settings, the output backend interface, reports and the observer of workspaces.
use crate::builtin_keys::OutputType;
use crate::builtin_keys::{
    BitcodeOptimization, MachineCodeOptimization, RuntimeSupport, WorkspaceFlag,
};
use crate::interp::Host;
use crate::ir;
use crate::sema::{Compiler, DeadCode, FileSystem, Options, ProgramSource};
use crate::target::{TargetCpu, TargetOs};
use std::path::PathBuf;
use std::rc::Rc;

/// Build settings of one workspace, as far as jaic uses them.
#[derive(Clone, Debug)]
pub struct BuildSettings {
    pub output_type: OutputType,
    /// `do_output = false` (from `set_build_options_dc`) suppresses output.
    pub do_output: bool,
    pub output_executable_name: String,
    pub output_path: String,
    /// `None`: the embedder's default import path.
    pub import_paths: Option<Vec<PathBuf>>,
    pub os: Option<TargetOs>,
    pub cpu: Option<TargetCpu>,
    /// `llvm_options.bitcode_optimization_setting`.
    pub optimization: BitcodeOptimization,
    pub additional_linker_arguments: Vec<String>,
    pub temporary_storage_size: Option<i64>,
    pub array_bounds_check: Option<bool>,
    /// `Build_Options.null_pointer_check`.
    pub null_pointer_check: Option<bool>,
    /// 0 = OFF, 1 = NONFATAL, 2 = FATAL (`Options::arithmetic_overflow_check`).
    pub arithmetic_overflow_check: Option<u8>,
    /// The same for `Options::cast_bounds_check`.
    pub cast_bounds_check: Option<u8>,
    pub stack_trace: Option<bool>,
    /// `emit_debug_info`: `Some(false)` for `.NONE`; `None` leaves the embedder's default.
    pub emit_debug_info: Option<bool>,
    /// `dead_code_elimination`; `None` leaves the embedder's default (`-no_dce`).
    pub dead_code_elimination: Option<DeadCode>,
    /// `llvm_options.target_system_triple`, `_cpu` and `_features` (empty: the default for
    /// `os`/`cpu`).
    pub llvm_target: Option<crate::target::Target>,
    pub llvm_cpu: String,
    pub llvm_features: String,
    /// `intermediate_path`: where object files (and LLVM IR or bitcode) go; empty: next to
    /// the output.
    pub intermediate_path: String,
    /// `entry_point_name`: the procedure the program starts in; empty: `main`.
    pub entry_point_name: String,
    /// `runtime_support_definitions` (`AUTO`...) and `backtrace_on_crash`, once a metaprogram
    /// changes them.
    pub runtime_support_definitions: Option<RuntimeSupport>,
    pub backtrace_on_crash: Option<bool>,
    /// `append_executable_filename_extension` (`.exe`, `.dll` on Windows...).
    pub append_extension: bool,
    /// `use_custom_link_command`: write objects, send `READY_FOR_CUSTOM_LINK_COMMAND` and let
    /// the metaprogram link.
    pub use_custom_link_command: bool,
    /// `minimum_os_version` as `major.minor`, for macOS targets; empty: jaic's default.
    pub minimum_os_version: String,
    /// `enable_frame_pointers`, `disable_redzone` and the `llvm_options` switches, each
    /// `None` until a metaprogram changes it (`docs/metaprogramming/build-options.md`).
    pub frame_pointers: Option<bool>,
    pub disable_redzone: bool,
    pub machine_code_optimization: MachineCodeOptimization,
    pub llvm_switches: Vec<(WorkspaceFlag, bool)>,
    /// `output_llvm_ir`, `output_bitcode` and their `_before_optimizations` variants.
    pub output_llvm_ir: bool,
    pub output_bitcode: bool,
    pub output_llvm_ir_before_optimizations: bool,
    pub output_bitcode_before_optimizations: bool,
}

impl BuildSettings {
    /// The value a metaprogram gave the `llvm_options` switch `flag` (`LlvmEnableLoopUnrolling`...).
    pub fn llvm_switch(&self, flag: WorkspaceFlag) -> Option<bool> {
        self.llvm_switches
            .iter()
            .rev()
            .find(|(n, _)| *n == flag)
            .map(|(_, on)| *on)
    }
}

impl Default for BuildSettings {
    fn default() -> Self {
        Self {
            output_type: OutputType::Executable,
            do_output: true,
            output_executable_name: String::new(),
            output_path: String::new(),
            import_paths: None,
            os: None,
            cpu: None,
            optimization: BitcodeOptimization::Unset,
            additional_linker_arguments: Vec::new(),
            temporary_storage_size: None,
            array_bounds_check: None,
            null_pointer_check: None,
            arithmetic_overflow_check: None,
            cast_bounds_check: None,
            stack_trace: None,
            emit_debug_info: None,
            dead_code_elimination: None,
            llvm_target: None,
            llvm_cpu: String::new(),
            llvm_features: String::new(),
            intermediate_path: String::new(),
            entry_point_name: String::new(),
            runtime_support_definitions: None,
            backtrace_on_crash: None,
            append_extension: true,
            use_custom_link_command: false,
            minimum_os_version: String::new(),
            frame_pointers: None,
            disable_redzone: false,
            machine_code_optimization: MachineCodeOptimization::Unset,
            llvm_switches: Vec::new(),
            output_llvm_ir: false,
            output_bitcode: false,
            output_llvm_ir_before_optimizations: false,
            output_bitcode_before_optimizations: false,
        }
    }
}

/// Writes a compiled workspace's output. Returns the files written.
pub trait OutputBackend {
    fn write_output(
        &mut self,
        program: &ir::Program,
        settings: &BuildSettings,
        output: &std::path::Path,
    ) -> Result<(), String>;

    /// For `use_custom_link_command`: write the object files for `output` and say what a
    /// linker needs, without linking.
    fn write_objects(
        &mut self,
        _program: &ir::Program,
        _settings: &BuildSettings,
        _output: &std::path::Path,
    ) -> Result<LinkObjects, String> {
        Err(
            "this backend cannot leave linking to the metaprogram (`use_custom_link_command`)"
                .into(),
        )
    }
}

/// What `READY_FOR_CUSTOM_LINK_COMMAND` hands the metaprogram (`Message_Phase` fields).
#[derive(Clone, Debug, Default)]
pub struct LinkObjects {
    /// `compiler_generated_object_files`.
    pub objects: Vec<String>,
    /// `system_libraries`: libraries linked by name.
    pub system_libraries: Vec<String>,
    /// `user_libraries`: library files by path and the directories they need.
    pub user_libraries: Vec<String>,
}

/// A workspace whose objects are written and whose metaprogram links them.
pub(super) struct PendingLink {
    /// The linker's exit code, once `compiler_custom_link_command_is_complete` says so.
    pub(super) exit_code: Option<i64>,
    pub(super) output: String,
}

/// How serious a reported message is; `-quiet` leaves out everything but errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReportSeverity {
    Error,
    Warning,
    Info,
}

impl ReportSeverity {
    /// Whether this is a warning or an informational note rather than an error.
    pub fn is_chatter(self) -> bool {
        self != ReportSeverity::Error
    }
}

/// Where diagnostics go: each message comes with its severity.
pub type ReportSink = Box<dyn FnMut(ReportSeverity, &str)>;

/// Everything needed to compile a workspace besides its sources.
pub struct BuildEnv {
    pub fs: Rc<dyn FileSystem>,
    /// Base options (import path, preload, target) for new workspaces.
    pub options: Options,
    pub backend: Option<Box<dyn OutputBackend>>,
    /// Without a backend: what to tell the user when a workspace asks for output that this
    /// command does not write (`jaic -check` only compiles workspaces). `None`
    /// says nothing (the browser and editors, where no output is ever expected).
    pub unwritten_output_hint: Option<UnwrittenOutputHint>,
    /// Arguments after `-` on the command line (`compile_time_command_line`).
    pub command_line: Vec<String>,
    /// Host for the compile-time interpreter of each workspace, given its target `OS`
    /// (`OS == .WASM` code expects the interpreter's sandbox, whatever the compiler runs on).
    pub make_host: Box<dyn Fn(TargetOs) -> Box<dyn Host>>,
    /// Where diagnostics of workspace compilations go.
    pub report: ReportSink,
    /// Sees each workspace compiler as it is made and when its workspace is done (tools that
    /// inspect what the metaprogram built, such as a linter).
    pub observer: Option<Box<dyn WorkspaceObserver>>,
}

/// The command line a warning about unwritten workspace output refers to.
#[derive(Clone, Debug)]
pub struct UnwrittenOutputHint {
    /// The main file as given on the command line.
    pub main_file: String,
}

/// `path` as messages show it (`crate::display_path`). A relative `path` is taken from the
/// current directory, the metaprogram's, which need not be where the tool started.
pub(super) fn shown_path(path: &std::path::Path) -> String {
    let absolute = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    let path: PathBuf = absolute
        .components()
        .filter(|c| !matches!(c, std::path::Component::CurDir))
        .collect();
    crate::display_path(&path)
}

/// Hooks into the life of workspace compilers. Called without the registry borrowed, but the
/// observer must not drive workspaces itself.
pub trait WorkspaceObserver {
    /// A compiler for a new workspace, before any source is added.
    fn created(&mut self, compiler: &mut Compiler);

    /// The workspace is done (`failed`: it did not compile); the compiler is handed over
    /// instead of being dropped.
    fn finished(&mut self, compiler: Box<Compiler>, failed: bool);

    /// The metaprogram added a file (`add_build_file`, as its canonical path) or a string
    /// (`add_build_string`) to workspace `workspace`, the id the workspace's compiler carries
    /// as `Compiler::workspace`.
    fn source_added(&mut self, _workspace: i64, _source: &ProgramSource) {
    }
}
