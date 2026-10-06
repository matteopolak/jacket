//! `bitwise_precedence`: `1 << n - 1`, `flags | 1 << 3`, `x & 0xFF + 1`.
//!
//! Jai puts all bitwise and shift operators on one level that binds tighter than `*`, and
//! evaluates them left to right. C, and most languages after it, put shifts below `+` and give
//! `&`, `^`, `|` levels of their own. So code that reads naturally to a C programmer means
//! something else in Jai:
//!
//! | Written           | Jai reads           | C would read        |
//! | ----------------- | ------------------- | ------------------- |
//! | `1 << n - 1`      | `(1 << n) - 1`      | `1 << (n - 1)`      |
//! | `flags | 1 << 3`  | `(flags | 1) << 3`  | `flags | (1 << 3)`  |
//! | `x & 0xFF + 1`    | `(x & 0xFF) + 1`    | `x & (0xFF + 1)`    |
//!
//! Fires when a bitwise or shift operation is an operand of `+`, `-`, `*`, `/` or `%` without
//! parentheses, or is the left operand of another bitwise operator that C would bind tighter
//! (`a | b << c`, `a ^ b & c`, `a | b & c`). The fix writes the parentheses Jai already
//! implies, so behaviour does not change; it is machine-applicable. If the C reading was the
//! intent, move the parentheses instead.
use super::{op_text, parenthesized};
use crate::syntax::{Cx, walk};
use crate::{Edit, Finding, Fix};
use jaic::ast::{BinOp, Expr, ExprKind as E};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        walk(&p.body.stmts, &mut |n, _| {
            let Some(e) = n.expr() else {
                return true;
            };
            let E::Binary(op, a, b) = &e.kind else {
                return true;
            };
            for (child, left) in [(a, true), (b, false)] {
                let E::Binary(inner, ..) = &child.kind else {
                    continue;
                };
                let Some(inner_c) = c_bitwise_level(*inner) else {
                    continue;
                };
                let surprising = if arithmetic(*op) {
                    true
                } else if let Some(outer_c) = c_bitwise_level(*op) {
                    left && inner_c < outer_c
                } else {
                    false
                };
                if !surprising || parenthesized(cx, child) {
                    continue;
                }
                out.push(report(cx, e, child, *op, *inner));
            }
            true
        });
    }
}

fn arithmetic(op: BinOp) -> bool {
    matches!(
        op,
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem
    )
}

/// C's precedence among the bitwise operators (higher binds tighter); `None` for others.
fn c_bitwise_level(op: BinOp) -> Option<u8> {
    Some(match op {
        BinOp::Shl | BinOp::Shr | BinOp::Rotl | BinOp::Rotr => 4,
        BinOp::BitAnd => 3,
        BinOp::BitXor => 2,
        BinOp::BitOr => 1,
        _ => return None,
    })
}

fn report(cx: &Cx, whole: &Expr, child: &Expr, outer: BinOp, inner: BinOp) -> Finding {
    let (start, end) = cx.whole(child.span);
    let child_text = &cx.text[start..end];
    let help = format!(
        "`{}` binds first here: write `({child_text})`",
        op_text(inner)
    );
    Finding {
        start: whole.span.start as usize,
        end: whole.span.end as usize,
        message: if arithmetic(outer) {
            format!(
                "`{}` binds tighter than `{}` in Jai, unlike in C",
                op_text(inner),
                op_text(outer)
            )
        } else {
            format!(
                "`{}` and `{}` share a level in Jai and apply left to right; C would apply `{}` first",
                op_text(inner),
                op_text(outer),
                op_text(outer)
            )
        },
        label: None,
        help: Some(help.clone()),
        fix: Some(Fix {
            title: format!("add parentheses around `{child_text}`"),
            edits: vec![
                Edit {
                    start,
                    end: start,
                    text: "(".into(),
                },
                Edit {
                    start: end,
                    end,
                    text: ")".into(),
                },
            ],
            machine_applicable: true,
        }),
    }
}
