//! The string-compare instructions.
use super::dispatch::SimdLocals;
use crate::diagnostics::Code;
use crate::sema::asm::vec::{VOpd, store_lane};
use crate::sema::asm::{
    AsmCtx, BinOp, CmpOp, Compiler, Flags, FnCtx, Intrinsic, Result, Ty, bin, bit_intrinsic, cmp,
    err, konst, resize, select,
};

impl Compiler {
    /// `pcmpistri`, `pcmpestri`, `pcmpistrm`, `pcmpestrm`: string comparison of two 16-byte
    /// operands. The operands are Jai's explicit form: `ecx` (or `xmm0`), then for the explicit
    /// length forms `eax` and `edx`, the two vectors and the control byte. The control byte is
    /// a constant, so it is part of the `StringCompare` intrinsic, and the index, mask and flags
    /// are taken from the intrinsic's result bits here.
    pub(super) fn asm_pcmpstr(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        c: &SimdLocals<'_>,
        (explicit, mask): (bool, bool),
    ) -> Result<()> {
        let (ops, span, name) = (c.ops, c.span, c.name);
        let want = if explicit {
            6
        } else {
            4
        };

        if ops.len() != want {
            return err(
                Code::AsmOperandCount,
                span,
                format!("`{name}` takes {want} operands"),
            );
        }

        let Some(VOpd::Imm(ctl)) = ops.last().copied() else {
            return err(
                Code::AsmNeeds8Bit,
                span,
                format!("`{name}` needs an 8-bit control constant"),
            );
        };

        if !(0..=255).contains(&ctl) {
            return err(
                Code::AsmNeeds8Bit,
                span,
                format!("`{name}` needs an 8-bit control constant"),
            );
        }

        let ctl = ctl as u8;
        let ty = if ctl & 1 != 0 {
            Ty::I16
        } else {
            Ty::I8
        };
        let n = 16 / ty.numeric_size();
        let first = if explicit {
            3
        } else {
            1
        };
        let pa = self.vec_ptr(f, ops[first], span)?;
        let pb = self.vec_ptr(f, ops[first + 1], span)?;
        let (la, lb) = if explicit {
            (
                self.vec_scalar_read(f, ops[1], Ty::I32, span)?,
                self.vec_scalar_read(f, ops[2], Ty::I32, span)?,
            )
        } else {
            let zero = konst(f, Ty::I32, 0);
            (zero, zero)
        };
        // One intrinsic, not an expansion per element: the n x n element compares of equal any
        // and equal ordered made blocks long enough for LLVM's machine passes to take minutes.
        let r = f.b.intrinsic(
            Intrinsic::StringCompare(ctl, explicit),
            vec![pa, pb, la, lb],
            &[Ty::I32, Ty::I8, Ty::I8],
        );
        let (res, b_ends, a_ends) = (r[0], r[1], r[2]);
        let zero = konst(f, Ty::I32, 0);
        let any = cmp(f, CmpOp::Ne, Ty::I32, res, zero);

        if mask {
            let tmp = f.b.alloca(16, 16);

            f.b.zero(tmp, 16);

            if ctl & 0x40 != 0 {
                // One element of all ones per set bit.
                for i in 0..n {
                    let at = konst(f, Ty::I32, i);
                    let moved = bin(f, BinOp::LShr, Ty::I32, res, at);
                    let one = konst(f, Ty::I32, 1);
                    let bit = bin(f, BinOp::And, Ty::I32, moved, one);
                    let bit = resize(f, bit, Ty::I32, ty, false);
                    let z = konst(f, ty, 0);
                    let ones = bin(f, BinOp::Sub, ty, z, bit);

                    store_lane(f, tmp, i, ty, ones);
                }
            } else {
                let bits_set = resize(f, res, Ty::I32, Ty::I16, false);

                store_lane(f, tmp, 0, Ty::I16, bits_set);
            }

            self.simd_out(f, cx, ops[0], tmp, 16, span)?;
        } else {
            // The index of the highest (or lowest) set element; the element count when none.
            let idx = if ctl & 0x40 != 0 {
                let lz = bit_intrinsic(f, Intrinsic::Ctlz, Ty::I32, res);
                let top = konst(f, Ty::I32, 31);
                let high = bin(f, BinOp::Sub, Ty::I32, top, lz);
                let none = konst(f, Ty::I32, n);

                select(f, Ty::I32, any, high, none)
            } else {
                let stop = konst(f, Ty::I32, 1 << n);
                let guarded = bin(f, BinOp::Or, Ty::I32, res, stop);

                bit_intrinsic(f, Intrinsic::Cttz, Ty::I32, guarded)
            };

            self.vec_scalar_write(f, ops[0], Ty::I32, idx, span)?;
        }

        let one = konst(f, Ty::I32, 1);
        let low = bin(f, BinOp::And, Ty::I32, res, one);
        // AF and PF are cleared (PF is the lazily evaluated parity byte, odd = clear).
        let odd = konst(f, Ty::I8, 1);

        cx.flags = Flags {
            cf: Some(any),
            zf: Some(b_ends),
            sf: Some(a_ends),
            of: Some(resize(f, low, Ty::I32, Ty::I8, false)),
            pf: Some(odd),
        };
        Ok(())
    }
}
