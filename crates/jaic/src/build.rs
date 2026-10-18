//! Workspaces: the compiler side of the `Compiler` module.
//!
//! A metaprogram creates workspaces, adds files and strings to them, sets
//! their build options and (optionally) intercepts their compiler messages.
//! Each workspace is compiled by a separate [`Compiler`] sharing the file
//! system; its output (executable, library, object) is written by the
//! embedder's [`OutputBackend`] (LLVM in `jaic-cli`, none in the browser).
//!
//! Jai code reaches this module only through a few `#compiler` primitives
//! (`__jaic_*`, see [`MetaOp`]) with a scalar/string ABI; the public
//! `Compiler` module API (Build_Options, Message structs...) is written in Jai
//! on top of them in `stdlib/Compiler/module.jai`.
use crate::builtin_keys::{
    ArrayBoundsCheck, BitcodeOptimization, CheckLevel, CpuTag, DeadCodeSetting, DebugInfoSetting,
    MachineCodeOptimization, OsTag, RuntimeSupport, SyntaxKey, Toggle, WorkspaceFlag,
    WorkspaceList, WorkspaceNumber, WorkspaceText,
};
use crate::fxhash::HashMap;
use crate::interp::{Host, Interp, Rets, Trap};
use crate::ir;
use crate::records::{FieldRef, Item, Records};
use crate::sema::{
    Compiler, DeadCode, FileSystem, Options, ProgramSource, ProvidedImport, ProvidedKind,
    TargetCpu, TargetOs,
};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::rc::Rc;

mod host;
mod messages;
mod option_snapshot;
mod plugin_program;
mod program_data;
mod records;
mod sources;
mod workspace;

pub use plugin_program::{PluginImport, PluginProgram, plugin_metaprogram};
pub use sources::parse_code_text;

pub use crate::builtin_keys::OutputType;

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
struct PendingLink {
    /// The linker's exit code, once `compiler_custom_link_command_is_complete` says so.
    exit_code: Option<i64>,
    output: String,
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
fn shown_path(path: &std::path::Path) -> String {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    /// Collecting files/strings/options; nothing compiled yet.
    Open,
    /// Sources loaded and their compile-time code run; more sources may be
    /// added (after `TYPECHECKED_ALL_WE_CAN`) before code generation.
    Checked,
    /// Output written (or failed); only queued events remain.
    Done,
}

struct Workspace {
    name: String,
    /// Sources added and not yet loaded into the workspace's compiler.
    pending: Vec<ProgramSource>,
    settings: BuildSettings,
    options: Option<option_snapshot::Snapshot>,
    intercepted: bool,
    /// The `Intercept_Flags` the intercepting metaprogram passed to `compiler_begin_intercept`.
    intercept_flags: i64,
    stage: Stage,
    /// The workspace's compiler between steps (taken out while it runs).
    compiler: Option<Box<Compiler>>,
    events: VecDeque<Event>,
    failed: bool,
    /// `compiler_modify_procedure` calls not applied yet: (body record, statement records).
    modifications: Vec<(i64, Vec<crate::sema::ModifiedStmt>)>,
    /// Set from `READY_FOR_CUSTOM_LINK_COMMAND` until the metaprogram's link is reported.
    link: Option<PendingLink>,
    /// `remap_import` rules: (importing module, imported name, replacement).
    remaps: Vec<(String, String, String)>,
    /// `provide_import` offers.
    provided: Vec<ProvidedImport>,
    /// Header records `compiler_make_procedure_live` named, not applied yet.
    live: Vec<i64>,
    /// A failed load held back so an intercepting metaprogram can `provide_import` what was
    /// missing; see [`resume_failed_load`].
    failed_load: Option<FailedLoad>,
}

/// A workspace whose sources did not load because an import found no module, while a
/// metaprogram was listening for `FAILED_IMPORT`.
struct FailedLoad {
    /// The diagnostic, reported if nothing is provided.
    text: String,
    /// The sources to load again once something was provided.
    sources: Vec<ProgramSource>,
    compiler: Box<Compiler>,
    /// How many offers there were when the imports failed.
    offers: usize,
}

impl Workspace {
    fn queue_source(&mut self, source: ProgramSource) -> Result<(), String> {
        if self.stage == Stage::Done {
            return Err(format!(
                "workspace `{}` has already been compiled; add its sources before it completes",
                self.name
            ));
        }
        self.pending.push(source);
        Ok(())
    }

    fn new(name: String) -> Self {
        Workspace {
            name,
            pending: Vec::new(),
            settings: BuildSettings::default(),
            options: None,
            intercepted: false,
            intercept_flags: 0,
            stage: Stage::Open,
            compiler: None,
            events: VecDeque::new(),
            failed: false,
            modifications: Vec::new(),
            link: None,
            remaps: Vec::new(),
            provided: Vec::new(),
            live: Vec::new(),
            failed_load: None,
        }
    }
}

/// One compiler message, in primitive form.
#[derive(Clone, Debug, Default)]
pub struct Event {
    pub kind: i64,
    pub ints: Vec<i64>,
    pub strings: Vec<Vec<u8>>,
}

pub const EVENT_FILE: i64 = 1;
pub const EVENT_PHASE: i64 = 2;
pub const EVENT_COMPLETE: i64 = 3;
pub const EVENT_IMPORT: i64 = 4;
pub const EVENT_TYPECHECKED: i64 = 5;
pub const EVENT_FAILED_IMPORT: i64 = 6;
pub const EVENT_ERROR: i64 = 7;

/// The text of a `DEBUG_DUMP` message is string 0.
pub const EVENT_DEBUG_DUMP: i64 = 8;

pub const EVENT_PERFORMANCE_REPORT: i64 = 9;

/// `Intercept_Flags.DO_PERFORMANCE_REPORT_POLYMORPHS` and `_RUNS` (`stdlib/Compiler/workspace.jai`).
const INTERCEPT_REPORT_POLYMORPHS: i64 = 0x1000;

const INTERCEPT_REPORT_RUNS: i64 = 0x2000;

const PHASE_ALL_SOURCE_CODE_PARSED: i64 = 0;
const PHASE_TYPECHECKED_ALL_WE_CAN: i64 = 1;
const PHASE_ALL_TARGET_CODE_BUILT: i64 = 2;
const PHASE_PRE_WRITE_EXECUTABLE: i64 = 3;
const PHASE_POST_WRITE_EXECUTABLE: i64 = 4;
const PHASE_READY_FOR_CUSTOM_LINK_COMMAND: i64 = 5;

/// The workspace registry of one top-level compilation.
pub struct Workspaces {
    env: BuildEnv,
    /// Some compile-time code can ask for a type table (`get_type_table`).
    type_table_used: bool,
    /// Index = workspace id; 0 and 1 are unused (`jai` reserves workspace 1, so the top-level
    /// program is workspace 2 and the first created workspace is 3).
    list: Vec<Workspace>,
    /// Workspace whose compile-time code is running.
    current: Vec<i64>,
    event: Event,
    /// Strings handed to Jai code; kept alive for the whole compilation.
    strings: Vec<Box<[u8]>>,
    /// Record strings handed to Jai code without copying (the record may later drop its own).
    kept: Vec<Rc<[u8]>>,
    /// Record tags by `__jaic_rec_tag_id`.
    tags: HashMap<&'static str, usize>,
    /// Messages, syntax trees and types exported to metaprograms.
    pub(crate) records: Records,
    /// The structs each program built from the records (`__jaic_rec_builder` numbers them from 1).
    builders: Vec<crate::record_build::Builder>,
    data_segments: Vec<DataSegment>,
    /// The absolute path of the program's first source file, when it comes from a file: the
    /// top-level workspace's default output name and directory derive from it.
    pub main_file: String,
    /// What the command line chose for the top-level program's output and crash handler.
    pub main_overrides: MainOverrides,
}

struct DataSegment {
    handle: u64,
    _name: Vec<u8>,
    workspace: i64,
    characteristics: u32,
    alignment: u64,
}

/// Command-line choices the target program's `get_build_options` reports before a metaprogram
/// changes anything (`-exe`, `-output_path`, `-no_backtrace_on_crash`).
#[derive(Clone, Debug, Default)]
pub struct MainOverrides {
    /// `output_executable_name`; empty: derived from the main file.
    pub exe: String,
    /// `output_path`; empty: the main file's directory.
    pub output_path: String,
    pub no_backtrace_on_crash: bool,
}

pub type SharedWorkspaces = Rc<RefCell<Workspaces>>;

/// The `#compiler` primitives, bound by name in `stdlib/Compiler`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetaOp {
    WorkspaceCreate,
    CurrentWorkspace,
    ValidateWorkspace,
    DataSegment,
    GlobalData,
    RuntimeData,
    AddFile,
    AddString,
    DestroyWorkspace,
    SetText,
    SetFlag,
    SetNumber,
    ClearList,
    AppendList,
    OptionsSnapshot,
    BeginIntercept,
    EndIntercept,
    WorkspaceStatus,
    NextEvent,
    EventInt,
    EventString,
    CommandLineCount,
    CommandLineArg,
    Report,
    CompilerVersion,
    CustomLinkComplete,
    AddStringToModule,
    AddStringToCode,
    CodeNodes,
    CodeIsNull,
    ParseCode,
    ModifyProcedure,
    SetTypeInfoFlags,
    StructLocation,
    BasePath,
    MainFile,
    MainOverride,
    TypeTable,
    AddLibraryDir,
    RemapImport,
    ProvideImport,
    MakeLive,
    RecTag,
    RecField,
    RecInt,
    RecString,
    RecRef,
    RecCount,
    RecItemInt,
    RecItemString,
    RecItemRef,
    RecTagId,
    RecBuilder,
    RecType,
    RecBuiltin,
    RecPlan,
    RecBuild,
    RecOf,
    SyntaxInt,
    SyntaxString,
    SyntaxChild,
    Clang,
    ClangText,
}

