//! Arithmetic, shift and bit-test instructions.
use super::helpers::{
    bin, bits, cmp, flag_not, flag_or, flag_xor, is_neg, is_zero, konst, resize, select,
    sign_flag_of,
};
use super::model::{Alu, AsmCtx, BitTest, Flags, Opd, ShiftKind};
use crate::ir::{BinOp, CmpOp, Ty, UnOp, Val};
use crate::sema::lower::FnCtx;
use crate::sema::*;

impl Compiler {
    /// `a op b` with flags; returns the result (the caller writes it back).
    pub(super) fn asm_alu(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        op: Alu,
        sz: Ty,
        a: Val,
        b: Val,
    ) -> Val {
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
    pub(super) fn asm_mul_flags(
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
    pub(super) fn asm_shift(
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

    #[allow(clippy::too_many_arguments)]
    pub(super) fn asm_bit_test(
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
