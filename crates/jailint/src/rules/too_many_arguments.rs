//! `too_many_arguments`: a procedure with more parameters than `[lint] max_arguments`.
//!
//! ```jai
//! draw :: (x: int, y: int, w: int, h: int, r: u8, g: u8, b: u8, a: u8) { ... }
//! ```
//!
//! A call with this many arguments is hard to read and easy to get in the wrong order, and
//! the parameters usually belong together: group them in a struct (`Rect`, `Color`) and pass
//! that. Every parameter counts, defaulted, baked (`$T`) and variadic ones included. Limit
//! 7 by default. Procedures without a body (foreign declarations, which mirror a C
//! signature) and generated files are skipped. No fix. Off by default.
use super::proc_label;
use crate::Finding;
use crate::syntax::Cx;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "too_many_arguments",
    category: crate::config::Category::Restriction,
    summary: "a procedure with more parameters than `max_arguments` (default 7)",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    if cx.generated() {
        return;
    }
    for p in &cx.procs {
        let count = p.header.params.len();
        if count <= cx.max_arguments || p.header.uses_c_abi() {
            continue;
        }
        let (span, name) = proc_label(p);
        out.push(super::finding(
            span,
            format!(
                "{name} takes {count} parameters, past the limit of {}",
                cx.max_arguments
            ),
            Some("group parameters that belong together in a struct and pass that".into()),
        ));
    }
}
