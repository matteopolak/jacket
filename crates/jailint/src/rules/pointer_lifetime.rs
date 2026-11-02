//! Pointer and logical-element lifetime checks over the selected semantic CFG.
use crate::Finding;
use crate::syntax::Cx;

pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "pointer_lifetime",
    category: crate::config::Category::Restriction,
    summary: "a pointer used after its storage moves or its element becomes invalid",
    check,
};

fn check(cx: &Cx, out: &mut Vec<Finding>) {
    super::lifetime_cfg::check(cx, out, false);
}
