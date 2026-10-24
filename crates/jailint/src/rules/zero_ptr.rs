//! `zero_ptr`: `cast(*T) 0` is `null`.
//!
//! A null pointer has a name. Casting the number `0` to a pointer type says the same thing the
//! long way, and hides it from a search for `null`.
//!
//! Fires on `cast(*T) 0` (and `0x0`, `0b0`), with no cast modifier. `null` converts to every
//! pointer type, so the editor offers it; `--fix` leaves it, since in `p := cast(*u8) 0;` the
//! cast also gives `p` its type. Macros and generated files are left alone.
use super::{suggesting, walk_code};
use crate::syntax::Cx;
use crate::{Edit, Finding};
use jaic::ast::{CastFlags, ExprKind as E, UnOp};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    walk_code(cx, &mut |n, _| {
        let Some(e) = n.expr() else {
            return;
        };
        if let E::Cast {
            ty: Some(ty),
            value,
            flags,
        } = &e.kind
            && *flags == CastFlags::default()
            && matches!(ty.kind, E::Unary(UnOp::Star, _))
            && matches!(&value.kind, E::Int(v) if v.get() == 0)
        {
            out.push(suggesting(
                e.span,
                format!("`{}` is a null pointer", cx.src(e.span)),
                "write `null`".into(),
                "write `null`",
                vec![Edit {
                    start: e.span.start as usize,
                    end: e.span.end as usize,
                    text: "null".into(),
                }],
            ));
        }
    });
}
