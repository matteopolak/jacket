//! The established resizable-array rule, backed by the selected semantic lifetime CFG.
use crate::{Finding, syntax::Cx};

pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "pointer_into_resizable",
    category: crate::config::Category::Suspicious,
    summary: "a pointer into a resizable array used after a `@GrowsArray` call may have moved the array (`p := *xs[i]; array_add(*xs, v); use(p);`)",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    super::lifetime_cfg::check(cx, out, true);
}