impl MetaOp {
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "__jaic_workspace_create" => Self::WorkspaceCreate,
            "get_current_workspace" | "__jaic_current_workspace" => Self::CurrentWorkspace,
            "__jaic_workspace_validate" => Self::ValidateWorkspace,
            "__jaic_data_segment" => Self::DataSegment,
            "__jaic_global_data" => Self::GlobalData,
            "__jaic_runtime_data" => Self::RuntimeData,
            "__jaic_workspace_add_file" => Self::AddFile,
            "__jaic_workspace_add_string" => Self::AddString,
            "__jaic_workspace_destroy" => Self::DestroyWorkspace,
            "__jaic_workspace_set_text" => Self::SetText,
            "__jaic_workspace_set_flag" => Self::SetFlag,
            "__jaic_workspace_set_number" => Self::SetNumber,
            "__jaic_workspace_clear_list" => Self::ClearList,
            "__jaic_workspace_append_list" => Self::AppendList,
            "__jaic_workspace_options_snapshot" => Self::OptionsSnapshot,
            "__jaic_workspace_begin_intercept" => Self::BeginIntercept,
            "__jaic_workspace_end_intercept" => Self::EndIntercept,
            "__jaic_workspace_status" => Self::WorkspaceStatus,
            "__jaic_workspace_next_event" => Self::NextEvent,
            "__jaic_event_int" => Self::EventInt,
            "__jaic_event_string" => Self::EventString,
            "__jaic_command_line_count" => Self::CommandLineCount,
            "__jaic_command_line_arg" => Self::CommandLineArg,
            "__jaic_report" => Self::Report,
            "__jaic_compiler_version" => Self::CompilerVersion,
            "__jaic_custom_link_complete" => Self::CustomLinkComplete,
            "__jaic_workspace_add_string_to_module" => Self::AddStringToModule,
            "__jaic_workspace_add_string_to_code" => Self::AddStringToCode,
            "__jaic_code_nodes" => Self::CodeNodes,
            "__jaic_code_is_null" => Self::CodeIsNull,
            "__jaic_parse_code" => Self::ParseCode,
            "__jaic_modify_procedure" => Self::ModifyProcedure,
            "__jaic_set_type_info_flags" => Self::SetTypeInfoFlags,
            "__jaic_struct_location" => Self::StructLocation,
            "__jaic_base_path" => Self::BasePath,
            "__jaic_main_file" => Self::MainFile,
            "__jaic_main_override" => Self::MainOverride,
            "__jaic_type_table" => Self::TypeTable,
            "__jaic_add_library_dir" => Self::AddLibraryDir,
            "__jaic_workspace_remap_import" => Self::RemapImport,
            "__jaic_workspace_provide_import" => Self::ProvideImport,
            "__jaic_make_procedure_live" => Self::MakeLive,
            "__jaic_rec_tag" => Self::RecTag,
            "__jaic_rec_field" => Self::RecField,
            "__jaic_rec_int" => Self::RecInt,
            "__jaic_rec_string" => Self::RecString,
            "__jaic_rec_ref" => Self::RecRef,
            "__jaic_rec_count" => Self::RecCount,
            "__jaic_rec_item_int" => Self::RecItemInt,
            "__jaic_rec_item_string" => Self::RecItemString,
            "__jaic_rec_item_ref" => Self::RecItemRef,
            "__jaic_rec_tag_id" => Self::RecTagId,
            "__jaic_rec_builder" => Self::RecBuilder,
            "__jaic_rec_type" => Self::RecType,
            "__jaic_rec_builtin" => Self::RecBuiltin,
            "__jaic_rec_plan" => Self::RecPlan,
            "__jaic_rec_build" => Self::RecBuild,
            "__jaic_rec_of" => Self::RecOf,
            "__jaic_syntax_int" => Self::SyntaxInt,
            "__jaic_syntax_string" => Self::SyntaxString,
            "__jaic_syntax_child" => Self::SyntaxChild,
            "__jaic_clang" => Self::Clang,
            "__jaic_clang_text" => Self::ClangText,
            _ => return None,
        })
    }
}

/// Id of the program being compiled by the command line (workspace 1 is reserved, as in `jai`).
pub const TOP_LEVEL_WORKSPACE: i64 = 2;

/// Jai language version reported to programs, as major / minor / micro. Build scripts check it
/// against a supported range, so it stays in the 0.2 line and comes first in the version text.
pub const LANGUAGE_VERSION: (u32, u32, u32) = (0, 2, 99);

/// Version text returned by `compiler_get_version_info`: the language version, whose three
/// integers fill `Version_Info`, then the Jacket product version.
pub fn compiler_version() -> String {
    let (major, minor, micro) = LANGUAGE_VERSION;
    format!(
        "Jai {major}.{minor}.{micro} (Jacket {})",
        env!("CARGO_PKG_VERSION")
    )
}

