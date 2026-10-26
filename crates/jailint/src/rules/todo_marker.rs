//! `todo_marker`: a comment that carries a `TODO`, `FIXME`, `XXX` or `HACK` marker.
//!
//! ```jai
//! // TODO: handle the empty list
//! ```
//!
//! For a project that wants unfinished work tracked outside the source, or a tree with no
//! markers left before a release. A marker is one of those words in capitals, standing alone:
//! `TODO:` and `(FIXME)` count, `TODOS`, `todo` and `AUTODOC` do not. Reports each comment
//! once, naming the first marker in it. Generated files are skipped. No fix: the comment is
//! the author's note.
use super::finding;
use crate::Finding;
use crate::syntax::Cx;
use jaic::lexer::CommentKind;

/// The markers a comment can carry; `detached_doc_comment` leaves comments that start with one.
pub(crate) const MARKERS: [&str; 4] = ["TODO", "FIXME", "XXX", "HACK"];

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "todo_marker",
    category: crate::config::Category::Restriction,
    summary: "a comment with a `TODO`, `FIXME`, `XXX` or `HACK` marker",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    if cx.generated() {
        return;
    }
    for c in cx.comments {
        if c.kind == CommentKind::Block && !c.terminated {
            continue;
        }
        let Some(marker) = first_marker(c.body(cx.text)) else {
            continue;
        };
        out.push(finding(
            c.span,
            format!("this comment has a `{marker}` marker"),
            Some("finish it, or track it outside the source and remove the marker".into()),
        ));
    }
}

/// The first marker in `text` that stands as a word of its own.
fn first_marker(text: &str) -> Option<&'static str> {
    let word = |c: char| c.is_alphanumeric() || c == '_';
    let mut best: Option<(usize, &'static str)> = None;
    for marker in MARKERS {
        let mut from = 0;
        while let Some(i) = text[from..].find(marker) {
            let at = from + i;
            let end = at + marker.len();
            from = end;
            let before = text[..at].chars().next_back().is_some_and(word);
            let after = text[end..].chars().next().is_some_and(word);
            if !before && !after {
                if best.is_none_or(|(p, _)| at < p) {
                    best = Some((at, marker));
                }
                break;
            }
        }
    }
    best.map(|(_, m)| m)
}
