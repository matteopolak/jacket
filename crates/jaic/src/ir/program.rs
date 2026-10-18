//! Globals, foreigns, libraries and the program that owns them.
use super::func::{DebugGlobal, DebugType, Func};
use super::insts::Sig;
use super::{ForeignId, FuncId, GlobalId};

/// Where a global variable lives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Storage {
    /// Program data (`x: T;`).
    Data(GlobalId),
    /// A symbol of a foreign library (`x: T #elsewhere lib;`).
    Foreign(ForeignId),
}

/// A relocation inside a global's initial bytes: an 8-byte pointer slot at
/// `offset` that must hold the address of `target` plus `addend`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Reloc {
    pub offset: u64,
    pub target: RelocTarget,
    pub addend: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RelocTarget {
    Global(GlobalId),
    Func(FuncId),
    Foreign(ForeignId),
}

#[derive(Clone, Debug)]
pub struct Global {
    pub name: String,
    pub size: u64,
    pub align: u64,
    /// Initial bytes (empty = zero-initialized).
    pub init: Vec<u8>,
    pub relocs: Vec<Reloc>,
    pub read_only: bool,
    /// `#program_export` name, if any.
    pub export: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Foreign {
    /// Symbol name in the library.
    pub symbol: String,
    /// Library handle index (`Program::libraries`); `None` for symbols found in the process.
    pub library: Option<usize>,
    pub sig: Sig,
    /// A foreign *variable* (`x: T #elsewhere lib`) rather than a function.
    pub is_data: bool,
}

#[derive(Clone, Debug)]
pub struct Library {
    /// Name as written (`"libc"`, `"SDL2"`, path...).
    pub name: String,
    pub system: bool,
    /// Linked even when no foreign procedure names it (`#library,link_always`).
    pub link_always: bool,
    /// Directory of the declaring source file (for relative library paths).
    pub base_dir: String,
}

/// `context.stack_trace` and the Preload types it links, as byte offsets read from their
/// declarations (`Compiler::enable_stack_traces`). The interpreter and the stack trace pass
/// (`stack_trace.rs`) write nodes with it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TraceLayout {
    /// `stack_trace` in the Context.
    pub context: u64,
    pub node: TraceNodeLayout,
    pub info: TraceInfoLayout,
}

/// `Stack_Trace_Node`: `next` and `info` are pointers, `hash` a `u64`, `call_depth` and
/// `line_number` `u32`s.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TraceNodeLayout {
    pub size: u64,
    pub next: u64,
    pub info: u64,
    pub hash: u64,
    pub call_depth: u64,
    pub line_number: u64,
}

/// `Stack_Trace_Procedure_Info`: `name` and the location's path are strings, its line and
/// column `s64`s, `procedure_address` a pointer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TraceInfoLayout {
    pub size: u64,
    pub name: u64,
    pub path: u64,
    pub line: u64,
    pub column: u64,
    pub procedure_address: u64,
}

/// All lowered code. Functions are lowered on demand, so slots may be empty.
#[derive(Clone, Default)]
pub struct Program {
    /// Target OS/CPU differ from the compiler host. Target link requirements
    /// must not initialize during an unrelated compile-time host foreign call.
    pub cross_target: bool,
    pub funcs: Vec<Option<Func>>,
    pub func_names: Vec<String>,
    /// Each function's signature as soon as it is reserved for a procedure, before its body
    /// is lowered (`reserve_func_with_sig`); `None` for functions reserved without one.
    pub reserved_sigs: Vec<Option<Sig>>,
    pub globals: Vec<Global>,
    pub foreigns: Vec<Foreign>,
    pub libraries: Vec<Library>,
    /// Variables whose compile-time state is discarded when the program runs
    /// (every user global except those declared `#no_reset`).
    pub reset_globals: Vec<GlobalId>,
    /// Path of each source file by `FileId`, for stack trace nodes.
    pub file_paths: Vec<String>,
    /// Where `context.stack_trace` is and how its nodes are laid out (`None`: stack traces
    /// are off).
    pub stack_trace: Option<TraceLayout>,
    /// Types named by debug information (`FuncDebug`, `debug_globals`), by key.
    pub debug_types: crate::fxhash::HashMap<u32, DebugType>,
    pub debug_globals: Vec<DebugGlobal>,
    /// `Runtime_Support`'s `__jaic_check_failed(reason, a, b, fatal, line, filename)`,
    /// which native code calls when a check fails (bounds, casts, `#complete` switches, a
    /// missing return) to say which one and where, before it traps.
    pub check_failed: Option<FuncId>,
}

impl Program {
    pub fn reserve_func(&mut self, name: String) -> FuncId {
        self.funcs.push(None);
        self.func_names.push(name);
        self.reserved_sigs.push(None);
        FuncId(self.funcs.len() as u32 - 1)
    }

    /// `reserve_func` for a function whose signature is already known.
    pub fn reserve_func_with_sig(&mut self, name: String, sig: Sig) -> FuncId {
        let id = self.reserve_func(name);
        self.reserved_sigs[id.0 as usize] = Some(sig);
        id
    }

    /// Function `id`'s signature: its lowered body's, or the one it was reserved with.
    pub fn func_sig(&self, id: FuncId) -> Option<&Sig> {
        let i = id.0 as usize;
        match self.funcs.get(i) {
            Some(Some(f)) => Some(&f.sig),
            _ => self.reserved_sigs.get(i)?.as_ref(),
        }
    }

    pub fn func(&self, id: FuncId) -> Option<&Func> {
        self.funcs[id.0 as usize].as_ref()
    }

    pub fn add_global(&mut self, global: Global) -> GlobalId {
        self.globals.push(global);
        GlobalId(self.globals.len() as u32 - 1)
    }

    pub fn add_foreign(&mut self, foreign: Foreign) -> ForeignId {
        self.foreigns.push(foreign);
        ForeignId(self.foreigns.len() as u32 - 1)
    }
}
