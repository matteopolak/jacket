//! Registers, operations, conditions and the operand and flag state of an `#asm` block.
use crate::ir::{BlockId, Ty, Val};
use crate::sema::*;

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
pub(super) const REG_CLASSES: &[(&str, AsmReg, u64)] = &[
    ("gpr", AsmReg::Gpr, 8),
    ("vec", AsmReg::Vec, 64),
    ("str", AsmReg::Vec, 64),
    ("omr", AsmReg::Mask, 8),
    ("kmask", AsmReg::Mask, 8),
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Alu {
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
pub(super) enum ShiftKind {
    Shl,
    Shr,
    Sar,
    Rol,
    Ror,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum BitTest {
    Bt,
    Bts,
    Btr,
    Btc,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Cond {
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
pub(super) enum Op {
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
    /// `nop`: no observable effect.
    Nop,
    Fence(ir::X86Fence),
    Pause,
    Int3,
    SetCarry(Option<bool>),
    Rdtsc,
    Rdtscp,
    Rdrand,
    Rdseed,
    Cpuid,
    Syscall,
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
pub(super) fn lookup_op(name: &str) -> Option<Op> {
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
        "nop" => Op::Nop,
        "mfence" => Op::Fence(ir::X86Fence::Full),
        "lfence" => Op::Fence(ir::X86Fence::Load),
        "sfence" => Op::Fence(ir::X86Fence::Store),
        "pause" => Op::Pause,
        "int3" => Op::Int3,
        "clc" => Op::SetCarry(Some(false)),
        "stc" => Op::SetCarry(Some(true)),
        "cmc" => Op::SetCarry(None),
        "rdtsc" => Op::Rdtsc,
        "rdtscp" => Op::Rdtscp,
        "rdrand" => Op::Rdrand,
        "rdseed" => Op::Rdseed,
        "cpuid" => Op::Cpuid,
        "syscall" => Op::Syscall,
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
pub(super) struct RegPlace {
    pub(super) addr: Val,
    /// Register class of the storage (`Ptr` for pointer variables).
    pub(super) storage: Ty,
    pub(super) signed: bool,
    /// A Jai variable (its type fixes the operand size) rather than a declared `gpr`.
    pub(super) natural: bool,
}

#[derive(Clone, Copy)]
pub(super) enum Imm {
    Int(i128),
    Float(f64),
}

#[derive(Clone, Copy)]
pub(super) enum Opd {
    Reg(RegPlace),
    Imm(Imm),
    /// Memory at this `I64` address.
    Mem(Val),
}

#[derive(Clone, Copy, Default)]
pub(super) struct Flags {
    pub(super) cf: Option<Val>,
    pub(super) zf: Option<Val>,
    pub(super) sf: Option<Val>,
    pub(super) of: Option<Val>,
    /// PF as a lazily evaluated byte: the flag is set when this `I8` has even parity.
    pub(super) pf: Option<Val>,
}

pub(super) struct AsmCtx {
    pub(super) scope: ScopeId,
    pub(super) flags: Flags,
    /// Default vector operand size in bytes: 16, 32 with AVX, 64 with AVX-512.
    pub(super) vec_width: u64,
    /// VEX/EVEX encoding (AVX and up): vector writes zero the rest of the register.
    pub(super) vex: bool,
    /// The direction flag (`std`/`cld`), tracked statically within the block. The ABI
    /// guarantees it is clear on entry, so a block that never sets it counts upwards.
    pub(super) df: bool,
    /// Float variables named by the vector instruction being lowered, as (register copy, address,
    /// size in bytes): they are vector registers, so the instruction works on a copy and the
    /// destination's low bytes are stored back after it.
    pub(super) float_vars: Vec<(Val, Val, u64)>,
    pub(super) float_dst: Option<(Val, Val, u64)>,
}

/// An in-progress read-modify-write of one operand.
pub(super) struct Rmw {
    pub(super) dst: Opd,
    pub(super) sz: Ty,
    /// Compare-and-swap loop: (pointer, old value, retry block, exit block).
    pub(super) cas: Option<(Val, Val, BlockId, BlockId)>,
}
