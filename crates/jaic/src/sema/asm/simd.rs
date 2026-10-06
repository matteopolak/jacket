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
use super::vec::{VOpd, all_ones_if, float_to_s32, fselect, lane_addr, load_lane, store_lane};
use super::*;

mod ext;
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

pub(super) fn elem_size(op: SOp) -> u64 {
    use SOp::*;
    match op {
        Bin(_, t)
        | RotImm(_, t)
        | Unary(_, t)
        | Round(t, _)
        | Fma(_, _, t, _)
        | AddSub(t)
        | Unpack(_, t)
        | Valign(t)
        | BlendImm(t)
        | BlendVar(t)
        | BlendMask(t)
        | Perm(t)
        | PermTwo(t, _)
        | PermilImm(t)
        | Shuf128(t)
        | Insert(_, t)
        | Extract(_, t)
        | Extend(_, t, _)
        | Narrow(_, t, _)
        | FCmp(t, _)
        | IntCmp(t, ..)
        | Testm(t, _)
        | MaskToVec(t)
        | VecToMask(t)
        | BroadcastMask(t)
        | Compress(t)
        | Expand(t)
        | Ternlog(t)
        | Dp(t)
        | MaskMov(t) => t.size(),
        Horizontal(_, _, t) => t.size(),
        Pmaddwd | DotAcc(_) => 4,
        Pmaddubsw | ShufHalf(_) => 2,
        Psadbw | MulEven(_) | Shufpd | Pclmul(_) | MovHalf(_) => 8,
        Pack(from, _) => from.size() / 2,
        Cvt(c) => match c {
            Conv::IntToFloat(_, to, _) | Conv::FloatToInt(_, to, _, _) => to.size(),
            Conv::FloatToFloat(_, to) | Conv::ScalarFloat(_, to) => to.size(),
            _ => 4,
        },
        Palignr | ByteShift(_) | Pshufb | Aes(_) | Gf(_) => 1,
        Half(to_half) => {
            if to_half {
                2
            } else {
                4
            }
        }
        Sha(_) => 4,
        Scatter(_, e) => e,
        _ => 4,
    }
}

/// The class of a fresh `name:` destination.
pub(super) fn dst_class(op: SOp, width: u64) -> &'static str {
    match op {
        SOp::IntCmp(..) | SOp::Testm(..) | SOp::VecToMask(_) => "omr",
        SOp::FCmp(..) if width == 64 => "omr",
        SOp::Pextr(_) | SOp::Extractps | SOp::Cvt(Conv::ScalarToGpr(..)) => "gpr",
        _ => "vec",
    }
}

/// Instructions that have a 64-bit MMX (`.q`) form.
pub(super) fn mmx_capable(op: SOp) -> bool {
    matches!(
        op,
        SOp::Bin(..)
            | SOp::Horizontal(_, _, Ty::I16 | Ty::I32)
            | SOp::Pmaddwd
            | SOp::Pmaddubsw
            | SOp::Psadbw
            | SOp::MulEven(false)
            | SOp::Unpack(..)
            | SOp::Pack(..)
            | SOp::Pshufb
            | SOp::Palignr
    )
}

/// Instructions that apply a `{k}` write mask themselves (no lane merge afterwards).
pub(super) fn consumes_mask(op: SOp) -> bool {
    matches!(
        op,
        SOp::BlendMask(_)
            | SOp::Compress(_)
            | SOp::Scatter(..)
            | SOp::IntCmp(..)
            | SOp::Testm(..)
            | SOp::VecToMask(_)
            | SOp::FCmp(..)
            | SOp::Half(true)
    )
}

/// The element an embedded broadcast (`[mem]!`) repeats; the masking element otherwise.
pub(super) fn broadcast_size(op: SOp) -> u64 {
    match op {
        // The affine matrix operand is one 64-bit matrix per qword.
        SOp::Gf(GfOp::Affine(_)) => 8,
        _ => elem_size(op),
    }
}

fn int_letter(c: &str) -> Option<Ty> {
    Some(match c {
        "b" => Ty::I8,
        "w" => Ty::I16,
        "d" => Ty::I32,
        "q" => Ty::I64,
        _ => return None,
    })
}

fn float_letters(c: &str) -> Option<(Ty, bool)> {
    Some(match c {
        "ps" => (Ty::F32, false),
        "pd" => (Ty::F64, false),
        "ss" => (Ty::F32, true),
        "sd" => (Ty::F64, true),
        _ => return None,
    })
}

fn size_letter(c: char) -> Option<Ty> {
    int_letter(&c.to_string())
}

