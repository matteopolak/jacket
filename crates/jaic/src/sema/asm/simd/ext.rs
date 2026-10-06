//! F16C half-precision conversions, the SHA-1/SHA-256 message and round
//! instructions, and GFNI (Galois field) byte arithmetic.
//!
//! Like the rest of the vector set these are lowered to plain integer IR, so they
//! give the same bits in the interpreter and in native code on any host. MXCSR is
//! not modeled: `cvtps2ph` with imm8 bit 2 ("use MXCSR.RC") rounds to nearest even,
//! which is the MXCSR default.
use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::sema) enum ShaOp {
    Sha1Rnds4,
    Sha1Nexte,
    Sha1Msg1,
    Sha1Msg2,
    /// Two SHA-256 rounds; the implicit `xmm0` (WK) is an explicit last operand.
    Sha256Rnds2,
    Sha256Msg1,
    Sha256Msg2,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::sema) enum GfOp {
    /// `gf2p8mulb`: bytes multiplied modulo x^8 + x^4 + x^3 + x + 1.
    Mul,
    /// `gf2p8affineqb` / `gf2p8affineinvqb` (true: invert each byte first).
    Affine(bool),
}

fn c32(f: &mut FnCtx, v: u64) -> Val {
    konst(f, Ty::I32, v)
}

fn op32(f: &mut FnCtx, op: BinOp, a: Val, b: Val) -> Val {
    bin(f, op, Ty::I32, a, b)
}

fn op32k(f: &mut FnCtx, op: BinOp, a: Val, k: u64) -> Val {
    let k = c32(f, k);
    bin(f, op, Ty::I32, a, k)
}

fn add_all(f: &mut FnCtx, vals: &[Val]) -> Val {
    let mut acc = vals[0];
    for &v in &vals[1..] {
        acc = op32(f, BinOp::Add, acc, v);
    }
    acc
}

fn xor_all(f: &mut FnCtx, vals: &[Val]) -> Val {
    let mut acc = vals[0];
    for &v in &vals[1..] {
        acc = op32(f, BinOp::Xor, acc, v);
    }
    acc
}

fn rotl(f: &mut FnCtx, x: Val, n: u64) -> Val {
    op32k(f, BinOp::Rotl, x, n)
}

fn rotr(f: &mut FnCtx, x: Val, n: u64) -> Val {
    op32k(f, BinOp::Rotr, x, n)
}

/// `(x ror a) ^ (x ror b) ^ (x ror c | x >> c)`: the four SHA-256 sigma functions.
fn sigma(f: &mut FnCtx, x: Val, a: u64, b: u64, c: u64, shift_last: bool) -> Val {
    let p = rotr(f, x, a);
    let q = rotr(f, x, b);
    let r = if shift_last {
        op32k(f, BinOp::LShr, x, c)
    } else {
        rotr(f, x, c)
    };
    xor_all(f, &[p, q, r])
}

/// Bitwise choose: `x ? y : z` per bit.
fn choose(f: &mut FnCtx, x: Val, y: Val, z: Val) -> Val {
    let xy = op32(f, BinOp::And, x, y);
    let nx = f.b.un(UnOp::Not, Ty::I32, x);
    let nxz = op32(f, BinOp::And, nx, z);
    op32(f, BinOp::Xor, xy, nxz)
}

/// Bitwise majority of three words.
fn majority(f: &mut FnCtx, x: Val, y: Val, z: Val) -> Val {
    let xy = op32(f, BinOp::And, x, y);
    let xz = op32(f, BinOp::And, x, z);
    let yz = op32(f, BinOp::And, y, z);
    xor_all(f, &[xy, xz, yz])
}

fn flag_and(f: &mut FnCtx, a: Val, b: Val) -> Val {
    bin(f, BinOp::And, Ty::I8, a, b)
}

