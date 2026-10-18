//! Computing the status flags and conditions of an `#asm` instruction.
use super::helpers::{
    bin, bit_intrinsic, flag_not, flag_or, flag_xor, is_neg, is_zero, konst, resize,
};
use super::model::{AsmCtx, Cond, Flags};
use crate::ir::{BinOp, Intrinsic, Ty, Val};
use crate::sema::lower::FnCtx;
use crate::sema::*;

impl Compiler {
    pub(super) fn flag(f: &mut FnCtx, v: Option<Val>) -> Val {
        v.unwrap_or_else(|| konst(f, Ty::I8, 0))
    }

    pub(super) fn set_zs(f: &mut FnCtx, cx: &mut AsmCtx, sz: Ty, res: Val) {
        Self::set_zs_into(f, &mut cx.flags, sz, res);
    }

    /// Flags of a logical operation: CF = OF = 0.
    pub(super) fn set_logic_flags(f: &mut FnCtx, cx: &mut AsmCtx, sz: Ty, res: Val) {
        let zero = konst(f, Ty::I8, 0);
        cx.flags.cf = Some(zero);
        cx.flags.of = Some(zero);
        Self::set_zs(f, cx, sz, res);
    }

    pub(super) fn eval_cond(f: &mut FnCtx, cx: &AsmCtx, cond: Cond) -> Val {
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

    /// ZF, SF and PF of a result.
    pub(super) fn set_zs_into(f: &mut FnCtx, flags: &mut Flags, sz: Ty, res: Val) {
        flags.zf = Some(is_zero(f, sz, res));
        flags.sf = Some(is_neg(f, sz, res));
        flags.pf = Some(resize(f, res, sz, Ty::I8, false));
    }

    /// PF: 1 when the low byte of the last result has an even number of set bits.
    pub(super) fn parity_flag(f: &mut FnCtx, cx: &AsmCtx) -> Val {
        let Some(byte) = cx.flags.pf else {
            return konst(f, Ty::I8, 0);
        };
        let ones = bit_intrinsic(f, Intrinsic::Popcount, Ty::I8, byte);
        let one = konst(f, Ty::I8, 1);
        let odd = bin(f, BinOp::And, Ty::I8, ones, one);
        flag_not(f, odd)
    }

    /// Encode an explicit 0/1 PF bit in the lazy result-byte representation:
    /// byte 1 has odd parity (PF=0), byte 0 has even parity (PF=1).
    pub(super) fn parity_byte_for_bit(f: &mut FnCtx, bit: Val) -> Val {
        flag_not(f, bit)
    }
}
