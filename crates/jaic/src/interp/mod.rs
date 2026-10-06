//! IR interpreter. Runs `#run` code at compile time and whole programs in
//! scripting mode and in the browser.
//!
//! Memory is real: pointers are host addresses, globals and stack slots live
//! in host allocations, and foreign procedures are called natively (or
//! through `Host` shims where no dynamic linker exists, e.g. wasm). Procedure
//! values are tagged non-canonical addresses that only the interpreter can
//! call.
#![allow(unsafe_code)]

mod native;
pub mod profile;
mod sandbox;
#[cfg(not(target_arch = "wasm32"))]
mod threads;
mod threads_inline;
use crate::fxhash::HashMap;
use crate::ir::{
    self, BinOp, Callee, CmpOp, ConvOp, ForeignId, FuncId, GlobalId, Inst, Program, Term, Ty, UnOp,
};
#[cfg(target_os = "macos")]
pub use native::main_thread;
pub use native::{library_dirs, set_library_dirs};
pub use sandbox::{SandboxHost, SharedHost};
use std::collections::BTreeMap;
use std::rc::Rc;

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
    /// A `Compiler` module primitive (`__jaic_*`); the flag says whether the
    /// procedure takes a context pointer.
    Meta(crate::build::MetaOp, bool),
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
#[derive(Debug, Clone)]
pub struct Trap {
    pub message: String,
    /// (file, line, column) of the last executed statement.
    pub loc: Option<(u32, u32, u32)>,
}

type Res<T> = std::result::Result<T, Trap>;

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

struct Frame {
    offsets: Vec<u64>,
    size: u64,
    /// Largest slot alignment: the frame's absolute start address is a multiple of it, so
    /// `#align 64` locals land aligned (offsets alone only align relative to the frame).
    align: u64,
}

/// Size of a `Stack_Trace_Node`.
const TRACE_NODE_SIZE: u64 = 32;

