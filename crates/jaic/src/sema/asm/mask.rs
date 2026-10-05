//! AVX-512 op-mask register (`omr`, k0-k7) instructions of `#asm`.
//!
//! A mask register is a 64-bit local (`AsmReg::Mask`). The `b/w/d/q` letter of a
//! `k*` mnemonic is the number of mask bits used (8/16/32/64); results are
//! zero-extended into the whole register, as on hardware. `name:` in a mask
//! position declares an `omr`. Vector instructions read masks through `{k}`
//! decorations (`v: &* k`) and write them from AVX-512 compares (`asm/simd.rs`).
use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum KOp {
    Mov,
    Bin(BinOp),
    AndNot,
    Xnor,
    Add,
    Not,
    ShiftLeft,
    ShiftRight,
    Test,
    OrTest,
    /// `kunpckbw/wd/dq`: concatenate the low halves.
    Unpack,
}

/// A mask instruction's operand: a mask register or anything general-purpose.
#[derive(Clone, Copy)]
pub(super) enum KOpd {
    Mask(Val),
    Other(Opd),
}

fn lookup_kop(name: &str) -> Option<(KOp, Ty)> {
    let letter = |c: &str| -> Option<Ty> {
        Some(match c {
            "b" => Ty::I8,
            "w" => Ty::I16,
            "d" => Ty::I32,
            "q" => Ty::I64,
            _ => return None,
        })
    };
    match name {
        "kunpckbw" => return Some((KOp::Unpack, Ty::I16)),
        "kunpckwd" => return Some((KOp::Unpack, Ty::I32)),
        "kunpckdq" => return Some((KOp::Unpack, Ty::I64)),
        _ => {}
    }
    for (prefix, op) in [
        ("kmov", KOp::Mov),
        ("kandn", KOp::AndNot),
        ("kand", KOp::Bin(BinOp::And)),
        ("kxnor", KOp::Xnor),
        ("kxor", KOp::Bin(BinOp::Xor)),
        ("kortest", KOp::OrTest),
        ("kor", KOp::Bin(BinOp::Or)),
        ("kadd", KOp::Add),
        ("knot", KOp::Not),
        ("kshiftl", KOp::ShiftLeft),
        ("kshiftr", KOp::ShiftRight),
        ("ktest", KOp::Test),
    ] {
        if let Some(rest) = name.strip_prefix(prefix)
            && let Some(ty) = letter(rest)
        {
            return Some((op, ty));
        }
    }
    None
}

