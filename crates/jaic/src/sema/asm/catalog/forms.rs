//! Operand forms of each instruction.
use super::decode::Kind;
use crate::ir::Ty;
use crate::sema::asm::mask::KOp;
use crate::sema::asm::scalar::{Rep, StrOp, XOp};
use crate::sema::asm::simd::{AesOp, Conv, GfOp, ShaOp};
use crate::sema::asm::vec::VOp;
use crate::sema::asm::{Alu, Op};

// Operand forms

const BIN: &str = "dst: vec, src: vec/mem";
const BIN3: &str = "dst: vec, a: vec, b: vec/mem";
const BIN_IMM: &str = "dst: vec, src: vec/mem, imm";
const BIN3_IMM: &str = "dst: vec, a: vec, b: vec/mem, imm";
const UNARY: &str = "dst: vec, src: vec/mem";

/// Two-source vector forms: the three-operand one first for `v` spellings.
fn binary(vex: bool, imm: bool) -> Vec<&'static str> {
    let (two, three) = if imm {
        (BIN_IMM, BIN3_IMM)
    } else {
        (BIN, BIN3)
    };
    if vex {
        vec![three, two]
    } else {
        vec![two, three]
    }
}

pub(super) fn forms(name: &str, kind: Kind, vex: bool) -> Vec<String> {
    let shapes: Vec<&str> = match kind {
        Kind::Core(op) => match op {
            Op::Mov => vec!["dst: gpr/mem, src: gpr/mem/imm"],
            Op::Movzx | Op::Movsx => vec!["dst: gpr, src: gpr/mem"],
            Op::Movbe => vec!["dst: gpr/mem, src: gpr/mem"],
            Op::Lea => vec!["dst: gpr, src: mem"],
            Op::Xchg => vec!["a: gpr/mem, b: gpr/mem"],
            Op::Xadd => vec!["dst: gpr/mem, src: gpr"],
            Op::Cmpxchg => vec!["dst: gpr/mem, src: gpr, acc: gpr"],
            Op::Alu(Alu::Cmp | Alu::Test) => vec!["a: gpr/mem, b: gpr/mem/imm"],
            Op::Alu(_) => vec!["dst: gpr/mem, src: gpr/mem/imm"],
            Op::Inc | Op::Dec | Op::Neg | Op::Not => vec!["dst: gpr/mem"],
            Op::Shift(_) => vec!["dst: gpr/mem, count: gpr/imm", "dst: gpr/mem"],
            Op::Bt(_) => vec!["base: gpr/mem, bit: gpr/imm"],
            Op::Bsf | Op::Bsr | Op::Popcnt | Op::Lzcnt | Op::Tzcnt => {
                vec!["dst: gpr, src: gpr/mem"]
            }
            Op::Bswap => vec!["dst: gpr"],
            Op::Blsr | Op::Blsi | Op::Blsmsk => vec!["dst: gpr, src: gpr/mem"],
            Op::Imul => vec![
                "dst: gpr, src: gpr/mem",
                "dst: gpr, src: gpr/mem, imm",
                "hi: gpr, lo: gpr, src: gpr/mem",
            ],
            Op::Mul => vec!["hi: gpr, lo: gpr, src: gpr/mem"],
            Op::Setcc(_) => vec!["dst: gpr/mem"],
            Op::Cmovcc(_) => vec!["dst: gpr, src: gpr/mem"],
            Op::Nop | Op::Fence(_) | Op::Pause | Op::Int3 | Op::SetCarry(_) => vec![""],
            Op::Rdtsc => vec!["hi: gpr, lo: gpr"],
            Op::Rdtscp => vec!["hi: gpr, lo: gpr, id: gpr"],
            Op::Rdrand | Op::Rdseed => vec!["dst: gpr"],
            Op::Cpuid => vec!["a: gpr, b: gpr, c: gpr, d: gpr"],
            Op::Syscall => vec!["rcx_out: gpr, r11_out: gpr, rax_inout: gpr"],
        },
        Kind::Scalar(op) => match op {
            XOp::Div(_) => vec![
                "hi: gpr, lo: gpr, divisor: gpr/mem",
                "ax: gpr, divisor: gpr/mem",
            ],
            XOp::WidenA(_) => vec!["a: gpr"],
            XOp::SignFill(_) => vec!["d: gpr, a: gpr"],
            XOp::DoubleShift(_) => vec!["dst: gpr/mem, src: gpr, count: gpr/imm"],
            XOp::RotateCarry(_) => vec!["dst: gpr/mem, count: gpr/imm", "dst: gpr/mem"],
            XOp::Mulx => vec!["hi: gpr, lo: gpr, src: gpr/mem, d: gpr"],
            XOp::AddCarry(_) => vec!["dst: gpr, src: gpr/mem"],
            XOp::Andn => vec!["dst: gpr, a: gpr, b: gpr/mem"],
            XOp::Bextr => vec!["dst: gpr, src: gpr/mem, control: gpr"],
            XOp::Bzhi => vec!["dst: gpr, src: gpr/mem, index: gpr"],
            XOp::Pdep | XOp::Pext => vec!["dst: gpr, src: gpr, mask: gpr/mem"],
            XOp::ShiftX(_) => vec!["dst: gpr, src: gpr/mem, count: gpr"],
            XOp::Rorx => vec!["dst: gpr, src: gpr/mem, imm"],
            XOp::Crc32 => vec!["crc: gpr, data: gpr/mem"],
            XOp::Lahf | XOp::Sahf => vec!["a: gpr"],
            XOp::Xlat => vec!["table: gpr, a: gpr"],
            XOp::Hint | XOp::Stmxcsr | XOp::Ldmxcsr => vec!["addr: mem"],
            XOp::Direction(_) => vec![""],
            XOp::CmpxchgPair(_) => vec!["d: gpr, a: gpr, target: mem, c: gpr, b: gpr"],
            XOp::Str(op, rep) => {
                let regs = match op {
                    StrOp::Movs | StrOp::Cmps => "di: gpr, si: gpr",
                    StrOp::Stos | StrOp::Scas => "di: gpr, a: gpr",
                    StrOp::Lods => "a: gpr, si: gpr",
                };
                return vec![match rep {
                    Rep::Once => format!("{name}.size {regs}"),
                    Rep::Count | Rep::Repe | Rep::Repne => format!("{name}.size {regs}, c: gpr"),
                }];
            }
            XOp::Int => vec!["imm"],
            XOp::Xgetbv => vec!["d: gpr, a: gpr, c: gpr"],
            XOp::Rdpid => vec!["dst: gpr"],
        },
        Kind::Mask(op, _) => match op {
            KOp::Mov => vec!["dst: omr/gpr/mem, src: omr/gpr/mem"],
            KOp::Bin(_) | KOp::AndNot | KOp::Xnor | KOp::Add | KOp::Unpack => {
                vec!["dst: omr, a: omr, b: omr"]
            }
            KOp::Not => vec!["dst: omr, src: omr"],
            KOp::ShiftLeft | KOp::ShiftRight => vec!["dst: omr, src: omr, imm"],
            KOp::Test | KOp::OrTest => vec!["a: omr, b: omr"],
        },
        Kind::Vector(op) => vector_forms(op, vex),
    };
    shapes
        .into_iter()
        .map(|s| {
            if s.is_empty() {
                name.to_owned()
            } else {
                format!("{name} {s}")
            }
        })
        .collect()
}