impl Workspaces {
    /// A registry whose workspace 1 is the top-level program.
    /// Compile-time code that can call `get_type_table` exists (see `type_table_used`).
    pub fn note_type_table_used(shared: &SharedWorkspaces) {
        if let Ok(mut reg) = shared.try_borrow_mut() {
            reg.type_table_used = true;
        }
    }

    pub fn new(env: BuildEnv) -> SharedWorkspaces {
        let mut top = Workspace::new("Target Program".into());
        // The embedder compiles the top-level program itself.
        top.stage = Stage::Checked;
        Rc::new(RefCell::new(Workspaces {
            env,
            type_table_used: false,
            list: vec![
                Workspace::new(String::new()),
                Workspace::new(String::new()),
                top,
            ],
            current: vec![TOP_LEVEL_WORKSPACE],
            event: Event::default(),
            strings: Vec::new(),
            kept: Vec::new(),
            tags: HashMap::default(),
            records: Records::default(),
            builders: Vec::new(),
            data_segments: Vec::new(),
            main_file: String::new(),
            main_overrides: MainOverrides::default(),
        }))
    }

    /// Settings of the top-level program (workspace 2).
    pub fn top_level_settings(&self) -> BuildSettings {
        self.list[TOP_LEVEL_WORKSPACE as usize].settings.clone()
    }

    /// How many workspaces compile-time code created (the top-level program is not one).
    pub fn created_workspaces(&self) -> usize {
        self.list
            .len()
            .saturating_sub(TOP_LEVEL_WORKSPACE as usize + 1)
    }

    /// Whether any workspace compilation failed.
    pub fn any_failed(&self) -> bool {
        self.list.iter().any(|w| w.failed)
    }

    fn ws(&mut self, id: i64) -> Result<&mut Workspace, String> {
        let id = if id == 0 {
            self.current_id()
        } else {
            id
        };
        self.list
            .get_mut(id as usize)
            .filter(|_| id > 0)
            .ok_or_else(|| format!("invalid workspace {id}"))
    }

    fn current_id(&self) -> i64 {
        *self.current.last().unwrap_or(&TOP_LEVEL_WORKSPACE)
    }

    fn workspace_settings(&mut self, id: i64) -> Result<&mut BuildSettings, String> {
        Ok(&mut self.ws(id)?.settings)
    }

    /// `compiler_destroy_workspace`: what was added is never compiled.
    fn destroy_workspace(&mut self, id: i64) -> Result<(), String> {
        let actual = if id == 0 {
            self.current_id()
        } else {
            id
        };
        if self.current.contains(&actual) {
            return Err("cannot destroy an active compiler workspace".into());
        }
        let ws = self.ws(id)?;
        ws.pending.clear();
        if ws.stage == Stage::Open {
            ws.stage = Stage::Done;
        }
        Ok(())
    }

    fn set_text(&mut self, id: i64, option: WorkspaceText, value: &str) -> Result<(), String> {
        let s = self.workspace_settings(id)?;
        match option {
            WorkspaceText::OutputExecutableName => s.output_executable_name = value.into(),
            WorkspaceText::OutputPath => s.output_path = value.into(),
            WorkspaceText::IntermediatePath => s.intermediate_path = value.into(),
            WorkspaceText::EntryPointName => s.entry_point_name = value.into(),
            WorkspaceText::LlvmTargetSystemTriple => {
                s.llvm_target = if value.is_empty() {
                    None
                } else {
                    Some(crate::target::Target::parse(value)?)
                }
            }
            WorkspaceText::LlvmTargetSystemCpu => s.llvm_cpu = value.into(),
            WorkspaceText::LlvmTargetSystemFeatures => s.llvm_features = value.into(),
            WorkspaceText::MinimumOsVersion => s.minimum_os_version = value.into(),
        }
        Ok(())
    }

    fn set_flag(&mut self, id: i64, option: WorkspaceFlag, on: bool) -> Result<(), String> {
        let s = self.workspace_settings(id)?;
        match option {
            WorkspaceFlag::DoOutput => s.do_output = on,
            WorkspaceFlag::StackTrace => s.stack_trace = Some(on),
            WorkspaceFlag::DisableRedzone => s.disable_redzone = on,
            WorkspaceFlag::EnableFramePointers => s.frame_pointers = Some(on),
            WorkspaceFlag::AppendExecutableFilenameExtension => s.append_extension = on,
            WorkspaceFlag::UseCustomLinkCommand => s.use_custom_link_command = on,
            WorkspaceFlag::OutputBitcode => s.output_bitcode = on,
            WorkspaceFlag::OutputLlvmIr => s.output_llvm_ir = on,
            WorkspaceFlag::OutputBitcodeBeforeOptimizations => {
                s.output_bitcode_before_optimizations = on
            }
            WorkspaceFlag::OutputLlvmIrBeforeOptimizations => {
                s.output_llvm_ir_before_optimizations = on
            }
            WorkspaceFlag::LlvmEnableTailCalls
            | WorkspaceFlag::LlvmEnableLoopUnrolling
            | WorkspaceFlag::LlvmEnableSlpVectorization
            | WorkspaceFlag::LlvmEnableLoopVectorization
            | WorkspaceFlag::LlvmPreserveDebugInfo
            | WorkspaceFlag::LlvmMergeFunctions
            | WorkspaceFlag::LlvmDisableInlining
            | WorkspaceFlag::LlvmDisableMem2reg
            | WorkspaceFlag::LlvmEnableSplitModules
            | WorkspaceFlag::LlvmFunctionSections => s.llvm_switches.push((option, on)),
        }
        Ok(())
    }

    fn set_number(&mut self, id: i64, option: WorkspaceNumber, raw: u64) -> Result<(), String> {
        fn member<T>(
            option: WorkspaceNumber,
            raw: u64,
            decode: fn(u64) -> Option<T>,
        ) -> Result<T, String> {
            decode(raw).ok_or_else(|| format!("internal: {option:?} has no value {raw}"))
        }
        let s = self.workspace_settings(id)?;
        match option {
            WorkspaceNumber::OutputType => {
                s.output_type = member(option, raw, OutputType::from_raw)?
            }
            WorkspaceNumber::OsTarget => {
                s.os = Some(match member(option, raw, OsTag::from_raw)? {
                    OsTag::Windows => TargetOs::Windows,
                    OsTag::Linux => TargetOs::Linux,
                    OsTag::MacOS => TargetOs::MacOS,
                    OsTag::Wasm => TargetOs::Wasm,
                })
            }
            WorkspaceNumber::CpuTarget => {
                s.cpu = Some(match member(option, raw, CpuTag::from_raw)? {
                    CpuTag::X64 => TargetCpu::X64,
                    CpuTag::Arm64 => TargetCpu::Arm64,
                    CpuTag::Custom => TargetCpu::Wasm,
                })
            }
            WorkspaceNumber::TemporaryStorageSize => s.temporary_storage_size = Some(raw as i64),
            WorkspaceNumber::ArrayBoundsCheck => {
                s.array_bounds_check =
                    Some(member(option, raw, ArrayBoundsCheck::from_raw)? != ArrayBoundsCheck::Off)
            }
            WorkspaceNumber::NullPointerCheck => {
                s.null_pointer_check = Some(member(option, raw, Toggle::from_raw)? != Toggle::Off)
            }
            WorkspaceNumber::ArithmeticOverflowCheck => {
                s.arithmetic_overflow_check = Some(member(option, raw, CheckLevel::from_raw)? as u8)
            }
            WorkspaceNumber::CastBoundsCheck => {
                s.cast_bounds_check = Some(member(option, raw, CheckLevel::from_raw)? as u8)
            }
            WorkspaceNumber::EmitDebugInfo => {
                s.emit_debug_info =
                    Some(member(option, raw, DebugInfoSetting::from_raw)? != DebugInfoSetting::None)
            }
            WorkspaceNumber::DeadCodeElimination => {
                s.dead_code_elimination =
                    Some(match member(option, raw, DeadCodeSetting::from_raw)? {
                        DeadCodeSetting::None => DeadCode::None,
                        DeadCodeSetting::All => DeadCode::All,
                        DeadCodeSetting::ModulesOnly => DeadCode::ModulesOnly,
                    })
            }
            WorkspaceNumber::RuntimeSupportDefinitions => {
                s.runtime_support_definitions = Some(member(option, raw, RuntimeSupport::from_raw)?)
            }
            WorkspaceNumber::BacktraceOnCrash => {
                s.backtrace_on_crash = Some(member(option, raw, Toggle::from_raw)? == Toggle::On)
            }
            WorkspaceNumber::BitcodeOptimization => {
                s.optimization = member(option, raw, BitcodeOptimization::from_raw)?
            }
            WorkspaceNumber::MachineCodeOptimization => {
                s.machine_code_optimization =
                    member(option, raw, MachineCodeOptimization::from_raw)?
            }
        }
        Ok(())
    }