/// The bits of an IEEE half (`I16`) as a single (`I32`). Exact: every half is
/// representable; a signaling NaN comes back quiet with its payload kept.
fn half_to_single(f: &mut FnCtx, h: Val) -> Val {
    let h = resize(f, h, Ty::I16, Ty::I32, false);
    let sign = op32k(f, BinOp::And, h, 0x8000);
    let sign = op32k(f, BinOp::Shl, sign, 16);
    let exp = op32k(f, BinOp::LShr, h, 10);
    let exp = op32k(f, BinOp::And, exp, 0x1f);
    let man = op32k(f, BinOp::And, h, 0x3ff);
    let frac = op32k(f, BinOp::Shl, man, 13);

    // Normal: rebias 15 -> 127.
    let e = op32k(f, BinOp::Add, exp, 112);
    let e = op32k(f, BinOp::Shl, e, 23);
    let normal = op32(f, BinOp::Or, e, frac);

    // Infinity or NaN; a NaN gets the quiet bit.
    let man_zero = is_zero(f, Ty::I32, man);
    let quiet = c32(f, 0x0040_0000);
    let none = c32(f, 0);
    let q = select(f, Ty::I32, man_zero, none, quiet);
    let special = op32k(f, BinOp::Or, frac, 0x7f80_0000);
    let special = op32(f, BinOp::Or, special, q);

    // Subnormal: man * 2^-24. With its top set bit at position p, the single has
    // exponent p - 24 and the bits below p as its fraction.
    let lz = bit_intrinsic(f, Intrinsic::Ctlz, Ty::I32, man);
    let thirty_one = c32(f, 31);
    let p = op32(f, BinOp::Sub, thirty_one, lz);
    let se = op32k(f, BinOp::Add, p, 103);
    let se = op32k(f, BinOp::Shl, se, 23);
    let twenty_three = c32(f, 23);
    let up = op32(f, BinOp::Sub, twenty_three, p);
    let sf = op32(f, BinOp::Shl, man, up);
    let sf = op32k(f, BinOp::And, sf, 0x007f_ffff);
    let sub = op32(f, BinOp::Or, se, sf);
    let sub = select(f, Ty::I32, man_zero, none, sub);

    let exp_zero = is_zero(f, Ty::I32, exp);
    let max = c32(f, 31);
    let exp_max = cmp(f, CmpOp::Eq, Ty::I32, exp, max);
    let r = select(f, Ty::I32, exp_max, special, normal);
    let r = select(f, Ty::I32, exp_zero, sub, r);
    op32(f, BinOp::Or, r, sign)
}

