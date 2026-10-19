//! The less common scalar instructions that `asm_scalar_inst` hands to a group by opcode.
use super::{
    AsmArgs, AsmCtx, AsmInst, BinOp, CmpOp, Compiler, FnCtx, Imm, Intrinsic, Opd, Result,
    ShiftKind, TargetCpu, Ty, UnOp, Val, XOp, bin, bits, cmp, crc32c, err, flag_not, flag_or,
    flag_xor, ir, is_neg, is_zero, konst, ptr_of, resize, select, wide_mul,
};

impl Compiler {
    pub(super) fn scalar_arith(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        inst: &AsmInst,
        c: &AsmArgs,
        op: XOp,
    ) -> Result<()> {
        let span = c.span;
        let name = c.name;
        let opds = &c.opds;
        let n = c.opds.len();
        let want = |lo: usize, hi: usize| c.want(lo, hi);
        match op {
            XOp::Div(signed) => {
                want(2, 3)?;
                let sz = self.asm_size(f, cx, inst, opds)?;
                if n == 2 {
                    // `div.b ax, src`: ax / src -> al (quotient), ah (remainder).
                    if sz != Ty::I8 {
                        return err(span, format!("`{name}` takes 3 operands (hi, lo, divisor)"));
                    }
                    let ax = self.asm_read(f, opds[0], Ty::I16, span)?;
                    let eight = konst(f, Ty::I16, 8);
                    let hi = bin(f, BinOp::LShr, Ty::I16, ax, eight);
                    let hi = resize(f, hi, Ty::I16, Ty::I8, false);
                    let lo = resize(f, ax, Ty::I16, Ty::I8, false);
                    let d = self.asm_read(f, opds[1], Ty::I8, span)?;
                    let (q, r) = self.asm_divide(f, Ty::I8, (hi, lo), d, signed, span)?;
                    let q = resize(f, q, Ty::I8, Ty::I16, false);
                    let r = resize(f, r, Ty::I8, Ty::I16, false);
                    let r = bin(f, BinOp::Shl, Ty::I16, r, eight);
                    let ax = bin(f, BinOp::Or, Ty::I16, r, q);
                    self.asm_write(f, opds[0], Ty::I16, ax, span)?;
                } else {
                    let hi = self.asm_read(f, opds[0], sz, span)?;
                    let lo = self.asm_read(f, opds[1], sz, span)?;
                    let d = self.asm_read(f, opds[2], sz, span)?;
                    let (q, r) = self.asm_divide(f, sz, (hi, lo), d, signed, span)?;
                    self.asm_write(f, opds[1], sz, q, span)?;
                    self.asm_write(f, opds[0], sz, r, span)?;
                }
            }
            XOp::WidenA(to) => {
                want(1, 1)?;
                let from = Ty::int(to.size() / 2);
                let v = self.asm_read(f, opds[0], from, span)?;
                let v = resize(f, v, from, to, true);
                self.asm_write(f, opds[0], to, v, span)?;
            }
            XOp::SignFill(sz) => {
                want(2, 2)?;
                let a = self.asm_read(f, opds[1], sz, span)?;
                let s = konst(f, sz, bits(sz) - 1);
                let fill = bin(f, BinOp::AShr, sz, a, s);
                self.asm_write(f, opds[0], sz, fill, span)?;
            }
            XOp::DoubleShift(left) => {
                want(3, 3)?;
                let sz = self.asm_size(f, cx, inst, &opds[..2])?;
                let c = self.asm_read(f, opds[2], Ty::I8, span)?;
                let c = resize(f, c, Ty::I8, Ty::I64, false);
                let m = konst(
                    f,
                    Ty::I64,
                    if sz == Ty::I64 {
                        63
                    } else {
                        31
                    },
                );
                let c = bin(f, BinOp::And, Ty::I64, c, m);
                let dst = self.asm_read(f, opds[0], sz, span)?;
                let src = self.asm_read(f, opds[1], sz, span)?;
                // Shift the double-width value dst:src (shld) or src:dst (shrd), as a pair of
                // 64-bit halves; counts above the width are undefined on hardware.
                let (res, cf) = Self::double_shift(f, sz, dst, src, c, left);
                let nonzero = {
                    let z = is_zero(f, Ty::I64, c);
                    flag_not(f, z)
                };
                let res = select(f, sz, nonzero, res, dst);
                self.asm_write(f, opds[0], sz, res, span)?;
                let mut new = cx.flags;
                Self::set_zs_into(f, &mut new, sz, res);
                new.cf = Some(cf);
                let a = is_neg(f, sz, res);
                let b = is_neg(f, sz, dst);
                new.of = Some(flag_xor(f, a, b));
                cx.flags = Self::flags_if(f, nonzero, new, cx.flags);
            }
            XOp::RotateCarry(left) => {
                want(1, 2)?;
                let sz = self.asm_size(f, cx, inst, &opds[..1])?;
                let w = bits(sz);
                let c = if n == 2 {
                    let c = self.asm_read(f, opds[1], Ty::I8, span)?;
                    let c = resize(f, c, Ty::I8, Ty::I64, false);
                    let m = konst(
                        f,
                        Ty::I64,
                        if sz == Ty::I64 {
                            63
                        } else {
                            31
                        },
                    );
                    let c = bin(f, BinOp::And, Ty::I64, c, m);
                    // Rotating through carry has a period of width + 1.
                    let period = konst(f, Ty::I64, w + 1);
                    bin(f, BinOp::URem, Ty::I64, c, period)
                } else {
                    konst(f, Ty::I64, 1)
                };
                let x = self.asm_read(f, opds[0], sz, span)?;
                let cin = Self::flag(f, cx.flags.cf);
                let (res, cf) = Self::rotate_carry(f, sz, x, cin, c, left);
                self.asm_write(f, opds[0], sz, res, span)?;
                let nonzero = {
                    let z = is_zero(f, Ty::I64, c);
                    flag_not(f, z)
                };
                let mut new = cx.flags;
                new.cf = Some(cf);
                let msb = is_neg(f, sz, res);
                new.of = Some(if left {
                    flag_xor(f, msb, cf)
                } else {
                    let s = konst(f, sz, w - 2);
                    let t = bin(f, BinOp::LShr, sz, res, s);
                    let one = konst(f, sz, 1);
                    let next = bin(f, BinOp::And, sz, t, one);
                    let next = resize(f, next, sz, Ty::I8, false);
                    flag_xor(f, msb, next)
                });
                cx.flags = Self::flags_if(f, nonzero, new, cx.flags);
            }
            XOp::Mulx => {
                want(4, 4)?;
                let sz = self.asm_size(f, cx, inst, opds)?;
                let a = self.asm_read(f, opds[3], sz, span)?;
                let b = self.asm_read(f, opds[2], sz, span)?;
                let (hi, lo) = wide_mul(f, sz, a, b, false);
                self.asm_write(f, opds[1], sz, lo, span)?;
                self.asm_write(f, opds[0], sz, hi, span)?;
            }
            XOp::AddCarry(through_cf) => {
                want(2, 2)?;
                let sz = self.asm_size(f, cx, inst, opds)?;
                let a = self.asm_read(f, opds[0], sz, span)?;
                let b = self.asm_read(f, opds[1], sz, span)?;
                let cin = Self::flag(
                    f,
                    if through_cf {
                        cx.flags.cf
                    } else {
                        cx.flags.of
                    },
                );
                let cin = resize(f, cin, Ty::I8, sz, false);
                let t = bin(f, BinOp::Add, sz, a, b);
                let res = bin(f, BinOp::Add, sz, t, cin);
                let c1 = cmp(f, CmpOp::ULt, sz, t, a);
                let c2 = cmp(f, CmpOp::ULt, sz, res, t);
                let cout = Some(flag_or(f, c1, c2));
                if through_cf {
                    cx.flags.cf = cout;
                } else {
                    cx.flags.of = cout;
                }
                self.asm_write(f, opds[0], sz, res, span)?;
            }
            _ => unreachable!("dispatched only for this group's operations"),
        }
        Ok(())
    }

