//! Less common general-purpose `#asm` instructions: division, string instructions,
//! accumulator sign extension, double-precision and through-carry shifts, BMI1/BMI2,
//! ADX, CRC32, 8/16-byte compare-exchange and a few hints and system queries.
//!
//! Implicit registers are explicit operands, as in Jai (the order follows the forms
//! Jai prints): `div hi, lo, src` (rdx, rax), `rep_movs.q di, si, c`,
//! `rep_stos.b di, a, c`, `rep_lods.d a, si, c`, `repe_cmps.b di, si, c`,
//! `repne_scas.b di, a, c`, `cqo d, a`, `cdqe a`, `mulx hi, lo, src, d`,
//! `cmpxchg16b d, a, [mem], c, b`.
use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum StrOp {
    Movs,
    Stos,
    Lods,
    Cmps,
    Scas,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Rep {
    Once,
    /// Repeat `c` times.
    Count,
    /// Repeat while equal (ZF = 1).
    Repe,
    /// Repeat while not equal (ZF = 0).
    Repne,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum XOp {
    /// `div` / `idiv` (signed).
    Div(bool),
    /// `cbw` / `cwde` / `cdqe`: sign-extend the low half of the accumulator into this size.
    WidenA(Ty),
    /// `cwd` / `cdq` / `cqo`: fill d with the sign of a (operand size).
    SignFill(Ty),
    /// `shld` (true) / `shrd`.
    DoubleShift(bool),
    /// `rcl` (true) / `rcr`.
    RotateCarry(bool),
    Mulx,
    /// `adcx` (carry through CF) / `adox` (carry through OF).
    AddCarry(bool),
    Andn,
    Bextr,
    Bzhi,
    Pdep,
    Pext,
    /// `shlx` / `shrx` / `sarx`: shifts that leave the flags alone.
    ShiftX(ShiftKind),
    Rorx,
    /// `crc32d` / `crc32q`: CRC-32C into a 32-bit accumulator.
    Crc32,
    Lahf,
    Sahf,
    Xlat,
    /// Cache hints (`prefetch*`, `clflush*`, `clwb`): one memory operand, no effect.
    Hint,
    /// `cld` (false) / `std` (true).
    Direction(bool),
    /// `cmpxchg8b` (I32 halves) / `cmpxchg16b` (I64 halves).
    CmpxchgPair(Ty),
    Str(StrOp, Rep),
    /// `int imm8`: only `int 3` (a breakpoint) is meaningful in user code.
    Int,
    /// `xgetbv d, a, c`: no extended state is reported (both halves 0).
    Xgetbv,
    /// `stmxcsr [mem]`: the default MXCSR (0x1f80); `ldmxcsr` is accepted and ignored.
    Stmxcsr,
    Ldmxcsr,
    /// `rdpid`: processor id 0.
    Rdpid,
}

pub(super) fn lookup_xop(name: &str) -> Option<XOp> {
    Some(match name {
        "div" => XOp::Div(false),
        "idiv" => XOp::Div(true),
        "cbw" => XOp::WidenA(Ty::I16),
        "cwde" => XOp::WidenA(Ty::I32),
        "cdqe" => XOp::WidenA(Ty::I64),
        "cwd" => XOp::SignFill(Ty::I16),
        "cdq" => XOp::SignFill(Ty::I32),
        "cqo" => XOp::SignFill(Ty::I64),
        "shld" => XOp::DoubleShift(true),
        "shrd" => XOp::DoubleShift(false),
        "rcl" => XOp::RotateCarry(true),
        "rcr" => XOp::RotateCarry(false),
        "mulx" => XOp::Mulx,
        "adcx" => XOp::AddCarry(true),
        "adox" => XOp::AddCarry(false),
        "andn" => XOp::Andn,
        "bextr" => XOp::Bextr,
        "bzhi" => XOp::Bzhi,
        "pdep" => XOp::Pdep,
        "pext" => XOp::Pext,
        "shlx" => XOp::ShiftX(ShiftKind::Shl),
        "shrx" => XOp::ShiftX(ShiftKind::Shr),
        "sarx" => XOp::ShiftX(ShiftKind::Sar),
        "rorx" => XOp::Rorx,
        "crc32d" | "crc32q" | "crc32" => XOp::Crc32,
        "lahf" => XOp::Lahf,
        "sahf" => XOp::Sahf,
        "xlat" | "xlatb" => XOp::Xlat,
        "prefetcht0" | "prefetcht1" | "prefetcht2" | "prefetchnta" | "prefetchw"
        | "prefetchwt1" | "clflush" | "clflushopt" | "clwb" | "cldemote" => XOp::Hint,
        "cld" => XOp::Direction(false),
        "std" => XOp::Direction(true),
        "cmpxchg8b" => XOp::CmpxchgPair(Ty::I32),
        "cmpxchg16b" => XOp::CmpxchgPair(Ty::I64),
        "int" => XOp::Int,
        "xgetbv" => XOp::Xgetbv,
        "stmxcsr" => XOp::Stmxcsr,
        "ldmxcsr" => XOp::Ldmxcsr,
        "rdpid" => XOp::Rdpid,
        _ => {
            let (rep, rest) = if let Some(r) = name.strip_prefix("rep_") {
                (Rep::Count, r)
            } else if let Some(r) = name.strip_prefix("repe_").or(name.strip_prefix("repz_")) {
                (Rep::Repe, r)
            } else if let Some(r) = name.strip_prefix("repne_").or(name.strip_prefix("repnz_")) {
                (Rep::Repne, r)
            } else {
                (Rep::Once, name)
            };
            let op = match rest {
                "movs" => StrOp::Movs,
                "stos" => StrOp::Stos,
                "lods" => StrOp::Lods,
                "cmps" => StrOp::Cmps,
                "scas" => StrOp::Scas,
                _ => return None,
            };
            // `rep` goes with moves, stores and loads; `repe`/`repne` with compares and scans.
            let compares = matches!(op, StrOp::Cmps | StrOp::Scas);
            match rep {
                Rep::Count if compares => return None,
                Rep::Repe | Rep::Repne if !compares => return None,
                _ => {}
            }
            XOp::Str(op, rep)
        }
    })
}

fn neg(f: &mut FnCtx, ty: Ty, v: Val) -> Val {
    let z = konst(f, ty, 0);
    bin(f, BinOp::Sub, ty, z, v)
}

/// Unsigned `(hi:lo) / d` for 64-bit halves where `hi < d`: (quotient, remainder).
/// A 64-bit division when `hi` is zero, otherwise restoring long division in a loop.
fn udiv128(f: &mut FnCtx, hi: Val, lo: Val, d: Val) -> (Val, Val) {
    let q_slot = f.b.alloca(8, 8);
    let r_slot = f.b.alloca(8, 8);
    let i_slot = f.b.alloca(8, 8);
    let fast = f.b.new_block();
    let slow = f.b.new_block();
    let head = f.b.new_block();
    let body = f.b.new_block();
    let done = f.b.new_block();
    let small = is_zero(f, Ty::I64, hi);
    f.b.branch(small, fast, slow);

    f.b.switch_to(fast);
    let q = bin(f, BinOp::UDiv, Ty::I64, lo, d);
    let r = bin(f, BinOp::URem, Ty::I64, lo, d);
    f.b.store(Ty::I64, q_slot, q);
    f.b.store(Ty::I64, r_slot, r);
    f.b.jump(done);

    f.b.switch_to(slow);
    f.b.store(Ty::I64, r_slot, hi);
    let zero = konst(f, Ty::I64, 0);
    f.b.store(Ty::I64, q_slot, zero);
    let n = konst(f, Ty::I64, 64);
    f.b.store(Ty::I64, i_slot, n);
    f.b.jump(head);

    f.b.switch_to(head);
    let i = f.b.load(Ty::I64, i_slot);
    let finished = is_zero(f, Ty::I64, i);
    f.b.branch(finished, done, body);

    f.b.switch_to(body);
    let one = konst(f, Ty::I64, 1);
    let i = bin(f, BinOp::Sub, Ty::I64, i, one);
    f.b.store(Ty::I64, i_slot, i);
    let rem = f.b.load(Ty::I64, r_slot);
    let q = f.b.load(Ty::I64, q_slot);
    let s63 = konst(f, Ty::I64, 63);
    let top = bin(f, BinOp::LShr, Ty::I64, rem, s63);
    let bit = bin(f, BinOp::LShr, Ty::I64, lo, i);
    let bit = bin(f, BinOp::And, Ty::I64, bit, one);
    let rem = bin(f, BinOp::Shl, Ty::I64, rem, one);
    let rem = bin(f, BinOp::Or, Ty::I64, rem, bit);
    let q = bin(f, BinOp::Shl, Ty::I64, q, one);
    let carry = cmp(f, CmpOp::Ne, Ty::I64, top, zero);
    let fits = cmp(f, CmpOp::UGe, Ty::I64, rem, d);
    let take = flag_or(f, carry, fits);
    let less = bin(f, BinOp::Sub, Ty::I64, rem, d);
    let rem = select(f, Ty::I64, take, less, rem);
    let take64 = resize(f, take, Ty::I8, Ty::I64, false);
    let q = bin(f, BinOp::Or, Ty::I64, q, take64);
    f.b.store(Ty::I64, r_slot, rem);
    f.b.store(Ty::I64, q_slot, q);
    f.b.jump(head);

    f.b.switch_to(done);
    (f.b.load(Ty::I64, q_slot), f.b.load(Ty::I64, r_slot))
}

/// CRC-32C (Castagnoli, reflected polynomial 0x82f63b78) of `bytes` little-endian bytes of `data`.
fn crc32c(f: &mut FnCtx, crc: Val, data: Val, bytes: u64) -> Val {
    let poly = konst(f, Ty::I32, 0x82f6_3b78);
    let one = konst(f, Ty::I32, 1);
    let zero = konst(f, Ty::I32, 0);
    let mut crc = crc;
    for k in 0..bytes {
        let shift = konst(f, Ty::I64, k * 8);
        let byte = bin(f, BinOp::LShr, Ty::I64, data, shift);
        let mask = konst(f, Ty::I64, 0xff);
        let byte = bin(f, BinOp::And, Ty::I64, byte, mask);
        let byte = resize(f, byte, Ty::I64, Ty::I32, false);
        crc = bin(f, BinOp::Xor, Ty::I32, crc, byte);
        for _ in 0..8 {
            let low = bin(f, BinOp::And, Ty::I32, crc, one);
            let m = bin(f, BinOp::Sub, Ty::I32, zero, low);
            let p = bin(f, BinOp::And, Ty::I32, poly, m);
            let shifted = bin(f, BinOp::LShr, Ty::I32, crc, one);
            crc = bin(f, BinOp::Xor, Ty::I32, shifted, p);
        }
    }
    crc
}

impl Compiler {
    /// Lower `inst` if it is one of the instructions of this file; false otherwise.
    pub(super) fn asm_scalar_inst(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        inst: &AsmInst,
        base: &str,
        lock: bool,
    ) -> Result<bool> {
        let Some(op) = lookup_xop(base) else {
            return Ok(false);
        };
        let name = inst.mnemonic.name.as_str();
        let span = inst.span;
        if lock && !matches!(op, XOp::CmpxchgPair(_)) {
            return err(
                inst.mnemonic.span,
                format!("`{name}` cannot take the lock_ prefix"),
            );
        }
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
            XOp::Div(signed) => {
                want(2, 3)?;
                let sz = self.asm_size(f, cx, inst, &opds)?;
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
                    let (q, r) = self.asm_divide(f, Ty::I8, (hi, lo), d, signed, span);
                    let q = resize(f, q, Ty::I8, Ty::I16, false);
                    let r = resize(f, r, Ty::I8, Ty::I16, false);
                    let r = bin(f, BinOp::Shl, Ty::I16, r, eight);
                    let ax = bin(f, BinOp::Or, Ty::I16, r, q);
                    self.asm_write(f, opds[0], Ty::I16, ax, span)?;
                } else {
                    let hi = self.asm_read(f, opds[0], sz, span)?;
                    let lo = self.asm_read(f, opds[1], sz, span)?;
                    let d = self.asm_read(f, opds[2], sz, span)?;
                    let (q, r) = self.asm_divide(f, sz, (hi, lo), d, signed, span);
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
                let sz = self.asm_size(f, cx, inst, &opds)?;
                let a = self.asm_read(f, opds[3], sz, span)?;
                let b = self.asm_read(f, opds[2], sz, span)?;
                let (hi, lo) = wide_mul(f, sz, a, b, false);
                self.asm_write(f, opds[1], sz, lo, span)?;
                self.asm_write(f, opds[0], sz, hi, span)?;
            }
            XOp::AddCarry(through_cf) => {
                want(2, 2)?;
                let sz = self.asm_size(f, cx, inst, &opds)?;
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
            XOp::Andn => {
                want(3, 3)?;
                let sz = self.asm_size(f, cx, inst, &opds)?;
                let a = self.asm_read(f, opds[1], sz, span)?;
                let b = self.asm_read(f, opds[2], sz, span)?;
                let na = f.b.un(UnOp::Not, sz, a);
                let res = bin(f, BinOp::And, sz, na, b);
                Self::set_logic_flags(f, cx, sz, res);
                self.asm_write(f, opds[0], sz, res, span)?;
            }
            XOp::Bextr | XOp::Bzhi => {
                want(3, 3)?;
                let sz = self.asm_size(f, cx, inst, &opds)?;
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
                let sz = self.asm_size(f, cx, inst, &opds)?;
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
                let sz = self.asm_size(f, cx, inst, &opds)?;
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
                if base == "crc32q" && !matches!(src_sz, Ty::I8 | Ty::I64) {
                    return err(span, "crc32q takes an 8- or 64-bit source");
                }
                let acc = self.asm_read(f, opds[0], Ty::I32, span)?;
                let data = self.asm_read(f, opds[1], src_sz, span)?;
                let data = resize(f, data, src_sz, Ty::I64, false);
                let crc = crc32c(f, acc, data, src_sz.size());
                self.asm_write(f, opds[0], Ty::I32, crc, span)?;
            }
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
                if !matches!(opds[0], Opd::Mem(_)) {
                    return err(span, format!("`{name}` takes a memory operand"));
                }
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
                self.asm_cmpxchg_pair(f, cx, half, addr, &opds, lock, span)?;
            }
            XOp::Str(sop, rep) => self.asm_string(f, cx, inst, sop, rep, &opds)?,
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
                let z = konst(f, Ty::I32, 0);
                self.asm_write(f, opds[0], Ty::I32, z, span)?;
                self.asm_write(f, opds[1], Ty::I32, z, span)?;
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
        }
        Ok(true)
    }

    /// `cond ? new : old` for every flag.
    fn flags_if(f: &mut FnCtx, cond: Val, new: Flags, old: Flags) -> Flags {
        let pick = |f: &mut FnCtx, n: Option<Val>, o: Option<Val>| match (n, o) {
            (Some(n), Some(o)) => Some(select(f, Ty::I8, cond, n, o)),
            (Some(n), None) => {
                let z = konst(f, Ty::I8, 0);
                Some(select(f, Ty::I8, cond, n, z))
            }
            (None, o) => o,
        };
        Flags {
            cf: pick(f, new.cf, old.cf),
            zf: pick(f, new.zf, old.zf),
            sf: pick(f, new.sf, old.sf),
            of: pick(f, new.of, old.of),
            pf: pick(f, new.pf, old.pf),
        }
    }

    /// Branch to a trap (`#DE` on hardware) when `cond` is set.
    fn divide_trap_if(&mut self, f: &mut FnCtx, span: Span, cond: Val) {
        let bad = f.b.new_block();
        let ok = f.b.new_block();
        f.b.branch(cond, bad, ok);
        f.b.switch_to(bad);
        self.emit_trap(f, crate::ir::TRAP_ASM_DIVIDE, span);
        f.b.terminate(crate::ir::Term::Unreachable);
        f.b.switch_to(ok);
    }

    /// Quotient and remainder of the dividend `hi:lo` divided by `d` at size `sz`, trapping
    /// like `#DE` on a zero divisor or a quotient that does not fit.
    fn asm_divide(
        &mut self,
        f: &mut FnCtx,
        sz: Ty,
        (hi, lo): (Val, Val),
        d: Val,
        signed: bool,
        span: Span,
    ) -> (Val, Val) {
        let w = bits(sz);
        // Magnitudes of the dividend (as 64-bit halves) and divisor.
        let (dividend_neg, divisor_neg, hi, lo, d) = if sz == Ty::I64 {
            if signed {
                let dn = is_neg(f, Ty::I64, hi);
                let sn = is_neg(f, Ty::I64, d);
                let nlo = neg(f, Ty::I64, lo);
                let lo_zero = is_zero(f, Ty::I64, lo);
                let borrow = resize(f, lo_zero, Ty::I8, Ty::I64, false);
                let nhi = f.b.un(UnOp::Not, Ty::I64, hi);
                let nhi = bin(f, BinOp::Add, Ty::I64, nhi, borrow);
                let lo = select(f, Ty::I64, dn, nlo, lo);
                let hi = select(f, Ty::I64, dn, nhi, hi);
                let nd = neg(f, Ty::I64, d);
                let d = select(f, Ty::I64, sn, nd, d);
                (Some(dn), Some(sn), hi, lo, d)
            } else {
                (None, None, hi, lo, d)
            }
        } else {
            // The whole dividend fits in 64 bits.
            let h = resize(f, hi, sz, Ty::I64, signed);
            let l = resize(f, lo, sz, Ty::I64, false);
            let s = konst(f, Ty::I64, w);
            let h = bin(f, BinOp::Shl, Ty::I64, h, s);
            let v = bin(f, BinOp::Or, Ty::I64, h, l);
            let d = resize(f, d, sz, Ty::I64, signed);
            let zero = konst(f, Ty::I64, 0);
            if signed {
                let dn = is_neg(f, Ty::I64, v);
                let sn = is_neg(f, Ty::I64, d);
                let nv = neg(f, Ty::I64, v);
                let v = select(f, Ty::I64, dn, nv, v);
                let nd = neg(f, Ty::I64, d);
                let d = select(f, Ty::I64, sn, nd, d);
                (Some(dn), Some(sn), zero, v, d)
            } else {
                (None, None, zero, v, d)
            }
        };
        let zero = konst(f, Ty::I64, 0);
        let by_zero = cmp(f, CmpOp::Eq, Ty::I64, d, zero);
        self.divide_trap_if(f, span, by_zero);
        // Unsigned magnitude limit: the quotient must stay below 2^w (2^(w-1) + sign when signed).
        let (q, r) = if sz == Ty::I64 {
            let too_big = cmp(f, CmpOp::UGe, Ty::I64, hi, d);
            self.divide_trap_if(f, span, too_big);
            udiv128(f, hi, lo, d)
        } else {
            (
                bin(f, BinOp::UDiv, Ty::I64, lo, d),
                bin(f, BinOp::URem, Ty::I64, lo, d),
            )
        };
        match (dividend_neg, divisor_neg) {
            (Some(dn), Some(sn)) => {
                let qneg = flag_xor(f, dn, sn);
                let max_pos = konst(f, Ty::I64, (1u64 << (w - 1)) - 1);
                let max_neg = konst(f, Ty::I64, 1u64 << (w - 1));
                let limit = select(f, Ty::I64, qneg, max_neg, max_pos);
                let over = cmp(f, CmpOp::UGt, Ty::I64, q, limit);
                self.divide_trap_if(f, span, over);
                let nq = neg(f, Ty::I64, q);
                let q = select(f, Ty::I64, qneg, nq, q);
                let nr = neg(f, Ty::I64, r);
                let r = select(f, Ty::I64, dn, nr, r);
                (
                    resize(f, q, Ty::I64, sz, false),
                    resize(f, r, Ty::I64, sz, false),
                )
            }
            _ => {
                if sz != Ty::I64 {
                    let limit = konst(f, Ty::I64, (1u64 << w) - 1);
                    let over = cmp(f, CmpOp::UGt, Ty::I64, q, limit);
                    self.divide_trap_if(f, span, over);
                }
                (
                    resize(f, q, Ty::I64, sz, false),
                    resize(f, r, Ty::I64, sz, false),
                )
            }
        }
    }

    /// `shld`/`shrd` by a count in 1..width (masked by the caller): (result, CF).
    fn double_shift(f: &mut FnCtx, sz: Ty, dst: Val, src: Val, c: Val, left: bool) -> (Val, Val) {
        let w = bits(sz);
        let c = resize(f, c, Ty::I64, sz, false);
        let width = konst(f, sz, w);
        let back = bin(f, BinOp::Sub, sz, width, c);
        let one = konst(f, sz, 1);
        // Counts at or above the width are undefined on hardware; IR shifts give 0 there.
        let (res, out) = if left {
            let a = bin(f, BinOp::Shl, sz, dst, c);
            let b = bin(f, BinOp::LShr, sz, src, back);
            let res = bin(f, BinOp::Or, sz, a, b);
            (res, bin(f, BinOp::LShr, sz, dst, back))
        } else {
            let a = bin(f, BinOp::LShr, sz, dst, c);
            let b = bin(f, BinOp::Shl, sz, src, back);
            let res = bin(f, BinOp::Or, sz, a, b);
            let prev = bin(f, BinOp::Sub, sz, c, one);
            (res, bin(f, BinOp::LShr, sz, dst, prev))
        };
        let cf = bin(f, BinOp::And, sz, out, one);
        (res, resize(f, cf, sz, Ty::I8, false))
    }

    /// `rcl`/`rcr` of `x` with carry-in `cin` by `c` in 0..=width: (result, CF).
    fn rotate_carry(f: &mut FnCtx, sz: Ty, x: Val, cin: Val, c: Val, left: bool) -> (Val, Val) {
        let w = bits(sz);
        // Rotate the (width + 1)-bit value CF:x, as 128-bit halves when width is 64.
        let c = resize(f, c, Ty::I64, sz, false);
        let one = konst(f, sz, 1);
        let cin = resize(f, cin, Ty::I8, sz, false);
        let width = konst(f, sz, w);
        let zero_count = is_zero(f, sz, c);
        if left {
            // res = x << c | cin << (c-1) | x >> (w+1-c); cf = bit (w-c) of x.
            let a = bin(f, BinOp::Shl, sz, x, c);
            let cm1 = bin(f, BinOp::Sub, sz, c, one);
            let b = bin(f, BinOp::Shl, sz, cin, cm1);
            let wp1 = bin(f, BinOp::Add, sz, width, one);
            let back = bin(f, BinOp::Sub, sz, wp1, c);
            let back_ok = cmp(f, CmpOp::ULt, sz, back, width);
            let d = bin(f, BinOp::LShr, sz, x, back);
            let zero = konst(f, sz, 0);
            let d = select(f, sz, back_ok, d, zero);
            let res = bin(f, BinOp::Or, sz, a, b);
            let res = bin(f, BinOp::Or, sz, res, d);
            let pos = bin(f, BinOp::Sub, sz, width, c);
            let out = bin(f, BinOp::LShr, sz, x, pos);
            let out = bin(f, BinOp::And, sz, out, one);
            let out = resize(f, out, sz, Ty::I8, false);
            let cin8 = resize(f, cin, sz, Ty::I8, false);
            let res = select(f, sz, zero_count, x, res);
            (res, select(f, Ty::I8, zero_count, cin8, out))
        } else {
            // res = x >> c | cin << (w-c) | x << (w+1-c); cf = bit (c-1) of x.
            let a = bin(f, BinOp::LShr, sz, x, c);
            let wmc = bin(f, BinOp::Sub, sz, width, c);
            let b = bin(f, BinOp::Shl, sz, cin, wmc);
            let wp1 = bin(f, BinOp::Add, sz, width, one);
            let back = bin(f, BinOp::Sub, sz, wp1, c);
            let back_ok = cmp(f, CmpOp::ULt, sz, back, width);
            let d = bin(f, BinOp::Shl, sz, x, back);
            let zero = konst(f, sz, 0);
            let d = select(f, sz, back_ok, d, zero);
            let res = bin(f, BinOp::Or, sz, a, b);
            let res = bin(f, BinOp::Or, sz, res, d);
            let cm1 = bin(f, BinOp::Sub, sz, c, one);
            let out = bin(f, BinOp::LShr, sz, x, cm1);
            let out = bin(f, BinOp::And, sz, out, one);
            let out = resize(f, out, sz, Ty::I8, false);
            let cin8 = resize(f, cin, sz, Ty::I8, false);
            let res = select(f, sz, zero_count, x, res);
            (res, select(f, Ty::I8, zero_count, cin8, out))
        }
    }

    /// `pdep` (deposit) / `pext` (extract) over the set bits of `mask`, in a loop.
    fn deposit_extract(f: &mut FnCtx, src: Val, mask: Val, deposit: bool) -> Val {
        let m_slot = f.b.alloca(8, 8);
        let k_slot = f.b.alloca(8, 8);
        let r_slot = f.b.alloca(8, 8);
        let zero = konst(f, Ty::I64, 0);
        let one = konst(f, Ty::I64, 1);
        f.b.store(Ty::I64, m_slot, mask);
        f.b.store(Ty::I64, k_slot, one);
        f.b.store(Ty::I64, r_slot, zero);
        let head = f.b.new_block();
        let body = f.b.new_block();
        let done = f.b.new_block();
        f.b.jump(head);
        f.b.switch_to(head);
        let m = f.b.load(Ty::I64, m_slot);
        let empty = is_zero(f, Ty::I64, m);
        f.b.branch(empty, done, body);
        f.b.switch_to(body);
        // `low` is the lowest set mask bit, `k` the matching bit of the packed value.
        let nm = neg(f, Ty::I64, m);
        let low = bin(f, BinOp::And, Ty::I64, m, nm);
        let k = f.b.load(Ty::I64, k_slot);
        let r = f.b.load(Ty::I64, r_slot);
        let (test, set) = if deposit {
            (k, low)
        } else {
            (low, k)
        };
        let hit = bin(f, BinOp::And, Ty::I64, src, test);
        let hit = cmp(f, CmpOp::Ne, Ty::I64, hit, zero);
        let add = select(f, Ty::I64, hit, set, zero);
        let r = bin(f, BinOp::Or, Ty::I64, r, add);
        f.b.store(Ty::I64, r_slot, r);
        let k = bin(f, BinOp::Shl, Ty::I64, k, one);
        f.b.store(Ty::I64, k_slot, k);
        let m1 = bin(f, BinOp::Sub, Ty::I64, m, one);
        let m = bin(f, BinOp::And, Ty::I64, m, m1);
        f.b.store(Ty::I64, m_slot, m);
        f.b.jump(head);
        f.b.switch_to(done);
        f.b.load(Ty::I64, r_slot)
    }

    /// `cmpxchg8b`/`cmpxchg16b d, a, [mem], c, b`: when [mem] == d:a store c:b and set ZF,
    /// otherwise load [mem] into d:a and clear ZF. With `lock_` the exchange holds a
    /// process-wide spin lock, so it is atomic with respect to other locked 8/16-byte
    /// exchanges (not to plain atomics on the same memory).
    #[allow(clippy::too_many_arguments)]
    fn asm_cmpxchg_pair(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        half: Ty,
        addr: Val,
        opds: &[Opd],
        lock: bool,
        span: Span,
    ) -> Result<()> {
        let lock_ptr = if lock {
            let global = *self.asm_pair_lock.get_or_insert_with(|| {
                self.program.add_global(crate::ir::Global {
                    name: "__jaic_asm_cmpxchg_lock".into(),
                    size: 8,
                    align: 8,
                    init: Vec::new(),
                    relocs: Vec::new(),
                    read_only: false,
                    export: None,
                })
            });
            let p = f.b.global_addr(global);
            let spin = f.b.new_block();
            let held = f.b.new_block();
            f.b.jump(spin);
            f.b.switch_to(spin);
            let zero = konst(f, Ty::I64, 0);
            let one = konst(f, Ty::I64, 1);
            let width = konst(f, Ty::I64, 8);
            let r = f.b.intrinsic(
                Intrinsic::CompareAndSwap,
                vec![p, zero, one, width],
                &[Ty::I8, Ty::I64],
            );
            f.b.branch(r[0], held, spin);
            f.b.switch_to(held);
            Some(p)
        } else {
            None
        };
        let hsz = half.size();
        let lo_addr = addr;
        let off = konst(f, Ty::I64, hsz);
        let hi_addr = bin(f, BinOp::Add, Ty::I64, addr, off);
        let mem_lo = self.asm_read(f, Opd::Mem(lo_addr), half, span)?;
        let mem_hi = self.asm_read(f, Opd::Mem(hi_addr), half, span)?;
        let d = self.asm_read(f, opds[0], half, span)?;
        let a = self.asm_read(f, opds[1], half, span)?;
        let c = self.asm_read(f, opds[3], half, span)?;
        let b = self.asm_read(f, opds[4], half, span)?;
        let eq_lo = cmp(f, CmpOp::Eq, half, mem_lo, a);
        let eq_hi = cmp(f, CmpOp::Eq, half, mem_hi, d);
        let eq = bin(f, BinOp::And, Ty::I8, eq_lo, eq_hi);
        let new_lo = select(f, half, eq, b, mem_lo);
        let new_hi = select(f, half, eq, c, mem_hi);
        self.asm_write(f, Opd::Mem(lo_addr), half, new_lo, span)?;
        self.asm_write(f, Opd::Mem(hi_addr), half, new_hi, span)?;
        if let Some(p) = lock_ptr {
            let zero = konst(f, Ty::I64, 0);
            let one = konst(f, Ty::I64, 1);
            let width = konst(f, Ty::I64, 8);
            f.b.intrinsic(
                Intrinsic::CompareAndSwap,
                vec![p, one, zero, width],
                &[Ty::I8, Ty::I64],
            );
        }
        // On failure the accumulator pair receives the memory value (unchanged on success).
        self.asm_write(f, opds[1], half, mem_lo, span)?;
        self.asm_write(f, opds[0], half, mem_hi, span)?;
        cx.flags.zf = Some(eq);
        Ok(())
    }

    /// String instructions: one element, or `rep*`-prefixed with a count register.
    fn asm_string(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        inst: &AsmInst,
        op: StrOp,
        rep: Rep,
        opds: &[Opd],
    ) -> Result<()> {
        let span = inst.span;
        let name = inst.mnemonic.name.as_str();
        let want = if rep == Rep::Once {
            2
        } else {
            3
        };
        if opds.len() != want {
            let regs = match op {
                StrOp::Movs | StrOp::Cmps => "di, si",
                StrOp::Stos | StrOp::Scas => "di, a",
                StrOp::Lods => "a, si",
            };
            let count = if rep == Rep::Once {
                ""
            } else {
                ", c"
            };
            return err(span, format!("`{name}` takes the operands {regs}{count}"));
        }
        let sz = match &inst.size {
            Some(_) => self.asm_size(f, cx, inst, &[])?,
            None => {
                return err(
                    span,
                    format!("`{name}` needs an element size (.b/.w/.d/.q)"),
                );
            }
        };
        for &o in opds {
            if !matches!(o, Opd::Reg(_)) {
                return err(span, format!("the operands of `{name}` must be registers"));
            }
        }
        let step = konst(
            f,
            Ty::I64,
            if cx.df {
                sz.size().wrapping_neg()
            } else {
                sz.size()
            },
        );
        // Flags survive the loop in slots (a zero count leaves them unchanged).
        let compares = matches!(op, StrOp::Cmps | StrOp::Scas);
        let flag_slots = if compares {
            let mut slots = [None; 5];
            let current = [
                cx.flags.cf,
                cx.flags.zf,
                cx.flags.sf,
                cx.flags.of,
                cx.flags.pf,
            ];
            for (slot, v) in slots.iter_mut().zip(current) {
                let p = f.b.alloca(1, 1);
                let v = v.unwrap_or_else(|| konst(f, Ty::I8, 0));
                f.b.store(Ty::I8, p, v);
                *slot = Some(p);
            }
            Some(slots.map(Option::unwrap))
        } else {
            None
        };
        let exit = f.b.new_block();
        let body = f.b.new_block();
        let head = if rep == Rep::Once {
            f.b.jump(body);
            None
        } else {
            let head = f.b.new_block();
            f.b.jump(head);
            f.b.switch_to(head);
            let c = self.asm_read(f, opds[2], Ty::I64, span)?;
            let done = is_zero(f, Ty::I64, c);
            f.b.branch(done, exit, body);
            Some(head)
        };
        f.b.switch_to(body);
        // Operand roles: (di, si) for movs/cmps, (di, a) for stos/scas, (a, si) for lods.
        let advance = |this: &mut Self, f: &mut FnCtx, o: Opd| -> Result<()> {
            let p = this.asm_read(f, o, Ty::I64, span)?;
            let p = bin(f, BinOp::Add, Ty::I64, p, step);
            this.asm_write(f, o, Ty::I64, p, span)
        };
        match op {
            StrOp::Movs => {
                let di = self.asm_read(f, opds[0], Ty::I64, span)?;
                let si = self.asm_read(f, opds[1], Ty::I64, span)?;
                let v = self.asm_read(f, Opd::Mem(si), sz, span)?;
                self.asm_write(f, Opd::Mem(di), sz, v, span)?;
                advance(self, f, opds[0])?;
                advance(self, f, opds[1])?;
            }
            StrOp::Stos => {
                let di = self.asm_read(f, opds[0], Ty::I64, span)?;
                let v = self.asm_read(f, opds[1], sz, span)?;
                self.asm_write(f, Opd::Mem(di), sz, v, span)?;
                advance(self, f, opds[0])?;
            }
            StrOp::Lods => {
                let si = self.asm_read(f, opds[1], Ty::I64, span)?;
                let v = self.asm_read(f, Opd::Mem(si), sz, span)?;
                self.asm_write(f, opds[0], sz, v, span)?;
                advance(self, f, opds[1])?;
            }
            StrOp::Cmps | StrOp::Scas => {
                // cmps: [si] - [di]; scas: a - [di].
                let di = self.asm_read(f, opds[0], Ty::I64, span)?;
                let right = self.asm_read(f, Opd::Mem(di), sz, span)?;
                let left = if op == StrOp::Cmps {
                    let si = self.asm_read(f, opds[1], Ty::I64, span)?;
                    self.asm_read(f, Opd::Mem(si), sz, span)?
                } else {
                    self.asm_read(f, opds[1], sz, span)?
                };
                self.asm_alu(f, cx, Alu::Cmp, sz, left, right);
                advance(self, f, opds[0])?;
                if op == StrOp::Cmps {
                    advance(self, f, opds[1])?;
                }
                let slots = flag_slots.unwrap();
                let now = [
                    cx.flags.cf,
                    cx.flags.zf,
                    cx.flags.sf,
                    cx.flags.of,
                    cx.flags.pf,
                ];
                for (p, v) in slots.into_iter().zip(now) {
                    f.b.store(Ty::I8, p, v.unwrap());
                }
            }
        }
        match head {
            None => f.b.jump(exit),
            Some(head) => {
                let c = self.asm_read(f, opds[2], Ty::I64, span)?;
                let one = konst(f, Ty::I64, 1);
                let c = bin(f, BinOp::Sub, Ty::I64, c, one);
                self.asm_write(f, opds[2], Ty::I64, c, span)?;
                match rep {
                    Rep::Repe | Rep::Repne => {
                        let zf = cx.flags.zf.unwrap();
                        let (more, stop) = if rep == Rep::Repe {
                            (head, exit)
                        } else {
                            (exit, head)
                        };
                        f.b.branch(zf, more, stop);
                    }
                    _ => f.b.jump(head),
                }
            }
        }
        f.b.switch_to(exit);
        if let Some(slots) = flag_slots {
            let [cf, zf, sf, of, pf] = slots.map(|p| f.b.load(Ty::I8, p));
            cx.flags = Flags {
                cf: Some(cf),
                zf: Some(zf),
                sf: Some(sf),
                of: Some(of),
                pf: Some(pf),
            };
        }
        Ok(())
    }
}
