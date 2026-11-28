//! `narrow_pointers`: code whose meaning depends on 8-byte pointers.
//!
//! ```jai
//! tag :: (node: *Node) -> u64 { return (cast(u64) node) & 0xffff_0000_0000_0000; }
//! raw :: (slot: **Node) -> *u64 { return cast(*u64) slot; }
//! ```
//!
//! A pointer is 4 bytes on i686, wasm32 and armv7, and 2 on AVR. Three patterns break there
//! without failing to compile:
//!
//! - a mask, `|`, `^` or shift that touches bit 32 or above of a value made from a pointer
//!   (high-bit pointer tagging);
//! - `cast(*u64)` or `cast(*s64)` of the address of a pointer, which reads the pointer and the
//!   bytes after it as one integer;
//! - an integer constant of 2³² or more cast to a pointer.
//!
//! Off by default (`restriction`): enable it for a project that targets a narrow CPU. No fix.
use super::finding;
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::walk::walk;
use jaic::ast::{BinOp, Expr, ExprKind as E};

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "narrow_pointers",
    category: crate::config::Category::Restriction,
    summary: "code that assumes 8-byte pointers (high-bit tagging, `*u64` over a pointer)",
    check,
};

const HELP: &str = "a pointer is 4 bytes on 32-bit targets: keep tags in the low bits, or store them next to the pointer";

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    let types = &cx.compiler.ty.types;

    for p in &cx.procs {
        if !p.typed() {
            continue;
        }

        walk(&p.body.stmts, &mut |n, _| {
            let Some(e) = n.expr() else {
                return true;
            };

            match &e.kind {
                E::Cast {
                    value, ..
                } => {
                    let Some(to) = cx.ty(e) else {
                        return true;
                    };
                    let Some(target) = types.pointee(to) else {
                        return true;
                    };

                    if let Some((64, _)) = types.int_info(target)
                        && let Some(from) = cx.ty(value)
                        && let Some(held) = types.pointee(from)
                        && types.is_pointer(held)
                    {
                        out.push(finding(
                            e.span,
                            "this reads a pointer as a 64-bit integer: it is 4 bytes on 32-bit targets".into(),
                            Some("read the pointer through its own type".into()),
                        ));
                    }

                    if let Some(k) = super::int_literal(uncast(value))
                        && !(0..=i128::from(u32::MAX)).contains(&k)
                    {
                        out.push(finding(
                            e.span,
                            format!("the address {k:#x} does not fit in a 32-bit pointer"),
                            Some(
                                "use a value below 2^32, or build the pointer from a variable"
                                    .into(),
                            ),
                        ));
                    }
                }
                E::Binary(op, a, b) => {
                    if let Some(f) = tagged(cx, e, *op, a, b) {
                        out.push(f);
                    }
                }
                _ => {}
            }

            true
        });
    }
}

/// A bit operation reaching bit 32 or above of a value that came from a pointer.
fn tagged(cx: &Cx, e: &Expr, op: BinOp, a: &Expr, b: &Expr) -> Option<Finding> {
    let types = &cx.compiler.ty.types;
    let (value, k) = match (super::int_literal(b), super::int_literal(a)) {
        (Some(k), _) => (a, k),
        (None, Some(k)) if op != BinOp::Shl && op != BinOp::Shr => (b, k),
        _ => return None,
    };
    let reaches = match op {
        BinOp::Shl | BinOp::Shr => k >= 32,
        BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor => {
            !(-(1i128 << 32)..(1i128 << 32)).contains(&k)
        }
        _ => return None,
    };
    let from_pointer = cx.ty(uncast(value)).is_some_and(|t| types.is_pointer(t));

    (reaches && from_pointer).then(|| {
        finding(
            e.span,
            "this changes the high half of a 64-bit pointer: the bits do not exist on 32-bit targets".into(),
            Some(HELP.into()),
        )
    })
}

/// The value under any casts.
fn uncast(e: &Expr) -> &Expr {
    match &e.kind {
        E::Cast {
            value, ..
        } => uncast(value),
        _ => e,
    }
}
