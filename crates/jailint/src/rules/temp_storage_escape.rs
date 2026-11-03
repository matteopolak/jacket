//! `temp_storage_escape` frontend for the shared semantic lifetime CFG.
use crate::{Finding, syntax::Cx};

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "temp_storage_escape",
    category: crate::config::Category::Pedantic,
    summary: "temporary storage (`tprint`, `talloc`, or a procedure returning it) stored in a global or through a pointer, or returned",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    out.extend(
        super::lifetime_cfg::resources::reports(cx)
            .temp_storage_escape
            .iter()
            .cloned(),
    );
}
