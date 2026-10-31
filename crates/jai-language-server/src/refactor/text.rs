//! Text and extent helpers for building refactoring edits.
use jaic::ast::Stmt;
use jaic::source::line_start;

pub(crate) fn trim(text: &str, start: usize, end: usize) -> (usize, usize) {
    let end = end.min(text.len());
    let start = start.min(end);
    let slice = &text[start..end];
    let lead = slice.len() - slice.trim_start().len();
    let trail = slice.trim_start().len() - slice.trim().len();
    (start + lead, end - trail)
}


/// The leading whitespace of the line holding `at`.
pub(super) fn indent_at(text: &str, at: usize) -> &str {
    let start = line_start(text, at);
    let line = &text[start..];
    let width = line
        .find(|c: char| c != ' ' && c != '\t')
        .unwrap_or(line.len());
    &line[..width]
}

/// Nothing but whitespace lies between the start of the line and `at`.
pub(super) fn starts_line(text: &str, at: usize) -> bool {
    text[line_start(text, at)..at].trim().is_empty()
}

/// The indentation one level deeper is made of: the document's smallest indent step.
pub(super) fn indent_unit(text: &str) -> String {
    let mut step: Option<usize> = None;
    for line in text.lines() {
        if line.starts_with('\t') {
            return "\t".into();
        }
        let width = line.len() - line.trim_start_matches(' ').len();
        if width > 0 && width < line.len() {
            step = Some(step.map_or(width, |s| s.min(width)));
        }
    }
    " ".repeat(step.filter(|s| (2..=8).contains(s)).unwrap_or(4))
}

/// Does `text` use `name` as a whole word?
fn mentions(text: &str, name: &str) -> bool {
    text.match_indices(name).any(|(at, _)| {
        let word = |c: char| c.is_alphanumeric() || c == '_';
        !text[..at].chars().next_back().is_some_and(word)
            && !text[at + name.len()..].chars().next().is_some_and(word)
    })
}

/// `base`, or `base2`, `base3` ... when the document already uses the name.
pub(super) fn fresh(text: &str, base: &str) -> String {
    if !mentions(text, base) {
        return base.into();
    }
    (2..u32::MAX)
        .map(|n| format!("{base}{n}"))
        .find(|c| !mentions(text, c))
        .unwrap_or_else(|| base.into())
}

/// A statement's extent, through its `;`.
pub(crate) fn stmt_extent(text: &str, s: &Stmt) -> (usize, usize) {
    let (a, mut b) = (s.span.start as usize, s.span.end as usize);
    if b <= text.len() && a <= b {
        let rest = text[b..].trim_start_matches([' ', '\t']);
        if rest.starts_with(';') {
            b = text.len() - rest.len() + 1;
        }
    }
    (a, b)
}
