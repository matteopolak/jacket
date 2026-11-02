//! Opt-in phase-one CFG analysis. See `lifetime_cfg` for its deliberately bounded coverage.
use crate::Finding;
use crate::syntax::Cx;

pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "pointer_lifetime",
    category: crate::config::Category::Restriction,
    summary: "experimental must-analysis of array pointers across branches and loops",
    check,
};

fn check(cx: &Cx, out: &mut Vec<Finding>) {
    super::lifetime_cfg::check(cx, out, false);
}
