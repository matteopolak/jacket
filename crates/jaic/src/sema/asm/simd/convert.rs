//! Binary lane operations, carry-less multiply and conversions.
use super::dispatch::SimdLocals;
use super::helpers::{aes_tables, float_to_int, saturate};
use super::{B2, Conv};
use crate::diagnostics::Code;
use crate::sema::Output;
use crate::sema::asm::vec::{load_lane, store_lane};
use crate::sema::asm::{
    AsmCtx, AsmSize, BinOp, CmpOp, Compiler, ConvOp, FnCtx, Result, Ty, Val, bin, bits, cmp, err,
    flag_not, flag_or, is_neg, is_zero, konst, resize, select,
};

impl Compiler {
    pub(super) fn simd_bin(f: &mut FnCtx, op: B2, ty: Ty, a: Val, b: Val) -> Val {
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
    pub(super) fn float_predicate(f: &mut FnCtx, ty: Ty, a: Val, b: Val, p: u64) -> Val {
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
    pub(super) fn clmul64(f: &mut FnCtx, a: Val, b: Val) -> (Val, Val) {
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

    /// Conversions (`Conv`).
    pub(super) fn simd_convert(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        c: &SimdLocals<'_>,
        conv: Conv,
    ) -> Result<()> {
        let (inst, ops, width, tmp) = (c.inst, c.ops, c.width, c.tmp);
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
                    _ => {
                        return err(
                            Code::AsmOperandForm,
                            span,
                            format!("`{name}` takes dst, [src,] integer"),
                        );
                    }
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
                    return err(
                        Code::AsmOperandCount,
                        span,
                        format!("`{name}` takes 2 operands"),
                    );
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

impl Output {
    /// Address of the S-box (256 bytes), then the inverse S-box and the field inverse.
    pub(super) fn aes_table_ptr(&mut self, f: &mut FnCtx) -> Val {
        let global = if let Some(g) = self.asm_aes_tables {
            g
        } else {
            let g = self.program.add_global(crate::ir::Global {
                name: crate::sema::private_names::ASM_AES_SBOX.into(),
                size: 768,
                align: 16,
                init: aes_tables(),
                relocs: Vec::new(),
                read_only: true,
                export: None,
            });
            self.asm_aes_tables = Some(g);
            g
        };
        f.b.global_addr(global)
    }
}
