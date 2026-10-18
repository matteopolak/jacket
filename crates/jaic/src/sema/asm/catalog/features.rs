//! The CPU features each instruction needs.
use super::decode::Kind;
use crate::ir::Ty;
use crate::sema::asm::Op;
use crate::sema::asm::mask::KOp;
use crate::sema::asm::scalar::XOp;
use crate::sema::asm::simd::{B2, Conv, SOp, U1};
use crate::sema::asm::vec::{Lane, Shift, VOp};

// Features

pub(super) fn core_feature(name: &str, kind: Kind) -> &'static str {
    let name = name.strip_prefix("lock_").unwrap_or(name);
    match kind {
        Kind::Core(op) => match op {
            Op::Movbe => "MOVBE",
            Op::Popcnt => "POPCNT",
            Op::Lzcnt => "LZCNT",
            Op::Tzcnt | Op::Blsr | Op::Blsi | Op::Blsmsk => "BMI1",
            Op::Cmovcc(_) => "CMOV",
            Op::Rdtsc => "TSC",
            Op::Rdtscp => "RDTSCP",
            Op::Syscall => "SYSCALL_SYSRET",
            Op::Rdseed => "RDSEED",
            Op::Rdrand => "RDRAND",
            Op::Fence(crate::ir::X86Fence::Store) => "SSE",
            Op::Fence(_) => "SSE2",
            Op::Pause => "SSE2",
            Op::Mov if name == "movnti" => "SSE2",
            Op::Mov
            | Op::Movzx
            | Op::Movsx
            | Op::Lea
            | Op::Xchg
            | Op::Xadd
            | Op::Cmpxchg
            | Op::Alu(_)
            | Op::Inc
            | Op::Dec
            | Op::Neg
            | Op::Not
            | Op::Shift(_)
            | Op::Bt(_)
            | Op::Bsf
            | Op::Bsr
            | Op::Bswap
            | Op::Imul
            | Op::Mul
            | Op::Setcc(_)
            | Op::Nop
            | Op::Int3
            | Op::SetCarry(_)
            | Op::Cpuid => "x86-64",
        },
        Kind::Scalar(op) => match op {
            XOp::Mulx | XOp::Bzhi | XOp::Pdep | XOp::Pext | XOp::ShiftX(_) | XOp::Rorx => "BMI2",
            XOp::Andn | XOp::Bextr => "BMI1",
            XOp::AddCarry(_) => "ADX",
            XOp::Crc32 => "SSE4_2",
            XOp::Lahf | XOp::Sahf => "LAHF_SAHF",
            XOp::Hint => match name {
                "prefetchw" => "PREFETCHW",
                "prefetchwt1" => "PREFETCHWT1",
                "clflush" => "CLFLUSH",
                "clflushopt" => "CLFLUSHOPT",
                "clwb" => "CLWB",
                "cldemote" => "CLDEMOTE",
                _ => "SSE",
            },
            XOp::CmpxchgPair(Ty::I32) => "CX8",
            XOp::CmpxchgPair(_) => "CMPXCHG16B",
            XOp::Xgetbv => "XSAVE",
            XOp::Stmxcsr | XOp::Ldmxcsr => "SSE",
            XOp::Rdpid => "RDPID",
            XOp::Div(_)
            | XOp::WidenA(_)
            | XOp::SignFill(_)
            | XOp::DoubleShift(_)
            | XOp::RotateCarry(_)
            | XOp::Xlat
            | XOp::Direction(_)
            | XOp::Str(..)
            | XOp::Int => "x86-64",
        },
        Kind::Mask(op, ty) => match (op, ty) {
            (KOp::Add | KOp::Test, Ty::I8 | Ty::I16) => "AVX512DQ",
            (KOp::Unpack, Ty::I16) => "AVX512F",
            (_, Ty::I8) => "AVX512DQ",
            (_, Ty::I16) => "AVX512F",
            _ => "AVX512BW",
        },
        Kind::Vector(_) => unreachable!("vector features come from vector_features"),
    }
}

/// Legacy SSE form and VEX/EVEX (`v`) form: `sse("SSE2", true)` has both (`AVX2` for the
/// integer `v` form, `AVX` for floats); `avx(..)` has only the `v` form.
fn sse(feature: &'static str, int: bool) -> (Option<&'static str>, Option<&'static str>) {
    (
        Some(feature),
        Some(if int {
            "AVX2"
        } else {
            "AVX"
        }),
    )
}

fn avx(feature: &'static str) -> (Option<&'static str>, Option<&'static str>) {
    (None, Some(feature))
}

/// `SSE` for single precision (`ps`/`ss`), `SSE2` for double.
fn fp(name: &str) -> (Option<&'static str>, Option<&'static str>) {
    if name.ends_with("ps") || name.ends_with("ss") {
        sse("SSE", false)
    } else {
        sse("SSE2", false)
    }
}

fn byte_word(ty: Ty, small: &'static str, large: &'static str) -> &'static str {
    if matches!(ty, Ty::I8 | Ty::I16) {
        small
    } else {
        large
    }
}

