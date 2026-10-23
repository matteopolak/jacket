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
//! or `all`, and suggests the closest rule name when one is near. `@jailint_allow(...)` notes are
//! not checked.
use super::RULES;
use crate::Finding;
use crate::config::Category;
use crate::suppress::directive;
use crate::syntax::Cx;
use jaic::lexer::CommentKind;

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
            out.push(Finding {
                start,
                end: start + name.len(),
                message: format!("`{name}` is not a jailint rule, so this comment allows nothing"),
                label: None,
                help: Some(match closest(name) {
                    Some(rule) => format!("did you mean `{rule}`? (`--list` shows every rule)"),
                    None => {
                        "`--list` shows every rule; `all` and the category names also work".into()
                    }
                }),
                fix: None,
            });
        }
    }
}

/// Where `part`, a slice of `text`, starts.
fn offset_in(text: &str, part: &str) -> Option<usize> {
    let start = (part.as_ptr() as usize).checked_sub(text.as_ptr() as usize)?;
    (start + part.len() <= text.len()).then_some(start)
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