    fn clear_list(&mut self, id: i64, option: WorkspaceList) -> Result<(), String> {
        let s = self.workspace_settings(id)?;
        match option {
            WorkspaceList::ImportPath => s.import_paths = Some(Vec::new()),
            WorkspaceList::AdditionalLinkerArgument => s.additional_linker_arguments.clear(),
        }
        Ok(())
    }

    fn append_list(&mut self, id: i64, option: WorkspaceList, value: &str) -> Result<(), String> {
        let s = self.workspace_settings(id)?;
        match option {
            WorkspaceList::ImportPath => s
                .import_paths
                .get_or_insert_with(Vec::new)
                .push(PathBuf::from(value)),
            WorkspaceList::AdditionalLinkerArgument => {
                s.additional_linker_arguments.push(value.into())
            }
        }
        Ok(())
    }

    fn keep_rc(&mut self, bytes: &Rc<[u8]>) -> (u64, u64) {
        self.kept.push(bytes.clone());
        (bytes.len() as u64, bytes.as_ptr() as u64)
    }

    fn keep_string(&mut self, bytes: &[u8]) -> (u64, u64) {
        let boxed: Box<[u8]> = bytes.into();
        let ptr = boxed.as_ptr() as u64;
        self.strings.push(boxed);
        (bytes.len() as u64, ptr)
    }
}

/// The module that makes a wasm program a WASI command (`_start`, the C library subset the
/// stdlib calls, an allocator); see `docs/native/wasm-target.md`.
pub const WASI_RUNTIME_IMPORT: &str =
    "#import \"Extensions/Wasi_Runtime\"; #import \"Wasi_Numeric_Runtime\";";

/// A host evaluation owns a fresh registry, preserving the embedder's real host
/// provider and reporting without inheriting the target workspace's settings.
pub(crate) fn isolated_host_environment(
    parent: &SharedWorkspaces,
    options: Options,
    fs: Rc<dyn FileSystem>,
) -> (SharedWorkspaces, Box<dyn Host>) {
    let host = (parent.borrow().env.make_host)(options.os);
    let factory_parent = parent.clone();
    let report_parent = parent.clone();
    let command_line = parent.borrow().env.command_line.clone();
    let shared = Workspaces::new(BuildEnv {
        fs,
        options,
        backend: None,
        unwritten_output_hint: None,
        command_line,
        make_host: Box::new(move |os| (factory_parent.borrow().env.make_host)(os)),
        report: Box::new(move |severity, message| {
            (report_parent.borrow_mut().env.report)(severity, message)
        }),
        observer: None,
    });
    (shared, host)
}

/// Whether a build for `target` is a WASI program, which gets [`WASI_RUNTIME_IMPORT`] added to
/// its sources. Other wasm builds supply their own runtime.
pub fn wants_wasi_runtime(target: Option<crate::target::Target>) -> bool {
    target.is_some_and(|t| t.is_wasm() && t.abi == crate::target::TargetAbi::Wasi)
}

/// `dead_code_elimination` as compile-time code last set it for workspace `id`, which may be
/// the workspace being compiled (`set_build_options` without a workspace).
pub fn dead_code_setting(shared: &SharedWorkspaces, id: i64) -> Option<DeadCode> {
    let reg = shared.borrow();
    reg.list.get(id as usize)?.settings.dead_code_elimination
}

/// Target runtime storage settings, including changes made by the compiling
/// workspace itself. NO_OUTPUT workspaces have no target runtime to initialize.
pub(crate) fn runtime_storage_setting(
    shared: &SharedWorkspaces,
    id: i64,
    default_size: i64,
) -> Option<(bool, i64)> {
    let reg = shared.borrow();
    let settings = &reg.list.get(id as usize)?.settings;
    Some((
        settings.output_type != OutputType::NoOutput,
        settings.temporary_storage_size.unwrap_or(default_size),
    ))
}

/// The `remap_import` and `provide_import` rules of workspace `id`, for the compiler that is
/// resolving its imports.
pub fn import_rules(
    shared: &SharedWorkspaces,
    id: i64,
) -> (Vec<(String, String, String)>, Vec<ProvidedImport>) {
    let reg = shared.borrow();
    match reg.list.get(id as usize) {
        Some(ws) => (ws.remaps.clone(), ws.provided.clone()),
        None => (Vec::new(), Vec::new()),
    }
}

/// Header records `compiler_make_procedure_live` named for workspace `id` since last asked.
pub fn take_live_requests(shared: &SharedWorkspaces, id: i64) -> Vec<i64> {
    let mut reg = shared.borrow_mut();
    match reg.list.get_mut(id as usize) {
        Some(ws) => std::mem::take(&mut ws.live),
        None => Vec::new(),
    }
}

/// Sources added to workspace `id` by its own compile-time code, for the
/// compiler that is building it (`Compiler::pull_workspace_sources`).
pub fn take_own_sources(shared: &SharedWorkspaces, id: i64) -> Vec<ProgramSource> {
    let mut reg = shared.borrow_mut();
    let running = reg.current_id() == id;
    match reg.list.get_mut(id as usize) {
        Some(ws) if running && ws.stage != Stage::Done => std::mem::take(&mut ws.pending),
        _ => Vec::new(),
    }
}

