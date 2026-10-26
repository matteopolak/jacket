//! Instructions, terminators, intrinsics and callee signatures.
use super::{
    BinOp, BlockId, CmpOp, Conv, ConvOp, ForeignId, FuncId, GlobalId, SlotId, Ty, UnOp, Val,
};
use crate::wide_float::{Arith as WideArith, WideFloat};

/// Memory orders of the atomic intrinsics (the `Memory_Order` enum of `stdlib/Atomics.jai`).
pub const ORDER_RELAXED: u8 = 0;

pub const ORDER_ACQUIRE: u8 = 1;
pub const ORDER_RELEASE: u8 = 2;
pub const ORDER_ACQ_REL: u8 = 3;
pub const ORDER_SEQ_CST: u8 = 4;

/// The operation of an `Intrinsic::AtomicRmw`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AtomicOp {
    Xchg,
    Add,
    Sub,
    And,
    Or,
    Xor,
}

/// Built-in operations with backend-specific implementations.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum X86Fence {
    Load,
    Store,
    Full,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum X86Prefetch {
    T0,
    T1,
    T2,
    NonTemporal,
}

/// Built-in operations with backend-specific implementations.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Intrinsic {
    Memcpy,
    Memset,
    Memcmp,
    /// (ptr, old, new) -> (success, `old_value`); width from operand type.
    CompareAndSwap,
    /// `(ptr, width) -> value`: an atomic load with the memory order of the payload (`ORDER_*`).
    /// `width` is the operand size in bytes (1, 2, 4 or 8), as for `CompareAndSwap`.
    AtomicLoad(u8),
    /// `(ptr, value, width)`: an atomic store with the memory order of the payload.
    AtomicStore(u8),
    /// `(ptr, value, width) -> previous value`: an atomic read-modify-write, `*ptr = op(*ptr, value)`.
    AtomicRmw(AtomicOp, u8),
    /// `(ptr, old, new, width) -> (success, previous)`: a compare-exchange with the success and
    /// failure memory orders of the payload.
    AtomicCas(u8, u8),
    DebugBreak,
    /// Abort with a runtime error. An optional `i64` operand says why (`CheckReason`), for the
    /// message (native code reports it through `Program::check_failed` when it has one).
    Trap,
    /// (index: s64, count: s64): trap unless `0 <= index < count`.
    BoundsCheck,
    /// (reason: `CheckReason`, a: s64, b: s64, fatal: I8): a runtime check failed (`check_message`).
    /// Fatal stops the program; otherwise it is reported as a warning and the program goes on.
    /// Native code reports through `Program::check_failed` at the current `Inst::Loc`.
    CheckFailed,
    /// (ptr: *u8, count: s64, `to_stderr`: bool): compile-time `write_string`.
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
    /// Real x86 RDTSC (I64), or RDTSCP (I64 timestamp, I32 `IA32_TSC_AUX`).
    X86Timestamp {
        aux: bool,
    },
    /// One hardware attempt: (value: width, carry: I8). Width is I16/I32/I64.
    X86Random {
        seed: bool,
        width: Ty,
    },
    X86Fence(X86Fence),
    X86Pause,
    /// One memory-address argument; a cache hint, not a data load or memory fence.
    X86Prefetch(X86Prefetch),
    /// `(leaf: I32, subleaf: I32) -> (eax, ebx, ecx, edx): I32`, real x86-64 CPUID.
    CpuId,
    /// `(index: I32) -> (edx, eax): I32`, real XGETBV; preserves flags.
    /// Interpreter execution validates XSAVE/OSXSAVE and optional index-1 support.
    X86Xgetbv,
    Pause,
    /// `(value) -> value`: the value, hidden from the optimizer (`black_box`). Native code is an
    /// empty inline assembly that takes the value in a register and gives it back, so nothing
    /// folds through it and the work behind it stays; no instruction is emitted.
    BlackBox,
    /// `(ptr)`: the memory at the address is read and written by code the optimizer cannot see
    /// (`black_box` of an aggregate, which the caller copied to a fresh stack slot first).
    BlackBoxMemory,
    /// `(number, a1..a6) -> raw result`, all `I64`: a Linux x86-64 system call (`#asm` `syscall`).
    /// The result is the kernel's raw return value, `-errno` on failure. Native code issues the
    /// `syscall` instruction; the interpreter calls the host's `syscall` on Linux x86-64 and the
    /// sandbox refuses it with `-EPERM`.
    Syscall,
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
    /// `long double` operations of a wide format (`Long_Double` from `Extensions/Long_Double`), on values in
    /// memory; see `WideOp` for the operands.
    Wide(WideOp, WideFloat),
    /// `(dst, table, indices)`, all addresses of bytes; the payload is the width in bytes (16, 32
    /// or 64). Per 16-byte block of `indices`: `dst[i] = idx[i] < 16 ? table[idx[i]] : 0`, for
    /// every index 0..255, whatever the target.
    ByteLookup(u8),
    /// `(src) -> I64`; the payload is the width in bytes (16, 32 or 64). Bit i of the result is the
    /// top bit of byte i (`pmovmskb`).
    SignMask(u8),
    /// `(fiber, stack: *u8, size: s64, entry, data)`: prepare the `Fiber_Context` at `fiber` so
    /// that the first `FiberSwitch` to it calls the `#c_call` procedure `entry(data)` on the
    /// `size` bytes of memory at `stack`. `entry` must never return.
    FiberInit,
    /// `(from, to)`: save the running fiber's registers and stack pointer in the `Fiber_Context`
    /// at `from`, and continue the one at `to`. Returns when something switches back to `from`.
    FiberSwitch,
    /// `(fiber)`: the context at `fiber` will not be used again; release what the backend keeps
    /// for it (nothing in native code).
    FiberFree,
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

impl Sig {
    /// The index of the out-pointer of a memory-class result: the last declared parameter, so
    /// before any variadic argument a call site appended.
    pub fn out_index(&self) -> usize {
        let end = if self.c_varargs {
            self.c_fixed as usize
        } else {
            self.params.len()
        };
        end.wrapping_sub(1)
    }
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
    /// Allocate runtime-sized, aligned storage until the current procedure returns.
    StackAlloc {
        dst: Val,
        size: Val,
        align: u64,
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

impl Term {
    /// The blocks this ending can go to, a target once per edge.
    pub fn successors(&self) -> Vec<BlockId> {
        match self {
            Term::Jump(b) => vec![*b],
            Term::Branch {
                then_block,
                else_block,
                ..
            } => vec![*then_block, *else_block],
            Term::Switch {
                cases,
                default,
                ..
            } => cases
                .iter()
                .map(|c| c.1)
                .chain(std::iter::once(*default))
                .collect(),
            Term::Ret(_) | Term::Unreachable => Vec::new(),
        }
    }
}
