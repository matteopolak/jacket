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

mod atomics;
mod code;
mod compile_calls;
pub(crate) use compile_calls::CompileCall;
#[cfg(not(target_arch = "wasm32"))]
mod crash;
mod errno;
#[cfg(not(target_arch = "wasm32"))]
pub use crash::CRASH_STATUS;
mod arith;
mod calls;
#[cfg(test)]
mod contention_tests;
mod executable_path;
mod fibers;
mod foreign;
mod frames;
mod globals;
mod host;
mod intrinsics;
mod last_error;
mod machine;
mod memory;
mod native;
mod probe;
pub mod profile;
mod read_only;
mod rets;
mod sandbox;
mod step;
#[cfg(not(target_arch = "wasm32"))]
mod threads;
mod threads_inline;
mod trap;
pub(super) use self::arith::{bin_total, cmp_shifted, divides, shift_of};
use self::arith::{conv, mask};
use self::calls::{FrameExit, suspended};
use self::frames::{ExecState, Frame, ValueStack};
use self::globals::GlobalMem;
pub(crate) use self::globals::ZeroedBlock;
pub use self::host::{Hook, Host, LocationLayout, NativeHost, PendingDataGlobal};
pub use self::rets::Rets;
pub use self::trap::{Trap, TrapDetails, TrapFrame, TrapKind};
use crate::fxhash::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use crate::ir::Ty;
use crate::ir::{self, FuncId, GlobalId, Program};
#[cfg(target_os = "macos")]
pub use native::main_thread;
pub use native::{
    add_library_dir, extra_library_dirs, homebrew_lib_dir, library_dirs, set_library_dirs,
};
pub use sandbox::{SandboxHost, SharedHost};
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(not(target_arch = "wasm32"))]
pub use threads::DEADLOCK_GRACE;

/// Tag bits marking an interpreted procedure address.
pub const FUNC_TAG: u64 = 0xFEED_0000_0000_0000;

/// Tag bits for foreign procedures that have no native address.
pub const FOREIGN_TAG: u64 = 0xFEEE_0000_0000_0000;

const TAG_MASK: u64 = 0xFFFF_0000_0000_0000;

const STACK_SIZE: usize = 32 << 20;
const MAX_DEPTH: usize = 20_000;

/// Word `i` of a call's arguments (or of an incoming stack area), or 0 when the call passed fewer.
///
/// A Win32 or syscall declaration can pass fewer words than the callee reads (an optional
/// out-parameter the caller left off, a trailing flags word), so a missing word reads as 0, the
/// value of an argument the caller did not set, rather than failing the call.
#[inline]
fn word_or_zero(words: &[u64], i: usize) -> u64 {
    words.get(i).copied().unwrap_or(0)
}

type Res<T> = std::result::Result<T, Trap>;

/// A running procedure and the location it was called from (`Interp::calls`).
type Call = (FuncId, Option<(u32, u32, u32)>);

/// Nanoseconds spent inside outermost compile-time calls (`#run`, metaprograms, macros),
/// summed over every interpreter; `-timings` reports it as the `interp` phase.
static COMPILE_TIME_NANOS: AtomicU64 = AtomicU64::new(0);

static COMPILE_TIME_CALLS: AtomicU64 = AtomicU64::new(0);

/// Wall time compile-time code has spent running so far and how many outermost calls that
/// took. It includes any compilation the code itself triggers (a metaprogram building a
/// workspace), so it overlaps `front end`.
pub fn compile_time_spent() -> (std::time::Duration, u64) {
    (
        std::time::Duration::from_nanos(COMPILE_TIME_NANOS.load(Ordering::Relaxed)),
        COMPILE_TIME_CALLS.load(Ordering::Relaxed),
    )
}