/// A compiler for workspace `id` with its build settings applied.
fn new_compiler(shared: &SharedWorkspaces, id: i64) -> Result<Box<Compiler>, String> {
    let (settings, mut options, fs, host) = {
        let mut reg = shared.borrow_mut();
        let settings = reg.ws(id)?.settings.clone();
        let options = reg.env.options.clone();
        let os = settings.os.unwrap_or(options.os);
        (
            settings,
            options,
            reg.env.fs.clone(),
            (reg.env.make_host)(os),
        )
    };
    if let Some(paths) = &settings.import_paths {
        // The module search path the metaprogram set, then the defaults (stdlib).
        let defaults = std::mem::take(&mut options.import_paths);
        options.import_paths = paths.clone();
        for d in defaults {
            if !options.import_paths.contains(&d) {
                options.import_paths.push(d);
            }
        }
    }
    if let Some(os) = settings.os {
        options.os = os;
    }
    if let Some(cpu) = settings.cpu {
        options.cpu = cpu;
    }
    if let Some(size) = settings.temporary_storage_size {
        options.temporary_storage_size = size;
    }
    if let Some(check) = settings.array_bounds_check {
        options.array_bounds_check = check;
    }
    if let Some(check) = settings.null_pointer_check {
        options.null_pointer_check = check;
    }
    if let Some(check) = settings.arithmetic_overflow_check {
        options.arithmetic_overflow_check = check;
    }
    if let Some(check) = settings.cast_bounds_check {
        options.cast_bounds_check = check;
    }
    if let Some(trace) = settings.stack_trace {
        options.stack_trace = trace;
    }
    if let Some(debug) = settings.emit_debug_info {
        options.debug_info &= debug;
    }
    if let Some(mode) = settings.dead_code_elimination {
        options.dead_code = mode;
    }
    if !settings.entry_point_name.is_empty() {
        options.entry_point = settings.entry_point_name.clone();
    }
    if let Some(which) = &settings.runtime_support_definitions {
        // AUTO and ENTRY_POINT_AND_INIT: both; ONLY_INIT: the program has its own entry point.
        options.runtime_entry_point =
            !matches!(which, RuntimeSupport::OnlyInit | RuntimeSupport::Omit);
        options.runtime_initialization = *which != RuntimeSupport::Omit;
    }
    if let Some(on) = settings.backtrace_on_crash {
        // WebAssembly has no signals to catch (`docs/metaprogramming/build-options.md`).
        options.backtrace_on_crash = on && options.os != TargetOs::Wasm;
    }
    let mut compiler = Box::new(Compiler::new(options, fs));
    compiler.interp.host = host;
    compiler.workspace = id;
    compiler.attach_workspaces(shared.clone());
    let observer = shared.borrow_mut().env.observer.take();
    if let Some(mut o) = observer {
        o.created(&mut compiler);
        shared.borrow_mut().env.observer = Some(o);
    }
    Ok(compiler)
}

fn record_event(kind: i64, record: i64) -> Event {
    Event {
        kind,
        ints: vec![record],
        strings: Vec::new(),
    }
}

fn phase(p: i64) -> Event {
    Event {
        kind: EVENT_PHASE,
        ints: vec![p, 0],
        strings: Vec::new(),
    }
}

/// Advance workspace `id` by one step, queueing the messages it produces:
/// Open → (load, run directives) Checked → (more sources, or generate code
/// and write output) Done. Errors are reported and end in a failed COMPLETE.
///
/// `budget` is the interpreter block budget of whoever drives the workspace (the metaprogram's
/// interpreter, or the top-level compiler). The workspace's compile-time code draws from it
/// and hands back what is left, so one budget bounds the whole compilation however many
/// workspaces a metaprogram creates.
fn step(shared: &SharedWorkspaces, id: i64, budget: &mut Option<u64>) -> Result<(), String> {
    if shared.borrow().current.contains(&id) {
        return Err(format!(
            "compile-time workspace dependency cycle: workspace {id} is already active"
        ));
    }
    if shared.borrow_mut().ws(id)?.link.is_some() {
        return finish_custom_link(shared, id);
    }
    if shared.borrow_mut().ws(id)?.failed_load.is_some() && resume_failed_load(shared, id)? {
        return Ok(());
    }
    let (stage, mut compiler, pending, modifications) = {
        let mut reg = shared.borrow_mut();
        let ws = reg.ws(id)?;
        if ws.stage == Stage::Done {
            return Ok(());
        }
        (
            ws.stage,
            ws.compiler.take(),
            std::mem::take(&mut ws.pending),
            std::mem::take(&mut ws.modifications),
        )
    };
    let mut lowering = false;
    let mut compiler = match compiler.take() {
        Some(c) => c,
        None => new_compiler(shared, id)?,
    };
    compiler.interp.block_budget = *budget;
    let mut events = Vec::new();
    // The registry borrow is released while the compiler runs: its
    // compile-time code may call back into the registry.
    shared.borrow_mut().current.push(id);
    let modified = {
        let reg = shared.borrow();
        modifications
            .iter()
            .try_for_each(|(body, stmts)| compiler.modify_procedure(&reg.records, *body, stmts))
    }
    .and_then(|()| compiler.relower_modified());
    let mut pending = pending;
    let original_sources = pending.clone();
    if stage == Stage::Open {
        let reg = shared.borrow();
        let settings = &reg.list[id as usize].settings;
        if wants_wasi_runtime(settings.llvm_target) {
            pending.push(ProgramSource::String(WASI_RUNTIME_IMPORT.into()));
        }
    }
    let result = modified.and_then(|()| match stage {
        Stage::Open => compiler.begin_sources(&pending).map(|()| {
            events.push(phase(PHASE_ALL_SOURCE_CODE_PARSED));
            events.push(phase(PHASE_TYPECHECKED_ALL_WE_CAN));
            Stage::Checked
        }),
        Stage::Checked if !pending.is_empty() => pending
            .iter()
            .try_for_each(|source| compiler.add_source(source))
            .and_then(|()| compiler.settle())
            .map(|()| {
                events.push(phase(PHASE_TYPECHECKED_ALL_WE_CAN));
                Stage::Checked
            }),
        // Out of sources: lower what is reachable first; what that reports may make
        // the metaprogram add more before the program is generated.
        Stage::Checked => {
            compiler.lower_reachable();
            lowering = true;
            Ok(Stage::Checked)
        }
        Stage::Done => unreachable!(),
    });
    let intercepted = shared.borrow_mut().ws(id)?.intercepted;
    let mut file_events = Vec::new();
    let mut reported = false;
    if intercepted {
        // The registry's records are taken out while the compiler exports:
        // resolving declarations may run compile-time code.
        let mut records = Records::lend(&mut shared.borrow_mut().records);
        for (kind, record) in compiler.export_file_events(&mut records) {
            file_events.push(record_event(
                if kind == crate::sema::code_export::message_kind::IMPORT {
                    EVENT_IMPORT
                } else {
                    EVENT_FILE
                },
                record,
            ));
        }
        // After the final lowering too: procedure bodies are reported once lowered.
        if matches!(result, Ok(Stage::Checked | Stage::Done))
            && let Some(message) = compiler.export_typechecked(&mut records)
        {
            reported = true;
            // Before TYPECHECKED_ALL_WE_CAN, after any files the export loaded.
            for (kind, record) in compiler.export_file_events(&mut records) {
                file_events.push(record_event(
                    if kind == crate::sema::code_export::message_kind::IMPORT {
                        EVENT_IMPORT
                    } else {
                        EVENT_FILE
                    },
                    record,
                ));
            }
            let at = events.len().saturating_sub(1);
            events.insert(at, record_event(EVENT_TYPECHECKED, message));
        }
        Records::give_back(&mut shared.borrow_mut().records, records);
    }
    // Lowering reported more: the metaprogram gets another TYPECHECKED_ALL_WE_CAN to
    // add code. Otherwise the program is complete.
    let result = match result {
        Ok(_) if lowering && reported => {
            events.push(phase(PHASE_TYPECHECKED_ALL_WE_CAN));
            Ok(Stage::Checked)
        }
        Ok(_) if lowering => {
            let finished = compiler.finish_program().map(|()| Stage::Done);
            // Checking what nothing reaches lowered more bodies: they are reported too, as the
            // last TYPECHECKED message.
            if finished.is_ok() && intercepted {
                let mut records = Records::lend(&mut shared.borrow_mut().records);
                if let Some(message) = compiler.export_typechecked(&mut records) {
                    events.push(record_event(EVENT_TYPECHECKED, message));
                }
                Records::give_back(&mut shared.borrow_mut().records, records);
            }
            finished
        }
        other => other,
    };
    shared.borrow_mut().current.pop();
    *budget = compiler.interp.block_budget;
    let mut failed = false;
    let mut held = None;
    let mut compiled = false;
    let next = match result {
        Ok(Stage::Done) => {
            // The procedures declared `#dump` are lowered; their listings go out before the
            // program is written.
            if intercepted {
                for text in compiler.debug_dumps() {
                    events.push(Event {
                        kind: EVENT_DEBUG_DUMP,
                        ints: Vec::new(),
                        strings: vec![text.into_bytes()],
                    });
                }
            }
            match write_output(shared, id, &mut compiler, &mut events)? {
                Written::Done(ok) => {
                    failed = !ok;
                    compiled = true;
                    Stage::Done
                }
                // The metaprogram links; `finish_custom_link` ends the workspace.
                Written::AwaitingLink => Stage::Checked,
            }
        }
        Ok(next) => next,
        Err(d) => {
            let text = compiler.render(&d);
            // An editor compile (see `WorkspaceObserver`) keeps the error with its facts.
            if let Some(ide) = compiler.ide.as_mut() {
                ide.errors.push((*d).clone());
            }
            let mut imports = Vec::new();
            if intercepted {
                // The imports that found nothing, then the error itself.
                let mut records = Records::lend(&mut shared.borrow_mut().records);
                imports = compiler.export_failed_imports(&mut records);
                Records::give_back(&mut shared.borrow_mut().records, records);
                for &record in &imports {
                    events.push(record_event(EVENT_FAILED_IMPORT, record));
                }
            }
            if !imports.is_empty() && stage == Stage::Open {
                // The metaprogram may `provide_import` what was missing: the error waits for
                // its answer (`resume_failed_load`).
                held = Some(text);
                Stage::Open
            } else {
                (shared.borrow_mut().env.report)(ReportSeverity::Error, &text);
                failed = true;
                if intercepted {
                    events.push(Event {
                        kind: EVENT_ERROR,
                        ints: Vec::new(),
                        strings: Vec::new(),
                    });
                }
                Stage::Done
            }
        }
    };
    if next == Stage::Done {
        if compiled {
            events.extend(performance_event(shared, id, &mut compiler));
        }
        events.push(Event {
            kind: EVENT_COMPLETE,
            ints: vec![failed as i64],
            strings: Vec::new(),
        });
    }
    file_events.extend(events);
    {
        let mut reg = shared.borrow_mut();
        // A metaprogram that asks for type tables may ask after `COMPLETE`: a finished
        // workspace keeps its compiler then, unless an observer takes it.
        let keep = reg.type_table_used && reg.env.observer.is_none();
        let ws = reg.ws(id)?;
        ws.events.extend(file_events);
        ws.failed |= failed;
        ws.stage = next;
        if let Some(text) = held {
            let offers = ws.provided.len();
            ws.failed_load = Some(FailedLoad {
                text,
                sources: original_sources,
                compiler,
                offers,
            });
            return Ok(());
        }
        if next != Stage::Done || keep {
            ws.compiler = Some(compiler);
            return Ok(());
        }
    }
    let observer = shared.borrow_mut().env.observer.take();
    if let Some(mut o) = observer {
        o.finished(compiler, failed);
        shared.borrow_mut().env.observer = Some(o);
    }
    Ok(())
}

