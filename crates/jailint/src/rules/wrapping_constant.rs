//! `wrapping_constant`: a constant that silently wraps to the type its context gives it.
//!
//! ```jai
//! h: s32 = 7;
//! a := (0xffff_ffff - 40) / h;   // 0xffff_ffd7 becomes the s32 -41: a is -5
//! b := h < 0x8000_0000;          // h < -2147483648: always false
//! ```
//!
//! A constant operand takes the type of the other operand. A value outside that type's range
//! but within its bits (`0xffff_ffff` for an `s32`, `-2` for a `u16`) is accepted as that bit
//! pattern, so the operator works on a value the source does not show. (A value too wide for
//! the bits widens the expression instead: `u8_value + 300` is an `s64` sum.)
//!
//! Fires on a constant integer expression written with literals and arithmetic (`+ - * / %`,
//! shifts, `& | ^`, unary minus) that is an operand of `/`, `%` or an ordering comparison, or
//! the right side of `/=` or `%=`, when the other operand is a non-constant integer whose type
//! cannot hold the constant's value but takes its bits. `+`, `-` and `*` are not reported:
//! they give the same bits whether the constant wraps first or the result does
//! (`h + 0xffff_ffff` is `h - 1` either way). Nor are bitwise operators and `==`/`!=`, where the
//! bit pattern is the point (`h & 0xffff_ffff`, `x == 0xFFFF_FFFF`); anything using `~`; named
//! constants; casts (`cast(u8) 300`, `xx`), which say what they mean; enum operands.
use super::op_text;
use crate::facts::Recorded;
use crate::syntax::{Cx, is_atom, walk};
use crate::{Edit, Finding, Fix};
use jaic::ast::{AssignOp, BinOp, Expr, ExprKind as E, StmtKind as S, UnOp};
use jaic::types::{TypeId, TypeKind};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            if let Some(e) = n.expr()
                && let E::Binary(op, a, b) = &e.kind
                && wrap_matters(*op)
            {
                check_operand(cx, *op, a, b, cx.ty(b), true, out);
                check_operand(cx, *op, b, a, cx.ty(a), true, out);
            }
            if let Some(s) = n.stmt()
                && let S::Assign {
                    op: AssignOp::Op(op @ (BinOp::Div | BinOp::Rem)),
                    lhs,
                    rhs,
                } = &s.kind
                && let ([l], [r]) = (lhs.as_slice(), rhs.as_slice())
            {
                // The target's span is also the checker's temporary pointer to it; the
                // operation it desugars to (`*tmp % r`) has the statement's span.
                check_operand(cx, *op, r, l, cx.compiler.type_at(s.span), false, out);
            }
            true
        });
    }
}

/// Operators whose result changes when a constant operand wraps.
fn wrap_matters(op: BinOp) -> bool {
    matches!(
        op,
        BinOp::Div | BinOp::Rem | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge
    )
}

/// `constant` is an operand of `op` whose other operand is `other`, of type `ty`; `castable`:
/// `other` can be wrapped in a cast (it is not an assignment's target).
fn check_operand(
    cx: &Cx,
    op: BinOp,
    constant: &Expr,
    other: &Expr,
    ty: Option<TypeId>,
    castable: bool,
    out: &mut Vec<Finding>,
) {
    let Some(value) = eval(constant) else {
        return;
    };
    if eval(other).is_some() || cx.constant(other) {
        return;
    }
    let types = &cx.compiler.types;
    let Some(ty) = ty else {
        return;
    };
    // Integers only: an enum's range is its values, not the question here.
    let TypeKind::Int {
        bits,
        signed,
    } = *types.kind(ty)
    else {
        return;
    };
    if !(1..=64).contains(&bits) || !wraps(value, bits, signed) {
        return;
    }
    let wrapped = wrap(value, bits, signed);
    let text = cx.whole_src(constant.span).trim();
    let other_text = cx.whole_src(other.span).trim();
    let type_name = types.name(ty);
    let message = format!(
        "constant `{text}` is {value}, which does not fit in `{type_name}`: `{}` uses it as {wrapped}",
        op_text(op)
    );
    let label = format!("`{type_name}` comes from `{other_text}`");
    let wider = wider_type(value);
    let fix = wider
        .filter(|_| castable)
        .map(|w| format!("cast({w}) {}", operand(cx, other)));
    let help = match (&fix, wider) {
        (Some(cast), Some(w)) => format!(
            "to use {value}, compute in `{w}` (`{cast}`); to mean {wrapped}, write {wrapped}"
        ),
        (None, Some(w)) => format!(
            "to use {value}, compute in `{w}` and convert back; to mean {wrapped}, write {wrapped}"
        ),
        _ => format!("to mean {wrapped}, write {wrapped}"),
    };
    let (start, end) = cx.whole(other.span);
    out.push(Finding {
        start: constant.span.start as usize,
        end: constant.span.end as usize,
        message,
        label: Some(label),
        help: Some(help.clone()),
        fix: fix.map(|cast| Fix {
            title: help,
            edits: vec![Edit {
                start,
                end,
                text: cast,
            }],
            // The result's type changes, which its uses may not accept.
            machine_applicable: false,
        }),
    });
}

