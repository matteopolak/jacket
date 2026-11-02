//! Selected collection-loop mutation analysis.
use crate::{Finding, syntax::Cx};

pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "remove_in_for",
    category: crate::config::Category::Correctness,
    summary: "removing from an array while a `for` walks it forward",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    super::lifetime_cfg::check_iteration(cx, out);
}