    /// Bit-manipulation instructions: `andn`, `bextr`, `bzhi`, `pdep`, `pext`, `shlx`/`shrx`/`sarx`, `rorx`, `crc32`.
    pub(super) fn scalar_bits(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        inst: &AsmInst,
        c: &AsmArgs,
        op: XOp,
    ) -> Result<()> {
        let span = c.span;
        let opds = &c.opds;
        let want = |lo: usize, hi: usize| c.want(lo, hi);
        match op {
            XOp::Andn => {
                want(3, 3)?;
                let sz = self.asm_size(f, cx, inst, opds)?;
                let a = self.asm_read(f, opds[1], sz, span)?;
                let b = self.asm_read(f, opds[2], sz, span)?;
                let na = f.b.un(UnOp::Not, sz, a);
                let res = bin(f, BinOp::And, sz, na, b);
                Self::set_logic_flags(f, cx, sz, res);
                self.asm_write(f, opds[0], sz, res, span)?;
            }
            XOp::Bextr | XOp::Bzhi => {
                want(3, 3)?;
                let sz = self.asm_size(f, cx, inst, opds)?;
                let src = self.asm_read(f, opds[1], sz, span)?;
                let ctl = self.asm_read(f, opds[2], sz, span)?;
                let src = resize(f, src, sz, Ty::I64, false);
                let ctl = resize(f, ctl, sz, Ty::I64, false);
                let byte = konst(f, Ty::I64, 0xff);
                let w = konst(f, Ty::I64, bits(sz));
                let zero = konst(f, Ty::I64, 0);
                let ones = konst(f, Ty::I64, u64::MAX);
                // Keep the low `len` bits of `v` (all of them when len >= 64).
                let keep_low = |f: &mut FnCtx, v: Val, len: Val| {
                    let big = cmp(f, CmpOp::UGe, Ty::I64, len, w);
                    let mask = bin(f, BinOp::Shl, Ty::I64, ones, len);
                    let mask = f.b.un(UnOp::Not, Ty::I64, mask);
                    let mask = select(f, Ty::I64, big, ones, mask);
                    bin(f, BinOp::And, Ty::I64, v, mask)
                };
                let (res, cf) = if op == XOp::Bextr {
                    let start = bin(f, BinOp::And, Ty::I64, ctl, byte);
                    let eight = konst(f, Ty::I64, 8);
                    let len = bin(f, BinOp::LShr, Ty::I64, ctl, eight);
                    let len = bin(f, BinOp::And, Ty::I64, len, byte);
                    let past = cmp(f, CmpOp::UGe, Ty::I64, start, w);
                    let shifted = bin(f, BinOp::LShr, Ty::I64, src, start);
                    let shifted = select(f, Ty::I64, past, zero, shifted);
                    (keep_low(f, shifted, len), konst(f, Ty::I8, 0))
                } else {
                    let idx = bin(f, BinOp::And, Ty::I64, ctl, byte);
                    let top = konst(f, Ty::I64, bits(sz) - 1);
                    let cf = cmp(f, CmpOp::UGt, Ty::I64, idx, top);
                    (keep_low(f, src, idx), cf)
                };
                let res = resize(f, res, Ty::I64, sz, false);
                Self::set_logic_flags(f, cx, sz, res);
                cx.flags.cf = Some(cf);
                self.asm_write(f, opds[0], sz, res, span)?;
            }
            XOp::Pdep | XOp::Pext => {
                want(3, 3)?;
                let sz = self.asm_size(f, cx, inst, opds)?;
                let src = self.asm_read(f, opds[1], sz, span)?;
                let mask = self.asm_read(f, opds[2], sz, span)?;
                let src = resize(f, src, sz, Ty::I64, false);
                let mask = resize(f, mask, sz, Ty::I64, false);
                let res = Self::deposit_extract(f, src, mask, op == XOp::Pdep);
                let res = resize(f, res, Ty::I64, sz, false);
                self.asm_write(f, opds[0], sz, res, span)?;
            }
            XOp::ShiftX(kind) => {
                want(3, 3)?;
                let sz = self.asm_size(f, cx, inst, opds)?;
                let src = self.asm_read(f, opds[1], sz, span)?;
                let c = self.asm_read(f, opds[2], sz, span)?;
                let m = konst(f, sz, bits(sz) - 1);
                let c = bin(f, BinOp::And, sz, c, m);
                let op = match kind {
                    ShiftKind::Shl => BinOp::Shl,
                    ShiftKind::Shr => BinOp::LShr,
                    _ => BinOp::AShr,
                };
                let res = bin(f, op, sz, src, c);
                self.asm_write(f, opds[0], sz, res, span)?;
            }
            XOp::Rorx => {
                want(3, 3)?;
                let sz = self.asm_size(f, cx, inst, &opds[..2])?;
                let src = self.asm_read(f, opds[1], sz, span)?;
                let c = self.asm_read(f, opds[2], Ty::I8, span)?;
                let c = resize(f, c, Ty::I8, sz, false);
                let m = konst(f, sz, bits(sz) - 1);
                let c = bin(f, BinOp::And, sz, c, m);
                let res = bin(f, BinOp::Rotr, sz, src, c);
                self.asm_write(f, opds[0], sz, res, span)?;
            }
            XOp::Crc32 => {
                // `crc32d.b acc, src`: the suffix is the source size; crc32q needs `.q`.
                want(2, 2)?;
                let src_sz = self.asm_size(f, cx, inst, &opds[1..])?;
                if c.base == "crc32q" && !matches!(src_sz, Ty::I8 | Ty::I64) {
                    return err(span, "crc32q takes an 8- or 64-bit source");
                }
                let acc = self.asm_read(f, opds[0], Ty::I32, span)?;
                let data = self.asm_read(f, opds[1], src_sz, span)?;
                let data = resize(f, data, src_sz, Ty::I64, false);
                let crc = crc32c(f, acc, data, src_sz.size());
                self.asm_write(f, opds[0], Ty::I32, crc, span)?;
            }
            _ => unreachable!("dispatched only for this group's operations"),
        }
        Ok(())
    }

