//! Doc comments with the line they start on.
use super::*;

/// A doc comment and the line of the file its first line is on (1-based). Each line of `text`
/// is the next line of the file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Located {
    pub line: usize,
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
    located(&t, &t.docs_run(line_start(&t.text, at)))
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
