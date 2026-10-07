//! IR interpreter. Runs `#run` code at compile time and whole programs in
//! scripting mode and in the browser.
//!
//! Memory is real: pointers are host addresses, globals and stack slots live
//! in host allocations, and foreign procedures are called natively (or
//! through `Host` shims where no dynamic linker exists, e.g. wasm). Procedure
//! values are tagged non-canonical addresses that only the interpreter can
//! call, except `#c_call` procedures under native linking, whose values are
//! C-callable thunks (`native/callbacks.rs`).
#![allow(unsafe_code)]

mod code;
#[cfg(not(target_arch = "wasm32"))]
mod crash;
#[cfg(not(target_arch = "wasm32"))]
pub use crash::CRASH_STATUS;
mod executable_path;
mod native;
mod probe;
pub mod profile;
mod sandbox;
#[cfg(not(target_arch = "wasm32"))]
mod threads;
mod threads_inline;
use crate::fxhash::HashMap;
use crate::ir::{
    self, BinOp, Callee, CmpOp, ConvOp, ForeignId, FuncId, GlobalId, Inst, Program, Ty, UnOp,
};
#[cfg(target_os = "macos")]
pub use native::main_thread;
pub use native::{homebrew_lib_dir, library_dirs, set_library_dirs};
pub use sandbox::{SandboxHost, SharedHost};
use std::collections::BTreeMap;
use std::rc::Rc;
#[cfg(not(target_arch = "wasm32"))]
pub use threads::DEADLOCK_GRACE;

/// Tag bits marking an interpreted procedure address.
pub const FUNC_TAG: u64 = 0xFEED_0000_0000_0000;

/// Tag bits for foreign procedures that have no native address.
pub const FOREIGN_TAG: u64 = 0xFEEE_0000_0000_0000;

const TAG_MASK: u64 = 0xFFFF_0000_0000_0000;

const STACK_SIZE: usize = 32 << 20;
const MAX_DEPTH: usize = 20_000;

/// Interpreter-implemented `#compiler` procedures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hook {
    /// `write_string(s: string, to_standard_error: bool)`
    WriteString,
    /// `write_strings(strings: ..string, to_standard_error: bool)`
    WriteStrings,
    DebugBreak,
    /// `runtime_support_report_assertion(loc: Source_Code_Location, message: string)`: a failed
    /// `assert`, reported by the compiler as a runtime error (`Trap::assertion`) rather than
    /// printed by the runtime. `context` says whether it takes a context pointer.
    AssertionFailed {
        context: bool,
        location: LocationLayout,
    },
    /// A `Compiler` module primitive (`__jaic_*`); the flag says whether the
    /// procedure takes a context pointer.
    Meta(crate::build::MetaOp, bool),
}

/// Byte offsets of a `Source_Code_Location`'s fields, from the Preload's declaration
/// (`Compiler::location_layout`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocationLayout {
    pub path: u64,
    pub line: u64,
    pub column: u64,
}

/// Where program output and platform services go.
pub trait Host {
    fn write(&mut self, bytes: &[u8], to_stderr: bool);

    /// Implement a foreign procedure without native linking. `None` = not provided.
    fn foreign(
        &mut self,
        symbol: &str,
        args: &[u64],
        sig: &ir::Sig,
    ) -> Option<Result<Vec<u64>, String>>;

    /// Whether foreign symbols may be resolved through the native dynamic linker.
    fn native_linking(&self) -> bool;

    /// Run threads on the interpreter's own stack, one after another (no OS threads); see
    /// `threads_inline.rs`. The sandbox says yes.
    fn cooperative_threads(&self) -> bool {
        false
    }

    /// Move the virtual clock forward (sleeping). Hosts with a real clock ignore it.
    fn advance_clock(&mut self, _nanoseconds: u64) {
    }

    /// The virtual clock's reading, nanoseconds since the epoch, if the host has one.
    fn virtual_now_ns(&mut self) -> Option<u64> {
        None
    }

    /// Asked when `Interp::block_budget` runs out: a new budget to go on with, or `None` to stop
    /// with "execution budget exhausted". The browser host refills it after the program waited
    /// for the page (an animation frame), so the budget bounds the work between two waits.
    fn refill_budget(&mut self) -> Option<u64> {
        None
    }
}

/// Writes to the process's stdout/stderr and links natively.
pub struct NativeHost;

impl Host for NativeHost {
    fn write(&mut self, bytes: &[u8], to_stderr: bool) {
        use std::io::Write;
        if to_stderr {
            let _ = std::io::stderr().write_all(bytes);
        } else {
            let mut out = std::io::stdout();
            let _ = out.write_all(bytes);
            let _ = out.flush();
        }
    }

    fn foreign(
        &mut self,
        _symbol: &str,
        _args: &[u64],
        _sig: &ir::Sig,
    ) -> Option<Result<Vec<u64>, String>> {
        None
    }

    fn native_linking(&self) -> bool {
        // Windows: `interp/native/windows.rs` (x64 and arm64).
        cfg!(any(
            unix,
            all(
                windows,
                any(target_arch = "x86_64", target_arch = "aarch64")
            )
        ))
    }
}

/// A runtime failure inside interpreted code.
#[derive(Debug, Clone, Default)]
pub struct Trap {
    pub message: String,
    /// (file, line, column) of the last executed statement.
    pub loc: Option<(u32, u32, u32)>,
    /// The interpreted procedures the failure unwound through, innermost first: each one's
    /// name and the statement it was executing (for the callee, the call). Filled by `exec` as
    /// the error propagates; capped at `MAX_TRAP_FRAMES`, the rest counted in `omitted_frames`.
    pub frames: Vec<TrapFrame>,
    pub omitted_frames: usize,
    /// A failed `assert`: the location it was given (path, line, column).
    pub assertion: Option<Box<(String, u32, u32)>>,
    /// An error a metaprogram reported itself (`compiler_report`, a failed workspace): the
    /// message is the program's own, shown without the compile-time-execution prefix, at the
    /// location it named (path, line, column; an empty path names none).
    pub reported: Option<Box<(String, u32, u32)>>,
    /// Which kind of failure this is, for the report's notes and help (the message is for
    /// people; nothing parses it).
    pub kind: Option<TrapKind>,
}

/// What failed, for failures the report treats specially.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapKind {
    /// A runtime check: an `ir::TRAP_*` reason and its details (`ir::check_message`).
    Check { reason: u64, a: u64, b: u64 },
    /// A load, store, copy or call through a null (or nearly null) pointer.
    NullPointer,
    /// Recursion deeper than the interpreter allows.
    StackOverflow,
    /// A failed `assert` without a message of its own.
    BareAssertion,
    /// A foreign procedure neither the host nor a loaded library provides.
    Unavailable,
    /// Not a failure: unwinds a thread of the sandbox's scheduler that blocks or yields, its
    /// frames saved so it can resume later (`threads_inline.rs`). Never reported.
    Suspended,
}

/// What a load or store at an address in the never-mapped first page did (`Interp::null_trap`).
fn null_access(kind: &str, addr: u64) -> String {
    if addr == 0 {
        format!("{kind} through a null pointer")
    } else {
        format!("{kind} at address {addr:#x}, just past null (a member of a null struct pointer?)")
    }
}

/// One procedure on the interpreter's call stack when a `Trap` was raised.
#[derive(Debug, Clone)]
pub struct TrapFrame {
    pub name: String,
    pub origin: ir::FuncOrigin,
    /// (file, line, column) of the statement this procedure was executing.
    pub loc: Option<(u32, u32, u32)>,
}

/// How many frames a `Trap` keeps (deep recursion keeps the innermost ones).
const MAX_TRAP_FRAMES: usize = 64;

impl Trap {
    /// The trap as text, for a report made where the compiler's sources are out of reach (a
    /// procedure called from C): the message, where it happened, and the procedures it unwound
    /// through. `file_paths` names files by id (`Program::file_paths`); without them a
    /// location is its line and column.
    pub fn describe(&self, file_paths: &[String]) -> String {
        let at = |(file, line, col): (u32, u32, u32)| match file_paths.get(file as usize) {
            Some(path) => format!(
                "{}:{line}:{col}",
                crate::display_path(std::path::Path::new(path))
            ),
            None => format!("line {line}, column {col}"),
        };
        let mut out = self.message.clone();
        if let Some(loc) = self.loc {
            out += &format!(" (at {})", at(loc));
        }
        for frame in &self.frames {
            out += &format!("\n    in `{}`", frame.name);
            if let Some(loc) = frame.loc {
                out += &format!(" at {}", at(loc));
            }
        }
        if self.omitted_frames > 0 {
            out += &format!("\n    ... {} more", self.omitted_frames);
        }
        out
    }

    fn push_frame(&mut self, func: &ir::Func, loc: Option<(u32, u32, u32)>) {
        if self.frames.len() < MAX_TRAP_FRAMES {
            self.frames.push(TrapFrame {
                name: func.name.clone(),
                origin: func.origin,
                loc,
            });
        } else {
            self.omitted_frames += 1;
        }
    }
}

type Res<T> = std::result::Result<T, Trap>;

/// A running procedure and the location it was called from (`Interp::calls`).
type Call = (FuncId, Option<(u32, u32, u32)>);

/// C functions whose calls leave nothing observable outside the interpreter's memory, so a
/// compile-time run that made only these can be repeated (see `Interp::effects`).
const UNOBSERVABLE_FOREIGNS: &[&str] = &[
    "malloc",
    "calloc",
    "realloc",
    "free",
    "posix_memalign",
    "aligned_alloc",
    "memcpy",
    "memmove",
    "memset",
    "memcmp",
    "strlen",
    "strcmp",
    "strncmp",
];

/// Foreign procedures that never release memory, so the pages `probe` found accessible stay so.
const KEEPS_MEMORY: &[&str] = &[
    "malloc",
    "calloc",
    "posix_memalign",
    "aligned_alloc",
    "memcpy",
    "memmove",
    "memset",
    "memcmp",
    "strlen",
    "strcmp",
    "strncmp",
];

