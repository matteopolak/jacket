//! Software arithmetic for the C `long double` formats wider than `f64`: the x87 80-bit
//! extended format (x86-64 System V, MinGW) and IEEE binary128 (Linux AArch64, wasm32).
//!
//! The interpreter evaluates `Long_Double` (module `Jaic_Extensions`) operations with these
//! on every host, so compile-time code and `jaic run` get the target's full precision, and
//! sema uses them to encode constants. Values are 16 little-endian bytes, the target's memory
//! image; for x87 the last six bytes are padding and written as zero.
//!
//! Rounding is to nearest, ties to even, everywhere. NaN payloads are not kept: a NaN result
//! is the format's default quiet NaN.
use std::cmp::Ordering;

/// A `long double` format wider than `f64`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum WideFloat {
    /// x87 extended precision: 64-bit significand with an explicit integer bit, 15-bit
    /// exponent; stored in 16 bytes (10 used).
    X87,
    /// IEEE 754 binary128: 113-bit significand (112 stored), 15-bit exponent.
    Binary128,
}

/// Operations of the `Wide` IR intrinsic that take two operands.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Arith {
    Add,
    Sub,
    Mul,
    Div,
}

pub type Bytes = [u8; 16];

/// Precision and exponent range of a binary format. `bits` counts the significand including
/// the leading bit; `emin`/`emax` bound the exponent of that leading bit for normal numbers.
#[derive(Clone, Copy)]
struct Prec {
    bits: u32,
    emin: i32,
    emax: i32,
}

const P32: Prec = Prec {
    bits: 24,
    emin: -126,
    emax: 127,
};
const P64: Prec = Prec {
    bits: 53,
    emin: -1022,
    emax: 1023,
};
const P80: Prec = Prec {
    bits: 64,
    emin: -16382,
    emax: 16383,
};
const P128: Prec = Prec {
    bits: 113,
    emin: -16382,
    emax: 16383,
};

impl Prec {
    /// Exponent of the least significant bit of the smallest subnormal.
    fn min_lsb(self) -> i32 {
        self.emin - (self.bits as i32 - 1)
    }
}

/// A decoded value. `Fin(sign, m, e)` is `±m * 2^e` with `m != 0`.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Num {
    Nan,
    Inf(bool),
    Zero(bool),
    Fin(bool, u128, i32),
}

fn bit_len(m: u128) -> i32 {
    128 - m.leading_zeros() as i32
}

/// Round `±m * 2^e` (plus a nonzero amount below bit 0 of `m` when `sticky`) to `p`.
fn round(sign: bool, m: u128, e: i32, sticky: bool, p: Prec) -> Num {
    if m == 0 {
        return Num::Zero(sign);
    }
    let mut drop = bit_len(m) - p.bits as i32;
    if e + drop < p.min_lsb() {
        drop = p.min_lsb() - e;
    }
    if drop <= 0 {
        debug_assert!(!sticky, "inexact value with no bits to round away");
        return Num::Fin(sign, m << (-drop) as u32, e + drop);
    }
    let drop = drop as u32;
    let kept = if drop >= 128 {
        0
    } else {
        m >> drop
    };
    let half = drop <= 128 && (m >> (drop - 1)) & 1 == 1;
    let below = if drop > 128 {
        m
    } else {
        m & ((1u128 << (drop - 1)) - 1)
    };
    let mut kept = kept;
    let mut e = e + drop as i32;
    if half && (below != 0 || sticky || kept & 1 == 1) {
        kept += 1;
        if kept == 1u128 << p.bits {
            kept >>= 1;
            e += 1;
        }
    }
    if kept == 0 {
        return Num::Zero(sign);
    }
    if e + bit_len(kept) - 1 > p.emax {
        return Num::Inf(sign);
    }
    Num::Fin(sign, kept, e)
}

/// Biased exponent field and significand (aligned for the format's encoding) of a value
/// already rounded to `p`.
fn fields(m: u128, e: i32, p: Prec) -> (u32, u128) {
    let lead = e + bit_len(m) - 1;
    if lead < p.emin {
        // Subnormal: the significand counts units of the smallest subnormal.
        (0, m << (e - p.min_lsb()) as u32)
    } else {
        let bias = p.emax;
        (
            (lead + bias) as u32,
            m << (p.bits as i32 - bit_len(m)) as u32,
        )
    }
}