/// The bits of a single (`I32`) rounded to an IEEE half (`I16`). `mode` is `n`
/// (nearest even), `d` (toward -inf), `u` (toward +inf) or `z` (toward zero).
/// Overflow gives infinity or the largest finite half as the mode dictates, tiny
/// values round into the half subnormals, and a NaN stays NaN (quieted, top payload bits
/// kept).
fn single_to_half(f: &mut FnCtx, x: Val, mode: char) -> Val {
    let neg = op32k(f, BinOp::LShr, x, 31);
    let e = op32k(f, BinOp::LShr, x, 23);
    let e = op32k(f, BinOp::And, e, 0xff);
    let m = op32k(f, BinOp::And, x, 0x007f_ffff);

    // Significand with the hidden bit; a single subnormal behaves as biased exponent 1.
    let e_zero = is_zero(f, Ty::I32, e);
    let hidden = op32k(f, BinOp::Or, m, 0x0080_0000);
    let sig = select(f, Ty::I32, e_zero, m, hidden);
    let one = c32(f, 1);
    let ue = select(f, Ty::I32, e_zero, one, e);

    // Halves are normal from biased single exponent 113 (2^-14) up. Below that each
    // step down drops one more bit; past 26 bits everything is sticky anyway.
    let lowest = c32(f, 113);
    let normal = cmp(f, CmpOp::UGe, Ty::I32, ue, lowest);
    let deficit = op32(f, BinOp::Sub, lowest, ue);
    let zero = c32(f, 0);
    let deficit = select(f, Ty::I32, normal, zero, deficit);
    let shift = op32k(f, BinOp::Add, deficit, 13);
    let cap = c32(f, 26);
    let too_far = cmp(f, CmpOp::UGt, Ty::I32, shift, cap);
    let shift = select(f, Ty::I32, too_far, cap, shift);

    let q = op32(f, BinOp::LShr, sig, shift);
    let unit = op32(f, BinOp::Shl, one, shift);
    let low_mask = op32(f, BinOp::Sub, unit, one);
    let rem = op32(f, BinOp::And, sig, low_mask);
    let half_unit = op32k(f, BinOp::LShr, unit, 1);
    let inexact = cmp(f, CmpOp::Ne, Ty::I32, rem, zero);
    let is_neg = cmp(f, CmpOp::Ne, Ty::I32, neg, zero);
    let inc = match mode {
        'z' => konst(f, Ty::I8, 0),
        'u' => {
            let pos = flag_not(f, is_neg);
            flag_and(f, inexact, pos)
        }
        'd' => flag_and(f, inexact, is_neg),
        _ => {
            let above = cmp(f, CmpOp::UGt, Ty::I32, rem, half_unit);
            let tie = cmp(f, CmpOp::Eq, Ty::I32, rem, half_unit);
            let odd = op32k(f, BinOp::And, q, 1);
            let odd = resize(f, odd, Ty::I32, Ty::I8, false);
            let tie_up = flag_and(f, tie, odd);
            flag_or(f, above, tie_up)
        }
    };
    let inc = resize(f, inc, Ty::I8, Ty::I32, false);
    let q = op32(f, BinOp::Add, q, inc);

    // (exponent - 1) << 10 plus a significand that still holds the hidden bit, so a
    // rounding carry moves into the exponent and the largest subnormal rounds up to
    // the smallest normal.
    let base = op32(f, BinOp::Sub, ue, lowest);
    let base = select(f, Ty::I32, normal, base, zero);
    let base = op32k(f, BinOp::Shl, base, 10);
    let mag = op32(f, BinOp::Add, base, q);

    let inf = c32(f, 0x7c00);
    let overflow = cmp(f, CmpOp::UGe, Ty::I32, mag, inf);
    let max_finite = c32(f, 0x7bff);
    let saturated = match mode {
        'n' => inf,
        'z' => max_finite,
        // Toward +inf: positive overflow is infinite, negative stops at -max.
        'u' => select(f, Ty::I32, is_neg, max_finite, inf),
        _ => select(f, Ty::I32, is_neg, inf, max_finite),
    };
    let mag = select(f, Ty::I32, overflow, saturated, mag);

    let all_ones = c32(f, 0xff);
    let e_max = cmp(f, CmpOp::Eq, Ty::I32, e, all_ones);
    let m_zero = is_zero(f, Ty::I32, m);
    let payload = op32k(f, BinOp::LShr, m, 13);
    let nan = op32k(f, BinOp::Or, payload, 0x7e00);
    let special = select(f, Ty::I32, m_zero, inf, nan);
    let mag = select(f, Ty::I32, e_max, special, mag);

    let s = op32k(f, BinOp::Shl, neg, 15);
    let r = op32(f, BinOp::Or, mag, s);
    resize(f, r, Ty::I32, Ty::I16, false)
}

/// Carry-less product of two bytes reduced modulo x^8 + x^4 + x^3 + x + 1.
fn gf_mul(f: &mut FnCtx, a: Val, b: Val) -> Val {
    let mut acc = konst(f, Ty::I8, 0);
    let mut a = a;
    for bit in 0..8 {
        let at = konst(f, Ty::I8, bit);
        let take = bin(f, BinOp::LShr, Ty::I8, b, at);
        let one = konst(f, Ty::I8, 1);
        let take = bin(f, BinOp::And, Ty::I8, take, one);
        let zero = konst(f, Ty::I8, 0);
        let all = bin(f, BinOp::Sub, Ty::I8, zero, take);
        let term = bin(f, BinOp::And, Ty::I8, a, all);
        acc = bin(f, BinOp::Xor, Ty::I8, acc, term);
        if bit < 7 {
            a = xtime(f, a);
        }
    }
    acc
}