struct Frame {
    offsets: Vec<u64>,
    size: u64,
    /// Largest slot alignment: the frame's absolute start address is a multiple of it, so
    /// `#align 64` locals land aligned (offsets alone only align relative to the frame).
    align: u64,
    /// The body in the interpreter's form (`code.rs`).
    code: code::Code,
}

struct GlobalMem {
    _storage: ZeroedBlock,
    addr: u64,
}

/// Zeroed host memory for program data (globals, sandbox `malloc`), released when dropped.
/// Allocation is fallible: a program asking for more than the host has gets an error or a null
/// pointer instead of aborting the compiler.
pub(crate) struct ZeroedBlock {
    ptr: std::ptr::NonNull<u8>,
    layout: std::alloc::Layout,
}

impl ZeroedBlock {
    pub(crate) fn new(size: usize, align: usize) -> Option<ZeroedBlock> {
        let layout = std::alloc::Layout::from_size_align(size.max(1), align).ok()?;
        // Zeroed pages are mapped lazily, so a large block nobody touches costs no memory.
        let ptr = std::ptr::NonNull::new(unsafe { std::alloc::alloc_zeroed(layout) })?;
        Some(ZeroedBlock {
            ptr,
            layout,
        })
    }

    pub(crate) fn addr(&self) -> u64 {
        self.ptr.as_ptr() as u64
    }
}

impl Drop for ZeroedBlock {
    fn drop(&mut self) {
        unsafe { std::alloc::dealloc(self.ptr.as_ptr(), self.layout) };
        // Its pages may be unmapped now.
        probe::invalidate();
    }
}

pub struct Interp {
    stack: Box<[u64]>,
    sp: u64,
    globals: Vec<Option<GlobalMem>>,
    /// start address -> (end, global) for address lookups.
    ranges: BTreeMap<u64, (u64, GlobalId)>,
    frames: Vec<Option<Rc<Frame>>>,
    /// Resolved foreign symbols by `ForeignId` (0 = not resolved yet).
    foreign_addrs: Vec<u64>,
    libraries: HashMap<usize, Option<native::Library>>,
    /// `#compiler` procedures handled by the compiler, indexed by `FuncId` (checked on every call).
    pub hooks: Vec<Option<Hook>>,
    pub host: Box<dyn Host>,
    depth: usize,
    /// The procedures running, outermost first, each with the location it was called from:
    /// what a crash in native code reports (`crash.rs`).
    calls: Vec<Call>,
    loc: Option<(u32, u32, u32)>,
    /// Inside procedures without a stack trace node (`Some`): the location of the call that
    /// entered them from a traced procedure. A traced call made there reports that line for
    /// the caller's node, not a line of the untraced callee's source.
    trace_loc: Option<Option<(u32, u32, u32)>>,
    /// True while evaluating compile-time code (`#compile_time`).
    pub compile_time: bool,
    /// The executable `jaic build` would write for the program `jaic run` is running: what the
    /// program's own executable-path queries return (not `jaic`'s path), so paths relative to
    /// the executable (`../assets`) work the same in both. `None`: no substitution.
    pub run_executable: Option<String>,
    /// The arguments `jaic run` hands the program (`argv`): what its Windows command-line
    /// query (`GetCommandLineW`) returns while `run_executable` is set, not `jaic`'s own.
    pub run_arguments: Vec<String>,
    /// `run_arguments` as a Windows command line, made on the first query and kept for the
    /// rest of the process, as the OS keeps its own.
    run_command_line: Option<u64>,
    /// `compiler_set_type_info_flags` calls (type descriptor global, flags) not applied yet.
    pub pending_type_flags: Vec<(GlobalId, u32)>,
    /// Declaration of each struct whose descriptor exists: file path, line, column
    /// (`compiler_get_struct_location`).
    pub struct_locations: std::collections::HashMap<GlobalId, (std::rc::Rc<str>, i64, i64)>,
    /// Workspace registry for the `Compiler` module, when the embedder has one.
    pub workspaces: Option<crate::build::SharedWorkspaces>,
    /// Bodies and source text of the compiler's `Code` values, by `CodeId`.
    pub codes: Vec<(std::rc::Rc<crate::ast::CodeBody>, std::rc::Rc<str>)>,
    /// Codes made by compile-time code (`compiler_get_code`): their index and the
    /// index of the code whose scope they take. The compiler adopts them lazily.
    pub made_codes: Vec<(usize, usize)>,
    /// The codes the current compile-time run made, in order, and how many of them its
    /// current attempt has made again: a repeated run (`export_request`) replays the earlier
    /// attempt, so its `compiler_get_code` calls give the codes the earlier one got.
    pub run_codes: (Vec<usize>, usize),
    /// Codes handed to `compiler_get_nodes`, newest last: the scopes a made code without
    /// its own scope falls back on for names its insertion site does not have.
    pub nodes_codes: Vec<usize>,
    /// Observable effects so far: output, foreign calls, workspace changes. A compile-time
    /// run is repeated (to serve `export_request`) only while this has not changed.
    pub effects: u64,
    /// `effects` when the current compile-time run started; `None` outside one.
    pub run_effects: Option<u64>,
    /// Set with a trap when `compiler_get_nodes` needs the compiler to export a code with
    /// resolved names and types (the compiler does, then runs the code again).
    pub export_request: Option<usize>,
    /// The function a trap found without a body (the compiler may lower it and run again).
    pub missing_func: Option<FuncId>,
    /// Those typed exports (root record, node records), by code index, in the order the
    /// run asks for them: each `compiler_get_nodes` call gets nodes of its own, which it may
    /// edit. `code_export_cursor` counts the ones this run took; it starts over on each run.
    pub code_exports: HashMap<usize, Vec<(i64, Vec<i64>)>>,
    pub code_export_cursor: HashMap<usize, usize>,
    /// Set in the child process after compile-time code calls `fork`.
    forked_child: bool,
    /// Stack trace node data per procedure (`Stack_Trace_Procedure_Info`), built on first call.
    /// `Stack_Trace_Procedure_Info` addresses by `FuncId` (0 = not made yet).
    trace_infos: Vec<u64>,
    /// Threads of the running program and the gate C callbacks enter through: made by the
    /// first thread, lock or thunk (`threads::Shared`).
    #[cfg(not(target_arch = "wasm32"))]
    shared: Option<std::sync::Arc<threads::Shared>>,
    /// Basic blocks run since the last preemption check (`threads::preempt`).
    #[cfg(not(target_arch = "wasm32"))]
    ticks: u64,
    /// Threads of the running program when the host schedules them cooperatively.
    isched: Option<Box<threads_inline::InlineSched>>,
    /// More than one thread exists: `run` offers the baton to the others now and then.
    multi: bool,
    /// `Interp::call`s (and C callbacks) running: the sandbox scheduler switches threads only
    /// in the outermost one, where every Rust frame of a thread is resumable.
    call_nesting: u32,
    /// While a thread suspends (`TrapKind::Suspended`): the block and op where the innermost
    /// `run_code` stopped, and the value registers `run` hands over, for `exec` to save.
    suspend_at: Option<(usize, usize)>,
    suspend_vals: Option<Vec<u64>>,
    /// The frames of the suspending thread saved so far, innermost first.
    captured: Vec<threads_inline::SuspFrame>,
    /// Basic blocks left to run before execution traps (editors bound compile-time code).
    pub block_budget: Option<u64>,
    /// Reused value-register vectors (see `run`).
    val_pool: Vec<Vec<u64>>,
    /// Blocks and instructions run in the current frame (`JAIC_PROFILE`, see `profile`).
    frame_blocks: u64,
    frame_insts: u64,
    profile: Option<Box<profile::Counts>>,
    /// The value of each procedure in interpreted code, by `FuncId` (0 = not decided yet):
    /// see `proc_value`.
    proc_values: Vec<u64>,
    /// Thunk address -> procedure, for every thunk this interpreter made.
    thunk_funcs: HashMap<u64, FuncId>,
    /// Thunks made since the last foreign call, which C has not been able to see yet. One that
    /// starts a Jai thread (`pthread_create`, `CreateThread`) leaves the list, since the
    /// scheduler runs it; the next foreign call hands the rest to C (`hand_thunks_to_c`).
    #[cfg(not(target_arch = "wasm32"))]
    unseen_thunks: Vec<u64>,
    /// Value stacks for procedures C calls back, reused.
    #[cfg(not(target_arch = "wasm32"))]
    callback_stacks: Vec<Box<[u64]>>,
    /// Procedures already recorded for `JAIC_COVERAGE`, by `FuncId`; `None` when it is unset.
    covered: Option<Vec<bool>>,
}

/// What `exec` saved on entering a frame, to restore when it returns.
struct FrameExit {
    id: FuncId,
    /// `sp` before the frame.
    base: u64,
    saved_loc: Option<(u32, u32, u32)>,
    saved_trace_loc: Option<Option<(u32, u32, u32)>>,
    /// The `context.stack_trace` slot and its previous top (`trace_enter`).
    pushed: Option<(u64, u64)>,
    /// The caller's profile counts.
    outer: (u64, u64),
}

/// Whether `result` is a thread of the sandbox's scheduler switching out (`threads_inline.rs`).
fn suspended<T>(result: &Res<T>) -> bool {
    matches!(result, Err(trap) if trap.kind == Some(TrapKind::Suspended))
}

/// What a thread running interpreted code keeps of `Interp` while another thread has it, or
/// while it is inside a native call.
#[derive(Default)]
struct ExecState {
    stack: Box<[u64]>,
    sp: u64,
    depth: usize,
    calls: Vec<Call>,
    loc: Option<(u32, u32, u32)>,
    trace_loc: Option<Option<(u32, u32, u32)>>,
}


impl Default for Interp {
    fn default() -> Self {
        Self::new(Box::new(NativeHost))
    }
}

