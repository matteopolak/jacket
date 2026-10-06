//! `lossy_xx`: `xx` that converts a number to a type that cannot hold every value of it.
//!
//! `xx` picks its target from the context, so `small = xx big;` reads like a plain
//! assignment while it narrows (`s64` to `u8`, `float64` to `float32`, a float to an
//! integer). Integer casts wrap silently, so a value that does not fit is changed without a
//! trace. Writing `cast(u8)` makes the narrowing visible where it happens.
//!
//! Constants are not reported (the compiler checks they fit), nor `xx` with a modifier
//! (`xx,trunc` says what it means), nor values masked or shifted to fit (`xx (v & 0xFF)`,
//! `xx (v >> 56)`). Off by default (`allow`): narrowing `xx` is common and deliberate in
//! code that talks to C APIs.
use crate::facts::Recorded;
use crate::syntax::{Cx, walk};
use crate::{Edit, Finding, Fix};
use jaic::ast::{BinOp, Expr, ExprKind as E};
use jaic::types::{TypeId, TypeKind, Types};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    let types = &cx.compiler.types;
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            let Some(e) = n.expr() else {
                return true;
            };
            let E::Cast {
                ty: None,
                value,
                flags,
            } = &e.kind
            else {
                return true;
            };
            if *flags != Default::default() || !cx.src(e.span).starts_with("xx") {
                return true;
            }
            let Some(c) = cx.compiler.cast_fact(e.span) else {
                return true;
            };
            // `xx` is how an integer becomes an enum value; the enum's range is the point.
            if c.conflicting || c.from_constant || matches!(types.kind(c.target), TypeKind::Enum(_))
            {
                return true;
            }
            let Some(lost) = narrowing(types, c.from, c.target) else {
                return true;
            };
            if let Some(bits) = target_bits(types, c.target)
                && fits(value, bits)
            {
                return true;
            }
            let target = types.name(c.target);
            let builtin = matches!(
                types.kind(c.target),
                TypeKind::Int { .. } | TypeKind::Float { .. }
            ) && !target.contains(' ');
            let help = format!("write `cast({target})` to make the conversion visible");
            out.push(Finding {
                start: e.span.start as usize,
                end: e.span.end as usize,
                message: format!(
                    "`xx` converts `{}` to `{target}`, {lost}",
                    types.name(c.from)
                ),
                label: None,
                help: Some(help.clone()),
                fix: builtin.then(|| Fix {
                    title: help,
                    edits: vec![Edit {
                        start: e.span.start as usize,
                        end: e.span.start as usize + 2,
                        text: format!("cast({target})"),
                    }],
                    machine_applicable: true,
                }),
            });
            true
        });
    }
}

/// What converting `from` to `to` can lose, when it can.
fn narrowing(types: &Types, from: TypeId, to: TypeId) -> Option<&'static str> {
    // Enums convert through their base; distinct and alias types are compared as written.
    let (from, to) = (types.repr(from), types.repr(to));
    match (types.kind(from), types.kind(to)) {
        (
            TypeKind::Int {
                ..
            },
            TypeKind::Int {
                ..
            },
        ) => {
            let (fb, _) = types.int_info(from)?;
            let (tb, _) = types.int_info(to)?;
            (tb < fb).then_some("which can lose the high bits")
        }
        (
            TypeKind::Float {
                ..
            },
            TypeKind::Int {
                ..
            },
        ) => Some("which drops the fraction"),
        (
            TypeKind::Float {
                ..
            },
            TypeKind::Float {
                ..
            },
        ) => (types.size_of(to) < types.size_of(from)).then_some("which can lose precision"),
        _ => None,
    }
}

fn target_bits(types: &Types, to: TypeId) -> Option<u8> {
    types
        .int_info(to)
        .map(|(bits, signed)| bits - u8::from(signed))
}

/// The value is masked, shifted or reduced into `bits` bits.
fn fits(value: &Expr, bits: u8) -> bool {
    let limit = if bits >= 64 {
        u128::MAX
    } else {
        1u128 << bits
    };
    match &value.kind {
        E::Binary(BinOp::BitAnd, a, b) => {
            matches!(b.kind, E::Int(m) if m < limit) || matches!(a.kind, E::Int(m) if m < limit)
        }
        E::Binary(BinOp::Rem, _, b) => matches!(b.kind, E::Int(m) if m <= limit),
        E::Binary(BinOp::Shr, _, b) => {
            matches!(b.kind, E::Int(s) if 64 - s.min(64) <= bits as u128)
        }
        _ => false,
    }
}