pub struct Interp {
    stack: ValueStack,
    sp: u64,
    globals: Vec<Option<GlobalMem>>,
    /// start address -> (end, global) for address lookups.
    ranges: BTreeMap<u64, (u64, GlobalId)>,
    /// Where the `read_only` globals live: memory interpreted code may not write.
    read_only: read_only::ReadOnlyData,
    /// Each procedure's frame layout and code, built on its first call. Frames are boxed and
    /// never freed while the interpreter lives (one replaced moves to `retired`), so a
    /// running procedure's frame stays where it is.
    frames: Vec<Option<Box<Frame>>>,
    #[allow(clippy::vec_box)] // The boxes are what keeps the frames in place.
    retired: Vec<Box<Frame>>,
    /// Resolved foreign symbols by `ForeignId` (0 = not resolved yet).
    foreign_addrs: Vec<u64>,
    /// What each foreign procedure's calls do (`foreign_effects`); 0 until looked at.
    foreign_flags: Vec<u8>,
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
    /// The executable `jaic` would write for the program `jaic -interp` is running: what the
    /// program's own executable-path queries return (not `jaic`'s path), so paths relative to
    /// the executable (`../assets`) work the same in both. `None`: no substitution.
    pub run_executable: Option<String>,
    /// The arguments `jaic -interp` hands the program (`argv`): what its Windows command-line
    /// query (`GetCommandLineW`) returns while `run_executable` is set, not `jaic`'s own.
    pub run_arguments: Vec<String>,
    /// `run_arguments` as a Windows command line, made on the first query and kept for the
    /// rest of the process, as the OS keeps its own.
    run_command_line: Option<u64>,
    /// The table `get_type_table()` gives compile-time code: address and count of descriptor
    /// pointers, in a vector kept in `ct_type_tables` (earlier ones stay for runs still using them).
    pub ct_type_table: (u64, u64),
    pub ct_type_tables: Vec<Vec<u64>>,
    /// `compiler_set_type_info_flags` calls (type descriptor global, flags) not applied yet.
    pub pending_type_flags: Vec<(GlobalId, u32)>,
    /// Declaration of each struct whose descriptor exists: file path, line, column
    /// (`compiler_get_struct_location`).
    pub struct_locations: crate::fxhash::HashMap<GlobalId, (std::rc::Rc<str>, i64, i64)>,
    /// Workspace registry for the `Compiler` module, when the embedder has one.
    pub workspaces: Option<crate::build::SharedWorkspaces>,
    /// Bodies and source text of the compiler's `Code` values, by `CodeId`.
    pub codes: Vec<(std::rc::Rc<crate::ast::CodeBody>, std::rc::Rc<str>)>,
    /// Codes made by compile-time code (`compiler_get_code`): their index and the
    /// index of the code whose scope they take. The compiler adopts them lazily.
    pub made_codes: Vec<(usize, usize)>,
    pub pending_data_globals: Vec<PendingDataGlobal>,
    data_segment_tags: HashMap<GlobalId, u16>,
    runtime_data_snapshots: Vec<Box<[u64]>>,
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
    defer_missing_bodies: bool,
    callable_request: Option<(FuncId, bool)>,
    inspected_callables: crate::fxhash::HashSet<FuncId>,
    ready_callables: crate::fxhash::HashSet<FuncId>,
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
    /// While a thread suspends (`TrapKind::Suspended`): the op where the innermost `run_code`
    /// stopped, for `exec` to save.
    suspend_at: Option<usize>,
    /// The frames of the suspending thread saved so far, innermost first.
    captured: Vec<threads_inline::SuspFrame>,
    /// Basic blocks left to run before execution traps (editors bound compile-time code).
    pub block_budget: Option<u64>,
    /// The pages of program memory the compiler found accessible (see `probe`).
    probe: Box<probe::Probe>,
    /// Blocks and instructions run in the current frame (`-profile`, see `profile`).
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
    callback_stacks: Vec<ValueStack>,
    /// Fibers by the address of their `Fiber_Context` (`fibers.rs`).
    #[cfg(all(
        any(unix, windows),
        any(target_arch = "x86_64", target_arch = "aarch64")
    ))]
    fibers: HashMap<u64, Box<fibers::FiberRec>>,
    /// A trap a fiber ended in, for the context that started it to return.
    #[cfg(all(
        any(unix, windows),
        any(target_arch = "x86_64", target_arch = "aarch64")
    ))]
    fiber_trap: Option<Trap>,
    /// `release_code` ran: no code runs anymore.
    closed: bool,
    /// Procedures already recorded for `-coverage_file`, by `FuncId`; `None` when it is unset.
    covered: Option<Vec<bool>>,
}

impl Default for Interp {
    fn default() -> Self {
        Self::new(Box::new(NativeHost))
    }
}

