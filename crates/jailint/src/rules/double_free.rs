//! `double_free` frontend for the shared semantic lifetime CFG.
use crate::{Finding, syntax::Cx};

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "double_free",
    category: crate::config::Category::Correctness,
    summary: "two calls that free the same variable with nothing in between (`free(p); free(p);`)",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    out.extend(
        super::lifetime_cfg::resources::reports(cx)
            .double_free
            .iter()
            .cloned(),
    );
}