impl Interp {
    pub fn new(host: Box<dyn Host>) -> Self {
        Self {
            stack: Vec::new().into_boxed_slice(),
            sp: 0,
            globals: Vec::new(),
            ranges: BTreeMap::new(),
            frames: Vec::new(),
            foreign_addrs: Vec::new(),
            libraries: HashMap::default(),
            hooks: Vec::new(),
            host,
            depth: 0,
            calls: Vec::new(),
            loc: None,
            trace_loc: None,
            compile_time: true,
            run_executable: None,
            run_arguments: Vec::new(),
            run_command_line: None,
            pending_type_flags: Vec::new(),
            struct_locations: std::collections::HashMap::new(),
            workspaces: None,
            codes: Vec::new(),
            made_codes: Vec::new(),
            run_codes: (Vec::new(), 0),
            nodes_codes: Vec::new(),
            effects: 0,
            run_effects: None,
            export_request: None,
            missing_func: None,
            code_exports: HashMap::default(),
            code_export_cursor: HashMap::default(),
            forked_child: false,
            trace_infos: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            shared: None,
            #[cfg(not(target_arch = "wasm32"))]
            ticks: 0,
            isched: None,
            multi: false,
            call_nesting: 0,
            suspend_at: None,
            suspend_vals: None,
            captured: Vec::new(),
            block_budget: None,
            val_pool: Vec::new(),
            frame_blocks: 0,
            frame_insts: 0,
            profile: profile::enabled().then(Default::default),
            proc_values: Vec::new(),
            thunk_funcs: HashMap::default(),
            #[cfg(not(target_arch = "wasm32"))]
            unseen_thunks: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            callback_stacks: Vec::new(),
            covered: profile::coverage_file().map(|_| Vec::new()),
        }
    }

    #[cold]
    fn trap<T>(&self, message: impl Into<String>) -> Res<T> {
        Err(Trap {
            message: message.into(),
            loc: self.loc,
            ..Trap::default()
        })
    }

    fn trap_of<T>(&self, kind: TrapKind, message: impl Into<String>) -> Res<T> {
        Err(Trap {
            message: message.into(),
            loc: self.loc,
            kind: Some(kind),
            ..Trap::default()
        })
    }

    /// A failed runtime check (`ir::TRAP_*`), worded by `ir::check_message`.
    fn check_trap<T>(&self, reason: u64, a: u64, b: u64) -> Res<T> {
        self.trap_of(
            TrapKind::Check {
                reason,
                a,
                b,
            },
            ir::check_message(reason, a, b),
        )
    }

    /// An access through a null pointer: `detail` says what was done.
    fn null_trap<T>(&self, detail: impl std::fmt::Display) -> Res<T> {
        self.trap_of(
            TrapKind::NullPointer,
            format!("null pointer dereference: {detail}"),
        )
    }

    // -----------------------------------------------------------------------
    // Memory
    // -----------------------------------------------------------------------

    /// Host address of a global, materializing it (and what it points to) on first use.
    pub fn global_addr(&mut self, program: &Program, g: GlobalId) -> Res<u64> {
        let i = g.0 as usize;
        if let Some(Some(m)) = self.globals.get(i) {
            return Ok(m.addr);
        }
        if self.globals.len() <= i {
            self.globals.resize_with(i + 1, || None);
        }
        let global = &program.globals[i];
        let align = global.align.max(8).next_power_of_two();
        let block = usize::try_from(global.size)
            .ok()
            .zip(usize::try_from(align).ok())
            .and_then(|(size, align)| ZeroedBlock::new(size, align));
        let Some(storage) = block else {
            return self.trap(format!(
                "cannot allocate {} bytes for a global variable",
                global.size
            ));
        };
        let addr = storage.addr();
        unsafe {
            std::ptr::copy_nonoverlapping(
                global.init.as_ptr(),
                addr as *mut u8,
                global.init.len().min(global.size as usize),
            )
        };
        self.globals[i] = Some(GlobalMem {
            _storage: storage,
            addr,
        });
        self.ranges.insert(addr, (addr + global.size.max(1), g));
        for r in &global.relocs {
            let target = match r.target {
                ir::RelocTarget::Global(t) => self.global_addr(program, t)?,
                ir::RelocTarget::Func(f) => self.proc_value(program, f),
                ir::RelocTarget::Foreign(f) => self.foreign_addr(program, f)?,
            };
            let value = target.wrapping_add(r.addend as u64);
            unsafe { std::ptr::write_unaligned((addr + r.offset) as *mut u64, value) };
        }
        Ok(addr)
    }

    /// Reload an already materialized global from its (rebuilt) initial contents, in place.
    pub fn refresh_global(&mut self, program: &Program, g: GlobalId) -> Res<()> {
        let i = g.0 as usize;
        let Some(Some(m)) = self.globals.get(i) else {
            return Ok(());
        };
        let addr = m.addr;
        let global = &program.globals[i];
        unsafe {
            std::ptr::write_bytes(addr as *mut u8, 0, global.size as usize);
            std::ptr::copy_nonoverlapping(
                global.init.as_ptr(),
                addr as *mut u8,
                global.init.len().min(global.size as usize),
            )
        };
        for r in &global.relocs {
            let target = match r.target {
                ir::RelocTarget::Global(t) => self.global_addr(program, t)?,
                ir::RelocTarget::Func(f) => self.proc_value(program, f),
                ir::RelocTarget::Foreign(f) => self.foreign_addr(program, f)?,
            };
            let value = target.wrapping_add(r.addend as u64);
            unsafe { std::ptr::write_unaligned((addr + r.offset) as *mut u64, value) };
        }
        Ok(())
    }

    /// Put every resettable global back to its initial contents (compile-time execution
    /// leaves no state behind in the program that then runs; `#no_reset` opts out).
    pub fn reset_globals(&mut self, program: &Program) -> Res<()> {
        for &g in &program.reset_globals {
            let i = g.0 as usize;
            let Some(Some(m)) = self.globals.get(i) else {
                continue;
            };
            let addr = m.addr;
            let global = &program.globals[i];
            let init = global.init.len().min(global.size as usize);
            unsafe {
                std::ptr::write_bytes(addr as *mut u8, 0, global.size as usize);
                std::ptr::copy_nonoverlapping(global.init.as_ptr(), addr as *mut u8, init);
            }
            for r in &global.relocs {
                let target = match r.target {
                    ir::RelocTarget::Global(t) => self.global_addr(program, t)?,
                    ir::RelocTarget::Func(f) => self.proc_value(program, f),
                    ir::RelocTarget::Foreign(f) => self.foreign_addr(program, f)?,
                };
                let value = target.wrapping_add(r.addend as u64);
                unsafe { std::ptr::write_unaligned((addr + r.offset) as *mut u64, value) };
            }
        }
        Ok(())
    }

    /// Where global `g` lives in interpreter memory, if compile-time code has used it.
    pub fn materialized_global(&self, g: GlobalId) -> Option<u64> {
        self.globals.get(g.0 as usize)?.as_ref().map(|m| m.addr)
    }

    /// The global containing `addr`, with the offset into it.
    pub fn global_at(&self, addr: u64) -> Option<(GlobalId, u64)> {
        let (&start, &(end, g)) = self.ranges.range(..=addr).next_back()?;
        (addr < end).then_some((g, addr - start))
    }

    // The compiler's own accesses to program memory (reading `#run` results, the arguments of
    // `compiler_*` calls, writing their out-parameters) go through `probe`, so an address the
    // program made up is an error instead of a crash. Interpreted loads and stores do not: they
    // behave like the native program's.

    /// `len` bytes of program memory at `addr`, or `None` when they cannot all be read.
    pub fn read(&self, addr: u64, len: usize) -> Option<Vec<u8>> {
        let mut out = vec![0; len];
        probe::read(addr, &mut out).then_some(out)
    }

    /// The `u64` at `addr` in program memory, or `None` when it cannot be read.
    pub fn read_u64(&self, addr: u64) -> Option<u64> {
        let mut out = [0; 8];
        probe::read(addr, &mut out).then(|| u64::from_le_bytes(out))
    }

    /// Store `bytes` at `addr` in program memory; false when it cannot be written.
    #[must_use]
    pub fn write(&mut self, addr: u64, bytes: &[u8]) -> bool {
        probe::write(addr, bytes)
    }

    /// `read` for a pointer an intrinsic or emulated library call was given: a trap when the
    /// memory cannot be read.
    fn fetch(&self, addr: u64, len: usize) -> Res<Vec<u8>> {
        match self.read(addr, len) {
            Some(bytes) => Ok(bytes),
            None => self.unreadable(addr, len),
        }
    }

    /// `read_u64` for a pointer an intrinsic or emulated library call was given.
    fn fetch_u64(&self, addr: u64) -> Res<u64> {
        match self.read_u64(addr) {
            Some(v) => Ok(v),
            None => self.unreadable(addr, 8),
        }
    }

    /// `write` for a pointer an intrinsic or emulated library call was given: a trap when the
    /// memory cannot be written.
    fn put(&mut self, addr: u64, bytes: &[u8]) -> Res<()> {
        if self.write(addr, bytes) {
            return Ok(());
        }
        if addr < 4096 {
            return self.null_trap(null_access("write", addr));
        }
        self.trap(format!(
            "invalid memory access: {} bytes at {addr:#x} cannot be written",
            bytes.len()
        ))
    }

    fn unreadable<T>(&self, addr: u64, len: usize) -> Res<T> {
        if addr < 4096 {
            return self.null_trap(null_access("read", addr));
        }
        self.trap(format!(
            "invalid memory access: {len} bytes at {addr:#x} cannot be read"
        ))
    }

    #[inline]
    fn load(&self, ty: Ty, addr: u64) -> Res<u64> {
        if addr < 4096 {
            return self.null_trap(null_access("read", addr));
        }
        if addr & TAG_MASK == FUNC_TAG || addr & TAG_MASK == FOREIGN_TAG {
            return self.trap("read through a procedure address");
        }
        let p = addr as *const u8;
        Ok(unsafe {
            match ty {
                Ty::I8 => *p as u64,
                Ty::I16 => std::ptr::read_unaligned(p as *const u16) as u64,
                Ty::I32 | Ty::F32 => std::ptr::read_unaligned(p as *const u32) as u64,
                Ty::I64 | Ty::F64 | Ty::Ptr => std::ptr::read_unaligned(p as *const u64),
                Ty::F80 | Ty::F128 => unreachable!("wide floats are never loaded into registers"),
            }
        })
    }