    pub(super) fn scalar_machine(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        inst: &AsmInst,
        c: &AsmArgs,
        op: XOp,
    ) -> Result<()> {
        let span = c.span;
        let name = c.name;
        let opds = &c.opds;
        let want = |lo: usize, hi: usize| c.want(lo, hi);
        match op {
            XOp::Lahf | XOp::Sahf => {
                // AH = SF:ZF:0:AF:0:PF:1:CF (AF is not modeled and reads as 0).
                want(1, 1)?;
                if op == XOp::Lahf {
                    let parts = [(cx.flags.cf, 0u64), (cx.flags.zf, 6), (cx.flags.sf, 7)];
                    let mut ah = konst(f, Ty::I64, 0b10);
                    for (flag, at) in parts {
                        if let Some(v) = flag {
                            let v = resize(f, v, Ty::I8, Ty::I64, false);
                            let s = konst(f, Ty::I64, at);
                            let v = bin(f, BinOp::Shl, Ty::I64, v, s);
                            ah = bin(f, BinOp::Or, Ty::I64, ah, v);
                        }
                    }
                    let pf = Self::parity_flag(f, cx);
                    let pf = resize(f, pf, Ty::I8, Ty::I64, false);
                    let two = konst(f, Ty::I64, 2);
                    let pf = bin(f, BinOp::Shl, Ty::I64, pf, two);
                    let ah = bin(f, BinOp::Or, Ty::I64, ah, pf);
                    let eight = konst(f, Ty::I64, 8);
                    let ah = bin(f, BinOp::Shl, Ty::I64, ah, eight);
                    let old = self.asm_read(f, opds[0], Ty::I64, span)?;
                    let keep = konst(f, Ty::I64, !0xff00);
                    let old = bin(f, BinOp::And, Ty::I64, old, keep);
                    let v = bin(f, BinOp::Or, Ty::I64, old, ah);
                    self.asm_write(f, opds[0], Ty::I64, v, span)?;
                } else {
                    let a = self.asm_read(f, opds[0], Ty::I64, span)?;
                    let bit = |f: &mut FnCtx, at: u64| {
                        let s = konst(f, Ty::I64, at + 8);
                        let v = bin(f, BinOp::LShr, Ty::I64, a, s);
                        let one = konst(f, Ty::I64, 1);
                        let v = bin(f, BinOp::And, Ty::I64, v, one);
                        resize(f, v, Ty::I64, Ty::I8, false)
                    };
                    cx.flags.cf = Some(bit(f, 0));
                    let pf = bit(f, 2);
                    cx.flags.pf = Some(flag_not(f, pf));
                    cx.flags.zf = Some(bit(f, 6));
                    cx.flags.sf = Some(bit(f, 7));
                }
            }
            XOp::Xlat => {
                // `xlat table, a`: al = [table + al].
                want(2, 2)?;
                let table = self.asm_read(f, opds[0], Ty::I64, span)?;
                let al = self.asm_read(f, opds[1], Ty::I8, span)?;
                let at = resize(f, al, Ty::I8, Ty::I64, false);
                let addr = bin(f, BinOp::Add, Ty::I64, table, at);
                let v = self.asm_read(f, Opd::Mem(addr), Ty::I8, span)?;
                self.asm_write(f, opds[1], Ty::I8, v, span)?;
            }
            XOp::Hint => {
                want(1, 1)?;
                let Opd::Mem(address) = opds[0] else {
                    return err(span, format!("`{name}` takes a memory operand"));
                };
                if self.options.cpu != TargetCpu::X64 {
                    return err(span, format!("{name} requires an x86-64 target"));
                }
                let hint = match name {
                    "prefetcht0" => ir::X86Prefetch::T0,
                    "prefetcht1" => ir::X86Prefetch::T1,
                    "prefetcht2" => ir::X86Prefetch::T2,
                    "prefetchnta" => ir::X86Prefetch::NonTemporal,
                    _ => return err(span, format!("{name} cache operation is not implemented")),
                };
                let pointer = ptr_of(f, address);
                f.b.intrinsic(Intrinsic::X86Prefetch(hint), vec![pointer], &[]);
            }
            XOp::Direction(set) => {
                want(0, 0)?;
                cx.df = set;
            }
            XOp::CmpxchgPair(half) => {
                want(5, 5)?;
                let Opd::Mem(addr) = opds[2] else {
                    return err(span, format!("`{name}` compares a memory operand ([ptr])"));
                };
                self.asm_cmpxchg_pair(f, cx, half, addr, opds, c.lock, span)?;
            }
            XOp::Str(sop, rep) => self.asm_string(f, cx, inst, sop, rep, opds)?,
            XOp::Int => {
                want(1, 1)?;
                match opds[0] {
                    Opd::Imm(Imm::Int(3)) => {
                        f.b.intrinsic(Intrinsic::DebugBreak, Vec::new(), &[]);
                    }
                    _ => {
                        return err(
                            span,
                            "only 'int 3' is supported (software interrupts have no meaning outside the OS)",
                        );
                    }
                }
            }
            XOp::Xgetbv => {
                want(3, 3)?;
                if self.options.cpu != TargetCpu::X64 {
                    return err(span, "xgetbv requires an x86-64 target");
                }
                if opds
                    .iter()
                    .any(|o| !matches!(o, Opd::Reg(r) if matches!(r.storage, Ty::I32 | Ty::I64)))
                {
                    return err(
                        span,
                        "xgetbv operands must be 32- or 64-bit integer registers",
                    );
                }
                let index = self.asm_read(f, opds[2], Ty::I32, span)?;
                let pair =
                    f.b.intrinsic(Intrinsic::X86Xgetbv, vec![index], &[Ty::I32; 2]);
                self.asm_write(f, opds[0], Ty::I32, pair[0], span)?;
                self.asm_write(f, opds[1], Ty::I32, pair[1], span)?;
            }
            XOp::Stmxcsr | XOp::Ldmxcsr => {
                want(1, 1)?;
                if !matches!(opds[0], Opd::Mem(_)) {
                    return err(span, format!("`{name}` takes a memory operand"));
                }
                if op == XOp::Stmxcsr {
                    let v = konst(f, Ty::I32, 0x1f80);
                    self.asm_write(f, opds[0], Ty::I32, v, span)?;
                }
            }
            XOp::Rdpid => {
                want(1, 1)?;
                let z = konst(f, Ty::I64, 0);
                self.asm_write(f, opds[0], Ty::I64, z, span)?;
            }
            _ => unreachable!("dispatched only for this group's operations"),
        }
        Ok(())
    }
}
