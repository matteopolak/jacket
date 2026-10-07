//! `#asm` lowering.
//!
//! jaic never emits machine code for `#asm`. Each x64 instruction is lowered to
//! ordinary IR operations on the Jai variables used as operands, so a block
//! behaves identically in the interpreter, in the browser and in the LLVM
//! backend, on any CPU.
//!
//! Model:
//! - Operands are Jai variables (read and written in place), integer
//!   immediates, `[base + index*scale +/- disp]` memory operands, and register
//!   declarations (`tmp:` / `tmp: gpr === a`). A declared `gpr` is a 64-bit
//!   local of the enclosing scope (blocks are not scopes), initialised to zero.
//!   Register pinning is accepted and ignored: registers are not modeled.
//! - The operation size is the `.b/.w/.d/.q` (or `.8/.16/.32/.64`, `?T`) suffix,
//!   else the size of the first Jai variable operand, else 64 bits. As on
//!   hardware, a 32-bit write zero-extends into a 64-bit register while 8- and
//!   16-bit writes merge into its low bits.
//! - Flags (CF, ZF, SF, OF) are IR values tracked while lowering one block;
//!   flags not yet set in the block read as 0. `setcc`/`cmovcc` read them.
//! - `lock_`-prefixed read-modify-write instructions on memory use a
//!   compare-and-swap loop, so they are atomic on native targets.
//! - Vector instructions (`vec` registers, SSE/AVX/AVX-512 mnemonics) are lowered
//!   lane by lane in `asm/vec.rs`.
//! - Division, string instructions, BMI1/BMI2, double shifts, CRC32 and the
//!   other less common general-purpose instructions live in `asm/scalar.rs`;
//!   mask-register (`omr`) instructions in `asm/mask.rs`.
//! - Anything else (x87, privileged and I/O instructions, `syscall`, ...) is a
//!   compile error naming the instruction.
use super::lower::{FnCtx, Operand};
use super::scope::{EntityKind, Found};
use super::*;
use crate::ast::{
    AsmBlock, AsmDecl, AsmInst, AsmItem, AsmMem, AsmOperand, AsmSize, CodeBody, Expr, ExprKind as E,
};
use crate::ir::{BinOp, BlockId, CmpOp, ConvOp, Intrinsic, Ty, UnOp, Val};

pub mod catalog;
mod mask;
mod scalar;
mod simd;
mod vec;

/// Feature-set modifiers accepted after `#asm`: every CPUID feature-flag name, plus the older
/// spellings without the underscore. Only `AVX*` changes lowering (the default vector width).
const FEATURES: &[&str] = &[
    "SSE",
    "SSE2",
    "SSE3",
    "SSSE3",
    "SSE4_1",
    "SSE4_2",
    "AVX",
    "AVX2",
    "AVX512F",
    "AVX512BW",
    "AVX512CD",
    "AVX512DQ",
    "AVX512VL",
    "AVX512VBMI",
    "AVX512VBMI2",
    "AVX512IFMA",
    "AVX512ER",
    "AVX512PF",
    "AVX512_VNNI",
    "AVX512_BITALG",
    "AVX512_VPOPCNTDQ",
    "BMI1",
    "BMI2",
    "ADX",
    "POPCNT",
    "LZCNT",
    "MOVBE",
    "FMA",
    "F16C",
    "AES",
    "PCLMULQDQ",
    "SHA",
    "RDRAND",
    "RDSEED",
    "CMPXCHG16B",
    "SYSCALL_SYSRET",
    "FSGSBASE",
    "CLFLUSHOPT",
    "CLWB",
    "MMX",
    "FXSR",
    "XSAVE",
    "XSAVEOPT",
    "X87",
    "TSC",
    "RDTSCP",
    "LAHF_SAHF",
    "PREFETCHW",
    "MWAITX",
    "CLZERO",
    // The remaining CPUID feature-flag names (Machine_X64 `x86_Feature_Flag`).
    "ABM",
    "ACPI",
    "AESKLE",
    "AESKLE_WIDE",
    "AMX_BF16",
    "AMX_INT8",
    "AMX_TILE",
    "APIC",
    "ARCH_CAPABILITIES",
    "AVX512_4FMAPS",
    "AVX512_4VNNIW",
    "AVX512_BF16",
    "AVX512_FP16",
    "AVX512_IFMA",
    "AVX512_VBMI",
    "AVX512_VBMI2",
    "AVX512_VP2INTERSECT",
    "AVX_VNNI",
    "CET_IBT",
    "CET_SS",
    "CLDEMOTE",
    "CLFLUSH",
    "CMOV",
    "CMP_LEGACY",
    "CNXT_ID",
    "CORE_CAPABILITIES",
    "CR8_LEGACY",
    "CX16",
    "CX8",
    "DBX",
    "DCA",
    "DE",
    "DEP_FPU_CS_DS",
    "DS",
    "DS_CPL",
    "DTEST64",
    "EIST",
    "ENHANCED_REP",
    "ENQCMD",
    "EXTAPIC",
    "FDP_EXCPTN_ONLY",
    "FIVE_LEVEL_PAGING",
    "FMA4",
    "FPU",
    "FSRCS",
    "FSRM",
    "FSRS",
    "FXSR_OPT",
    "FZRM",
    "GFNI",
    "HLE",
    "HRESET",
    "HTT",
    "HYBRID",
    "HYPERVISOR",
    "IA64",
    "IBS",
    "INTEL_PT",
    "INVLPGB",
    "INVPCID",
    "KEY_LOCKER_MSR",
    "KL",
    "L1D_FLUSH",
    "LAM",
    "LBR",
    "LWP",
    "MAWAU_0",
    "MAWAU_1",
    "MAWAU_2",
    "MAWAU_3",
    "MAWAU_4",
    "MCA",
    "MCE",
    "MCOMMIT",
    "MD_CLEAR",
    "MISALIGNED_SSE",
    "MMX_EXT",
    "MONITOR",
    "MOVDIR64B",
    "MOVDIRI",
    "MP",
    "MPX",
    "MSR",
    "MTRR",
    "NODE_ID",
    "NX",
    "OSPKE",
    "OSVW",
    "OSXSAVE",
    "PAE",
    "PAT",
    "PBE",
    "PCID",
    "PCOMMIT",
    "PCONFIG",
    "PCX_L2I",
    "PDCM",
    "PDPE1GB",
    "PERFCTR_CORE",
    "PERFCTR_NB",
    "PERFTSC",
    "PGE",
    "PKS",
    "PKU",
    "PREFETCHWT1",
    "PSE",
    "PSE_36",
    "PSN",
    "RDPID",
    "RDPRU",
    "RDT_A",
    "RDT_M",
    "RTM",
    "SDBG",
    "SEP",
    "SERIALIZE",
    "SGX",
    "SGX_LC",
    "SKINIT",
    "SMAP",
    "SMEP",
    "SMX",
    "SPEC_CTRL",
    "SRBDS_CTRL",
    "SS",
    "SSBD",
    "SSE4A",
    "STIBP",
    "SVM",
    "TBM",
    "TCE",
    "TM",
    "TM2",
    "TME_EN",
    "TOPOLOGY_EXTENSIONS",
    "TSC_ADJUST",
    "TSC_DEADLINE",
    "TSXLDTRK",
    "TSX_FORCE_ABORT",
    "UMIP",
    "VAES",
    "VME",
    "VMX",
    "VPCLMULQDQ",
    "WAITPKG",
    "WBNOINVD",
    "WDT",
    "X2APIC",
    "XFD",
    "XGETBV_ECX1",
    "XOP",
    "XSAVEC",
    "XSAVES_XRSTORS",
    "XTPR",
    "_3DNOW",
    "_3DNOW_EXT",
    "_3DNOW_PREFETCH",
    "_64BIT_MODE",
];

/// What a name declared by an `#asm` register declaration is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AsmReg {
    /// General-purpose register: a 64-bit local.
    Gpr,
    /// Vector register: a 64-byte local (zmm-sized; xmm/ymm use its low bytes). `str`
    /// (MMX) registers are vector registers used with the 8-byte `.q` size.
    Vec,
    /// AVX-512 op-mask register (`omr`, k0-k7): a 64-bit local.
    Mask,
}