fn vector_forms(op: VOp, vex: bool) -> Vec<&'static str> {
    match op {
        VOp::Move => vec!["dst: vec/mem, src: vec/mem"],
        VOp::MovScalarInt(_) => vec!["dst: vec/gpr/mem, src: vec/gpr/mem"],
        VOp::MovScalarFloat(_) => vec!["dst: vec/mem, src: vec/mem", "dst: vec, a: vec, b: vec"],
        VOp::Broadcast(_) => vec!["dst: vec, src: vec/gpr/mem"],
        VOp::Lanes(..) => binary(vex, false),
        VOp::Sqrt(_, true) => binary(vex, false),
        VOp::Sqrt(_, false) | VOp::Abs(_) | VOp::CvtDq(_) => vec![UNARY],
        VOp::Shift(..) => {
            if vex {
                vec![
                    "dst: vec, src: vec/mem, count: vec/imm",
                    "dst: vec, count: vec/imm",
                ]
            } else {
                vec![
                    "dst: vec, count: vec/imm",
                    "dst: vec, src: vec/mem, count: vec/imm",
                ]
            }
        }
        VOp::Pshufd => vec![BIN_IMM],
        VOp::Shufps => binary(vex, true),
        VOp::Movmsk(_) => vec!["dst: gpr, src: vec"],
        VOp::Gather(..) => vec![
            "dst: vec, [base + vindex*scale], mask: vec",
            "dst: vec &k, [base + vindex*scale]",
        ],
        VOp::Nop => vec![""],
        _ => simd_forms(op, vex),
    }
}