/// The `PERFORMANCE_REPORT` message of a workspace that compiled, when its metaprogram asked for
/// one with `DO_PERFORMANCE_REPORT_RUNS` or `_POLYMORPHS`: sent after everything else and right
/// before `COMPLETE`.
fn performance_event(shared: &SharedWorkspaces, id: i64, compiler: &mut Compiler) -> Option<Event> {
    let flags = {
        let mut reg = shared.borrow_mut();
        let ws = reg.ws(id).ok()?;
        if !ws.intercepted {
            return None;
        }
        ws.intercept_flags
    };
    let runs = flags & INTERCEPT_REPORT_RUNS != 0;
    let polymorphs = flags & INTERCEPT_REPORT_POLYMORPHS != 0;
    if !runs && !polymorphs {
        return None;
    }
    let mut records = Records::lend(&mut shared.borrow_mut().records);
    let record = compiler.export_performance_report(&mut records, runs, polymorphs);
    Records::give_back(&mut shared.borrow_mut().records, records);
    Some(record_event(EVENT_PERFORMANCE_REPORT, record))
}

/// A workspace held back by a failed import (see [`FailedLoad`]) is asked for its answer: when the
/// metaprogram has offered anything since, the load is tried again from scratch (true if it
/// was not and the workspace is finished instead); otherwise the error is reported and
/// followed by `ERROR` and `COMPLETE`.
fn resume_failed_load(shared: &SharedWorkspaces, id: i64) -> Result<bool, String> {
    let held = {
        let mut reg = shared.borrow_mut();
        let ws = reg.ws(id)?;
        match ws.failed_load.take() {
            Some(held) if ws.provided.len() > held.offers => {
                ws.pending = held.sources;
                ws.compiler = None;
                return Ok(false);
            }
            other => other,
        }
    };
    let Some(held) = held else {
        return Ok(false);
    };
    (shared.borrow_mut().env.report)(ReportSeverity::Error, &held.text);
    {
        let mut reg = shared.borrow_mut();
        let ws = reg.ws(id)?;
        ws.events.push_back(Event {
            kind: EVENT_ERROR,
            ints: Vec::new(),
            strings: Vec::new(),
        });
        ws.events.push_back(Event {
            kind: EVENT_COMPLETE,
            ints: vec![1],
            strings: Vec::new(),
        });
        ws.failed = true;
        ws.stage = Stage::Done;
    }
    let observer = shared.borrow_mut().env.observer.take();
    if let Some(mut o) = observer {
        o.finished(held.compiler, true);
        shared.borrow_mut().env.observer = Some(o);
    }
    Ok(true)
}

/// What [`write_output`] did.
enum Written {
    /// Whether the output was written (or not wanted).
    Done(bool),
    /// Objects are written and `READY_FOR_CUSTOM_LINK_COMMAND` queued: the metaprogram links.
    AwaitingLink,
}

