//! Applying fixes to source text.
use crate::{Edit, Lint};

/// `text` with the machine-applicable fixes of `lints` applied (those [`choose`] picks). Run
/// again to apply the rest. Returns the new text and how many fixes were applied.
pub fn apply(text: &str, lints: &[&Lint]) -> (String, usize) {
    let chosen: Vec<&Lint> = choose(lints)
        .into_iter()
        .filter(|l| {
            l.fix.as_ref().is_some_and(|f| {
                f.edits
                    .iter()
                    .all(|e| e.end <= text.len() && e.start <= e.end)
            })
        })
        .collect();
    let edits = chosen
        .iter()
        .flat_map(|l| l.fix.iter().flat_map(|f| f.edits.iter().cloned()))
        .collect();
    (apply_edits(text, edits), chosen.len())
}

/// The lints, in order, whose machine-applicable fix does not overlap the fix of one chosen
/// before it: the fixes that can be applied together.
pub fn choose<'l>(lints: &[&'l Lint]) -> Vec<&'l Lint> {
    let mut taken: Vec<&Edit> = Vec::new();
    let mut chosen = Vec::new();
    for &lint in lints {
        let Some(fix) = lint.fix.as_ref().filter(|f| f.machine_applicable) else {
            continue;
        };
        let clashes = fix.edits.iter().any(|e| {
            taken
                .iter()
                .any(|t| e.start < t.end.max(t.start + 1) && t.start < e.end.max(e.start + 1))
        });
        if clashes || fix.edits.iter().any(|e| e.start > e.end) {
            continue;
        }
        taken.extend(&fix.edits);
        chosen.push(lint);
    }
    chosen
}

/// `text` with non-overlapping `edits` applied.
pub fn apply_edits(text: &str, mut edits: Vec<Edit>) -> String {
    edits.sort_by_key(|e| std::cmp::Reverse((e.start, e.end)));
    let mut out = text.to_string();
    for e in edits {
        if e.end <= out.len() && out.is_char_boundary(e.start) && out.is_char_boundary(e.end) {
            out.replace_range(e.start..e.end, &e.text);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edits_apply_back_to_front() {
        let edits = vec![
            Edit {
                start: 0,
                end: 1,
                text: "AA".into(),
            },
            Edit {
                start: 4,
                end: 5,
                text: String::new(),
            },
        ];
        assert_eq!(apply_edits("a b c", edits), "AA b ");
    }
}
