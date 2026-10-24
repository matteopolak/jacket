//! `autocast_in_arg`: `xx` written as an argument of a call.
//!
//! ```jai
//! draw(xx width, xx height);
//! ```
//!
//! The call site now depends on the parameter types: changing `draw` to take `float` instead of
//! `s32` changes what `xx` does at every call, and nothing there says so. Passing `cast(s32)
//! width` (or converting before the call) keeps the conversion visible where it happens.
//!
//! Fires on an `xx` that is directly an argument of a call; other uses of `xx` belong to
//! `autocast_used`. When the compiler resolved the cast, the message names both types and the
//! editor offers `cast(T)` for plain `xx`; it is not applied by `--fix`. Off by default
//! (`allow`): `xx` in arguments is idiomatic Jai, especially for C APIs.
use super::autocast_used::{each_autocast, report};
use crate::Finding;
use crate::syntax::Cx;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    each_autocast(cx, &mut |e, as_argument| {
        if as_argument {
            out.push(report(cx, e, "leaves the target type to the parameter"));
        }
    });
}
