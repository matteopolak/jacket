//! `use_after_free` frontend for the shared semantic lifetime CFG.
use crate::{Finding, syntax::Cx};

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "use_after_free",
    category: crate::config::Category::Correctness,
    summary: "a variable read after a `@FreesArgument` call freed it (`free(p); use(p);`)",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    out.extend(
        super::lifetime_cfg::resources::reports(cx)
            .use_after_free
            .iter()
            .cloned(),
    );
}