/// Encode into an IEEE interchange format with a hidden leading bit (f32, f64, binary128),
/// `exp_bits` wide.
fn encode_ieee(n: Num, p: Prec, exp_bits: u32) -> u128 {
    let frac_bits = p.bits - 1;
    let sign_at = frac_bits + exp_bits;
    let max_exp = (1u128 << exp_bits) - 1;
    match n {
        Num::Nan => (max_exp << frac_bits) | (1u128 << (frac_bits - 1)),
        Num::Inf(s) => ((s as u128) << sign_at) | (max_exp << frac_bits),
        Num::Zero(s) => (s as u128) << sign_at,
        Num::Fin(s, m, e) => {
            let (exp, sig) = fields(m, e, p);
            let frac = sig & ((1u128 << frac_bits) - 1);
            ((s as u128) << sign_at) | ((exp as u128) << frac_bits) | frac
        }
    }
}

fn decode_ieee(bits: u128, p: Prec, exp_bits: u32) -> Num {
    let frac_bits = p.bits - 1;
    let sign = (bits >> (frac_bits + exp_bits)) & 1 == 1;
    let exp = ((bits >> frac_bits) & ((1u128 << exp_bits) - 1)) as i32;
    let frac = bits & ((1u128 << frac_bits) - 1);
    let max_exp = (1i32 << exp_bits) - 1;
    if exp == max_exp {
        return if frac == 0 {
            Num::Inf(sign)
        } else {
            Num::Nan
        };
    }
    if exp == 0 {
        return if frac == 0 {
            Num::Zero(sign)
        } else {
            Num::Fin(sign, frac, p.min_lsb())
        };
    }
    Num::Fin(
        sign,
        frac | (1u128 << frac_bits),
        exp - p.emax - frac_bits as i32,
    )
}

fn encode_x87(n: Num) -> u128 {
    let (sign, exp, sig): (bool, u128, u128) = match n {
        Num::Nan => (false, 0x7fff, 0xc000_0000_0000_0000),
        Num::Inf(s) => (s, 0x7fff, 0x8000_0000_0000_0000),
        Num::Zero(s) => (s, 0, 0),
        Num::Fin(s, m, e) => {
            let (exp, sig) = fields(m, e, P80);
            (s, exp as u128, sig)
        }
    };
    ((sign as u128) << 79) | (exp << 64) | sig
}

fn decode_x87(bits: u128) -> Num {
    let sign = (bits >> 79) & 1 == 1;
    let exp = ((bits >> 64) & 0x7fff) as i32;
    let sig = bits & u64::MAX as u128;
    let integer_bit = sig >> 63 == 1;
    if exp == 0x7fff {
        return if sig == 1u128 << 63 {
            Num::Inf(sign)
        } else {
            Num::Nan
        };
    }
    if sig == 0 {
        return Num::Zero(sign);
    }
    if exp == 0 {
        // Denormals (and pseudo-denormals, whose integer bit is set) share the minimum exponent.
        return Num::Fin(sign, sig, P80.min_lsb());
    }
    if !integer_bit {
        // Unnormals are invalid operands on every x87 since the 387.
        return Num::Nan;
    }
    Num::Fin(sign, sig, exp - P80.emax - 63)
}

fn prec(fmt: WideFloat) -> Prec {
    match fmt {
        WideFloat::X87 => P80,
        WideFloat::Binary128 => P128,
    }
}

fn decode(fmt: WideFloat, b: &Bytes) -> Num {
    let bits = u128::from_le_bytes(*b);
    match fmt {
        WideFloat::X87 => decode_x87(bits & ((1u128 << 80) - 1)),
        WideFloat::Binary128 => decode_ieee(bits, P128, 15),
    }
}

/// Encode a value already rounded to `fmt`'s precision.
fn encode(fmt: WideFloat, n: Num) -> Bytes {
    match fmt {
        WideFloat::X87 => encode_x87(n),
        WideFloat::Binary128 => encode_ieee(n, P128, 15),
    }
    .to_le_bytes()
}

fn from_num(fmt: WideFloat, n: Num) -> Bytes {
    let n = match n {
        Num::Fin(s, m, e) => round(s, m, e, false, prec(fmt)),
        other => other,
    };
    encode(fmt, n)
}

pub fn from_f64(fmt: WideFloat, x: f64) -> Bytes {
    from_num(fmt, decode_ieee(x.to_bits() as u128, P64, 11))
}

pub fn from_f32(fmt: WideFloat, x: f32) -> Bytes {
    from_num(fmt, decode_ieee(x.to_bits() as u128, P32, 8))
}

