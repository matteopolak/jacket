//! Text and extent helpers for building refactoring edits.
use super::Sp;
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

/// Parentheses and brackets of `text` outside string literals: the count of `(` minus `)`.
fn paren_balance(text: &str) -> i32 {
    let mut balance = 0;
    let mut in_string = false;
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' if in_string => {
                chars.next();
            }
            '"' => in_string = !in_string,
            '(' if !in_string => balance += 1,
            ')' if !in_string => balance -= 1,
            _ => {}
        }
    }
    balance
}

/// The source extent of an expression. The parser's span can leave out parentheses around the
/// expression's first or last operand, so they are taken back in.
pub(crate) fn extent(text: &str, span: Sp) -> (usize, usize) {
    let (mut a, mut b) = (span.start as usize, span.end as usize);
    if a > b || b > text.len() || !text.is_char_boundary(a) || !text.is_char_boundary(b) {
        return (a, b);
    }
    let balance = paren_balance(&text[a..b]);
    for _ in 0..balance.unsigned_abs() {
        if balance < 0 {
            let before = text[..a].trim_end();
            if before.ends_with('(') {
                a = before.len() - 1;
            }
        } else {
            let after = text[b..].trim_start();
            if after.starts_with(')') {
                b = text.len() - after.len() + 1;
            }
        }
    }
    (a, b)
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

/// `selection` without one pair of outer parentheses that enclose all of it.
pub(super) fn unparenthesized(text: &str, (a, b): (usize, usize)) -> Option<(usize, usize)> {
    let slice = text.get(a..b)?;
    if slice.starts_with('(')
        && slice.ends_with(')')
        && paren_balance(&slice[1..slice.len() - 1]) == 0
    {
        // `(a) + (b)` has balanced insides too; the first `(` must close at the very end.
        let mut depth = 0;
        for (i, c) in slice.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 && i + 1 != slice.len() {
                        return None;
                    }
                }
                _ => {}
            }
        }
        return Some(trim(text, a + 1, b - 1));
    }
    None
}
