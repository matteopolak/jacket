//! Low-level IR shared by the interpreter and the LLVM backend.
//!
//! Values are scalars in virtual registers (`Val`), each defined exactly once.
//! Every aggregate lives in memory: locals are stack `Slot`s, and aggregate
//! parameters/results travel as pointers. Mutable locals are also slots, so no
//! phi nodes are needed (LLVM's mem2reg recovers SSA form).
//!
//! Jai calling convention (`Conv::Jai`): an optional leading context pointer,
//! then each parameter (aggregates by pointer to a caller-owned copy), then one
//! out-pointer per aggregate result. Scalar results are returned directly.
pub use crate::wide_float::{Arith as WideArith, WideFloat};
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct FuncId(pub u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct GlobalId(pub u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct ForeignId(pub u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Val(pub u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BlockId(pub u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SlotId(pub u32);

/// Scalar register classes. Booleans are `I8` holding 0 or 1.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Ty {
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Ptr,
    /// A C `long double` wider than `f64` (x87 extended, or binary128): 16 bytes of memory.
    /// Only aggregate layouts (`AggLayout::fields`) use these two; no `Val` has them, since
    /// such values live in memory and the `Wide` intrinsic works on their addresses.
    F80,
    F128,
}

impl Ty {
    pub fn size(self) -> u64 {
        match self {
            Ty::I8 => 1,
            Ty::I16 => 2,
            Ty::I32 | Ty::F32 => 4,
            Ty::I64 | Ty::F64 | Ty::Ptr => 8,
            Ty::F80 | Ty::F128 => 16,
        }
    }

    /// The memory-only class of a `long double` format.
    pub fn wide(fmt: WideFloat) -> Ty {
        match fmt {
            WideFloat::X87 => Ty::F80,
            WideFloat::Binary128 => Ty::F128,
        }
    }

    pub fn is_float(self) -> bool {
        matches!(self, Ty::F32 | Ty::F64)
    }

    pub fn int(bytes: u64) -> Ty {
        match bytes {
            1 => Ty::I8,
            2 => Ty::I16,
            4 => Ty::I32,
            _ => Ty::I64,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    SDiv,
    UDiv,
    SRem,
    URem,
    And,
    Or,
    Xor,
    Shl,
    LShr,
    AShr,
    Rotl,
    Rotr,
    FAdd,
    FSub,
    FMul,
    FDiv,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CmpOp {
    Eq,
    Ne,
    SLt,
    SLe,
    SGt,
    SGe,
    ULt,
    ULe,
    UGt,
    UGe,
    FEq,
    FNe,
    FLt,
    FLe,
    FGt,
    FGe,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConvOp {
    Trunc,
    ZExt,
    SExt,
    FToS,
    FToU,
    SToF,
    UToF,
    FExt,
    FTrunc,
    /// Same-size reinterpretation (int <-> float, ptr <-> int).
    Bitcast,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnOp {
    Neg,
    Not,
    FNeg,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Conv {
    Jai,
    C,
}

/// `Intrinsic::Trap` reason: a procedure with results fell off the end of its body.
pub const TRAP_MISSING_RETURN: u64 = 1;

/// `Intrinsic::Trap` reason: an `#asm` divide faulted (`#DE`: divisor zero or quotient too big).
pub const TRAP_ASM_DIVIDE: u64 = 2;

/// Check failure reason: an array index out of range (a: the index, b: the count).
pub const TRAP_BOUNDS: u64 = 3;

/// Check failure reason: an integer cast whose value does not fit the target (a: the value's
/// bits, b: `cast_check_code`).
pub const TRAP_CAST_OVERFLOW: u64 = 4;

/// Check failure reason: a `#complete` switch without a default matched no case (a: the value).
pub const TRAP_SWITCH_UNMATCHED: u64 = 5;

/// Check failure reason: an integer division or remainder by zero.
pub const TRAP_DIVIDE_BY_ZERO: u64 = 6;

/// `b` of a `TRAP_CAST_OVERFLOW` failure: the target's size in bytes (low byte), whether the
/// target is signed (bit 8) and whether the value is (bit 9).
pub fn cast_check_code(target_bytes: u64, target_signed: bool, value_signed: bool) -> u64 {
    target_bytes | u64::from(target_signed) << 8 | u64::from(value_signed) << 9
}

/// The integer type a `cast_check_code` names (`u8`, `s32`, ...).
pub fn cast_check_target(code: u64) -> String {
    let signed = code & 0x100 != 0;
    format!(
        "{}{}",
        if signed {
            's'
        } else {
            'u'
        },
        (code & 0xff) * 8
    )
}

/// What a failed runtime check says, the same in the interpreter and in native code
/// (`stdlib/Runtime_Support.jai`, `runtime_support_check_failed`).
pub fn check_message(reason: u64, a: u64, b: u64) -> String {
    match reason {
        TRAP_MISSING_RETURN => "reached the end of a procedure that must return a value".into(),
        TRAP_ASM_DIVIDE => {
            "#asm division fault: the divisor is zero or the quotient does not fit".into()
        }
        TRAP_BOUNDS => {
            let (index, count) = (a as i64, b as i64);
            format!(
                "array bounds check failed: index {index} is outside an array of {count} element{}",
                if count == 1 {
                    ""
                } else {
                    "s"
                }
            )
        }
        TRAP_CAST_OVERFLOW => {
            let value = if b & 0x200 != 0 {
                (a as i64).to_string()
            } else {
                a.to_string()
            };
            format!("cast of {value} to `{}` overflows", cast_check_target(b))
        }
        TRAP_DIVIDE_BY_ZERO => "integer division by zero".into(),
        TRAP_SWITCH_UNMATCHED => {
            format!(
                "no case of the `#complete` switch matches its value, {}",
                a as i64
            )
        }
        _ => "runtime check failed".into(),
    }
}

/// Built-in operations with backend-specific implementations.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Intrinsic {
    Memcpy,
    Memset,
    Memcmp,
    /// (ptr, old, new) -> (success, old_value); width from operand type.
    CompareAndSwap,
    DebugBreak,
    /// Abort with a runtime error. An optional `i64` operand says why (`TRAP_*`), for the
    /// message (native code reports it through `Program::check_failed` when it has one).
    Trap,
    /// (index: s64, count: s64): trap unless `0 <= index < count`.
    BoundsCheck,
    /// (reason: `TRAP_*`, a: s64, b: s64, fatal: I8): a runtime check failed (`check_message`).
    /// Fatal stops the program; otherwise it is reported as a warning and the program goes on.
    /// Native code reports through `Program::check_failed` at the current `Inst::Loc`.
    CheckFailed,
    /// (ptr: *u8, count: s64, to_stderr: bool): compile-time `write_string`.
    CompilerWrite,
    Sqrt,
    Sin,
    Cos,
    Floor,
    Ceil,
    Round,
    Trunc,
    Fabs,
    /// `(a, b, c) -> a * b + c` with a single rounding (`F64`; `#asm` FMA instructions).
    Fma,
    /// Return address / frame queries used by stack traces.
    ReturnAddress,
    /// Read the cycle counter (`rdtsc`).
    CycleCounter,
    Pause,
    /// Bit counting used by `#asm` lowering: (x, bits) -> count, same width as `x`.
    /// `Ctlz`/`Cttz` return `bits` for a zero input.
    Popcount,
    Ctlz,
    Cttz,
    /// (x, bits) -> x with its bytes reversed.
    Bswap,
    /// 1 when executing at compile time (`#compile_time`).
    IsCompileTime,
    /// `(a, b, width_bytes) -> bool`: 1 when the signed (`S`) or unsigned (`U`) operation does not
    /// fit in `width_bytes` bytes. Used by arithmetic overflow checks.
    SAddOverflow,
    UAddOverflow,
    SSubOverflow,
    USubOverflow,
    SMulOverflow,
    UMulOverflow,
    /// `long double` operations of a wide format (`Jaic_Extensions.Long_Double`), on values in
    /// memory; see `WideOp` for the operands.
    Wide(WideOp, WideFloat),
}

/// Operations on wide (`long double`) floats. `dst`, `a` and `b` are addresses of 16-byte
/// values; scalar operands and results are ordinary registers.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WideOp {
    /// (dst, a, b)
    Arith(WideArith),
    /// (dst, a)
    Neg,
    /// (a, b) -> I8; one of the `F*` comparisons.
    Cmp(CmpOp),
    /// (dst, F64 x)
    FromF64,
    /// (dst, F32 x)
    FromF32,
    /// (dst, I64 x)
    FromS64,
    FromU64,
    /// (a) -> F64
    ToF64,
    /// (a) -> F32
    ToF32,
    /// (a) -> I64, truncating toward zero
    ToS64,
    ToU64,
}

#[derive(Clone, Debug)]
pub enum Callee {
    Func(FuncId),
    Foreign(ForeignId),
    /// Indirect call through a procedure pointer with the given signature.
    Indirect(Val, Box<Sig>),
}

/// A lowered call signature (scalar parameter/result classes).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Sig {
    pub params: Vec<Ty>,
    pub returns: Vec<Ty>,
    pub conv: Conv,
    /// C variadic. A call site appends its variadic arguments' classes to `params`; the first
    /// `c_fixed` are the declared parameters (Apple arm64 passes the rest on the stack).
    pub c_varargs: bool,
    pub c_fixed: u32,
    /// Per-parameter C ABI aggregate descriptions (for by-value structs in C calls).
    pub c_abi: Option<Box<CAbi>>,
}

/// C ABI facts for aggregates passed by value to/from foreign code. Index i
/// describes IR parameter i: `Some(layout)` means the IR passes a pointer to
/// an aggregate that the C ABI passes by value.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct CAbi {
    pub params: Vec<Option<AggLayout>>,
    /// The single C return value is an aggregate returned through the last IR param (out-pointer).
    pub ret: Option<AggLayout>,
    /// The aggregate result is returned through a hidden result pointer whatever its
    /// size (`#cpp_return_type_is_non_pod`).
    pub ret_indirect: bool,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AggLayout {
    pub size: u64,
    pub align: u64,
    /// Flattened scalar fields: (offset, class).
    pub fields: Vec<(u64, Ty)>,
}

#[derive(Clone, Debug)]
pub struct CallInst {
    pub results: Vec<Val>,
    pub callee: Callee,
    pub args: Vec<Val>,
}

#[derive(Clone, Debug)]
pub struct IntrinsicInst {
    pub results: Vec<Val>,
    pub op: Intrinsic,
    pub args: Vec<Val>,
}

#[derive(Clone, Debug)]
pub enum Inst {
    IConst {
        dst: Val,
        ty: Ty,
        value: u64,
    },
    FConst {
        dst: Val,
        ty: Ty,
        value: f64,
    },
    Bin {
        dst: Val,
        op: BinOp,
        ty: Ty,
        a: Val,
        b: Val,
    },
    Un {
        dst: Val,
        op: UnOp,
        ty: Ty,
        a: Val,
    },
    /// Result is `I8` 0/1.
    Cmp {
        dst: Val,
        op: CmpOp,
        ty: Ty,
        a: Val,
        b: Val,
    },
    Conv {
        dst: Val,
        op: ConvOp,
        from: Ty,
        to: Ty,
        src: Val,
    },
    SlotAddr {
        dst: Val,
        slot: SlotId,
    },
    GlobalAddr {
        dst: Val,
        global: GlobalId,
    },
    FuncAddr {
        dst: Val,
        func: FuncId,
    },
    ForeignAddr {
        dst: Val,
        foreign: ForeignId,
    },
    Load {
        dst: Val,
        ty: Ty,
        addr: Val,
    },
    Store {
        ty: Ty,
        addr: Val,
        value: Val,
    },
    /// `dst = base + offset` (bytes).
    PtrAdd {
        dst: Val,
        base: Val,
        offset: Val,
    },
    /// Copy `size` bytes (non-overlapping).
    Copy {
        dst: Val,
        src: Val,
        size: u64,
    },
    /// Zero `size` bytes.
    Zero {
        dst: Val,
        size: u64,
    },
    /// Boxed, like `Intrinsic`, so the common instructions stay small (an `Inst` is 24
    /// bytes instead of 120), which matters for IR memory and for the interpreter's cache use.
    Call(Box<CallInst>),
    Intrinsic(Box<IntrinsicInst>),
    /// Source line marker for debug info and runtime error locations. `scope` indexes
    /// `FuncDebug::scopes` (0: the procedure itself, and always 0 without debug info).
    Loc {
        line: u32,
        col: u32,
        file: u32,
        scope: u32,
    },
}

// Keep instructions small: every function body is a long array of them.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<Inst>() <= 24);

#[derive(Clone, Debug)]
pub enum Term {
    Jump(BlockId),
    Branch {
        cond: Val,
        then_block: BlockId,
        else_block: BlockId,
    },
    /// Integer switch: `cases` are (value, target).
    Switch {
        value: Val,
        ty: Ty,
        cases: Vec<(u64, BlockId)>,
        default: BlockId,
    },
    Ret(Vec<Val>),
    Unreachable,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub insts: Vec<Inst>,
    pub term: Term,
}

#[derive(Clone, Copy, Debug)]
pub struct Slot {
    pub size: u64,
    pub align: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Linkage {
    Internal,
    /// `#program_export "name"`
    Export(String),
}

#[derive(Clone, Debug)]
pub struct Func {
    pub name: String,
    /// What the function was made for: a procedure, or compile-time code the compiler wraps
    /// in one.
    pub origin: FuncOrigin,
    pub sig: Sig,
    pub linkage: Linkage,
    pub slots: Vec<Slot>,
    pub blocks: Vec<Block>,
    /// Register class of each `Val`.
    pub vals: Vec<Ty>,
    /// The first `sig.params.len()` vals are the parameters, in order.
    pub source_file: u32,
    /// Present for procedures that take a context: what a stack trace node says about it.
    pub trace: Option<TraceInfo>,
    /// Native debug information (named variables, lexical scopes), recorded only when the
    /// program is built with debug info. Boxed so the interpreter's hot data stays small.
    pub debug: Option<Box<FuncDebug>>,
}

/// What a function was made for. Compile-time code outside a procedure runs in a function of
/// its own, which reports name by what it is rather than by its `name`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FuncOrigin {
    #[default]
    Procedure,
    /// A `#run` directive's code.
    Run,
    /// A constant's initializer evaluated at compile time.
    ConstInit,
}

impl FuncOrigin {
    /// The function name for compile-time code (IR dumps, stack trace nodes).
    pub fn thunk_name(self) -> &'static str {
        match self {
            FuncOrigin::Procedure => "",
            FuncOrigin::Run => "#run",
            FuncOrigin::ConstInit => "#const",
        }
    }
}

/// Debug information of one procedure, for the native backend (`docs/native/debug-info.md`).
#[derive(Clone, Debug, Default)]
pub struct FuncDebug {
    /// Procedure name as written (polymorph instances share it).
    pub name: String,
    pub file: u32,
    pub line: u32,
    /// Lexical scopes. Entry 0 is the procedure body itself; `Inst::Loc::scope` and
    /// `DebugVar::scope` index this.
    pub scopes: Vec<DebugScope>,
    pub vars: Vec<DebugVar>,
}

#[derive(Clone, Copy, Debug)]
pub struct DebugScope {
    /// Enclosing scope (entry 0 is its own parent).
    pub parent: u32,
    pub line: u32,
    pub col: u32,
}

/// A named local variable or parameter.
#[derive(Clone, Debug)]
pub struct DebugVar {
    pub name: String,
    /// Key into `Program::debug_types` (a `TypeId`).
    pub ty: u32,
    /// The variable's address: usually a `SlotAddr` result, or a parameter (aggregates
    /// arrive by pointer).
    pub addr: Val,
    /// 1-based parameter position; 0 for a local.
    pub arg: u32,
    pub scope: u32,
    pub line: u32,
    pub col: u32,
}

/// A global variable with debug information.
#[derive(Clone, Debug)]
pub struct DebugGlobal {
    pub global: GlobalId,
    pub name: String,
    pub ty: u32,
    pub file: u32,
    pub line: u32,
}

/// Debug description of a Jai type, keyed by `TypeId` in `Program::debug_types`
/// (plus the synthetic keys [`DEBUG_CHAR`] and [`DEBUG_CHAR_PTR`]).
#[derive(Clone, Debug)]
pub struct DebugType {
    pub name: String,
    pub size: u64,
    pub align: u64,
    pub kind: DebugTypeKind,
}

/// `u8` shown as a character: the pointee of `string.data`, so debuggers print text.
pub const DEBUG_CHAR: u32 = u32::MAX - 1;

pub const DEBUG_CHAR_PTR: u32 = u32::MAX - 2;

#[derive(Clone, Debug)]
pub enum DebugTypeKind {
    Void,
    Bool,
    Int {
        signed: bool,
    },
    Char,
    Float,
    /// Pointee key (a `Void` pointee is `*void`).
    Pointer(u32),
    /// Structs, unions and the built-in aggregates (`string`, `[] T`, `[..] T`, `Any`).
    Struct {
        fields: Vec<DebugField>,
        union: bool,
    },
    Array {
        elem: u32,
        count: u64,
    },
    Enum {
        base: u32,
        members: Vec<(String, i64)>,
    },
    /// Another name for a type (`#type,distinct`, `Type`, procedure types).
    Typedef(u32),
}

#[derive(Clone, Debug)]
pub struct DebugField {
    pub name: String,
    pub ty: u32,
    pub offset: u64,
}

/// Name and declaration site of a procedure, for `context.stack_trace` nodes.
#[derive(Clone, Debug)]
pub struct TraceInfo {
    /// Empty for an anonymous procedure.
    pub name: String,
    pub file: u32,
    pub line: u32,
    pub col: u32,
}

impl Func {
    pub fn param(&self, i: usize) -> Val {
        Val(i as u32)
    }
}

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
#[derive(Clone, Debug, PartialEq)]
pub struct Reloc {
    pub offset: u64,
    pub target: RelocTarget,
    pub addend: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Default)]
pub struct Program {
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
    /// Runtime_Support's `runtime_support_check_failed(reason, a, b, fatal, line, filename)`,
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

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

pub struct Builder {
    pub func: Func,
    pub current: BlockId,
    terminated: crate::fxhash::HashSet<BlockId>,
    /// The last `loc` marker, for `repeat_loc`.
    last_loc: Option<(u32, u32, u32, u32)>,
}

impl Builder {
    pub fn new(name: String, sig: Sig) -> Self {
        let vals = sig.params.clone();
        let func = Func {
            name,
            origin: FuncOrigin::Procedure,
            sig,
            linkage: Linkage::Internal,
            slots: Vec::new(),
            blocks: vec![Block {
                insts: Vec::new(),
                term: Term::Unreachable,
            }],
            vals,
            source_file: 0,
            trace: None,
            debug: None,
        };
        Self {
            func,
            current: BlockId(0),
            terminated: Default::default(),
            last_loc: None,
        }
    }

    pub fn param(&self, i: usize) -> Val {
        Val(i as u32)
    }

    pub fn new_val(&mut self, ty: Ty) -> Val {
        self.func.vals.push(ty);
        Val(self.func.vals.len() as u32 - 1)
    }

    pub fn val_ty(&self, v: Val) -> Ty {
        self.func.vals[v.0 as usize]
    }

    pub fn new_block(&mut self) -> BlockId {
        self.func.blocks.push(Block {
            insts: Vec::new(),
            term: Term::Unreachable,
        });
        BlockId(self.func.blocks.len() as u32 - 1)
    }

    pub fn switch_to(&mut self, block: BlockId) {
        self.current = block;
    }

    /// True once the current block has been terminated by `terminate`.
    pub fn is_terminated(&self) -> bool {
        self.terminated.contains(&self.current)
    }

    pub fn slot(&mut self, size: u64, align: u64) -> SlotId {
        self.func.slots.push(Slot {
            size,
            align: align.max(1),
        });
        SlotId(self.func.slots.len() as u32 - 1)
    }

    pub fn push(&mut self, inst: Inst) {
        if self.is_terminated() {
            // Code after return/break is unreachable; emit into a fresh dead block.
            let dead = self.new_block();
            self.current = dead;
        }
        self.func.blocks[self.current.0 as usize].insts.push(inst);
    }

    pub fn terminate(&mut self, term: Term) {
        if self.is_terminated() {
            return;
        }
        self.func.blocks[self.current.0 as usize].term = term;
        self.terminated.insert(self.current);
    }

    pub fn iconst(&mut self, ty: Ty, value: u64) -> Val {
        let dst = self.new_val(ty);
        self.push(Inst::IConst {
            dst,
            ty,
            value,
        });
        dst
    }

    pub fn fconst(&mut self, ty: Ty, value: f64) -> Val {
        let dst = self.new_val(ty);
        self.push(Inst::FConst {
            dst,
            ty,
            value,
        });
        dst
    }

    pub fn bin(&mut self, op: BinOp, ty: Ty, a: Val, b: Val) -> Val {
        let dst = self.new_val(ty);
        self.push(Inst::Bin {
            dst,
            op,
            ty,
            a,
            b,
        });
        dst
    }

    pub fn un(&mut self, op: UnOp, ty: Ty, a: Val) -> Val {
        let dst = self.new_val(ty);
        self.push(Inst::Un {
            dst,
            op,
            ty,
            a,
        });
        dst
    }

    pub fn cmp(&mut self, op: CmpOp, ty: Ty, a: Val, b: Val) -> Val {
        let dst = self.new_val(Ty::I8);
        self.push(Inst::Cmp {
            dst,
            op,
            ty,
            a,
            b,
        });
        dst
    }

    pub fn conv(&mut self, op: ConvOp, from: Ty, to: Ty, src: Val) -> Val {
        if from == to && matches!(op, ConvOp::Bitcast) {
            return src;
        }
        let dst = self.new_val(to);
        self.push(Inst::Conv {
            dst,
            op,
            from,
            to,
            src,
        });
        dst
    }

    pub fn slot_addr(&mut self, slot: SlotId) -> Val {
        let dst = self.new_val(Ty::Ptr);
        self.push(Inst::SlotAddr {
            dst,
            slot,
        });
        dst
    }

    /// Allocate a fresh stack slot and return its address.
    pub fn alloca(&mut self, size: u64, align: u64) -> Val {
        let slot = self.slot(size, align);
        self.slot_addr(slot)
    }

    /// Address of a variable's storage: program data or a foreign symbol.
    pub fn storage_addr(&mut self, storage: Storage) -> Val {
        match storage {
            Storage::Data(global) => self.global_addr(global),
            Storage::Foreign(foreign) => self.foreign_addr(foreign),
        }
    }

    pub fn global_addr(&mut self, global: GlobalId) -> Val {
        let dst = self.new_val(Ty::Ptr);
        self.push(Inst::GlobalAddr {
            dst,
            global,
        });
        dst
    }

    pub fn func_addr(&mut self, func: FuncId) -> Val {
        let dst = self.new_val(Ty::Ptr);
        self.push(Inst::FuncAddr {
            dst,
            func,
        });
        dst
    }

    pub fn foreign_addr(&mut self, foreign: ForeignId) -> Val {
        let dst = self.new_val(Ty::Ptr);
        self.push(Inst::ForeignAddr {
            dst,
            foreign,
        });
        dst
    }

    pub fn load(&mut self, ty: Ty, addr: Val) -> Val {
        let dst = self.new_val(ty);
        self.push(Inst::Load {
            dst,
            ty,
            addr,
        });
        dst
    }

    pub fn store(&mut self, ty: Ty, addr: Val, value: Val) {
        self.push(Inst::Store {
            ty,
            addr,
            value,
        });
    }

    pub fn ptr_add(&mut self, base: Val, offset: Val) -> Val {
        let dst = self.new_val(Ty::Ptr);
        self.push(Inst::PtrAdd {
            dst,
            base,
            offset,
        });
        dst
    }

    pub fn ptr_offset(&mut self, base: Val, offset: u64) -> Val {
        if offset == 0 {
            return base;
        }
        let off = self.iconst(Ty::I64, offset);
        self.ptr_add(base, off)
    }

    pub fn copy(&mut self, dst: Val, src: Val, size: u64) {
        if size > 0 {
            self.push(Inst::Copy {
                dst,
                src,
                size,
            });
        }
    }

    pub fn zero(&mut self, dst: Val, size: u64) {
        if size > 0 {
            self.push(Inst::Zero {
                dst,
                size,
            });
        }
    }

    pub fn call(&mut self, callee: Callee, args: Vec<Val>, returns: &[Ty]) -> Vec<Val> {
        let results: Vec<Val> = returns.iter().map(|&t| self.new_val(t)).collect();
        self.push(Inst::Call(Box::new(CallInst {
            results: results.clone(),
            callee,
            args,
        })));
        results
    }

    pub fn intrinsic(&mut self, op: Intrinsic, args: Vec<Val>, returns: &[Ty]) -> Vec<Val> {
        let results: Vec<Val> = returns.iter().map(|&t| self.new_val(t)).collect();
        self.push(Inst::Intrinsic(Box::new(IntrinsicInst {
            results: results.clone(),
            op,
            args,
        })));
        results
    }

    pub fn jump(&mut self, target: BlockId) {
        self.terminate(Term::Jump(target));
    }

    pub fn branch(&mut self, cond: Val, then_block: BlockId, else_block: BlockId) {
        self.terminate(Term::Branch {
            cond,
            then_block,
            else_block,
        });
    }

    pub fn ret(&mut self, values: Vec<Val>) {
        self.terminate(Term::Ret(values));
    }

    pub fn loc(&mut self, file: u32, line: u32, col: u32, scope: u32) {
        self.last_loc = Some((file, line, col, scope));
        if !self.is_terminated() {
            self.push(Inst::Loc {
                line,
                col,
                file,
                scope,
            });
        }
    }

    /// Repeat the last `loc` marker in the current block: a block placed out of line (a failed
    /// check's) still reports the statement that branched to it.
    pub fn repeat_loc(&mut self) {
        if let Some((file, line, col, scope)) = self.last_loc {
            self.loc(file, line, col, scope);
        }
    }

    pub fn finish(mut self) -> Func {
        // Blocks that were never terminated fall off the end: void return.
        for (i, block) in self.func.blocks.iter_mut().enumerate() {
            if !self.terminated.contains(&BlockId(i as u32)) && self.func.sig.returns.is_empty() {
                block.term = Term::Ret(Vec::new());
            }
        }
        self.func
    }
}

impl fmt::Display for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "func {} {:?} -> {:?}",
            self.name, self.sig.params, self.sig.returns
        )?;
        for (i, slot) in self.slots.iter().enumerate() {
            writeln!(f, "  slot{i}: {} align {}", slot.size, slot.align)?;
        }
        for (i, block) in self.blocks.iter().enumerate() {
            writeln!(f, " b{i}:")?;
            for inst in &block.insts {
                writeln!(f, "    {inst:?}")?;
            }
            writeln!(f, "    {:?}", block.term)?;
        }
        Ok(())
    }
}
