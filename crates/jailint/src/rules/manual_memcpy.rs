//! `manual_memcpy`: `for i: 0..n-1 { dst[i] = src[i]; }`, an element-by-element copy.
//!
//! ```jai
//! for i: 0..count-1 {
//!     dst[i] = src[i];
//! }
//! ```
//!
//! `memcpy(dst.data, src.data, count * size_of(T))` moves the same bytes in one call (and a
//! fixed array can be assigned whole: `dst = src;`). The loop is the long way round, and each
//! iteration pays for a bounds check on both arrays.
//!
//! Fires on a `for` over a range starting at `0` whose body is only `dst[i] = src[i];` with `i`
//! the loop's variable, `dst` and `src` different names or member paths, and the same element
//! type. `memcpy` assumes the two do not overlap, which two different arrays do not, but a
//! pointer or slice can alias: that is for the author to check, so there is no fix.
use crate::Finding;
use crate::syntax::{Cx, is_path, walk};
use jaic::ast::{AssignOp, BinOp, Expr, ExprKind as E, For, ForOver, Stmt, StmtKind as S};
use jaic::intern::Sym;
use jaic::types::TypeKind;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            if let Some(s) = n.stmt()
                && let S::For(f) = &s.kind
                && let Some(found) = examine(cx, s, f)
            {
                out.push(found);
            }
            true
        });
    }
}

/// The single statement of the loop body, braces or not.
fn only_statement(body: &Stmt) -> Option<&Stmt> {
    match &body.kind {
        S::Block(b) => match b.stmts.as_slice() {
            [one] => only_statement(one),
            _ => None,
        },
        _ => Some(body),
    }
}

fn examine(cx: &Cx, stmt: &Stmt, f: &For) -> Option<Finding> {
    let types = &cx.compiler.ty.types;
    if f.by_pointer
        || f.reverse
        || f.pointer_if.is_some()
        || f.reverse_if.is_some()
        || f.iterator.is_some()
        || f.backtick_names
        || f.index.is_some()
    {
        return None;
    }
    let ForOver::Range(lo, hi) = &f.over else {
        return None;
    };
    if !matches!(lo.kind, E::Int(n) if n.get() == 0) {
        return None;
    }
    let var = f.it.map_or_else(|| Sym::intern("it"), |i| i.name);
    let body = only_statement(&f.body)?;
    let S::Assign {
        op: AssignOp::Assign,
        lhs,
        rhs,
    } = &body.kind
    else {
        return None;
    };
    let ([to], [from]) = (lhs.as_slice(), rhs.as_slice()) else {
        return None;
    };
    let (dst, src) = (indexed_by(to, var)?, indexed_by(from, var)?);
    if !is_path(dst) || !is_path(src) || cx.same_tokens(dst.span, src.span) {
        return None;
    }
    // Arrays or pointers on both sides, with the very same element type.
    let element = |base: &Expr| match types.kind(cx.ty(base)?) {
        TypeKind::Array {
            elem, ..
        } => Some((*elem, true)),
        TypeKind::Pointer(elem) => Some((*elem, false)),
        _ => None,
    };
    let ((dt, d_array), (st, s_array)) = (element(dst)?, element(src)?);
    if dt != st || cx.ty(to)? != dt || cx.ty(from)? != dt {
        return None;
    }
    let count = match &hi.kind {
        E::Binary(BinOp::Sub, c, one) if matches!(one.kind, E::Int(n) if n.get() == 1) => {
            cx.whole_src(c.span).trim().to_string()
        }
        E::Int(n) => (n.get() + 1).to_string(),
        _ => format!("({} + 1)", cx.whole_src(hi.span).trim()),
    };
    let (d, s) = (cx.src(dst.span).trim(), cx.src(src.span).trim());
    let data = |name: &str, array: bool| {
        if array {
            format!("{name}.data")
        } else {
            name.to_string()
        }
    };
    let help = format!(
        "copy the elements at once: `memcpy({}, {}, {count} * size_of({}))`",
        data(d, d_array),
        data(s, s_array),
        types.name(dt)
    );
    let header = jaic::source::Span::new(
        cx.file,
        stmt.span.start as usize,
        f.body.span.start as usize,
    );
    Some(super::finding(
        header,
        format!("this loop copies `{s}` into `{d}` one element at a time"),
        Some(help),
    ))
}

/// `base[var]`: the base.
fn indexed_by(e: &Expr, var: Sym) -> Option<&Expr> {
    match &e.kind {
        E::Index(base, index) if matches!(index.kind, E::Ident(n) if n == var) => Some(base),
        _ => None,
    }
}