/// One byte through an 8x8 bit matrix: result bit i is the parity of
/// `rows[7 - i] & x`, then XOR the constant `c`.
fn gf_affine(f: &mut FnCtx, rows: &[Val; 8], x: Val, c: u64) -> Val {
    let mut r = konst(f, Ty::I8, c);
    for i in 0..8 {
        let t = bin(f, BinOp::And, Ty::I8, rows[7 - i], x);
        let n = bit_intrinsic(f, Intrinsic::Popcount, Ty::I8, t);
        let one = konst(f, Ty::I8, 1);
        let parity = bin(f, BinOp::And, Ty::I8, n, one);
        let at = konst(f, Ty::I8, i as u64);
        let bit = bin(f, BinOp::Shl, Ty::I8, parity, at);
        r = bin(f, BinOp::Xor, Ty::I8, r, bit);
    }
    r
}

impl Compiler {
    /// Store `bytes` of `tmp` (zero above them) to `dst`. A register destination is
    /// written in full: cleared above the result for VEX/EVEX encodings, kept for
    /// legacy SSE encodings, whatever the block's default.
    #[allow(clippy::too_many_arguments)]
    fn ext_store(
        &mut self,
        f: &mut FnCtx,
        cx: &AsmCtx,
        dst: VOpd,
        tmp: Val,
        bytes: u64,
        vex: bool,
        span: Span,
    ) -> Result<()> {
        match dst {
            VOpd::Reg(reg) => {
                if !vex && bytes < 64 {
                    let keep = lane_addr(f, tmp, bytes);
                    let old = lane_addr(f, reg, bytes);
                    f.b.copy(keep, old, 64 - bytes);
                }
                self.vec_store(f, cx, dst, tmp, 64, span)
            }
            _ => self.vec_store(f, cx, dst, tmp, bytes, span),
        }
    }

