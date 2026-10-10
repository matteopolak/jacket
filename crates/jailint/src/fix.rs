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

/// The lints, in order, whose machine-applicable fix can be applied together with the others.
///
/// A fix is left for a later pass when its edits overlap those of one chosen before it, or when
/// another fix sits entirely inside the span its edits cover: the inner fix changes what the
/// outer one was computed from (two nested `for` loops that both start binding `it`). The
/// innermost fixes go first; [`pending`] says whether another pass (a fresh lint) is due.
pub fn choose<'l>(lints: &[&'l Lint]) -> Vec<&'l Lint> {
    let candidates: Vec<(&'l Lint, (usize, usize))> = lints
        .iter()
        .filter_map(|&l| {
            let fix = l.fix.as_ref().filter(|f| f.machine_applicable)?;
            let lo = fix.edits.iter().map(|e| e.start).min()?;
            let hi = fix.edits.iter().map(|e| e.end).max()?;
            Some((l, (lo, hi)))
        })
        .collect();
    let mut taken: Vec<&Edit> = Vec::new();
    let mut chosen = Vec::new();
    for (i, &(lint, (lo, hi))) in candidates.iter().enumerate() {
        let Some(fix) = lint.fix.as_ref() else {
            continue;
        };
        let wraps_another = candidates.iter().enumerate().any(|(j, &(_, (olo, ohi)))| {
            j != i && lo <= olo && ohi <= hi && (olo, ohi) != (lo, hi)
        });
        let clashes = fix.edits.iter().any(|e| {
            taken
                .iter()
                .any(|t| e.start < t.end.max(t.start + 1) && t.start < e.end.max(e.start + 1))
        });
        if wraps_another || clashes || fix.edits.iter().any(|e| e.start > e.end) {
            continue;
        }
        taken.extend(&fix.edits);
        chosen.push(lint);
    }
    chosen
}

/// How many machine-applicable fixes [`choose`] leaves for a later pass.
pub fn pending(lints: &[&Lint]) -> usize {
    let total = lints
        .iter()
        .filter(|l| l.fix.as_ref().is_some_and(|f| f.machine_applicable))
        .count();
    total - choose(lints).len()
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

    fn lint(edits: &[(usize, usize)]) -> Lint {
        Lint {
            rule: "test",
            level: crate::config::Level::Warn,
            path: String::new(),
            start: 0,
            end: 0,
            message: String::new(),
            label: None,
            help: None,
            fix: Some(crate::Fix {
                title: String::new(),
                edits: edits
                    .iter()
                    .map(|&(start, end)| Edit {
                        start,
                        end,
                        text: "x".into(),
                    })
                    .collect(),
                machine_applicable: true,
            }),
        }
    }

    #[test]
    fn a_fix_around_another_waits_for_the_next_pass() {
        let outer = lint(&[(0, 2), (20, 22)]);
        let inner = lint(&[(5, 7), (10, 12)]);
        let apart = lint(&[(30, 31)]);
        let all = [&outer, &inner, &apart];
        let chosen = choose(&all);
        assert_eq!(chosen, vec![&inner, &apart]);
        assert_eq!(pending(&all), 1);
        assert_eq!(pending(&[&inner, &apart]), 0);
    }

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