pub(super) fn lookup_simd(name: &str) -> Option<SOp> {
    use SOp::*;
    let i = int_letter;
    Some(match name {
        "paddsb" => Bin(B2::AddSatS, Ty::I8),
        "paddsw" => Bin(B2::AddSatS, Ty::I16),
        "paddusb" => Bin(B2::AddSatU, Ty::I8),
        "paddusw" => Bin(B2::AddSatU, Ty::I16),
        "psubsb" => Bin(B2::SubSatS, Ty::I8),
        "psubsw" => Bin(B2::SubSatS, Ty::I16),
        "psubusb" => Bin(B2::SubSatU, Ty::I8),
        "psubusw" => Bin(B2::SubSatU, Ty::I16),
        "pavgb" => Bin(B2::Avg, Ty::I8),
        "pavgw" => Bin(B2::Avg, Ty::I16),
        "pmulhw" => Bin(B2::MulHiS, Ty::I16),
        "pmulhuw" => Bin(B2::MulHiU, Ty::I16),
        "pmulhrsw" => Bin(B2::MulHrs, Ty::I16),
        "pmuludq" => MulEven(false),
        "pmuldq" => MulEven(true),
        "pmaddwd" => Pmaddwd,
        "pmaddubsw" => Pmaddubsw,
        "psadbw" => Psadbw,
        "pdpbusd" => DotAcc(Ty::I8),
        "pdpwssd" => DotAcc(Ty::I16),
        "phminposuw" => Phminposuw,
        "phaddw" => Horizontal(false, false, Ty::I16),
        "phaddd" => Horizontal(false, false, Ty::I32),
        "phaddsw" => Horizontal(false, true, Ty::I16),
        "phsubw" => Horizontal(true, false, Ty::I16),
        "phsubd" => Horizontal(true, false, Ty::I32),
        "phsubsw" => Horizontal(true, true, Ty::I16),
        "haddps" => Horizontal(false, false, Ty::F32),
        "haddpd" => Horizontal(false, false, Ty::F64),
        "hsubps" => Horizontal(true, false, Ty::F32),
        "hsubpd" => Horizontal(true, false, Ty::F64),
        "addsubps" => AddSub(Ty::F32),
        "addsubpd" => AddSub(Ty::F64),
        "unpcklps" => Unpack(false, Ty::I32),
        "unpckhps" => Unpack(true, Ty::I32),
        "unpcklpd" => Unpack(false, Ty::I64),
        "unpckhpd" => Unpack(true, Ty::I64),
        "punpcklqdq" => Unpack(false, Ty::I64),
        "punpckhqdq" => Unpack(true, Ty::I64),
        "pshufhw" => ShufHalf(true),
        "pshuflw" => ShufHalf(false),
        "shufpd" => Shufpd,
        "palignr" => Palignr,
        "alignd" | "valignd" => Valign(Ty::I32),
        "alignq" | "valignq" => Valign(Ty::I64),
        "pslldq" => ByteShift(true),
        "psrldq" => ByteShift(false),
        "pblendw" => BlendImm(Ty::I16),
        "pblendd" | "blendps" => BlendImm(Ty::I32),
        "blendpd" => BlendImm(Ty::I64),
        "pblendvb" => BlendVar(Ty::I8),
        "blendvps" => BlendVar(Ty::I32),
        "blendvpd" => BlendVar(Ty::I64),
        "blendmps" => BlendMask(Ty::I32),
        "blendmpd" => BlendMask(Ty::I64),
        "pshufb" => Pshufb,
        "permps" => Perm(Ty::I32),
        "permpd" => Perm(Ty::I64),
        "permilps" => PermilImm(Ty::I32),
        "permilpd" => PermilImm(Ty::I64),
        "perm2i128" | "perm2f128" => Perm2x128,
        "shufi32x4" | "shuff32x4" => Shuf128(Ty::I32),
        "shufi64x2" | "shuff64x2" => Shuf128(Ty::I64),
        "inserti128" | "insertf128" | "inserti32x4" | "insertf32x4" => Insert(16, Ty::I32),
        "inserti64x2" | "insertf64x2" => Insert(16, Ty::I64),
        "inserti32x8" | "insertf32x8" => Insert(32, Ty::I32),
        "inserti64x4" | "insertf64x4" => Insert(32, Ty::I64),
        "extracti128" | "extractf128" | "extracti32x4" | "extractf32x4" => Extract(16, Ty::I32),
        "extracti64x2" | "extractf64x2" => Extract(16, Ty::I64),
        "extracti32x8" | "extractf32x8" => Extract(32, Ty::I32),
        "extracti64x4" | "extractf64x4" => Extract(32, Ty::I64),
        "insertps" => Insertps,
        "extractps" => Extractps,
        "movhps" | "movhpd" => MovHalf(true),
        "movlps" | "movlpd" => MovHalf(false),
        "movhlps" => Movhlps,
        "movlhps" => Movlhps,
        "movddup" => Dup(DupKind::Low64),
        "movshdup" => Dup(DupKind::OddF32),
        "movsldup" => Dup(DupKind::EvenF32),
        "packsswb" => Pack(Ty::I16, false),
        "packssdw" => Pack(Ty::I32, false),
        "packuswb" => Pack(Ty::I16, true),
        "packusdw" => Pack(Ty::I32, true),
        "cvtdq2pd" => Cvt(Conv::IntToFloat(Ty::I32, Ty::F64, false)),
        "cvtudq2ps" => Cvt(Conv::IntToFloat(Ty::I32, Ty::F32, true)),
        "cvtudq2pd" => Cvt(Conv::IntToFloat(Ty::I32, Ty::F64, true)),
        "cvtqq2ps" => Cvt(Conv::IntToFloat(Ty::I64, Ty::F32, false)),
        "cvtqq2pd" => Cvt(Conv::IntToFloat(Ty::I64, Ty::F64, false)),
        "cvtuqq2ps" => Cvt(Conv::IntToFloat(Ty::I64, Ty::F32, true)),
        "cvtuqq2pd" => Cvt(Conv::IntToFloat(Ty::I64, Ty::F64, true)),
        "cvtpd2dq" => Cvt(Conv::FloatToInt(Ty::F64, Ty::I32, false, false)),
        "cvttpd2dq" => Cvt(Conv::FloatToInt(Ty::F64, Ty::I32, true, false)),
        "cvtps2udq" => Cvt(Conv::FloatToInt(Ty::F32, Ty::I32, false, true)),
        "cvttps2udq" => Cvt(Conv::FloatToInt(Ty::F32, Ty::I32, true, true)),
        "cvtpd2udq" => Cvt(Conv::FloatToInt(Ty::F64, Ty::I32, false, true)),
        "cvttpd2udq" => Cvt(Conv::FloatToInt(Ty::F64, Ty::I32, true, true)),
        "cvtps2qq" => Cvt(Conv::FloatToInt(Ty::F32, Ty::I64, false, false)),
        "cvttps2qq" => Cvt(Conv::FloatToInt(Ty::F32, Ty::I64, true, false)),
        "cvtpd2qq" => Cvt(Conv::FloatToInt(Ty::F64, Ty::I64, false, false)),
        "cvttpd2qq" => Cvt(Conv::FloatToInt(Ty::F64, Ty::I64, true, false)),
        "cvtps2uqq" => Cvt(Conv::FloatToInt(Ty::F32, Ty::I64, false, true)),
        "cvttps2uqq" => Cvt(Conv::FloatToInt(Ty::F32, Ty::I64, true, true)),
        "cvtpd2uqq" => Cvt(Conv::FloatToInt(Ty::F64, Ty::I64, false, true)),
        "cvttpd2uqq" => Cvt(Conv::FloatToInt(Ty::F64, Ty::I64, true, true)),
        "cvtps2pd" => Cvt(Conv::FloatToFloat(Ty::F32, Ty::F64)),
        "cvtpd2ps" => Cvt(Conv::FloatToFloat(Ty::F64, Ty::F32)),
        "cvtss2sd" => Cvt(Conv::ScalarFloat(Ty::F32, Ty::F64)),
        "cvtsd2ss" => Cvt(Conv::ScalarFloat(Ty::F64, Ty::F32)),
        "cvtsi2ss" => Cvt(Conv::GprToScalar(Ty::F32, false)),
        "cvtsi2sd" => Cvt(Conv::GprToScalar(Ty::F64, false)),
        "cvtusi2ss" => Cvt(Conv::GprToScalar(Ty::F32, true)),
        "cvtusi2sd" => Cvt(Conv::GprToScalar(Ty::F64, true)),
        "cvtss2si" => Cvt(Conv::ScalarToGpr(Ty::F32, false, false)),
        "cvttss2si" => Cvt(Conv::ScalarToGpr(Ty::F32, true, false)),
        "cvtsd2si" => Cvt(Conv::ScalarToGpr(Ty::F64, false, false)),
        "cvttsd2si" => Cvt(Conv::ScalarToGpr(Ty::F64, true, false)),
        "cvtss2usi" => Cvt(Conv::ScalarToGpr(Ty::F32, false, true)),
        "cvttss2usi" => Cvt(Conv::ScalarToGpr(Ty::F32, true, true)),
        "cvtsd2usi" => Cvt(Conv::ScalarToGpr(Ty::F64, false, true)),
        "cvttsd2usi" => Cvt(Conv::ScalarToGpr(Ty::F64, true, true)),
        "comiss" | "ucomiss" => Comis(Ty::F32),
        "comisd" | "ucomisd" => Comis(Ty::F64),
        "ptest" => Ptest,
        "pternlogd" => Ternlog(Ty::I32),
        "pternlogq" => Ternlog(Ty::I64),
        "dpps" => Dp(Ty::F32),
        "dppd" => Dp(Ty::F64),
        "pclmulqdq" => Pclmul(None),
        "pclmullqlqdq" => Pclmul(Some(0x00)),
        "pclmulhqlqdq" => Pclmul(Some(0x01)),
        "pclmullqhqdq" => Pclmul(Some(0x10)),
        "pclmulhqhqdq" => Pclmul(Some(0x11)),
        "aesenc" => Aes(AesOp::Enc),
        "aesenclast" => Aes(AesOp::EncLast),
        "aesdec" => Aes(AesOp::Dec),
        "aesdeclast" => Aes(AesOp::DecLast),
        "aesimc" => Aes(AesOp::Imc),
        "aeskeygenassist" => Aes(AesOp::KeygenAssist),
        "cvtph2ps" => Half(false),
        "cvtps2ph" => Half(true),
        "sha1rnds4" => Sha(ShaOp::Sha1Rnds4),
        "sha1nexte" => Sha(ShaOp::Sha1Nexte),
        "sha1msg1" => Sha(ShaOp::Sha1Msg1),
        "sha1msg2" => Sha(ShaOp::Sha1Msg2),
        "sha256rnds2" => Sha(ShaOp::Sha256Rnds2),
        "sha256msg1" => Sha(ShaOp::Sha256Msg1),
        "sha256msg2" => Sha(ShaOp::Sha256Msg2),
        "gf2p8mulb" => Gf(GfOp::Mul),
        "gf2p8affineqb" => Gf(GfOp::Affine(false)),
        "gf2p8affineinvqb" => Gf(GfOp::Affine(true)),
        "pmaskmovd" | "maskmovps" => MaskMov(Ty::I32),
        "pmaskmovq" | "maskmovpd" => MaskMov(Ty::I64),
        "pbroadcastmw2d" => BroadcastMask(Ty::I32),
        "pbroadcastmb2q" => BroadcastMask(Ty::I64),
        "compressps" => Compress(Ty::I32),
        "compresspd" => Compress(Ty::I64),
        "expandps" => Expand(Ty::I32),
        "expandpd" => Expand(Ty::I64),
        "pscatterdd" | "scatterdps" => Scatter(4, 4),
        "pscatterdq" | "scatterdpd" => Scatter(4, 8),
        "pscatterqd" | "scatterqps" => Scatter(8, 4),
        "pscatterqq" | "scatterqpd" => Scatter(8, 8),
        _ => {
            if let Some(rest) = name
                .strip_prefix("punpckl")
                .or(name.strip_prefix("punpckh"))
            {
                // punpcklbw / punpckhwd / punpckhdq: the source lane is the first letter.
                let high = name.starts_with("punpckh");
                let mut chars = rest.chars();
                let lane = size_letter(chars.next()?)?;
                return Some(Unpack(high, lane));
            }
            for (prefix, op) in [
                ("psllv", B2::Shlv),
                ("psrlv", B2::Shrv),
                ("psrav", B2::Sarv),
                ("prolv", B2::Rolv),
                ("prorv", B2::Rorv),
                ("psign", B2::Sign),
            ] {
                if let Some(t) = name.strip_prefix(prefix).and_then(i) {
                    return Some(Bin(op, t));
                }
            }
            if let Some(t) = name.strip_prefix("prol").and_then(i) {
                return Some(RotImm(true, t));
            }
            if let Some(t) = name.strip_prefix("pror").and_then(i) {
                return Some(RotImm(false, t));
            }
            for (prefix, op) in [
                ("popcnt", U1::Popcnt),
                ("plzcnt", U1::Lzcnt),
                ("pconflict", U1::Conflict),
            ] {
                if let Some(t) = name.strip_prefix(prefix).and_then(i) {
                    return Some(Unary(op, t));
                }
            }
            if let Some(t) = name.strip_prefix("perm").and_then(i) {
                return Some(Perm(t));
            }
            if let Some(t) = name.strip_prefix("permt2").and_then(i) {
                return Some(PermTwo(t, false));
            }
            if let Some(t) = name.strip_prefix("permi2").and_then(i) {
                return Some(PermTwo(t, true));
            }
            match name {
                "permt2ps" => return Some(PermTwo(Ty::I32, false)),
                "permt2pd" => return Some(PermTwo(Ty::I64, false)),
                "permi2ps" => return Some(PermTwo(Ty::I32, true)),
                "permi2pd" => return Some(PermTwo(Ty::I64, true)),
                _ => {}
            }
            for (prefix, op) in [
                ("pblendm", BlendMask as fn(Ty) -> SOp),
                ("pcompress", Compress),
                ("pexpand", Expand),
                ("pmovm2", MaskToVec),
                ("pinsr", Pinsr),
                ("pextr", Pextr),
            ] {
                if let Some(t) = name.strip_prefix(prefix).and_then(i) {
                    return Some(op(t));
                }
            }
            if let Some(rest) = name.strip_prefix("pmov")
                && let Some(t) = rest.strip_suffix("2m").and_then(i)
            {
                return Some(VecToMask(t));
            }
            if let Some(rest) = name.strip_prefix("ptestnm").and_then(i) {
                return Some(Testm(rest, true));
            }
            if let Some(rest) = name.strip_prefix("ptestm").and_then(i) {
                return Some(Testm(rest, false));
            }
            // pmovsx / pmovzx <from><to>
            for (prefix, signed) in [("pmovsx", true), ("pmovzx", false)] {
                if let Some(rest) = name.strip_prefix(prefix) {
                    let mut c = rest.chars();
                    let (from, to) = (size_letter(c.next()?)?, size_letter(c.next()?)?);
                    if c.next().is_some() || from.size() >= to.size() {
                        return None;
                    }
                    return Some(Extend(from, to, signed));
                }
            }
            // pmov[s|us]<from><to> down-conversions (pmovwb, pmovsdb, pmovusqd...).
            for (prefix, sat) in [
                ("pmovus", Sat::Unsigned),
                ("pmovs", Sat::Signed),
                ("pmov", Sat::Wrap),
            ] {
                if let Some(rest) = name.strip_prefix(prefix) {
                    let mut c = rest.chars();
                    let (Some(a), Some(b), None) = (c.next(), c.next(), c.next()) else {
                        continue;
                    };
                    let (Some(from), Some(to)) = (size_letter(a), size_letter(b)) else {
                        continue;
                    };
                    if from.size() <= to.size() {
                        continue;
                    }
                    return Some(Narrow(from, to, sat));
                }
            }
            // pcmp[u]<b|w|d|q> with a predicate immediate (AVX-512).
            if let Some(rest) = name.strip_prefix("pcmp") {
                let (unsigned, rest) = match rest.strip_prefix('u') {
                    Some(r) => (true, r),
                    None => (false, rest),
                };
                if let Some(t) = i(rest) {
                    return Some(IntCmp(t, unsigned, None));
                }
            }
            if let Some(form) = name.strip_prefix("cmp")
                && let Some((t, scalar)) = float_letters(form)
            {
                return Some(FCmp(t, scalar));
            }
            for prefix in ["round", "rndscale"] {
                if let Some(form) = name.strip_prefix(prefix)
                    && let Some((t, scalar)) = float_letters(form)
                {
                    return Some(Round(t, scalar));
                }
            }
            for (prefix, op) in [
                ("rcp14", U1::Rcp),
                ("rcp28", U1::Rcp),
                ("rcp", U1::Rcp),
                ("rsqrt14", U1::Rsqrt),
                ("rsqrt28", U1::Rsqrt),
                ("rsqrt", U1::Rsqrt),
            ] {
                if let Some(form) = name.strip_prefix(prefix)
                    && let Some((t, false)) = float_letters(form)
                {
                    return Some(Unary(op, t));
                }
            }
            // FMA: f[n]m{add,sub,addsub,subadd}{132,213,231}{ps,pd,ss,sd}
            if let Some(rest) = name.strip_prefix('f') {
                let (neg, rest) = match rest.strip_prefix('n') {
                    Some(r) => (true, r),
                    None => (false, rest),
                };
                let rest = rest.strip_prefix('m')?;
                let (kind, rest) = if let Some(r) = rest.strip_prefix("addsub") {
                    (FmaKind::MaddSub, r)
                } else if let Some(r) = rest.strip_prefix("subadd") {
                    (FmaKind::MsubAdd, r)
                } else if let Some(r) = rest.strip_prefix("add") {
                    (
                        if neg {
                            FmaKind::Nmadd
                        } else {
                            FmaKind::Madd
                        },
                        r,
                    )
                } else {
                    let r = rest.strip_prefix("sub")?;
                    (
                        if neg {
                            FmaKind::Nmsub
                        } else {
                            FmaKind::Msub
                        },
                        r,
                    )
                };
                if neg && matches!(kind, FmaKind::MaddSub | FmaKind::MsubAdd) {
                    return None;
                }
                let order: u16 = rest.get(..3)?.parse().ok()?;
                if !matches!(order, 132 | 213 | 231) {
                    return None;
                }
                let (t, scalar) = float_letters(&rest[3..])?;
                if scalar && matches!(kind, FmaKind::MaddSub | FmaKind::MsubAdd) {
                    return None;
                }
                return Some(Fma(kind, order, t, scalar));
            }
            return None;
        }
    })
}

/// Saturate a value of the wide type `wt` to `ty` lanes (signed or unsigned range).
fn saturate(f: &mut FnCtx, wt: Ty, v: Val, ty: Ty, signed: bool) -> Val {
    let b = bits(ty);
    let (lo, hi) = if signed {
        ((1u64 << (b - 1)).wrapping_neg(), (1u64 << (b - 1)) - 1)
    } else {
        (0, (1u64 << b) - 1)
    };
    let lo_v = konst(f, wt, lo);
    let hi_v = konst(f, wt, hi);
    let below = cmp(f, CmpOp::SLt, wt, v, lo_v);
    let above = cmp(f, CmpOp::SGt, wt, v, hi_v);
    let v = select(f, wt, below, lo_v, v);
    let v = select(f, wt, above, hi_v, v);
    resize(f, v, wt, ty, false)
}

