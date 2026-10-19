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
pub(super) use self::tables::{
    broadcast_size, consumes_mask, dst_class, elem_size, lookup_simd, mmx_capable,
};
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

/// Instruction semantics (see `lookup_simd`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::sema) enum SOp {
    Bin(B2, Ty),
    /// `prold`/`prorq` by an immediate: (left, lane).
    RotImm(bool, Ty),
    Unary(U1, Ty),
    /// `roundps`/`rndscaleps` (lane, scalar).
    Round(Ty, bool),
    /// (kind, operand order 132/213/231, lane, scalar).
    Fma(FmaKind, u16, Ty, bool),
    /// Pairwise horizontal add/sub (`phaddw`, `haddps`): (subtract, saturate, lane).
    Horizontal(bool, bool, Ty),
    AddSub(Ty),
    Pmaddwd,
    Pmaddubsw,
    Psadbw,
    /// `mpsadbw`: sums of absolute differences of 4-byte groups, picked by the immediate.
    Mpsadbw,
    /// `pmadd52luq` (false) / `pmadd52huq`: 52-bit multiply, low or high half added to qwords.
    Madd52(bool),
    /// `pmuludq` / `pmuldq` (signed).
    MulEven(bool),
    /// `pdpbusd` (bytes) / `pdpwssd` (words): dot products accumulated into dword lanes.
    DotAcc(Ty),
    Phminposuw,
    /// `punpckl*`/`punpckh*`/`unpck*`: (high, lane).
    Unpack(bool, Ty),
    /// `pshufhw` (true) / `pshuflw`.
    ShufHalf(bool),
    Shufpd,
    Palignr,
    /// `alignd`/`alignq`: element-granular `palignr` across the whole vector.
    Valign(Ty),
    /// `pslldq` (true) / `psrldq`.
    ByteShift(bool),
    BlendImm(Ty),
    BlendVar(Ty),
    BlendMask(Ty),
    Pshufb,
    /// `permd`/`permps`/`permb`/`permw`/`permq`/`permpd`: (lane). `permq`/`permpd` with an
    /// immediate permute 64-bit lanes within each 256-bit half.
    Perm(Ty),
    /// `permt2*` (false) / `permi2*` (true).
    PermTwo(Ty, bool),
    /// `permilps`/`permilpd` with an immediate or a vector control.
    PermilImm(Ty),
    Perm2x128,
    /// `shufi32x4`...: 128-bit chunks from both sources (lane size for masking).
    Shuf128(Ty),
    /// Insert / extract a chunk of this many bytes (lane size for masking).
    Insert(u64, Ty),
    Extract(u64, Ty),
    Pinsr(Ty),
    Pextr(Ty),
    Insertps,
    Extractps,
    /// `movhps`/`movhpd` (true) or `movlps`/`movlpd`: 64-bit half loads and stores.
    MovHalf(bool),
    Movhlps,
    Movlhps,
    Dup(DupKind),
    /// `pmovsx*` / `pmovzx*`: (from, to, signed).
    Extend(Ty, Ty, bool),
    /// `pmov*` down-conversions: (from, to, saturation).
    Narrow(Ty, Ty, Sat),
    /// `pack*`: (from lane, unsigned saturation).
    Pack(Ty, bool),
    Cvt(Conv),
    /// `cmpps` & co. with a predicate immediate: (lane, scalar).
    FCmp(Ty, bool),
    /// AVX-512 integer compares into a mask: (lane, unsigned, fixed predicate).
    IntCmp(Ty, bool, Option<u8>),
    /// `comiss`/`ucomiss`/`comisd`/`ucomisd`.
    Comis(Ty),
    Ptest,
    /// `pcmpistri/pcmpestri/pcmpistrm/pcmpestrm`: (explicit lengths, mask result).
    Pcmpstr(bool, bool),
    /// `ptestm*` (false) / `ptestnm*` (true).
    Testm(Ty, bool),
    MaskToVec(Ty),
    VecToMask(Ty),
    BroadcastMask(Ty),
    Compress(Ty),
    Expand(Ty),
    Ternlog(Ty),
    /// `dpps` / `dppd`.
    Dp(Ty),
    /// `pclmulqdq imm` (None) or `pclmul{l,h}q{l,h}qdq` (the immediate).
    Pclmul(Option<u8>),
    Aes(AesOp),
    /// F16C `cvtph2ps` (false) / `cvtps2ph` (true).
    Half(bool),
    Sha(ShaOp),
    Gf(GfOp),
    /// `pscatter*`/`scatter*`: (index size, element size).
    Scatter(u64, u64),
    /// `pmaskmovd/q`, `maskmovps/pd`: masked load or store with a vector mask.
    MaskMov(Ty),
}
