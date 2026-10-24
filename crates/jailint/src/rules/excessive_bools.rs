//! `excessive_bools`: a procedure with more `bool` parameters than `[lint] max_bools`.
//!
//! ```jai
//! open :: (path: string, read: bool, write: bool, create: bool, truncate: bool) { ... }
//! ```
//!
//! At the call, `open(path, true, false, true, false)` says nothing about which flag is
//! which. Replace the flags with an enum or a `#flags` enum (`Open_Flags.READ | .CREATE`) or
//! split the procedure. A parameter counts when its type is written `bool` or it has no type
//! and defaults to `true` or `false`. Limit 3 by default. `#c_call` and foreign procedures
//! (whose signature is fixed by C) and generated files are skipped. No fix. Off by default.
use super::proc_label;
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::{ExprKind as E, Param};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    if cx.generated() {
        return;
    }
    for p in &cx.procs {
        if p.header.uses_c_abi() {
            continue;
        }
        let count = p.header.params.iter().filter(|q| is_bool(q)).count();
        if count <= cx.max_bools {
            continue;
        }
        let (span, name) = proc_label(p);
        out.push(super::finding(
            span,
            format!(
                "{name} takes {count} `bool` parameters, past the limit of {}",
                cx.max_bools
            ),
            Some(
                "use an enum or a `#flags` enum so each argument says what it sets, or split the procedure"
                    .into(),
            ),
        ));
    }
}

fn is_bool(param: &Param) -> bool {
    if param.variadic {
        return false;
    }
    match (&param.ty, &param.default) {
        (Some(ty), _) => matches!(&ty.kind, E::Ident(n) if n.as_str() == "bool"),
        (None, Some(value)) => matches!(value.kind, E::Bool(_)),
        (None, None) => false,
    }
}