/// Code generation is done: queue the write phases and call the backend.
fn write_output(
    shared: &SharedWorkspaces,
    id: i64,
    compiler: &mut Compiler,
    events: &mut Vec<Event>,
) -> Result<Written, String> {
    events.push(phase(PHASE_ALL_TARGET_CODE_BUILT));
    let settings = shared.borrow_mut().ws(id)?.settings.clone();
    if !settings.do_output || settings.output_type == OutputType::NoOutput {
        return Ok(Written::Done(true));
    }
    let output = output_path(&settings, compiler);
    let name = output.display().to_string().into_bytes();
    events.push(Event {
        kind: EVENT_PHASE,
        ints: vec![PHASE_PRE_WRITE_EXECUTABLE, 0],
        strings: vec![name.clone()],
    });
    // Output preparation prunes checked-but-unreferenced bodies and freezes
    // runtime globals for emission. A check-only embedder keeps this compiler
    // for further semantic queries; there is no output IR to prepare.
    if shared.borrow().env.backend.is_some() {
        compiler
            .prepare_compiled_output()
            .map_err(|error| compiler.render(&error))?;
    }
    let links = matches!(
        settings.output_type,
        OutputType::Executable | OutputType::DynamicLibrary
    );
    if settings.use_custom_link_command && links {
        let mut reg = shared.borrow_mut();
        if let Some(backend) = reg.env.backend.as_mut() {
            let objects = backend.write_objects(&compiler.program, &settings, &output);
            return match objects {
                Ok(o) => {
                    let mut strings: Vec<Vec<u8>> = Vec::new();
                    for list in [&o.objects, &o.system_libraries, &o.user_libraries] {
                        strings.extend(list.iter().map(|s| s.clone().into_bytes()));
                    }
                    strings.push(name);
                    events.push(Event {
                        kind: EVENT_PHASE,
                        ints: vec![
                            PHASE_READY_FOR_CUSTOM_LINK_COMMAND,
                            o.objects.len() as i64,
                            0,
                            o.system_libraries.len() as i64,
                            o.user_libraries.len() as i64,
                        ],
                        strings,
                    });
                    reg.ws(id)?.link = Some(PendingLink {
                        exit_code: None,
                        output: output.display().to_string(),
                    });
                    Ok(Written::AwaitingLink)
                }
                Err(message) => {
                    (reg.env.report)(
                        ReportSeverity::Error,
                        &format!("error: writing {}: {message}", output.display()),
                    );
                    Ok(Written::Done(false))
                }
            };
        }
    }
    let written = {
        let mut reg = shared.borrow_mut();
        match reg.env.backend.as_mut() {
            Some(backend) => backend.write_output(&compiler.program, &settings, &output),
            // No backend: checking only.
            None => {
                if let Some(hint) = reg.env.unwritten_output_hint.clone() {
                    let name = reg.ws(id)?.name.clone();
                    let kind = match settings.output_type {
                        OutputType::DynamicLibrary => "a dynamic library",
                        OutputType::StaticLibrary => "a static library",
                        OutputType::ObjectFile => "an object file",
                        _ => "an executable",
                    };
                    (reg.env.report)(
                        ReportSeverity::Warning,
                        &format!(
                            "warning: `jaic -check` does not write {} (workspace `{name}` asks for \
                         {kind})\n\
                         help: `jaic {main}` (or `jaic {main} -interp`) writes it",
                            shown_path(&output),
                            main = hint.main_file,
                        ),
                    );
                }
                Ok(())
            }
        }
    };
    if let Err(message) = &written {
        (shared.borrow_mut().env.report)(
            ReportSeverity::Error,
            &format!("error: writing {}: {message}", output.display()),
        );
    }
    events.push(Event {
        kind: EVENT_PHASE,
        ints: vec![PHASE_POST_WRITE_EXECUTABLE, written.is_err() as i64, 0],
        strings: vec![name],
    });
    Ok(Written::Done(written.is_ok()))
}

/// End a workspace whose metaprogram links it (`use_custom_link_command`): the linker's exit
/// code decides whether it failed. A metaprogram that waits for the next message without
/// saying the link is done gets an error instead of a workspace that never finishes.
fn finish_custom_link(shared: &SharedWorkspaces, id: i64) -> Result<(), String> {
    let (link, compiler) = {
        let mut reg = shared.borrow_mut();
        let ws = reg.ws(id)?;
        (ws.link.take().expect("a pending link"), ws.compiler.take())
    };
    let failed = match link.exit_code {
        Some(0) => false,
        Some(code) => {
            (shared.borrow_mut().env.report)(
                ReportSeverity::Error,
                &format!(
                    "error: the custom link command for {} failed with exit code {code}",
                    link.output
                ),
            );
            true
        }
        None => {
            (shared.borrow_mut().env.report)(
                ReportSeverity::Error,
                &format!(
                    "error: the metaprogram did not link {} after READY_FOR_CUSTOM_LINK_COMMAND\n\
                 help: call `compiler_custom_link_command_is_complete(workspace, exit_code)` once \
                 its link command has run",
                    link.output
                ),
            );
            true
        }
    };
    let mut compiler = compiler;
    let report = compiler
        .as_mut()
        .and_then(|compiler| performance_event(shared, id, compiler));
    {
        let mut reg = shared.borrow_mut();
        let ws = reg.ws(id)?;
        ws.events.push_back(Event {
            kind: EVENT_PHASE,
            ints: vec![
                PHASE_POST_WRITE_EXECUTABLE,
                failed as i64,
                link.exit_code.unwrap_or(-1),
            ],
            strings: vec![link.output.into_bytes()],
        });
        ws.events.extend(report);
        ws.events.push_back(Event {
            kind: EVENT_COMPLETE,
            ints: vec![failed as i64],
            strings: Vec::new(),
        });
        ws.failed |= failed;
        ws.stage = Stage::Done;
    }
    let observer = shared.borrow_mut().env.observer.take();
    if let (Some(mut o), Some(compiler)) = (observer, compiler) {
        o.finished(compiler, failed);
        shared.borrow_mut().env.observer = Some(o);
    }
    Ok(())
}

/// Where a workspace's output goes: `output_path/output_executable_name`
/// (default name: the first source file's stem).
fn output_path(settings: &BuildSettings, compiler: &Compiler) -> PathBuf {
    let name = if settings.output_executable_name.is_empty() {
        compiler
            .main_module
            .and_then(|m| compiler.files.iter().find(|f| f.module == m))
            .and_then(|f| f.path.file_stem())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "output".into())
    } else {
        settings.output_executable_name.clone()
    };
    PathBuf::from(&settings.output_path).join(name)
}

/// Compile every workspace created but never driven by a message loop
/// (Jai compiles them after the metaprogram's `#run` returns). Their compile-time code draws
/// from `budget`, the top-level compiler's interpreter budget.
pub fn finish_all(shared: &SharedWorkspaces, budget: &mut Option<u64>) -> Result<(), String> {
    let mut id = TOP_LEVEL_WORKSPACE + 1;
    while id < shared.borrow().list.len() as i64 {
        // A workspace nobody added a file or string to has nothing to compile; building it
        // would link an executable without `main`.
        {
            let mut reg = shared.borrow_mut();
            let ws = &mut reg.list[id as usize];
            if ws.stage == Stage::Open && ws.pending.is_empty() {
                ws.stage = Stage::Done;
            }
        }
        while shared.borrow().list[id as usize].stage != Stage::Done {
            step(shared, id, budget)?;
        }
        shared.borrow_mut().list[id as usize].events.clear();
        id += 1;
    }
    Ok(())
}

/// The primitives are grouped by what they act on; each family has its own module.
enum Family {
    Workspace,
    Messages,
    Sources,
    Host,
    ProgramData,
    Records,
}