pub(super) fn vector_features(name: &str, op: VOp) -> (Option<&'static str>, Option<&'static str>) {
    match op {
        VOp::Move => match name {
            "movups" | "movaps" | "movntps" => sse("SSE", false),
            "movupd" | "movapd" | "movntpd" => sse("SSE2", false),
            "movdqu" | "movdqa" | "movntdq" => sse("SSE2", true),
            "lddqu" => sse("SSE3", true),
            "movntdqa" => sse("SSE4_1", true),
            "movdqu8" | "movdqu16" => avx("AVX512BW"),
            _ => avx("AVX512F"),
        },
        VOp::MovScalarInt(_) => sse("SSE2", false),
        VOp::MovScalarFloat(ty) => sse(
            if ty == Ty::F32 {
                "SSE"
            } else {
                "SSE2"
            },
            false,
        ),
        VOp::Broadcast(_) => match name {
            "broadcastss" | "broadcastsd" | "broadcastf128" => avx("AVX"),
            "pbroadcastb" | "pbroadcastw" | "pbroadcastd" | "pbroadcastq" | "broadcasti128" => {
                avx("AVX2")
            }
            _ if name.contains("32x2") || name.contains("64x2") || name.contains("32x8") => {
                avx("AVX512DQ")
            }
            _ => avx("AVX512F"),
        },
        VOp::Bin(lane, ty, _) => match lane {
            Lane::FAdd | Lane::FSub | Lane::FMul | Lane::FDiv | Lane::FMin | Lane::FMax => fp(name),
            Lane::And | Lane::Or | Lane::Xor | Lane::AndNot => {
                if !name.starts_with('p') {
                    fp(name)
                } else if name.ends_with('d') || name.ends_with('q') {
                    avx("AVX512F")
                } else {
                    sse("SSE2", true)
                }
            }
            Lane::Mul => match ty {
                Ty::I16 => sse("SSE2", true),
                Ty::I32 => sse("SSE4_1", true),
                _ => avx("AVX512DQ"),
            },
            Lane::Add | Lane::Sub => sse("SSE2", true),
            Lane::CmpEq if ty == Ty::I64 => sse("SSE4_1", true),
            Lane::CmpGt if ty == Ty::I64 => sse("SSE4_2", true),
            Lane::CmpEq | Lane::CmpGt => sse("SSE2", true),
            Lane::MinS | Lane::MaxS | Lane::MinU | Lane::MaxU => match (lane, ty) {
                (_, Ty::I64) => avx("AVX512F"),
                (Lane::MinS | Lane::MaxS, Ty::I16) | (Lane::MinU | Lane::MaxU, Ty::I8) => {
                    sse("SSE2", true)
                }
                _ => sse("SSE4_1", true),
            },
        },
        VOp::Sqrt(..) | VOp::Shufps => fp(name),
        VOp::Abs(Ty::I64) | VOp::Shift(Shift::Arith, Ty::I64) => avx("AVX512F"),
        VOp::Abs(_) => sse("SSSE3", true),
        VOp::Shift(..) | VOp::Pshufd => sse("SSE2", true),
        VOp::Cvt(_) => sse("SSE2", false),
        VOp::Movmsk(4) => sse("SSE", false),
        VOp::Movmsk(8) => sse("SSE2", false),
        VOp::Movmsk(_) => sse("SSE2", true),
        VOp::Gather(..) => avx("AVX2"),
        VOp::Nop if name == "emms" => (Some("MMX"), None),
        VOp::Nop => avx("AVX"),
        VOp::Ext(s) => simd_features(name, s),
    }
}

