//! Integer intervals: what an expression of integer or bool type can evaluate to. A bool is
//! `0..=1`. The analyses that fade unreachable code build on this; a lint for conditions that
//! always hold, or a bounds check that cannot fail, can ask the same questions.
//!
//! Every function is sound: the interval returned holds every value the operation can give for
//! operands inside the intervals given, or the answer is `None` (nothing known). Values are
//! mathematical, so a caller must say when an operation may wrap in its type (it compares the
//! result with the type's range, see `Interval::within`).
use crate::ast::{BinOp, UnOp};
use crate::int_arith::{IntOp, int_binop};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Interval {
    pub lo: i128,
    pub hi: i128,
}

impl Interval {
    pub const BOOL: Interval = Interval {
        lo: 0,
        hi: 1,
    };

    pub fn new(lo: i128, hi: i128) -> Option<Interval> {
        (lo <= hi).then_some(Interval {
            lo,
            hi,
        })
    }

    pub fn point(v: i128) -> Interval {
        Interval {
            lo: v,
            hi: v,
        }
    }

    /// Every value of an integer type of `bits` bits; `None` past 64 bits, where the values
    /// would not fit the arithmetic here.
    pub fn of_type(bits: u8, signed: bool) -> Option<Interval> {
        if bits == 0 || bits > 64 {
            return None;
        }
        Some(if signed {
            Interval {
                lo: -(1i128 << (bits - 1)),
                hi: (1i128 << (bits - 1)) - 1,
            }
        } else {
            Interval {
                lo: 0,
                hi: (1i128 << bits) - 1,
            }
        })
    }

    pub fn singleton(self) -> Option<i128> {
        (self.lo == self.hi).then_some(self.lo)
    }

    pub fn within(self, outer: Interval) -> bool {
        outer.lo <= self.lo && self.hi <= outer.hi
    }

    /// The values in both, if any.
    pub fn meet(self, other: Interval) -> Option<Interval> {
        Interval::new(self.lo.max(other.lo), self.hi.min(other.hi))
    }

    /// Whether the value is known to be nonzero (`Some(true)`) or zero (`Some(false)`).
    pub fn truth(self) -> Option<bool> {
        if self.lo > 0 || self.hi < 0 {
            Some(true)
        } else if self.lo == 0 && self.hi == 0 {
            Some(false)
        } else {
            None
        }
    }

    fn from_truth(t: Option<bool>) -> Interval {
        match t {
            Some(b) => Interval::point(b as i128),
            None => Interval::BOOL,
        }
    }
}

pub fn is_comparison(op: BinOp) -> bool {
    matches!(
        op,
        BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge
    )
}

/// `a op b` for every pair of values in the intervals: always true, always false, or neither.
pub fn compare(op: BinOp, a: Interval, b: Interval) -> Option<bool> {
    match op {
        BinOp::Eq => {
            if a.meet(b).is_none() {
                Some(false)
            } else if a.singleton().is_some() && a == b {
                Some(true)
            } else {
                None
            }
        }
        BinOp::Ne => compare(BinOp::Eq, a, b).map(|t| !t),
        BinOp::Lt => {
            if a.hi < b.lo {
                Some(true)
            } else if a.lo >= b.hi {
                Some(false)
            } else {
                None
            }
        }
        BinOp::Le => {
            if a.hi <= b.lo {
                Some(true)
            } else if a.lo > b.hi {
                Some(false)
            } else {
                None
            }
        }
        BinOp::Gt => compare(BinOp::Lt, b, a),
        BinOp::Ge => compare(BinOp::Le, b, a),
        _ => None,
    }
}

/// The smallest `2^n - 1` at or above `v` (for `v >= 0`): every bit an `or` or `xor` of values
/// up to `v` could set.
fn smear(v: i128) -> i128 {
    if v <= 0 {
        0
    } else {
        let bits = 128 - v.leading_zeros();
        if bits >= 126 {
            i128::MAX
        } else {
            (1i128 << bits) - 1
        }
    }
}

