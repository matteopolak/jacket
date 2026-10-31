//! `element_count_as_bytes`: `memcpy(dest.data, values.data, values.count)` with `values` an
//! array of `s32`.
//!
//! `count` is a number of elements; `memcpy`, `memset` and `memcmp` take a number of bytes.
//! With elements wider than a byte, the call copies, clears or compares only the first part of
//! the array.
//!
//! Fires on a call of a `@TakesByteCount` procedure whose last argument is `xs.count`, where
//! `xs` is an array (fixed-size, view or resizable) of elements larger than one byte and
//! `xs.data` is another argument of the same call. Arrays of `u8`, `s8` and `bool` are left
//! alone, as is a count already scaled (`xs.count * size_of(s32)`).
use super::{calls_noted, replacing};
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::walk::walk;
use jaic::ast::{Expr, ExprKind as E};
use jaic::types::TypeKind;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "element_count_as_bytes",
    category: crate::config::Category::Correctness,
    summary: "`memcpy(d, xs.data, xs.count)` with elements wider than a byte",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            let Some(E::Call {
                callee,
                args,
                ..
            }) = n.expr().map(|e| &e.kind)
            else {
                return true;
            };
            let Some((last, rest)) = args.split_last() else {
                return true;
            };
            if last.spread || last.context || !calls_noted(cx, callee, "TakesByteCount") {
                return true;
            }
            let E::Member(array, member) = &last.value.kind else {
                return true;
            };
            if member.name.as_str() != "count" {
                return true;
            }
            let data_passed = rest.iter().any(|a| {
                matches!(&uncast(&a.value).kind, E::Member(base, m)
                    if m.name.as_str() == "data" && cx.same_tokens(base, array))
            });
            if !data_passed {
                return true;
            }
            let Some((elem, size)) = element(cx, array) else {
                return true;
            };
            if size <= 1 {
                return true;
            }
            let count = cx.src(last.value.span).trim();
            out.push(replacing(
                last.value.span,
                format!(
                    "`{count}` counts `{elem}` elements, but this argument is a number of bytes"
                ),
                format!("multiply by the element size: `{count} * size_of({elem})`"),
                (last.value.span.end as usize, last.value.span.end as usize),
                format!(" * size_of({elem})"),
                false,
            ));
            true
        });
    }
}

/// The array's element type, by name, and its size in bytes.
fn element(cx: &Cx, array: &Expr) -> Option<(String, u64)> {
    let types = &cx.compiler.ty.types;
    let ty = cx.ty(array)?;
    let TypeKind::Array {
        elem, ..
    } = *types.kind(types.repr_struct(ty))
    else {
        return None;
    };
    Some((types.name(elem), types.size_of(elem)))
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
