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

/// What a workspace produces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputType {
    Executable,
    DynamicLibrary,
    StaticLibrary,
    ObjectFile,
    NoOutput,
}

impl OutputType {
    fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "EXECUTABLE" => Self::Executable,
            "DYNAMIC_LIBRARY" => Self::DynamicLibrary,
            "STATIC_LIBRARY" => Self::StaticLibrary,
            "OBJECT_FILE" => Self::ObjectFile,
            "NO_OUTPUT" => Self::NoOutput,
            _ => return None,
        })
    }
}

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
    /// `O0`..`O3`, `OS`, `OZ` (`bitcode_optimization_setting` names).
    pub optimization: String,
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
    pub llvm_triple: String,
    pub llvm_cpu: String,
    pub llvm_features: String,
    /// `intermediate_path`: where object files (and LLVM IR or bitcode) go; empty: next to
    /// the output.
    pub intermediate_path: String,
    /// `entry_point_name`: the procedure the program starts in; empty: `main`.
    pub entry_point_name: String,
    /// `runtime_support_definitions` (`AUTO`...) and `backtrace_on_crash`, once a metaprogram
    /// changes them.
    pub runtime_support_definitions: Option<String>,
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
    pub machine_code_optimization: String,
    pub llvm_switches: Vec<(String, bool)>,
    /// `output_llvm_ir`, `output_bitcode` and their `_before_optimizations` variants.
    pub output_llvm_ir: bool,
    pub output_bitcode: bool,
    pub output_llvm_ir_before_optimizations: bool,
    pub output_bitcode_before_optimizations: bool,
}

impl BuildSettings {
    /// The value a metaprogram gave `llvm_options.<name>` (`enable_loop_unrolling`...).
    pub fn llvm_switch(&self, name: &str) -> Option<bool> {
        self.llvm_switches
            .iter()
            .rev()
            .find(|(n, _)| n == name)
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
            optimization: String::new(),
            additional_linker_arguments: Vec::new(),
            temporary_storage_size: None,
            array_bounds_check: None,
            null_pointer_check: None,
            arithmetic_overflow_check: None,
            cast_bounds_check: None,
            stack_trace: None,
            emit_debug_info: None,
            dead_code_elimination: None,
            llvm_triple: String::new(),
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
            machine_code_optimization: String::new(),
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
    pub report: Box<dyn FnMut(&str)>,
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
    fn new(name: String) -> Self {
        Workspace {
            name,
            pending: Vec::new(),
            settings: BuildSettings::default(),
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
    SetOption,
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
    Clang,
    ClangText,
}

impl MetaOp {
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "__jaic_workspace_create" => Self::WorkspaceCreate,
            "__jaic_current_workspace" => Self::CurrentWorkspace,
            "__jaic_workspace_validate" => Self::ValidateWorkspace,
            "__jaic_data_segment" => Self::DataSegment,
            "__jaic_global_data" => Self::GlobalData,
            "__jaic_runtime_data" => Self::RuntimeData,
            "__jaic_workspace_add_file" => Self::AddFile,
            "__jaic_workspace_add_string" => Self::AddString,
            "__jaic_workspace_set_option" => Self::SetOption,
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
            "__jaic_clang" => Self::Clang,
            "__jaic_clang_text" => Self::ClangText,
            _ => return None,
        })
    }
}

/// Id of the program being compiled by the command line (workspace 1 is reserved, as in `jai`).
pub const TOP_LEVEL_WORKSPACE: i64 = 2;

