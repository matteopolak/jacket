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
mod analysis;
mod builder;
mod checks;
mod func;
mod insts;
mod program;

pub use self::analysis::{inst_vals, term_uses};
pub use self::builder::Builder;
pub use self::checks::{
    CheckReason, NULL_COPY, NULL_FILL, NULL_READ, NULL_WRITE, cast_check_code, cast_check_target,
    check_message,
};
pub use self::func::{
    Block, DEBUG_CHAR, DEBUG_CHAR_PTR, DebugField, DebugGlobal, DebugScope, DebugType,
    DebugTypeKind, DebugVar, Func, FuncDebug, FuncOrigin, InlineHint, Linkage, Slot, TraceInfo,
};
pub use self::insts::{
    AggLayout, AtomicOp, CAbi, CallInst, Callee, Inst, Intrinsic, IntrinsicInst, ORDER_ACQ_REL,
    ORDER_ACQUIRE, ORDER_RELAXED, ORDER_RELEASE, ORDER_SEQ_CST, Sig, Term, WideOp, X86Fence,
    X86Prefetch,
};
pub use self::program::{
    Foreign, Global, Library, Program, Reloc, RelocTarget, Storage, TraceInfoLayout, TraceLayout,
    TraceNodeLayout,
};
pub use crate::wide_float::{Arith as WideArith, WideFloat};

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
