//! `unknown_allow`: a `// jailint: allow(...)` comment that names a rule jailint does not have.
//!
//! ```jai
//! // jailint: allow(unused_varaible)
//! x := 1;
//! ```
//!
//! A misspelt name silences nothing, so the finding it was meant to hide stays, or worse, the
//! comment is kept for a rule that was renamed and quietly stops covering anything. Reports each
//! name in a line comment's `allow(...)` or `allow-file(...)` list that is not a rule, a category
//! or `all`, and suggests the closest rule name when one is near. The editor offers to swap in the
//! suggestion; `--fix` does not, since the nearest name may not be the one meant. `@jailint_allow(...)` notes are
//! not checked.
use super::RULES;
use crate::config::Category;
use crate::suppress::{directive, offset_in};
use crate::syntax::Cx;
use crate::{Edit, Finding, Fix};
use jaic::lexer::CommentKind;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "unknown_allow",
    category: crate::config::Category::Suspicious,
    summary: "a `// jailint: allow(...)` comment naming a rule that does not exist",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for c in cx.comments {
        if c.kind != CommentKind::Line {
            continue;
        }
        let Some((_, names)) = directive(c.body(cx.text)) else {
            continue;
        };
        for name in names {
            if name == "all"
                || Category::parse(name).is_some()
                || RULES.iter().any(|r| r.name == name)
            {
                continue;
            }
            let Some(start) = offset_in(cx.text, name) else {
                continue;
            };
            let near = closest(name);
            out.push(Finding {
                start,
                end: start + name.len(),
                message: format!("`{name}` is not a jailint rule, so this comment allows nothing"),
                label: None,
                help: Some(match near {
                    Some(rule) => format!("did you mean `{rule}`? (`--list` shows every rule)"),
                    None => {
                        "`--list` shows every rule; `all` and the category names also work".into()
                    }
                }),
                fix: near.map(|rule| Fix {
                    title: format!("replace `{name}` with `{rule}`"),
                    edits: vec![Edit {
                        start,
                        end: start + name.len(),
                        text: rule.into(),
                    }],
                    machine_applicable: false,
                }),
            });
        }
    }
}

/// The rule whose name is within two edits of `name`.
fn closest(name: &str) -> Option<&'static str> {
    RULES
        .iter()
        .map(|r| (distance(name, r.name), r.name))
        .filter(|(d, _)| *d <= 2)
        .min_by_key(|(d, _)| *d)
        .map(|(_, n)| n)
}

/// Edit distance between two names (insertions, deletions and substitutions).
fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = (diagonal + usize::from(ca != *cb))
                .min(above + 1)
                .min(row[j] + 1);
            diagonal = above;
        }
    }
    row[b.len()]
}
