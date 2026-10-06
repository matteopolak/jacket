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
use crate::interp::{Host, Interp, Trap};
use crate::ir;
use crate::records::{Field, Item, Records};
use crate::sema::{Compiler, DeadCode, FileSystem, Options, ProgramSource, TargetCpu, TargetOs};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::PathBuf;
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
            arithmetic_overflow_check: None,
            cast_bounds_check: None,
            stack_trace: None,
            emit_debug_info: None,
            dead_code_elimination: None,
            llvm_triple: String::new(),
            llvm_cpu: String::new(),
            llvm_features: String::new(),
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
}

/// Everything needed to compile a workspace besides its sources.
pub struct BuildEnv {
    pub fs: Rc<dyn FileSystem>,
    /// Base options (import path, preload, target) for new workspaces.
    pub options: Options,
    pub backend: Option<Box<dyn OutputBackend>>,
    /// Without a backend: what to tell the user when a workspace asks for output that this
    /// command does not write (`jaic check` only compiles workspaces). `None`
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
    stage: Stage,
    /// The workspace's compiler between steps (taken out while it runs).
    compiler: Option<Box<Compiler>>,
    events: VecDeque<Event>,
    failed: bool,
    /// `compiler_modify_procedure` calls not applied yet: (body record, statement records).
    modifications: Vec<(i64, Vec<crate::sema::ModifiedStmt>)>,
}

impl Workspace {
    fn new(name: String) -> Self {
        Workspace {
            name,
            pending: Vec::new(),
            settings: BuildSettings::default(),
            intercepted: false,
            stage: Stage::Open,
            compiler: None,
            events: VecDeque::new(),
            failed: false,
            modifications: Vec::new(),
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

const PHASE_ALL_SOURCE_CODE_PARSED: i64 = 0;
const PHASE_TYPECHECKED_ALL_WE_CAN: i64 = 1;
const PHASE_ALL_TARGET_CODE_BUILT: i64 = 2;
const PHASE_PRE_WRITE_EXECUTABLE: i64 = 3;
const PHASE_POST_WRITE_EXECUTABLE: i64 = 4;

/// The workspace registry of one top-level compilation.
pub struct Workspaces {
    env: BuildEnv,
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
    tags: Vec<&'static str>,
    /// Messages, syntax trees and types exported to metaprograms.
    pub(crate) records: Records,
}

pub type SharedWorkspaces = Rc<RefCell<Workspaces>>;

/// The `#compiler` primitives, bound by name in `stdlib/Compiler`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetaOp {
    WorkspaceCreate,
    CurrentWorkspace,
    AddFile,
    AddString,
    SetOption,
    BeginIntercept,
    NextEvent,
    EventInt,
    EventString,
    CommandLineCount,
    CommandLineArg,
    Report,
    CompilerVersion,
    CustomLinkComplete,
    AddStringToModule,
    CodeNodes,
    ParseCode,
    ModifyProcedure,
    SetTypeInfoFlags,
    StructLocation,
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
    RecFill,
    RecFillList,
    Clang,
    ClangText,
}

impl MetaOp {
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "__jaic_workspace_create" => Self::WorkspaceCreate,
            "__jaic_current_workspace" => Self::CurrentWorkspace,
            "__jaic_workspace_add_file" => Self::AddFile,
            "__jaic_workspace_add_string" => Self::AddString,
            "__jaic_workspace_set_option" => Self::SetOption,
            "__jaic_workspace_begin_intercept" => Self::BeginIntercept,
            "__jaic_workspace_next_event" => Self::NextEvent,
            "__jaic_event_int" => Self::EventInt,
            "__jaic_event_string" => Self::EventString,
            "__jaic_command_line_count" => Self::CommandLineCount,
            "__jaic_command_line_arg" => Self::CommandLineArg,
            "__jaic_report" => Self::Report,
            "__jaic_compiler_version" => Self::CompilerVersion,
            "__jaic_custom_link_complete" => Self::CustomLinkComplete,
            "__jaic_workspace_add_string_to_module" => Self::AddStringToModule,
            "__jaic_code_nodes" => Self::CodeNodes,
            "__jaic_parse_code" => Self::ParseCode,
            "__jaic_modify_procedure" => Self::ModifyProcedure,
            "__jaic_set_type_info_flags" => Self::SetTypeInfoFlags,
            "__jaic_struct_location" => Self::StructLocation,
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
            "__jaic_rec_fill" => Self::RecFill,
            "__jaic_rec_fill_list" => Self::RecFillList,
            "__jaic_clang" => Self::Clang,
            "__jaic_clang_text" => Self::ClangText,
            _ => return None,
        })
    }
}

