//! `banner_comment`: a comment made only of rule characters, used to fence off a section.
//!
//! ```jai
//! // ---------------------------------
//! // Names
//! // ---------------------------------
//! ```
//!
//! Write the section name as one plain `// Names` line, or move the section into its own file.
//! A rule line hides the real comment in the noise and invites a matching rule at every
//! heading. Reports each comment whose text is three or more of `-`, `=`, `*`, `#`, `~` or `_`
//! and nothing else, line or block. Generated files are skipped.
use crate::Finding;
use crate::syntax::Cx;
use jaic::lexer::CommentKind;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    if cx.generated() {
        return;
    }
    for c in cx.comments {
        if c.kind == CommentKind::Block && !c.terminated {
            continue;
        }
        if !is_rule(c.body(cx.text)) {
            continue;
        }
        out.push(Finding {
            start: c.span.start as usize,
            end: c.span.end as usize,
            message: "this comment is only a line of rule characters".into(),
            label: None,
            help: Some(
                "name the section in one plain `//` line, or drop the line; a long section can move to its own file"
                    .into(),
            ),
            fix: None,
        });
    }
}

/// Three or more rule characters, with nothing else but spaces around them.
fn is_rule(body: &str) -> bool {
    let body = body.trim();
    body.chars().count() >= 3 && body.chars().all(|c| "-=*#~_".contains(c))
}
