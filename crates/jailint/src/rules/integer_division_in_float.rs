//! `integer_division_in_float`: an integer division whose result is used as a float.
//!
//! ```jai
//! ratio := cast(float)(done / total);   // 0 until done == total
//! half  := 1 / 2 * width;               // 0 * width, with width: float
//! ```
//!
//! Operands are typed before the operator sees its context, so `done / total` divides two
//! integers and drops the remainder; converting the quotient to a float afterwards does not
//! bring it back. The fraction was almost certainly wanted.
//!
//! Fires on `/` between integers (by the checked types) whose value is directly converted to a
//! float (`cast(float32)`, `cast(float64)`, or `xx` to a float) or is an operand of arithmetic
//! that produces a float. A variable divided by a literal power of two (`cast(float)(w / 2)`,
//! centring on whole pixels) is taken as deliberate and not reported.
use super::{finding, int_literal};
use crate::Finding;
use crate::syntax::{Cx, Node, walk};
use jaic::ast::{BinOp, Expr, ExprKind as E};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, chain| {
            let Some(e) = n.expr() else {
                return true;
            };
            let E::Binary(BinOp::Div, a, b) = &e.kind else {
                return true;
            };
            let types = &cx.compiler.types;
            let int = |x: &Expr| cx.ty(x).is_some_and(|t| types.is_integer(t));
            if !int(e) || !int(a) || !int(b) {
                return true;
            }
            let Some(Node::Expr(parent)) = chain.last() else {
                return true;
            };
            let float = cx.ty(parent).is_some_and(|t| types.is_float(t));
            if !float {
                return true;
            }
            // Halving a variable (`w / 2`, centring on whole pixels) is usually meant.
            if !cx.constant(a) && int_literal(b).is_some_and(|d| d > 0 && (d & (d - 1)) == 0) {
                return true;
            }
            let how = match &parent.kind {
                E::Cast {
                    ..
                } => "converted to a float",
                E::Binary(BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem, ..) => {
                    "used in float arithmetic"
                }
                _ => return true,
            };
            let a_text = cx.whole_src(a.span).trim();
            let float_a = match a.kind {
                E::Int(_) => format!("{a_text}.0"),
                _ => format!("cast(float) {a_text}"),
            };
            let b_text = cx.whole_src(b.span).trim();
            out.push(finding(
                e.span,
                format!(
                    "integer division `{a_text} / {b_text}` is {how}: the remainder is lost first"
                ),
                Some(format!("divide as floats: `{float_a} / {b_text}`")),
            ));
            true
        });
    }
}
