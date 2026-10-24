//! `lossy_float_literal`: `x: float32 = 31415.92653;`, a literal with more digits than
//! its `float32` type keeps.
//!
//! A `float32` holds about seven decimal digits. The digits written past the shortest spelling
//! that reads back as the same `float32` change nothing: they only suggest a precision the
//! program does not have (and the value stored is `3.1415927` either way). Copying a constant
//! from a `float64` table into `float32` code is the usual way to get here.
//!
//! Fires on a float literal that becomes a `float32` where it stands (the value of a
//! declaration of type `float32`, the right side of an assignment to a `float32`, the operand
//! beside a `float32` value) and whose significant digits outnumber those of the shortest
//! spelling of the same `float32`. `--fix` writes that spelling, which stores exactly the
//! same value. The compiler itself rejects a `float32` literal with more
//! than 8 digits after the point and types one with exactly 8 as `float64` (not seen here);
//! this finds what is left, such as `123456.789012`, where the digits before the point use up
//! the precision. Literals of other types, and hexadecimal floats, are left
//! alone.
use crate::syntax::{Cx, Node, walk};
use crate::{Edit, Finding, Fix};
use jaic::ast::{AssignOp, BinOp, Expr, ExprKind as E, Stmt, StmtKind as S, UnOp};
use jaic::types::TypeKind;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    look(cx, cx.stmts, out);
    for p in &cx.procs {
        if p.typed() {
            look(cx, &p.body.stmts, out);
        }
    }
}

fn look(cx: &Cx, stmts: &[Stmt], out: &mut Vec<Finding>) {
    walk(stmts, &mut |n, chain| {
        if let Node::Expr(e) = n
            && let E::Float(value) = e.kind
            && in_float32_context(cx, e, chain)
        {
            examine(cx, e, value, out);
        }
        true
    });
}

/// The literal is converted to `float32` by where it stands: the value of a declaration of
/// type `float32`, the right side of an assignment to a `float32`, or the other operand of an
/// arithmetic or comparison operator whose first operand is a `float32` value. (The type the
/// compiler records for a bare literal is its own, not the type it is converted to.)
fn in_float32_context(cx: &Cx, lit: &Expr, chain: &[Node<'_>]) -> bool {
    let types = &cx.compiler.ty.types;
    let float32 = |e: &Expr| {
        !cx.constant(e)
            && cx.ty(e).is_some_and(|t| {
                matches!(
                    types.kind(types.repr(t)),
                    TypeKind::Float {
                        bits: 32
                    }
                )
            })
    };
    // A minus sign in front is part of the value.
    let (lit, chain) = match chain.split_last() {
        Some((
            Node::Expr(
                parent @ Expr {
                    kind: E::Unary(UnOp::Neg, inner),
                    ..
                },
            ),
            rest,
        )) if inner.span == lit.span => (*parent, rest),
        _ => (lit, chain),
    };
    match chain.last() {
        Some(Node::Stmt(s)) => match &s.kind {
            S::Decl(d) => {
                d.value.as_ref().is_some_and(|v| v.span == lit.span)
                    && d.names.len() == 1
                    && d.ty
                        .as_ref()
                        .is_some_and(|t| matches!(cx.src(t.span).trim(), "float32" | "float"))
            }
            S::Assign {
                op: AssignOp::Assign,
                lhs,
                rhs,
            } => matches!(
                (lhs.as_slice(), rhs.as_slice()),
                ([target], [value]) if value.span == lit.span && float32(target)
            ),
            _ => false,
        },
        Some(Node::Expr(Expr {
            kind: E::Binary(op, a, b),
            ..
        })) if !matches!(op, BinOp::And | BinOp::Or) => {
            let other = if a.span == lit.span {
                b
            } else {
                a
            };
            !matches!(other.kind, E::Float(_) | E::Int(_)) && float32(other)
        }
        _ => false,
    }
}

fn examine(cx: &Cx, e: &Expr, value: f64, out: &mut Vec<Finding>) {
    let text = cx.src(e.span).trim();
    if text.is_empty() || text.contains(['h', 'x', 'X', 'H']) || !value.is_finite() {
        return;
    }
    let narrow = value as f32;
    if !narrow.is_finite() {
        return;
    }
    let written = significant_digits(text);
    let shortest = format!("{narrow:e}");
    let kept = significant_digits(&shortest);
    if written <= kept {
        return;
    }
    let replacement = spelling(narrow);
    let help = format!("write `{replacement}`, which is the same `float32`");
    out.push(Finding {
        start: e.span.start as usize,
        end: e.span.end as usize,
        message: format!("`{text}` has {written} significant digits, but a `float32` keeps {kept}"),
        label: None,
        help: Some(help.clone()),
        fix: Some(Fix {
            title: help,
            edits: vec![Edit {
                start: e.span.start as usize,
                end: e.span.end as usize,
                text: replacement,
            }],
            machine_applicable: true,
        }),
    });
}

/// How many digits of a decimal number carry value: those between the first and last non-zero
/// digit of its mantissa.
fn significant_digits(text: &str) -> usize {
    let mantissa = text.split(['e', 'E']).next().unwrap_or("");
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    digits.trim_matches('0').len()
}

/// The shortest decimal spelling that reads back as `value`, always with a `.` or an exponent
/// so it stays a float literal.
fn spelling(value: f32) -> String {
    let plain = format!("{value}");
    let exponent = format!("{value:e}");
    let short = if plain.len() <= exponent.len() + 4 {
        plain
    } else {
        exponent
    };
    if short.contains('.') {
        short
    } else if let Some((mantissa, exponent)) = short.split_once('e') {
        format!("{mantissa}.0e{exponent}")
    } else {
        format!("{short}.0")
    }
}