pub fn from_i64(fmt: WideFloat, v: i64) -> Bytes {
    if v == 0 {
        return encode(fmt, Num::Zero(false));
    }
    from_num(fmt, Num::Fin(v < 0, v.unsigned_abs() as u128, 0))
}

pub fn from_u64(fmt: WideFloat, v: u64) -> Bytes {
    if v == 0 {
        return encode(fmt, Num::Zero(false));
    }
    from_num(fmt, Num::Fin(false, v as u128, 0))
}

/// Round to a narrower format `p`, encoded `exp_bits` wide.
fn narrow(fmt: WideFloat, b: &Bytes, p: Prec, exp_bits: u32) -> u128 {
    let n = match decode(fmt, b) {
        Num::Fin(s, m, e) => round(s, m, e, false, p),
        other => other,
    };
    encode_ieee(n, p, exp_bits)
}

pub fn to_f64(fmt: WideFloat, b: &Bytes) -> f64 {
    f64::from_bits(narrow(fmt, b, P64, 11) as u64)
}

pub fn to_f32(fmt: WideFloat, b: &Bytes) -> f32 {
    f32::from_bits(narrow(fmt, b, P32, 8) as u32)
}

/// The integer part (truncated toward zero) as an `i128`, saturating far outside the 64-bit
/// range; NaN is 0.
fn truncate(fmt: WideFloat, b: &Bytes) -> i128 {
    match decode(fmt, b) {
        Num::Nan | Num::Zero(_) => 0,
        Num::Inf(s) => {
            if s {
                i128::MIN
            } else {
                i128::MAX
            }
        }
        Num::Fin(s, m, e) => {
            let magnitude = if e >= 0 {
                if bit_len(m) + e > 100 {
                    i128::MAX
                } else {
                    (m << e as u32) as i128
                }
            } else if -e >= 128 {
                0
            } else {
                (m >> (-e) as u32) as i128
            };
            if s {
                -magnitude
            } else {
                magnitude
            }
        }
    }
}

/// Conversion to `s64`. Out of range (undefined in C and LLVM) this does what the target's
/// hardware does: x87 `fistp` gives the "integer indefinite" `i64::MIN` (NaN too), binary128
/// conversions saturate.
pub fn to_i64(fmt: WideFloat, b: &Bytes) -> i64 {
    let v = truncate(fmt, b);
    let in_range = v >= i64::MIN as i128 && v <= i64::MAX as i128;
    match fmt {
        WideFloat::X87 if !in_range || decode(fmt, b) == Num::Nan => i64::MIN,
        _ => v.clamp(i64::MIN as i128, i64::MAX as i128) as i64,
    }
}

/// Conversion to `u64`; out-of-range values saturate.
pub fn to_u64(fmt: WideFloat, b: &Bytes) -> u64 {
    truncate(fmt, b).clamp(0, u64::MAX as i128) as u64
}

pub fn neg(fmt: WideFloat, b: &Bytes) -> Bytes {
    let mut out = *b;
    match fmt {
        WideFloat::X87 => out[9] ^= 0x80,
        WideFloat::Binary128 => out[15] ^= 0x80,
    }
    out
}

/// `None` when either operand is NaN (unordered).
pub fn compare(fmt: WideFloat, a: &Bytes, b: &Bytes) -> Option<Ordering> {
    let key = |n: Num| -> Option<(i8, i32, u128)> {
        // (sign class, exponent of the leading bit, significand aligned to bit 127)
        Some(match n {
            Num::Nan => return None,
            Num::Zero(_) => (0, 0, 0),
            Num::Inf(s) => (
                if s {
                    -2
                } else {
                    2
                },
                0,
                0,
            ),
            Num::Fin(s, m, e) => (
                if s {
                    -1
                } else {
                    1
                },
                e + bit_len(m) - 1,
                m << m.leading_zeros(),
            ),
        })
    };
    let (x, y) = (key(decode(fmt, a))?, key(decode(fmt, b))?);
    Some(match x.0.cmp(&y.0) {
        Ordering::Equal if x.0 == 1 => (x.1, x.2).cmp(&(y.1, y.2)),
        Ordering::Equal if x.0 == -1 => (y.1, y.2).cmp(&(x.1, x.2)),
        other => other,
    })
}

pub fn arith(fmt: WideFloat, op: Arith, a: &Bytes, b: &Bytes) -> Bytes {
    let p = prec(fmt);
    let (x, y) = (decode(fmt, a), decode(fmt, b));
    let n = match op {
        Arith::Add => add(x, y, p),
        Arith::Sub => add(x, negate(y), p),
        Arith::Mul => mul(x, y, p),
        Arith::Div => div(x, y, p),
    };
    encode(fmt, n)
}