pub const COMPILER_VERSION: &str = concat!("jaic ", env!("CARGO_PKG_VERSION"));

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

    fn set_option(&mut self, id: i64, key: &str, value: &str) -> Result<(), String> {
        let s = &mut self.ws(id)?.settings;
        let flag = || value == "true";
        match key {
            "output_executable_name" => s.output_executable_name = value.into(),
            "output_path" => s.output_path = value.into(),
            "output_type" => {
                s.output_type = OutputType::parse(value)
                    .ok_or_else(|| format!("unknown output_type `{value}`"))?
            }
            "do_output" => s.do_output = flag(),
            "import_path_clear" => s.import_paths = Some(Vec::new()),
            "import_path" => s
                .import_paths
                .get_or_insert_with(Vec::new)
                .push(PathBuf::from(value)),
            "os_target" => {
                s.os = Some(match value {
                    "WINDOWS" => TargetOs::Windows,
                    "LINUX" => TargetOs::Linux,
                    "MACOS" => TargetOs::MacOS,
                    "WASM" => TargetOs::Wasm,
                    _ => return Err(format!("jaic cannot build for `os_target = .{value}`")),
                })
            }
            "cpu_target" => {
                s.cpu = Some(match value {
                    "X64" => TargetCpu::X64,
                    "ARM64" => TargetCpu::Arm64,
                    // `.CUSTOM` is how a wasm target spells its CPU (the triple says which).
                    "WASM" | "CUSTOM" => TargetCpu::Wasm,
                    _ => return Err(format!("jaic cannot build for `cpu_target = .{value}`")),
                })
            }
            "optimization" => s.optimization = value.into(),
            "additional_linker_arguments_clear" => s.additional_linker_arguments.clear(),
            "additional_linker_argument" => s.additional_linker_arguments.push(value.into()),
            "temporary_storage_size" => s.temporary_storage_size = value.parse().ok(),
            "array_bounds_check" => s.array_bounds_check = Some(value != "OFF"),
            "null_pointer_check" => s.null_pointer_check = Some(value != "OFF"),
            "arithmetic_overflow_check" => s.arithmetic_overflow_check = Some(check_level(value)),
            "cast_bounds_check" => s.cast_bounds_check = Some(check_level(value)),
            "stack_trace" => s.stack_trace = Some(value == "true"),
            "emit_debug_info" => s.emit_debug_info = Some(value != "NONE"),
            "dead_code_elimination" => s.dead_code_elimination = DeadCode::from_name(value),
            "llvm_target_system_triple" => s.llvm_triple = value.into(),
            "llvm_target_system_cpu" => s.llvm_cpu = value.into(),
            "llvm_target_system_features" => s.llvm_features = value.into(),
            "intermediate_path" => s.intermediate_path = value.into(),
            "entry_point_name" => s.entry_point_name = value.into(),
            "runtime_support_definitions" => s.runtime_support_definitions = Some(value.into()),
            "backtrace_on_crash" => s.backtrace_on_crash = Some(value == "ON"),
            "append_executable_filename_extension" => s.append_extension = flag(),
            "use_custom_link_command" => s.use_custom_link_command = flag(),
            "minimum_os_version" => s.minimum_os_version = value.into(),
            "enable_frame_pointers" => s.frame_pointers = Some(flag()),
            "disable_redzone" => s.disable_redzone = flag(),
            "machine_code_optimization" => s.machine_code_optimization = value.into(),
            "output_llvm_ir" => s.output_llvm_ir = flag(),
            "output_bitcode" => s.output_bitcode = flag(),
            "output_llvm_ir_before_optimizations" => s.output_llvm_ir_before_optimizations = flag(),
            "output_bitcode_before_optimizations" => s.output_bitcode_before_optimizations = flag(),
            "destroy" => {
                // `compiler_destroy_workspace`: what was added is never compiled.
                let ws = self.ws(id)?;
                ws.pending.clear();
                if ws.stage == Stage::Open {
                    ws.stage = Stage::Done;
                }
            }
            _ => match key.strip_prefix("llvm_") {
                Some(name) => s.llvm_switches.push((name.into(), flag())),
                // The Compiler module sends only keys listed here: anything else is a bug there,
                // not a setting to drop.
                None => return Err(format!("internal: unknown build option `{key}`")),
            },
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

/// A runtime check's level from its `Build_Options` enum name: 0 `OFF`, 1 `NONFATAL`, 2 `FATAL`.
fn check_level(value: &str) -> u8 {
    match value {
        "NONFATAL" => 1,
        "FATAL" => 2,
        _ => 0,
    }
}

/// The module that makes a wasm program a WASI command (`_start`, the C library subset the
/// stdlib calls, an allocator); see `docs/native/wasm-target.md`.
pub const WASI_RUNTIME_IMPORT: &str = "#import \"Extensions/Wasi_Runtime\";";

/// Whether a build for `os` with LLVM triple `triple` is a WASI program, which gets
/// [`WASI_RUNTIME_IMPORT`] added to its sources. Other wasm builds supply their own runtime.
pub fn wants_wasi_runtime(os: TargetOs, triple: &str) -> bool {
    os == TargetOs::Wasm && triple.split('-').any(|part| part.starts_with("wasi"))
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
    match reg.list.get_mut(id as usize) {
        Some(ws) if ws.stage != Stage::Open => std::mem::take(&mut ws.pending),
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
        options.runtime_entry_point = !matches!(which.as_str(), "ONLY_INIT" | "OMIT");
        options.runtime_initialization = which != "OMIT";
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
        if wants_wasi_runtime(compiler.options.os, &settings.llvm_triple) {
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
                (shared.borrow_mut().env.report)(&text);
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
    (shared.borrow_mut().env.report)(&held.text);
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
    compiler.prepare_compiled_output();
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
                    (reg.env.report)(&format!("error: writing {}: {message}", output.display()));
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
                    (reg.env.report)(&format!(
                        "warning: `jaic -check` does not write {} (workspace `{name}` asks for \
                         {kind})\n\
                         help: `jaic {main}` (or `jaic {main} -interp`) writes it",
                        shown_path(&output),
                        main = hint.main_file,
                    ));
                }
                Ok(())
            }
        }
    };
    if let Err(message) = &written {
        (shared.borrow_mut().env.report)(&format!(
            "error: writing {}: {message}",
            output.display()
        ));
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
            (shared.borrow_mut().env.report)(&format!(
                "error: the custom link command for {} failed with exit code {code}",
                link.output
            ));
            true
        }
        None => {
            (shared.borrow_mut().env.report)(&format!(
                "error: the metaprogram did not link {} after READY_FOR_CUSTOM_LINK_COMMAND\n\
                 help: call `compiler_custom_link_command_is_complete(workspace, exit_code)` once \
                 its link command has run",
                link.output
            ));
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
    let arg = |i: usize| args.get(i).copied().unwrap_or(0);
    // Primitives that only read the registry (or make records nobody else sees yet) leave a
    // compile-time run repeatable; the rest are effects.
    if !matches!(
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
    ) {
        interp.effects += 1;
    }
    let trap = Trap::new;
    // Every pointer here comes from the program: reading or writing through one it made up is
    // an error, not a crash.
    let load = |interp: &Interp, p: u64| interp.read_u64(p).ok_or_else(|| unreadable(p));
    let string = |interp: &Interp, i: usize| -> Result<Vec<u8>, Trap> {
        let p = arg(i);
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
    };
    let text = |interp: &Interp, i: usize| -> Result<String, Trap> {
        Ok(String::from_utf8_lossy(&string(interp, i)?).into_owned())
    };
    let return_string = |interp: &mut Interp, bytes: &[u8], out_index: usize| {
        let (count, data) = shared.borrow_mut().keep_string(bytes);
        store_pair(interp, arg(out_index), count, data)
    };
    match op {
        MetaOp::SetTypeInfoFlags => {
            // (type: Type, flags): applied by the compiler after the running `#run` returns.
            let Some((global, 0)) = interp.global_at(arg(0)) else {
                return Err(trap("compiler_set_type_info_flags: not a type".into()));
            };
            interp.pending_type_flags.push((global, arg(1) as u32));
            Ok(Rets::default())
        }
        MetaOp::TypeTable => {
            // (workspace, out table): the descriptors of every type of the workspace's program
            // so far. The running workspace's table was made when its compile-time run began;
            // another workspace's compiler is between steps, so it makes one now.
            let id = arg(0) as i64;
            let current = shared.borrow().current_id();
            let (table, count) = if id < 0 || id == current {
                interp.ct_type_table
            } else {
                let taken = shared.borrow_mut().ws(id).map_err(trap)?.compiler.take();
                let Some(mut compiler) = taken else {
                    return Err(trap(
                        "get_type_table: that workspace has nothing compiled yet".into(),
                    ));
                };
                shared.borrow_mut().current.push(id);
                compiler.refresh_type_table();
                shared.borrow_mut().current.pop();
                let found = compiler.interp.ct_type_table;
                shared.borrow_mut().ws(id).map_err(trap)?.compiler = Some(compiler);
                found
            };
            store_pair(interp, arg(1), count, table)?;
            Ok(Rets::default())
        }
        MetaOp::StructLocation => {
            // (info, out path, out line, out column); unknown structs leave them zero.
            let found = interp
                .global_at(arg(0))
                .and_then(|(g, offset)| (offset == 0).then_some(g))
                .and_then(|g| interp.struct_locations.get(&g).cloned());
            if let Some((path, line, col)) = found {
                return_string(interp, path.as_bytes(), 1)?;
                store(interp, arg(2), &line.to_le_bytes())?;
                store(interp, arg(3), &col.to_le_bytes())?;
            }
            Ok(Rets::default())
        }
        MetaOp::BasePath => {
            // The install root: the directory above the stdlib, which is the last module
            // directory on the import path.
            let base = shared
                .borrow()
                .env
                .options
                .import_paths
                .last()
                .and_then(|stdlib| stdlib.parent())
                .map(|p| {
                    let mut text = crate::canonicalize(p)
                        .unwrap_or_else(|_| p.to_path_buf())
                        .display()
                        .to_string();
                    if !text.ends_with('/') && !text.ends_with('\\') {
                        text.push('/');
                    }
                    text
                })
                .unwrap_or_default();
            return_string(interp, base.as_bytes(), 0)?;
            Ok(Rets::default())
        }
        MetaOp::MainFile => {
            let file = shared.borrow().main_file.clone();
            return_string(interp, file.as_bytes(), 0)?;
            Ok(Rets::default())
        }
        MetaOp::MainOverride => {
            // 0: executable name, 1: output path, 2: `OFF` when the crash handler is not wanted.
            let value = {
                let reg = shared.borrow();
                let overrides = &reg.main_overrides;
                match arg(0) {
                    0 => overrides.exe.clone(),
                    1 => overrides.output_path.clone(),
                    _ if overrides.no_backtrace_on_crash => "OFF".to_string(),
                    _ => String::new(),
                }
            };
            return_string(interp, value.as_bytes(), 1)?;
            Ok(Rets::default())
        }
        MetaOp::AddLibraryDir => {
            let dir = text(interp, 0)?;
            crate::interp::add_library_dir(PathBuf::from(dir));
            Ok(Rets::default())
        }
        MetaOp::RemapImport => {
            let id = arg(0) as i64;
            let rule = (text(interp, 1)?, text(interp, 2)?, text(interp, 3)?);
            shared.borrow_mut().ws(id).map_err(trap)?.remaps.push(rule);
            Ok(Rets::default())
        }
        MetaOp::ProvideImport => {
            let id = arg(0) as i64;
            let kind = match arg(3) {
                0 => ProvidedKind::ShortName,
                1 => ProvidedKind::PathToFile,
                2 => ProvidedKind::PathToDirectory,
                _ => ProvidedKind::FullText,
            };
            let offer = ProvidedImport {
                host: text(interp, 1)?,
                target: text(interp, 2)?,
                kind,
                value: text(interp, 4)?,
            };
            // An answer already given adds nothing: the load would fail the same way again, and a
            // metaprogram that answers every FAILED_IMPORT with it (a file the host cannot
            // see, say) would loop forever, each retry counting as a new offer.
            let mut reg = shared.borrow_mut();
            let provided = &mut reg.ws(id).map_err(trap)?.provided;
            if !provided.contains(&offer) {
                provided.push(offer);
            }
            Ok(Rets::default())
        }
        MetaOp::MakeLive => {
            let id = arg(0) as i64;
            shared
                .borrow_mut()
                .ws(id)
                .map_err(trap)?
                .live
                .push(arg(1) as i64);
            Ok(Rets::default())
        }
        MetaOp::WorkspaceCreate => {
            let name = text(interp, 0)?;
            let mut reg = shared.borrow_mut();
            let ws = Workspace::new(name);
            reg.list.push(ws);
            Ok(Rets::one(reg.list.len() as u64 - 1))
        }
        MetaOp::CurrentWorkspace => Ok(Rets::one(shared.borrow().current_id() as u64)),
        MetaOp::ValidateWorkspace => {
            shared.borrow_mut().ws(arg(0) as i64).map_err(trap)?;
            Ok(Rets::default())
        }
        MetaOp::RuntimeData => {
            let id = arg(0) as i64;
            let mut reg = shared.borrow_mut();
            reg.ws(id).map_err(trap)?;
            if id != reg.current_id() {
                return Err(trap(
                    "get_runtime_info: global-data snapshots require the running workspace".into(),
                ));
            }
            Ok(Rets::one(interp.runtime_data_snapshot(program)?))
        }
        MetaOp::DataSegment => {
            let name = string(interp, 0)?;
            let characteristics = arg(1) as u32;
            let alignment = arg(2) as i32;
            let workspace = arg(3) as i64;
            let mut reg = shared.borrow_mut();
            reg.ws(workspace).map_err(trap)?;
            if workspace != reg.current_id() {
                return Err(trap(
                    "add_data_segment: registration requires the running workspace".into(),
                ));
            }
            if characteristics & !0xf != 0 || characteristics & 4 != 0 {
                return Err(trap("add_data_segment: executable or unknown segment characteristics are unavailable".into()));
            }
            if alignment <= 0 || !(alignment as u32).is_power_of_two() {
                return Err(trap(
                    "add_data_segment: alignment must be a positive power of two".into(),
                ));
            }
            let segment = DataSegment {
                // Opaque handles must survive #run freezing and relocation without
                // being copied to an unrelated address. The interpreter adopts this
                // allocation as a real compiler-owned global after the run.
                handle: interp.register_data_global(&[0; 16], 16, true, 2)?,
                _name: name,
                workspace,
                characteristics,
                alignment: alignment as u64,
            };
            let address = segment.handle;
            reg.data_segments.push(segment);
            Ok(Rets::one(address))
        }
        MetaOp::GlobalData => {
            let workspace = arg(0) as i64;
            let bytes = string(interp, 1)?;
            let segment = arg(2) as u16;
            let mut reg = shared.borrow_mut();
            reg.ws(workspace).map_err(trap)?;
            if workspace != reg.current_id() {
                return Err(trap(
                    "add_global_data: registration requires the running workspace".into(),
                ));
            }
            let (align, read_only, zeroed, tag) = match segment {
                0 => (16, false, false, 1),
                2 => (16, true, false, 2),
                3 => (16, false, true, 0),
                0x10 => {
                    let handle = arg(3);
                    let descriptor = reg
                        .data_segments
                        .iter()
                        .find(|s| s.handle == handle && s.workspace == workspace)
                        .ok_or_else(|| {
                            trap(
                                "add_global_data: invalid user segment handle for this workspace"
                                    .into(),
                            )
                        })?;
                    (
                        descriptor.alignment,
                        descriptor.characteristics & 2 == 0,
                        descriptor.characteristics & 8 != 0,
                        5,
                    )
                }
                1 => {
                    return Err(trap(
                        "add_global_data: WRITABLE_NO_RESET reset semantics are unavailable".into(),
                    ));
                }
                _ => return Err(trap("add_global_data: unknown segment index".into())),
            };
            if zeroed && bytes.iter().any(|&b| b != 0) {
                return Err(trap(
                    "add_global_data: nonzero input for BSS or ZEROED is not supported".into(),
                ));
            }
            let address = interp.register_data_global(&bytes, align, read_only, tag)?;
            store_pair(interp, arg(4), bytes.len() as u64, address)?;
            Ok(Rets::default())
        }
        MetaOp::AddFile | MetaOp::AddString => {
            let id = arg(0) as i64;
            let value = text(interp, 1)?;
            let mut reg = shared.borrow_mut();
            let ws = reg.ws(id).map_err(trap)?;
            if ws.stage == Stage::Done {
                return Err(trap(format!(
                    "workspace `{}` has already been compiled; add its sources before it completes",
                    ws.name
                )));
            }
            let fs = reg.env.fs.clone();
            // Resolved now: the metaprogram may change directory before it compiles.
            let cwd = interp
                .host
                .working_directory()
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let path = fs.canonical(&cwd.join(&value));
            if op == MetaOp::AddFile && !fs.is_file(&path) {
                return Err(trap(if fs.is_dir(&path) {
                    format!("add_build_file: `{value}` is a directory, not a .jai file")
                } else {
                    format!(
                        "add_build_file: file `{value}` does not exist (relative paths start from the metaprogram's directory, {})",
                        cwd.display()
                    )
                }));
            }
            let ws = reg.ws(id).map_err(trap)?;
            ws.pending.push(if op == MetaOp::AddFile {
                ProgramSource::File(path)
            } else {
                ProgramSource::String(value)
            });
            Ok(Rets::default())
        }
        MetaOp::SetOption => {
            let (key, mut value) = (text(interp, 1)?, text(interp, 2)?);
            // A module folder relative to a sandbox's own working directory: the process's is
            // somewhere else.
            if key == "import_path"
                && Path::new(&value).is_relative()
                && let Some(cwd) = interp.host.working_directory()
            {
                value = Path::new(&cwd).join(&value).to_string_lossy().into_owned();
            }
            shared
                .borrow_mut()
                .set_option(arg(0) as i64, &key, &value)
                .map_err(trap)?;
            Ok(Rets::default())
        }
        MetaOp::BeginIntercept => {
            #[cfg(target_arch = "wasm32")]
            if arg(1) as i64 & (INTERCEPT_REPORT_RUNS | INTERCEPT_REPORT_POLYMORPHS) != 0 {
                return Err(trap("performance reporting requires a host monotonic clock; this WASM compiler host does not provide one".into()));
            }
            let mut reg = shared.borrow_mut();
            let ws = reg.ws(arg(0) as i64).map_err(trap)?;
            ws.intercepted = true;
            ws.intercept_flags = arg(1) as i64;
            Ok(Rets::default())
        }
        MetaOp::EndIntercept => {
            shared
                .borrow_mut()
                .ws(arg(0) as i64)
                .map_err(trap)?
                .intercepted = false;
            Ok(Rets::default())
        }
        MetaOp::WorkspaceStatus => {
            let mut reg = shared.borrow_mut();
            let ws = reg.ws(arg(0) as i64).map_err(trap)?;
            match arg(1) {
                0 if ws.failed => {
                    return Err(trap("a failed workspace cannot be reset to OK".into()));
                }
                0 => {}
                1 => ws.failed = true,
                status => return Err(trap(format!("invalid workspace status {status}"))),
            }
            Ok(Rets::default())
        }
        MetaOp::NextEvent => {
            let id = arg(0) as i64;
            if id == shared.borrow().current_id() || id == TOP_LEVEL_WORKSPACE {
                // A workspace cannot wait on its own compilation.
                return Ok(Rets::one(0));
            }
            loop {
                let mut reg = shared.borrow_mut();
                let ws = reg.ws(id).map_err(trap)?;
                if let Some(event) = ws.events.pop_front() {
                    let kind = event.kind;
                    reg.event = event;
                    return Ok(Rets::one(kind as u64));
                }
                if ws.stage == Stage::Done {
                    return Ok(Rets::one(0));
                }
                drop(reg);
                let mut budget = interp.block_budget;
                let stepped = step(shared, id, &mut budget);
                interp.block_budget = budget;
                stepped.map_err(trap)?;
            }
        }
        MetaOp::EventInt => {
            let reg = shared.borrow();
            Ok(Rets::one(
                reg.event.ints.get(arg(0) as usize).copied().unwrap_or(0) as u64,
            ))
        }
        MetaOp::EventString => {
            let bytes = shared
                .borrow()
                .event
                .strings
                .get(arg(0) as usize)
                .cloned()
                .unwrap_or_default();
            return_string(interp, &bytes, 1)?;
            Ok(Rets::default())
        }
        MetaOp::CommandLineCount => Ok(Rets::one(shared.borrow().env.command_line.len() as u64)),
        MetaOp::CommandLineArg => {
            let value = shared
                .borrow()
                .env
                .command_line
                .get(arg(0) as usize)
                .cloned()
                .unwrap_or_default();
            return_string(interp, value.as_bytes(), 1)?;
            Ok(Rets::default())
        }
        MetaOp::Report => {
            let (message, file) = (text(interp, 0)?, text(interp, 1)?);
            // `mode`: `Report` in the Compiler module (ERROR, ERROR_CONTINUABLE, WARNING, INFO).
            let (line, column, mode) = (arg(2), arg(3), arg(4));
            let location = if file.is_empty() {
                String::new()
            } else {
                format!("{file}:{line}:{column}: ")
            };
            if mode == 0 {
                let mut t = trap(message.trim_end().to_string());
                t.reported = Some(Box::new((file, line as u32, column as u32)));
                return Err(t);
            }
            let severity = match mode {
                1 => "error",
                2 => "warning",
                _ => "info",
            };
            let rendered = format!("{location}{severity}: {}", message.trim_end());
            let mut reg = shared.borrow_mut();
            (reg.env.report)(&rendered);
            if mode == 1 {
                // A continuable error: the metaprogram goes on, but the build has failed.
                let current = reg.current_id();
                reg.ws(current).map_err(trap)?.failed = true;
            }
            Ok(Rets::default())
        }
        MetaOp::CompilerVersion => {
            return_string(interp, COMPILER_VERSION.as_bytes(), 0)?;
            Ok(Rets::default())
        }
        MetaOp::CustomLinkComplete => {
            let (id, exit_code) = (arg(0) as i64, arg(1) as i64);
            let mut reg = shared.borrow_mut();
            match reg.ws(id).map_err(trap)?.link.as_mut() {
                Some(link) => {
                    link.exit_code = Some(exit_code);
                    Ok(Rets::default())
                }
                None => Err(trap(format!(
                    "compiler_custom_link_command_is_complete: workspace {id} is not waiting \
                     for a custom link (it gets READY_FOR_CUSTOM_LINK_COMMAND first, with \
                     `use_custom_link_command` set)"
                ))),
            }
        }
        MetaOp::AddStringToModule => {
            let (id, value, record) = (arg(0) as i64, text(interp, 1)?, arg(2) as i64);
            let mut reg = shared.borrow_mut();
            if let Some(Item::Int(file)) = reg.records.item(record, "__file", None) {
                reg.ws(id)
                    .map_err(trap)?
                    .pending
                    .push(ProgramSource::FileString(
                        value,
                        crate::source::FileId(file as u32),
                    ));
                return Ok(Rets::default());
            }
            // A FILE message stands for its module (the enclosing import).
            let records = &reg.records;
            let message = match records.item(record, "enclosing_import", None) {
                Some(Item::Ref(import)) => import,
                _ => record,
            };
            let module = match records.item(message, "__module", None) {
                Some(Item::Int(m)) => m as u32,
                _ => return Err(trap("add_build_string: the message names no module".into())),
            };
            let ws = reg.ws(id).map_err(trap)?;
            ws.pending.push(ProgramSource::ModuleString(value, module));
            Ok(Rets::default())
        }
        MetaOp::AddStringToCode => {
            let id = arg(0) as i64;
            let source = text(interp, 1)?;
            let code = arg(2);
            let mut reg = shared.borrow_mut();
            if id != reg.current_id() {
                return Err(trap(
                    "add_build_string: Code scope must belong to the running workspace".into(),
                ));
            }
            if code >= interp.codes.len() as u64 {
                return Err(trap("add_build_string: invalid Code scope".into()));
            }
            reg.ws(id)
                .map_err(trap)?
                .pending
                .push(ProgramSource::CodeString(
                    source,
                    crate::sema::value::CodeId(code as u32),
                ));
            Ok(Rets::default())
        }
        MetaOp::CodeIsNull => {
            let null = interp.codes.get(arg(0) as usize).is_some_and(|(body, _)| {
                matches!(&**body, crate::ast::CodeBody::Expr(e) if matches!(e.kind, crate::ast::ExprKind::Null))
            });
            Ok(Rets::one(null as u64))
        }
        MetaOp::CodeNodes => {
            let code = arg(0) as usize;
            let Some((body, text)) = interp.codes.get(code).cloned() else {
                return Err(trap("compiler_get_nodes: not a Code value".into()));
            };
            interp.nodes_codes.retain(|&c| c != code);
            interp.nodes_codes.push(code);
            let mut reg = shared.borrow_mut();
            let taken = interp.code_export_cursor.entry(code).or_default();
            if let Some((root, nodes)) = interp.code_exports.get(&code).and_then(|e| e.get(*taken))
            {
                *taken += 1;
                let mut result = crate::records::Record::new("Code_Nodes");
                result
                    .ptr("root", *root)
                    .refs("expressions", nodes.iter().copied());
                return Ok(Rets::one(reg.records.add(result) as u64));
            }
            // Names and types need the compiler: while nothing observable happened in this
            // compile-time run, ask it to export the code and run again (`call_thunk`).
            if interp.run_effects == Some(interp.effects) {
                interp.export_request = Some(code);
                return Err(trap("compiler_get_nodes: typed export requested".into()));
            }
            let (root, nodes) =
                crate::sema::code_export::export_code(&mut reg.records, &body, &text);
            let mut result = crate::records::Record::new("Code_Nodes");
            result.ptr("root", root).refs("expressions", nodes);
            Ok(Rets::one(reg.records.add(result) as u64))
        }
        MetaOp::ParseCode => {
            let (source, scope_from) = (text(interp, 0)?, arg(1) as usize);
            if scope_from >= interp.codes.len() {
                return Err(trap(
                    "compiler_get_code: no code to take the scope from".into(),
                ));
            }
            // A compile-time run that asks for typed nodes is run again from the start
            // (`export_request`), and does what the earlier attempt did: its n-th code is the
            // n-th code that attempt made, or it would ask for that new code's nodes again.
            let (made, replayed) = &mut interp.run_codes;
            if let Some(&id) = made.get(*replayed)
                && interp.made_codes.contains(&(id, scope_from))
                && interp.codes.get(id).is_some_and(|c| *c.1 == *source)
            {
                *replayed += 1;
                return Ok(Rets::one(id as u64));
            }
            let body = parse_code_text(crate::source::FileId(u32::MAX), &source)
                .map_err(|e| trap(format!("compiler_get_code: {e}")))?;
            let id = interp.codes.len();
            interp.codes.push((body, source.into()));
            interp.made_codes.push((id, scope_from));
            let (made, replayed) = &mut interp.run_codes;
            made.truncate(*replayed);
            made.push(id);
            *replayed = made.len();
            Ok(Rets::one(id as u64))
        }
        MetaOp::ModifyProcedure => {
            let (id, body, data, sources, count) =
                (arg(0) as i64, arg(1) as i64, arg(2), arg(3), arg(4));
            // A statement without a record (new, or edited in place) comes as source text.
            let stmts = (0..count)
                .map(|i| {
                    Ok(match load(interp, data + i * 8)? as i64 {
                        0 => {
                            let at = sources + i * 16;
                            let (len, ptr) = (load(interp, at)?, load(interp, at + 8)?);
                            let bytes = interp
                                .read(ptr, len as usize)
                                .ok_or_else(|| unreadable(ptr))?;
                            crate::sema::ModifiedStmt::Source(
                                String::from_utf8_lossy(&bytes).into_owned(),
                            )
                        }
                        r => crate::sema::ModifiedStmt::Record(r),
                    })
                })
                .collect::<Result<_, Trap>>()?;
            let mut reg = shared.borrow_mut();
            reg.ws(id).map_err(trap)?.modifications.push((body, stmts));
            Ok(Rets::default())
        }
        MetaOp::Clang => {
            let (name, bytes) = (text(interp, 0)?, string(interp, 3)?);
            crate::clang::call(&name, arg(1) as i64, arg(2) as i64, &bytes)
                .map(|v| Rets::one(v as u64))
                .map_err(trap)
        }
        MetaOp::ClangText => {
            return_string(interp, &crate::clang::last_text(), 0)?;
            Ok(Rets::default())
        }
        MetaOp::RecTag => {
            // Tags are static: hand out the text itself.
            let tag = shared
                .borrow()
                .records
                .get(arg(0) as i64)
                .map_or("", |r| r.tag);
            store_pair(interp, arg(1), tag.len() as u64, tag.as_ptr() as u64)?;
            Ok(Rets::default())
        }
        MetaOp::RecTagId => {
            // A small number per distinct tag, so Jai can cache tag -> struct type in an array.
            let mut reg = shared.borrow_mut();
            let Some(tag) = reg.records.get(arg(0) as i64).map(|r| r.tag) else {
                return Ok(Rets::one(u64::MAX));
            };
            let next = reg.tags.len();
            let id = *reg.tags.entry(tag).or_insert(next);
            Ok(Rets::one(id as u64))
        }
        MetaOp::RecBuilder => {
            let mut reg = shared.borrow_mut();
            reg.builders.push(Default::default());
            Ok(Rets::one(reg.builders.len() as u64))
        }
        MetaOp::RecType | MetaOp::RecBuiltin => {
            let name = text(interp, 1)?;
            let mut reg = shared.borrow_mut();
            let builder = builder(&mut reg, arg(0))?;
            if op == MetaOp::RecType {
                builder.add_type(name, arg(2));
            } else {
                builder.add_builtin(name, arg(2));
            }
            Ok(Rets::default())
        }
        MetaOp::RecPlan => {
            let mut reg = shared.borrow_mut();
            let builder = builder(&mut reg, arg(0))?;
            builder.add_plan(interp, arg(1), arg(2), arg(3), arg(4), arg(5) & 1 != 0)?;
            Ok(Rets::default())
        }
        MetaOp::RecBuild => {
            let mut reg = shared.borrow_mut();
            let Workspaces {
                records,
                kept,
                builders,
                ..
            } = &mut *reg;
            let builder = (arg(0) as usize)
                .checked_sub(1)
                .and_then(|at| builders.get_mut(at))
                .ok_or_else(|| trap("records: no such builder".into()))?;
            Ok(Rets::one(builder.build(
                records,
                kept,
                interp,
                arg(1) as i64,
                arg(2),
            )?))
        }
        MetaOp::RecOf => {
            let mut reg = shared.borrow_mut();
            Ok(Rets::one(
                builder(&mut reg, arg(0))?.record_of(arg(1)) as u64
            ))
        }
        MetaOp::RecField => Ok(Rets::one(field_name(interp, arg(1), |name| {
            shared.borrow().records.kind(arg(0) as i64, name) as u64
        }))),
        MetaOp::RecCount => {
            let reg = shared.borrow();
            Ok(Rets::one(field_name(interp, arg(1), |name| {
                match reg.records.field(arg(0) as i64, name) {
                    Some(FieldRef::List(items)) => items.len() as u64,
                    _ => 0,
                }
            })))
        }
        MetaOp::RecInt | MetaOp::RecRef | MetaOp::RecItemInt | MetaOp::RecItemRef => {
            let index =
                matches!(op, MetaOp::RecItemInt | MetaOp::RecItemRef).then(|| arg(2) as usize);
            let reg = shared.borrow();
            Ok(Rets::one(field_name(interp, arg(1), |name| {
                match reg.records.item(arg(0) as i64, name, index) {
                    Some(Item::Int(v) | Item::Ref(v)) => v as u64,
                    _ => 0,
                }
            })))
        }
        MetaOp::RecString | MetaOp::RecItemString => {
            let (index, out) = if op == MetaOp::RecItemString {
                (Some(arg(2) as usize), 3)
            } else {
                (None, 2)
            };
            let mut reg = shared.borrow_mut();
            let found = field_name(interp, arg(1), |name| {
                match reg.records.item(arg(0) as i64, name, index) {
                    Some(Item::Str(s)) => Some(s),
                    _ => None,
                }
            });
            let (count, data) = found.map_or((0, 0), |s| reg.keep_rc(&s));
            store_pair(interp, arg(out), count, data)?;
            Ok(Rets::default())
        }
    }
}

/// Run `f` on the Jai `string` at `p` as a field name. Field names are ASCII identifiers, so
/// anything else (a null string, or one that cannot be read) names no field: `f` gets "".
fn field_name<R>(interp: &Interp, p: u64, f: impl FnOnce(&str) -> R) -> R {
    // Names are short: they are read onto the stack, not into an allocation.
    let mut stack = [0u8; 48];
    let heap;
    let bytes = match interp.read_pair(p) {
        Some((count, data)) if count as usize <= stack.len() => {
            let out = &mut stack[..count as usize];
            interp.read_into(data, out).then_some(&*out)
        }
        Some((count, data)) => {
            heap = interp.read(data, count as usize);
            heap.as_deref()
        }
        None => None,
    };
    f(bytes
        .and_then(|bytes| std::str::from_utf8(bytes).ok())
        .unwrap_or(""))
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

/// The builder `__jaic_rec_builder` numbered `handle`.
fn builder(reg: &mut Workspaces, handle: u64) -> Result<&mut crate::record_build::Builder, Trap> {
    (handle as usize)
        .checked_sub(1)
        .and_then(|at| reg.builders.get_mut(at))
        .ok_or_else(|| Trap::new("records: no such builder"))
}

/// Parse the source text of a `Code` value (an expression, statement or block).
pub fn parse_code_text(
    file: crate::source::FileId,
    text: &str,
) -> Result<Rc<crate::ast::CodeBody>, String> {
    let source = format!("__jaic_code :: #code {text};\n");
    let parsed = crate::parser::parse_file(file, &source).map_err(|d| d.message.clone())?;
    parsed
        .stmts
        .into_iter()
        .find_map(|s| match s.kind {
            crate::ast::StmtKind::Decl(d) => match d.value.as_ref().map(|v| &v.kind) {
                Some(crate::ast::ExprKind::Code(body)) => Some(body.clone()),
                _ => None,
            },
            _ => None,
        })
        .ok_or_else(|| "could not parse the printed code".to_string())
}