    /// `vcvtph2ps dst, src` and `vcvtps2ph dst, src, imm8`. `width` is the size of
    /// the single-precision vector; the half vector is half of it.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn simd_half(
        &mut self,
        f: &mut FnCtx,
        cx: &AsmCtx,
        inst: &AsmInst,
        to_half: bool,
        ops: &[VOpd],
        width: u64,
        wm: Option<WriteMask>,
        tmp: Val,
    ) -> Result<()> {
        let span = inst.span;
        let name = inst.mnemonic.name.as_str();
        let lanes = width / 4;
        let dst = ops[0];
        if !to_half {
            if ops.len() != 2 {
                return err(span, format!("`{name}` takes dst, src"));
            }
            let (s, _) = self.simd_args(f, ops, 1, false, span)?;
            for i in 0..lanes {
                let h = load_lane(f, s[0], i, Ty::I16);
                let r = half_to_single(f, h);
                store_lane(f, tmp, i, Ty::I32, r);
            }
            // F16C only exists VEX/EVEX encoded.
            return self.ext_store(f, cx, dst, tmp, width, true, span);
        }
        if ops.len() != 3 {
            return err(span, format!("`{name}` takes dst, src, imm8"));
        }
        let (s, k) = self.simd_args(f, ops, 1, true, span)?;
        // imm8[2] defers to MXCSR.RC, which is always round-to-nearest here.
        let mode = if k & 4 != 0 {
            'n'
        } else {
            ['n', 'd', 'u', 'z'][(k & 3) as usize]
        };
        for i in 0..lanes {
            let x = load_lane(f, s[0], i, Ty::I32);
            let h = single_to_half(f, x, mode);
            store_lane(f, tmp, i, Ty::I16, h);
        }
        // The EVEX form masks half-sized elements of a narrower destination, so the
        // generic merge (which works on the full vector width) is done here instead.
        if let Some(wm) = wm {
            match dst {
                VOpd::Mem(addr) => {
                    let p = ptr_of(f, addr);
                    for i in 0..lanes {
                        let on = mask_bit(f, wm.bits, i);
                        let v = load_lane(f, tmp, i, Ty::I16);
                        when(f, on, |f| store_lane(f, p, i, Ty::I16, v));
                    }
                    return Ok(());
                }
                VOpd::Reg(reg) => {
                    for i in 0..lanes {
                        let on = mask_bit(f, wm.bits, i);
                        let new = load_lane(f, tmp, i, Ty::I16);
                        let old = if wm.zeroing {
                            konst(f, Ty::I16, 0)
                        } else {
                            load_lane(f, reg, i, Ty::I16)
                        };
                        let v = select(f, Ty::I16, on, new, old);
                        store_lane(f, tmp, i, Ty::I16, v);
                    }
                }
                _ => {}
            }
        }
        self.ext_store(f, cx, dst, tmp, lanes * 2, true, span)
    }

    /// The SHA extensions. All are legacy SSE encoded and 128 bits wide.
    pub(super) fn simd_sha(
        &mut self,
        f: &mut FnCtx,
        cx: &AsmCtx,
        inst: &AsmInst,
        sha: ShaOp,
        ops: &[VOpd],
        tmp: Val,
    ) -> Result<()> {
        let span = inst.span;
        let name = inst.mnemonic.name.as_str();
        if inst.evex.mask.is_some() {
            return err(span, format!("`{name}` cannot be masked"));
        }
        if sha == ShaOp::Sha256Rnds2 && !matches!(ops.len(), 3 | 4) {
            return err(
                span,
                format!("`{name}` takes dst, src, wk (the implicit xmm0 as an operand)"),
            );
        }
        let (s, k) = match sha {
            ShaOp::Sha1Rnds4 => self.simd_args(f, ops, 2, true, span)?,
            ShaOp::Sha256Rnds2 => self.simd_args(f, ops, 3, false, span)?,
            _ => self.simd_args(f, ops, 2, false, span)?,
        };
        // Dwords from the top: x[0] is bits 127:96, x[3] bits 31:0.
        let top_down = |f: &mut FnCtx, p: Val| -> [Val; 4] {
            [
                load_lane(f, p, 3, Ty::I32),
                load_lane(f, p, 2, Ty::I32),
                load_lane(f, p, 1, Ty::I32),
                load_lane(f, p, 0, Ty::I32),
            ]
        };
        let x = top_down(f, s[0]);
        let y = top_down(f, s[1]);
        let out: [Val; 4] = match sha {
            ShaOp::Sha1Rnds4 => {
                let (konst_k, fun) = [
                    (0x5a82_7999u64, 0),
                    (0x6ed9_eba1, 1),
                    (0x8f1b_bcdc, 2),
                    (0xca62_c1d6, 1),
                ][(k & 3) as usize];
                let [mut a, mut b, mut c, mut d] = x;
                let mut e = None;
                for &w in &y {
                    let mix = match fun {
                        0 => choose(f, b, c, d),
                        1 => xor_all(f, &[b, c, d]),
                        _ => majority(f, b, c, d),
                    };
                    let a5 = rotl(f, a, 5);
                    let kk = c32(f, konst_k);
                    // The first word already has E folded in (see sha1nexte).
                    let mut t = add_all(f, &[a5, mix, w, kk]);
                    if let Some(e) = e {
                        t = op32(f, BinOp::Add, t, e);
                    }
                    e = Some(d);
                    d = c;
                    c = rotl(f, b, 30);
                    b = a;
                    a = t;
                }
                [a, b, c, d]
            }
            ShaOp::Sha1Nexte => {
                let e = rotl(f, x[0], 30);
                let top = op32(f, BinOp::Add, y[0], e);
                [top, y[1], y[2], y[3]]
            }
            ShaOp::Sha1Msg1 => [
                op32(f, BinOp::Xor, x[2], x[0]),
                op32(f, BinOp::Xor, x[3], x[1]),
                op32(f, BinOp::Xor, y[0], x[2]),
                op32(f, BinOp::Xor, y[1], x[3]),
            ],
            ShaOp::Sha1Msg2 => {
                let mut w = [Val(0); 4];
                for i in 0..4 {
                    // The fourth word depends on the first result.
                    let other = if i < 3 {
                        y[i + 1]
                    } else {
                        w[0]
                    };
                    let t = op32(f, BinOp::Xor, x[i], other);
                    w[i] = rotl(f, t, 1);
                }
                w
            }
            ShaOp::Sha256Rnds2 => {
                let wk = [
                    load_lane(f, s[2], 0, Ty::I32),
                    load_lane(f, s[2], 1, Ty::I32),
                ];
                let (mut a, mut b, mut c, mut d) = (y[0], y[1], x[0], x[1]);
                let (mut e, mut ff, mut g, mut h) = (y[2], y[3], x[2], x[3]);
                for w in wk {
                    let ch = choose(f, e, ff, g);
                    let s1 = sigma(f, e, 6, 11, 25, false);
                    let t1 = add_all(f, &[ch, s1, w, h]);
                    let mj = majority(f, a, b, c);
                    let s0 = sigma(f, a, 2, 13, 22, false);
                    let new_a = add_all(f, &[t1, mj, s0]);
                    let new_e = op32(f, BinOp::Add, t1, d);
                    (h, g, ff, e) = (g, ff, e, new_e);
                    (d, c, b, a) = (c, b, a, new_a);
                }
                [a, b, e, ff]
            }
            ShaOp::Sha256Msg1 => {
                // Each dword gains sigma0 of the dword above it; the top one of src1
                // takes the lowest of src2.
                let next = [y[3], x[0], x[1], x[2]];
                let mut out = [Val(0); 4];
                for i in 0..4 {
                    let s0 = sigma(f, next[i], 7, 18, 3, true);
                    out[i] = op32(f, BinOp::Add, x[i], s0);
                }
                out
            }
            ShaOp::Sha256Msg2 => {
                let s = sigma(f, y[1], 17, 19, 10, true);
                let w16 = op32(f, BinOp::Add, x[3], s);
                let s = sigma(f, y[0], 17, 19, 10, true);
                let w17 = op32(f, BinOp::Add, x[2], s);
                let s = sigma(f, w16, 17, 19, 10, true);
                let w18 = op32(f, BinOp::Add, x[1], s);
                let s = sigma(f, w17, 17, 19, 10, true);
                let w19 = op32(f, BinOp::Add, x[0], s);
                [w19, w18, w17, w16]
            }
        };
        for (i, v) in out.into_iter().enumerate() {
            store_lane(f, tmp, 3 - i as u64, Ty::I32, v);
        }
        self.ext_store(f, cx, ops[0], tmp, 16, false, span)
    }

    /// `gf2p8mulb`, `gf2p8affineqb` and `gf2p8affineinvqb`, legacy (two sources with
    /// the destination first) or `v`-prefixed.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn simd_gf(
        &mut self,
        f: &mut FnCtx,
        cx: &AsmCtx,
        inst: &AsmInst,
        gf: GfOp,
        ops: &[VOpd],
        width: u64,
        tmp: Val,
    ) -> Result<()> {
        let span = inst.span;
        let vex = inst.mnemonic.name.as_str().starts_with('v');
        match gf {
            GfOp::Mul => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                for i in 0..width {
                    let a = load_lane(f, s[0], i, Ty::I8);
                    let b = load_lane(f, s[1], i, Ty::I8);
                    let r = gf_mul(f, a, b);
                    store_lane(f, tmp, i, Ty::I8, r);
                }
            }
            GfOp::Affine(invert) => {
                let (s, k) = self.simd_args(f, ops, 2, true, span)?;
                let inverses = if invert {
                    let tables = self.aes_table_ptr(f);
                    Some(f.b.ptr_offset(tables, 512))
                } else {
                    None
                };
                for q in 0..width / 8 {
                    let mut rows = [Val(0); 8];
                    for (j, row) in rows.iter_mut().enumerate() {
                        *row = load_lane(f, s[1], 8 * q + j as u64, Ty::I8);
                    }
                    for j in 0..8 {
                        let mut x = load_lane(f, s[0], 8 * q + j, Ty::I8);
                        if let Some(table) = inverses {
                            let at = resize(f, x, Ty::I8, Ty::I64, false);
                            x = load_dyn(f, table, at, Ty::I8);
                        }
                        let r = gf_affine(f, &rows, x, k);
                        store_lane(f, tmp, 8 * q + j, Ty::I8, r);
                    }
                }
            }
        }
        self.ext_store(f, cx, ops[0], tmp, width, vex, span)
    }
}