fn negate(n: Num) -> Num {
    match n {
        Num::Nan => Num::Nan,
        Num::Inf(s) => Num::Inf(!s),
        Num::Zero(s) => Num::Zero(!s),
        Num::Fin(s, m, e) => Num::Fin(!s, m, e),
    }
}

/// Shift a nonzero significand so its leading bit is bit `top`, adjusting the exponent.
fn align(m: u128, e: i32, top: i32) -> (u128, i32) {
    let k = top - (bit_len(m) - 1);
    if k >= 0 {
        (m << k as u32, e - k)
    } else {
        (m >> (-k) as u32, e - k)
    }
}

fn add(x: Num, y: Num, p: Prec) -> Num {
    match (x, y) {
        (Num::Nan, _) | (_, Num::Nan) => Num::Nan,
        (Num::Inf(a), Num::Inf(b)) if a != b => Num::Nan,
        (Num::Inf(s), _) | (_, Num::Inf(s)) => Num::Inf(s),
        (Num::Zero(a), Num::Zero(b)) => Num::Zero(a && b),
        (Num::Zero(_), v) | (v, Num::Zero(_)) => v,
        (Num::Fin(sa, ma, ea), Num::Fin(sb, mb, eb)) => {
            // Both leading bits at 125: room for a carry, and at least 12 bits below any
            // rounding position (precision is at most 113 bits).
            let (ma, ea) = align(ma, ea, 125);
            let (mb, eb) = align(mb, eb, 125);
            let ((sa, ma, ea), (sb, mb, eb)) = if ea >= eb {
                ((sa, ma, ea), (sb, mb, eb))
            } else {
                ((sb, mb, eb), (sa, ma, ea))
            };
            let shift = (ea - eb) as u32;
            // Bits shifted out are folded into bit 0 ("jamming"); it lies below the rounding
            // position, so the rounding decision is unchanged.
            let mb = if shift >= 128 {
                (mb != 0) as u128
            } else {
                let lost = mb & ((1u128 << shift) - 1) != 0;
                (mb >> shift) | lost as u128
            };
            if sa == sb {
                round(sa, ma + mb, ea, false, p)
            } else if ma == mb {
                Num::Zero(false)
            } else if ma > mb {
                round(sa, ma - mb, ea, false, p)
            } else {
                round(sb, mb - ma, ea, false, p)
            }
        }
    }
}

/// Full 256-bit product of two 128-bit values as (high, low).
fn mul_wide(a: u128, b: u128) -> (u128, u128) {
    let mask = u64::MAX as u128;
    let (a1, a0) = (a >> 64, a & mask);
    let (b1, b0) = (b >> 64, b & mask);
    let p00 = a0 * b0;
    let p01 = a0 * b1;
    let p10 = a1 * b0;
    let p11 = a1 * b1;
    let mid = (p00 >> 64) + (p01 & mask) + (p10 & mask);
    let low = (p00 & mask) | (mid << 64);
    let high = p11 + (p01 >> 64) + (p10 >> 64) + (mid >> 64);
    (high, low)
}

fn mul(x: Num, y: Num, p: Prec) -> Num {
    match (x, y) {
        (Num::Nan, _) | (_, Num::Nan) => Num::Nan,
        (Num::Inf(_), Num::Zero(_)) | (Num::Zero(_), Num::Inf(_)) => Num::Nan,
        (Num::Inf(a), Num::Inf(b) | Num::Fin(b, ..)) | (Num::Fin(a, ..), Num::Inf(b)) => {
            Num::Inf(a != b)
        }
        (Num::Zero(a), Num::Zero(b) | Num::Fin(b, ..)) | (Num::Fin(a, ..), Num::Zero(b)) => {
            Num::Zero(a != b)
        }
        (Num::Fin(sa, ma, ea), Num::Fin(sb, mb, eb)) => {
            let (high, low) = mul_wide(ma, mb);
            let e = ea + eb;
            if high == 0 {
                return round(sa != sb, low, e, false, p);
            }
            // Keep the top 126 bits; everything below becomes the sticky flag.
            let len = 128 + bit_len(high);
            let shift = (len - 126) as u32;
            let kept = (high << (128 - shift)) | (low >> shift);
            let sticky = low & ((1u128 << shift) - 1) != 0;
            round(sa != sb, kept, e + shift as i32, sticky, p)
        }
    }
}

