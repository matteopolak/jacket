//! `incomplete_enum_switch`: a `switch` on an enum that neither covers every member nor says
//! what happens to the rest.
//!
//! ```jai
//! if kind == {
//!     case .CIRCLE;   area_circle();
//!     case .SQUARE;   area_square();
//! }
//! ```
//!
//! When `Shape` later gains a `TRIANGLE`, this switch quietly does nothing for it. Mark the
//! switch `#complete` so the compiler lists every member it forgot, or add `case;` to name the
//! fallback. The finding lists the members no case mentions.
//!
//! Fires on a `switch` whose subject is a plain enum (not an `enum_flags`, not a loose enum)
//! when it is not `#complete`, has no default case and has a case that is not a plain member
//! name (a computed case value hides which members are covered, so those switches are left
//! alone). A member counts as covered when a case lists another member with the same value.
//! Off by default (`allow`): a switch that handles two members of ten is often deliberate. No fix.
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::{ExprKind as E, StmtKind as S};
use jaic::fxhash::HashSet;
use jaic::types::TypeKind;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "incomplete_enum_switch",
    category: crate::config::Category::Pedantic,
    summary: "a `switch` on an enum that neither covers every member nor has a default case",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    let types = &cx.compiler.ty.types;
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        crate::syntax::walk(&p.body.stmts, &mut |n, _| {
            let Some(s) = n.stmt() else {
                return true;
            };
            let S::Switch(sw) = &s.kind else {
                return true;
            };
            if sw.complete || sw.cases.iter().any(|c| c.values.is_empty()) {
                return true;
            }
            let Some(ty) = cx.ty(&sw.value) else {
                return true;
            };
            let TypeKind::Enum(id) = types.kind(ty) else {
                return true;
            };
            let info = types.enum_info(*id);
            if info.is_flags || info.loose_of.is_some() || types.is_loose_enum(ty) {
                return true;
            }
            // The names the cases list; anything else means the coverage cannot be read.
            let mut listed = HashSet::default();
            for v in sw.cases.iter().flat_map(|c| &c.values) {
                match &v.kind {
                    E::InferredMember(m) | E::Member(_, m) => {
                        listed.insert(m.name);
                    }
                    _ => return true,
                }
            }
            let covered: HashSet<i128> = info
                .members
                .iter()
                .filter(|(name, _)| listed.contains(name))
                .map(|(_, value)| *value)
                .collect();
            let mut seen = HashSet::default();
            let missing: Vec<String> = info
                .members
                .iter()
                .filter(|(_, value)| !covered.contains(value) && seen.insert(*value))
                .map(|(name, _)| format!(".{}", name.as_str()))
                .collect();
            if missing.is_empty() {
                return true;
            }
            let header =
                jaic::source::Span::new(cx.file, s.span.start as usize, sw.header.end as usize);
            out.push(super::finding(
                header,
                format!(
                    "this `switch` on `{}` does not handle {}",
                    types.name(ty),
                    missing.join(", ")
                ),
                Some(
                    "handle them, or add `case;` for the rest, or write `#complete` so the compiler checks the list"
                        .into(),
                ),
            ));
            true
        });
    }
}
