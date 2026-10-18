//! Operand forms of each instruction.
use super::decode::Kind;
use crate::ir::Ty;
use crate::sema::asm::mask::KOp;
use crate::sema::asm::scalar::{Rep, StrOp, XOp};
use crate::sema::asm::simd::{AesOp, Conv, GfOp, SOp, ShaOp};
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
        VOp::Bin(..) => binary(vex, false),
        VOp::Sqrt(_, true) => binary(vex, false),
        VOp::Sqrt(_, false) | VOp::Abs(_) | VOp::Cvt(_) => vec![UNARY],
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
        VOp::Ext(s) => simd_forms(s, vex),
    }
}

fn simd_forms(op: SOp, vex: bool) -> Vec<&'static str> {
    match op {
        SOp::Bin(..)
        | SOp::Horizontal(..)
        | SOp::AddSub(_)
        | SOp::Pmaddwd
        | SOp::Pmaddubsw
        | SOp::Psadbw
        | SOp::MulEven(_)
        | SOp::Unpack(..)
        | SOp::Pshufb
        | SOp::Pack(..)
        | SOp::BlendMask(_)
        | SOp::Movhlps
        | SOp::Movlhps
        | SOp::Pclmul(Some(_))
        | SOp::Cvt(Conv::ScalarFloat(..)) => binary(vex, false),
        SOp::IntCmp(_, _, Some(_)) | SOp::Testm(..) => vec!["dst: omr, a: vec, b: vec/mem"],
        SOp::IntCmp(_, _, None) => vec!["dst: omr, a: vec, b: vec/mem, imm"],
        SOp::Aes(AesOp::Imc) => vec![UNARY],
        SOp::Aes(AesOp::KeygenAssist) => vec![BIN_IMM],
        SOp::Aes(_) | SOp::Gf(GfOp::Mul) => binary(vex, false),
        SOp::Sha(ShaOp::Sha1Rnds4) => vec![BIN_IMM],
        SOp::Sha(ShaOp::Sha256Rnds2) => vec!["cdgh: vec, abef: vec, wk: vec"],
        SOp::Sha(_) => vec![BIN],
        SOp::RotImm(..) | SOp::ShufHalf(_) => vec![BIN_IMM],
        SOp::Round(_, false) => vec![BIN_IMM],
        SOp::Unary(..) | SOp::Dup(_) | SOp::Extend(..) | SOp::Phminposuw => vec![UNARY],
        SOp::Cvt(Conv::GprToScalar(..)) => {
            if vex {
                vec!["dst: vec, a: vec, b: gpr/mem", "dst: vec, src: gpr/mem"]
            } else {
                vec!["dst: vec, src: gpr/mem", "dst: vec, a: vec, b: gpr/mem"]
            }
        }
        SOp::Cvt(Conv::ScalarToGpr(..)) => vec!["dst: gpr, src: vec/mem"],
        SOp::Cvt(_) => vec![UNARY],
        SOp::Fma(..) => vec![BIN3],
        SOp::DotAcc(_) | SOp::Madd52(_) => vec!["acc: vec, a: vec, b: vec/mem"],
        SOp::Mpsadbw => vec![
            "dst: vec, src: vec/mem, imm8",
            "dst: vec, a: vec, b: vec/mem, imm8",
        ],
        SOp::Round(_, true)
        | SOp::Shufpd
        | SOp::Palignr
        | SOp::Valign(_)
        | SOp::BlendImm(_)
        | SOp::Shuf128(_)
        | SOp::Perm2x128
        | SOp::Insertps
        | SOp::Dp(_)
        | SOp::Pclmul(None)
        | SOp::FCmp(..)
        | SOp::Ternlog(_)
        | SOp::Gf(GfOp::Affine(_)) => binary(vex, true),
        SOp::ByteShift(_) => {
            if vex {
                vec!["dst: vec, src: vec, imm", "dst: vec, imm"]
            } else {
                vec!["dst: vec, imm", "dst: vec, src: vec, imm"]
            }
        }
        SOp::BlendVar(_) => {
            if vex {
                vec!["dst: vec, a: vec, b: vec/mem, mask: vec"]
            } else {
                vec!["dst: vec, src: vec/mem, mask: vec"]
            }
        }
        SOp::Perm(Ty::I64) => vec!["dst: vec, idx: vec, src: vec/mem", BIN_IMM],
        SOp::Perm(_) => vec!["dst: vec, idx: vec, src: vec/mem"],
        SOp::PermTwo(_, false) => vec!["dst: vec, idx: vec, table: vec/mem"],
        SOp::PermTwo(_, true) => vec!["idx: vec, a: vec, table: vec/mem"],
        SOp::PermilImm(_) => vec![BIN_IMM, "dst: vec, src: vec, control: vec/mem"],
        SOp::Insert(..) => vec![BIN3_IMM],
        SOp::Extract(..) => vec!["dst: vec/mem, src: vec, imm"],
        SOp::Pinsr(_) => {
            if vex {
                vec!["dst: vec, a: vec, b: gpr/mem, imm"]
            } else {
                vec!["dst: vec, src: gpr/mem, imm"]
            }
        }
        SOp::Pextr(_) | SOp::Extractps => vec!["dst: gpr/mem, src: vec, imm"],
        SOp::MovHalf(_) => {
            if vex {
                vec!["dst: vec, a: vec, b: mem", "dst: mem, src: vec"]
            } else {
                vec!["dst: vec, src: mem", "dst: mem, src: vec"]
            }
        }
        SOp::Narrow(..) => vec!["dst: vec/mem, src: vec"],
        SOp::Half(false) => vec![UNARY],
        SOp::Half(true) => vec!["dst: vec/mem, src: vec, imm"],
        SOp::Comis(_) | SOp::Ptest => vec!["a: vec, b: vec/mem"],
        SOp::Pcmpstr(false, false) => vec!["ecx: gpr, a: vec, b: vec/mem, imm8"],
        SOp::Pcmpstr(true, false) => vec!["ecx: gpr, eax: gpr, edx: gpr, a: vec, b: vec/mem, imm8"],
        SOp::Pcmpstr(false, true) => vec!["xmm0: vec, a: vec, b: vec/mem, imm8"],
        SOp::Pcmpstr(true, true) => vec!["xmm0: vec, eax: gpr, edx: gpr, a: vec, b: vec/mem, imm8"],
        SOp::MaskToVec(_) | SOp::BroadcastMask(_) => vec!["dst: vec, src: omr"],
        SOp::VecToMask(_) => vec!["dst: omr, src: vec"],
        SOp::Compress(_) => vec!["dst: vec/mem &k, src: vec"],
        SOp::Expand(_) => vec!["dst: vec &k, src: vec/mem"],
        SOp::Scatter(..) => vec!["[base + vindex*scale] &k, src: vec"],
        SOp::MaskMov(_) => vec![
            "dst: vec, mask: vec, src: mem",
            "dst: mem, mask: vec, src: vec",
        ],
    }
}

// Descriptions