    #[inline]
    fn store(&self, ty: Ty, addr: u64, v: u64) -> Res<()> {
        if addr < 4096 {
            return self.null_trap(null_access("write", addr));
        }
        let p = addr as *mut u8;
        unsafe {
            match ty {
                Ty::I8 => *p = v as u8,
                Ty::I16 => std::ptr::write_unaligned(p as *mut u16, v as u16),
                Ty::I32 | Ty::F32 => std::ptr::write_unaligned(p as *mut u32, v as u32),
                Ty::I64 | Ty::F64 | Ty::Ptr => std::ptr::write_unaligned(p as *mut u64, v),
                Ty::F80 | Ty::F128 => unreachable!("wide floats are never stored from registers"),
            }
        }
        Ok(())
    }

    fn frame(&mut self, program: &Program, id: FuncId) -> Rc<Frame> {
        let i = id.0 as usize;
        let func = program.funcs[i].as_ref().unwrap();
        if let Some(Some(f)) = self.frames.get(i)
            && f.code.source == code::fingerprint(func)
        {
            return f.clone();
        }
        let mut offsets = Vec::with_capacity(func.slots.len());
        let mut size = 0u64;
        let mut frame_align = 16u64;
        for slot in &func.slots {
            let align = slot.align.clamp(8, 4096);
            frame_align = frame_align.max(align);
            size = size.next_multiple_of(align);
            offsets.push(size);
            size += slot.size.max(1);
        }
        // `#c_call` procedures may be thunks (`proc_value`).
        let native = self.host.native_linking();
        let late_value = |f: FuncId| {
            native
                && program
                    .funcs
                    .get(f.0 as usize)
                    .is_none_or(|g| g.as_ref().is_none_or(|g| g.sig.conv == ir::Conv::C))
        };
        let code = code::build(func, &offsets, &late_value);
        let frame = Rc::new(Frame {
            offsets,
            size: size.next_multiple_of(16),
            align: frame_align,
            code,
        });
        if self.frames.len() <= i {
            self.frames.resize_with(i + 1, || None);
        }
        self.frames[i] = Some(frame.clone());
        frame
    }

    // -----------------------------------------------------------------------
    // Foreign symbols
    // -----------------------------------------------------------------------

    pub fn foreign_addr(&mut self, program: &Program, id: ForeignId) -> Res<u64> {
        if let Some(&a) = self.foreign_addrs.get(id.0 as usize)
            && a != 0
        {
            return Ok(a);
        }
        let foreign = &program.foreigns[id.0 as usize];
        let counted = (self.host.native_linking() && !foreign.is_data)
            .then(|| crate::memory_limit::foreign_override(&foreign.symbol))
            .flatten();
        let addr = if counted.is_some() {
            counted
        } else if self.host.native_linking() {
            let lib = match foreign.library {
                Some(l) => {
                    let lib_info = &program.libraries[l];
                    self.libraries
                        .entry(l)
                        .or_insert_with(|| {
                            native::Library::open(
                                &lib_info.name,
                                lib_info.system,
                                &lib_info.base_dir,
                            )
                        })
                        .clone()
                }
                None => None,
            };
            native::lookup(lib.as_ref(), &foreign.symbol)
        } else {
            None
        };
        let addr = match addr {
            Some(a) => a,
            None if foreign.is_data => {
                return self.trap(format!(
                    "foreign variable `{}` is not available",
                    foreign.symbol
                ));
            }
            None => FOREIGN_TAG | id.0 as u64,
        };
        if self.foreign_addrs.len() <= id.0 as usize {
            self.foreign_addrs.resize(id.0 as usize + 1, 0);
        }
        self.foreign_addrs[id.0 as usize] = addr;
        Ok(addr)
    }

    fn call_foreign(
        &mut self,
        program: &Program,
        id: ForeignId,
        args: &[u64],
        sig: &ir::Sig,
    ) -> Res<Vec<u64>> {
        let symbol = program.foreigns[id.0 as usize].symbol.clone();
        // Foreign code may release memory the compiler has found readable.
        if !KEEPS_MEMORY.contains(&symbol.as_str()) {
            probe::invalidate();
        }
        if !UNOBSERVABLE_FOREIGNS.contains(&symbol.as_str()) {
            self.effects += 1;
        }
        if !self.compile_time
            && let Some(result) = self.executable_path_foreign(&symbol, args)
        {
            return result;
        }
        if self.host.cooperative_threads() {
            if let Some(result) = self.inline_thread_foreign(program, &symbol, args) {
                return result;
            }
        } else {
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(result) = self.thread_foreign(program, &symbol, args) {
                return result;
            }
        }
        if let Some(result) = self.host.foreign(&symbol, args, sig) {
            return result.map_err(|m| Trap {
                message: m,
                loc: self.loc,
                ..Trap::default()
            });
        }
        let addr = self.foreign_addr(program, id)?;
        if addr & TAG_MASK == FOREIGN_TAG {
            let mut trap = self
                .trap::<()>(format!(
                    "foreign procedure `{symbol}` is not available here"
                ))
                .unwrap_err();
            trap.kind = Some(TrapKind::Unavailable);
            return Err(trap);
        }
        #[cfg(target_os = "macos")]
        native::main_thread::note_symbol(&symbol);
        // The child of a fork has only the forking thread: it must not wait for the others.
        let forks = matches!(&*symbol, "fork" | "vfork");
        #[cfg(target_os = "macos")]
        let result = if forks {
            native::main_thread::direct(|| {
                self.call_native(program, addr, args, sig, Some(&symbol), false)
            })?
        } else {
            self.call_native(program, addr, args, sig, Some(&symbol), true)?
        };
        #[cfg(not(target_os = "macos"))]
        let result = self.call_native(program, addr, args, sig, Some(&symbol), !forks)?;
        if &*symbol == "fork" && result.first() == Some(&0) {
            self.forked_child = true;
            #[cfg(target_os = "macos")]
            native::main_thread::disable();
        }
        Ok(result)
    }

    /// Call C function `addr` (foreign procedure `symbol`, when it is one). Unless `release` is
    /// false, other threads may run interpreted code until it returns.
    fn call_native(
        &mut self,
        program: &Program,
        addr: u64,
        args: &[u64],
        sig: &ir::Sig,
        symbol: Option<&str>,
        release: bool,
    ) -> Res<Vec<u64>> {
        // Native code (through a procedure pointer too) may release memory, unless it is one of
        // the C library procedures known not to.
        if !symbol.is_some_and(|symbol| KEEPS_MEMORY.contains(&symbol)) {
            probe::invalidate();
        }
        // A `#c_call` procedure handed to C is a native thunk already (`proc_value`), unless C
        // cannot call it: that stayed tagged, and asking for its thunk again says why.
        let mut argv = args.to_vec();
        for v in &mut argv {
            if *v & TAG_MASK != FUNC_TAG {
                continue;
            }
            let id = FuncId((*v & !TAG_MASK) as u32);
            let Some(sig) = program.func_sig(id).filter(|s| s.conv == ir::Conv::C) else {
                continue;
            };
            let sig = sig.clone();
            *v = self.thunk(program, id, &sig).map_err(|m| Trap {
                message: m,
                loc: self.loc,
                ..Trap::default()
            })?;
        }
        #[cfg(not(target_arch = "wasm32"))]
        self.hand_thunks_to_c();
        let mine = self.take_exec_state();
        let loc = mine.loc;
        // A crash in C names `symbol` and this thread's interpreted call stack.
        #[cfg(not(target_arch = "wasm32"))]
        let crash_report = symbol.map(|symbol| crash::ForeignCall::enter(symbol, &mine, program));
        #[cfg(target_arch = "wasm32")]
        let _ = symbol;
        let result = self.call_unlocked(addr, &argv, sig, release);
        #[cfg(not(target_arch = "wasm32"))]
        drop(crash_report);
        self.put_exec_state(mine);
        result.map_err(|m| Trap {
            message: m,
            loc,
            ..Trap::default()
        })
    }

    /// `native::call`, during which other threads may take the baton (unless `release` is
    /// false) and C may call interpreted procedures back, on this thread or on others.
    #[cfg(not(target_arch = "wasm32"))]
    fn call_unlocked(
        &mut self,
        addr: u64,
        argv: &[u64],
        sig: &ir::Sig,
        release: bool,
    ) -> Result<Vec<u64>, String> {
        let Some(shared) = self.shared.clone().filter(|_| release) else {
            return native::call(addr, argv, sig);
        };
        let me = shared.enter_native(self as *mut Interp as usize);
        let result = native::calling_out((shared.key(), me), || native::call(addr, argv, sig));
        shared.leave_native(me);
        result
    }

    #[cfg(target_arch = "wasm32")]
    fn call_unlocked(
        &mut self,
        addr: u64,
        argv: &[u64],
        sig: &ir::Sig,
        _release: bool,
    ) -> Result<Vec<u64>, String> {
        native::call(addr, argv, sig)
    }

    /// The value of procedure `id` in interpreted code (`FuncAddr`, relocations). A `#c_call`
    /// procedure is a native thunk wherever the host links natively, so C can call it
    /// wherever the program stores it (struct fields, globals, arrays); interpreted calls
    /// through a thunk address are mapped back to the procedure. Anything else is a tagged
    /// id. A procedure keeps one value for the interpreter's lifetime.
    fn proc_value(&mut self, program: &Program, id: FuncId) -> u64 {
        let i = id.0 as usize;
        if let Some(&v) = self.proc_values.get(i)
            && v != 0
        {
            return v;
        }
        let tagged = FUNC_TAG | id.0 as u64;
        // The signature is known from the moment the procedure is reserved, so the value is
        // the same whether it is taken before or after the body is lowered.
        let value = match program.func_sig(id) {
            Some(sig) if sig.conv == ir::Conv::C && self.host.native_linking() => {
                // A procedure C cannot call (variadic, `long double`, out of thunks) stays
                // tagged; passing it to C directly reports why.
                let sig = sig.clone();
                self.thunk(program, id, &sig).unwrap_or(tagged)
            }
            _ => tagged,
        };
        if self.proc_values.len() <= i {
            self.proc_values.resize(i + 1, 0);
        }
        self.proc_values[i] = value;
        value
    }

