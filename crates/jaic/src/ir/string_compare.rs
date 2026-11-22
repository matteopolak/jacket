//! What `Intrinsic::StringCompare` computes: the SSE4.2 string compares (`pcmpistri`,
//! `pcmpestri`, `pcmpistrm`, `pcmpestrm`) on two 16-byte operands. The interpreter runs this, and
//! the LLVM backend's portable lowering (`jaic_llvm::simd::generic::string_compare`) must agree
//! with it.

/// The result of one compare.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct StringCompare {
    /// Bit i is element i's result after the polarity (`IntRes2`), for the element count of the
    /// control byte: 16 bytes or 8 words.
    pub result: u32,
    /// ZF: `b` ends before its last element.
    pub b_ends: bool,
    /// SF: `a` ends before its last element.
    pub a_ends: bool,
}

/// The compare with control byte `ctl` of the 16 bytes at `a` and `b`. With `explicit`
/// lengths, `len_a` and `len_b` (absolute value, at most the element count) say how many
/// elements are part of each string; otherwise each ends before its first zero element.
///
/// ```
/// use jaic::ir::string_compare;
/// // Equal any (control 0): which bytes of "hello" are one of "lo".
/// let mut a = [0u8; 16];
/// a[..2].copy_from_slice(b"lo");
/// let mut b = [0u8; 16];
/// b[..5].copy_from_slice(b"hello");
/// let r = string_compare(0, false, &a, &b, 0, 0);
/// assert_eq!(r.result, 0b11100);
/// assert!(r.b_ends && r.a_ends);
/// ```
#[allow(
    clippy::disallowed_methods,
    reason = "compares UTF-16LE text, whose byte order is part of the format"
)]
pub fn string_compare(
    ctl: u8,
    explicit: bool,
    a: &[u8; 16],
    b: &[u8; 16],
    len_a: i32,
    len_b: i32,
) -> StringCompare {
    let words = ctl & 1 != 0;
    let signed = ctl & 2 != 0;
    let n = if words {
        8
    } else {
        16
    };
    // Sign- or zero-extended, so one comparison of `i64`s serves both.
    let lane = |v: &[u8; 16], i: usize| -> i64 {
        match (words, signed) {
            (false, false) => v[i] as i64,
            (false, true) => v[i] as i8 as i64,
            (true, false) => u16::from_le_bytes([v[2 * i], v[2 * i + 1]]) as i64,
            (true, true) => i16::from_le_bytes([v[2 * i], v[2 * i + 1]]) as i64,
        }
    };
    let a: Vec<i64> = (0..n).map(|i| lane(a, i)).collect();
    let b: Vec<i64> = (0..n).map(|i| lane(b, i)).collect();
    let valid = |v: &[i64], len: i32| -> Vec<bool> {
        if explicit {
            let len = len.unsigned_abs();
            (0..n).map(|i| (i as u32) < len).collect()
        } else {
            let end = v.iter().position(|&e| e == 0).unwrap_or(n);
            (0..n).map(|i| i < end).collect()
        }
    };
    let va = valid(&a, len_a);
    let vb = valid(&b, len_b);
    let mut result = 0u32;
    for i in 0..n {
        let r = match (ctl >> 2) & 3 {
            // Equal any: some valid element of `a` equals this valid element of `b`.
            0 => vb[i] && (0..n).any(|j| va[j] && a[j] == b[i]),
            // Ranges: `a` holds (low, high) pairs; this element lies in one of them.
            1 => {
                vb[i]
                    && (0..n / 2).any(|p| {
                        va[2 * p] && va[2 * p + 1] && a[2 * p] <= b[i] && b[i] <= a[2 * p + 1]
                    })
            }
            // Equal each: pairwise; two missing elements match, one missing does not.
            2 => (va[i] && vb[i] && a[i] == b[i]) || (!va[i] && !vb[i]),
            // Equal ordered: `a` occurs in `b` starting here (a missing `a` element matches).
            _ => (0..n - i).all(|j| !va[j] || (vb[i + j] && a[j] == b[i + j])),
        };
        // Polarity: negative inverts, masked negative inverts only the valid elements of `b`.
        let r = match (ctl >> 4) & 3 {
            1 => !r,
            3 => r != vb[i],
            _ => r,
        };
        result |= (r as u32) << i;
    }
    StringCompare {
        result,
        b_ends: !vb[n - 1],
        a_ends: !va[n - 1],
    }
}
