//! Lane sizes, mnemonic tables and the lookups from names to operations.
use super::ext::{GfOp, ShaOp};
use super::{AesOp, B2, Conv, DupKind, FmaKind, SOp, Sat, U1};
use crate::sema::asm::*;

pub(in crate::sema::asm) fn elem_size(op: SOp) -> u64 {
    use SOp::{
        AddSub, Aes, Bin, BlendImm, BlendMask, BlendVar, BroadcastMask, ByteShift, Compress, Cvt,
        DotAcc, Dp, Expand, Extend, Extract, FCmp, Fma, Gf, Half, Horizontal, Insert, IntCmp,
        Madd52, MaskMov, MaskToVec, MovHalf, Mpsadbw, MulEven, Narrow, Pack, Palignr, Pclmul, Perm,
        PermTwo, PermilImm, Pmaddubsw, Pmaddwd, Psadbw, Pshufb, RotImm, Round, Scatter, Sha,
        Shuf128, ShufHalf, Shufpd, Ternlog, Testm, Unary, Unpack, Valign, VecToMask,
    };
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
        Pmaddubsw | ShufHalf(_) | Mpsadbw => 2,
        Psadbw | MulEven(_) | Shufpd | Pclmul(_) | MovHalf(_) | Madd52(_) => 8,
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
pub(in crate::sema::asm) fn dst_class(op: SOp, width: u64) -> &'static str {
    match op {
        SOp::IntCmp(..) | SOp::Testm(..) | SOp::VecToMask(_) => "omr",
        SOp::FCmp(..) if width == 64 => "omr",
        SOp::Pextr(_)
        | SOp::Extractps
        | SOp::Cvt(Conv::ScalarToGpr(..))
        | SOp::Pcmpstr(_, false) => "gpr",
        _ => "vec",
    }
}

