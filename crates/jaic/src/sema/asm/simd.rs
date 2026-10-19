//! The wider SSE..AVX-512 instruction set of `#asm`, on top of the core vector
//! instructions in `asm/vec.rs` (which owns operand decoding, the vector size,
//! EVEX broadcast and `{k}` merge/zero masking).
//!
//! Every instruction computes its result lane by lane into a scratch buffer and
//! then stores it, so sources may alias the destination. Operand shapes:
//! `dst, src...[, imm8]`; one source fewer than the instruction takes means the
//! destination is also the first source (the legacy SSE form). Shuffles and other
//! "in-lane" operations work per 128-bit block, as on hardware.
//!
//! Instructions that write a mask register (AVX-512 compares, `ptestm`,
//! `pmov*2m`) AND their result with a `{k}` write mask; `blendm*`, compress and
//! scatter consume the write mask themselves.
use super::vec::{VOpd, lane_addr, load_lane, store_lane};
use super::{
    AsmCtx, AsmInst, BinOp, CmpOp, Compiler, FnCtx, Intrinsic, Result, Span, Ty, UnOp, Val, bin,
    bit_intrinsic, cmp, err, flag_not, flag_or, is_zero, konst, ptr_of, resize, select,
};

mod args;
mod compare;
mod convert;
mod dispatch;
mod ext;
mod helpers;
mod lanes;
mod masks;
mod pcmpstr;
mod shuffle;
mod tables;
pub(super) use self::helpers::WriteMask;
use self::helpers::{load_dyn, mask_bit, when, xtime};
pub(super) use self::tables::{consumes_mask, dst_class, elem_size, lookup_simd, mmx_capable};
pub(in crate::sema) use ext::{GfOp, ShaOp};

/// Lane-wise binary operations beyond `vec::Lane`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::sema) enum B2 {
    AddSatS,
    AddSatU,
    SubSatS,
    SubSatU,
    /// Unsigned rounding average.
    Avg,
    MulHiS,
    MulHiU,
    /// `pmulhrsw`: rounded high half of the signed product, scaled.
    MulHrs,
    /// `psign*`: negate, zero or keep by the sign of the second operand.
    Sign,
    /// Per-lane variable shifts and rotates.
    Shlv,
    Shrv,
    Sarv,
    Rolv,
    Rorv,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::sema) enum U1 {
    Popcnt,
    Lzcnt,
    /// `rcpps`/`rcp14ps`...: computed exactly (hardware returns an approximation).
    Rcp,
    Rsqrt,
    Conflict,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::sema) enum FmaKind {
    Madd,
    Msub,
    Nmadd,
    Nmsub,
    /// Subtract in even lanes, add in odd lanes.
    MaddSub,
    /// Add in even lanes, subtract in odd lanes.
    MsubAdd,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::sema) enum Sat {
    Wrap,
    Signed,
    Unsigned,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::sema) enum DupKind {
    /// `movddup`: the even double of each pair.
    Low64,
    /// `movshdup`: the odd float of each pair.
    OddF32,
    /// `movsldup`: the even float of each pair.
    EvenF32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::sema) enum AesOp {
    Enc,
    EncLast,
    Dec,
    DecLast,
    Imc,
    KeygenAssist,
}

/// Conversions between lane types. `(from, to)` lane types; integer sources and
/// destinations carry a signedness.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::sema) enum Conv {
    /// Integer lanes to float lanes: (from, to, unsigned).
    IntToFloat(Ty, Ty, bool),
    /// Float lanes to integer lanes: (from, to, truncate, unsigned).
    FloatToInt(Ty, Ty, bool, bool),
    /// Float to float (`cvtps2pd`, `cvtpd2ps`).
    FloatToFloat(Ty, Ty),
    /// Scalar float to float, upper lanes from the first source (`cvtss2sd`).
    ScalarFloat(Ty, Ty),
    /// General-purpose integer to lane 0 (`cvtsi2ss.q`): (to, unsigned).
    GprToScalar(Ty, bool),
    /// Lane 0 to a general-purpose integer (`cvttsd2si.q`): (from, truncate, unsigned).
    ScalarToGpr(Ty, bool, bool),
}

pub(in crate::sema::asm) use super::vec::VOp;