impl Compiler {
    /// Lower `inst` if it is a mask-register instruction; false otherwise.
    pub(super) fn asm_mask_inst(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        inst: &AsmInst,
        base: &str,
    ) -> Result<bool> {
        let Some((op, ty)) = lookup_kop(base) else {
            return Ok(false);
        };
        let name = inst.mnemonic.name.as_str();
        let span = inst.span;
        let mut opds = Vec::with_capacity(inst.operands.len());
        for o in &inst.operands {
            opds.push(self.mask_operand(f, cx, o)?);
        }
        let n = opds.len();
        let want = |k: usize| -> Result<()> {
            if n != k {
                return err(span, format!("'{name}' takes {k} operand(s), found {n}"));
            }
            Ok(())
        };
        let need_mask = |o: KOpd| -> Result<Val> {
            match o {
                KOpd::Mask(p) => Ok(p),
                KOpd::Other(_) => err(span, format!("'{name}' needs mask register operands")),
            }
        };
        match op {
            KOp::Mov => {
                want(2)?;
                let v = match opds[1] {
                    KOpd::Mask(p) => {
                        let v = f.b.load(Ty::I64, p);
                        resize(f, v, Ty::I64, ty, false)
                    }
                    KOpd::Other(o) => self.asm_read(f, o, ty, span)?,
                };
                match opds[0] {
                    KOpd::Mask(p) => Self::mask_store(f, p, ty, v),
                    KOpd::Other(o @ Opd::Mem(_)) => self.asm_write(f, o, ty, v, span)?,
                    // To a general-purpose register: zero-extended (32-bit write below .q).
                    KOpd::Other(o) => {
                        let wide = if ty == Ty::I64 {
                            Ty::I64
                        } else {
                            Ty::I32
                        };
                        let v = resize(f, v, ty, wide, false);
                        self.asm_write(f, o, wide, v, span)?;
                    }
                }
            }
            KOp::Bin(_) | KOp::AndNot | KOp::Xnor | KOp::Add | KOp::Unpack => {
                want(3)?;
                let dst = need_mask(opds[0])?;
                let src_ty = if op == KOp::Unpack {
                    Ty::int(ty.size() / 2)
                } else {
                    ty
                };
                let a = Self::mask_load(f, need_mask(opds[1])?, src_ty);
                let b = Self::mask_load(f, need_mask(opds[2])?, src_ty);
                let r = match op {
                    KOp::Bin(o) => bin(f, o, ty, a, b),
                    KOp::AndNot => {
                        let na = f.b.un(UnOp::Not, ty, a);
                        bin(f, BinOp::And, ty, na, b)
                    }
                    KOp::Xnor => {
                        let x = bin(f, BinOp::Xor, ty, a, b);
                        f.b.un(UnOp::Not, ty, x)
                    }
                    KOp::Add => bin(f, BinOp::Add, ty, a, b),
                    _ => {
                        // dst = a[low half] << half | b[low half]
                        let hi = resize(f, a, src_ty, ty, false);
                        let lo = resize(f, b, src_ty, ty, false);
                        let s = konst(f, ty, bits(src_ty));
                        let hi = bin(f, BinOp::Shl, ty, hi, s);
                        bin(f, BinOp::Or, ty, hi, lo)
                    }
                };
                Self::mask_store(f, dst, ty, r);
            }
            KOp::Not => {
                want(2)?;
                let dst = need_mask(opds[0])?;
                let a = Self::mask_load(f, need_mask(opds[1])?, ty);
                let r = f.b.un(UnOp::Not, ty, a);
                Self::mask_store(f, dst, ty, r);
            }
            KOp::ShiftLeft | KOp::ShiftRight => {
                want(3)?;
                let dst = need_mask(opds[0])?;
                let a = Self::mask_load(f, need_mask(opds[1])?, ty);
                let count = match opds[2] {
                    KOpd::Other(Opd::Imm(Imm::Int(c))) if (0..=255).contains(&c) => c as u64,
                    _ => return err(span, format!("'{name}' takes an 8-bit immediate count")),
                };
                // Counts at or above the mask width clear it (IR shifts are total).
                let c = konst(f, ty, count.min(bits(ty)));
                let r = if op == KOp::ShiftLeft {
                    bin(f, BinOp::Shl, ty, a, c)
                } else {
                    bin(f, BinOp::LShr, ty, a, c)
                };
                Self::mask_store(f, dst, ty, r);
            }
            KOp::Test | KOp::OrTest => {
                want(2)?;
                let a = Self::mask_load(f, need_mask(opds[0])?, ty);
                let b = Self::mask_load(f, need_mask(opds[1])?, ty);
                let (zf, cf) = if op == KOp::Test {
                    // ZF: a & b == 0; CF: !a & b == 0.
                    let and = bin(f, BinOp::And, ty, a, b);
                    let na = f.b.un(UnOp::Not, ty, a);
                    let andn = bin(f, BinOp::And, ty, na, b);
                    (is_zero(f, ty, and), is_zero(f, ty, andn))
                } else {
                    // ZF: a | b == 0; CF: a | b all ones.
                    let or = bin(f, BinOp::Or, ty, a, b);
                    let ones = konst(f, ty, u64::MAX);
                    (is_zero(f, ty, or), cmp(f, CmpOp::Eq, ty, or, ones))
                };
                let zero = konst(f, Ty::I8, 0);
                let odd = konst(f, Ty::I8, 1);
                cx.flags = Flags {
                    cf: Some(cf),
                    zf: Some(zf),
                    sf: Some(zero),
                    of: Some(zero),
                    pf: Some(odd),
                };
            }
        }
        Ok(true)
    }

    /// The low `ty` bits of a mask register.
    pub(super) fn mask_load(f: &mut FnCtx, p: Val, ty: Ty) -> Val {
        let v = f.b.load(Ty::I64, p);
        resize(f, v, Ty::I64, ty, false)
    }

    /// Write `ty` bits to a mask register, clearing the bits above.
    pub(super) fn mask_store(f: &mut FnCtx, p: Val, ty: Ty, v: Val) {
        let v = resize(f, v, ty, Ty::I64, false);
        f.b.store(Ty::I64, p, v);
    }

    fn mask_operand(&mut self, f: &mut FnCtx, cx: &AsmCtx, o: &AsmOperand) -> Result<KOpd> {
        match o {
            AsmOperand::Decl(d) if d.colon => {
                let (kind, addr) = self.asm_declare(f, cx, d, "omr")?;
                match kind {
                    AsmReg::Mask => Ok(KOpd::Mask(addr)),
                    _ => Ok(KOpd::Other(self.asm_gpr_opd(kind, addr, d.name.span)?)),
                }
            }
            AsmOperand::Value(e) => match self.asm_reg_named(cx.scope, e)? {
                Some((AsmReg::Mask, addr)) => Ok(KOpd::Mask(addr)),
                Some((AsmReg::Vec, _)) => err(
                    e.span,
                    "a vector register is not valid in a mask instruction",
                ),
                _ => Ok(KOpd::Other(self.asm_operand(f, cx, o)?)),
            },
            _ => Ok(KOpd::Other(self.asm_operand(f, cx, o)?)),
        }
    }
}
