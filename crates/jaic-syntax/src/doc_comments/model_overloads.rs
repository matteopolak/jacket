//! Overloads that are one procedure written for several systems.
use super::{Overload, os_mask, os_names};

/// Declarations of one signature are alternatives (the branches of an `#if`, or files loaded for
/// different systems), not overloads: they become one, limited to the systems of them all. Their
/// docs are kept as long as they do not tell different things; a version with docs wins over one
/// without.
pub(super) fn merge_alternatives(overloads: &mut Vec<Overload>) {
    let mut merged: Vec<Overload> = Vec::new();
    for o in std::mem::take(overloads) {
        let same = |h: &Overload| {
            h.signature
                .split_whitespace()
                .eq(o.signature.split_whitespace())
                && (h.docs == o.docs || h.docs.is_empty() || o.docs.is_empty())
        };
        let Some(at) = merged.iter().position(same) else {
            merged.push(o);
            continue;
        };
        let have = &mut merged[at];
        let mask = os_mask(&have.platforms) | os_mask(&o.platforms);
        have.platforms = os_names(mask);
        if have.docs.is_empty() {
            have.docs = o.docs;
            have.docs_line = o.docs_line;
            have.params = o.params;
            have.returns = o.returns;
        }
    }
    *overloads = merged;
}
