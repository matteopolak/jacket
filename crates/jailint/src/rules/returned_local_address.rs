//! `returned_local_address` frontend for the shared semantic lifetime CFG.
use crate::{Finding, syntax::Cx};

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "returned_local_address",
    category: crate::config::Category::Correctness,
    summary: "`return *x;` with `x` a local, a pointer into a stack frame that is gone",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    out.extend(
        super::lifetime_cfg::resources::reports(cx)
            .returned_local_address
            .iter()
            .cloned(),
    );
}