/// Instructions that have a 64-bit MMX (`.q`) form.
pub(in crate::sema::asm) fn mmx_capable(op: SOp) -> bool {
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
pub(in crate::sema::asm) fn consumes_mask(op: SOp) -> bool {
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
pub(in crate::sema::asm) fn broadcast_size(op: SOp) -> u64 {
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

/// Instructions whose name alone says what they do.
const NAMED: &[(&str, SOp)] = {
    use SOp::{
        AddSub, Aes, Bin, BlendImm, BlendMask, BlendVar, BroadcastMask, ByteShift, Comis, Compress,
        Cvt, DotAcc, Dp, Dup, Expand, Extract, Extractps, Gf, Half, Horizontal, Insert, Insertps,
        Madd52, MaskMov, MovHalf, Movhlps, Movlhps, Mpsadbw, MulEven, Pack, Palignr, Pclmul,
        Pcmpstr, Perm, Perm2x128, PermTwo, PermilImm, Phminposuw, Pmaddubsw, Pmaddwd, Psadbw,
        Pshufb, Ptest, Scatter, Sha, Shuf128, ShufHalf, Shufpd, Ternlog, Unpack, Valign,
    };
    &[
        ("paddsb", Bin(B2::AddSatS, Ty::I8)),
        ("paddsw", Bin(B2::AddSatS, Ty::I16)),
        ("paddusb", Bin(B2::AddSatU, Ty::I8)),
        ("paddusw", Bin(B2::AddSatU, Ty::I16)),
        ("psubsb", Bin(B2::SubSatS, Ty::I8)),
        ("psubsw", Bin(B2::SubSatS, Ty::I16)),
        ("psubusb", Bin(B2::SubSatU, Ty::I8)),
        ("psubusw", Bin(B2::SubSatU, Ty::I16)),
        ("pavgb", Bin(B2::Avg, Ty::I8)),
        ("pavgw", Bin(B2::Avg, Ty::I16)),
        ("pmulhw", Bin(B2::MulHiS, Ty::I16)),
        ("pmulhuw", Bin(B2::MulHiU, Ty::I16)),
        ("pmulhrsw", Bin(B2::MulHrs, Ty::I16)),
        ("pmuludq", MulEven(false)),
        ("pmuldq", MulEven(true)),
        ("pmaddwd", Pmaddwd),
        ("pmaddubsw", Pmaddubsw),
        ("psadbw", Psadbw),
        ("mpsadbw", Mpsadbw),
        ("pmadd52luq", Madd52(false)),
        ("pmadd52huq", Madd52(true)),
        ("pdpbusd", DotAcc(Ty::I8)),
        ("pdpwssd", DotAcc(Ty::I16)),
        ("phminposuw", Phminposuw),
        ("phaddw", Horizontal(false, false, Ty::I16)),
        ("phaddd", Horizontal(false, false, Ty::I32)),
        ("phaddsw", Horizontal(false, true, Ty::I16)),
        ("phsubw", Horizontal(true, false, Ty::I16)),
        ("phsubd", Horizontal(true, false, Ty::I32)),
        ("phsubsw", Horizontal(true, true, Ty::I16)),
        ("haddps", Horizontal(false, false, Ty::F32)),
        ("haddpd", Horizontal(false, false, Ty::F64)),
        ("hsubps", Horizontal(true, false, Ty::F32)),
        ("hsubpd", Horizontal(true, false, Ty::F64)),
        ("addsubps", AddSub(Ty::F32)),
        ("addsubpd", AddSub(Ty::F64)),
        ("unpcklps", Unpack(false, Ty::I32)),
        ("unpckhps", Unpack(true, Ty::I32)),
        ("unpcklpd", Unpack(false, Ty::I64)),
        ("unpckhpd", Unpack(true, Ty::I64)),
        ("punpcklqdq", Unpack(false, Ty::I64)),
        ("punpckhqdq", Unpack(true, Ty::I64)),
        ("pshufhw", ShufHalf(true)),
        ("pshuflw", ShufHalf(false)),
        ("shufpd", Shufpd),
        ("palignr", Palignr),
        ("alignd", Valign(Ty::I32)),
        ("valignd", Valign(Ty::I32)),
        ("alignq", Valign(Ty::I64)),
        ("valignq", Valign(Ty::I64)),
        ("pslldq", ByteShift(true)),
        ("psrldq", ByteShift(false)),
        ("pblendw", BlendImm(Ty::I16)),
        ("pblendd", BlendImm(Ty::I32)),
        ("blendps", BlendImm(Ty::I32)),
        ("blendpd", BlendImm(Ty::I64)),
        ("pblendvb", BlendVar(Ty::I8)),
        ("blendvps", BlendVar(Ty::I32)),
        ("blendvpd", BlendVar(Ty::I64)),
        ("blendmps", BlendMask(Ty::I32)),
        ("blendmpd", BlendMask(Ty::I64)),
        ("pshufb", Pshufb),
        ("permps", Perm(Ty::I32)),
        ("permpd", Perm(Ty::I64)),
        ("permilps", PermilImm(Ty::I32)),
        ("permilpd", PermilImm(Ty::I64)),
        ("perm2i128", Perm2x128),
        ("perm2f128", Perm2x128),
        ("shufi32x4", Shuf128(Ty::I32)),
        ("shuff32x4", Shuf128(Ty::I32)),
        ("shufi64x2", Shuf128(Ty::I64)),
        ("shuff64x2", Shuf128(Ty::I64)),
        ("inserti128", Insert(16, Ty::I32)),
        ("insertf128", Insert(16, Ty::I32)),
        ("inserti32x4", Insert(16, Ty::I32)),
        ("insertf32x4", Insert(16, Ty::I32)),
        ("inserti64x2", Insert(16, Ty::I64)),
        ("insertf64x2", Insert(16, Ty::I64)),
        ("inserti32x8", Insert(32, Ty::I32)),
        ("insertf32x8", Insert(32, Ty::I32)),
        ("inserti64x4", Insert(32, Ty::I64)),
        ("insertf64x4", Insert(32, Ty::I64)),
        ("extracti128", Extract(16, Ty::I32)),
        ("extractf128", Extract(16, Ty::I32)),
        ("extracti32x4", Extract(16, Ty::I32)),
        ("extractf32x4", Extract(16, Ty::I32)),
        ("extracti64x2", Extract(16, Ty::I64)),
        ("extractf64x2", Extract(16, Ty::I64)),
        ("extracti32x8", Extract(32, Ty::I32)),
        ("extractf32x8", Extract(32, Ty::I32)),
        ("extracti64x4", Extract(32, Ty::I64)),
        ("extractf64x4", Extract(32, Ty::I64)),
        ("insertps", Insertps),
        ("extractps", Extractps),
        ("movhps", MovHalf(true)),
        ("movhpd", MovHalf(true)),
        ("movlps", MovHalf(false)),
        ("movlpd", MovHalf(false)),
        ("movhlps", Movhlps),
        ("movlhps", Movlhps),
        ("movddup", Dup(DupKind::Low64)),
        ("movshdup", Dup(DupKind::OddF32)),
        ("movsldup", Dup(DupKind::EvenF32)),
        ("packsswb", Pack(Ty::I16, false)),
        ("packssdw", Pack(Ty::I32, false)),
        ("packuswb", Pack(Ty::I16, true)),
        ("packusdw", Pack(Ty::I32, true)),
        ("cvtdq2pd", Cvt(Conv::IntToFloat(Ty::I32, Ty::F64, false))),
        ("cvtudq2ps", Cvt(Conv::IntToFloat(Ty::I32, Ty::F32, true))),
        ("cvtudq2pd", Cvt(Conv::IntToFloat(Ty::I32, Ty::F64, true))),
        ("cvtqq2ps", Cvt(Conv::IntToFloat(Ty::I64, Ty::F32, false))),
        ("cvtqq2pd", Cvt(Conv::IntToFloat(Ty::I64, Ty::F64, false))),
        ("cvtuqq2ps", Cvt(Conv::IntToFloat(Ty::I64, Ty::F32, true))),
        ("cvtuqq2pd", Cvt(Conv::IntToFloat(Ty::I64, Ty::F64, true))),
        (
            "cvtpd2dq",
            Cvt(Conv::FloatToInt(Ty::F64, Ty::I32, false, false)),
        ),
        (
            "cvttpd2dq",
            Cvt(Conv::FloatToInt(Ty::F64, Ty::I32, true, false)),
        ),
        (
            "cvtps2udq",
            Cvt(Conv::FloatToInt(Ty::F32, Ty::I32, false, true)),
        ),
        (
            "cvttps2udq",
            Cvt(Conv::FloatToInt(Ty::F32, Ty::I32, true, true)),
        ),
        (
            "cvtpd2udq",
            Cvt(Conv::FloatToInt(Ty::F64, Ty::I32, false, true)),
        ),
        (
            "cvttpd2udq",
            Cvt(Conv::FloatToInt(Ty::F64, Ty::I32, true, true)),
        ),
        (
            "cvtps2qq",
            Cvt(Conv::FloatToInt(Ty::F32, Ty::I64, false, false)),
        ),
        (
            "cvttps2qq",
            Cvt(Conv::FloatToInt(Ty::F32, Ty::I64, true, false)),
        ),
        (
            "cvtpd2qq",
            Cvt(Conv::FloatToInt(Ty::F64, Ty::I64, false, false)),
        ),
        (
            "cvttpd2qq",
            Cvt(Conv::FloatToInt(Ty::F64, Ty::I64, true, false)),
        ),
        (
            "cvtps2uqq",
            Cvt(Conv::FloatToInt(Ty::F32, Ty::I64, false, true)),
        ),
        (
            "cvttps2uqq",
            Cvt(Conv::FloatToInt(Ty::F32, Ty::I64, true, true)),
        ),
        (
            "cvtpd2uqq",
            Cvt(Conv::FloatToInt(Ty::F64, Ty::I64, false, true)),
        ),
        (
            "cvttpd2uqq",
            Cvt(Conv::FloatToInt(Ty::F64, Ty::I64, true, true)),
        ),
        ("cvtps2pd", Cvt(Conv::FloatToFloat(Ty::F32, Ty::F64))),
        ("cvtpd2ps", Cvt(Conv::FloatToFloat(Ty::F64, Ty::F32))),
        ("cvtss2sd", Cvt(Conv::ScalarFloat(Ty::F32, Ty::F64))),
        ("cvtsd2ss", Cvt(Conv::ScalarFloat(Ty::F64, Ty::F32))),
        ("cvtsi2ss", Cvt(Conv::GprToScalar(Ty::F32, false))),
        ("cvtsi2sd", Cvt(Conv::GprToScalar(Ty::F64, false))),
        ("cvtusi2ss", Cvt(Conv::GprToScalar(Ty::F32, true))),
        ("cvtusi2sd", Cvt(Conv::GprToScalar(Ty::F64, true))),
        ("cvtss2si", Cvt(Conv::ScalarToGpr(Ty::F32, false, false))),
        ("cvttss2si", Cvt(Conv::ScalarToGpr(Ty::F32, true, false))),
        ("cvtsd2si", Cvt(Conv::ScalarToGpr(Ty::F64, false, false))),
        ("cvttsd2si", Cvt(Conv::ScalarToGpr(Ty::F64, true, false))),
        ("cvtss2usi", Cvt(Conv::ScalarToGpr(Ty::F32, false, true))),
        ("cvttss2usi", Cvt(Conv::ScalarToGpr(Ty::F32, true, true))),
        ("cvtsd2usi", Cvt(Conv::ScalarToGpr(Ty::F64, false, true))),
        ("cvttsd2usi", Cvt(Conv::ScalarToGpr(Ty::F64, true, true))),
        ("comiss", Comis(Ty::F32)),
        ("ucomiss", Comis(Ty::F32)),
        ("comisd", Comis(Ty::F64)),
        ("ucomisd", Comis(Ty::F64)),
        ("ptest", Ptest),
        ("pcmpistri", Pcmpstr(false, false)),
        ("pcmpestri", Pcmpstr(true, false)),
        ("pcmpistrm", Pcmpstr(false, true)),
        ("pcmpestrm", Pcmpstr(true, true)),
        ("pternlogd", Ternlog(Ty::I32)),
        ("pternlogq", Ternlog(Ty::I64)),
        ("dpps", Dp(Ty::F32)),
        ("dppd", Dp(Ty::F64)),
        ("pclmulqdq", Pclmul(None)),
        ("pclmullqlqdq", Pclmul(Some(0x00))),
        ("pclmulhqlqdq", Pclmul(Some(0x01))),
        ("pclmullqhqdq", Pclmul(Some(0x10))),
        ("pclmulhqhqdq", Pclmul(Some(0x11))),
        ("aesenc", Aes(AesOp::Enc)),
        ("aesenclast", Aes(AesOp::EncLast)),
        ("aesdec", Aes(AesOp::Dec)),
        ("aesdeclast", Aes(AesOp::DecLast)),
        ("aesimc", Aes(AesOp::Imc)),
        ("aeskeygenassist", Aes(AesOp::KeygenAssist)),
        ("cvtph2ps", Half(false)),
        ("cvtps2ph", Half(true)),
        ("sha1rnds4", Sha(ShaOp::Sha1Rnds4)),
        ("sha1nexte", Sha(ShaOp::Sha1Nexte)),
        ("sha1msg1", Sha(ShaOp::Sha1Msg1)),
        ("sha1msg2", Sha(ShaOp::Sha1Msg2)),
        ("sha256rnds2", Sha(ShaOp::Sha256Rnds2)),
        ("sha256msg1", Sha(ShaOp::Sha256Msg1)),
        ("sha256msg2", Sha(ShaOp::Sha256Msg2)),
        ("gf2p8mulb", Gf(GfOp::Mul)),
        ("gf2p8affineqb", Gf(GfOp::Affine(false))),
        ("gf2p8affineinvqb", Gf(GfOp::Affine(true))),
        ("pmaskmovd", MaskMov(Ty::I32)),
        ("maskmovps", MaskMov(Ty::I32)),
        ("pmaskmovq", MaskMov(Ty::I64)),
        ("maskmovpd", MaskMov(Ty::I64)),
        ("pbroadcastmw2d", BroadcastMask(Ty::I32)),
        ("pbroadcastmb2q", BroadcastMask(Ty::I64)),
        ("compressps", Compress(Ty::I32)),
        ("compresspd", Compress(Ty::I64)),
        ("expandps", Expand(Ty::I32)),
        ("expandpd", Expand(Ty::I64)),
        ("pscatterdd", Scatter(4, 4)),
        ("scatterdps", Scatter(4, 4)),
        ("pscatterdq", Scatter(4, 8)),
        ("scatterdpd", Scatter(4, 8)),
        ("pscatterqd", Scatter(8, 4)),
        ("scatterqps", Scatter(8, 4)),
        ("pscatterqq", Scatter(8, 8)),
        ("scatterqpd", Scatter(8, 8)),
        ("punpcklbw", Unpack(false, Ty::I8)),
        ("punpcklwd", Unpack(false, Ty::I16)),
        ("punpckldq", Unpack(false, Ty::I32)),
        ("punpckhbw", Unpack(true, Ty::I8)),
        ("punpckhwd", Unpack(true, Ty::I16)),
        ("punpckhdq", Unpack(true, Ty::I32)),
        ("permt2ps", PermTwo(Ty::I32, false)),
        ("permt2pd", PermTwo(Ty::I64, false)),
        ("permi2ps", PermTwo(Ty::I32, true)),
        ("permi2pd", PermTwo(Ty::I64, true)),
    ]
};

/// Builds a family's operation from its lane type.
type IntFamily = fn(Ty) -> SOp;

/// Builds a family's operation from its lane type and whether the form is scalar.
type FloatFamily = fn(Ty, bool) -> SOp;

/// Families named by a prefix and an integer lane letter (`b`, `w`, `d`, `q`): `psllvd`,
/// `pcmpuw`, `pextrb`.
const INT_LANE_FAMILIES: &[(&str, IntFamily)] = {
    use SOp::{
        Bin, BlendMask, Compress, Expand, IntCmp, MaskToVec, Perm, PermTwo, Pextr, Pinsr, RotImm,
        Testm, Unary,
    };
    &[
        ("psllv", |t| Bin(B2::Shlv, t)),
        ("psrlv", |t| Bin(B2::Shrv, t)),
        ("psrav", |t| Bin(B2::Sarv, t)),
        ("prolv", |t| Bin(B2::Rolv, t)),
        ("prorv", |t| Bin(B2::Rorv, t)),
        ("psign", |t| Bin(B2::Sign, t)),
        ("prol", |t| RotImm(true, t)),
        ("pror", |t| RotImm(false, t)),
        ("popcnt", |t| Unary(U1::Popcnt, t)),
        ("plzcnt", |t| Unary(U1::Lzcnt, t)),
        ("pconflict", |t| Unary(U1::Conflict, t)),
        ("perm", Perm),
        ("permt2", |t| PermTwo(t, false)),
        ("permi2", |t| PermTwo(t, true)),
        ("pblendm", BlendMask),
        ("pcompress", Compress),
        ("pexpand", Expand),
        ("pmovm2", MaskToVec),
        ("pinsr", Pinsr),
        ("pextr", Pextr),
        ("ptestnm", |t| Testm(t, true)),
        ("ptestm", |t| Testm(t, false)),
        // With a predicate immediate (AVX-512).
        ("pcmp", |t| IntCmp(t, false, None)),
        ("pcmpu", |t| IntCmp(t, true, None)),
    ]
};

/// Families named by a prefix and a float form (`ps`, `pd`, `ss`, `sd`), with whether the
/// family has scalar forms.
const FLOAT_FORM_FAMILIES: &[(&str, bool, FloatFamily)] = {
    use SOp::{FCmp, Round, Unary};
    &[
        ("cmp", true, FCmp),
        ("round", true, Round),
        ("rndscale", true, Round),
        ("rcp14", false, |t, _| Unary(U1::Rcp, t)),
        ("rcp28", false, |t, _| Unary(U1::Rcp, t)),
        ("rcp", false, |t, _| Unary(U1::Rcp, t)),
        ("rsqrt14", false, |t, _| Unary(U1::Rsqrt, t)),
        ("rsqrt28", false, |t, _| Unary(U1::Rsqrt, t)),
        ("rsqrt", false, |t, _| Unary(U1::Rsqrt, t)),
    ]
};

pub(in crate::sema::asm) fn lookup_simd(name: &str) -> Option<SOp> {
    if let Some(&(_, op)) = NAMED.iter().find(|(named, _)| *named == name) {
        return Some(op);
    }
    for &(prefix, op) in INT_LANE_FAMILIES {
        if let Some(t) = name.strip_prefix(prefix).and_then(int_letter) {
            return Some(op(t));
        }
    }
    for &(prefix, scalar_forms, op) in FLOAT_FORM_FAMILIES {
        if let Some((t, scalar)) = name.strip_prefix(prefix).and_then(float_letters)
            && (scalar_forms || !scalar)
        {
            return Some(op(t, scalar));
        }
    }
    lane_conversion(name).or_else(|| fma(name))
}

/// The `pmov` conversions spelled with lane letters: `pmov<t>2m`, `pmovsx`/`pmovzx<from><to>`
/// and `pmov[s|us]<from><to>`.
fn lane_conversion(name: &str) -> Option<SOp> {
    if let Some(rest) = name.strip_prefix("pmov")
        && let Some(t) = rest.strip_suffix("2m").and_then(int_letter)
    {
        return Some(SOp::VecToMask(t));
    }
    // pmovsx / pmovzx <from><to>
    for (prefix, signed) in [("pmovsx", true), ("pmovzx", false)] {
        if let Some(rest) = name.strip_prefix(prefix) {
            let mut c = rest.chars();
            let (from, to) = (size_letter(c.next()?)?, size_letter(c.next()?)?);
            if c.next().is_some() || from.size() >= to.size() {
                return None;
            }
            return Some(SOp::Extend(from, to, signed));
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
            return Some(SOp::Narrow(from, to, sat));
        }
    }
    None
}

/// FMA: `f[n]m{add,sub,addsub,subadd}{132,213,231}{ps,pd,ss,sd}`.
fn fma(name: &str) -> Option<SOp> {
    let rest = name.strip_prefix('f')?;
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
    Some(SOp::Fma(kind, order, t, scalar))
}
