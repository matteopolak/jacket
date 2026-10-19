//! The registry of workspaces: option setters, compiler creation and event construction.
use super::meta_op::TOP_LEVEL_WORKSPACE;
use super::settings::{BuildEnv, BuildSettings};
use super::state::{EVENT_PHASE, Event, SharedWorkspaces, Stage, Workspace, Workspaces};
use crate::builtin_keys::OutputType;
use crate::builtin_keys::{
    ArrayBoundsCheck, BitcodeOptimization, CheckLevel, CpuTag, DeadCodeSetting, DebugInfoSetting,
    MachineCodeOptimization, OsTag, RuntimeSupport, Toggle, WorkspaceFlag, WorkspaceList,
    WorkspaceNumber, WorkspaceText,
};
use crate::interp::Host;
use crate::records::Records;
use crate::sema::{
    Compiler, DeadCode, FileSystem, Options, ProgramSource, ProvidedImport, TargetCpu, TargetOs,
};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

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
            records: Records::default(),
            builders: Vec::new(),
            data_segments: Vec::new(),
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

    pub(super) fn ws(&mut self, id: i64) -> Result<&mut Workspace, String> {
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

    pub(super) fn current_id(&self) -> i64 {
        *self.current.last().unwrap_or(&TOP_LEVEL_WORKSPACE)
    }

    fn workspace_settings(&mut self, id: i64) -> Result<&mut BuildSettings, String> {
        Ok(&mut self.ws(id)?.settings)
    }

    /// `compiler_destroy_workspace`: what was added is never compiled.
    pub(super) fn destroy_workspace(&mut self, id: i64) -> Result<(), String> {
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

    pub(super) fn set_text(
        &mut self,
        id: i64,
        option: WorkspaceText,
        value: &str,
    ) -> Result<(), String> {
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

    pub(super) fn set_flag(
        &mut self,
        id: i64,
        option: WorkspaceFlag,
        on: bool,
    ) -> Result<(), String> {
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
                s.output_bitcode_before_optimizations = on;
            }
            WorkspaceFlag::OutputLlvmIrBeforeOptimizations => {
                s.output_llvm_ir_before_optimizations = on;
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

    pub(super) fn set_number(
        &mut self,
        id: i64,
        option: WorkspaceNumber,
        raw: u64,
    ) -> Result<(), String> {
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
                s.output_type = member(option, raw, OutputType::from_raw)?;
            }
            WorkspaceNumber::OsTarget => {
                s.os = Some(match member(option, raw, OsTag::from_raw)? {
                    OsTag::Windows => TargetOs::Windows,
                    OsTag::Linux => TargetOs::Linux,
                    OsTag::MacOS => TargetOs::MacOS,
                    OsTag::Wasm => TargetOs::Wasm,
                });
            }
            WorkspaceNumber::CpuTarget => {
                s.cpu = Some(match member(option, raw, CpuTag::from_raw)? {
                    CpuTag::X64 => TargetCpu::X64,
                    CpuTag::Arm64 => TargetCpu::Arm64,
                    CpuTag::Custom => TargetCpu::Wasm,
                });
            }
            WorkspaceNumber::TemporaryStorageSize => s.temporary_storage_size = Some(raw as i64),
            WorkspaceNumber::ArrayBoundsCheck => {
                s.array_bounds_check =
                    Some(member(option, raw, ArrayBoundsCheck::from_raw)? != ArrayBoundsCheck::Off);
            }
            WorkspaceNumber::NullPointerCheck => {
                s.null_pointer_check = Some(member(option, raw, Toggle::from_raw)? != Toggle::Off);
            }
            WorkspaceNumber::ArithmeticOverflowCheck => {
                s.arithmetic_overflow_check =
                    Some(member(option, raw, CheckLevel::from_raw)? as u8);
            }
            WorkspaceNumber::CastBoundsCheck => {
                s.cast_bounds_check = Some(member(option, raw, CheckLevel::from_raw)? as u8);
            }
            WorkspaceNumber::EmitDebugInfo => {
                s.emit_debug_info = Some(
                    member(option, raw, DebugInfoSetting::from_raw)? != DebugInfoSetting::None,
                );
            }
            WorkspaceNumber::DeadCodeElimination => {
                s.dead_code_elimination =
                    Some(match member(option, raw, DeadCodeSetting::from_raw)? {
                        DeadCodeSetting::None => DeadCode::None,
                        DeadCodeSetting::All => DeadCode::All,
                        DeadCodeSetting::ModulesOnly => DeadCode::ModulesOnly,
                    });
            }
            WorkspaceNumber::RuntimeSupportDefinitions => {
                s.runtime_support_definitions =
                    Some(member(option, raw, RuntimeSupport::from_raw)?);
            }
            WorkspaceNumber::BacktraceOnCrash => {
                s.backtrace_on_crash = Some(member(option, raw, Toggle::from_raw)? == Toggle::On);
            }
            WorkspaceNumber::BitcodeOptimization => {
                s.optimization = member(option, raw, BitcodeOptimization::from_raw)?;
            }
            WorkspaceNumber::MachineCodeOptimization => {
                s.machine_code_optimization =
                    member(option, raw, MachineCodeOptimization::from_raw)?;
            }
        }
        Ok(())
    }

    pub(super) fn clear_list(&mut self, id: i64, option: WorkspaceList) -> Result<(), String> {
        let s = self.workspace_settings(id)?;
        match option {
            WorkspaceList::ImportPath => s.import_paths = Some(Vec::new()),
            WorkspaceList::AdditionalLinkerArgument => s.additional_linker_arguments.clear(),
        }
        Ok(())
    }

    pub(super) fn append_list(
        &mut self,
        id: i64,
        option: WorkspaceList,
        value: &str,
    ) -> Result<(), String> {
        let s = self.workspace_settings(id)?;
        match option {
            WorkspaceList::ImportPath => s
                .import_paths
                .get_or_insert_with(Vec::new)
                .push(PathBuf::from(value)),
            WorkspaceList::AdditionalLinkerArgument => {
                s.additional_linker_arguments.push(value.into());
            }
        }
        Ok(())
    }

    pub(super) fn keep_string(&mut self, bytes: &[u8]) -> (u64, u64) {
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
            (report_parent.borrow_mut().env.report)(severity, message);
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
/// workspace itself. `NO_OUTPUT` workspaces have no target runtime to initialize.
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
pub(super) fn new_compiler(shared: &SharedWorkspaces, id: i64) -> Result<Box<Compiler>, String> {
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
        options.import_paths.clone_from(paths);
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
        options.entry_point.clone_from(&settings.entry_point_name);
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

pub(super) fn record_event(kind: i64, record: i64) -> Event {
    Event {
        kind,
        ints: vec![record],
        strings: Vec::new(),
    }
}

pub(super) fn phase(p: i64) -> Event {
    Event {
        kind: EVENT_PHASE,
        ints: vec![p, 0],
        strings: Vec::new(),
    }
}