fn simd_features(name: &str, op: SOp) -> (Option<&'static str>, Option<&'static str>) {
    match op {
        SOp::Bin(b, ty) => match b {
            B2::AddSatS | B2::AddSatU | B2::SubSatS | B2::SubSatU | B2::Avg => sse("SSE2", true),
            B2::MulHiS | B2::MulHiU => sse("SSE2", true),
            B2::MulHrs | B2::Sign => sse("SSSE3", true),
            B2::Shlv | B2::Shrv | B2::Sarv if ty == Ty::I16 => avx("AVX512BW"),
            B2::Sarv if ty == Ty::I64 => avx("AVX512F"),
            B2::Shlv | B2::Shrv | B2::Sarv => avx("AVX2"),
            B2::Rolv | B2::Rorv => avx("AVX512F"),
        },
        SOp::RotImm(..) | SOp::Valign(_) | SOp::Shuf128(_) | SOp::Ternlog(_) => avx("AVX512F"),
        SOp::Scatter(..) => avx("AVX512F"),
        SOp::Unary(u, ty) => match u {
            U1::Popcnt => avx(byte_word(ty, "AVX512_BITALG", "AVX512_VPOPCNTDQ")),
            U1::Lzcnt | U1::Conflict => avx("AVX512CD"),
            U1::Rcp | U1::Rsqrt if name.contains("14") => avx("AVX512F"),
            U1::Rcp | U1::Rsqrt if name.contains("28") => avx("AVX512ER"),
            U1::Rcp | U1::Rsqrt => fp(name),
        },
        SOp::Round(..) if name.starts_with("rndscale") => avx("AVX512F"),
        SOp::Round(..) | SOp::Dp(_) | SOp::Insertps | SOp::Extractps => sse("SSE4_1", false),
        SOp::BlendVar(_) => sse("SSE4_1", !name.starts_with('b')),
        SOp::Fma(..) => avx("FMA"),
        SOp::Horizontal(_, _, Ty::F32 | Ty::F64) | SOp::AddSub(_) | SOp::Dup(_) => {
            sse("SSE3", false)
        }
        SOp::Horizontal(..) | SOp::Pmaddubsw | SOp::Palignr | SOp::Pshufb => sse("SSSE3", true),
        SOp::Pmaddwd | SOp::Psadbw | SOp::MulEven(false) | SOp::ShufHalf(_) => sse("SSE2", true),
        SOp::Mpsadbw => sse("SSE4_1", true),
        SOp::Madd52(_) => avx("AVX512_IFMA"),
        SOp::ByteShift(_) => sse("SSE2", true),
        SOp::MulEven(true) | SOp::Phminposuw | SOp::Ptest | SOp::Extend(..) => sse("SSE4_1", true),
        SOp::Pcmpstr(..) => sse("SSE4_2", false),
        SOp::DotAcc(_) => avx("AVX512_VNNI"),
        SOp::Unpack(..) if name.starts_with('u') => fp(name),
        SOp::Unpack(..) => sse("SSE2", true),
        SOp::Shufpd | SOp::MovHalf(_) if name.ends_with("pd") => sse("SSE2", false),
        SOp::Shufpd | SOp::MovHalf(_) | SOp::Movhlps | SOp::Movlhps => sse("SSE", false),
        SOp::BlendImm(_) => match name {
            "pblendd" => avx("AVX2"),
            "pblendw" => sse("SSE4_1", true),
            _ => sse("SSE4_1", false),
        },
        SOp::PermTwo(Ty::I8, _) => avx("AVX512VBMI"),
        SOp::BlendMask(ty) | SOp::IntCmp(ty, ..) | SOp::Testm(ty, _) | SOp::PermTwo(ty, _) => {
            avx(byte_word(ty, "AVX512BW", "AVX512F"))
        }
        SOp::Narrow(from, ..) => avx(if from == Ty::I16 {
            "AVX512BW"
        } else {
            "AVX512F"
        }),
        SOp::MaskToVec(ty) | SOp::VecToMask(ty) => avx(byte_word(ty, "AVX512BW", "AVX512DQ")),
        SOp::BroadcastMask(_) => avx("AVX512CD"),
        SOp::Compress(ty) | SOp::Expand(ty) => avx(byte_word(ty, "AVX512_VBMI2", "AVX512F")),
        SOp::Perm(ty) => match ty {
            Ty::I8 => avx("AVX512VBMI"),
            Ty::I16 => avx("AVX512BW"),
            _ => avx("AVX2"),
        },
        SOp::PermilImm(_) => avx("AVX"),
        SOp::Perm2x128 | SOp::Insert(..) | SOp::Extract(..) => {
            if name.contains("128") {
                // `perm2f128` / `insertf128`: AVX; the integer `i128` forms: AVX2.
                avx(if name.contains("f128") {
                    "AVX"
                } else {
                    "AVX2"
                })
            } else if name.contains("32x8") || name.contains("64x2") {
                avx("AVX512DQ")
            } else {
                avx("AVX512F")
            }
        }
        SOp::Pinsr(Ty::I16) | SOp::Pextr(Ty::I16) => sse("SSE2", true),
        SOp::Pinsr(_) | SOp::Pextr(_) => sse("SSE4_1", true),
        SOp::Pack(Ty::I32, true) => sse("SSE4_1", true),
        SOp::Pack(..) => sse("SSE2", true),
        SOp::Cvt(conv) => match conv {
            _ if name.contains("qq") => avx("AVX512DQ"),
            _ if name.contains("2u") || name.starts_with("cvtu") => avx("AVX512F"),
            Conv::GprToScalar(Ty::F32, _) | Conv::ScalarToGpr(Ty::F32, ..) => sse("SSE", false),
            Conv::IntToFloat(..)
            | Conv::FloatToInt(..)
            | Conv::FloatToFloat(..)
            | Conv::ScalarFloat(..)
            | Conv::GprToScalar(..)
            | Conv::ScalarToGpr(..) => sse("SSE2", false),
        },
        SOp::Comis(Ty::F32) => sse("SSE", false),
        SOp::Comis(_) => sse("SSE2", false),
        SOp::FCmp(..) => fp(name),
        SOp::Pclmul(_) => (Some("PCLMULQDQ"), Some("PCLMULQDQ")),
        SOp::Aes(_) => (Some("AES"), Some("AES")),
        SOp::Half(_) => avx("F16C"),
        SOp::Sha(_) => (Some("SHA"), None),
        SOp::Gf(_) => (Some("GFNI"), Some("GFNI")),
        SOp::MaskMov(_) if name.starts_with('p') => avx("AVX2"),
        SOp::MaskMov(_) => avx("AVX"),
    }
}