    /// A C-callable thunk for `#c_call` procedure `id`.
    #[cfg(not(target_arch = "wasm32"))]
    fn thunk(&mut self, program: &Program, id: FuncId, sig: &ir::Sig) -> Result<u64, String> {
        let gate: std::sync::Arc<dyn native::Gate> = self.shared();
        let addr = native::callback_addr(&gate, program as *const Program as u64, id, sig)?;
        if self.thunk_funcs.insert(addr, id).is_none() {
            self.unseen_thunks.push(addr);
        }
        Ok(addr)
    }

    /// A foreign call is about to run: C can reach every thunk made so far (as an argument, or
    /// through memory the program shares with it), so from now on a thread C started may call
    /// back, and the scheduler must not take all-threads-blocked for a deadlock at once.
    #[cfg(not(target_arch = "wasm32"))]
    fn hand_thunks_to_c(&mut self) {
        if !self.unseen_thunks.is_empty() {
            self.unseen_thunks.clear();
            self.shared().callbacks_possible();
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn thunk(&mut self, _program: &Program, _id: FuncId, _sig: &ir::Sig) -> Result<u64, String> {
        Err("C cannot call interpreted procedures here".into())
    }

    /// The interpreted procedure a procedure value names: a tagged id, or a thunk this
    /// interpreter made.
    pub fn func_of(&self, value: u64) -> Option<FuncId> {
        if value & TAG_MASK == FUNC_TAG {
            return Some(FuncId((value & 0xFFFF_FFFF) as u32));
        }
        self.thunk_funcs.get(&value).copied()
    }

    fn take_exec_state(&mut self) -> ExecState {
        ExecState {
            stack: std::mem::take(&mut self.stack),
            sp: self.sp,
            depth: self.depth,
            calls: std::mem::take(&mut self.calls),
            loc: self.loc,
            trace_loc: self.trace_loc,
        }
    }

    fn put_exec_state(&mut self, state: ExecState) {
        self.stack = state.stack;
        self.sp = state.sp;
        self.depth = state.depth;
        self.calls = state.calls;
        self.loc = state.loc;
        self.trace_loc = state.trace_loc;
    }

    /// Run `func` for C, on whichever thread now holds the baton: on a value stack of its own,
    /// since a thread inside a native call keeps its frames.
    #[cfg(not(target_arch = "wasm32"))]
    fn run_callback(&mut self, program: &Program, func: FuncId, args: &[u64]) -> Res<Vec<u64>> {
        let stack = self
            .callback_stacks
            .pop()
            .unwrap_or_else(|| vec![0u64; STACK_SIZE / 8].into_boxed_slice());
        let outer = self.take_exec_state();
        self.put_exec_state(ExecState {
            stack,
            ..ExecState::default()
        });
        self.call_nesting += 1;
        let result = self.exec(program, func, args);
        self.call_nesting -= 1;
        let mine = self.take_exec_state();
        self.put_exec_state(outer);
        self.callback_stacks.push(mine.stack);
        result.map(Rets::into_vec)
    }

    // -----------------------------------------------------------------------
    // Execution
    // -----------------------------------------------------------------------

    /// Call a function with raw argument values.
    pub fn call(&mut self, program: &Program, func: FuncId, args: &[u64]) -> Res<Vec<u64>> {
        if self.stack.is_empty() {
            self.stack = vec![0u64; STACK_SIZE / 8].into_boxed_slice();
            self.sp = 0;
        }
        let (sp, depth, calls) = (self.sp, self.depth, self.calls.len());
        self.call_nesting += 1;
        let mut result = self.exec(program, func, args);
        if self.call_nesting == 1 && suspended(&result) {
            // A thread of the sandbox's scheduler blocked: run the others until this one ends.
            result = self.inline_schedule(program, result);
            if result.is_err() {
                (self.sp, self.depth) = (sp, depth);
                self.calls.truncate(calls);
            }
        }
        self.call_nesting -= 1;
        let result = result.map(Rets::into_vec);
        if self.forked_child {
            // Compile-time code forked and the child came back here (its `exec*` failed
            // or it trapped): it must not go on compiling alongside the parent.
            if let Err(t) = &result {
                eprintln!(
                    "error in a process forked by compile-time code: {}",
                    t.message
                );
            }
            exit_forked_child(if result.is_ok() {
                0
            } else {
                127
            });
        }
        result
    }

    fn exec(&mut self, program: &Program, id: FuncId, args: &[u64]) -> Res<Rets> {
        if let Some(&Some(hook)) = self.hooks.get(id.0 as usize) {
            return self.run_hook(hook, args).map(Rets::from);
        }
        let Some(func) = program.funcs.get(id.0 as usize).and_then(Option::as_ref) else {
            self.missing_func = Some(id);
            let name = program
                .func_names
                .get(id.0 as usize)
                .cloned()
                .unwrap_or_default();
            return self.trap(format!(
                "procedure `{name}` has no body available at this point"
            ));
        };
        if self.depth >= MAX_DEPTH {
            return self.trap_of(
                TrapKind::StackOverflow,
                "stack overflow (recursion too deep)",
            );
        }
        if let Some(covered) = self.covered.as_mut() {
            record_coverage(covered, program, id, func);
        }
        let frame = self.frame(program, id);
        let base = self.sp;
        let stack_start = self.stack.as_mut_ptr() as u64;
        let start = (stack_start + base).next_multiple_of(frame.align) - stack_start;
        let node_size = match &program.stack_trace {
            Some(layout) if func.trace.is_some() => layout.node.size.next_multiple_of(8),
            _ => 0,
        };
        let traced = node_size != 0;
        if start + frame.size + node_size > (self.stack.len() * 8) as u64 {
            return self.trap_of(TrapKind::StackOverflow, "interpreter stack overflow");
        }
        self.sp = start + frame.size + node_size;
        self.depth += 1;
        self.calls.push((id, self.loc));
        let stack_base = stack_start + start;
        let saved_loc = self.loc;
        let saved_trace_loc = self.trace_loc;
        let pushed = if traced {
            self.trace_enter(program, id, func, args, stack_base + frame.size)
        } else {
            None
        };
        if traced {
            self.trace_loc = None;
        } else if self.trace_loc.is_none() {
            self.trace_loc = Some(self.loc);
        }
        let outer = (self.frame_blocks, self.frame_insts);
        (self.frame_blocks, self.frame_insts) = (0, 0);
        let result = self.run(program, func, &frame, stack_base, args);
        let exit = FrameExit {
            id,
            base,
            saved_loc,
            saved_trace_loc,
            pushed,
            outer,
        };
        if suspended(&result) {
            // The thread is switching out: the frame stays on its value stack, to be resumed.
            (self.frame_blocks, self.frame_insts) = outer;
            self.capture_frame(exit, frame, stack_base);
            return result;
        }
        self.leave_frame(func, exit, result)
    }

    /// What `exec` restores when a frame returns (also for a frame `resume_frames` resumed).
    fn leave_frame(
        &mut self,
        func: &ir::Func,
        exit: FrameExit,
        mut result: Res<Rets>,
    ) -> Res<Rets> {
        if let Err(trap) = &mut result {
            trap.push_frame(func, self.loc);
        }
        if let Some(counts) = self.profile.as_mut() {
            counts.add(
                exit.id.0 as usize,
                &func.name,
                self.frame_blocks,
                self.frame_insts,
            );
        }
        (self.frame_blocks, self.frame_insts) = exit.outer;
        if let Some((slot, previous)) = exit.pushed {
            unsafe { std::ptr::write_unaligned(slot as *mut u64, previous) };
        }
        self.loc = exit.saved_loc;
        self.trace_loc = exit.saved_trace_loc;
        self.depth -= 1;
        self.calls.pop();
        self.sp = exit.base;
        result
    }

    /// Push this call onto `context.stack_trace`: the node lives in the caller-visible stack
    /// frame at `node`, and the caller's node learns the line the call was made from.
    /// Returns the context slot and the previous top, to restore on return.
    fn trace_enter(
        &mut self,
        program: &Program,
        id: FuncId,
        func: &ir::Func,
        args: &[u64],
        node: u64,
    ) -> Option<(u64, u64)> {
        let layout = program.stack_trace.as_ref()?;
        let n = &layout.node;
        let info = func.trace.as_ref()?;
        let context = *args.first()?;
        if context == 0 {
            return None;
        }
        let slot = context + layout.context;
        let line = self
            .trace_loc
            .unwrap_or(self.loc)
            .map_or(0, |(_, line, _)| line);
        unsafe {
            let previous = std::ptr::read_unaligned(slot as *const u64);
            let (mut depth, mut hash) = (1u32, 0xcbf2_9ce4_8422_2325u64);
            if previous != 0 {
                std::ptr::write_unaligned((previous + n.line_number) as *mut u32, line);
                depth = std::ptr::read_unaligned((previous + n.call_depth) as *const u32)
                    .wrapping_add(1);
                hash = std::ptr::read_unaligned((previous + n.hash) as *const u64);
            }
            hash = (hash ^ (id.0 as u64) ^ ((line as u64) << 32)).wrapping_mul(0x0100_0000_01b3);
            let info_addr = self.trace_info(program, id, info);
            std::ptr::write_unaligned((node + n.next) as *mut u64, previous);
            std::ptr::write_unaligned((node + n.info) as *mut u64, info_addr);
            std::ptr::write_unaligned((node + n.hash) as *mut u64, hash);
            std::ptr::write_unaligned((node + n.call_depth) as *mut u32, depth);
            std::ptr::write_unaligned((node + n.line_number) as *mut u32, info.line);
            std::ptr::write_unaligned(slot as *mut u64, node);
            Some((slot, previous))
        }
    }

    /// The `Stack_Trace_Procedure_Info` of a procedure (name, declaration site, address).
    fn trace_info(&mut self, program: &Program, id: FuncId, info: &ir::TraceInfo) -> u64 {
        if let Some(&addr) = self.trace_infos.get(id.0 as usize)
            && addr != 0
        {
            return addr;
        }
        fn leak(text: &str) -> u64 {
            Box::leak(text.as_bytes().to_vec().into_boxed_slice()).as_ptr() as u64
        }
        let path = program
            .file_paths
            .get(info.file as usize)
            .map_or("", String::as_str);
        let Some(layout) = &program.stack_trace else {
            return 0;
        };
        let at = &layout.info;
        let words = vec![0u64; at.size.div_ceil(8) as usize];
        let addr = Box::leak(words.into_boxed_slice()).as_mut_ptr() as u64;
        // A string is {count, data}.
        for (offset, value) in [
            (at.name, info.name.len() as u64),
            (at.name + 8, leak(&info.name)),
            (at.path, path.len() as u64),
            (at.path + 8, leak(path)),
            (at.line, info.line as u64),
            (at.column, info.col as u64),
            (at.procedure_address, FUNC_TAG | id.0 as u64),
        ] {
            unsafe { std::ptr::write_unaligned((addr + offset) as *mut u64, value) };
        }
        if self.trace_infos.len() <= id.0 as usize {
            self.trace_infos.resize(id.0 as usize + 1, 0);
        }
        self.trace_infos[id.0 as usize] = addr;
        addr
    }

    fn run_hook(&mut self, hook: Hook, args: &[u64]) -> Res<Vec<u64>> {
        match hook {
            Hook::WriteString => {
                let s = args[0];
                let count = self.fetch_u64(s)? as usize;
                let data = self.fetch_u64(s + 8)?;
                let bytes = self.fetch(data, count)?;
                self.effects += 1;
                self.host
                    .write(&bytes, args.get(1).is_some_and(|&v| v & 1 != 0));
            }
            Hook::WriteStrings => {
                let view = args[0];
                let count = self.fetch_u64(view)? as usize;
                let data = self.fetch_u64(view + 8)?;
                let to_stderr = args.get(1).is_some_and(|&v| v & 1 != 0);
                self.effects += 1;
                for i in 0..count {
                    let s = data + i as u64 * 16;
                    let n = self.fetch_u64(s)? as usize;
                    let p = self.fetch_u64(s + 8)?;
                    let bytes = self.fetch(p, n)?;
                    self.host.write(&bytes, to_stderr);
                }
            }
            Hook::DebugBreak => return self.trap("debug_break() was called"),
            Hook::AssertionFailed {
                context,
                location,
            } => {
                let args = &args[usize::from(context)..];
                let string = |s: &Self, at: u64| -> Res<String> {
                    let count = s.fetch_u64(at)? as usize;
                    let data = s.fetch_u64(at + 8)?;
                    Ok(String::from_utf8_lossy(&s.fetch(data, count)?).into_owned())
                };
                let (loc, message) = (args[0], args[1]);
                let path = string(self, loc + location.path)?;
                let line = self.fetch_u64(loc + location.line)? as u32;
                let col = self.fetch_u64(loc + location.column)? as u32;
                let message = string(self, message)?;
                let mut trap = self
                    .trap::<()>(if message.is_empty() {
                        "assertion failed".to_string()
                    } else {
                        format!("assertion failed: {message}")
                    })
                    .unwrap_err();
                trap.assertion = Some(Box::new((path, line, col)));
                if message.is_empty() {
                    trap.kind = Some(TrapKind::BareAssertion);
                }
                return Err(trap);
            }
            Hook::Meta(op, has_context) => {
                let Some(workspaces) = self.workspaces.clone() else {
                    return self.trap("this build has no compiler workspaces (Compiler module)");
                };
                let loc = self.loc;
                return crate::build::call(&workspaces, op, has_context, args, self).map_err(
                    |mut t| {
                        t.loc = t.loc.or(loc);
                        t
                    },
                );
            }
        }
        Ok(Vec::new())
    }

    fn run(
        &mut self,
        program: &Program,
        func: &ir::Func,
        frame: &Frame,
        stack_base: u64,
        args: &[u64],
    ) -> Res<Rets> {
        // Value registers come from a pool: a fresh Vec per call is a malloc/free pair.
        let mut vals = self.val_pool.pop().unwrap_or_default();
        vals.clear();
        vals.resize(func.vals.len(), 0);
        vals[..args.len().min(func.sig.params.len())]
            .copy_from_slice(&args[..args.len().min(func.sig.params.len())]);
        let result = self.run_code(program, func, frame, stack_base, &mut vals, None);
        if suspended(&result) {
            self.suspend_vals = Some(vals);
        } else {
            self.val_pool.push(vals);
        }
        result
    }

    fn step(
        &mut self,
        program: &Program,
        inst: &Inst,
        vals: &mut [u64],
        frame: &Frame,
        stack_base: u64,
    ) -> Res<()> {
        match inst {
            Inst::IConst {
                dst,
                ty,
                value,
            } => vals[dst.0 as usize] = mask(*ty, *value),
            Inst::FConst {
                dst,
                ty,
                value,
            } => {
                vals[dst.0 as usize] = if *ty == Ty::F32 {
                    (*value as f32).to_bits() as u64
                } else {
                    value.to_bits()
                };
            }
            Inst::Bin {
                dst,
                op,
                ty,
                a,
                b,
            } => {
                let (x, y) = (vals[a.0 as usize], vals[b.0 as usize]);
                vals[dst.0 as usize] = self.bin(*op, *ty, x, y)?;
            }
            Inst::Un {
                dst,
                op,
                ty,
                a,
            } => {
                let x = vals[a.0 as usize];
                vals[dst.0 as usize] = match op {
                    UnOp::Neg => mask(*ty, x.wrapping_neg()),
                    UnOp::Not => mask(*ty, !x),
                    UnOp::FNeg => {
                        if *ty == Ty::F32 {
                            (-f32::from_bits(x as u32)).to_bits() as u64
                        } else {
                            (-f64::from_bits(x)).to_bits()
                        }
                    }
                };
            }
            Inst::Cmp {
                dst,
                op,
                ty,
                a,
                b,
            } => {
                let (x, y) = (vals[a.0 as usize], vals[b.0 as usize]);
                vals[dst.0 as usize] = cmp(*op, *ty, x, y) as u64;
            }
            Inst::Conv {
                dst,
                op,
                from,
                to,
                src,
            } => vals[dst.0 as usize] = conv(*op, *from, *to, vals[src.0 as usize]),
            Inst::SlotAddr {
                dst,
                slot,
            } => vals[dst.0 as usize] = stack_base + frame.offsets[slot.0 as usize],
            Inst::GlobalAddr {
                dst,
                global,
            } => vals[dst.0 as usize] = self.global_addr(program, *global)?,
            Inst::FuncAddr {
                dst,
                func,
            } => vals[dst.0 as usize] = self.proc_value(program, *func),
            Inst::ForeignAddr {
                dst,
                foreign,
            } => vals[dst.0 as usize] = self.foreign_addr(program, *foreign)?,
            Inst::Load {
                dst,
                ty,
                addr,
            } => vals[dst.0 as usize] = self.load(*ty, vals[addr.0 as usize])?,
            Inst::Store {
                ty,
                addr,
                value,
            } => self.store(*ty, vals[addr.0 as usize], vals[value.0 as usize])?,
            Inst::PtrAdd {
                dst,
                base,
                offset,
            } => vals[dst.0 as usize] = vals[base.0 as usize].wrapping_add(vals[offset.0 as usize]),
            Inst::Copy {
                dst,
                src,
                size,
            } => {
                let (d, s) = (vals[dst.0 as usize], vals[src.0 as usize]);
                if d < 4096 || s < 4096 {
                    return self.null_trap("memory copy through a null pointer");
                }
                unsafe { std::ptr::copy(s as *const u8, d as *mut u8, *size as usize) };
            }
            Inst::Zero {
                dst,
                size,
            } => {
                let d = vals[dst.0 as usize];
                if d < 4096 {
                    return self.null_trap("memory fill through a null pointer");
                }
                unsafe { std::ptr::write_bytes(d as *mut u8, 0, *size as usize) };
            }
            Inst::Call(call) => {
                let ir::CallInst {
                    results,
                    callee,
                    args,
                } = &**call;
                let (mut small, mut heap) = ([0u64; 8], Vec::new());
                let argv = gather(vals, args, &mut small, &mut heap);
                let out = match callee {
                    Callee::Func(f) => self.exec(program, *f, argv)?,
                    Callee::Foreign(f) => {
                        let sig = program.foreigns[f.0 as usize].sig.clone();
                        self.call_foreign(program, *f, argv, &sig)?.into()
                    }
                    Callee::Indirect(target, sig) => {
                        let addr = vals[target.0 as usize];
                        match addr & TAG_MASK {
                            FUNC_TAG => {
                                self.exec(program, FuncId((addr & !TAG_MASK) as u32), argv)?
                            }
                            FOREIGN_TAG => self
                                .call_foreign(
                                    program,
                                    ForeignId((addr & !TAG_MASK) as u32),
                                    argv,
                                    sig,
                                )?
                                .into(),
                            _ if addr < 4096 => {
                                return self.null_trap("call through a null procedure pointer");
                            }
                            _ => match self.thunk_funcs.get(&addr) {
                                // A thunk this interpreter made: no need to go through C.
                                Some(&f) => self.exec(program, f, argv)?,
                                None => self
                                    .call_native(program, addr, argv, sig, None, true)?
                                    .into(),
                            },
                        }
                    }
                };
                for (r, &v) in results.iter().zip(out.iter()) {
                    vals[r.0 as usize] = v;
                }
            }
            Inst::Intrinsic(call) => {
                let ir::IntrinsicInst {
                    results,
                    op,
                    args,
                } = &**call;
                let (mut small, mut heap) = ([0u64; 8], Vec::new());
                let argv = gather(vals, args, &mut small, &mut heap);
                if *op == ir::Intrinsic::CheckFailed {
                    return self.check_failed(program, argv);
                }
                let out = self.intrinsic(*op, argv, results.first().map(|_| ()).is_some())?;
                for (r, v) in results.iter().zip(out) {
                    vals[r.0 as usize] = v;
                }
            }
            Inst::Loc {
                line,
                col,
                file,
                ..
            } => self.loc = Some((*file, *line, *col)),
        }
        Ok(())
    }

    fn bin(&self, op: BinOp, ty: Ty, x: u64, y: u64) -> Res<u64> {
        let bits = ty.size() as u32 * 8;
        Ok(match op {
            BinOp::Add => mask(ty, x.wrapping_add(y)),
            BinOp::Sub => mask(ty, x.wrapping_sub(y)),
            BinOp::Mul => mask(ty, x.wrapping_mul(y)),
            BinOp::SDiv | BinOp::SRem => {
                let (a, b) = (sext(ty, x), sext(ty, y));
                if b == 0 {
                    return self.check_trap(ir::TRAP_DIVIDE_BY_ZERO, 0, 0);
                }
                let r = if op == BinOp::SDiv {
                    a.wrapping_div(b)
                } else {
                    a.wrapping_rem(b)
                };
                mask(ty, r as u64)
            }
            BinOp::UDiv | BinOp::URem => {
                if y == 0 {
                    return self.check_trap(ir::TRAP_DIVIDE_BY_ZERO, 0, 0);
                }
                if op == BinOp::UDiv {
                    x / y
                } else {
                    x % y
                }
            }
            BinOp::And => x & y,
            BinOp::Or => x | y,
            BinOp::Xor => x ^ y,
            BinOp::Shl => {
                if y >= bits as u64 {
                    0
                } else {
                    mask(ty, x << y)
                }
            }
            BinOp::LShr => {
                if y >= bits as u64 {
                    0
                } else {
                    x >> y
                }
            }
            BinOp::AShr => mask(ty, (sext(ty, x) >> y.min(63)) as u64),
            BinOp::Rotl | BinOp::Rotr => {
                let s = (y % bits as u64) as u32;
                let s = if op == BinOp::Rotr {
                    (bits - s) % bits
                } else {
                    s
                };
                if s == 0 {
                    x
                } else {
                    mask(ty, (x << s) | (x >> (bits - s)))
                }
            }
            BinOp::FAdd | BinOp::FSub | BinOp::FMul | BinOp::FDiv => {
                if ty == Ty::F32 {
                    let (a, b) = (f32::from_bits(x as u32), f32::from_bits(y as u32));
                    (match op {
                        BinOp::FAdd => a + b,
                        BinOp::FSub => a - b,
                        BinOp::FMul => a * b,
                        _ => a / b,
                    })
                    .to_bits() as u64
                } else {
                    let (a, b) = (f64::from_bits(x), f64::from_bits(y));
                    (match op {
                        BinOp::FAdd => a + b,
                        BinOp::FSub => a - b,
                        BinOp::FMul => a * b,
                        _ => a / b,
                    })
                    .to_bits()
                }
            }
        })
    }

    /// `Intrinsic::CheckFailed` (reason, a, b, fatal): a trap, or a warning with the location
    /// on stderr when the check is not fatal.
    fn check_failed(&mut self, program: &Program, a: &[u64]) -> Res<()> {
        if a[3] & 0xff != 0 {
            return self.check_trap(a[0], a[1], a[2]);
        }
        let message = ir::check_message(a[0], a[1], a[2]);
        let at = self
            .loc
            .and_then(|(file, line, _)| {
                let path = program.file_paths.get(file as usize)?;
                Some(format!(
                    "{}:{line}: ",
                    crate::display_path(std::path::Path::new(path))
                ))
            })
            .unwrap_or_default();
        self.effects += 1;
        self.host
            .write(format!("{at}warning: {message}\n").as_bytes(), true);
        Ok(())
    }

    fn intrinsic(&mut self, op: ir::Intrinsic, a: &[u64], _has_result: bool) -> Res<Vec<u64>> {
        use ir::Intrinsic as I;
        let f64_of = |x: u64| f64::from_bits(x);
        Ok(match op {
            I::Memcpy => {
                if a[2] > 0 {
                    if a[0] < 4096 || a[1] < 4096 {
                        return self.null_trap("memcpy through a null pointer");
                    }
                    unsafe { std::ptr::copy(a[1] as *const u8, a[0] as *mut u8, a[2] as usize) };
                }
                vec![]
            }
            I::Memset => {
                if a[2] > 0 {
                    if a[0] < 4096 {
                        return self.null_trap("memset through a null pointer");
                    }
                    unsafe { std::ptr::write_bytes(a[0] as *mut u8, a[1] as u8, a[2] as usize) };
                }
                vec![]
            }
            I::Memcmp => {
                let n = a[2] as usize;
                if n > 0 && (a[0] < 4096 || a[1] < 4096) {
                    return self.null_trap("memcmp through a null pointer");
                }
                let (x, y) = (self.read(a[0], n), self.read(a[1], n));
                let r: i16 = match x.cmp(&y) {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                };
                vec![r as u16 as u64]
            }
            I::CompareAndSwap => {
                // (ptr, old, new, width) -> (success, previous)
                let width = a.get(3).copied().unwrap_or(8);
                let ty = Ty::int(width);
                let current = self.load(ty, a[0])?;
                let success = current == mask(ty, a[1]);
                if success {
                    self.store(ty, a[0], a[2])?;
                }
                // A failed or no-op swap is a thread waiting for another one.
                if self.multi
                    && self.host.cooperative_threads()
                    && (!success || mask(ty, a[2]) == current)
                {
                    self.inline_poll();
                }
                vec![success as u64, current]
            }
            I::DebugBreak => return self.trap("debug_break() was called"),
            I::Trap => {
                return self.check_trap(a.first().copied().unwrap_or(0), 0, 0);
            }
            I::BoundsCheck => {
                let (index, count) = (a[0] as i64, a[1] as i64);
                if index < 0 || index >= count {
                    return self.check_trap(ir::TRAP_BOUNDS, a[0], a[1]);
                }
                vec![]
            }
            // Needs the program for its location: `step` handles it.
            I::CheckFailed => unreachable!("CheckFailed is handled by `step`"),
            I::CompilerWrite => {
                let bytes = self.fetch(a[0], a[1] as usize)?;
                self.effects += 1;
                self.host.write(&bytes, a.get(2).is_some_and(|&v| v != 0));
                vec![]
            }
            I::Sqrt => vec![f64_of(a[0]).sqrt().to_bits()],
            I::Sin => vec![f64_of(a[0]).sin().to_bits()],
            I::Cos => vec![f64_of(a[0]).cos().to_bits()],
            I::Floor => vec![f64_of(a[0]).floor().to_bits()],
            I::Ceil => vec![f64_of(a[0]).ceil().to_bits()],
            I::Round => vec![f64_of(a[0]).round().to_bits()],
            I::Trunc => vec![f64_of(a[0]).trunc().to_bits()],
            I::Fabs => vec![f64_of(a[0]).abs().to_bits()],
            I::Fma => vec![f64_of(a[0]).mul_add(f64_of(a[1]), f64_of(a[2])).to_bits()],
            I::ReturnAddress => vec![0],
            I::CycleCounter => vec![cycle_counter()],
            I::Pause => {
                if self.multi && self.host.cooperative_threads() {
                    self.inline_poll();
                }
                vec![]
            }
            I::Popcount => vec![a[0].count_ones() as u64],
            I::Ctlz => {
                let bits = a[1] as u32;
                let lead = if a[0] == 0 {
                    64
                } else {
                    a[0].leading_zeros()
                };
                vec![(lead - (64 - bits)) as u64]
            }
            I::Cttz => vec![(a[0].trailing_zeros()).min(a[1] as u32) as u64],
            I::Bswap => vec![a[0].swap_bytes() >> (64 - a[1] as u32)],
            I::IsCompileTime => vec![self.compile_time as u64],
            I::SAddOverflow | I::SSubOverflow | I::SMulOverflow => {
                let bits = (a[2] as u32 * 8).min(64);
                let wide = |v: u64| ((v as i128) << (128 - bits)) >> (128 - bits);
                let (x, y) = (wide(a[0]), wide(a[1]));
                let r = match op {
                    I::SAddOverflow => x + y,
                    I::SSubOverflow => x - y,
                    _ => x * y,
                };
                let limit = 1i128 << (bits - 1);
                vec![(r < -limit || r >= limit) as u64]
            }
            I::UAddOverflow | I::USubOverflow | I::UMulOverflow => {
                let bits = (a[2] as u32 * 8).min(64);
                let keep = if bits == 64 {
                    u64::MAX
                } else {
                    (1u64 << bits) - 1
                };
                let (x, y) = ((a[0] & keep) as i128, (a[1] & keep) as i128);
                let r = match op {
                    I::UAddOverflow => x + y,
                    I::USubOverflow => x - y,
                    _ => x * y,
                };
                vec![(r < 0 || r > keep as i128) as u64]
            }
            I::Wide(op, fmt) => self.wide(op, fmt, a)?,
        })
    }

    /// `long double` operations in software (`crate::wide_float`), whatever the host.
    fn wide(&mut self, op: ir::WideOp, fmt: ir::WideFloat, a: &[u64]) -> Res<Vec<u64>> {
        use crate::wide_float as w;
        use ir::WideOp as W;
        let value = |s: &Self, addr: u64| -> Res<w::Bytes> {
            if addr < 4096 {
                return s.null_trap(null_access("read", addr));
            }
            let mut out = [0u8; 16];
            out.copy_from_slice(&s.fetch(addr, 16)?);
            Ok(out)
        };
        let put = |s: &mut Self, addr: u64, v: w::Bytes| -> Res<Vec<u64>> {
            if addr < 4096 {
                return s.null_trap(null_access("write", addr));
            }
            if !s.write(addr, &v) {
                return s.trap(format!(
                    "invalid memory access: 16 bytes at {addr:#x} cannot be written"
                ));
            }
            Ok(Vec::new())
        };
        match op {
            W::Arith(kind) => {
                let r = w::arith(fmt, kind, &value(self, a[1])?, &value(self, a[2])?);
                put(self, a[0], r)
            }
            W::Neg => {
                let r = w::neg(fmt, &value(self, a[1])?);
                put(self, a[0], r)
            }
            W::Cmp(cmp) => {
                use std::cmp::Ordering as O;
                let order = w::compare(fmt, &value(self, a[0])?, &value(self, a[1])?);
                let yes = match (cmp, order) {
                    (CmpOp::FNe, None) => true,
                    (_, None) => false,
                    (CmpOp::FEq, Some(o)) => o == O::Equal,
                    (CmpOp::FNe, Some(o)) => o != O::Equal,
                    (CmpOp::FLt, Some(o)) => o == O::Less,
                    (CmpOp::FLe, Some(o)) => o != O::Greater,
                    (CmpOp::FGt, Some(o)) => o == O::Greater,
                    (CmpOp::FGe, Some(o)) => o != O::Less,
                    _ => return self.trap("invalid long double comparison"),
                };
                Ok(vec![yes as u64])
            }
            W::FromF64 => put(self, a[0], w::from_f64(fmt, f64::from_bits(a[1]))),
            W::FromF32 => put(self, a[0], w::from_f32(fmt, f32::from_bits(a[1] as u32))),
            W::FromS64 => put(self, a[0], w::from_i64(fmt, a[1] as i64)),
            W::FromU64 => put(self, a[0], w::from_u64(fmt, a[1])),
            W::ToF64 => Ok(vec![w::to_f64(fmt, &value(self, a[0])?).to_bits()]),
            W::ToF32 => Ok(vec![w::to_f32(fmt, &value(self, a[0])?).to_bits() as u64]),
            W::ToS64 => Ok(vec![w::to_i64(fmt, &value(self, a[0])?) as u64]),
            W::ToU64 => Ok(vec![w::to_u64(fmt, &value(self, a[0])?)]),
        }
    }
}

/// Native: nanoseconds since the epoch. wasm32 has no clock (`SystemTime::now` panics there), so
/// it counts calls instead.
fn cycle_counter() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        use std::cell::Cell;
        thread_local! { static TICKS: Cell<u64> = const { Cell::new(0) }; }
        TICKS.with(|t| {
            t.set(t.get() + 1);
            t.get()
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos() as u64)
    }
}

#[inline]
fn mask(ty: Ty, v: u64) -> u64 {
    match ty {
        Ty::I8 => v & 0xff,
        Ty::I16 => v & 0xffff,
        Ty::I32 | Ty::F32 => v & 0xffff_ffff,
        _ => v,
    }
}

#[inline]
fn sext(ty: Ty, v: u64) -> i64 {
    match ty {
        Ty::I8 => v as u8 as i8 as i64,
        Ty::I16 => v as u16 as i16 as i64,
        Ty::I32 => v as u32 as i32 as i64,
        _ => v as i64,
    }
}

#[inline]
fn cmp(op: CmpOp, ty: Ty, x: u64, y: u64) -> bool {
    let (ux, uy) = (mask(ty, x), mask(ty, y));
    let (sx, sy) = (sext(ty, x), sext(ty, y));
    let float = |x: u64| {
        if ty == Ty::F32 {
            f32::from_bits(x as u32) as f64
        } else {
            f64::from_bits(x)
        }
    };
    match op {
        CmpOp::Eq => ux == uy,
        CmpOp::Ne => ux != uy,
        CmpOp::SLt => sx < sy,
        CmpOp::SLe => sx <= sy,
        CmpOp::SGt => sx > sy,
        CmpOp::SGe => sx >= sy,
        CmpOp::ULt => ux < uy,
        CmpOp::ULe => ux <= uy,
        CmpOp::UGt => ux > uy,
        CmpOp::UGe => ux >= uy,
        CmpOp::FEq => float(x) == float(y),
        CmpOp::FNe => float(x) != float(y),
        CmpOp::FLt => float(x) < float(y),
        CmpOp::FLe => float(x) <= float(y),
        CmpOp::FGt => float(x) > float(y),
        CmpOp::FGe => float(x) >= float(y),
    }
}

fn conv(op: ConvOp, from: Ty, to: Ty, v: u64) -> u64 {
    let f = |v: u64| {
        if from == Ty::F32 {
            f32::from_bits(v as u32) as f64
        } else {
            f64::from_bits(v)
        }
    };
    let out_f = |x: f64| {
        if to == Ty::F32 {
            (x as f32).to_bits() as u64
        } else {
            x.to_bits()
        }
    };
    match op {
        ConvOp::Trunc => mask(to, v),
        ConvOp::ZExt => mask(from, v),
        ConvOp::SExt => mask(to, sext(from, v) as u64),
        ConvOp::FToS => mask(
            to,
            match to {
                Ty::I8 => f(v) as i8 as u64,
                Ty::I16 => f(v) as i16 as u64,
                Ty::I32 => f(v) as i32 as u64,
                _ => f(v) as i64 as u64,
            },
        ),
        ConvOp::FToU => mask(
            to,
            match to {
                Ty::I8 => f(v) as u8 as u64,
                Ty::I16 => f(v) as u16 as u64,
                Ty::I32 => f(v) as u32 as u64,
                _ => f(v) as u64,
            },
        ),
        // Straight to the target width: going through f64 first rounds twice, which is wrong
        // for 64-bit integers whose bits below f64's precision decide an f32 tie.
        ConvOp::SToF if to == Ty::F32 => (sext(from, v) as f32).to_bits() as u64,
        ConvOp::UToF if to == Ty::F32 => (mask(from, v) as f32).to_bits() as u64,
        ConvOp::SToF => out_f(sext(from, v) as f64),
        ConvOp::UToF => out_f(mask(from, v) as f64),
        ConvOp::FExt | ConvOp::FTrunc => out_f(f(v)),
        ConvOp::Bitcast => mask(to, v),
    }
}

/// Leave a process forked by compile-time code without running the parent's cleanup.
fn exit_forked_child(code: i32) -> ! {
    #[cfg(unix)]
    {
        unsafe extern "C" {
            fn _exit(code: i32) -> !;
        }
        unsafe { _exit(code) }
    }
    #[cfg(not(unix))]
    std::process::exit(code)
}

/// A call's results: up to four inline. A `Vec` per call was a malloc and free on every call.
#[derive(Debug, Default)]
pub struct Rets {
    len: usize,
    inline: [u64; 4],
    spill: Vec<u64>,
}

impl Rets {
    fn collect(values: impl ExactSizeIterator<Item = u64>) -> Self {
        let mut rets = Rets {
            len: values.len(),
            ..Default::default()
        };
        if rets.len <= 4 {
            for (slot, v) in rets.inline.iter_mut().zip(values) {
                *slot = v;
            }
        } else {
            rets.spill = values.collect();
        }
        rets
    }