fn div(x: Num, y: Num, p: Prec) -> Num {
    match (x, y) {
        (Num::Nan, _) | (_, Num::Nan) => Num::Nan,
        (Num::Inf(_), Num::Inf(_)) | (Num::Zero(_), Num::Zero(_)) => Num::Nan,
        (Num::Inf(a), Num::Zero(b) | Num::Fin(b, ..)) => Num::Inf(a != b),
        (Num::Zero(a) | Num::Fin(a, ..), Num::Inf(b)) => Num::Zero(a != b),
        (Num::Fin(a, ..), Num::Zero(b)) => Num::Inf(a != b),
        (Num::Zero(a), Num::Fin(b, ..)) => Num::Zero(a != b),
        (Num::Fin(sa, ma, ea), Num::Fin(sb, mb, eb)) => {
            let (mut r, ea) = align(ma, ea, 114);
            let (mb, eb) = align(mb, eb, 114);
            let mut e = ea - eb;
            if r < mb {
                r <<= 1;
                e -= 1;
            }
            // Restoring division: `p + 2` quotient bits, then the remainder is the sticky bit.
            let steps = p.bits + 2;
            let mut q = 0u128;
            for _ in 0..steps {
                q <<= 1;
                if r >= mb {
                    r -= mb;
                    q |= 1;
                }
                r <<= 1;
            }
            round(sa != sb, q, e - (steps as i32 - 1), r != 0, p)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [WideFloat; 2] = [WideFloat::X87, WideFloat::Binary128];

    fn bits(b: Bytes) -> u128 {
        u128::from_le_bytes(b)
    }

    #[test]
    fn known_encodings() {
        // 1.0 and 1/3 as x87 and binary128 bit patterns.
        let one = from_f64(WideFloat::X87, 1.0);
        assert_eq!(bits(one), 0x3fff_8000_0000_0000_0000);
        let third = arith(
            WideFloat::X87,
            Arith::Div,
            &one,
            &from_f64(WideFloat::X87, 3.0),
        );
        assert_eq!(bits(third), 0x3ffd_aaaa_aaaa_aaaa_aaab);
        let q1 = from_f64(WideFloat::Binary128, 1.0);
        assert_eq!(bits(q1), 0x3fff_0000_0000_0000_0000_0000_0000_0000);
        let q3 = arith(
            WideFloat::Binary128,
            Arith::Div,
            &q1,
            &from_f64(WideFloat::Binary128, 3.0),
        );
        assert_eq!(bits(q3), 0x3ffd_5555_5555_5555_5555_5555_5555_5555);
        assert_eq!(
            bits(from_f64(WideFloat::Binary128, -2.5)),
            0xc000_4000_0000_0000_0000_0000_0000_0000
        );
    }

    #[test]
    fn f64_round_trips_exactly() {
        let samples = [
            0.0,
            -0.0,
            1.5,
            -3.25e-300,
            f64::MAX,
            f64::MIN_POSITIVE,
            5e-324,
            f64::INFINITY,
            123456.789,
        ];
        for fmt in ALL {
            for x in samples {
                let back = to_f64(fmt, &from_f64(fmt, x));
                assert_eq!(back.to_bits(), x.to_bits(), "{fmt:?} {x}");
            }
            assert!(to_f64(fmt, &from_f64(fmt, f64::NAN)).is_nan());
        }
    }

    /// A small deterministic generator (xorshift) of interesting doubles.
    fn doubles(n: usize) -> Vec<f64> {
        let mut s = 0x9e37_79b9_7f4a_7c15u64;
        let mut out = Vec::new();
        for _ in 0..n {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            let exp = (s >> 52) % 200;
            let x = f64::from_bits((s & ((1 << 52) - 1)) | ((1023 + exp - 100) << 52));
            out.push(if s & 1 == 1 {
                -x
            } else {
                x
            });
        }
        out
    }

    /// binary128 has more than 2 * 53 + 2 bits, so rounding its correctly rounded result to f64
    /// gives the correctly rounded f64 result: every operation must agree with the host's.
    #[test]
    fn binary128_agrees_with_f64_after_rounding() {
        let q = WideFloat::Binary128;
        let xs = doubles(400);
        for w in xs.windows(2) {
            let (a, b) = (w[0], w[1]);
            let (qa, qb) = (from_f64(q, a), from_f64(q, b));
            for (op, want) in [
                (Arith::Add, a + b),
                (Arith::Sub, a - b),
                (Arith::Mul, a * b),
                (Arith::Div, a / b),
            ] {
                let got = to_f64(q, &arith(q, op, &qa, &qb));
                assert_eq!(got.to_bits(), want.to_bits(), "{a} {op:?} {b}");
            }
            assert_eq!(compare(q, &qa, &qb), a.partial_cmp(&b));
        }
    }

    /// A product of two doubles is exact in binary128, so rounding it to x87 once is the
    /// correctly rounded x87 product; sums of nearby doubles are exact too.
    #[test]
    fn x87_matches_binary128_where_that_is_exact() {
        let (x, q) = (WideFloat::X87, WideFloat::Binary128);
        let reround = |b: Bytes| match decode(q, &b) {
            Num::Fin(s, m, e) => encode(x, round(s, m, e, false, P80)),
            other => encode(x, other),
        };
        let xs = doubles(300);
        for w in xs.windows(2) {
            let (a, b) = (w[0], w[1]);
            let exact = arith(q, Arith::Mul, &from_f64(q, a), &from_f64(q, b));
            let got = arith(x, Arith::Mul, &from_f64(x, a), &from_f64(x, b));
            assert_eq!(bits(got), bits(reround(exact)), "{a} * {b}");
            let b2 = a * 1.75;
            let exact = arith(q, Arith::Add, &from_f64(q, a), &from_f64(q, b2));
            let got = arith(x, Arith::Add, &from_f64(x, a), &from_f64(x, b2));
            assert_eq!(bits(got), bits(reround(exact)), "{a} + {b2}");
        }
    }

    #[test]
    fn extra_precision_is_kept() {
        for fmt in ALL {
            // 1 + 2^-60 is exact in both formats, not in f64.
            let one = from_f64(fmt, 1.0);
            let tiny = from_f64(fmt, 2f64.powi(-60));
            let sum = arith(fmt, Arith::Add, &one, &tiny);
            assert_eq!(compare(fmt, &sum, &one), Some(Ordering::Greater));
            let back = arith(fmt, Arith::Sub, &sum, &one);
            assert_eq!(to_f64(fmt, &back), 2f64.powi(-60));
            assert_eq!(to_f64(fmt, &sum), 1.0);
        }
    }

    #[test]
    fn integers_and_specials() {
        for fmt in ALL {
            let big = from_u64(fmt, u64::MAX);
            assert_eq!(to_u64(fmt, &big), u64::MAX);
            assert_eq!(to_i64(fmt, &from_i64(fmt, i64::MIN)), i64::MIN);
            assert_eq!(to_i64(fmt, &from_f64(fmt, -7.9)), -7);
            assert_eq!(to_u64(fmt, &from_f64(fmt, -1.0)), 0);
            let zero = from_f64(fmt, 0.0);
            let inf = arith(fmt, Arith::Div, &from_f64(fmt, 1.0), &zero);
            assert_eq!(to_f64(fmt, &inf), f64::INFINITY);
            let nan = arith(fmt, Arith::Mul, &inf, &zero);
            assert_eq!(compare(fmt, &nan, &nan), None);
            assert_eq!(to_f64(fmt, &neg(fmt, &inf)), f64::NEG_INFINITY);
            let minus_zero = neg(fmt, &zero);
            assert_eq!(compare(fmt, &minus_zero, &zero), Some(Ordering::Equal));
            assert_eq!(to_f32(fmt, &from_f64(fmt, 0.1)), 0.1f32);
            // Smallest subnormals survive a round trip through arithmetic.
            let tiny = from_f64(fmt, 5e-324);
            let two = from_f64(fmt, 2.0);
            let t2 = arith(fmt, Arith::Mul, &tiny, &two);
            assert_eq!(to_f64(fmt, &t2), 1e-323);
        }
    }

    #[test]
    fn subnormal_and_overflow_limits() {
        for fmt in ALL {
            let p = prec(fmt);
            // The smallest subnormal halves to zero (a tie, rounds to even).
            let smallest = encode(fmt, Num::Fin(false, 1, p.min_lsb()));
            let half = arith(fmt, Arith::Div, &smallest, &from_f64(fmt, 2.0));
            assert_eq!(decode(fmt, &half), Num::Zero(false));
            let max = encode(
                fmt,
                Num::Fin(false, (1u128 << p.bits) - 1, p.emax - p.bits as i32 + 1),
            );
            let twice = arith(fmt, Arith::Add, &max, &max);
            assert_eq!(decode(fmt, &twice), Num::Inf(false));
        }
    }
}