fn simd_forms(op: VOp, vex: bool) -> Vec<&'static str> {
    match op {
        VOp::Bin(..)
        | VOp::Horizontal(..)
        | VOp::AddSub(_)
        | VOp::Pmaddwd
        | VOp::Pmaddubsw
        | VOp::Psadbw
        | VOp::MulEven(_)
        | VOp::Unpack(..)
        | VOp::Pshufb
        | VOp::Pack(..)
        | VOp::BlendMask(_)
        | VOp::Movhlps
        | VOp::Movlhps
        | VOp::Pclmul(Some(_))
        | VOp::Cvt(Conv::ScalarFloat(..)) => binary(vex, false),
        VOp::IntCmp(_, _, Some(_)) | VOp::Testm(..) => vec!["dst: omr, a: vec, b: vec/mem"],
        VOp::IntCmp(_, _, None) => vec!["dst: omr, a: vec, b: vec/mem, imm"],
        VOp::Aes(AesOp::Imc) => vec![UNARY],
        VOp::Aes(AesOp::KeygenAssist) => vec![BIN_IMM],
        VOp::Aes(_) | VOp::Gf(GfOp::Mul) => binary(vex, false),
        VOp::Sha(ShaOp::Sha1Rnds4) => vec![BIN_IMM],
        VOp::Sha(ShaOp::Sha256Rnds2) => vec!["cdgh: vec, abef: vec, wk: vec"],
        VOp::Sha(_) => vec![BIN],
        VOp::RotImm(..) | VOp::ShufHalf(_) => vec![BIN_IMM],
        VOp::Round(_, false) => vec![BIN_IMM],
        VOp::Unary(..) | VOp::Dup(_) | VOp::Extend(..) | VOp::Phminposuw => vec![UNARY],
        VOp::Cvt(Conv::GprToScalar(..)) => {
            if vex {
                vec!["dst: vec, a: vec, b: gpr/mem", "dst: vec, src: gpr/mem"]
            } else {
                vec!["dst: vec, src: gpr/mem", "dst: vec, a: vec, b: gpr/mem"]
            }
        }
        VOp::Cvt(Conv::ScalarToGpr(..)) => vec!["dst: gpr, src: vec/mem"],
        VOp::Cvt(_) => vec![UNARY],
        VOp::Fma(..) => vec![BIN3],
        VOp::DotAcc(_) | VOp::Madd52(_) => vec!["acc: vec, a: vec, b: vec/mem"],
        VOp::Mpsadbw => vec![
            "dst: vec, src: vec/mem, imm8",
            "dst: vec, a: vec, b: vec/mem, imm8",
        ],
        VOp::Round(_, true)
        | VOp::Shufpd
        | VOp::Palignr
        | VOp::Valign(_)
        | VOp::BlendImm(_)
        | VOp::Shuf128(_)
        | VOp::Perm2x128
        | VOp::Insertps
        | VOp::Dp(_)
        | VOp::Pclmul(None)
        | VOp::FCmp(..)
        | VOp::Ternlog(_)
        | VOp::Gf(GfOp::Affine(_)) => binary(vex, true),
        VOp::ByteShift(_) => {
            if vex {
                vec!["dst: vec, src: vec, imm", "dst: vec, imm"]
            } else {
                vec!["dst: vec, imm", "dst: vec, src: vec, imm"]
            }
        }
        VOp::BlendVar(_) => {
            if vex {
                vec!["dst: vec, a: vec, b: vec/mem, mask: vec"]
            } else {
                vec!["dst: vec, src: vec/mem, mask: vec"]
            }
        }
        VOp::Perm(Ty::I64) => vec!["dst: vec, idx: vec, src: vec/mem", BIN_IMM],
        VOp::Perm(_) => vec!["dst: vec, idx: vec, src: vec/mem"],
        VOp::PermTwo(_, false) => vec!["dst: vec, idx: vec, table: vec/mem"],
        VOp::PermTwo(_, true) => vec!["idx: vec, a: vec, table: vec/mem"],
        VOp::PermilImm(_) => vec![BIN_IMM, "dst: vec, src: vec, control: vec/mem"],
        VOp::Insert(..) => vec![BIN3_IMM],
        VOp::Extract(..) => vec!["dst: vec/mem, src: vec, imm"],
        VOp::Pinsr(_) => {
            if vex {
                vec!["dst: vec, a: vec, b: gpr/mem, imm"]
            } else {
                vec!["dst: vec, src: gpr/mem, imm"]
            }
        }
        VOp::Pextr(_) | VOp::Extractps => vec!["dst: gpr/mem, src: vec, imm"],
        VOp::MovHalf(_) => {
            if vex {
                vec!["dst: vec, a: vec, b: mem", "dst: mem, src: vec"]
            } else {
                vec!["dst: vec, src: mem", "dst: mem, src: vec"]
            }
        }
        VOp::Narrow(..) => vec!["dst: vec/mem, src: vec"],
        VOp::Half(false) => vec![UNARY],
        VOp::Half(true) => vec!["dst: vec/mem, src: vec, imm"],
        VOp::Comis(_) | VOp::Ptest => vec!["a: vec, b: vec/mem"],
        VOp::Pcmpstr(false, false) => vec!["ecx: gpr, a: vec, b: vec/mem, imm8"],
        VOp::Pcmpstr(true, false) => vec!["ecx: gpr, eax: gpr, edx: gpr, a: vec, b: vec/mem, imm8"],
        VOp::Pcmpstr(false, true) => vec!["xmm0: vec, a: vec, b: vec/mem, imm8"],
        VOp::Pcmpstr(true, true) => vec!["xmm0: vec, eax: gpr, edx: gpr, a: vec, b: vec/mem, imm8"],
        VOp::MaskToVec(_) | VOp::BroadcastMask(_) => vec!["dst: vec, src: omr"],
        VOp::VecToMask(_) => vec!["dst: omr, src: vec"],
        VOp::Compress(_) => vec!["dst: vec/mem &k, src: vec"],
        VOp::Expand(_) => vec!["dst: vec &k, src: vec/mem"],
        VOp::Scatter(..) => vec!["[base + vindex*scale] &k, src: vec"],
        VOp::MaskMov(_) => vec![
            "dst: vec, mask: vec, src: mem",
            "dst: mem, mask: vec, src: vec",
        ],
        _ => unreachable!("core vector ops are described by their own table"),
    }
}

// Descriptions