/// Round an `F64` to an integral value: n(earest even), d(own), u(p) or z (toward zero).
fn round_f64(f: &mut FnCtx, x: Val, mode: char) -> Val {
    match mode {
        'd' => f.b.intrinsic(Intrinsic::Floor, vec![x], &[Ty::F64])[0],
        'u' => f.b.intrinsic(Intrinsic::Ceil, vec![x], &[Ty::F64])[0],
        'z' => f.b.intrinsic(Intrinsic::Trunc, vec![x], &[Ty::F64])[0],
        _ => {
            // Below 2^52, adding and removing 2^52 rounds to nearest even; above, x is integral.
            let ax = f.b.intrinsic(Intrinsic::Fabs, vec![x], &[Ty::F64])[0];
            let big = f.b.fconst(Ty::F64, 4503599627370496.0);
            let up = bin(f, BinOp::FAdd, Ty::F64, ax, big);
            let r = bin(f, BinOp::FSub, Ty::F64, up, big);
            let zero = f.b.fconst(Ty::F64, 0.0);
            let neg = cmp(f, CmpOp::FLt, Ty::F64, x, zero);
            let nr = f.b.un(UnOp::FNeg, Ty::F64, r);
            let r = fselect(f, Ty::F64, neg, nr, r);
            let small = cmp(f, CmpOp::FLt, Ty::F64, ax, big);
            fselect(f, Ty::F64, small, r, x)
        }
    }
}

fn to_f64(f: &mut FnCtx, ty: Ty, x: Val) -> Val {
    if ty == Ty::F32 {
        f.b.conv(ConvOp::FExt, Ty::F32, Ty::F64, x)
    } else {
        x
    }
}

fn from_f64(f: &mut FnCtx, ty: Ty, x: Val) -> Val {
    if ty == Ty::F32 {
        f.b.conv(ConvOp::FTrunc, Ty::F64, Ty::F32, x)
    } else {
        x
    }
}

/// Float to integer the way the SSE/AVX conversions do: out-of-range and NaN give the
/// "integer indefinite" value (the minimum for signed, all ones for unsigned).
fn float_to_int(f: &mut FnCtx, from: Ty, x: Val, to: Ty, mode: char, unsigned: bool) -> Val {
    if from == Ty::F32 && to == Ty::I32 && !unsigned {
        return float_to_s32(f, x, mode);
    }
    let x = to_f64(f, from, x);
    let r = round_f64(f, x, mode);
    let w = bits(to) as i32;
    let (lo, hi) = if unsigned {
        (0.0, 2f64.powi(w))
    } else {
        (-(2f64.powi(w - 1)), 2f64.powi(w - 1))
    };
    let lo_v = f.b.fconst(Ty::F64, lo);
    let hi_v = f.b.fconst(Ty::F64, hi);
    let ge = cmp(f, CmpOp::FGe, Ty::F64, r, lo_v);
    let lt = cmp(f, CmpOp::FLt, Ty::F64, r, hi_v);
    let ok = bin(f, BinOp::And, Ty::I8, ge, lt);
    let zero = f.b.fconst(Ty::F64, 0.0);
    let safe = fselect(f, Ty::F64, ok, r, zero);
    let v = if unsigned {
        f.b.conv(ConvOp::FToU, Ty::F64, Ty::I64, safe)
    } else {
        f.b.conv(ConvOp::FToS, Ty::F64, Ty::I64, safe)
    };
    let v = resize(f, v, Ty::I64, to, false);
    let bad = if unsigned {
        konst(f, to, u64::MAX)
    } else {
        konst(f, to, 1u64 << (w - 1))
    };
    select(f, to, ok, v, bad)
}

/// The AES S-box, its inverse and the GF(2^8) multiplicative inverse (0 for 0, used by
/// `gf2p8affineinvqb`), generated from the field inverse and the affine map.
fn aes_tables() -> Vec<u8> {
    let mul = |mut a: u8, mut b: u8| {
        let mut p = 0u8;
        while b != 0 {
            if b & 1 != 0 {
                p ^= a;
            }
            let hi = a & 0x80;
            a <<= 1;
            if hi != 0 {
                a ^= 0x1b;
            }
            b >>= 1;
        }
        p
    };
    let mut sbox = [0u8; 256];
    let mut inv = [0u8; 256];
    let mut field_inv = [0u8; 256];
    for x in 0..=255u8 {
        let mut y = 0u8;
        if x != 0 {
            // x^254 is the multiplicative inverse in GF(2^8).
            let mut acc = 1u8;
            let mut base = x;
            let mut e = 254u32;
            while e > 0 {
                if e & 1 != 0 {
                    acc = mul(acc, base);
                }
                base = mul(base, base);
                e >>= 1;
            }
            y = acc;
        }
        field_inv[x as usize] = y;
        let s =
            y ^ y.rotate_left(1) ^ y.rotate_left(2) ^ y.rotate_left(3) ^ y.rotate_left(4) ^ 0x63;
        sbox[x as usize] = s;
        inv[s as usize] = x;
    }
    let mut out = sbox.to_vec();
    out.extend_from_slice(&inv);
    out.extend_from_slice(&field_inv);
    out
}

/// Multiply an `I8` by 2 in GF(2^8).
fn xtime(f: &mut FnCtx, x: Val) -> Val {
    let one = konst(f, Ty::I8, 1);
    let seven = konst(f, Ty::I8, 7);
    let t = bin(f, BinOp::Shl, Ty::I8, x, one);
    let hi = bin(f, BinOp::AShr, Ty::I8, x, seven);
    let poly = konst(f, Ty::I8, 0x1b);
    let m = bin(f, BinOp::And, Ty::I8, hi, poly);
    bin(f, BinOp::Xor, Ty::I8, t, m)
}

fn xor8(f: &mut FnCtx, a: Val, b: Val) -> Val {
    bin(f, BinOp::Xor, Ty::I8, a, b)
}

/// AES MixColumns (or InvMixColumns) of the 16 bytes of `s`.
fn mix_columns(f: &mut FnCtx, s: &[Val; 16], inverse: bool) -> [Val; 16] {
    let mut out = *s;
    for c in 0..4 {
        let a: Vec<Val> = (0..4).map(|r| s[4 * c + r]).collect();
        let x2: Vec<Val> = a.iter().map(|&v| xtime(f, v)).collect();
        if !inverse {
            for r in 0..4 {
                // 2*a[r] ^ 3*a[r+1] ^ a[r+2] ^ a[r+3]
                let n1 = a[(r + 1) % 4];
                let t = xor8(f, x2[r], x2[(r + 1) % 4]);
                let t = xor8(f, t, n1);
                let t = xor8(f, t, a[(r + 2) % 4]);
                out[4 * c + r] = xor8(f, t, a[(r + 3) % 4]);
            }
        } else {
            let x4: Vec<Val> = x2.iter().map(|&v| xtime(f, v)).collect();
            let x8: Vec<Val> = x4.iter().map(|&v| xtime(f, v)).collect();
            // 14 = 8^4^2, 11 = 8^2^1, 13 = 8^4^1, 9 = 8^1
            let m14: Vec<Val> = (0..4)
                .map(|k| {
                    let t = xor8(f, x8[k], x4[k]);
                    xor8(f, t, x2[k])
                })
                .collect();
            let m11: Vec<Val> = (0..4)
                .map(|k| {
                    let t = xor8(f, x8[k], x2[k]);
                    xor8(f, t, a[k])
                })
                .collect();
            let m13: Vec<Val> = (0..4)
                .map(|k| {
                    let t = xor8(f, x8[k], x4[k]);
                    xor8(f, t, a[k])
                })
                .collect();
            let m9: Vec<Val> = (0..4).map(|k| xor8(f, x8[k], a[k])).collect();
            for r in 0..4 {
                let t = xor8(f, m14[r], m11[(r + 1) % 4]);
                let t = xor8(f, t, m13[(r + 2) % 4]);
                out[4 * c + r] = xor8(f, t, m9[(r + 3) % 4]);
            }
        }
    }
    out
}

/// Load `ty` at `base + index * ty.size()` for a runtime lane index (`I64`).
fn load_dyn(f: &mut FnCtx, base: Val, index: Val, ty: Ty) -> Val {
    let size = konst(f, Ty::I64, ty.size());
    let off = bin(f, BinOp::Mul, Ty::I64, index, size);
    let p = f.b.ptr_add(base, off);
    f.b.load(ty, p)
}

fn store_dyn(f: &mut FnCtx, base: Val, index: Val, ty: Ty, v: Val) {
    let size = konst(f, Ty::I64, ty.size());
    let off = bin(f, BinOp::Mul, Ty::I64, index, size);
    let p = f.b.ptr_add(base, off);
    f.b.store(ty, p, v);
}

/// Bit `i` of a 64-bit mask value as an `I8` flag.
fn mask_bit(f: &mut FnCtx, mask: Val, i: u64) -> Val {
    let at = konst(f, Ty::I64, i);
    let b = bin(f, BinOp::LShr, Ty::I64, mask, at);
    let one = konst(f, Ty::I64, 1);
    let b = bin(f, BinOp::And, Ty::I64, b, one);
    resize(f, b, Ty::I64, Ty::I8, false)
}

/// Run `body` only when `cond` is set (for per-lane memory accesses that must not fault).
fn when(f: &mut FnCtx, cond: Val, body: impl FnOnce(&mut FnCtx)) {
    let then = f.b.new_block();
    let join = f.b.new_block();
    f.b.branch(cond, then, join);
    f.b.switch_to(then);
    body(f);
    f.b.jump(join);
    f.b.switch_to(join);
}

/// A `{k}` write mask: its bits, zeroing (`&*`) and the register (cleared by scatters).
#[derive(Clone, Copy)]
pub(super) struct WriteMask {
    pub bits: Val,
    pub zeroing: bool,
    pub reg: Option<Val>,
}

impl Compiler {
    /// Sources and immediate of `dst, src..., [imm8]`. With one source fewer than
    /// `nsrc`, the destination is also the first source.
    fn simd_args(
        &mut self,
        f: &mut FnCtx,
        ops: &[VOpd],
        nsrc: usize,
        imm: bool,
        span: Span,
    ) -> Result<(Vec<Val>, u64)> {
        if ops.is_empty() {
            return err(span, "missing destination operand");
        }
        let mut rest = &ops[1..];
        let mut k = 0;
        if imm {
            match rest.last() {
                Some(&VOpd::Imm(v)) if (0..=255).contains(&v) => {
                    k = v as u64;
                    rest = &rest[..rest.len() - 1];
                }
                _ => return err(span, "expected an 8-bit immediate as the last operand"),
            }
        }
        let mut srcs = Vec::with_capacity(nsrc);
        if rest.len() + 1 == nsrc {
            srcs.push(self.vec_ptr(f, ops[0], span)?);
        } else if rest.len() != nsrc {
            return err(
                span,
                format!(
                    "expected {nsrc} source operand(s){}",
                    if imm {
                        " and an immediate"
                    } else {
                        ""
                    }
                ),
            );
        }
        for &o in rest {
            srcs.push(self.vec_ptr(f, o, span)?);
        }
        Ok((srcs, k))
    }

    /// Write `bytes` of the scratch buffer to the destination (at least the 16 bytes of
    /// an xmm register when the destination is a register).
    fn simd_out(
        &mut self,
        f: &mut FnCtx,
        cx: &AsmCtx,
        dst: VOpd,
        tmp: Val,
        bytes: u64,
        span: Span,
    ) -> Result<()> {
        let size = match dst {
            VOpd::Reg(_) if bytes < 16 => {
                let rest = lane_addr(f, tmp, bytes);
                f.b.zero(rest, 16 - bytes);
                16
            }
            _ => bytes,
        };
        self.vec_store(f, cx, dst, tmp, size, span)
    }