fn corners(a: Interval, b: Interval, f: impl Fn(i128, i128) -> Option<i128>) -> Option<Interval> {
    let mut lo = i128::MAX;
    let mut hi = i128::MIN;
    for x in [a.lo, a.hi] {
        for y in [b.lo, b.hi] {
            let v = f(x, y)?;
            lo = lo.min(v);
            hi = hi.max(v);
        }
    }
    Interval::new(lo, hi)
}

/// `x op y` as the language computes it in a type wide enough to hold either, for the operators
/// where an interval's endpoints decide the result. `None` for a division by zero or one that
/// would wrap, which the arithmetic here does not model.
fn scalar(op: IntOp, x: i128, y: i128) -> Option<i128> {
    if op == IntOp::Div && x == i128::MIN && y == -1 {
        return None;
    }
    int_binop(op, x, y, 128, true).ok()
}

/// The mathematical result of `a op b`, `None` when it is not known (rotates, overflow of the
/// arithmetic here, a division that may be by zero). `&&` and `||` treat a nonzero operand as
/// true and give `0..=1`.
pub fn binary(op: BinOp, a: Interval, b: Interval) -> Option<Interval> {
    if is_comparison(op) {
        return Some(Interval::from_truth(compare(op, a, b)));
    }
    match op {
        BinOp::And => Some(Interval::from_truth(match (a.truth(), b.truth()) {
            (Some(false), _) | (_, Some(false)) => Some(false),
            (Some(true), Some(true)) => Some(true),
            _ => None,
        })),
        BinOp::Or => Some(Interval::from_truth(match (a.truth(), b.truth()) {
            (Some(true), _) | (_, Some(true)) => Some(true),
            (Some(false), Some(false)) => Some(false),
            _ => None,
        })),
        BinOp::Add => Interval::new(a.lo.checked_add(b.lo)?, a.hi.checked_add(b.hi)?),
        BinOp::Sub => Interval::new(a.lo.checked_sub(b.hi)?, a.hi.checked_sub(b.lo)?),
        BinOp::Mul => corners(a, b, i128::checked_mul),
        BinOp::Div => {
            if b.lo <= 0 && b.hi >= 0 {
                return None;
            }
            corners(a, b, |x, y| scalar(IntOp::Div, x, y))
        }
        BinOp::Rem => {
            let m = b.lo.checked_abs()?.max(b.hi.checked_abs()?);
            if m == 0 {
                return None;
            }
            let bound = m - 1;
            if a.lo >= 0 {
                Interval::new(0, a.hi.min(bound))
            } else if a.hi <= 0 {
                Interval::new(a.lo.max(-bound), 0)
            } else {
                Interval::new(a.lo.max(-bound), a.hi.min(bound))
            }
        }
        BinOp::BitAnd => match (a.lo >= 0, b.lo >= 0) {
            (true, true) => Interval::new(0, a.hi.min(b.hi)),
            (true, false) => Interval::new(0, a.hi),
            (false, true) => Interval::new(0, b.hi),
            _ => None,
        },
        BinOp::BitOr => {
            if a.lo >= 0 && b.lo >= 0 {
                Interval::new(a.lo.max(b.lo), smear(a.hi.max(b.hi)))
            } else {
                None
            }
        }
        BinOp::BitXor => {
            if a.lo >= 0 && b.lo >= 0 {
                Interval::new(0, smear(a.hi.max(b.hi)))
            } else {
                None
            }
        }
        BinOp::Shl => {
            if a.lo < 0 || b.lo < 0 || b.hi > 62 {
                return None;
            }
            Interval::new(
                a.lo.checked_mul(1i128 << b.lo)?,
                a.hi.checked_mul(1i128 << b.hi)?,
            )
        }
        BinOp::Shr => {
            if b.lo < 0 || b.hi > 126 {
                return None;
            }
            corners(a, b, |x, k| scalar(IntOp::Shr, x, k))
        }
        _ => None,
    }
}