/// The register classes a declaration may name (`x: gpr;`), with their storage in bytes.
const REG_CLASSES: &[(&str, AsmReg, u64)] = &[
    ("gpr", AsmReg::Gpr, 8),
    ("vec", AsmReg::Vec, 64),
    ("str", AsmReg::Vec, 64),
    ("omr", AsmReg::Mask, 8),
    ("kmask", AsmReg::Mask, 8),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Alu {
    Add,
    Sub,
    Adc,
    Sbb,
    And,
    Or,
    Xor,
    Cmp,
    Test,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ShiftKind {
    Shl,
    Shr,
    Sar,
    Rol,
    Ror,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BitTest {
    Bt,
    Bts,
    Btr,
    Btc,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cond {
    E,
    Ne,
    B,
    Ae,
    Be,
    A,
    S,
    Ns,
    O,
    No,
    L,
    Ge,
    Le,
    G,
    P,
    Np,
}

/// Instruction semantics. One entry per mnemonic family in `lookup_op`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Op {
    Mov,
    Movzx,
    Movsx,
    Movbe,
    Lea,
    Xchg,
    Xadd,
    Cmpxchg,
    Alu(Alu),
    Inc,
    Dec,
    Neg,
    Not,
    Shift(ShiftKind),
    Bt(BitTest),
    Bsf,
    Bsr,
    Popcnt,
    Lzcnt,
    Tzcnt,
    Bswap,
    Blsr,
    Blsi,
    Blsmsk,
    Imul,
    Mul,
    Setcc(Cond),
    Cmovcc(Cond),
    /// `nop`, fences, `cld`/`std`: no observable effect in this model.
    Nop,
    Pause,
    Int3,
    SetCarry(Option<bool>),
    Rdtsc,
    Rdtscp,
    Rdrand,
    Cpuid,
}

fn lookup_cond(name: &str) -> Option<Cond> {
    Some(match name {
        "e" | "z" => Cond::E,
        "ne" | "nz" => Cond::Ne,
        "b" | "c" | "nae" => Cond::B,
        "ae" | "nb" | "nc" => Cond::Ae,
        "be" | "na" => Cond::Be,
        "a" | "nbe" => Cond::A,
        "s" => Cond::S,
        "ns" => Cond::Ns,
        "o" => Cond::O,
        "no" => Cond::No,
        "l" | "nge" => Cond::L,
        "ge" | "nl" => Cond::Ge,
        "le" | "ng" => Cond::Le,
        "g" | "nle" => Cond::G,
        "p" | "pe" => Cond::P,
        "np" | "po" => Cond::Np,
        _ => return None,
    })
}

/// The instruction table: mnemonic (without `lock_`) to semantics.
fn lookup_op(name: &str) -> Option<Op> {
    Some(match name {
        "mov" | "movnti" => Op::Mov,
        "movbe" => Op::Movbe,
        "lea" => Op::Lea,
        "xchg" => Op::Xchg,
        "xadd" => Op::Xadd,
        "cmpxchg" => Op::Cmpxchg,
        "add" => Op::Alu(Alu::Add),
        "sub" => Op::Alu(Alu::Sub),
        "adc" => Op::Alu(Alu::Adc),
        "sbb" => Op::Alu(Alu::Sbb),
        "and" => Op::Alu(Alu::And),
        "or" => Op::Alu(Alu::Or),
        "xor" => Op::Alu(Alu::Xor),
        "cmp" => Op::Alu(Alu::Cmp),
        "test" => Op::Alu(Alu::Test),
        "inc" => Op::Inc,
        "dec" => Op::Dec,
        "neg" => Op::Neg,
        "not" => Op::Not,
        "shl" | "sal" => Op::Shift(ShiftKind::Shl),
        "shr" => Op::Shift(ShiftKind::Shr),
        "sar" => Op::Shift(ShiftKind::Sar),
        "rol" => Op::Shift(ShiftKind::Rol),
        "ror" => Op::Shift(ShiftKind::Ror),
        "bt" => Op::Bt(BitTest::Bt),
        "bts" => Op::Bt(BitTest::Bts),
        "btr" => Op::Bt(BitTest::Btr),
        "btc" => Op::Bt(BitTest::Btc),
        "bsf" => Op::Bsf,
        "bsr" => Op::Bsr,
        "popcnt" => Op::Popcnt,
        "lzcnt" => Op::Lzcnt,
        "tzcnt" => Op::Tzcnt,
        "bswap" => Op::Bswap,
        "blsr" => Op::Blsr,
        "blsi" => Op::Blsi,
        "blsmsk" => Op::Blsmsk,
        "imul" => Op::Imul,
        "mul" => Op::Mul,
        "nop" | "mfence" | "lfence" | "sfence" => Op::Nop,
        "pause" => Op::Pause,
        "int3" => Op::Int3,
        "clc" => Op::SetCarry(Some(false)),
        "stc" => Op::SetCarry(Some(true)),
        "cmc" => Op::SetCarry(None),
        "rdtsc" => Op::Rdtsc,
        "rdtscp" => Op::Rdtscp,
        "rdrand" | "rdseed" => Op::Rdrand,
        "cpuid" => Op::Cpuid,
        "movsxd" => Op::Movsx,
        _ => {
            if let Some(rest) = name.strip_prefix("set") {
                return lookup_cond(rest).map(Op::Setcc);
            }
            if let Some(rest) = name.strip_prefix("cmov") {
                return lookup_cond(rest).map(Op::Cmovcc);
            }
            if name.starts_with("movzx") {
                return Some(Op::Movzx);
            }
            if name.starts_with("movsx") {
                return Some(Op::Movsx);
            }
            return None;
        }
    })
}

/// A scalar variable operand: its storage and how the instruction may use it.
#[derive(Clone, Copy)]
struct RegPlace {
    addr: Val,
    /// Register class of the storage (`Ptr` for pointer variables).
    storage: Ty,
    signed: bool,
    /// A Jai variable (its type fixes the operand size) rather than a declared `gpr`.
    natural: bool,
}

#[derive(Clone, Copy)]
enum Imm {
    Int(i128),
    Float(f64),
}

#[derive(Clone, Copy)]
enum Opd {
    Reg(RegPlace),
    Imm(Imm),
    /// Memory at this `I64` address.
    Mem(Val),
}

#[derive(Clone, Copy, Default)]
struct Flags {
    cf: Option<Val>,
    zf: Option<Val>,
    sf: Option<Val>,
    of: Option<Val>,
    /// PF as a lazily evaluated byte: the flag is set when this `I8` has even parity.
    pf: Option<Val>,
}

struct AsmCtx {
    scope: ScopeId,
    flags: Flags,
    /// Default vector operand size in bytes: 16, 32 with AVX, 64 with AVX-512.
    vec_width: u64,
    /// VEX/EVEX encoding (AVX and up): vector writes zero the rest of the register.
    vex: bool,
    /// The direction flag (`std`/`cld`), tracked statically within the block. The ABI
    /// guarantees it is clear on entry, so a block that never sets it counts upwards.
    df: bool,
}

/// An in-progress read-modify-write of one operand.
struct Rmw {
    dst: Opd,
    sz: Ty,
    /// Compare-and-swap loop: (pointer, old value, retry block, exit block).
    cas: Option<(Val, Val, BlockId, BlockId)>,
}

// ---------------------------------------------------------------------------
// Small IR helpers
// ---------------------------------------------------------------------------

fn bits(ty: Ty) -> u64 {
    ty.size() * 8
}

fn konst(f: &mut FnCtx, ty: Ty, v: u64) -> Val {
    f.b.iconst(ty, v)
}

fn bin(f: &mut FnCtx, op: BinOp, ty: Ty, a: Val, b: Val) -> Val {
    f.b.bin(op, ty, a, b)
}

fn cmp(f: &mut FnCtx, op: CmpOp, ty: Ty, a: Val, b: Val) -> Val {
    f.b.cmp(op, ty, a, b)
}

fn resize(f: &mut FnCtx, v: Val, from: Ty, to: Ty, signed: bool) -> Val {
    if from == to {
        v
    } else if from.size() > to.size() {
        f.b.conv(ConvOp::Trunc, from, to, v)
    } else if signed {
        f.b.conv(ConvOp::SExt, from, to, v)
    } else {
        f.b.conv(ConvOp::ZExt, from, to, v)
    }
}

fn is_zero(f: &mut FnCtx, ty: Ty, v: Val) -> Val {
    let z = konst(f, ty, 0);
    cmp(f, CmpOp::Eq, ty, v, z)
}

fn is_neg(f: &mut FnCtx, ty: Ty, v: Val) -> Val {
    let z = konst(f, ty, 0);
    cmp(f, CmpOp::SLt, ty, v, z)
}

fn flag_not(f: &mut FnCtx, v: Val) -> Val {
    let one = konst(f, Ty::I8, 1);
    bin(f, BinOp::Xor, Ty::I8, v, one)
}

fn flag_or(f: &mut FnCtx, a: Val, b: Val) -> Val {
    bin(f, BinOp::Or, Ty::I8, a, b)
}

fn flag_xor(f: &mut FnCtx, a: Val, b: Val) -> Val {
    bin(f, BinOp::Xor, Ty::I8, a, b)
}

/// `cond ? a : b` for integers of class `ty` without branching.
fn select(f: &mut FnCtx, ty: Ty, cond: Val, a: Val, b: Val) -> Val {
    let wide = resize(f, cond, Ty::I8, ty, false);
    let zero = konst(f, ty, 0);
    let mask = bin(f, BinOp::Sub, ty, zero, wide);
    let inv = f.b.un(UnOp::Not, ty, mask);
    let x = bin(f, BinOp::And, ty, a, mask);
    let y = bin(f, BinOp::And, ty, b, inv);
    bin(f, BinOp::Or, ty, x, y)
}

fn sign_flag_of(f: &mut FnCtx, ty: Ty, a: Val, b: Val, res: Val, sub: bool) -> Val {
    // Add: ((a ^ res) & (b ^ res)) < 0.  Sub: ((a ^ b) & (a ^ res)) < 0.
    let (x, y) = if sub {
        (bin(f, BinOp::Xor, ty, a, b), bin(f, BinOp::Xor, ty, a, res))
    } else {
        (
            bin(f, BinOp::Xor, ty, a, res),
            bin(f, BinOp::Xor, ty, b, res),
        )
    };
    let both = bin(f, BinOp::And, ty, x, y);
    is_neg(f, ty, both)
}

fn bit_intrinsic(f: &mut FnCtx, op: Intrinsic, ty: Ty, x: Val) -> Val {
    let width = konst(f, Ty::I64, bits(ty));
    f.b.intrinsic(op, vec![x, width], &[ty])[0]
}

fn ptr_of(f: &mut FnCtx, addr: Val) -> Val {
    f.b.conv(ConvOp::Bitcast, Ty::I64, Ty::Ptr, addr)
}

/// 64x64 -> 128 multiply: (high, low). Narrower sizes multiply in 64 bits.
fn wide_mul(f: &mut FnCtx, sz: Ty, a: Val, b: Val, signed: bool) -> (Val, Val) {
    if sz != Ty::I64 {
        let a = resize(f, a, sz, Ty::I64, signed);
        let b = resize(f, b, sz, Ty::I64, signed);
        let p = bin(f, BinOp::Mul, Ty::I64, a, b);
        let shift = konst(f, Ty::I64, bits(sz));
        let hi = bin(f, BinOp::LShr, Ty::I64, p, shift);
        return (
            resize(f, hi, Ty::I64, sz, false),
            resize(f, p, Ty::I64, sz, false),
        );
    }
    let m32 = konst(f, Ty::I64, 0xffff_ffff);
    let s32 = konst(f, Ty::I64, 32);
    let a0 = bin(f, BinOp::And, Ty::I64, a, m32);
    let a1 = bin(f, BinOp::LShr, Ty::I64, a, s32);
    let b0 = bin(f, BinOp::And, Ty::I64, b, m32);
    let b1 = bin(f, BinOp::LShr, Ty::I64, b, s32);
    let p00 = bin(f, BinOp::Mul, Ty::I64, a0, b0);
    let p01 = bin(f, BinOp::Mul, Ty::I64, a0, b1);
    let p10 = bin(f, BinOp::Mul, Ty::I64, a1, b0);
    let p11 = bin(f, BinOp::Mul, Ty::I64, a1, b1);
    let t0 = bin(f, BinOp::LShr, Ty::I64, p00, s32);
    let t1 = bin(f, BinOp::And, Ty::I64, p01, m32);
    let t2 = bin(f, BinOp::And, Ty::I64, p10, m32);
    let mid = bin(f, BinOp::Add, Ty::I64, t0, t1);
    let mid = bin(f, BinOp::Add, Ty::I64, mid, t2);
    let carry = bin(f, BinOp::LShr, Ty::I64, mid, s32);
    let u01 = bin(f, BinOp::LShr, Ty::I64, p01, s32);
    let u10 = bin(f, BinOp::LShr, Ty::I64, p10, s32);
    let mut hi = bin(f, BinOp::Add, Ty::I64, p11, u01);
    hi = bin(f, BinOp::Add, Ty::I64, hi, u10);
    hi = bin(f, BinOp::Add, Ty::I64, hi, carry);
    let lo = bin(f, BinOp::Mul, Ty::I64, a, b);
    if signed {
        // High word of the signed product: subtract the operand when the other is negative.
        let s63 = konst(f, Ty::I64, 63);
        let ma = bin(f, BinOp::AShr, Ty::I64, a, s63);
        let mb = bin(f, BinOp::AShr, Ty::I64, b, s63);
        let fix_a = bin(f, BinOp::And, Ty::I64, ma, b);
        let fix_b = bin(f, BinOp::And, Ty::I64, mb, a);
        hi = bin(f, BinOp::Sub, Ty::I64, hi, fix_a);
        hi = bin(f, BinOp::Sub, Ty::I64, hi, fix_b);
    }
    (hi, lo)
}

impl Compiler {
    // -----------------------------------------------------------------------
    // Entry point
    // -----------------------------------------------------------------------

    pub(super) fn check_asm(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        block: &AsmBlock,
    ) -> Result<Operand> {
        for feature in &block.features {
            if !FEATURES.contains(&feature.name.as_str()) {
                return err(
                    feature.span,
                    format!("unsupported #asm feature modifier `{}`", feature.name),
                );
            }
        }
        let has = |prefix: &str| {
            block
                .features
                .iter()
                .any(|f| f.name.as_str().starts_with(prefix))
        };
        let vex = has("AVX");
        let mut cx = AsmCtx {
            scope,
            flags: Flags::default(),
            vec_width: if has("AVX512") {
                64
            } else if vex {
                32
            } else {
                16
            },
            vex,
            df: false,
        };
        for item in &block.items {
            match item {
                AsmItem::Decl(decl) => self.asm_decl_item(f, &cx, decl)?,
                AsmItem::Inst(inst) => self.asm_inst(f, &mut cx, inst)?,
            }
        }
        Ok(Operand::Void)
    }

    // -----------------------------------------------------------------------
    // Registers and operands
    // -----------------------------------------------------------------------

    /// `x: gpr;`, `x: gpr === a;` and `x === a;` as statements.
    fn asm_decl_item(&mut self, f: &mut FnCtx, cx: &AsmCtx, decl: &AsmDecl) -> Result<()> {
        if decl.colon {
            self.asm_declare(f, cx, decl, "gpr")?;
            return Ok(());
        }
        // `x === a` pins an existing variable.
        match self.lookup_full(cx.scope, decl.name.name)? {
            Found::Entities(ids) if !ids.is_empty() => Ok(()),
            _ => Err(Box::new(
                Diagnostic::error(
                    decl.name.span,
                    format!("unknown identifier `{}`", decl.name.name),
                )
                .with_kind(DiagnosticKind::UnknownIdentifier {
                    scope: Some(cx.scope.0),
                }),
            )),
        }
    }

    /// Declare the register named by `decl` in the enclosing scope (reusing an earlier
    /// declaration of the same name in that scope). `default_class` applies to `name:`
    /// without a class (`vec` in a vector operand position).
    fn asm_declare(
        &mut self,
        f: &mut FnCtx,
        cx: &AsmCtx,
        decl: &AsmDecl,
        default_class: &str,
    ) -> Result<(AsmReg, Val)> {
        let class = decl.class.map_or(default_class, |c| c.name.as_str());
        let Some(&(_, kind, size)) = REG_CLASSES.iter().find(|(name, ..)| *name == class) else {
            return err(
                decl.class.map_or(decl.name.span, |c| c.span),
                format!("unknown #asm register class `{class}`"),
            );
        };
        let name = decl.name.name;
        if let Some(ids) = self.scope(cx.scope).names.get(&name)
            && let Some(&existing) = ids.last()
            && self.asm_regs.get(&existing) == Some(&kind)
            && let EntityKind::Local {
                addr, ..
            } = self.entity(existing).kind
        {
            return Ok((kind, addr));
        }
        let addr = f.b.alloca(size, 8);
        f.b.zero(addr, size);
        let depth = self.scope(cx.scope).proc_depth;
        let id = self.add_entity(
            cx.scope,
            name,
            decl.name.span,
            EntityKind::Local {
                ty: TypeId::U64,
                addr,
                depth,
            },
            false,
        );
        self.asm_regs.insert(id, kind);
        Ok((kind, addr))
    }

    /// Errors unless `kind` is a general-purpose register.
    fn asm_require_gpr(kind: AsmReg, span: Span) -> Result<()> {
        match kind {
            AsmReg::Gpr => Ok(()),
            AsmReg::Vec => err(
                span,
                "a vector register is not valid in a general-purpose instruction",
            ),
            AsmReg::Mask => err(
                span,
                "a mask register is only valid in mask (k*) and AVX-512 instructions",
            ),
        }
    }

    fn asm_gpr_opd(&self, kind: AsmReg, addr: Val, span: Span) -> Result<Opd> {
        Self::asm_require_gpr(kind, span)?;
        Ok(Opd::Reg(RegPlace {
            addr,
            storage: Ty::I64,
            signed: false,
            natural: false,
        }))
    }

    fn asm_operand(&mut self, f: &mut FnCtx, cx: &AsmCtx, o: &AsmOperand) -> Result<Opd> {
        match o {
            AsmOperand::Value(e) => self.asm_value(f, cx.scope, e, 0),
            AsmOperand::Mem(m) => Ok(Opd::Mem(self.asm_mem(f, cx.scope, m)?)),
            AsmOperand::Decl(d) if d.colon => {
                let (kind, addr) = self.asm_declare(f, cx, d, "gpr")?;
                self.asm_gpr_opd(kind, addr, d.name.span)
            }
            AsmOperand::Decl(d) => err(
                d.name.span,
                "a register pin is only valid as a separate statement",
            ),
        }
    }

    /// A variable or constant named in an operand position.
    fn asm_value(&mut self, f: &mut FnCtx, scope: ScopeId, e: &Expr, depth: u32) -> Result<Opd> {
        let mut is_gpr = false;
        if let E::Ident(name) = &e.kind
            && let Found::Entities(ids) = self.lookup_full(scope, *name)?
            && let Some(&id) = ids.last()
            && let Some(&kind) = self.asm_regs.get(&id)
        {
            is_gpr = true;
            Self::asm_require_gpr(kind, e.span)?;
        }
        let op = self.check_expr(f, scope, e, None)?;
        match op {
            Operand::Place {
                ty,
                addr,
            } => self.asm_place(ty, addr, !is_gpr, e.span),
            Operand::Const {
                value: Value::Code(code),
                ..
            } => {
                // A `__reg` macro parameter: the caller's operand expression.
                if depth > 8 {
                    return err(e.span, "#asm register argument is nested too deeply");
                }
                let body = self.codes[code.0 as usize].clone();
                let code_scope = self.code_scopes[code.0 as usize];
                match &*body {
                    CodeBody::Expr(inner) => self.asm_value(f, code_scope, inner, depth + 1),
                    CodeBody::Block(_) => err(e.span, "an #asm operand must be a variable"),
                }
            }
            Operand::Const {
                value, ..
            } => match value {
                Value::Int(v) => Ok(Opd::Imm(Imm::Int(v))),
                Value::Float(v) => Ok(Opd::Imm(Imm::Float(v))),
                Value::Bool(v) => Ok(Opd::Imm(Imm::Int(v as i128))),
                Value::Null => Ok(Opd::Imm(Imm::Int(0))),
                _ => err(e.span, "unsupported constant in an #asm operand"),
            },
            other @ Operand::Value {
                ..
            } => {
                // A computed value: usable as a source (the temporary absorbs writes).
                let span = e.span;
                let (ty, val) = self.rvalue(f, other, span)?;
                let addr = self.spill(f, ty, val, span)?;
                self.asm_place(ty, addr, true, span)
            }
            _ => err(e.span, "an #asm operand must be a variable or a constant"),
        }
    }

    fn asm_place(&mut self, ty: TypeId, addr: Val, natural: bool, span: Span) -> Result<Opd> {
        let storage = match self.ir_ty(ty) {
            Some(t) if !t.is_float() => t,
            Some(_) => {
                return err(
                    span,
                    "floating-point variables are not supported in #asm (SIMD/x87 instructions are not lowered)",
                );
            }
            None => {
                return err(
                    span,
                    format!(
                        "#asm operand of type `{}` is not a scalar",
                        self.types.name(ty)
                    ),
                );
            }
        };
        let signed = self.types.int_info(ty).is_some_and(|(_, s)| s);
        Ok(Opd::Reg(RegPlace {
            addr,
            storage,
            signed,
            natural,
        }))
    }

    /// `[base + index*scale +/- disp]` as an `I64` address.
    fn asm_mem(&mut self, f: &mut FnCtx, scope: ScopeId, m: &AsmMem) -> Result<Val> {
        let mut acc: Option<Val> = None;
        let mut disp: i128 = 0;
        for term in &m.terms {
            let scale = match &term.scale {
                Some(e) => match self.asm_value(f, scope, e, 0)? {
                    Opd::Imm(Imm::Int(s)) => s,
                    _ => return err(e.span, "the scale of a memory operand must be a constant"),
                },
                None => 1,
            };
            let sign = if term.negate {
                -1
            } else {
                1
            };
            match self.asm_value(f, scope, &term.value, 0)? {
                Opd::Imm(Imm::Int(c)) => disp = disp.wrapping_add(c.wrapping_mul(scale) * sign),
                Opd::Reg(r) => {
                    let mut v = Self::asm_load_i64(f, r);
                    if scale != 1 {
                        let s = konst(f, Ty::I64, scale as u64);
                        v = bin(f, BinOp::Mul, Ty::I64, v, s);
                    }
                    acc = Some(match acc {
                        None if sign < 0 => {
                            let z = konst(f, Ty::I64, 0);
                            bin(f, BinOp::Sub, Ty::I64, z, v)
                        }
                        None => v,
                        Some(a) if sign < 0 => bin(f, BinOp::Sub, Ty::I64, a, v),
                        Some(a) => bin(f, BinOp::Add, Ty::I64, a, v),
                    });
                }
                _ => {
                    return err(
                        term.value.span,
                        "a memory operand term must be an integer variable or constant",
                    );
                }
            }
        }
        Ok(match acc {
            None => konst(f, Ty::I64, disp as u64),
            Some(a) if disp == 0 => a,
            Some(a) => {
                let d = konst(f, Ty::I64, disp as u64);
                bin(f, BinOp::Add, Ty::I64, a, d)
            }
        })
    }

    /// Load a variable as a 64-bit integer (extended by its own signedness).
    fn asm_load_i64(f: &mut FnCtx, r: RegPlace) -> Val {
        let v = f.b.load(r.storage, r.addr);
        if r.storage == Ty::Ptr {
            return f.b.conv(ConvOp::Bitcast, Ty::Ptr, Ty::I64, v);
        }
        resize(f, v, r.storage, Ty::I64, r.signed)
    }

    fn asm_imm_bits(&self, imm: Imm, sz: Ty, span: Span) -> Result<u64> {
        match imm {
            Imm::Int(v) => {
                let n = bits(sz) as u32;
                let min = -(1i128 << (n - 1));
                let max = (1i128 << n) - 1;
                if v < min || v > max {
                    return err(
                        span,
                        format!(
                            "immediate {v} does not fit in {n} bits (no matching instruction form)"
                        ),
                    );
                }
                let mask = if n == 64 {
                    u64::MAX
                } else {
                    (1u64 << n) - 1
                };
                Ok(v as u64 & mask)
            }
            Imm::Float(x) => match sz {
                Ty::I32 => Ok((x as f32).to_bits() as u64),
                Ty::I64 => Ok(x.to_bits()),
                _ => err(
                    span,
                    "floating-point immediates need a 32- or 64-bit operand size",
                ),
            },
        }
    }

    fn asm_read(&mut self, f: &mut FnCtx, o: Opd, sz: Ty, span: Span) -> Result<Val> {
        Ok(match o {
            Opd::Reg(r) => {
                let v = f.b.load(r.storage, r.addr);
                let (v, from) = if r.storage == Ty::Ptr {
                    (f.b.conv(ConvOp::Bitcast, Ty::Ptr, Ty::I64, v), Ty::I64)
                } else {
                    (v, r.storage)
                };
                resize(f, v, from, sz, false)
            }
            Opd::Imm(imm) => {
                let bits = self.asm_imm_bits(imm, sz, span)?;
                konst(f, sz, bits)
            }
            Opd::Mem(addr) => {
                let p = ptr_of(f, addr);
                f.b.load(sz, p)
            }
        })
    }

    fn asm_write(&mut self, f: &mut FnCtx, o: Opd, sz: Ty, v: Val, span: Span) -> Result<()> {
        match o {
            Opd::Reg(r) => {
                let st = if r.storage == Ty::Ptr {
                    Ty::I64
                } else {
                    r.storage
                };
                let new = if sz == st {
                    v
                } else if sz.size() > st.size() {
                    resize(f, v, sz, st, false)
                } else if sz == Ty::I32 {
                    // 32-bit writes clear the upper half of the register.
                    resize(f, v, sz, st, false)
                } else {
                    // 8- and 16-bit writes keep the rest of the register.
                    let old = f.b.load(r.storage, r.addr);
                    let old = if r.storage == Ty::Ptr {
                        f.b.conv(ConvOp::Bitcast, Ty::Ptr, Ty::I64, old)
                    } else {
                        old
                    };
                    let low = resize(f, v, sz, st, false);
                    let mask = konst(f, st, (1u64 << bits(sz)) - 1);
                    let keep = f.b.un(UnOp::Not, st, mask);
                    let kept = bin(f, BinOp::And, st, old, keep);
                    bin(f, BinOp::Or, st, kept, low)
                };
                let new = if r.storage == Ty::Ptr {
                    f.b.conv(ConvOp::Bitcast, Ty::I64, Ty::Ptr, new)
                } else {
                    new
                };
                f.b.store(r.storage, r.addr, new);
                Ok(())
            }
            Opd::Mem(addr) => {
                let p = ptr_of(f, addr);
                f.b.store(sz, p, v);
                Ok(())
            }
            Opd::Imm(_) => err(
                span,
                "an immediate cannot be the destination of an #asm instruction",
            ),
        }
    }

    // -----------------------------------------------------------------------
    // Read-modify-write
    // -----------------------------------------------------------------------

    /// Start updating `dst`. With `lock` on a memory operand this opens a
    /// compare-and-swap loop that `rmw_end` closes; the value computed in
    /// between must derive only from the returned old value and values
    /// computed before the call.
    fn rmw_begin(
        &mut self,
        f: &mut FnCtx,
        dst: Opd,
        sz: Ty,
        lock: bool,
        span: Span,
    ) -> Result<(Rmw, Val)> {
        if lock && let Opd::Mem(addr) = dst {
            let retry = f.b.new_block();
            let exit = f.b.new_block();
            f.b.jump(retry);
            f.b.switch_to(retry);
            let ptr = ptr_of(f, addr);
            let old = f.b.load(sz, ptr);
            return Ok((
                Rmw {
                    dst,
                    sz,
                    cas: Some((ptr, old, retry, exit)),
                },
                old,
            ));
        }
        let old = self.asm_read(f, dst, sz, span)?;
        Ok((
            Rmw {
                dst,
                sz,
                cas: None,
            },
            old,
        ))
    }

    fn rmw_end(&mut self, f: &mut FnCtx, rmw: Rmw, new: Val, span: Span) -> Result<()> {
        match rmw.cas {
            Some((ptr, old, retry, exit)) => {
                let width = konst(f, Ty::I64, rmw.sz.size());
                let r = f.b.intrinsic(
                    Intrinsic::CompareAndSwap,
                    vec![ptr, old, new, width],
                    &[Ty::I8, rmw.sz],
                );
                f.b.branch(r[0], exit, retry);
                f.b.switch_to(exit);
                Ok(())
            }
            None => self.asm_write(f, rmw.dst, rmw.sz, new, span),
        }
    }

    // -----------------------------------------------------------------------
    // Flags
    // -----------------------------------------------------------------------

    fn flag(f: &mut FnCtx, v: Option<Val>) -> Val {
        v.unwrap_or_else(|| konst(f, Ty::I8, 0))
    }

    fn set_zs(f: &mut FnCtx, cx: &mut AsmCtx, sz: Ty, res: Val) {
        Self::set_zs_into(f, &mut cx.flags, sz, res);
    }

    /// Flags of a logical operation: CF = OF = 0.
    fn set_logic_flags(f: &mut FnCtx, cx: &mut AsmCtx, sz: Ty, res: Val) {
        let zero = konst(f, Ty::I8, 0);
        cx.flags.cf = Some(zero);
        cx.flags.of = Some(zero);
        Self::set_zs(f, cx, sz, res);
    }

    fn eval_cond(f: &mut FnCtx, cx: &AsmCtx, cond: Cond) -> Val {
        let cf = Self::flag(f, cx.flags.cf);
        let zf = Self::flag(f, cx.flags.zf);
        let sf = Self::flag(f, cx.flags.sf);
        let of = Self::flag(f, cx.flags.of);
        match cond {
            Cond::E => zf,
            Cond::Ne => flag_not(f, zf),
            Cond::B => cf,
            Cond::Ae => flag_not(f, cf),
            Cond::Be => flag_or(f, cf, zf),
            Cond::A => {
                let be = flag_or(f, cf, zf);
                flag_not(f, be)
            }
            Cond::S => sf,
            Cond::Ns => flag_not(f, sf),
            Cond::O => of,
            Cond::No => flag_not(f, of),
            Cond::L => flag_xor(f, sf, of),
            Cond::Ge => {
                let l = flag_xor(f, sf, of);
                flag_not(f, l)
            }
            Cond::Le => {
                let l = flag_xor(f, sf, of);
                flag_or(f, zf, l)
            }
            Cond::G => {
                let l = flag_xor(f, sf, of);
                let le = flag_or(f, zf, l);
                flag_not(f, le)
            }
            Cond::P => Self::parity_flag(f, cx),
            Cond::Np => {
                let p = Self::parity_flag(f, cx);
                flag_not(f, p)
            }
        }
    }

    // -----------------------------------------------------------------------
    // Instructions
    // -----------------------------------------------------------------------

    fn asm_inst(&mut self, f: &mut FnCtx, cx: &mut AsmCtx, inst: &AsmInst) -> Result<()> {
        let name = inst.mnemonic.name.as_str();
        let (lock, base) = match name.strip_prefix("lock_") {
            Some(rest) => (true, rest),
            None => (false, name),
        };
        let span = inst.span;
        let Some(op) = lookup_op(base) else {
            if self.asm_scalar_inst(f, cx, inst, base, lock)? {
                return Ok(());
            }
            if !lock
                && (self.asm_mask_inst(f, cx, inst, base)?
                    || self.asm_vec_inst(f, cx, inst, base)?)
            {
                return Ok(());
            }
            return err(
                inst.mnemonic.span,
                format!("unsupported #asm instruction `{name}`"),
            );
        };
        let mut opds = Vec::with_capacity(inst.operands.len());
        for o in &inst.operands {
            opds.push(self.asm_operand(f, cx, o)?);
        }
        let n = opds.len();
        let want = |lo: usize, hi: usize| -> Result<()> {
            if n < lo || n > hi {
                let expected = if lo == hi {
                    lo.to_string()
                } else {
                    format!("{lo} to {hi}")
                };
                return err(
                    span,
                    format!("`{name}` takes {expected} operand(s), found {n}"),
                );
            }
            Ok(())
        };
        match op {
            Op::Nop => {
                want(0, 0)?;
            }
            Op::Pause => {
                want(0, 0)?;
                f.b.intrinsic(Intrinsic::Pause, Vec::new(), &[]);
            }
            Op::Int3 => {
                want(0, 0)?;
                f.b.intrinsic(Intrinsic::DebugBreak, Vec::new(), &[]);
            }
            Op::SetCarry(value) => {
                want(0, 0)?;
                cx.flags.cf = Some(match value {
                    Some(v) => konst(f, Ty::I8, v as u64),
                    None => {
                        let cf = Self::flag(f, cx.flags.cf);
                        flag_not(f, cf)
                    }
                });
            }
            Op::Rdtsc | Op::Rdtscp => {
                // (edx, eax) = counter; rdtscp also writes the processor id (ecx) as 0.
                let max = if op == Op::Rdtscp {
                    3
                } else {
                    2
                };
                want(2, max)?;
                let t =
                    f.b.intrinsic(Intrinsic::CycleCounter, Vec::new(), &[Ty::I64])[0];
                let s32 = konst(f, Ty::I64, 32);
                let hi = bin(f, BinOp::LShr, Ty::I64, t, s32);
                let hi = resize(f, hi, Ty::I64, Ty::I32, false);
                let lo = resize(f, t, Ty::I64, Ty::I32, false);
                self.asm_write(f, opds[0], Ty::I32, hi, span)?;
                self.asm_write(f, opds[1], Ty::I32, lo, span)?;
                if n == 3 {
                    let z = konst(f, Ty::I32, 0);
                    self.asm_write(f, opds[2], Ty::I32, z, span)?;
                }
            }
            Op::Rdrand => {
                // A scrambled cycle counter; CF = 1 reports success.
                want(1, 1)?;
                let sz = self.asm_size(f, cx, inst, &opds)?;
                let t =
                    f.b.intrinsic(Intrinsic::CycleCounter, Vec::new(), &[Ty::I64])[0];
                let k = konst(f, Ty::I64, 0x9E37_79B9_7F4A_7C15);
                let m = bin(f, BinOp::Mul, Ty::I64, t, k);
                let s = konst(f, Ty::I64, 29);
                let h = bin(f, BinOp::LShr, Ty::I64, m, s);
                let v = bin(f, BinOp::Xor, Ty::I64, m, h);
                let v = resize(f, v, Ty::I64, sz, false);
                self.asm_write(f, opds[0], sz, v, span)?;
                let one = konst(f, Ty::I8, 1);
                let zero = konst(f, Ty::I8, 0);
                cx.flags = Flags {
                    cf: Some(one),
                    zf: Some(zero),
                    sf: Some(zero),
                    of: Some(zero),
                    pf: Some(one),
                };
            }
            Op::Cpuid => {
                // No CPU features are reported: eax, ebx, ecx and edx become 0.
                want(4, 4)?;
                for &o in &opds {
                    let z = konst(f, Ty::I32, 0);
                    self.asm_write(f, o, Ty::I32, z, span)?;
                }
            }
            Op::Mov => {
                want(2, 2)?;
                let sz = self.asm_size(f, cx, inst, &opds)?;
                let v = self.asm_read(f, opds[1], sz, span)?;
                self.asm_write(f, opds[0], sz, v, span)?;
            }
            Op::Movbe => {
                want(2, 2)?;
                let sz = self.asm_size(f, cx, inst, &opds)?;
                let v = self.asm_read(f, opds[1], sz, span)?;
                let v = if sz == Ty::I8 {
                    v
                } else {
                    bit_intrinsic(f, Intrinsic::Bswap, sz, v)
                };
                self.asm_write(f, opds[0], sz, v, span)?;
            }
            Op::Movzx | Op::Movsx => {
                want(2, 2)?;
                let (from, to) = Self::asm_extension_sizes(base, inst, span)?;
                let v = self.asm_read(f, opds[1], from, span)?;
                let v = resize(f, v, from, to, op == Op::Movsx);
                self.asm_write(f, opds[0], to, v, span)?;
            }
            Op::Lea => {
                want(2, 2)?;
                let Opd::Mem(addr) = opds[1] else {
                    return err(
                        span,
                        "#asm lea source must be a bracketed address expression",
                    );
                };
                let sz = self.asm_size(f, cx, inst, &opds[..1])?;
                let v = resize(f, addr, Ty::I64, sz, false);
                self.asm_write(f, opds[0], sz, v, span)?;
            }
            Op::Xchg => {
                want(2, 2)?;
                let sz = self.asm_size(f, cx, inst, &opds)?;
                // Exchanging with memory is always atomic.
                let (target, other) = if matches!(opds[1], Opd::Mem(_)) {
                    (opds[1], opds[0])
                } else {
                    (opds[0], opds[1])
                };
                let new = self.asm_read(f, other, sz, span)?;
                let (rmw, old) = self.rmw_begin(f, target, sz, true, span)?;
                self.rmw_end(f, rmw, new, span)?;
                self.asm_write(f, other, sz, old, span)?;
            }
            Op::Xadd => {
                want(2, 2)?;
                let sz = self.asm_size(f, cx, inst, &opds)?;
                let addend = self.asm_read(f, opds[1], sz, span)?;
                let (rmw, old) = self.rmw_begin(f, opds[0], sz, lock, span)?;
                let new = self.asm_alu(f, cx, Alu::Add, sz, old, addend);
                self.rmw_end(f, rmw, new, span)?;
                self.asm_write(f, opds[1], sz, old, span)?;
            }
            Op::Cmpxchg => {
                // `cmpxchg dest, src, acc` or `cmpxchg acc, dest, src` (the memory
                // operand is the destination). The accumulator is explicit because
                // registers are not pinned.
                want(3, 3)?;
                let (dst, src, acc) =
                    if matches!(opds[0], Opd::Mem(_)) || !matches!(opds[1], Opd::Mem(_)) {
                        (opds[0], opds[1], opds[2])
                    } else {
                        (opds[1], opds[2], opds[0])
                    };
                let sz = self.asm_size(f, cx, inst, &opds)?;
                let src_v = self.asm_read(f, src, sz, span)?;
                let acc_v = self.asm_read(f, acc, sz, span)?;
                let (rmw, old) = self.rmw_begin(f, dst, sz, lock, span)?;
                let eq = cmp(f, CmpOp::Eq, sz, old, acc_v);
                let new = select(f, sz, eq, src_v, old);
                self.asm_alu(f, cx, Alu::Cmp, sz, acc_v, old);
                self.rmw_end(f, rmw, new, span)?;
                let acc_new = select(f, sz, eq, acc_v, old);
                self.asm_write(f, acc, sz, acc_new, span)?;
            }
            Op::Alu(alu) => {
                want(2, 2)?;
                let sz = self.asm_size(f, cx, inst, &opds)?;
                let b = self.asm_read(f, opds[1], sz, span)?;
                if matches!(alu, Alu::Cmp | Alu::Test) {
                    let a = self.asm_read(f, opds[0], sz, span)?;
                    self.asm_alu(f, cx, alu, sz, a, b);
                } else {
                    let (rmw, a) = self.rmw_begin(f, opds[0], sz, lock, span)?;
                    let res = self.asm_alu(f, cx, alu, sz, a, b);
                    self.rmw_end(f, rmw, res, span)?;
                }
            }
            Op::Inc | Op::Dec | Op::Neg | Op::Not => {
                want(1, 1)?;
                let sz = self.asm_size(f, cx, inst, &opds)?;
                let (rmw, a) = self.rmw_begin(f, opds[0], sz, lock, span)?;
                let res = match op {
                    Op::Not => f.b.un(UnOp::Not, sz, a),
                    Op::Neg => {
                        let zero = konst(f, sz, 0);
                        self.asm_alu(f, cx, Alu::Sub, sz, zero, a)
                    }
                    _ => {
                        // inc/dec leave CF alone.
                        let one = konst(f, sz, 1);
                        let carry = cx.flags.cf;
                        let alu = if op == Op::Inc {
                            Alu::Add
                        } else {
                            Alu::Sub
                        };
                        let res = self.asm_alu(f, cx, alu, sz, a, one);
                        cx.flags.cf = carry;
                        res
                    }
                };
                self.rmw_end(f, rmw, res, span)?;
            }
            Op::Shift(kind) => {
                want(1, 2)?;
                let sz = self.asm_size(f, cx, inst, &opds[..1])?;
                let count = if n == 2 {
                    Some(opds[1])
                } else {
                    None
                };
                self.asm_shift(f, cx, kind, sz, opds[0], count, lock, span)?;
            }
            Op::Bt(kind) => {
                want(2, 2)?;
                let sz = self.asm_size(f, cx, inst, &opds[..1])?;
                self.asm_bit_test(f, cx, kind, sz, opds[0], opds[1], lock, span)?;
            }
            Op::Bsf | Op::Bsr | Op::Popcnt | Op::Lzcnt | Op::Tzcnt => {
                want(2, 2)?;
                let sz = self.asm_size(f, cx, inst, &opds)?;
                let src = self.asm_read(f, opds[1], sz, span)?;
                let src_zero = is_zero(f, sz, src);
                let zero8 = konst(f, Ty::I8, 0);
                let res = match op {
                    Op::Popcnt => {
                        let odd = konst(f, Ty::I8, 1);
                        cx.flags = Flags {
                            cf: Some(zero8),
                            zf: Some(src_zero),
                            sf: Some(zero8),
                            of: Some(zero8),
                            pf: Some(odd),
                        };
                        bit_intrinsic(f, Intrinsic::Popcount, sz, src)
                    }
                    Op::Lzcnt | Op::Tzcnt => {
                        let res = if op == Op::Lzcnt {
                            bit_intrinsic(f, Intrinsic::Ctlz, sz, src)
                        } else {
                            bit_intrinsic(f, Intrinsic::Cttz, sz, src)
                        };
                        cx.flags.cf = Some(src_zero);
                        cx.flags.zf = Some(is_zero(f, sz, res));
                        res
                    }
                    _ => {
                        // bsf/bsr: the destination is unchanged for a zero source.
                        let found = if op == Op::Bsf {
                            bit_intrinsic(f, Intrinsic::Cttz, sz, src)
                        } else {
                            let lz = bit_intrinsic(f, Intrinsic::Ctlz, sz, src);
                            let top = konst(f, sz, bits(sz) - 1);
                            bin(f, BinOp::Sub, sz, top, lz)
                        };
                        let old = self.asm_read(f, opds[0], sz, span)?;
                        cx.flags.zf = Some(src_zero);
                        select(f, sz, src_zero, old, found)
                    }
                };
                self.asm_write(f, opds[0], sz, res, span)?;
            }
            Op::Bswap => {
                want(1, 1)?;
                let sz = self.asm_size(f, cx, inst, &opds)?;
                let v = self.asm_read(f, opds[0], sz, span)?;
                let v = if sz == Ty::I8 {
                    v
                } else {
                    bit_intrinsic(f, Intrinsic::Bswap, sz, v)
                };
                self.asm_write(f, opds[0], sz, v, span)?;
            }
            Op::Blsr | Op::Blsi | Op::Blsmsk => {
                want(2, 2)?;
                let sz = self.asm_size(f, cx, inst, &opds)?;
                let a = self.asm_read(f, opds[1], sz, span)?;
                let one = konst(f, sz, 1);
                let zero = konst(f, sz, 0);
                let (res, cf) = match op {
                    Op::Blsr => {
                        let m = bin(f, BinOp::Sub, sz, a, one);
                        (bin(f, BinOp::And, sz, a, m), is_zero(f, sz, a))
                    }
                    Op::Blsi => {
                        let m = bin(f, BinOp::Sub, sz, zero, a);
                        let nz = is_zero(f, sz, a);
                        (bin(f, BinOp::And, sz, a, m), flag_not(f, nz))
                    }
                    _ => {
                        let m = bin(f, BinOp::Sub, sz, a, one);
                        (bin(f, BinOp::Xor, sz, a, m), is_zero(f, sz, a))
                    }
                };
                Self::set_logic_flags(f, cx, sz, res);
                cx.flags.cf = Some(cf);
                if op == Op::Blsmsk {
                    cx.flags.zf = Some(konst(f, Ty::I8, 0));
                }
                self.asm_write(f, opds[0], sz, res, span)?;
            }
            Op::Imul | Op::Mul => {
                want(
                    if op == Op::Mul {
                        3
                    } else {
                        2
                    },
                    3,
                )?;
                let sz = self.asm_size(f, cx, inst, &opds)?;
                let signed = op == Op::Imul;
                if n == 2 {
                    // imul dst, src
                    let a = self.asm_read(f, opds[0], sz, span)?;
                    let b = self.asm_read(f, opds[1], sz, span)?;
                    let (hi, lo) = wide_mul(f, sz, a, b, true);
                    self.asm_mul_flags(f, cx, sz, hi, lo, true);
                    self.asm_write(f, opds[0], sz, lo, span)?;
                } else if signed && matches!(opds[2], Opd::Imm(_)) {
                    // imul dst, src, imm
                    let a = self.asm_read(f, opds[1], sz, span)?;
                    let b = self.asm_read(f, opds[2], sz, span)?;
                    let (hi, lo) = wide_mul(f, sz, a, b, true);
                    self.asm_mul_flags(f, cx, sz, hi, lo, true);
                    self.asm_write(f, opds[0], sz, lo, span)?;
                } else {
                    // mul/imul high, low, src: high:low = low * src
                    let a = self.asm_read(f, opds[1], sz, span)?;
                    let b = self.asm_read(f, opds[2], sz, span)?;
                    let (hi, lo) = wide_mul(f, sz, a, b, signed);
                    self.asm_mul_flags(f, cx, sz, hi, lo, signed);
                    self.asm_write(f, opds[1], sz, lo, span)?;
                    self.asm_write(f, opds[0], sz, hi, span)?;
                }
            }
            Op::Setcc(cond) => {
                want(1, 1)?;
                let v = Self::eval_cond(f, cx, cond);
                self.asm_write(f, opds[0], Ty::I8, v, span)?;
            }
            Op::Cmovcc(cond) => {
                want(2, 2)?;
                let sz = self.asm_size(f, cx, inst, &opds)?;
                let c = Self::eval_cond(f, cx, cond);
                let src = self.asm_read(f, opds[1], sz, span)?;
                let old = self.asm_read(f, opds[0], sz, span)?;
                let v = select(f, sz, c, src, old);
                self.asm_write(f, opds[0], sz, v, span)?;
            }
        }
        Ok(())
    }

    /// Sizes of `movzx`/`movsx`: the mnemonic carries `<src><dst>` letters (`movzxbw`).
    fn asm_extension_sizes(base: &str, inst: &AsmInst, span: Span) -> Result<(Ty, Ty)> {
        let letters = base
            .strip_prefix("movzx")
            .or_else(|| base.strip_prefix("movsx"))
            .unwrap_or("");
        let size_of_letter = |c: char| match c {
            'b' => Some(Ty::I8),
            'w' => Some(Ty::I16),
            'd' => Some(Ty::I32),
            'q' => Some(Ty::I64),
            _ => None,
        };
        let chars: Vec<char> = letters.chars().collect();
        match (base, chars.as_slice()) {
            ("movsxd", []) => Ok((Ty::I32, Ty::I64)),
            (_, [s, d]) => match (size_of_letter(*s), size_of_letter(*d)) {
                (Some(s), Some(d)) if s.size() < d.size() => Ok((s, d)),
                _ => err(
                    span,
                    format!("invalid operand sizes in `{}`", inst.mnemonic.name),
                ),
            },
            _ => err(
                span,
                format!(
                    "`{}` needs source and destination sizes (for example movzxbw)",
                    inst.mnemonic.name
                ),
            ),
        }
    }

    /// Instruction operand size: the explicit suffix, else the size of the first Jai
    /// variable operand, else 64 bits.
    fn asm_size(&mut self, f: &mut FnCtx, cx: &AsmCtx, inst: &AsmInst, opds: &[Opd]) -> Result<Ty> {
        match &inst.size {
            Some(AsmSize::Suffix(s)) => match s.name.as_str() {
                "b" | "8" => Ok(Ty::I8),
                "w" | "16" => Ok(Ty::I16),
                "d" | "32" => Ok(Ty::I32),
                "q" | "64" => Ok(Ty::I64),
                other => err(
                    s.span,
                    format!(
                        "unsupported #asm operand size `.{other}` (vector sizes are not supported)"
                    ),
                ),
            },
            Some(AsmSize::Dynamic(e)) => {
                let bits = match self.check_expr(f, cx.scope, e, None)? {
                    Operand::Type(t) => self.size_of(t, e.span)? * 8,
                    Operand::Const {
                        value: Value::Int(v),
                        ..
                    } => v as u64,
                    _ => {
                        return err(
                            e.span,
                            "the size after `?` must be a type or a number of bits",
                        );
                    }
                };
                match bits {
                    8 => Ok(Ty::I8),
                    16 => Ok(Ty::I16),
                    32 => Ok(Ty::I32),
                    64 => Ok(Ty::I64),
                    _ => err(
                        e.span,
                        format!("no #asm instruction form with a {bits}-bit operand"),
                    ),
                }
            }
            None => {
                for o in opds {
                    if let Opd::Reg(r) = o
                        && r.natural
                    {
                        return Ok(Ty::int(r.storage.size()));
                    }
                }
                Ok(Ty::I64)
            }
        }
    }

    /// `a op b` with flags; returns the result (the caller writes it back).
    fn asm_alu(&mut self, f: &mut FnCtx, cx: &mut AsmCtx, op: Alu, sz: Ty, a: Val, b: Val) -> Val {
        let res;
        match op {
            Alu::Add => {
                res = bin(f, BinOp::Add, sz, a, b);
                cx.flags.cf = Some(cmp(f, CmpOp::ULt, sz, res, a));
                cx.flags.of = Some(sign_flag_of(f, sz, a, b, res, false));
            }
            Alu::Adc => {
                let cin = Self::flag(f, cx.flags.cf);
                let cin = resize(f, cin, Ty::I8, sz, false);
                let t = bin(f, BinOp::Add, sz, a, b);
                res = bin(f, BinOp::Add, sz, t, cin);
                let c1 = cmp(f, CmpOp::ULt, sz, t, a);
                let c2 = cmp(f, CmpOp::ULt, sz, res, t);
                cx.flags.cf = Some(flag_or(f, c1, c2));
                cx.flags.of = Some(sign_flag_of(f, sz, a, b, res, false));
            }
            Alu::Sub | Alu::Cmp => {
                res = bin(f, BinOp::Sub, sz, a, b);
                cx.flags.cf = Some(cmp(f, CmpOp::ULt, sz, a, b));
                cx.flags.of = Some(sign_flag_of(f, sz, a, b, res, true));
            }
            Alu::Sbb => {
                let cin = Self::flag(f, cx.flags.cf);
                let cin = resize(f, cin, Ty::I8, sz, false);
                let t = bin(f, BinOp::Sub, sz, a, b);
                res = bin(f, BinOp::Sub, sz, t, cin);
                let c1 = cmp(f, CmpOp::ULt, sz, a, b);
                let c2 = cmp(f, CmpOp::ULt, sz, t, cin);
                cx.flags.cf = Some(flag_or(f, c1, c2));
                cx.flags.of = Some(sign_flag_of(f, sz, a, b, res, true));
            }
            Alu::And | Alu::Test => {
                res = bin(f, BinOp::And, sz, a, b);
                Self::set_logic_flags(f, cx, sz, res);
                return res;
            }
            Alu::Or => {
                res = bin(f, BinOp::Or, sz, a, b);
                Self::set_logic_flags(f, cx, sz, res);
                return res;
            }
            Alu::Xor => {
                res = bin(f, BinOp::Xor, sz, a, b);
                Self::set_logic_flags(f, cx, sz, res);
                return res;
            }
        }
        Self::set_zs(f, cx, sz, res);
        res
    }

    /// CF = OF = "the high half is significant".
    fn asm_mul_flags(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        sz: Ty,
        hi: Val,
        lo: Val,
        signed: bool,
    ) {
        let overflow = if signed {
            let s = konst(f, sz, bits(sz) - 1);
            let ext = bin(f, BinOp::AShr, sz, lo, s);
            cmp(f, CmpOp::Ne, sz, hi, ext)
        } else {
            let z = konst(f, sz, 0);
            cmp(f, CmpOp::Ne, sz, hi, z)
        };
        cx.flags.cf = Some(overflow);
        cx.flags.of = Some(overflow);
    }

    #[allow(clippy::too_many_arguments)]
    fn asm_shift(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        kind: ShiftKind,
        sz: Ty,
        dst: Opd,
        count: Option<Opd>,
        lock: bool,
        span: Span,
    ) -> Result<()> {
        let n = bits(sz);
        // The count is taken mod 32 (mod 64 for 64-bit operands), as on hardware.
        let c = match count {
            None => konst(f, sz, 1),
            Some(o) => {
                let c8 = self.asm_read(f, o, Ty::I8, span)?;
                let c = resize(f, c8, Ty::I8, sz, false);
                let mask = konst(
                    f,
                    sz,
                    if sz == Ty::I64 {
                        63
                    } else {
                        31
                    },
                );
                bin(f, BinOp::And, sz, c, mask)
            }
        };
        let (rmw, x) = self.rmw_begin(f, dst, sz, lock, span)?;
        let one = konst(f, sz, 1);
        let (res, cf, of) = match kind {
            ShiftKind::Shl => {
                let res = bin(f, BinOp::Shl, sz, x, c);
                let width = konst(f, sz, n);
                let back = bin(f, BinOp::Sub, sz, width, c);
                let t = bin(f, BinOp::LShr, sz, x, back);
                let cf = bin(f, BinOp::And, sz, t, one);
                let cf = resize(f, cf, sz, Ty::I8, false);
                let msb = is_neg(f, sz, res);
                (res, cf, flag_xor(f, msb, cf))
            }
            ShiftKind::Shr | ShiftKind::Sar => {
                let op = if kind == ShiftKind::Shr {
                    BinOp::LShr
                } else {
                    BinOp::AShr
                };
                let res = bin(f, op, sz, x, c);
                let prev = bin(f, BinOp::Sub, sz, c, one);
                let t = bin(f, op, sz, x, prev);
                let cf = bin(f, BinOp::And, sz, t, one);
                let cf = resize(f, cf, sz, Ty::I8, false);
                let of = if kind == ShiftKind::Shr {
                    is_neg(f, sz, x)
                } else {
                    konst(f, Ty::I8, 0)
                };
                (res, cf, of)
            }
            ShiftKind::Rol | ShiftKind::Ror => {
                let op = if kind == ShiftKind::Rol {
                    BinOp::Rotl
                } else {
                    BinOp::Rotr
                };
                let res = bin(f, op, sz, x, c);
                let msb = is_neg(f, sz, res);
                if kind == ShiftKind::Rol {
                    let cf = bin(f, BinOp::And, sz, res, one);
                    let cf = resize(f, cf, sz, Ty::I8, false);
                    (res, cf, flag_xor(f, msb, cf))
                } else {
                    let s = konst(f, sz, n - 2);
                    let t = bin(f, BinOp::LShr, sz, res, s);
                    let second = bin(f, BinOp::And, sz, t, one);
                    let second = resize(f, second, sz, Ty::I8, false);
                    (res, msb, flag_xor(f, msb, second))
                }
            }
        };
        self.rmw_end(f, rmw, res, span)?;
        // A zero count leaves the flags alone.
        let mut new = cx.flags;
        new.cf = Some(cf);
        new.of = Some(of);
        if matches!(kind, ShiftKind::Shl | ShiftKind::Shr | ShiftKind::Sar) {
            Self::set_zs_into(f, &mut new, sz, res);
        }
        let nonzero = {
            let z = is_zero(f, sz, c);
            flag_not(f, z)
        };
        let keep = |f: &mut FnCtx, fresh: Option<Val>, old: Option<Val>| match (fresh, old) {
            (Some(n), Some(o)) => Some(select(f, Ty::I8, nonzero, n, o)),
            (Some(n), None) => {
                let z = konst(f, Ty::I8, 0);
                Some(select(f, Ty::I8, nonzero, n, z))
            }
            (None, o) => o,
        };
        cx.flags = Flags {
            cf: keep(f, new.cf, cx.flags.cf),
            zf: keep(f, new.zf, cx.flags.zf),
            sf: keep(f, new.sf, cx.flags.sf),
            of: keep(f, new.of, cx.flags.of),
            pf: keep(f, new.pf, cx.flags.pf),
        };
        Ok(())
    }

    /// ZF, SF and PF of a result.
    fn set_zs_into(f: &mut FnCtx, flags: &mut Flags, sz: Ty, res: Val) {
        flags.zf = Some(is_zero(f, sz, res));
        flags.sf = Some(is_neg(f, sz, res));
        flags.pf = Some(resize(f, res, sz, Ty::I8, false));
    }

    /// PF: 1 when the low byte of the last result has an even number of set bits.
    fn parity_flag(f: &mut FnCtx, cx: &AsmCtx) -> Val {
        let Some(byte) = cx.flags.pf else {
            return konst(f, Ty::I8, 0);
        };
        let ones = bit_intrinsic(f, Intrinsic::Popcount, Ty::I8, byte);
        let one = konst(f, Ty::I8, 1);
        let odd = bin(f, BinOp::And, Ty::I8, ones, one);
        flag_not(f, odd)
    }

    #[allow(clippy::too_many_arguments)]
    fn asm_bit_test(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        kind: BitTest,
        sz: Ty,
        dst: Opd,
        index: Opd,
        lock: bool,
        span: Span,
    ) -> Result<()> {
        let n = bits(sz);
        let idx = self.asm_read(f, index, sz, span)?;
        let mask = konst(f, sz, n - 1);
        let pos = bin(f, BinOp::And, sz, idx, mask);
        // A register bit offset on a memory operand addresses a bit string: the offset
        // is signed and may reach outside the operand.
        let dst = match (dst, index) {
            (Opd::Mem(addr), Opd::Reg(_)) => {
                let wide = resize(f, idx, sz, Ty::I64, true);
                let shift = konst(f, Ty::I64, n.trailing_zeros() as u64);
                let element = bin(f, BinOp::AShr, Ty::I64, wide, shift);
                let bytes = konst(f, Ty::I64, sz.size());
                let off = bin(f, BinOp::Mul, Ty::I64, element, bytes);
                Opd::Mem(bin(f, BinOp::Add, Ty::I64, addr, off))
            }
            _ => dst,
        };
        let writes = kind != BitTest::Bt;
        let (rmw, old) = self.rmw_begin(f, dst, sz, lock && writes, span)?;
        let one = konst(f, sz, 1);
        let shifted = bin(f, BinOp::LShr, sz, old, pos);
        let bit = bin(f, BinOp::And, sz, shifted, one);
        cx.flags.cf = Some(resize(f, bit, sz, Ty::I8, false));
        if writes {
            let m = bin(f, BinOp::Shl, sz, one, pos);
            let new = match kind {
                BitTest::Bts => bin(f, BinOp::Or, sz, old, m),
                BitTest::Btr => {
                    let inv = f.b.un(UnOp::Not, sz, m);
                    bin(f, BinOp::And, sz, old, inv)
                }
                _ => bin(f, BinOp::Xor, sz, old, m),
            };
            self.rmw_end(f, rmw, new, span)?;
        }
        Ok(())
    }
}
