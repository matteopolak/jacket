//! `absurd_comparison`: `count >= 0` with an unsigned `count`, `index < 0` with `index: u32`.
//!
//! An unsigned integer is never below zero, so these comparisons are always true or always
//! false. As a guard (`if i < 0 return;`) the check does nothing, and as a loop condition
//! (`while i >= 0 { ...; i -= 1; }` counting down an unsigned `i`) it never stops: `i` wraps
//! around to its largest value instead of going negative.
//!
//! Fires when an integer literal is compared with a non-constant integer whose type cannot
//! hold any value on the other side of the literal: `u >= 0`, `u < 0`, `0 > u`, `u > -1`, and
//! the same at the bottom of a signed type (`s8 < -128`). Only the low end is checked: the
//! width of aliases like `c_ulong` differs between platforms, so a comparison with the top of
//! a type may matter on another one.
use super::{int_literal, op_text, replacing};
use crate::Finding;
use crate::syntax::{Cx, walk};
use jaic::ast::{BinOp, ExprKind as E};
use jaic::types::TypeKind;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            let Some(e) = n.expr() else {
                return true;
            };
            let E::Binary(op, a, b) = &e.kind else {
                return true;
            };
            // As `value OP literal`.
            let (value, op, literal) = match (int_literal(a), int_literal(b)) {
                (None, Some(c)) => (a, *op, c),
                (Some(c), None) => (b, flip(*op), c),
                _ => return true,
            };
            let Some(ty) = cx.ty(value) else {
                return true;
            };
            let TypeKind::Int {
                bits,
                signed,
            } = *cx.compiler.types.kind(ty)
            else {
                return true;
            };
            if cx.constant(value) {
                return true;
            }
            let min: i128 = if signed {
                -(1i128 << (bits - 1))
            } else {
                0
            };
            let always = match op {
                BinOp::Ge if literal <= min => true,
                BinOp::Gt if literal < min => true,
                BinOp::Lt if literal <= min => false,
                BinOp::Le if literal < min => false,
                _ => return true,
            };
            let ty_name = cx.compiler.types.name(ty);
            let shown = cx.whole_src(value.span).trim();
            let why = if signed {
                format!("`{ty_name}` holds nothing below {min}")
            } else {
                format!("`{ty_name}` is unsigned and holds nothing below 0")
            };
            // Writing the answer down is offered, not applied by `--fix`: the check is
            // usually a bug to understand (a wrapping countdown), not text to simplify.
            out.push(replacing(
                e.span,
                format!("`{shown} {} {literal}` is always `{always}`", op_text(op)),
                format!("{why}; compare with a signed value, or drop the check"),
                (e.span.start as usize, e.span.end as usize),
                always.to_string(),
                false,
            ));
            true
        });
    }
}

/// `c OP x` as `x OP' c`.
fn flip(op: BinOp) -> BinOp {
    match op {
        BinOp::Lt => BinOp::Gt,
        BinOp::Gt => BinOp::Lt,
        BinOp::Le => BinOp::Ge,
        BinOp::Ge => BinOp::Le,
        other => other,
    }
}