/// The mathematical result of `op a` for a value of a type with values in `range`.
pub fn unary(op: UnOp, a: Interval, range: Interval) -> Option<Interval> {
    match op {
        UnOp::Plus => Some(a),
        UnOp::Neg => Interval::new(a.hi.checked_neg()?, a.lo.checked_neg()?),
        UnOp::Not => Some(Interval::from_truth(a.truth().map(|t| !t))),
        UnOp::BitNot => {
            if range.lo < 0 {
                Interval::new(-a.hi - 1, -a.lo - 1)
            } else {
                Interval::new(range.hi - a.hi, range.hi - a.lo)
            }
        }
        UnOp::Star | UnOp::Deref => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn iv(lo: i128, hi: i128) -> Interval {
        Interval::new(lo, hi).unwrap()
    }

    #[test]
    fn an_integer_type_spans_exactly_its_values() {
        assert_eq!(Interval::of_type(8, true), Some(iv(-128, 127)));
        assert_eq!(Interval::of_type(8, false), Some(iv(0, 255)));
        assert_eq!(Interval::of_type(1, true), Some(iv(-1, 0)));
        assert_eq!(Interval::of_type(1, false), Some(iv(0, 1)));
        assert_eq!(
            Interval::of_type(32, true),
            Some(iv(-(1 << 31), (1 << 31) - 1))
        );
        assert_eq!(
            Interval::of_type(64, true),
            Some(iv(i64::MIN.into(), i64::MAX.into()))
        );
        assert_eq!(Interval::of_type(64, false), Some(iv(0, u64::MAX.into())));
        // Past 64 bits the arithmetic here would overflow, and 0 bits is no type.
        assert_eq!(Interval::of_type(65, true), None);
        assert_eq!(Interval::of_type(0, false), None);
    }

    /// Every interval inside -5..=5.
    fn small() -> Vec<Interval> {
        (-5..=5)
            .flat_map(|lo| (lo..=5).map(move |hi| iv(lo, hi)))
            .collect()
    }

    fn values(a: Interval) -> impl Iterator<Item = i128> {
        a.lo..=a.hi
    }

    /// `op` applied to a pair of values the way the language does, for the operators whose result
    /// is defined for every pair here (`None` for a division by zero or a negative shift).
    fn concrete(op: BinOp, x: i128, y: i128) -> Option<i128> {
        Some(match op {
            BinOp::Add => x + y,
            BinOp::Sub => x - y,
            BinOp::Mul => x * y,
            BinOp::Div => scalar(IntOp::Div, x, y)?,
            BinOp::Rem => scalar(IntOp::Rem, x, y)?,
            BinOp::BitAnd => scalar(IntOp::And, x, y)?,
            BinOp::BitOr => scalar(IntOp::Or, x, y)?,
            BinOp::BitXor => scalar(IntOp::Xor, x, y)?,
            // A negative amount is left to the program.
            BinOp::Shl => scalar(IntOp::Shl, x, u32::try_from(y).ok()?.into())?,
            BinOp::Shr => scalar(IntOp::Shr, x, u32::try_from(y).ok()?.into())?,
            BinOp::Eq => i128::from(x == y),
            BinOp::Ne => i128::from(x != y),
            BinOp::Lt => i128::from(x < y),
            BinOp::Le => i128::from(x <= y),
            BinOp::Gt => i128::from(x > y),
            BinOp::Ge => i128::from(x >= y),
            BinOp::And => i128::from(x != 0 && y != 0),
            BinOp::Or => i128::from(x != 0 || y != 0),
            _ => return None,
        })
    }

    #[test]
    fn a_binary_result_holds_every_pair_and_is_exact_where_the_corners_decide_it() {
        use BinOp::*;
        for op in [
            Add, Sub, Mul, Div, Rem, BitAnd, BitOr, BitXor, Shl, Shr, Eq, Ne, Lt, Le, Gt, Ge, And,
            Or,
        ] {
            for a in small() {
                for b in small() {
                    let pairs: Vec<i128> = values(a)
                        .flat_map(|x| values(b).filter_map(move |y| concrete(op, x, y)))
                        .collect();
                    let Some(result) = binary(op, a, b) else {
                        continue;
                    };
                    // A division by zero or a negative shift is left to the program.
                    for v in &pairs {
                        assert!(
                            result.lo <= *v && *v <= result.hi,
                            "{op:?} {a:?} {b:?} = {result:?} misses {v}"
                        );
                    }
                    if matches!(op, Add | Sub | Mul | Div | Shr) && !pairs.is_empty() {
                        let (lo, hi) = (pairs.iter().min().unwrap(), pairs.iter().max().unwrap());
                        assert_eq!(
                            (result.lo, result.hi),
                            (*lo, *hi),
                            "{op:?} {a:?} {b:?} is not the tightest interval"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn comparisons_are_decided_only_when_every_pair_agrees_everywhere() {
        use BinOp::*;
        for op in [Eq, Ne, Lt, Le, Gt, Ge] {
            for a in small() {
                for b in small() {
                    let outcomes: Vec<bool> = values(a)
                        .flat_map(|x| values(b).map(move |y| concrete(op, x, y).unwrap() == 1))
                        .collect();
                    let all = |t: bool| outcomes.iter().all(|&o| o == t);
                    if let Some(t) = compare(op, a, b) {
                        assert!(all(t), "{op:?} {a:?} {b:?} said {t}");
                    }
                    // Ordering is decided whenever the intervals do not overlap or touch.
                    if matches!(op, Lt | Le | Gt | Ge) && (all(true) || all(false)) {
                        let overlap = a.meet(b).is_some();
                        if !overlap {
                            assert_eq!(compare(op, a, b), Some(all(true)), "{op:?} {a:?} {b:?}");
                        }
                    }
                    assert!(is_comparison(op));
                }
            }
        }
        for op in [Add, Sub, And, Or, BitAnd, Shl] {
            assert!(!is_comparison(op));
            assert_eq!(compare(op, iv(0, 1), iv(0, 1)), None);
        }
        // The boundaries: touching intervals decide `<=` and `>=` but not `<` and `>`.
        assert_eq!(compare(Lt, iv(1, 2), iv(2, 3)), None);
        assert_eq!(compare(Le, iv(1, 2), iv(2, 3)), Some(true));
        assert_eq!(compare(Lt, iv(2, 3), iv(1, 2)), Some(false));
        assert_eq!(compare(Le, iv(3, 4), iv(1, 2)), Some(false));
        assert_eq!(compare(Le, iv(2, 4), iv(1, 2)), None);
        assert_eq!(compare(Gt, iv(2, 3), iv(1, 2)), None);
        assert_eq!(compare(Ge, iv(2, 3), iv(1, 2)), Some(true));
    }

    #[test]
    fn logic_follows_three_valued_truth_for_every_pair() {
        use BinOp::{And, Or};
        let (zero, one, either) = (iv(0, 0), iv(1, 3), iv(0, 3));
        let truth = |r: Option<Interval>| r.and_then(Interval::singleton);
        assert_eq!(truth(binary(And, zero, either)), Some(0));
        assert_eq!(truth(binary(And, either, zero)), Some(0));
        assert_eq!(truth(binary(And, one, one)), Some(1));
        assert_eq!(truth(binary(And, one, either)), None);
        assert_eq!(truth(binary(And, either, one)), None);
        assert_eq!(truth(binary(Or, one, either)), Some(1));
        assert_eq!(truth(binary(Or, either, one)), Some(1));
        assert_eq!(truth(binary(Or, zero, zero)), Some(0));
        assert_eq!(truth(binary(Or, zero, either)), None);
        assert_eq!(truth(binary(Or, either, zero)), None);
        assert_eq!(binary(And, one, either), Some(Interval::BOOL));
    }

    #[test]
    fn unary_results_hold_every_value() {
        let signed = Interval::of_type(8, true).unwrap();
        let unsigned = Interval::of_type(8, false).unwrap();
        for a in small() {
            assert_eq!(unary(UnOp::Plus, a, signed), Some(a));
            let neg = unary(UnOp::Neg, a, signed).unwrap();
            assert_eq!((neg.lo, neg.hi), (-a.hi, -a.lo));
            let flipped = unary(UnOp::BitNot, a, signed).unwrap();
            assert_eq!((flipped.lo, flipped.hi), (-a.hi - 1, -a.lo - 1));
            if a.lo >= 0 {
                let flipped = unary(UnOp::BitNot, a, unsigned).unwrap();
                assert_eq!((flipped.lo, flipped.hi), (255 - a.hi, 255 - a.lo));
            }
            let not = unary(UnOp::Not, a, signed).unwrap();
            match a.truth() {
                Some(t) => assert_eq!(not, Interval::point(i128::from(!t))),
                None => assert_eq!(not, Interval::BOOL),
            }
        }
        assert_eq!(unary(UnOp::Deref, signed, signed), None);
        assert_eq!(unary(UnOp::Star, signed, signed), None);
    }

    #[test]
    fn containment_and_singletons() {
        for a in small() {
            for b in small() {
                let inside = values(a).all(|x| b.lo <= x && x <= b.hi);
                assert_eq!(a.within(b), inside, "{a:?} within {b:?}");
            }
            assert_eq!(a.singleton(), (a.lo == a.hi).then_some(a.lo));
        }
        assert!(iv(1, 2).within(iv(1, 2)));
        assert!(!iv(0, 2).within(iv(1, 2)));
        assert!(!iv(1, 3).within(iv(1, 2)));
    }

    #[test]
    fn comparisons_are_decided_only_when_every_pair_agrees() {
        let byte = Interval::of_type(8, false).unwrap();
        assert_eq!(compare(BinOp::Gt, byte, Interval::point(255)), Some(false));
        assert_eq!(compare(BinOp::Ge, byte, Interval::point(0)), Some(true));
        assert_eq!(compare(BinOp::Lt, byte, Interval::point(0)), Some(false));
        assert_eq!(compare(BinOp::Eq, byte, Interval::point(300)), Some(false));
        assert_eq!(compare(BinOp::Gt, byte, Interval::point(254)), None);
        assert_eq!(
            compare(BinOp::Ne, Interval::point(3), Interval::point(3)),
            Some(false)
        );
    }

    #[test]
    fn arithmetic_follows_the_operands() {
        let masked = binary(BinOp::BitAnd, iv(0, 1000), Interval::point(0xF)).unwrap();
        assert_eq!(masked, iv(0, 15));
        assert_eq!(binary(BinOp::Add, iv(1, 2), iv(10, 20)), Some(iv(11, 22)));
        assert_eq!(binary(BinOp::Sub, iv(1, 2), iv(10, 20)), Some(iv(-19, -8)));
        assert_eq!(binary(BinOp::Mul, iv(-2, 3), iv(4, 5)), Some(iv(-10, 15)));
        assert_eq!(binary(BinOp::Div, iv(1, 9), iv(-1, 1)), None);
        assert_eq!(
            binary(BinOp::Rem, iv(0, 100), Interval::point(7)),
            Some(iv(0, 6))
        );
        assert_eq!(
            binary(BinOp::Shr, iv(0, 255), Interval::point(4)),
            Some(iv(0, 15))
        );
        assert_eq!(binary(BinOp::BitOr, iv(0, 5), iv(0, 9)), Some(iv(0, 15)));
    }

    #[test]
    fn logic_combines_what_is_known() {
        let unknown = Interval::BOOL;
        let no = Interval::point(0);
        let yes = Interval::point(1);
        assert_eq!(binary(BinOp::And, unknown, no), Some(no));
        assert_eq!(binary(BinOp::Or, unknown, yes), Some(yes));
        assert_eq!(binary(BinOp::And, unknown, yes), Some(unknown));
        assert_eq!(unary(UnOp::Not, no, Interval::BOOL), Some(yes));
    }
}