struct GlobalMem {
    _storage: Box<[u64]>,
    addr: u64,
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
    loc: Option<(u32, u32, u32)>,
    /// Inside procedures without a stack trace node (`Some`): the location of the call that
    /// entered them from a traced procedure. A traced call made there reports that line for
    /// the caller's node, not a line of the untraced callee's source.
    trace_loc: Option<Option<(u32, u32, u32)>>,
    /// True while evaluating compile-time code (`#compile_time`).
    pub compile_time: bool,
    /// `compiler_set_type_info_flags` calls (type descriptor global, flags) not applied yet.
    pub pending_type_flags: Vec<(GlobalId, u32)>,
    /// Workspace registry for the `Compiler` module, when the embedder has one.
    pub workspaces: Option<crate::build::SharedWorkspaces>,
    /// Bodies and source text of the compiler's `Code` values, by `CodeId`.
    pub codes: Vec<(std::rc::Rc<crate::ast::CodeBody>, std::rc::Rc<str>)>,
    /// Codes made by compile-time code (`compiler_get_code`): their index and the
    /// index of the code whose scope they take. The compiler adopts them lazily.
    pub made_codes: Vec<(usize, usize)>,
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
    /// Threads of the running program (created by the first `pthread_*` call).
    #[cfg(not(target_arch = "wasm32"))]
    sched: Option<Box<threads::Sched>>,
    /// Threads of the running program when the host schedules them cooperatively.
    isched: Option<Box<threads_inline::InlineSched>>,
    /// More than one thread exists: `run` offers the baton to the others now and then.
    multi: bool,
    /// Basic blocks left to run before execution traps (editors bound compile-time code).
    pub block_budget: Option<u64>,
    /// Reused value-register vectors (see `run`).
    val_pool: Vec<Vec<u64>>,
    /// Blocks and instructions run in the current frame (`JAIC_PROFILE`, see `profile`).
    frame_blocks: u64,
    frame_insts: u64,
    profile: Option<Box<profile::Counts>>,
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
            loc: None,
            trace_loc: None,
            compile_time: true,
            pending_type_flags: Vec::new(),
            workspaces: None,
            codes: Vec::new(),
            made_codes: Vec::new(),
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
            sched: None,
            isched: None,
            multi: false,
            block_budget: None,
            val_pool: Vec::new(),
            frame_blocks: 0,
            frame_insts: 0,
            profile: profile::enabled().then(Default::default),
        }
    }

    #[cold]
    fn trap<T>(&self, message: impl Into<String>) -> Res<T> {
        Err(Trap {
            message: message.into(),
            loc: self.loc,
        })
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
        let align = global.align.max(8);
        let words = (global.size + align).div_ceil(8) as usize;
        let mut storage = vec![0u64; words.max(1)].into_boxed_slice();
        let base = storage.as_mut_ptr() as u64;
        let addr = base.next_multiple_of(align);
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
                ir::RelocTarget::Func(f) => FUNC_TAG | f.0 as u64,
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
                ir::RelocTarget::Func(f) => FUNC_TAG | f.0 as u64,
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
                    ir::RelocTarget::Func(f) => FUNC_TAG | f.0 as u64,
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

    pub fn read(&self, addr: u64, len: usize) -> Vec<u8> {
        if len == 0 {
            return Vec::new();
        }
        unsafe { std::slice::from_raw_parts(addr as *const u8, len) }.to_vec()
    }
    /// `len` bytes of program memory at `addr`, borrowed (empty for a null address).
    pub fn bytes(&self, addr: u64, len: usize) -> &[u8] {
        if len == 0 || addr == 0 {
            return &[];
        }
        unsafe { std::slice::from_raw_parts(addr as *const u8, len) }
    }
    pub fn read_u64(&self, addr: u64) -> u64 {
        unsafe { std::ptr::read_unaligned(addr as *const u64) }
    }
    pub fn write(&mut self, addr: u64, bytes: &[u8]) {
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), addr as *mut u8, bytes.len()) };
    }

    #[inline]
    fn load(&self, ty: Ty, addr: u64) -> Res<u64> {
        if addr < 4096 {
            return self.trap(format!(
                "invalid memory read at address {addr:#x} (null pointer?)"
            ));
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
            }
        })
    }

    #[inline]
    fn store(&self, ty: Ty, addr: u64, v: u64) -> Res<()> {
        if addr < 4096 {
            return self.trap(format!(
                "invalid memory write at address {addr:#x} (null pointer?)"
            ));
        }
        let p = addr as *mut u8;
        unsafe {
            match ty {
                Ty::I8 => *p = v as u8,
                Ty::I16 => std::ptr::write_unaligned(p as *mut u16, v as u16),
                Ty::I32 | Ty::F32 => std::ptr::write_unaligned(p as *mut u32, v as u32),
                Ty::I64 | Ty::F64 | Ty::Ptr => std::ptr::write_unaligned(p as *mut u64, v),
            }
        }
        Ok(())
    }

    fn frame(&mut self, program: &Program, id: FuncId) -> Rc<Frame> {
        let i = id.0 as usize;
        if let Some(Some(f)) = self.frames.get(i) {
            return f.clone();
        }
        let func = program.funcs[i].as_ref().unwrap();
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
        let frame = Rc::new(Frame {
            offsets,
            size: size.next_multiple_of(16),
            align: frame_align,
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
        let addr = if self.host.native_linking() {
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
                    "foreign variable '{}' is not available",
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
        if !UNOBSERVABLE_FOREIGNS.contains(&symbol.as_str()) {
            self.effects += 1;
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
            });
        }
        let addr = self.foreign_addr(program, id)?;
        if addr & TAG_MASK == FOREIGN_TAG {
            return self.trap(format!(
                "foreign procedure '{symbol}' is not available here"
            ));
        }
        #[cfg(target_os = "macos")]
        native::main_thread::note_symbol(&symbol);
        #[cfg(target_os = "macos")]
        let result = if matches!(&*symbol, "fork" | "vfork") {
            native::main_thread::direct(|| self.call_native(program, addr, args, sig))?
        } else {
            self.call_native(program, addr, args, sig)?
        };
        #[cfg(not(target_os = "macos"))]
        let result = self.call_native(program, addr, args, sig)?;
        if &*symbol == "fork" && result.first() == Some(&0) {
            self.forked_child = true;
            #[cfg(target_os = "macos")]
            native::main_thread::disable();
        }
        Ok(result)
    }

    fn call_native(
        &mut self,
        program: &Program,
        addr: u64,
        args: &[u64],
        sig: &ir::Sig,
    ) -> Res<Vec<u64>> {
        // `#c_call` procedures handed to C become native thunks that call back in here.
        let mut argv = args.to_vec();
        for v in &mut argv {
            if *v & TAG_MASK != FUNC_TAG {
                continue;
            }
            let id = FuncId((*v & !TAG_MASK) as u32);
            let Some(func) = program.funcs.get(id.0 as usize).and_then(Option::as_ref) else {
                continue;
            };
            if func.sig.conv == ir::Conv::C {
                let identity = program as *const Program as u64;
                *v = native::callback_addr(identity, id, &func.sig).map_err(|m| Trap {
                    message: m,
                    loc: self.loc,
                })?;
            }
        }
        let me: *mut Interp = self;
        let mut reenter = |func: FuncId, args: &[u64]| {
            // SAFETY: this interpreter is suspended in the native call below; C calls back on
            // this thread before that call returns.
            let interp = unsafe { &mut *me };
            interp
                .exec(program, func, args)
                .map(Rets::into_vec)
                .map_err(|t| t.message)
        };
        native::call(addr, &argv, sig, &mut reenter).map_err(|m| Trap {
            message: m,
            loc: self.loc,
        })
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
        let result = self.exec(program, func, args).map(Rets::into_vec);
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
                "procedure '{name}' has no body available at this point"
            ));
        };
        if self.depth >= MAX_DEPTH {
            return self.trap("stack overflow (recursion too deep)");
        }
        let frame = self.frame(program, id);
        let base = self.sp;
        let stack_start = self.stack.as_mut_ptr() as u64;
        let start = (stack_start + base).next_multiple_of(frame.align) - stack_start;
        let traced = func.trace.is_some() && program.stack_trace_offset.is_some();
        let node_size = if traced {
            TRACE_NODE_SIZE
        } else {
            0
        };
        if start + frame.size + node_size > (self.stack.len() * 8) as u64 {
            return self.trap("interpreter stack overflow");
        }
        self.sp = start + frame.size + node_size;
        self.depth += 1;
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
        if let Some(counts) = self.profile.as_mut() {
            counts.add(
                id.0 as usize,
                &func.name,
                self.frame_blocks,
                self.frame_insts,
            );
        }
        (self.frame_blocks, self.frame_insts) = outer;
        if let Some((slot, previous)) = pushed {
            unsafe { std::ptr::write_unaligned(slot as *mut u64, previous) };
        }
        self.loc = saved_loc;
        self.trace_loc = saved_trace_loc;
        self.depth -= 1;
        self.sp = base;
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
        let offset = program.stack_trace_offset?;
        let info = func.trace.as_ref()?;
        let context = *args.first()?;
        if context == 0 {
            return None;
        }
        let slot = context + offset;
        let line = self
            .trace_loc
            .unwrap_or(self.loc)
            .map_or(0, |(_, line, _)| line);
        unsafe {
            let previous = std::ptr::read_unaligned(slot as *const u64);
            let (mut depth, mut hash) = (1u32, 0xcbf2_9ce4_8422_2325u64);
            if previous != 0 {
                std::ptr::write_unaligned((previous + 28) as *mut u32, line);
                depth = std::ptr::read_unaligned((previous + 24) as *const u32).wrapping_add(1);
                hash = std::ptr::read_unaligned((previous + 16) as *const u64);
            }
            hash = (hash ^ (id.0 as u64) ^ ((line as u64) << 32)).wrapping_mul(0x0100_0000_01b3);
            let info_addr = self.trace_info(program, id, info);
            std::ptr::write_unaligned(node as *mut u64, previous);
            std::ptr::write_unaligned((node + 8) as *mut u64, info_addr);
            std::ptr::write_unaligned((node + 16) as *mut u64, hash);
            std::ptr::write_unaligned((node + 24) as *mut u32, depth);
            std::ptr::write_unaligned((node + 28) as *mut u32, info.line);
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
        let words: [u64; 7] = [
            leak(&info.name),
            info.name.len() as u64,
            leak(path),
            path.len() as u64,
            info.line as u64,
            info.col as u64,
            FUNC_TAG | id.0 as u64,
        ];
        // `name` is {count, data}.
        let layout = [
            words[1], words[0], words[3], words[2], words[4], words[5], words[6],
        ];
        let addr = Box::leak(Box::new(layout)).as_ptr() as u64;
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
                let count = self.read_u64(s) as usize;
                let data = self.read_u64(s + 8);
                let bytes = self.read(data, count);
                self.effects += 1;
                self.host
                    .write(&bytes, args.get(1).is_some_and(|&v| v & 1 != 0));
            }
            Hook::WriteStrings => {
                let view = args[0];
                let count = self.read_u64(view) as usize;
                let data = self.read_u64(view + 8);
                let to_stderr = args.get(1).is_some_and(|&v| v & 1 != 0);
                self.effects += 1;
                for i in 0..count {
                    let s = data + i as u64 * 16;
                    let n = self.read_u64(s) as usize;
                    let p = self.read_u64(s + 8);
                    let bytes = self.read(p, n);
                    self.host.write(&bytes, to_stderr);
                }
            }
            Hook::DebugBreak => return self.trap("debug_break() was called"),
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
        let result = self.run_blocks(program, func, frame, stack_base, &mut vals);
        self.val_pool.push(vals);
        result
    }

    fn run_blocks(
        &mut self,
        program: &Program,
        func: &ir::Func,
        frame: &Frame,
        stack_base: u64,
        vals: &mut [u64],
    ) -> Res<Rets> {
        let mut block = 0usize;
        loop {
            if let Some(left) = self.block_budget.as_mut() {
                if *left == 0 {
                    return self.trap("execution budget exhausted");
                }
                *left -= 1;
            }
            if self.multi {
                if self.host.cooperative_threads() {
                    self.inline_preempt(program)?;
                } else {
                    #[cfg(not(target_arch = "wasm32"))]
                    self.preempt(program)?;
                }
            }
            let b = &func.blocks[block];
            self.frame_blocks += 1;
            self.frame_insts += b.insts.len() as u64;
            if let Some(counts) = self.profile.as_mut() {
                counts.block(b);
            }
            for inst in &b.insts {
                self.step(program, inst, vals, frame, stack_base)?;
            }
            match &b.term {
                Term::Jump(t) => block = t.0 as usize,
                Term::Branch {
                    cond,
                    then_block,
                    else_block,
                } => {
                    block = if vals[cond.0 as usize] & 0xff != 0 {
                        then_block.0
                    } else {
                        else_block.0
                    } as usize;
                }
                Term::Switch {
                    value,
                    ty,
                    cases,
                    default,
                } => {
                    let v = mask(*ty, vals[value.0 as usize]);
                    block = cases
                        .iter()
                        .find(|(c, _)| mask(*ty, *c) == v)
                        .map_or(default.0, |(_, t)| t.0) as usize;
                }
                Term::Ret(values) => {
                    return Ok(Rets::collect(values.iter().map(|v| vals[v.0 as usize])));
                }
                Term::Unreachable => {
                    return self.trap(format!("reached unreachable code in '{}'", func.name));
                }
            }
        }
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
            } => vals[dst.0 as usize] = FUNC_TAG | func.0 as u64,
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
                    return self.trap("copy through a null pointer");
                }
                unsafe { std::ptr::copy(s as *const u8, d as *mut u8, *size as usize) };
            }
            Inst::Zero {
                dst,
                size,
            } => {
                let d = vals[dst.0 as usize];
                if d < 4096 {
                    return self.trap("write through a null pointer");
                }
                unsafe { std::ptr::write_bytes(d as *mut u8, 0, *size as usize) };
            }
            Inst::Call {
                results,
                callee,
                args,
            } => {
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
                                return self.trap("call through a null procedure pointer");
                            }
                            _ => self.call_native(program, addr, argv, sig)?.into(),
                        }
                    }
                };
                for (r, &v) in results.iter().zip(out.iter()) {
                    vals[r.0 as usize] = v;
                }
            }
            Inst::Intrinsic {
                results,
                op,
                args,
            } => {
                let (mut small, mut heap) = ([0u64; 8], Vec::new());
                let argv = gather(vals, args, &mut small, &mut heap);
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
                    return self.trap("integer division by zero");
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
                    return self.trap("integer division by zero");
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

    fn intrinsic(&mut self, op: ir::Intrinsic, a: &[u64], _has_result: bool) -> Res<Vec<u64>> {
        use ir::Intrinsic as I;
        let f64_of = |x: u64| f64::from_bits(x);
        Ok(match op {
            I::Memcpy => {
                if a[2] > 0 {
                    unsafe { std::ptr::copy(a[1] as *const u8, a[0] as *mut u8, a[2] as usize) };
                }
                vec![]
            }
            I::Memset => {
                if a[2] > 0 {
                    unsafe { std::ptr::write_bytes(a[0] as *mut u8, a[1] as u8, a[2] as usize) };
                }
                vec![]
            }
            I::Memcmp => {
                let n = a[2] as usize;
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
                vec![success as u64, current]
            }
            I::DebugBreak => return self.trap("debug_break() was called"),
            I::Trap => return self.trap("runtime check failed"),
            I::BoundsCheck => {
                let (index, count) = (a[0] as i64, a[1] as i64);
                if index < 0 || index >= count {
                    return self.trap(format!(
                        "array bounds check failed: index {index} is outside an array of {count} element{}",
                        if count == 1 { "" } else { "s" }
                    ));
                }
                vec![]
            }
            I::CompilerWrite => {
                let bytes = self.read(a[0], a[1] as usize);
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
            I::Pause => vec![],
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
        })
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

fn mask(ty: Ty, v: u64) -> u64 {
    match ty {
        Ty::I8 => v & 0xff,
        Ty::I16 => v & 0xffff,
        Ty::I32 | Ty::F32 => v & 0xffff_ffff,
        _ => v,
    }
}

fn sext(ty: Ty, v: u64) -> i64 {
    match ty {
        Ty::I8 => v as u8 as i8 as i64,
        Ty::I16 => v as u16 as i16 as i64,
        Ty::I32 => v as u32 as i32 as i64,
        _ => v as i64,
    }
}

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

impl Drop for Interp {
    fn drop(&mut self) {
        self.flush_profile();
    }
}