/// Id of the program being compiled by the command line (workspace 1 is reserved, as in `jai`).
pub const TOP_LEVEL_WORKSPACE: i64 = 2;

pub const COMPILER_VERSION: &str = "the, jaic";

impl Workspaces {
    /// A registry whose workspace 1 is the top-level program.
    pub fn new(env: BuildEnv) -> SharedWorkspaces {
        let mut top = Workspace::new("Target Program".into());
        // The embedder compiles the top-level program itself.
        top.stage = Stage::Checked;
        Rc::new(RefCell::new(Workspaces {
            env,
            list: vec![
                Workspace::new(String::new()),
                Workspace::new(String::new()),
                top,
            ],
            current: vec![TOP_LEVEL_WORKSPACE],
            event: Event::default(),
            strings: Vec::new(),
            kept: Vec::new(),
            tags: Vec::new(),
            records: Records::default(),
        }))
    }

    /// Settings of the top-level program (workspace 2).
    pub fn top_level_settings(&self) -> BuildSettings {
        self.list[TOP_LEVEL_WORKSPACE as usize].settings.clone()
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
                    _ => return Ok(()), // Targets jaic cannot build for keep the default.
                })
            }
            "cpu_target" => {
                s.cpu = Some(match value {
                    "X64" => TargetCpu::X64,
                    "ARM64" => TargetCpu::Arm64,
                    // `.CUSTOM` is how a wasm target spells its CPU (the triple says which).
                    "WASM" | "CUSTOM" => TargetCpu::Wasm,
                    _ => return Ok(()),
                })
            }
            "optimization" => s.optimization = value.into(),
            "additional_linker_arguments_clear" => s.additional_linker_arguments.clear(),
            "additional_linker_argument" => s.additional_linker_arguments.push(value.into()),
            "temporary_storage_size" => s.temporary_storage_size = value.parse().ok(),
            "array_bounds_check" => s.array_bounds_check = Some(value != "OFF"),
            "arithmetic_overflow_check" => {
                s.arithmetic_overflow_check = Some(match value {
                    "NONFATAL" => 1,
                    "FATAL" => 2,
                    _ => 0,
                })
            }
            "cast_bounds_check" => {
                s.cast_bounds_check = Some(match value {
                    "NONFATAL" => 1,
                    "FATAL" => 2,
                    _ => 0,
                })
            }
            "stack_trace" => s.stack_trace = Some(value == "true"),
            "emit_debug_info" => s.emit_debug_info = Some(value != "NONE"),
            "dead_code_elimination" => s.dead_code_elimination = DeadCode::from_name(value),
            "llvm_target_system_triple" => s.llvm_triple = value.into(),
            "llvm_target_system_cpu" => s.llvm_cpu = value.into(),
            "llvm_target_system_features" => s.llvm_features = value.into(),
            // Accepted and ignored: checks, added-string dumps...
            _ => {}
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
pub const WASI_RUNTIME_IMPORT: &str = "#import \"Wasi_Runtime\";";

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
fn step(shared: &SharedWorkspaces, id: i64) -> Result<(), String> {
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
        let mut records = std::mem::take(&mut shared.borrow_mut().records);
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
        let mut reg = shared.borrow_mut();
        let added = std::mem::replace(&mut reg.records, records);
        debug_assert!(added.is_empty());
    }
    // Lowering reported more: the metaprogram gets another TYPECHECKED_ALL_WE_CAN to
    // add code. Otherwise the program is complete.
    let result = match result {
        Ok(_) if lowering && reported => {
            events.push(phase(PHASE_TYPECHECKED_ALL_WE_CAN));
            Ok(Stage::Checked)
        }
        Ok(_) if lowering => compiler.finish_program().map(|()| Stage::Done),
        other => other,
    };
    shared.borrow_mut().current.pop();
    let mut failed = false;
    let next = match result {
        Ok(Stage::Done) => {
            failed = !write_output(shared, id, &mut compiler, &mut events)?;
            Stage::Done
        }
        Ok(next) => next,
        Err(d) => {
            let text = compiler.render(&d);
            (shared.borrow_mut().env.report)(&text);
            failed = true;
            Stage::Done
        }
    };
    if next == Stage::Done {
        events.push(Event {
            kind: EVENT_COMPLETE,
            ints: vec![failed as i64],
            strings: Vec::new(),
        });
    }
    file_events.extend(events);
    {
        let mut reg = shared.borrow_mut();
        let ws = reg.ws(id)?;
        ws.events.extend(file_events);
        ws.failed |= failed;
        ws.stage = next;
        if next != Stage::Done {
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

/// Code generation is done: queue the write phases and call the backend.
/// Returns whether the output was written (or not wanted).
fn write_output(
    shared: &SharedWorkspaces,
    id: i64,
    compiler: &mut Compiler,
    events: &mut Vec<Event>,
) -> Result<bool, String> {
    events.push(phase(PHASE_ALL_TARGET_CODE_BUILT));
    let settings = shared.borrow_mut().ws(id)?.settings.clone();
    if !settings.do_output || settings.output_type == OutputType::NoOutput {
        return Ok(true);
    }
    let output = output_path(&settings, compiler);
    let name = output.display().to_string().into_bytes();
    events.push(Event {
        kind: EVENT_PHASE,
        ints: vec![PHASE_PRE_WRITE_EXECUTABLE, 0],
        strings: vec![name.clone()],
    });
    compiler.prepare_compiled_output();
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
                        "warning: `jaic check` does not write {} (workspace `{name}` asks for \
                         {kind})\n\
                         help: `jaic build {main}` (or `jaic run {main}`) writes it",
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
        ints: vec![PHASE_POST_WRITE_EXECUTABLE, 0],
        strings: vec![name],
    });
    Ok(written.is_ok())
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
/// (Jai compiles them after the metaprogram's `#run` returns).
pub fn finish_all(shared: &SharedWorkspaces) -> Result<(), String> {
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
            step(shared, id)?;
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
    interp: &mut Interp,
) -> Result<Vec<u64>, Trap> {
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
            | MetaOp::EventInt
            | MetaOp::EventString
            | MetaOp::CommandLineCount
            | MetaOp::CommandLineArg
            | MetaOp::CompilerVersion
            | MetaOp::StructLocation
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
            | MetaOp::RecFill
            | MetaOp::RecFillList
    ) {
        interp.effects += 1;
    }
    let string = |interp: &Interp, i: usize| -> Vec<u8> {
        let p = arg(i);
        if p == 0 {
            return Vec::new();
        }
        let count = interp.read_u64(p) as usize;
        let data = interp.read_u64(p + 8);
        if count == 0 || data == 0 {
            Vec::new()
        } else {
            interp.read(data, count)
        }
    };
    let text = |interp: &Interp, i: usize| String::from_utf8_lossy(&string(interp, i)).into_owned();
    let trap = |message: String| Trap {
        message,
        ..Trap::default()
    };
    let return_string = |interp: &mut Interp, bytes: &[u8], out_index: usize| {
        let (count, data) = shared.borrow_mut().keep_string(bytes);
        let out = arg(out_index);
        interp.write(out, &count.to_le_bytes());
        interp.write(out + 8, &data.to_le_bytes());
    };
    match op {
        MetaOp::SetTypeInfoFlags => {
            // (type: Type, flags): applied by the compiler after the running `#run` returns.
            let Some((global, 0)) = interp.global_at(arg(0)) else {
                return Err(trap("compiler_set_type_info_flags: not a type".into()));
            };
            interp.pending_type_flags.push((global, arg(1) as u32));
            Ok(Vec::new())
        }
        MetaOp::StructLocation => {
            // (info, out path, out line, out column); unknown structs leave them zero.
            let found = interp
                .global_at(arg(0))
                .and_then(|(g, offset)| (offset == 0).then_some(g))
                .and_then(|g| interp.struct_locations.get(&g).cloned());
            if let Some((path, line, col)) = found {
                return_string(interp, path.as_bytes(), 1);
                interp.write(arg(2), &line.to_le_bytes());
                interp.write(arg(3), &col.to_le_bytes());
            }
            Ok(Vec::new())
        }
        MetaOp::WorkspaceCreate => {
            let name = text(interp, 0);
            let mut reg = shared.borrow_mut();
            let ws = Workspace::new(name);
            reg.list.push(ws);
            Ok(vec![reg.list.len() as u64 - 1])
        }
        MetaOp::CurrentWorkspace => Ok(vec![shared.borrow().current_id() as u64]),
        MetaOp::AddFile | MetaOp::AddString => {
            let id = arg(0) as i64;
            let value = text(interp, 1);
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
            let path = fs.canonical(&PathBuf::from(&value));
            if op == MetaOp::AddFile && !fs.is_file(&path) {
                let cwd = std::env::current_dir().unwrap_or_default();
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
            Ok(Vec::new())
        }
        MetaOp::SetOption => {
            let (key, value) = (text(interp, 1), text(interp, 2));
            shared
                .borrow_mut()
                .set_option(arg(0) as i64, &key, &value)
                .map_err(trap)?;
            Ok(Vec::new())
        }
        MetaOp::BeginIntercept => {
            shared
                .borrow_mut()
                .ws(arg(0) as i64)
                .map_err(trap)?
                .intercepted = true;
            Ok(Vec::new())
        }
        MetaOp::NextEvent => {
            let id = arg(0) as i64;
            if id == shared.borrow().current_id() || id == TOP_LEVEL_WORKSPACE {
                // A workspace cannot wait on its own compilation.
                return Ok(vec![0]);
            }
            loop {
                let mut reg = shared.borrow_mut();
                let ws = reg.ws(id).map_err(trap)?;
                if let Some(event) = ws.events.pop_front() {
                    let kind = event.kind;
                    reg.event = event;
                    return Ok(vec![kind as u64]);
                }
                if ws.stage == Stage::Done {
                    return Ok(vec![0]);
                }
                drop(reg);
                step(shared, id).map_err(trap)?;
            }
        }
        MetaOp::EventInt => {
            let reg = shared.borrow();
            Ok(vec![
                reg.event.ints.get(arg(0) as usize).copied().unwrap_or(0) as u64,
            ])
        }
        MetaOp::EventString => {
            let bytes = shared
                .borrow()
                .event
                .strings
                .get(arg(0) as usize)
                .cloned()
                .unwrap_or_default();
            return_string(interp, &bytes, 1);
            Ok(Vec::new())
        }
        MetaOp::CommandLineCount => Ok(vec![shared.borrow().env.command_line.len() as u64]),
        MetaOp::CommandLineArg => {
            let value = shared
                .borrow()
                .env
                .command_line
                .get(arg(0) as usize)
                .cloned()
                .unwrap_or_default();
            return_string(interp, value.as_bytes(), 1);
            Ok(Vec::new())
        }
        MetaOp::Report => {
            let (message, file) = (text(interp, 0), text(interp, 1));
            let (line, column, is_error) = (arg(2), arg(3), arg(4) & 1 != 0);
            let location = if file.is_empty() {
                String::new()
            } else {
                format!("{file}:{line}:{column}: ")
            };
            let severity = if is_error {
                "error"
            } else {
                "warning"
            };
            if is_error {
                let mut t = trap(message.trim_end().to_string());
                t.reported = Some(Box::new((file, line as u32, column as u32)));
                return Err(t);
            }
            let rendered = format!("{location}{severity}: {}", message.trim_end());
            (shared.borrow_mut().env.report)(&rendered);
            Ok(Vec::new())
        }
        MetaOp::CompilerVersion => {
            return_string(interp, COMPILER_VERSION.as_bytes(), 0);
            Ok(Vec::new())
        }
        MetaOp::CustomLinkComplete => Ok(Vec::new()),
        MetaOp::AddStringToModule => {
            let (id, value, record) = (arg(0) as i64, text(interp, 1), arg(2) as i64);
            let mut reg = shared.borrow_mut();
            // A FILE message stands for its module (the enclosing import).
            let records = &reg.records;
            let message = match records.item(record, "enclosing_import", None) {
                Some(Item::Ref(import)) => *import,
                _ => record,
            };
            let module = match records.item(message, "__module", None) {
                Some(Item::Int(m)) => *m as u32,
                _ => return Err(trap("add_build_string: the message names no module".into())),
            };
            let ws = reg.ws(id).map_err(trap)?;
            ws.pending.push(ProgramSource::ModuleString(value, module));
            Ok(Vec::new())
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
                return Ok(vec![reg.records.add(result) as u64]);
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
            Ok(vec![reg.records.add(result) as u64])
        }
        MetaOp::ParseCode => {
            let (source, scope_from) = (text(interp, 0), arg(1) as usize);
            if scope_from >= interp.codes.len() {
                return Err(trap(
                    "compiler_get_code: no code to take the scope from".into(),
                ));
            }
            // A compile-time run that asks for typed nodes is run again from the start
            // (`export_request`): the same text with the same scope must be the same code
            // there, or each run would make a new one and ask for its nodes again.
            if let Some(&(id, _)) = interp.made_codes.iter().find(|&&(id, scope)| {
                scope == scope_from && interp.codes.get(id).is_some_and(|c| *c.1 == *source)
            }) {
                return Ok(vec![id as u64]);
            }
            let body = parse_code_text(crate::source::FileId(u32::MAX), &source)
                .map_err(|e| trap(format!("compiler_get_code: {e}")))?;
            let id = interp.codes.len();
            interp.codes.push((body, source.into()));
            interp.made_codes.push((id, scope_from));
            Ok(vec![id as u64])
        }
        MetaOp::ModifyProcedure => {
            let (id, body, data, sources, count) =
                (arg(0) as i64, arg(1) as i64, arg(2), arg(3), arg(4));
            // A statement without a record (new, or edited in place) comes as source text.
            let stmts = (0..count)
                .map(|i| match interp.read_u64(data + i * 8) as i64 {
                    0 => {
                        let at = sources + i * 16;
                        let (len, ptr) = (interp.read_u64(at), interp.read_u64(at + 8));
                        let bytes = interp.read(ptr, len as usize);
                        crate::sema::ModifiedStmt::Source(
                            String::from_utf8_lossy(&bytes).into_owned(),
                        )
                    }
                    r => crate::sema::ModifiedStmt::Record(r),
                })
                .collect();
            let mut reg = shared.borrow_mut();
            reg.ws(id).map_err(trap)?.modifications.push((body, stmts));
            Ok(Vec::new())
        }
        MetaOp::Clang => {
            let (name, bytes) = (text(interp, 0), string(interp, 3));
            crate::clang::call(&name, arg(1) as i64, arg(2) as i64, &bytes)
                .map(|v| vec![v as u64])
                .map_err(trap)
        }
        MetaOp::ClangText => {
            return_string(interp, &crate::clang::last_text(), 0);
            Ok(Vec::new())
        }
        MetaOp::RecTag => {
            // Tags are static: hand out the text itself.
            let tag = shared
                .borrow()
                .records
                .get(arg(0) as i64)
                .map_or("", |r| r.tag);
            let out = arg(1);
            interp.write(out, &(tag.len() as u64).to_le_bytes());
            interp.write(out + 8, &(tag.as_ptr() as u64).to_le_bytes());
            Ok(Vec::new())
        }
        MetaOp::RecTagId => {
            // A small number per distinct tag, so Jai can cache tag -> struct type in an array.
            let mut reg = shared.borrow_mut();
            let Some(tag) = reg.records.get(arg(0) as i64).map(|r| r.tag) else {
                return Ok(vec![u64::MAX]);
            };
            let id = match reg.tags.iter().position(|t| *t == tag) {
                Some(i) => i,
                None => {
                    reg.tags.push(tag);
                    reg.tags.len() - 1
                }
            };
            Ok(vec![id as u64])
        }
        MetaOp::RecFill => {
            let mut reg = shared.borrow_mut();
            Ok(vec![rec_fill(&mut reg, interp, args)])
        }
        MetaOp::RecFillList => {
            let mut reg = shared.borrow_mut();
            Ok(vec![rec_fill_list(&mut reg, interp, args)])
        }
        MetaOp::RecField => Ok(vec![
            shared
                .borrow()
                .records
                .kind(arg(0) as i64, field_name(interp, arg(1))) as u64,
        ]),
        MetaOp::RecCount => {
            let reg = shared.borrow();
            Ok(vec![
                match reg.records.field(arg(0) as i64, field_name(interp, arg(1))) {
                    Some(Field::List(items)) => items.len() as u64,
                    _ => 0,
                },
            ])
        }
        MetaOp::RecInt | MetaOp::RecRef | MetaOp::RecItemInt | MetaOp::RecItemRef => {
            let index =
                matches!(op, MetaOp::RecItemInt | MetaOp::RecItemRef).then(|| arg(2) as usize);
            let reg = shared.borrow();
            Ok(vec![match reg
                .records
                .item(arg(0) as i64, field_name(interp, arg(1)), index)
            {
                Some(Item::Int(v) | Item::Ref(v)) => *v as u64,
                _ => 0,
            }])
        }
        MetaOp::RecString | MetaOp::RecItemString => {
            let (index, out) = if op == MetaOp::RecItemString {
                (Some(arg(2) as usize), 3)
            } else {
                (None, 2)
            };
            let mut reg = shared.borrow_mut();
            let (count, data) =
                match reg
                    .records
                    .item(arg(0) as i64, field_name(interp, arg(1)), index)
                {
                    Some(Item::Str(s)) => {
                        let s = s.clone();
                        reg.keep_rc(&s)
                    }
                    _ => (0, 0),
                };
            let out = arg(out);
            interp.write(out, &count.to_le_bytes());
            interp.write(out + 8, &data.to_le_bytes());
            Ok(Vec::new())
        }
    }
}

/// The Jai `string` at `p` as a field name. Field names are ASCII identifiers, so anything else
/// (or a null string) names no field.
fn field_name(interp: &Interp, p: u64) -> &str {
    if p == 0 {
        return "";
    }
    let bytes = interp.bytes(interp.read_u64(p + 8), interp.read_u64(p) as usize);
    std::str::from_utf8(bytes).unwrap_or("")
}

/// Plan entry kinds shared with `Record_Plan_Entry` in `stdlib/Compiler/records.jai`.
const PLAN_OTHER: u64 = 0;

const PLAN_INT: u64 = 1;
const PLAN_STRING: u64 = 2;
const PLAN_POINTER: u64 = 3;

/// Where `__jaic_rec_fill` and `__jaic_rec_fill_list` find the Jai side's `record_structs`.
struct Built {
    data: u64,
    count: i64,
}

/// Write one item as a member of plan kind `kind` and `size` bytes at `target`. Returns false when
/// Jai must fill it: a reference to a record not built yet, or a value of an unexpected shape.
fn write_item(
    interp: &mut Interp,
    keep: &mut Vec<Rc<[u8]>>,
    built: &Built,
    kind: u64,
    size: usize,
    item: &Item,
    target: u64,
) -> bool {
    match (kind, item) {
        (PLAN_INT, Item::Int(v) | Item::Ref(v)) => {
            interp.write(target, &v.to_le_bytes()[..size.min(8)]);
            true
        }
        (PLAN_STRING, Item::Str(s)) => {
            interp.write(target, &(s.len() as u64).to_le_bytes());
            interp.write(target + 8, &(s.as_ptr() as u64).to_le_bytes());
            keep.push(s.clone());
            true
        }
        (PLAN_POINTER, Item::Ref(r)) => {
            if *r <= 0 {
                return true;
            }
            let pointer = if *r < built.count {
                interp.read_u64(built.data + *r as u64 * 8)
            } else {
                0
            };
            interp.write(target, &pointer.to_le_bytes());
            pointer != 0
        }
        (PLAN_POINTER, Item::Int(_)) => false,
        // Mismatched shapes read as zero, and the memory is already zeroed.
        (PLAN_INT | PLAN_STRING | PLAN_POINTER, _) => true,
        _ => false,
    }
}

/// `__jaic_rec_fill(record, memory, plan, plan_count, built, built_count) -> left`: write the
/// plan's integer, string and pointer members of `memory` from the record's fields. Pointers are
/// written only when the referenced record is already built (`built[ref]`, the Jai side's
/// `record_structs`). Returns a bit per plan entry (first 64) that Jai must still fill itself:
/// the field exists but is an array, an in-place struct or an unbuilt reference.
fn rec_fill(reg: &mut Workspaces, interp: &mut Interp, args: &[u64]) -> u64 {
    let arg = |i: usize| args.get(i).copied().unwrap_or(0);
    let (id, memory, plan, count) = (arg(0) as i64, arg(1), arg(2), arg(3) as usize);
    let built = Built {
        data: arg(4),
        count: arg(5) as i64,
    };
    let Some(record) = reg.records.get(id) else {
        return 0;
    };
    let mut left = 0u64;
    let mut keep = Vec::new();
    // Record_Plan_Entry :: struct { offset: s64; kind: s64; size: s64; name: string; type: *Type_Info; }
    const ENTRY: u64 = 48;
    for k in 0..count.min(64) {
        let entry = plan + k as u64 * ENTRY;
        let offset = interp.read_u64(entry);
        let kind = interp.read_u64(entry + 8);
        let size = interp.read_u64(entry + 16) as usize;
        let filled = match record.field(field_name(interp, entry + 24)) {
            None => true,
            Some(Field::Item(item)) => {
                write_item(interp, &mut keep, &built, kind, size, item, memory + offset)
            }
            // A list read as a single value is zero, except as an array member.
            Some(Field::List(_)) => kind != PLAN_OTHER,
        };
        if !filled {
            left |= 1 << k;
        }
    }
    reg.kept.extend(keep);
    left
}

/// `__jaic_rec_fill_list(record, name, data, count, kind, size, built, built_count) -> left`:
/// write the elements of list field `name` into `data` (`count` elements of `size` bytes).
/// Returns how many elements Jai must still fill (see `write_item`).
fn rec_fill_list(reg: &mut Workspaces, interp: &mut Interp, args: &[u64]) -> u64 {
    let arg = |i: usize| args.get(i).copied().unwrap_or(0);
    let (id, data, count, kind, size) = (arg(0) as i64, arg(2), arg(3), arg(4), arg(5));
    let built = Built {
        data: arg(6),
        count: arg(7) as i64,
    };
    let Some(Field::List(items)) = reg.records.field(id, field_name(interp, arg(1))) else {
        return count;
    };
    let mut left = 0;
    let mut keep = Vec::new();
    for (k, item) in items.iter().take(count as usize).enumerate() {
        let target = data + k as u64 * size;
        if !write_item(interp, &mut keep, &built, kind, size as usize, item, target) {
            left += 1;
        }
    }
    reg.kept.extend(keep);
    left
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