    pub fn into_vec(self) -> Vec<u64> {
        if self.len <= 4 {
            self.inline[..self.len].to_vec()
        } else {
            self.spill
        }
    }
}

impl From<Vec<u64>> for Rets {
    fn from(spill: Vec<u64>) -> Self {
        Rets {
            len: usize::MAX,
            inline: [0; 4],
            spill,
        }
    }
}

impl std::ops::Deref for Rets {
    type Target = [u64];

    fn deref(&self) -> &[u64] {
        if self.len <= 4 {
            &self.inline[..self.len]
        } else {
            &self.spill
        }
    }
}

/// Reads an instruction's operands. Most fit in `small` on the Rust stack; longer lists use `heap`.
fn gather<'a>(
    vals: &[u64],
    args: &[ir::Val],
    small: &'a mut [u64; 8],
    heap: &'a mut Vec<u64>,
) -> &'a [u64] {
    if args.len() <= small.len() {
        for (slot, a) in small.iter_mut().zip(args) {
            *slot = vals[a.0 as usize];
        }
        &small[..args.len()]
    } else {
        *heap = args.iter().map(|a| vals[a.0 as usize]).collect();
        heap
    }
}

impl Interp {
    /// Merge this interpreter's `JAIC_PROFILE` counts into the process table (see `profile`).
    pub fn flush_profile(&mut self) {
        if let Some(counts) = self.profile.as_mut() {
            counts.flush();
        }
    }
}

/// Record procedure `id` for `JAIC_COVERAGE` the first time it runs.
#[cold]
fn record_coverage(covered: &mut Vec<bool>, program: &Program, id: FuncId, func: &ir::Func) {
    let index = id.0 as usize;
    if covered.len() <= index {
        covered.resize(index + 1, false);
    }
    if std::mem::replace(&mut covered[index], true) {
        return;
    }
    let (file, line, name) = match (&func.trace, &func.debug) {
        (Some(t), _) => (t.file, t.line, t.name.as_str()),
        (None, Some(d)) => (d.file, d.line, d.name.as_str()),
        (None, None) => (func.source_file, 0, func.name.as_str()),
    };
    let path = program
        .file_paths
        .get(file as usize)
        .map_or("", String::as_str);
    profile::cover(path, line, name);
}

impl Drop for Interp {
    fn drop(&mut self) {
        self.flush_profile();
        // Thunks C may still hold now fail with a message instead of running a dead interpreter.
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(shared) = self.shared.take() {
            shared.close();
            let gate: std::sync::Arc<dyn native::Gate> = shared;
            native::release_callbacks(&gate);
        }
    }
}
