//! `#asm` lowering.
//!
//! Instructions lower to IR operations on the Jai variables used as operands.
//! Arithmetic and vector operations are portable; architectural queries such as
//! CPUID use explicit intrinsics and require a supported execution host/target.
//! The LLVM backend emits the real instruction for those queries.
//!
//! Model:
//! - Operands are Jai variables (read and written in place), integer
//!   immediates, `[base + index*scale +/- disp]` memory operands, and register
//!   declarations (`tmp:` / `tmp: gpr === a`). A declared `gpr` is a 64-bit
//!   local of the enclosing scope (blocks are not scopes), initialised to zero.
//!   Register pinning is accepted and ignored, except that `syscall` looks up the
//!   declared `gpr` registers pinned to its argument registers.
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
//! - `syscall` is the one place pins matter: it reads the number and arguments from the
//!   registers pinned to `a`, `di`, `si`, `d`, `10`, `8` and `9` (unpinned ones read 0) and
//!   lowers to `Intrinsic::Syscall`.
//! - Anything else (x87, privileged and I/O instructions, ...) is a compile error naming the
//!   instruction.
use self::features::FEATURES;
use self::helpers::{
    bin, bit_intrinsic, bits, cmp, flag_not, flag_or, flag_xor, is_neg, is_zero, konst, ptr_of,
    resize, select, wide_mul,
};
pub use self::model::AsmReg;
use self::model::{
    Alu, AsmArgs, AsmCtx, BitTest, Cond, Flags, Imm, Op, Opd, REG_CLASSES, ShiftKind, Site,
    lookup_op,
};
use super::lower::{FnCtx, Operand};
use super::scope::{EntityKind, Found};
use super::{Compiler, Result, ScopeId, Span, TargetCpu, Value, err, ir};
use crate::ast::{AsmBlock, AsmInst, AsmItem, AsmMem, AsmOperand, AsmSize, Expr, ExprKind as E};
use crate::ir::{BinOp, CmpOp, ConvOp, Intrinsic, Ty, UnOp, Val};

mod alu;
pub mod catalog;
mod features;
mod flags;
mod helpers;
mod inst;
mod mask;
mod model;
mod operands;
mod scalar;
mod simd;
mod vec;

// Small IR helpers

impl Compiler {
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
        self.asm_pins.clear();
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
            float_vars: Vec::new(),
            float_dst: None,
        };
        for item in &block.items {
            match item {
                AsmItem::Decl(decl) => self.asm_decl_item(f, &cx, decl)?,
                AsmItem::Inst(inst) => self.asm_inst(f, &mut cx, inst)?,
            }
        }
        Ok(Operand::Void)
    }
}