    /// Write lane flags (bit i = lane i) to a mask destination, ANDed with the write mask.
    fn simd_mask_out(
        &mut self,
        f: &mut FnCtx,
        dst: VOpd,
        flags: Vec<Val>,
        wm: Option<WriteMask>,
        span: Span,
    ) -> Result<()> {
        let mut m = konst(f, Ty::I64, 0);
        for (i, flag) in flags.into_iter().enumerate() {
            let b = resize(f, flag, Ty::I8, Ty::I64, false);
            let at = konst(f, Ty::I64, i as u64);
            let b = bin(f, BinOp::Shl, Ty::I64, b, at);
            m = bin(f, BinOp::Or, Ty::I64, m, b);
        }
        if let Some(wm) = wm {
            m = bin(f, BinOp::And, Ty::I64, m, wm.bits);
        }
        match dst {
            VOpd::Mask(p) => {
                f.b.store(Ty::I64, p, m);
                Ok(())
            }
            _ => err(span, "the destination must be a mask register"),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn asm_simd(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        inst: &AsmInst,
        op: SOp,
        ops: &[VOpd],
        width: u64,
        wm: Option<WriteMask>,
    ) -> Result<()> {
        let span = inst.span;
        let name = inst.mnemonic.name.as_str();
        let dst = match ops.first() {
            Some(&d) => d,
            None => return err(span, format!("`{name}` needs operands")),
        };
        let tmp = f.b.alloca(64, 16);
        f.b.zero(tmp, 64);
        // In-lane operations work per 128-bit block (8 bytes for MMX).
        let block = width.min(16);
        let last_imm = matches!(ops.last(), Some(VOpd::Imm(_)));
        match op {
            SOp::Bin(b2, ty) => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                for i in 0..width / ty.size() {
                    let a = load_lane(f, s[0], i, ty);
                    let b = load_lane(f, s[1], i, ty);
                    let r = Self::simd_bin(f, b2, ty, a, b);
                    store_lane(f, tmp, i, ty, r);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::RotImm(left, ty) => {
                let (s, k) = self.simd_args(f, ops, 1, true, span)?;
                let c = konst(f, ty, k % bits(ty));
                for i in 0..width / ty.size() {
                    let a = load_lane(f, s[0], i, ty);
                    let r = bin(
                        f,
                        if left {
                            BinOp::Rotl
                        } else {
                            BinOp::Rotr
                        },
                        ty,
                        a,
                        c,
                    );
                    store_lane(f, tmp, i, ty, r);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::Unary(u, ty) => {
                let (s, _) = self.simd_args(f, ops, 1, false, span)?;
                let (lane, fty) = match ty {
                    Ty::F32 => (Ty::F32, true),
                    Ty::F64 => (Ty::F64, true),
                    t => (t, false),
                };
                let n = width / lane.size();
                for i in 0..n {
                    let a = load_lane(f, s[0], i, lane);
                    let r = match u {
                        U1::Popcnt => bit_intrinsic(f, Intrinsic::Popcount, lane, a),
                        U1::Lzcnt => bit_intrinsic(f, Intrinsic::Ctlz, lane, a),
                        U1::Rcp | U1::Rsqrt if fty => {
                            let x = to_f64(f, lane, a);
                            let x = if u == U1::Rsqrt {
                                f.b.intrinsic(Intrinsic::Sqrt, vec![x], &[Ty::F64])[0]
                            } else {
                                x
                            };
                            let one = f.b.fconst(Ty::F64, 1.0);
                            let r = bin(f, BinOp::FDiv, Ty::F64, one, x);
                            from_f64(f, lane, r)
                        }
                        U1::Conflict => {
                            let mut m = konst(f, lane, 0);
                            for j in 0..i {
                                let b = load_lane(f, s[0], j, lane);
                                let eq = cmp(f, CmpOp::Eq, lane, a, b);
                                let eq = resize(f, eq, Ty::I8, lane, false);
                                let at = konst(f, lane, j);
                                let eq = bin(f, BinOp::Shl, lane, eq, at);
                                m = bin(f, BinOp::Or, lane, m, eq);
                            }
                            m
                        }
                        _ => return err(span, format!("`{name}` needs float lanes")),
                    };
                    store_lane(f, tmp, i, lane, r);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::Round(ty, scalar) => {
                let (s, k) = self.simd_args(
                    f,
                    ops,
                    if scalar {
                        2
                    } else {
                        1
                    },
                    true,
                    span,
                )?;
                let mode = match (inst.evex.round, k & 4 != 0) {
                    (Some(Some(m)), _) => m,
                    (_, true) => 'n',
                    _ => ['n', 'd', 'u', 'z'][(k & 3) as usize],
                };
                // rndscale: round to a multiple of 2^-M (M = imm[7:4]).
                let scale = (1u64 << (k >> 4)) as f64;
                let lanes = if scalar {
                    1
                } else {
                    width / ty.size()
                };
                let src = *s.last().unwrap();
                if scalar {
                    f.b.copy(tmp, s[0], 16);
                }
                for i in 0..lanes {
                    let x = load_lane(f, src, i, ty);
                    let x = to_f64(f, ty, x);
                    let sc = f.b.fconst(Ty::F64, scale);
                    let y = bin(f, BinOp::FMul, Ty::F64, x, sc);
                    let y = round_f64(f, y, mode);
                    let y = bin(f, BinOp::FDiv, Ty::F64, y, sc);
                    let y = from_f64(f, ty, y);
                    store_lane(f, tmp, i, ty, y);
                }
                self.simd_out(
                    f,
                    cx,
                    dst,
                    tmp,
                    if scalar {
                        16
                    } else {
                        width
                    },
                    span,
                )?;
            }
            SOp::Fma(kind, order, ty, scalar) => {
                let (s, _) = self.simd_args(f, ops, 3, false, span)?;
                let (d, s2, s3) = (s[0], s[1], s[2]);
                let (pa, pb, pc) = match order {
                    132 => (d, s3, s2),
                    213 => (s2, d, s3),
                    _ => (s2, s3, d),
                };
                let lanes = if scalar {
                    1
                } else {
                    width / ty.size()
                };
                if scalar {
                    f.b.copy(tmp, d, 16);
                }
                for i in 0..lanes {
                    let a = load_lane(f, pa, i, ty);
                    let b = load_lane(f, pb, i, ty);
                    let c = load_lane(f, pc, i, ty);
                    let (a, b, c) = (to_f64(f, ty, a), to_f64(f, ty, b), to_f64(f, ty, c));
                    let even = i % 2 == 0;
                    let (neg_ab, neg_c) = match kind {
                        FmaKind::Madd => (false, false),
                        FmaKind::Msub => (false, true),
                        FmaKind::Nmadd => (true, false),
                        FmaKind::Nmsub => (true, true),
                        FmaKind::MaddSub => (false, even),
                        FmaKind::MsubAdd => (false, !even),
                    };
                    let a = if neg_ab {
                        f.b.un(UnOp::FNeg, Ty::F64, a)
                    } else {
                        a
                    };
                    let c = if neg_c {
                        f.b.un(UnOp::FNeg, Ty::F64, c)
                    } else {
                        c
                    };
                    let r = f.b.intrinsic(Intrinsic::Fma, vec![a, b, c], &[Ty::F64])[0];
                    let r = from_f64(f, ty, r);
                    store_lane(f, tmp, i, ty, r);
                }
                self.simd_out(
                    f,
                    cx,
                    dst,
                    tmp,
                    if scalar {
                        16
                    } else {
                        width
                    },
                    span,
                )?;
            }
            SOp::Horizontal(sub, sat, ty) => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                let per = block / ty.size();
                for blk in 0..width / block {
                    let base = blk * per;
                    for j in 0..per {
                        let src = if j < per / 2 {
                            s[0]
                        } else {
                            s[1]
                        };
                        let k = j % (per / 2);
                        let x = load_lane(f, src, base + 2 * k, ty);
                        let y = load_lane(f, src, base + 2 * k + 1, ty);
                        let r = if ty.is_float() {
                            bin(
                                f,
                                if sub {
                                    BinOp::FSub
                                } else {
                                    BinOp::FAdd
                                },
                                ty,
                                x,
                                y,
                            )
                        } else if sat {
                            Self::simd_bin(
                                f,
                                if sub {
                                    B2::SubSatS
                                } else {
                                    B2::AddSatS
                                },
                                ty,
                                x,
                                y,
                            )
                        } else {
                            bin(
                                f,
                                if sub {
                                    BinOp::Sub
                                } else {
                                    BinOp::Add
                                },
                                ty,
                                x,
                                y,
                            )
                        };
                        store_lane(f, tmp, base + j, ty, r);
                    }
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::AddSub(ty) => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                for i in 0..width / ty.size() {
                    let x = load_lane(f, s[0], i, ty);
                    let y = load_lane(f, s[1], i, ty);
                    let r = bin(
                        f,
                        if i % 2 == 0 {
                            BinOp::FSub
                        } else {
                            BinOp::FAdd
                        },
                        ty,
                        x,
                        y,
                    );
                    store_lane(f, tmp, i, ty, r);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::Pmaddwd | SOp::Pmaddubsw => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                let (src, out) = if op == SOp::Pmaddwd {
                    (Ty::I16, Ty::I32)
                } else {
                    (Ty::I8, Ty::I16)
                };
                for i in 0..width / out.size() {
                    let mut sum = konst(f, Ty::I32, 0);
                    for k in 0..2 {
                        let a = load_lane(f, s[0], 2 * i + k, src);
                        let b = load_lane(f, s[1], 2 * i + k, src);
                        // pmaddubsw: unsigned bytes of the first source times signed bytes.
                        let a = resize(f, a, src, Ty::I32, op == SOp::Pmaddwd);
                        let b = resize(f, b, src, Ty::I32, true);
                        let p = bin(f, BinOp::Mul, Ty::I32, a, b);
                        sum = bin(f, BinOp::Add, Ty::I32, sum, p);
                    }
                    let r = if out == Ty::I16 {
                        saturate(f, Ty::I32, sum, Ty::I16, true)
                    } else {
                        sum
                    };
                    store_lane(f, tmp, i, out, r);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::Psadbw => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                for i in 0..width / 8 {
                    let mut sum = konst(f, Ty::I64, 0);
                    for k in 0..8 {
                        let a = load_lane(f, s[0], 8 * i + k, Ty::I8);
                        let b = load_lane(f, s[1], 8 * i + k, Ty::I8);
                        let a = resize(f, a, Ty::I8, Ty::I64, false);
                        let b = resize(f, b, Ty::I8, Ty::I64, false);
                        let d = bin(f, BinOp::Sub, Ty::I64, a, b);
                        let nd = bin(f, BinOp::Sub, Ty::I64, b, a);
                        let lt = cmp(f, CmpOp::ULt, Ty::I64, a, b);
                        let d = select(f, Ty::I64, lt, nd, d);
                        sum = bin(f, BinOp::Add, Ty::I64, sum, d);
                    }
                    store_lane(f, tmp, i, Ty::I64, sum);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::MulEven(signed) => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                for i in 0..width / 8 {
                    let a = load_lane(f, s[0], 2 * i, Ty::I32);
                    let b = load_lane(f, s[1], 2 * i, Ty::I32);
                    let a = resize(f, a, Ty::I32, Ty::I64, signed);
                    let b = resize(f, b, Ty::I32, Ty::I64, signed);
                    let r = bin(f, BinOp::Mul, Ty::I64, a, b);
                    store_lane(f, tmp, i, Ty::I64, r);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::DotAcc(src) => {
                let (s, _) = self.simd_args(f, ops, 3, false, span)?;
                let per = 4 / src.size();
                for i in 0..width / 4 {
                    let mut sum = load_lane(f, s[0], i, Ty::I32);
                    for k in 0..per {
                        let a = load_lane(f, s[1], per * i + k, src);
                        let b = load_lane(f, s[2], per * i + k, src);
                        // pdpbusd: unsigned bytes times signed bytes; pdpwssd: signed words.
                        let a = resize(f, a, src, Ty::I32, src == Ty::I16);
                        let b = resize(f, b, src, Ty::I32, true);
                        let p = bin(f, BinOp::Mul, Ty::I32, a, b);
                        sum = bin(f, BinOp::Add, Ty::I32, sum, p);
                    }
                    store_lane(f, tmp, i, Ty::I32, sum);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::Phminposuw => {
                let (s, _) = self.simd_args(f, ops, 1, false, span)?;
                let mut best = load_lane(f, s[0], 0, Ty::I16);
                let mut at = konst(f, Ty::I16, 0);
                for i in 1..8 {
                    let v = load_lane(f, s[0], i, Ty::I16);
                    let lt = cmp(f, CmpOp::ULt, Ty::I16, v, best);
                    best = select(f, Ty::I16, lt, v, best);
                    let iv = konst(f, Ty::I16, i);
                    at = select(f, Ty::I16, lt, iv, at);
                }
                store_lane(f, tmp, 0, Ty::I16, best);
                store_lane(f, tmp, 1, Ty::I16, at);
                self.simd_out(f, cx, dst, tmp, 16, span)?;
            }
            SOp::Unpack(high, ty) => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                let per = block / ty.size();
                for blk in 0..width / block {
                    let base = blk * per;
                    let from = base
                        + if high {
                            per / 2
                        } else {
                            0
                        };
                    for k in 0..per / 2 {
                        let a = load_lane(f, s[0], from + k, ty);
                        let b = load_lane(f, s[1], from + k, ty);
                        store_lane(f, tmp, base + 2 * k, ty, a);
                        store_lane(f, tmp, base + 2 * k + 1, ty, b);
                    }
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::ShufHalf(high) => {
                let (s, k) = self.simd_args(f, ops, 1, true, span)?;
                f.b.copy(tmp, s[0], width);
                for blk in 0..width / 16 {
                    let base = blk * 8
                        + if high {
                            4
                        } else {
                            0
                        };
                    for j in 0..4 {
                        let pick = (k >> (2 * j)) & 3;
                        let v = load_lane(f, s[0], base + pick, Ty::I16);
                        store_lane(f, tmp, base + j, Ty::I16, v);
                    }
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::Shufpd => {
                let (s, k) = self.simd_args(f, ops, 2, true, span)?;
                for i in 0..width / 8 {
                    let blk = i / 2 * 2;
                    let src = if i % 2 == 0 {
                        s[0]
                    } else {
                        s[1]
                    };
                    let pick = (k >> i) & 1;
                    let v = load_lane(f, src, blk + pick, Ty::I64);
                    store_lane(f, tmp, i, Ty::I64, v);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::Palignr => {
                // Per block: bytes [imm, imm + block) of hi:lo (lo = second source).
                let (s, k) = self.simd_args(f, ops, 2, true, span)?;
                for blk in 0..width / block {
                    let base = blk * block;
                    for j in 0..block {
                        let at = j + k;
                        if at < block {
                            let v = load_lane(f, s[1], base + at, Ty::I8);
                            store_lane(f, tmp, base + j, Ty::I8, v);
                        } else if at < 2 * block {
                            let v = load_lane(f, s[0], base + at - block, Ty::I8);
                            store_lane(f, tmp, base + j, Ty::I8, v);
                        }
                    }
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::Valign(ty) => {
                let (s, k) = self.simd_args(f, ops, 2, true, span)?;
                let n = width / ty.size();
                let k = k % n;
                for j in 0..n {
                    let at = j + k;
                    let v = if at < n {
                        load_lane(f, s[1], at, ty)
                    } else {
                        load_lane(f, s[0], at - n, ty)
                    };
                    store_lane(f, tmp, j, ty, v);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::ByteShift(left) => {
                let (s, k) = self.simd_args(f, ops, 1, true, span)?;
                for blk in 0..width / 16 {
                    let base = blk * 16;
                    for j in 0..16u64 {
                        let from = if left {
                            j.checked_sub(k)
                        } else {
                            Some(j + k)
                        };
                        if let Some(from) = from.filter(|&x| x < 16) {
                            let v = load_lane(f, s[0], base + from, Ty::I8);
                            store_lane(f, tmp, base + j, Ty::I8, v);
                        }
                    }
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::BlendImm(ty) => {
                let (s, k) = self.simd_args(f, ops, 2, true, span)?;
                for i in 0..width / ty.size() {
                    let src = if (k >> (i % 8)) & 1 != 0 {
                        s[1]
                    } else {
                        s[0]
                    };
                    let v = load_lane(f, src, i, ty);
                    store_lane(f, tmp, i, ty, v);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::BlendVar(ty) => {
                let (s, _) = self.simd_args(f, ops, 3, false, span)?;
                for i in 0..width / ty.size() {
                    let a = load_lane(f, s[0], i, ty);
                    let b = load_lane(f, s[1], i, ty);
                    let m = load_lane(f, s[2], i, ty);
                    let on = is_neg(f, ty, m);
                    let r = select(f, ty, on, b, a);
                    store_lane(f, tmp, i, ty, r);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::BlendMask(ty) => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                for i in 0..width / ty.size() {
                    let a = load_lane(f, s[0], i, ty);
                    let b = load_lane(f, s[1], i, ty);
                    let r = match wm {
                        Some(wm) => {
                            let on = mask_bit(f, wm.bits, i);
                            let off = if wm.zeroing {
                                konst(f, ty, 0)
                            } else {
                                a
                            };
                            select(f, ty, on, b, off)
                        }
                        None => b,
                    };
                    store_lane(f, tmp, i, ty, r);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::Pshufb => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                let idx_mask = konst(f, Ty::I64, block - 1);
                for blk in 0..width / block {
                    let base = lane_addr(f, s[0], blk * block);
                    for j in 0..block {
                        let ctl = load_lane(f, s[1], blk * block + j, Ty::I8);
                        let neg = is_neg(f, Ty::I8, ctl);
                        let at = resize(f, ctl, Ty::I8, Ty::I64, false);
                        let at = bin(f, BinOp::And, Ty::I64, at, idx_mask);
                        let v = load_dyn(f, base, at, Ty::I8);
                        let zero = konst(f, Ty::I8, 0);
                        let v = select(f, Ty::I8, neg, zero, v);
                        store_lane(f, tmp, blk * block + j, Ty::I8, v);
                    }
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::Perm(ty) if last_imm => {
                // permq/permpd imm: 64-bit lanes within each 256-bit half.
                if ty != Ty::I64 {
                    return err(span, format!("`{name}` does not take an immediate"));
                }
                let (s, k) = self.simd_args(f, ops, 1, true, span)?;
                for i in 0..width / 8 {
                    let base = i / 4 * 4;
                    let pick = (k >> (2 * (i % 4))) & 3;
                    let v = load_lane(f, s[0], base + pick, Ty::I64);
                    store_lane(f, tmp, i, Ty::I64, v);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::Perm(ty) => {
                // `perm* dst, index, table`
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                let n = width / ty.size();
                let m = konst(f, Ty::I64, n - 1);
                for i in 0..n {
                    let idx = load_lane(f, s[0], i, ty);
                    let idx = resize(f, idx, ty, Ty::I64, false);
                    let idx = bin(f, BinOp::And, Ty::I64, idx, m);
                    let v = load_dyn(f, s[1], idx, ty);
                    store_lane(f, tmp, i, ty, v);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::PermTwo(ty, index_in_dst) => {
                // permt2: dst = table1, (index, table2); permi2: dst = index, (table1, table2).
                let (s, _) = self.simd_args(f, ops, 3, false, span)?;
                let (idx, t1, t2) = if index_in_dst {
                    (s[0], s[1], s[2])
                } else {
                    (s[1], s[0], s[2])
                };
                let n = width / ty.size();
                let m = konst(f, Ty::I64, n - 1);
                let sel_bit = konst(f, Ty::I64, n);
                for i in 0..n {
                    let ix = load_lane(f, idx, i, ty);
                    let ix = resize(f, ix, ty, Ty::I64, false);
                    let second = bin(f, BinOp::And, Ty::I64, ix, sel_bit);
                    let zero = konst(f, Ty::I64, 0);
                    let second = cmp(f, CmpOp::Ne, Ty::I64, second, zero);
                    let at = bin(f, BinOp::And, Ty::I64, ix, m);
                    let a = load_dyn(f, t1, at, ty);
                    let b = load_dyn(f, t2, at, ty);
                    let v = select(f, ty, second, b, a);
                    store_lane(f, tmp, i, ty, v);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::PermilImm(ty) if !last_imm => {
                // permilps/pd with a vector control: `permilps dst, src, control`.
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                let per = 16 / ty.size();
                for i in 0..width / ty.size() {
                    let c = load_lane(f, s[1], i, ty);
                    let c = resize(f, c, ty, Ty::I64, false);
                    let c = if ty == Ty::I64 {
                        let one = konst(f, Ty::I64, 1);
                        bin(f, BinOp::LShr, Ty::I64, c, one)
                    } else {
                        c
                    };
                    let m = konst(f, Ty::I64, per - 1);
                    let c = bin(f, BinOp::And, Ty::I64, c, m);
                    let base = lane_addr(f, s[0], i / per * 16);
                    let v = load_dyn(f, base, c, ty);
                    store_lane(f, tmp, i, ty, v);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::PermilImm(ty) => {
                let (s, k) = self.simd_args(f, ops, 1, true, span)?;
                let per = 16 / ty.size();
                for i in 0..width / ty.size() {
                    let pick = if ty == Ty::I64 {
                        (k >> (i % 8)) & 1
                    } else {
                        (k >> (2 * (i % 4))) & 3
                    };
                    let v = load_lane(f, s[0], i / per * per + pick, ty);
                    store_lane(f, tmp, i, ty, v);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::Perm2x128 => {
                let (s, k) = self.simd_args(f, ops, 2, true, span)?;
                for half in 0..2 {
                    let sel = (k >> (4 * half)) & 0xf;
                    if sel & 8 == 0 {
                        let src = if sel & 2 == 0 {
                            s[0]
                        } else {
                            s[1]
                        };
                        let from = lane_addr(f, src, (sel & 1) * 16);
                        let to = lane_addr(f, tmp, half * 16);
                        f.b.copy(to, from, 16);
                    }
                }
                self.simd_out(f, cx, dst, tmp, 32, span)?;
            }
            SOp::Shuf128(_) => {
                let (s, k) = self.simd_args(f, ops, 2, true, span)?;
                let chunks = width / 16;
                let bits_per = if chunks == 2 {
                    1
                } else {
                    2
                };
                for c in 0..chunks {
                    let src = if c < chunks / 2 {
                        s[0]
                    } else {
                        s[1]
                    };
                    let pick = (k >> (bits_per * c)) & ((1 << bits_per) - 1);
                    let from = lane_addr(f, src, pick * 16);
                    let to = lane_addr(f, tmp, c * 16);
                    f.b.copy(to, from, 16);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::Insert(chunk, _) => {
                let (s, k) = self.simd_args(f, ops, 2, true, span)?;
                f.b.copy(tmp, s[0], width);
                let slots = (width / chunk).max(1);
                let at = lane_addr(f, tmp, (k % slots) * chunk);
                f.b.copy(at, s[1], chunk);
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::Extract(chunk, _) => {
                let (s, k) = self.simd_args(f, ops, 1, true, span)?;
                let slots = (width / chunk).max(1);
                let from = lane_addr(f, s[0], (k % slots) * chunk);
                f.b.copy(tmp, from, chunk);
                self.simd_out(f, cx, dst, tmp, chunk, span)?;
            }
            SOp::Pinsr(ty) => {
                // `pinsrb dst, [a,] src, imm`: src is a general-purpose register or memory.
                let (a, src, k) = match ops {
                    [_, src, VOpd::Imm(k)] => (self.vec_ptr(f, dst, span)?, *src, *k as u64),
                    [_, a, src, VOpd::Imm(k)] => (self.vec_ptr(f, *a, span)?, *src, *k as u64),
                    _ => return err(span, format!("`{name}` takes dst, [src,] value, imm8")),
                };
                let v = self.vec_scalar_read(f, src, ty, span)?;
                f.b.copy(tmp, a, 16);
                store_lane(f, tmp, k % (16 / ty.size()), ty, v);
                self.simd_out(f, cx, dst, tmp, 16, span)?;
            }
            SOp::Pextr(_) | SOp::Extractps => {
                let ty = match op {
                    SOp::Pextr(ty) => ty,
                    _ => Ty::I32,
                };
                let (src, k) = match ops {
                    [_, src, VOpd::Imm(k)] => (self.vec_ptr(f, *src, span)?, *k as u64),
                    _ => return err(span, format!("`{name}` takes dst, src, imm8")),
                };
                let v = load_lane(f, src, k % (16 / ty.size()), ty);
                match dst {
                    // Into a register the value is zero-extended (a 32-bit write below .q).
                    VOpd::Gpr(_) => {
                        let wide = if ty == Ty::I64 {
                            Ty::I64
                        } else {
                            Ty::I32
                        };
                        let v = resize(f, v, ty, wide, false);
                        self.vec_scalar_write(f, dst, wide, v, span)?;
                    }
                    _ => self.vec_scalar_write(f, dst, ty, v, span)?,
                }
            }
            SOp::Insertps => {
                let (s, k) = self.simd_args(f, ops, 2, true, span)?;
                let from_mem = matches!(ops[ops.len() - 2], VOpd::Mem(_));
                let pick = if from_mem {
                    0
                } else {
                    (k >> 6) & 3
                };
                f.b.copy(tmp, s[0], 16);
                let v = load_lane(f, s[1], pick, Ty::I32);
                store_lane(f, tmp, (k >> 4) & 3, Ty::I32, v);
                for j in 0..4 {
                    if k & (1 << j) != 0 {
                        let z = konst(f, Ty::I32, 0);
                        store_lane(f, tmp, j, Ty::I32, z);
                    }
                }
                self.simd_out(f, cx, dst, tmp, 16, span)?;
            }
            SOp::MovHalf(high) => {
                let at = if high {
                    8
                } else {
                    0
                };
                match ops {
                    // Store: `movhps [mem], src`.
                    [VOpd::Mem(_), src] => {
                        let src = self.vec_ptr(f, *src, span)?;
                        let p = self.vec_ptr(f, dst, span)?;
                        let from = lane_addr(f, src, at);
                        f.b.copy(p, from, 8);
                    }
                    // Load: `movhps dst, [a,] [mem]`.
                    _ => {
                        let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                        f.b.copy(tmp, s[0], 16);
                        let to = lane_addr(f, tmp, at);
                        f.b.copy(to, s[1], 8);
                        self.simd_out(f, cx, dst, tmp, 16, span)?;
                    }
                }
            }
            SOp::Movhlps | SOp::Movlhps => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                let hi = lane_addr(f, tmp, 8);
                if op == SOp::Movhlps {
                    let b_hi = lane_addr(f, s[1], 8);
                    let a_hi = lane_addr(f, s[0], 8);
                    f.b.copy(tmp, b_hi, 8);
                    f.b.copy(hi, a_hi, 8);
                } else {
                    f.b.copy(tmp, s[0], 8);
                    f.b.copy(hi, s[1], 8);
                }
                self.simd_out(f, cx, dst, tmp, 16, span)?;
            }
            SOp::Dup(kind) => {
                let (s, _) = self.simd_args(f, ops, 1, false, span)?;
                let (ty, pick) = match kind {
                    DupKind::Low64 => (Ty::I64, 0),
                    DupKind::EvenF32 => (Ty::I32, 0),
                    DupKind::OddF32 => (Ty::I32, 1),
                };
                for i in 0..width / ty.size() {
                    let v = load_lane(f, s[0], i / 2 * 2 + pick, ty);
                    store_lane(f, tmp, i, ty, v);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::Extend(from, to, signed) => {
                let (s, _) = self.simd_args(f, ops, 1, false, span)?;
                for i in 0..width / to.size() {
                    let v = load_lane(f, s[0], i, from);
                    let v = resize(f, v, from, to, signed);
                    store_lane(f, tmp, i, to, v);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::Narrow(from, to, sat) => {
                let (s, _) = self.simd_args(f, ops, 1, false, span)?;
                let n = width / from.size();
                for i in 0..n {
                    let v = load_lane(f, s[0], i, from);
                    let v = match sat {
                        Sat::Wrap => resize(f, v, from, to, false),
                        Sat::Signed => saturate(f, from, v, to, true),
                        Sat::Unsigned => {
                            let max = konst(f, from, (1u64 << bits(to)) - 1);
                            let over = cmp(f, CmpOp::UGt, from, v, max);
                            let v = select(f, from, over, max, v);
                            resize(f, v, from, to, false)
                        }
                    };
                    store_lane(f, tmp, i, to, v);
                }
                self.simd_out(f, cx, dst, tmp, n * to.size(), span)?;
            }
            SOp::Pack(from, unsigned) => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                let to = Ty::int(from.size() / 2);
                let per = block / from.size();
                for blk in 0..width / block {
                    for (h, src) in [s[0], s[1]].into_iter().enumerate() {
                        for j in 0..per {
                            let v = load_lane(f, src, blk * per + j, from);
                            let wide = if from == Ty::I16 {
                                Ty::I32
                            } else {
                                Ty::I64
                            };
                            let v = resize(f, v, from, wide, true);
                            let v = saturate(f, wide, v, to, !unsigned);
                            let at = blk * 2 * per + h as u64 * per + j;
                            store_lane(f, tmp, at, to, v);
                        }
                    }
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::Cvt(conv) => self.simd_convert(f, cx, inst, conv, ops, width, tmp)?,
            SOp::Half(to_half) => self.simd_half(f, cx, inst, to_half, ops, width, wm, tmp)?,
            SOp::Sha(sha) => self.simd_sha(f, cx, inst, sha, ops, tmp)?,
            SOp::Gf(gf) => self.simd_gf(f, cx, inst, gf, ops, width, tmp)?,
            SOp::FCmp(ty, scalar) => {
                let (s, k) = self.simd_args(f, ops, 2, true, span)?;
                let lanes = if scalar {
                    1
                } else {
                    width / ty.size()
                };
                let mut flags = Vec::new();
                if scalar {
                    f.b.copy(tmp, s[0], 16);
                }
                for i in 0..lanes {
                    let a = load_lane(f, s[0], i, ty);
                    let b = load_lane(f, s[1], i, ty);
                    let c = Self::float_predicate(f, ty, a, b, k & 0xf);
                    if matches!(dst, VOpd::Mask(_)) {
                        flags.push(c);
                    } else {
                        let it = Ty::int(ty.size());
                        let v = all_ones_if(f, it, c);
                        store_lane(f, tmp, i, it, v);
                    }
                }
                if matches!(dst, VOpd::Mask(_)) {
                    self.simd_mask_out(f, dst, flags, wm, span)?;
                } else {
                    self.simd_out(
                        f,
                        cx,
                        dst,
                        tmp,
                        if scalar {
                            16
                        } else {
                            width
                        },
                        span,
                    )?;
                }
            }
            SOp::IntCmp(ty, unsigned, fixed) => {
                let (s, k) = match fixed {
                    Some(p) => (self.simd_args(f, ops, 2, false, span)?.0, p as u64),
                    None => self.simd_args(f, ops, 2, true, span)?,
                };
                let mut flags = Vec::new();
                for i in 0..width / ty.size() {
                    let a = load_lane(f, s[0], i, ty);
                    let b = load_lane(f, s[1], i, ty);
                    let (lt, le) = if unsigned {
                        (CmpOp::ULt, CmpOp::ULe)
                    } else {
                        (CmpOp::SLt, CmpOp::SLe)
                    };
                    let c = match k & 7 {
                        0 => cmp(f, CmpOp::Eq, ty, a, b),
                        1 => cmp(f, lt, ty, a, b),
                        2 => cmp(f, le, ty, a, b),
                        3 => konst(f, Ty::I8, 0),
                        4 => cmp(f, CmpOp::Ne, ty, a, b),
                        5 => {
                            let c = cmp(f, lt, ty, a, b);
                            flag_not(f, c)
                        }
                        6 => {
                            let c = cmp(f, le, ty, a, b);
                            flag_not(f, c)
                        }
                        _ => konst(f, Ty::I8, 1),
                    };
                    flags.push(c);
                }
                self.simd_mask_out(f, dst, flags, wm, span)?;
            }
            SOp::Comis(ty) => {
                // `comiss a, b`: unordered 111, a < b 001, a == b 100, a > b 000 (ZF, PF, CF).
                if ops.len() != 2 {
                    return err(span, format!("`{name}` takes 2 operands"));
                }
                let pa = self.vec_ptr(f, ops[0], span)?;
                let pb = self.vec_ptr(f, ops[1], span)?;
                let a = load_lane(f, pa, 0, ty);
                let b = load_lane(f, pb, 0, ty);
                let ua = cmp(f, CmpOp::FNe, ty, a, a);
                let ub = cmp(f, CmpOp::FNe, ty, b, b);
                let uno = flag_or(f, ua, ub);
                let eq = cmp(f, CmpOp::FEq, ty, a, b);
                let lt = cmp(f, CmpOp::FLt, ty, a, b);
                let zero = konst(f, Ty::I8, 0);
                cx.flags = Flags {
                    zf: Some(flag_or(f, uno, eq)),
                    cf: Some(flag_or(f, uno, lt)),
                    // PF = unordered: an even-parity byte (0) sets it.
                    pf: Some(flag_not(f, uno)),
                    sf: Some(zero),
                    of: Some(zero),
                };
            }
            SOp::Ptest => {
                if ops.len() != 2 {
                    return err(span, format!("`{name}` takes 2 operands"));
                }
                let pa = self.vec_ptr(f, ops[0], span)?;
                let pb = self.vec_ptr(f, ops[1], span)?;
                let mut and = konst(f, Ty::I64, 0);
                let mut andn = konst(f, Ty::I64, 0);
                for i in 0..width / 8 {
                    let a = load_lane(f, pa, i, Ty::I64);
                    let b = load_lane(f, pb, i, Ty::I64);
                    let x = bin(f, BinOp::And, Ty::I64, a, b);
                    and = bin(f, BinOp::Or, Ty::I64, and, x);
                    let na = f.b.un(UnOp::Not, Ty::I64, a);
                    let y = bin(f, BinOp::And, Ty::I64, na, b);
                    andn = bin(f, BinOp::Or, Ty::I64, andn, y);
                }
                let zero = konst(f, Ty::I8, 0);
                let odd = konst(f, Ty::I8, 1);
                cx.flags = Flags {
                    zf: Some(is_zero(f, Ty::I64, and)),
                    cf: Some(is_zero(f, Ty::I64, andn)),
                    sf: Some(zero),
                    of: Some(zero),
                    pf: Some(odd),
                };
            }
            SOp::Testm(ty, not) => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                let mut flags = Vec::new();
                for i in 0..width / ty.size() {
                    let a = load_lane(f, s[0], i, ty);
                    let b = load_lane(f, s[1], i, ty);
                    let x = bin(f, BinOp::And, ty, a, b);
                    let z = is_zero(f, ty, x);
                    flags.push(if not {
                        z
                    } else {
                        flag_not(f, z)
                    });
                }
                self.simd_mask_out(f, dst, flags, wm, span)?;
            }
            SOp::VecToMask(ty) => {
                let (s, _) = self.simd_args(f, ops, 1, false, span)?;
                let mut flags = Vec::new();
                for i in 0..width / ty.size() {
                    let a = load_lane(f, s[0], i, ty);
                    flags.push(is_neg(f, ty, a));
                }
                self.simd_mask_out(f, dst, flags, wm, span)?;
            }
            SOp::MaskToVec(ty) | SOp::BroadcastMask(ty) => {
                let VOpd::Mask(p) = ops.get(1).copied().unwrap_or(dst) else {
                    return err(span, format!("`{name}` reads a mask register"));
                };
                if ops.len() != 2 {
                    return err(span, format!("`{name}` takes 2 operands"));
                }
                let m = f.b.load(Ty::I64, p);
                for i in 0..width / ty.size() {
                    let v = if op == SOp::MaskToVec(ty) {
                        let b = mask_bit(f, m, i);
                        all_ones_if(f, ty, b)
                    } else {
                        let low = if ty == Ty::I32 {
                            0xffff
                        } else {
                            0xff
                        };
                        let low = konst(f, Ty::I64, low);
                        let v = bin(f, BinOp::And, Ty::I64, m, low);
                        resize(f, v, Ty::I64, ty, false)
                    };
                    store_lane(f, tmp, i, ty, v);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::Compress(ty) => {
                let (s, _) = self.simd_args(f, ops, 1, false, span)?;
                let n = width / ty.size();
                let all = konst(f, Ty::I64, u64::MAX);
                let bits = wm.map_or(all, |w| w.bits);
                // Pack active lanes to the front of a scratch buffer.
                let mut pos = konst(f, Ty::I64, 0);
                for i in 0..n {
                    let v = load_lane(f, s[0], i, ty);
                    store_dyn(f, tmp, pos, ty, v);
                    let on = mask_bit(f, bits, i);
                    let on = resize(f, on, Ty::I8, Ty::I64, false);
                    pos = bin(f, BinOp::Add, Ty::I64, pos, on);
                }
                match dst {
                    VOpd::Mem(addr) => {
                        // Memory receives only the active elements.
                        let p = ptr_of(f, addr);
                        for j in 0..n {
                            let jv = konst(f, Ty::I64, j);
                            let inside = cmp(f, CmpOp::ULt, Ty::I64, jv, pos);
                            let v = load_lane(f, tmp, j, ty);
                            when(f, inside, |f| store_lane(f, p, j, ty, v));
                        }
                    }
                    _ => {
                        let old = self.vec_ptr(f, dst, span)?;
                        let zeroing = wm.is_some_and(|w| w.zeroing);
                        for j in 0..n {
                            let jv = konst(f, Ty::I64, j);
                            let inside = cmp(f, CmpOp::ULt, Ty::I64, jv, pos);
                            let v = load_lane(f, tmp, j, ty);
                            let keep = if zeroing {
                                konst(f, ty, 0)
                            } else {
                                load_lane(f, old, j, ty)
                            };
                            let r = select(f, ty, inside, v, keep);
                            store_lane(f, tmp, j, ty, r);
                        }
                        self.simd_out(f, cx, dst, tmp, width, span)?;
                    }
                }
            }
            SOp::Expand(ty) => {
                let (s, _) = self.simd_args(f, ops, 1, false, span)?;
                let n = width / ty.size();
                let all = konst(f, Ty::I64, u64::MAX);
                let bits = wm.map_or(all, |w| w.bits);
                let mut pos = konst(f, Ty::I64, 0);
                let slot = f.b.alloca(8, 8);
                for i in 0..n {
                    let on = mask_bit(f, bits, i);
                    // Only active lanes read the source (it may be memory ending early).
                    let zero = konst(f, ty, 0);
                    f.b.store(ty, slot, zero);
                    let src = s[0];
                    let at = pos;
                    when(f, on, |f| {
                        let v = load_dyn(f, src, at, ty);
                        f.b.store(ty, slot, v);
                    });
                    let v = f.b.load(ty, slot);
                    store_lane(f, tmp, i, ty, v);
                    let on = resize(f, on, Ty::I8, Ty::I64, false);
                    pos = bin(f, BinOp::Add, Ty::I64, pos, on);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::Ternlog(ty) => {
                let (s, k) = self.simd_args(f, ops, 3, true, span)?;
                for i in 0..width / 8 {
                    let a = load_lane(f, s[0], i, Ty::I64);
                    let b = load_lane(f, s[1], i, Ty::I64);
                    let c = load_lane(f, s[2], i, Ty::I64);
                    let na = f.b.un(UnOp::Not, Ty::I64, a);
                    let nb = f.b.un(UnOp::Not, Ty::I64, b);
                    let nc = f.b.un(UnOp::Not, Ty::I64, c);
                    let mut r = konst(f, Ty::I64, 0);
                    for t in 0..8u64 {
                        if (k >> t) & 1 == 0 {
                            continue;
                        }
                        // Truth-table row t = (a << 2) | (b << 1) | c.
                        let x = if t & 4 != 0 {
                            a
                        } else {
                            na
                        };
                        let y = if t & 2 != 0 {
                            b
                        } else {
                            nb
                        };
                        let z = if t & 1 != 0 {
                            c
                        } else {
                            nc
                        };
                        let term = bin(f, BinOp::And, Ty::I64, x, y);
                        let term = bin(f, BinOp::And, Ty::I64, term, z);
                        r = bin(f, BinOp::Or, Ty::I64, r, term);
                    }
                    store_lane(f, tmp, i, Ty::I64, r);
                }
                let _ = ty;
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::Dp(ty) => {
                let (s, k) = self.simd_args(f, ops, 2, true, span)?;
                let per = 16 / ty.size();
                for blk in 0..width / 16 {
                    let base = blk * per;
                    let mut terms = Vec::new();
                    for i in 0..per {
                        let a = load_lane(f, s[0], base + i, ty);
                        let b = load_lane(f, s[1], base + i, ty);
                        let p = bin(f, BinOp::FMul, ty, a, b);
                        let z = f.b.fconst(ty, 0.0);
                        terms.push(if (k >> (4 + i)) & 1 != 0 {
                            p
                        } else {
                            z
                        });
                    }
                    // (t0 + t1) + (t2 + t3), as the hardware adds.
                    let sum = if per == 4 {
                        let x = bin(f, BinOp::FAdd, ty, terms[0], terms[1]);
                        let y = bin(f, BinOp::FAdd, ty, terms[2], terms[3]);
                        bin(f, BinOp::FAdd, ty, x, y)
                    } else {
                        bin(f, BinOp::FAdd, ty, terms[0], terms[1])
                    };
                    for i in 0..per {
                        let v = if (k >> i) & 1 != 0 {
                            sum
                        } else {
                            f.b.fconst(ty, 0.0)
                        };
                        store_lane(f, tmp, base + i, ty, v);
                    }
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::Pclmul(fixed) => {
                let (s, k) = match fixed {
                    Some(k) => (self.simd_args(f, ops, 2, false, span)?.0, k as u64),
                    None => self.simd_args(f, ops, 2, true, span)?,
                };
                for blk in 0..width / 16 {
                    let a = load_lane(f, s[0], 2 * blk + (k & 1), Ty::I64);
                    let b = load_lane(f, s[1], 2 * blk + ((k >> 4) & 1), Ty::I64);
                    let (lo, hi) = Self::clmul64(f, a, b);
                    store_lane(f, tmp, 2 * blk, Ty::I64, lo);
                    store_lane(f, tmp, 2 * blk + 1, Ty::I64, hi);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            SOp::Aes(aes) => {
                let (s, k) = match aes {
                    AesOp::Imc => self.simd_args(f, ops, 1, false, span)?,
                    AesOp::KeygenAssist => self.simd_args(f, ops, 1, true, span)?,
                    _ => self.simd_args(f, ops, 2, false, span)?,
                };
                let tables = self.aes_table_ptr(f);
                let inv_tables = f.b.ptr_offset(tables, 256);
                let blocks = if matches!(aes, AesOp::Imc | AesOp::KeygenAssist) {
                    1
                } else {
                    width / 16
                };
                for blk in 0..blocks {
                    let base = blk * 16;
                    let mut st = [Val(0); 16];
                    for (j, b) in st.iter_mut().enumerate() {
                        *b = load_lane(f, s[0], base + j as u64, Ty::I8);
                    }
                    let sub = |f: &mut FnCtx, table: Val, v: Val| {
                        let at = resize(f, v, Ty::I8, Ty::I64, false);
                        load_dyn(f, table, at, Ty::I8)
                    };
                    let out = match aes {
                        AesOp::Imc => mix_columns(f, &st, true),
                        AesOp::KeygenAssist => {
                            // [Sub(X1), Rot(Sub(X1)) ^ rcon, Sub(X3), Rot(Sub(X3)) ^ rcon]
                            let mut out = [Val(0); 16];
                            for (w, x) in [(0usize, 1usize), (2, 3)] {
                                let word: Vec<Val> =
                                    (0..4).map(|b| sub(f, tables, st[4 * x + b])).collect();
                                for b in 0..4 {
                                    out[4 * w + b] = word[b];
                                    out[4 * (w + 1) + b] = word[(b + 1) % 4];
                                }
                                let rcon = konst(f, Ty::I8, k);
                                out[4 * (w + 1)] = xor8(f, out[4 * (w + 1)], rcon);
                            }
                            out
                        }
                        _ => {
                            let enc = matches!(aes, AesOp::Enc | AesOp::EncLast);
                            // (Inv)ShiftRows + (Inv)SubBytes: row r rotates by r columns.
                            let mut sh = [Val(0); 16];
                            for c in 0..4 {
                                for r in 0..4 {
                                    let from = if enc {
                                        (c + r) % 4
                                    } else {
                                        (c + 4 - r) % 4
                                    };
                                    let table = if enc {
                                        tables
                                    } else {
                                        inv_tables
                                    };
                                    sh[4 * c + r] = sub(f, table, st[4 * from + r]);
                                }
                            }
                            match aes {
                                AesOp::Enc => mix_columns(f, &sh, false),
                                AesOp::Dec => mix_columns(f, &sh, true),
                                _ => sh,
                            }
                        }
                    };
                    for (j, &v) in out.iter().enumerate() {
                        let v = if matches!(aes, AesOp::Imc | AesOp::KeygenAssist) {
                            v
                        } else {
                            let key = load_lane(f, s[1], base + j as u64, Ty::I8);
                            xor8(f, v, key)
                        };
                        store_lane(f, tmp, base + j as u64, Ty::I8, v);
                    }
                }
                self.simd_out(f, cx, dst, tmp, blocks * 16, span)?;
            }
            SOp::Scatter(index_size, elem_size) => {
                let (
                    VOpd::Vsib {
                        base,
                        index,
                        scale,
                    },
                    Some(src),
                ) = (dst, ops.get(1))
                else {
                    return err(
                        span,
                        format!("`{name}` takes [base + vindex*scale] &mask, src"),
                    );
                };
                let Some(wm) = wm else {
                    return err(span, format!("`{name}` needs a mask (&k)"));
                };
                let src = self.vec_ptr(f, *src, span)?;
                let (it, et) = (Ty::int(index_size), Ty::int(elem_size));
                let lanes = width / index_size.max(elem_size);
                let scale = konst(f, Ty::I64, scale);
                for i in 0..lanes {
                    let on = mask_bit(f, wm.bits, i);
                    let idx = load_lane(f, index, i, it);
                    let idx = resize(f, idx, it, Ty::I64, true);
                    let off = bin(f, BinOp::Mul, Ty::I64, idx, scale);
                    let addr = bin(f, BinOp::Add, Ty::I64, base, off);
                    let v = load_lane(f, src, i, et);
                    when(f, on, |f| {
                        let p = ptr_of(f, addr);
                        f.b.store(et, p, v);
                    });
                }
                // The mask register is cleared once every element is stored.
                if let Some(reg) = wm.reg {
                    let z = konst(f, Ty::I64, 0);
                    f.b.store(Ty::I64, reg, z);
                }
            }
            SOp::MaskMov(ty) => {
                if ops.len() != 3 {
                    return err(span, format!("`{name}` takes 3 operands"));
                }
                let mask = self.vec_ptr(f, ops[1], span)?;
                let n = width / ty.size();
                match (ops[0], ops[2]) {
                    (VOpd::Mem(addr), src) => {
                        let src = self.vec_ptr(f, src, span)?;
                        let p = ptr_of(f, addr);
                        for i in 0..n {
                            let m = load_lane(f, mask, i, ty);
                            let on = is_neg(f, ty, m);
                            let v = load_lane(f, src, i, ty);
                            when(f, on, |f| store_lane(f, p, i, ty, v));
                        }
                    }
                    (_, VOpd::Mem(addr)) => {
                        let p = ptr_of(f, addr);
                        let slot = f.b.alloca(8, 8);
                        for i in 0..n {
                            let m = load_lane(f, mask, i, ty);
                            let on = is_neg(f, ty, m);
                            let zero = konst(f, ty, 0);
                            f.b.store(ty, slot, zero);
                            when(f, on, |f| {
                                let v = load_lane(f, p, i, ty);
                                f.b.store(ty, slot, v);
                            });
                            let v = f.b.load(ty, slot);
                            store_lane(f, tmp, i, ty, v);
                        }
                        self.simd_out(f, cx, dst, tmp, width, span)?;
                    }
                    _ => {
                        return err(
                            span,
                            format!("`{name}` moves between a register and memory"),
                        );
                    }
                }
            }
        }
        Ok(())
    }

    fn simd_bin(f: &mut FnCtx, op: B2, ty: Ty, a: Val, b: Val) -> Val {
        let n = bits(ty);
        match op {
            B2::AddSatS | B2::AddSatU | B2::SubSatS | B2::SubSatU => {
                let signed = matches!(op, B2::AddSatS | B2::SubSatS);
                let wa = resize(f, a, ty, Ty::I32, signed);
                let wb = resize(f, b, ty, Ty::I32, signed);
                let r = if matches!(op, B2::AddSatS | B2::AddSatU) {
                    bin(f, BinOp::Add, Ty::I32, wa, wb)
                } else {
                    bin(f, BinOp::Sub, Ty::I32, wa, wb)
                };
                saturate(f, Ty::I32, r, ty, signed)
            }
            B2::Avg => {
                let wa = resize(f, a, ty, Ty::I32, false);
                let wb = resize(f, b, ty, Ty::I32, false);
                let s = bin(f, BinOp::Add, Ty::I32, wa, wb);
                let one = konst(f, Ty::I32, 1);
                let s = bin(f, BinOp::Add, Ty::I32, s, one);
                let s = bin(f, BinOp::LShr, Ty::I32, s, one);
                resize(f, s, Ty::I32, ty, false)
            }
            B2::MulHiS | B2::MulHiU | B2::MulHrs => {
                let signed = op != B2::MulHiU;
                let wa = resize(f, a, ty, Ty::I32, signed);
                let wb = resize(f, b, ty, Ty::I32, signed);
                let p = bin(f, BinOp::Mul, Ty::I32, wa, wb);
                let r = if op == B2::MulHrs {
                    let s14 = konst(f, Ty::I32, 14);
                    let one = konst(f, Ty::I32, 1);
                    let t = bin(f, BinOp::AShr, Ty::I32, p, s14);
                    let t = bin(f, BinOp::Add, Ty::I32, t, one);
                    bin(f, BinOp::AShr, Ty::I32, t, one)
                } else {
                    let s16 = konst(f, Ty::I32, 16);
                    bin(f, BinOp::LShr, Ty::I32, p, s16)
                };
                resize(f, r, Ty::I32, ty, false)
            }
            B2::Sign => {
                let neg = is_neg(f, ty, b);
                let zero_b = is_zero(f, ty, b);
                let zero = konst(f, ty, 0);
                let na = bin(f, BinOp::Sub, ty, zero, a);
                let r = select(f, ty, neg, na, a);
                select(f, ty, zero_b, zero, r)
            }
            B2::Shlv | B2::Shrv | B2::Sarv => {
                // Counts at or above the lane width clear the lane (fill with the sign for sar).
                let max = konst(f, ty, n - 1);
                let over = cmp(f, CmpOp::UGt, ty, b, max);
                if op == B2::Sarv {
                    let c = select(f, ty, over, max, b);
                    bin(f, BinOp::AShr, ty, a, c)
                } else {
                    let r = bin(
                        f,
                        if op == B2::Shlv {
                            BinOp::Shl
                        } else {
                            BinOp::LShr
                        },
                        ty,
                        a,
                        b,
                    );
                    let zero = konst(f, ty, 0);
                    select(f, ty, over, zero, r)
                }
            }
            B2::Rolv | B2::Rorv => {
                let m = konst(f, ty, n - 1);
                let c = bin(f, BinOp::And, ty, b, m);
                bin(
                    f,
                    if op == B2::Rolv {
                        BinOp::Rotl
                    } else {
                        BinOp::Rotr
                    },
                    ty,
                    a,
                    c,
                )
            }
        }
    }

    /// `cmpps` predicate `p` (0..15; bit 4 only changes signaling, not the result).
    fn float_predicate(f: &mut FnCtx, ty: Ty, a: Val, b: Val, p: u64) -> Val {
        let ua = cmp(f, CmpOp::FNe, ty, a, a);
        let ub = cmp(f, CmpOp::FNe, ty, b, b);
        let uno = flag_or(f, ua, ub);
        let eq = cmp(f, CmpOp::FEq, ty, a, b);
        match p {
            0 => eq,
            1 => cmp(f, CmpOp::FLt, ty, a, b),
            2 => cmp(f, CmpOp::FLe, ty, a, b),
            3 => uno,
            4 => flag_not(f, eq),
            5 => {
                let c = cmp(f, CmpOp::FLt, ty, a, b);
                flag_not(f, c)
            }
            6 => {
                let c = cmp(f, CmpOp::FLe, ty, a, b);
                flag_not(f, c)
            }
            7 => flag_not(f, uno),
            8 => flag_or(f, eq, uno),
            9 => {
                let c = cmp(f, CmpOp::FGe, ty, a, b);
                flag_not(f, c)
            }
            10 => {
                let c = cmp(f, CmpOp::FGt, ty, a, b);
                flag_not(f, c)
            }
            11 => konst(f, Ty::I8, 0),
            12 => {
                let ne = flag_not(f, eq);
                let ord = flag_not(f, uno);
                bin(f, BinOp::And, Ty::I8, ne, ord)
            }
            13 => cmp(f, CmpOp::FGe, ty, a, b),
            14 => cmp(f, CmpOp::FGt, ty, a, b),
            _ => konst(f, Ty::I8, 1),
        }
    }

    /// Carry-less 64x64 -> 128 multiply: (low, high), as a 64-step loop.
    fn clmul64(f: &mut FnCtx, a: Val, b: Val) -> (Val, Val) {
        let lo_s = f.b.alloca(8, 8);
        let hi_s = f.b.alloca(8, 8);
        let i_s = f.b.alloca(8, 8);
        let zero = konst(f, Ty::I64, 0);
        f.b.store(Ty::I64, lo_s, zero);
        f.b.store(Ty::I64, hi_s, zero);
        f.b.store(Ty::I64, i_s, zero);
        let head = f.b.new_block();
        let body = f.b.new_block();
        let done = f.b.new_block();
        f.b.jump(head);
        f.b.switch_to(head);
        let i = f.b.load(Ty::I64, i_s);
        let n = konst(f, Ty::I64, 64);
        let end = cmp(f, CmpOp::Eq, Ty::I64, i, n);
        f.b.branch(end, done, body);
        f.b.switch_to(body);
        let one = konst(f, Ty::I64, 1);
        let zero = konst(f, Ty::I64, 0);
        let bit = bin(f, BinOp::LShr, Ty::I64, b, i);
        let bit = bin(f, BinOp::And, Ty::I64, bit, one);
        let on = cmp(f, CmpOp::Ne, Ty::I64, bit, zero);
        let lo_part = bin(f, BinOp::Shl, Ty::I64, a, i);
        // a >> (64 - i), which is 0 for i == 0 (IR shifts are total).
        let n = konst(f, Ty::I64, 64);
        let back = bin(f, BinOp::Sub, Ty::I64, n, i);
        let hi_part = bin(f, BinOp::LShr, Ty::I64, a, back);
        let lo_part = select(f, Ty::I64, on, lo_part, zero);
        let hi_part = select(f, Ty::I64, on, hi_part, zero);
        let lo = f.b.load(Ty::I64, lo_s);
        let hi = f.b.load(Ty::I64, hi_s);
        let lo = bin(f, BinOp::Xor, Ty::I64, lo, lo_part);
        let hi = bin(f, BinOp::Xor, Ty::I64, hi, hi_part);
        f.b.store(Ty::I64, lo_s, lo);
        f.b.store(Ty::I64, hi_s, hi);
        let i = bin(f, BinOp::Add, Ty::I64, i, one);
        f.b.store(Ty::I64, i_s, i);
        f.b.jump(head);
        f.b.switch_to(done);
        (f.b.load(Ty::I64, lo_s), f.b.load(Ty::I64, hi_s))
    }

    /// Address of the S-box (256 bytes), then the inverse S-box and the field inverse.
    fn aes_table_ptr(&mut self, f: &mut FnCtx) -> Val {
        let global = match self.asm_aes_tables {
            Some(g) => g,
            None => {
                let g = self.program.add_global(crate::ir::Global {
                    name: "__jaic_asm_aes_sbox".into(),
                    size: 768,
                    align: 16,
                    init: aes_tables(),
                    relocs: Vec::new(),
                    read_only: true,
                    export: None,
                });
                self.asm_aes_tables = Some(g);
                g
            }
        };
        f.b.global_addr(global)
    }

    /// Conversions (`Conv`).
    #[allow(clippy::too_many_arguments)]
    fn simd_convert(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        inst: &AsmInst,
        conv: Conv,
        ops: &[VOpd],
        width: u64,
        tmp: Val,
    ) -> Result<()> {
        let span = inst.span;
        let name = inst.mnemonic.name.as_str();
        let dst = ops[0];
        let round = inst.evex.round.flatten();
        // The scalar/general-purpose forms size their integer by the .d/.q suffix.
        let int_size = || match &inst.size {
            Some(AsmSize::Suffix(s)) if matches!(s.name.as_str(), "q" | "64") => Ty::I64,
            _ => Ty::I32,
        };
        match conv {
            Conv::IntToFloat(from, to, unsigned) => {
                let (s, _) = self.simd_args(f, ops, 1, false, span)?;
                let n = width / from.size().max(to.size());
                for i in 0..n {
                    let x = load_lane(f, s[0], i, from);
                    let op = if unsigned {
                        ConvOp::UToF
                    } else {
                        ConvOp::SToF
                    };
                    let r = f.b.conv(op, from, to, x);
                    store_lane(f, tmp, i, to, r);
                }
                self.simd_out(f, cx, dst, tmp, n * to.size(), span)
            }
            Conv::FloatToInt(from, to, trunc, unsigned) => {
                let (s, _) = self.simd_args(f, ops, 1, false, span)?;
                let mode = match (round, trunc) {
                    (Some(m), _) => m,
                    (None, true) => 'z',
                    _ => 'n',
                };
                let n = width / from.size().max(to.size());
                for i in 0..n {
                    let x = load_lane(f, s[0], i, from);
                    let r = float_to_int(f, from, x, to, mode, unsigned);
                    store_lane(f, tmp, i, to, r);
                }
                self.simd_out(f, cx, dst, tmp, n * to.size(), span)
            }
            Conv::FloatToFloat(from, to) => {
                let (s, _) = self.simd_args(f, ops, 1, false, span)?;
                let n = width / from.size().max(to.size());
                for i in 0..n {
                    let x = load_lane(f, s[0], i, from);
                    let op = if from == Ty::F32 {
                        ConvOp::FExt
                    } else {
                        ConvOp::FTrunc
                    };
                    let r = f.b.conv(op, from, to, x);
                    store_lane(f, tmp, i, to, r);
                }
                self.simd_out(f, cx, dst, tmp, n * to.size(), span)
            }
            Conv::ScalarFloat(from, to) => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                f.b.copy(tmp, s[0], 16);
                let x = load_lane(f, s[1], 0, from);
                let op = if from == Ty::F32 {
                    ConvOp::FExt
                } else {
                    ConvOp::FTrunc
                };
                let r = f.b.conv(op, from, to, x);
                store_lane(f, tmp, 0, to, r);
                self.simd_out(f, cx, dst, tmp, 16, span)
            }
            Conv::GprToScalar(to, unsigned) => {
                // `cvtsi2ss.q dst, [a,] src`: the upper lanes come from `a` (or dst).
                let (a, src) = match ops {
                    [_, src] => (self.vec_ptr(f, dst, span)?, *src),
                    [_, a, src] => (self.vec_ptr(f, *a, span)?, *src),
                    _ => return err(span, format!("`{name}` takes dst, [src,] integer")),
                };
                let from = int_size();
                let x = self.vec_scalar_read(f, src, from, span)?;
                let op = if unsigned {
                    ConvOp::UToF
                } else {
                    ConvOp::SToF
                };
                let r = f.b.conv(op, from, to, x);
                f.b.copy(tmp, a, 16);
                store_lane(f, tmp, 0, to, r);
                self.simd_out(f, cx, dst, tmp, 16, span)
            }
            Conv::ScalarToGpr(from, trunc, unsigned) => {
                if ops.len() != 2 {
                    return err(span, format!("`{name}` takes 2 operands"));
                }
                let src = self.vec_ptr(f, ops[1], span)?;
                let to = int_size();
                let mode = match (round, trunc) {
                    (Some(m), _) => m,
                    (None, true) => 'z',
                    _ => 'n',
                };
                let x = load_lane(f, src, 0, from);
                let r = float_to_int(f, from, x, to, mode, unsigned);
                self.vec_scalar_write(f, dst, to, r, span)
            }
        }
    }
}