impl Interp {
    pub fn new(host: Box<dyn Host>) -> Self {
        Self {
            stack: ValueStack::default(),
            sp: 0,
            globals: Vec::new(),
            ranges: BTreeMap::new(),
            read_only: read_only::ReadOnlyData::default(),
            frames: Vec::new(),
            retired: Vec::new(),
            foreign_addrs: Vec::new(),
            libraries: HashMap::default(),
            hooks: Vec::new(),
            host,
            depth: 0,
            calls: Vec::new(),
            loc: None,
            trace_loc: None,
            compile_time: true,
            foreign_flags: Vec::new(),
            run_executable: None,
            run_arguments: Vec::new(),
            run_command_line: None,
            ct_type_table: (0, 0),
            ct_type_tables: Vec::new(),
            pending_type_flags: Vec::new(),
            struct_locations: crate::fxhash::HashMap::default(),
            workspaces: None,
            codes: Vec::new(),
            made_codes: Vec::new(),
            pending_data_globals: Vec::new(),
            data_segment_tags: HashMap::default(),
            runtime_data_snapshots: Vec::new(),
            run_codes: (Vec::new(), 0),
            nodes_codes: Vec::new(),
            effects: 0,
            run_effects: None,
            export_request: None,
            missing_func: None,
            defer_missing_bodies: false,
            callable_request: None,
            inspected_callables: crate::fxhash::HashSet::default(),
            ready_callables: crate::fxhash::HashSet::default(),
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
            captured: Vec::new(),
            block_budget: None,
            probe: Box::default(),
            frame_blocks: 0,
            frame_insts: 0,
            profile: profile::enabled().then(Default::default),
            proc_values: Vec::new(),
            thunk_funcs: HashMap::default(),
            #[cfg(not(target_arch = "wasm32"))]
            unseen_thunks: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            callback_stacks: Vec::new(),
            #[cfg(all(
                any(unix, windows),
                any(target_arch = "x86_64", target_arch = "aarch64")
            ))]
            fibers: HashMap::default(),
            #[cfg(all(
                any(unix, windows),
                any(target_arch = "x86_64", target_arch = "aarch64")
            ))]
            fiber_trap: None,
            closed: false,
            covered: profile::coverage_file().map(|_| Vec::new()),
        }
    }

    /// Stop running code for good: every later call traps. Drops the procedures' frames (their
    /// bodies were released), the value stacks and the globals' memory (a global asked for
    /// again is made anew); nothing may be running.
    pub fn release_code(&mut self) {
        self.closed = true;
        self.frames = Vec::new();
        self.retired = Vec::new();
        self.globals = Vec::new();
        self.ranges = BTreeMap::new();
        self.read_only = read_only::ReadOnlyData::default();
        self.ct_type_tables = Vec::new();
        self.ct_type_table = (0, 0);
        self.stack = ValueStack::default();
        self.sp = 0;
        #[cfg(all(
            any(unix, windows),
            any(target_arch = "x86_64", target_arch = "aarch64")
        ))]
        {
            self.fibers = HashMap::default();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.callback_stacks = Vec::new();
        }
    }


    /// Merge this interpreter's `-profile` counts into the process table (see `profile`).
    pub fn flush_profile(&mut self) {
        if let Some(counts) = self.profile.as_mut() {
            counts.flush();
        }
    }
}

/// Leave a process forked by compile-time code without running the parent's cleanup.
fn exit_forked_child(code: i32) -> ! {
    #[cfg(unix)]
    {
        unsafe extern "C" {
            fn _exit(code: i32) -> !;
        }
        // SAFETY: `_exit` ends the process at once, without unwinding, and accepts any status.
        unsafe { _exit(code) }
    }
    #[cfg(not(unix))]
    std::process::exit(code)
}

/// Record procedure `id` for `-coverage_file` the first time it runs.
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

impl Interp {
    /// Stop for good: flush the profile, stop and join the Jai threads, and make the thunks C
    /// may still hold fail with a message instead of running a dead interpreter. Calling it
    /// again does nothing more.
    pub fn shutdown(&mut self) {
        self.flush_profile();
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(shared) = self.shared.take() {
            shared.close();
            let gate: std::sync::Arc<dyn native::Gate> = shared;
            native::release_callbacks(&gate);
        }
    }
}

impl Drop for Interp {
    fn drop(&mut self) {
        self.shutdown();
    }
}
