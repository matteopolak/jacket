//! `shift_overflow`: `x << 40` with `x: s32`, a shift by at least the width of the operand.
//!
//! Every bit of the value is shifted out: `x << 40` and `x >> 32` are `0` for a 32-bit `x`
//! (`-1` for `>>` on a negative signed one), whatever `x` holds. The compiler accepts it. It
//! usually means the operand is narrower than the author thought (`u8 << 8`, a `u32` where a
//! `u64` was needed), so the bits that should survive are lost; widen the operand in parentheses
//! first, `(cast(u64) x) << 40` (`cast(u64) x << 40` shifts first and casts after).
//!
//! Fires on `<<`, `>>` and their compound assignments when the shift count is an integer literal
//! no less than the bit width of the left operand's integer type. Left operands that are
//! constants (`1 << 40` takes its type from where it is used) and rotates (`<<<`, `>>>`, which
//! wrap the count) are not looked at.
use super::int_literal;
use crate::Finding;
use crate::syntax::{Cx, walk};
use jaic::ast::{AssignOp, BinOp, Expr, ExprKind as E, StmtKind as S};
use jaic::source::Span;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            match n {
                crate::syntax::Node::Expr(e) => {
                    if let E::Binary(BinOp::Shl | BinOp::Shr, value, count) = &e.kind {
                        examine(cx, e.span, value, count, out);
                    }
                }
                crate::syntax::Node::Stmt(s) => {
                    if let S::Assign {
                        op: AssignOp::Op(BinOp::Shl | BinOp::Shr),
                        lhs,
                        rhs,
                    } = &s.kind
                        && let ([value], [count]) = (lhs.as_slice(), rhs.as_slice())
                    {
                        examine(cx, s.span, value, count, out);
                    }
                }
            }
            true
        });
    }
}

fn examine(cx: &Cx, span: Span, value: &Expr, count: &Expr, out: &mut Vec<Finding>) {
    let types = &cx.compiler.ty.types;
    let Some(n) = int_literal(count).filter(|&n| n >= 0) else {
        return;
    };
    if cx.constant(value) {
        return;
    }
    let Some(ty) = cx.ty(value) else {
        return;
    };
    let Some((bits, _)) = types.int_info(ty) else {
        return;
    };
    if n < i128::from(bits) {
        return;
    }
    out.push(super::finding(
        span,
        format!(
            "shifting a `{}` by {n} shifts out all {bits} of its bits",
            types.name(ty)
        ),
        Some(format!(
            "the shift count must be below {bits}; widen the value first, `(cast(u64) x) << {n}`, if more bits are needed"
        )),
    ));
}