impl Family {
    /// Exhaustive, so a new [`MetaOp`] must be given a family.
    fn of(op: MetaOp) -> Family {
        match op {
            MetaOp::WorkspaceCreate
            | MetaOp::CurrentWorkspace
            | MetaOp::ValidateWorkspace
            | MetaOp::DestroyWorkspace
            | MetaOp::WorkspaceStatus
            | MetaOp::MakeLive
            | MetaOp::AddLibraryDir
            | MetaOp::RemapImport
            | MetaOp::ProvideImport
            | MetaOp::SetText
            | MetaOp::SetFlag
            | MetaOp::SetNumber
            | MetaOp::ClearList
            | MetaOp::AppendList
            | MetaOp::OptionsSnapshot
            | MetaOp::CustomLinkComplete => Family::Workspace,
            MetaOp::BeginIntercept
            | MetaOp::EndIntercept
            | MetaOp::NextEvent
            | MetaOp::EventInt
            | MetaOp::EventString
            | MetaOp::Report => Family::Messages,
            MetaOp::AddFile
            | MetaOp::AddString
            | MetaOp::AddStringToModule
            | MetaOp::AddStringToCode
            | MetaOp::CodeIsNull
            | MetaOp::CodeNodes
            | MetaOp::ParseCode
            | MetaOp::ModifyProcedure => Family::Sources,
            MetaOp::CompilerVersion
            | MetaOp::BasePath
            | MetaOp::MainFile
            | MetaOp::MainOverride
            | MetaOp::CommandLineCount
            | MetaOp::CommandLineArg
            | MetaOp::Clang
            | MetaOp::ClangText => Family::Host,
            MetaOp::SetTypeInfoFlags
            | MetaOp::TypeTable
            | MetaOp::StructLocation
            | MetaOp::RuntimeData
            | MetaOp::DataSegment
            | MetaOp::GlobalData => Family::ProgramData,
            MetaOp::SyntaxInt
            | MetaOp::SyntaxString
            | MetaOp::SyntaxChild
            | MetaOp::RecTag
            | MetaOp::RecTagId
            | MetaOp::RecBuilder
            | MetaOp::RecType
            | MetaOp::RecPlan
            | MetaOp::RecBuild
            | MetaOp::RecOf
            | MetaOp::RecField
            | MetaOp::RecCount
            | MetaOp::RecInt
            | MetaOp::RecString
            | MetaOp::RecBuiltin
            | MetaOp::RecRef
            | MetaOp::RecItemInt
            | MetaOp::RecItemRef
            | MetaOp::RecItemString => Family::Records,
        }
    }
}

/// What one primitive call sees besides the interpreter: the registry, the program, and the
/// arguments (without the context pointer).
struct Call<'a> {
    shared: &'a SharedWorkspaces,
    program: &'a ir::Program,
    args: &'a [u64],
}

impl Call<'_> {
    fn arg(&self, i: usize) -> u64 {
        self.args.get(i).copied().unwrap_or(0)
    }

    /// Every pointer here comes from the program: reading or writing through one it made up is
    /// an error, not a crash.
    fn load(&self, interp: &Interp, p: u64) -> Result<u64, Trap> {
        interp.read_u64(p).ok_or_else(|| unreadable(p))
    }

    fn string(&self, interp: &Interp, i: usize) -> Result<Vec<u8>, Trap> {
        let p = self.arg(i);
        if p == 0 {
            return Ok(Vec::new());
        }
        let (count, data) = interp.read_pair(p).ok_or_else(|| unreadable(p))?;
        let count = count as usize;
        if count == 0 || data == 0 {
            Ok(Vec::new())
        } else {
            interp.read(data, count).ok_or_else(|| unreadable(data))
        }
    }

    fn text(&self, interp: &Interp, i: usize) -> Result<String, Trap> {
        Ok(String::from_utf8_lossy(&self.string(interp, i)?).into_owned())
    }

    fn return_string(
        &self,
        interp: &mut Interp,
        bytes: &[u8],
        out_index: usize,
    ) -> Result<(), Trap> {
        let (count, data) = self.shared.borrow_mut().keep_string(bytes);
        store_pair(interp, self.arg(out_index), count, data)
    }
}

/// Primitives that only read the registry (or make records nobody else sees yet) leave a
/// compile-time run repeatable; the rest are effects.
fn is_effect(op: MetaOp) -> bool {
    !matches!(
        op,
        MetaOp::CurrentWorkspace
            | MetaOp::ValidateWorkspace
            | MetaOp::RuntimeData
            | MetaOp::EventInt
            | MetaOp::EventString
            | MetaOp::CommandLineCount
            | MetaOp::CommandLineArg
            | MetaOp::CompilerVersion
            | MetaOp::StructLocation
            | MetaOp::BasePath
            | MetaOp::MainFile
            | MetaOp::MainOverride
            | MetaOp::TypeTable
            | MetaOp::CodeIsNull
            | MetaOp::CodeNodes
            | MetaOp::ParseCode
            | MetaOp::RecTag
            | MetaOp::RecField
            | MetaOp::RecInt
            | MetaOp::RecString
            | MetaOp::RecRef
            | MetaOp::RecCount
            | MetaOp::RecItemInt
            | MetaOp::RecItemString
            | MetaOp::RecItemRef
            | MetaOp::RecTagId
            | MetaOp::RecBuilder
            | MetaOp::RecType
            | MetaOp::RecBuiltin
            | MetaOp::RecPlan
            | MetaOp::RecBuild
            | MetaOp::RecOf
            | MetaOp::SyntaxInt
            | MetaOp::SyntaxString
            | MetaOp::SyntaxChild
    )
}

/// Execute a primitive. `args` follow the IR calling convention: the context
/// pointer (when `has_context`), the parameters (memory types by address),
/// then result addresses for memory-typed results.
pub fn call(
    shared: &SharedWorkspaces,
    op: MetaOp,
    has_context: bool,
    args: &[u64],
    program: &ir::Program,
    interp: &mut Interp,
) -> Result<Rets, Trap> {
    let args = if has_context {
        &args[1..]
    } else {
        args
    };
    if is_effect(op) {
        interp.effects += 1;
    }
    let cx = Call {
        shared,
        program,
        args,
    };
    match Family::of(op) {
        Family::Workspace => workspace::call(&cx, op, interp),
        Family::Messages => messages::call(&cx, op, interp),
        Family::Sources => sources::call(&cx, op, interp),
        Family::Host => host::call(&cx, op, interp),
        Family::ProgramData => program_data::call(&cx, op, interp),
        Family::Records => records::call(&cx, op, interp),
    }
}

/// The error for a program pointer the compiler could not read.
fn unreadable(p: u64) -> Trap {
    Trap::new(format!("invalid memory access: {p:#x} cannot be read"))
}

/// Store `bytes` at program address `p`, or the error for one that cannot be written.
fn store(interp: &mut Interp, p: u64, bytes: &[u8]) -> Result<(), Trap> {
    if interp.write(p, bytes) {
        Ok(())
    } else {
        Err(Trap::new(format!(
            "invalid memory access: {p:#x} cannot be written"
        )))
    }
}

/// Store a Jai `string` or array view (`count`, `data`) at program address `p`.
fn store_pair(interp: &mut Interp, p: u64, count: u64, data: u64) -> Result<(), Trap> {
    store(interp, p, &count.to_le_bytes())?;
    store(interp, p + 8, &data.to_le_bytes())
}
