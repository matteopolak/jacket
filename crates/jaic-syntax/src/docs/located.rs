//! Doc comments with the line they start on.
use super::{Comment, CommentKind, DocText, Trivia, decoration, line_start, trivia, valid};

/// A doc comment and the line of the file its first line is on (1-based). Each line of `text`
/// is the next line of the file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Located {
    /// The line of the file the first line of `text` is on, counting from 1.
    pub line: usize,
    /// The comment's lines without their markers, with leading and trailing blank lines removed.
    pub text: String,
}

fn located(t: &Trivia, run: &[&Comment]) -> Option<Located> {
    let first = Trivia::start(run.first()?);
    let doc = DocText::new(t, run);
    let mut lines: Vec<&str> = doc.text.lines().collect();
    let mut line = t.text[..first].matches('\n').count() + 1;

    while lines.first().is_some_and(|l| l.is_empty()) {
        lines.remove(0);
        line += 1;
    }

    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }

    let text = lines.join("\n");

    (!text.trim().is_empty()).then_some(Located {
        line,
        text,
    })
}

/// [`above`], with the line it starts on.
pub fn located_above(text: &str, at: usize) -> Option<Located> {
    if !valid(text, at) {
        return None;
    }

    let t = trivia(text);

    located(
        &t,
        &t.docs_run(export_lines_above(&t.text, line_start(&t.text, at))),
    )
}

/// The comments that open `text`, before any code: a block comment, or the run of `//` lines
/// (up to a rule). They document a module when no declaration follows them directly.
pub fn opening(text: &str) -> Option<Located> {
    let t = trivia(text);
    let first = t.comments.first()?;

    if !t.owns_line(first) || t.text[..Trivia::start(first)].trim() != "" {
        return None;
    }

    if t.tokens
        .first()
        .is_some_and(|token| (token.span.start as usize) < Trivia::start(first))
    {
        return None;
    }

    let mut run = vec![first];

    if first.kind == CommentKind::Block {
        run.retain(|c| c.terminated);
    } else {
        for c in &t.comments[1..] {
            let last = run.last()?;

            if c.kind != CommentKind::Line || !t.directly_above(last, Trivia::start(c)) {
                break;
            }

            run.push(c);
        }
    }

    if run.iter().any(|c| decoration(c.body(&t.text))) {
        return None;
    }

    located(&t, &run)
}

/// Where the comments [`super::declaration`] reads for a field or member named at `at` start:
/// the lines above it, else its trailing comment.
pub fn declaration_comments(text: &str, at: usize) -> Vec<usize> {
    if !valid(text, at) {
        return Vec::new();
    }

    let t = trivia(text);

    if super::enclosing_header_in(&t, at).is_some() {
        return Vec::new();
    }

    if above_in(&t, at).is_some() {
        return t
            .docs_run(export_lines_above(&t.text, line_start(&t.text, at)))
            .into_iter()
            .map(Trivia::start)
            .collect();
    }

    if super::trailing_in(&t, at).is_some() {
        return t.comment_after(at).map(Trivia::start).into_iter().collect();
    }

    Vec::new()
}

/// The start of the `#program_export` lines directly above the line starting at `line`, else
/// `line`: an export line belongs to the declaration below it, so the docs sit above both.
pub(super) fn export_lines_above(text: &str, mut line: usize) -> usize {
    while line > 0 {
        let above = line_start(text, line - 1);

        if !text[above..line]
            .trim_start()
            .starts_with("#program_export")
        {
            break;
        }

        line = above;
    }

    line
}

pub(super) fn above_in(t: &Trivia, at: usize) -> Option<String> {
    let run = t.docs_run(export_lines_above(&t.text, line_start(&t.text, at)));
    let mut lines: Vec<&str> = run
        .into_iter()
        .flat_map(|c| t.bodies(c))
        .map(|b| b.text)
        .collect();

    while lines.first().is_some_and(|l| l.is_empty()) {
        lines.remove(0);
    }

    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }

    let doc = lines.join("\n");
    (!doc.trim().is_empty()).then_some(doc)
}