/// `other` as written, parenthesized unless it is a single term.
fn operand(cx: &Cx, other: &Expr) -> String {
    let text = cx.whole_src(other.span).trim();
    if is_atom(other) {
        text.to_string()
    } else {
        format!("({text})")
    }
}

/// `s64`, or `u64` for values above it.
fn wider_type(value: i128) -> Option<&'static str> {
    if i64::try_from(value).is_ok() {
        Some("s64")
    } else if u64::try_from(value).is_ok() {
        Some("u64")
    } else {
        None
    }
}

/// `value` is outside the type's range but within its bits, so the compiler takes the bits.
fn wraps(value: i128, bits: u8, signed: bool) -> bool {
    let half = 1i128 << (bits - 1);
    let (min, max) = if signed {
        (-half, half - 1)
    } else {
        (0, 2 * half - 1)
    };
    !(min..=max).contains(&value) && (-half..2 * half).contains(&value)
}

/// `value` reduced to `bits` bits, as the type reads them.
fn wrap(value: i128, bits: u8, signed: bool) -> i128 {
    let modulus = 1i128 << bits;
    let low = value.rem_euclid(modulus);
    if signed && low >= modulus / 2 {
        low - modulus
    } else {
        low
    }
}

/// The value of an integer expression made of literals and arithmetic; `None` for anything
/// else, including `~` (whose untyped value is negative by design) and overflow.
fn eval(e: &Expr) -> Option<i128> {
    Some(match &e.kind {
        E::Int(v) => i128::try_from(*v).ok()?,
        E::Unary(UnOp::Neg, a) => eval(a)?.checked_neg()?,
        E::Unary(UnOp::Plus, a) => eval(a)?,
        E::Binary(op, a, b) => {
            let (x, y) = (eval(a)?, eval(b)?);
            match op {
                BinOp::Add => x.checked_add(y)?,
                BinOp::Sub => x.checked_sub(y)?,
                BinOp::Mul => x.checked_mul(y)?,
                BinOp::Div => x.checked_div(y)?,
                BinOp::Rem => x.checked_rem(y)?,
                BinOp::Shl if (0..64).contains(&y) => x.checked_mul(1i128 << y)?,
                BinOp::Shr if (0..128).contains(&y) => x >> y,
                BinOp::BitAnd => x & y,
                BinOp::BitOr => x | y,
                BinOp::BitXor => x ^ y,
                _ => return None,
            }
        }
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_like_the_type() {
        assert_eq!(wrap(0xffff_ffd7, 32, true), -41);
        assert_eq!(wrap(-1, 8, false), 255);
        assert_eq!(wrap(0x8000_0000, 32, true), -0x8000_0000);
        assert!(wraps(0xffff_ffff, 32, true));
        assert!(wraps(-2, 16, false));
        assert!(!wraps(300, 8, false));
        assert!(!wraps(0x7fff_ffff, 32, true));
        assert!(!wraps(-129, 8, false));
        assert_eq!(wider_type(1 << 63), Some("u64"));
        assert_eq!(wider_type(-1), Some("s64"));
    }
}
