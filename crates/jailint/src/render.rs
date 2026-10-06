//! Diagnostics as text, in the style of rustc and clippy:
//!
//! ```text
//! warning[index_only_loop]: `i` is only used to index `xs`
//!  --> sum.jai:4:5
//!   |
//! 4 |     for i: 0..xs.count-1 {
//!   |     ^^^^^^^^^^^^^^^^^^^^ loop over `xs` itself
//!   |
//! help: iterate over the elements of `xs` and use `it`
//!   |
//! 4 ~     for xs {
//! 5 ~         total += it;
//!   |
//!   = note: `index_only_loop` is `warn` by default
//! ```
use crate::{Level, Lint};

/// Longest stretch of changed code shown under a help.
const MAX_HELP_LINES: usize = 8;

pub struct Style {
    pub color: bool,
}

impl Style {
    fn paint(&self, code: &str, text: &str) -> String {
        if self.color {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }
}

/// One-based line and column (in characters) of a byte offset.
pub fn line_col(text: &str, offset: usize) -> (usize, usize) {
    let offset = text.floor_char_boundary(offset.min(text.len()));
    let line_start = text[..offset].rfind('\n').map_or(0, |n| n + 1);
    let line = text[..offset].matches('\n').count() + 1;
    (line, text[line_start..offset].chars().count() + 1)
}

/// `lint` in `text` (the file's contents), its path shown as `path`.
pub fn render(lint: &Lint, text: &str, path: &str, default: Level, style: &Style) -> String {
    let (head, color) = match lint.level {
        Level::Deny => ("error", "1;31"),
        _ => ("warning", "1;33"),
    };
    let blue = "1;34";
    let (line, col) = line_col(text, lint.start);
    let (end_line, _) = line_col(text, lint.end.max(lint.start));
    let width = end_line.max(line + 8).to_string().len();
    let gutter = " ".repeat(width);
    let bar = style.paint(blue, "|");
    let mut out = String::new();
    out.push_str(&format!(
        "{}{}\n",
        style.paint(color, &format!("{head}[{}]", lint.rule)),
        style.paint("1", &format!(": {}", lint.message))
    ));
    out.push_str(&format!(
        "{gutter}{} {path}:{line}:{col}\n",
        style.paint(blue, "-->")
    ));
    out.push_str(&format!("{gutter} {bar}\n"));
    // The first line of the range, with carets under it.
    let line_start = text[..lint.start.min(text.len())]
        .rfind('\n')
        .map_or(0, |n| n + 1);
    let line_end = text[line_start..]
        .find('\n')
        .map_or(text.len(), |n| line_start + n);
    let source = &text[line_start..line_end];
    out.push_str(&format!(
        "{} {bar} {}\n",
        style.paint(blue, &format!("{line:>width$}")),
        source.trim_end()
    ));
    let caret_start = text[line_start..lint.start.min(line_end)].chars().count();
    let caret_len = text[lint.start.min(line_end)..lint.end.clamp(lint.start, line_end)]
        .chars()
        .count()
        .max(1);
    let pad: String = source
        .chars()
        .take(caret_start)
        .map(|c| {
            if c == '\t' {
                '\t'
            } else {
                ' '
            }
        })
        .collect();
    let mut carets = style.paint(color, &"^".repeat(caret_len));
    if let Some(label) = &lint.label {
        carets.push(' ');
        carets.push_str(&style.paint(color, label));
    }
    out.push_str(&format!("{gutter} {bar} {pad}{carets}\n"));
    out.push_str(&format!("{gutter} {bar}\n"));
    match (&lint.help, &lint.fix) {
        (Some(help), Some(fix)) if !fix.edits.is_empty() => {
            out.push_str(&format!("{}: {help}\n", style.paint("1;36", "help")));
            out.push_str(&format!("{gutter} {bar}\n"));
            for (n, mark, code) in changed_lines(text, fix) {
                let paint = match mark {
                    '-' => "31",
                    '+' => "32",
                    _ => blue,
                };
                out.push_str(&format!(
                    "{} {} {code}\n",
                    style.paint(blue, &format!("{n:>width$}")),
                    style.paint(paint, &mark.to_string())
                ));
            }
            out.push_str(&format!("{gutter} {bar}\n"));
        }
        (Some(help), _) => {
            out.push_str(&format!(
                "{gutter} {} {}: {help}\n",
                style.paint(blue, "="),
                style.paint("1", "help")
            ));
        }
        _ => {}
    }
    if lint.level == default {
        out.push_str(&format!(
            "{gutter} {} {}: `{}` is `{}` by default\n",
            style.paint(blue, "="),
            style.paint("1", "note"),
            lint.rule,
            default.as_str()
        ));
    }
    out
}

/// The code a fix changes, as (line number, mark, text). When the fix keeps the number of
/// lines, the changed lines as fixed (`~`); otherwise the old lines (`-`) then the new (`+`).
/// At most `MAX_HELP_LINES` of each are shown.
fn changed_lines(text: &str, fix: &crate::Fix) -> Vec<(String, char, String)> {
    let mut edits = fix.edits.clone();
    edits.sort_by_key(|e| e.start);
    let first = edits.first().map_or(0, |e| e.start);
    let last = edits.iter().map(|e| e.end).max().unwrap_or(0);
    let fixed = crate::fix::apply_edits(text, edits.clone());
    // Where the last edit ends in the fixed text.
    let mut shift: isize = 0;
    let mut last_new = 0usize;
    for e in &edits {
        let start = (e.start as isize + shift) as usize;
        last_new = last_new.max(start + e.text.len());
        shift += e.text.len() as isize - (e.end - e.start) as isize;
    }
    let (first_line, _) = line_col(text, first);
    // An edit ending at a line's start (a removed whole line) ends on the line before.
    let (old_last, _) = line_col(text, last.saturating_sub(1).max(first));
    let (new_last, _) = line_col(&fixed, last_new.saturating_sub(1).max(first));
    let old: Vec<&str> = text.lines().collect();
    let new: Vec<&str> = fixed.lines().collect();
    let take = |lines: &[&str], from: usize, to: usize, mark: char, out: &mut Vec<_>| {
        for n in from..=to.max(from) {
            if n - from == MAX_HELP_LINES {
                out.push(("...".to_string(), mark, String::new()));
                break;
            }
            if let Some(l) = lines.get(n - 1) {
                out.push((n.to_string(), mark, l.trim_end().to_string()));
            }
        }
    };
    let mut out = Vec::new();
    let removed = |e: &crate::Edit| text[e.start..e.end].matches('\n').count();
    let added = |e: &crate::Edit| e.text.matches('\n').count();
    if edits.iter().all(|e| removed(e) == added(e)) {
        take(&new, first_line, new_last, '~', &mut out);
    } else if edits.iter().all(|e| {
        let line_start = e.start == 0 || text[..e.start].ends_with('\n');
        e.text.is_empty() && e.end > e.start && line_start && text[..e.end].ends_with('\n')
    }) {
        // Whole lines removed: show just those.
        for e in &edits {
            let (a, _) = line_col(text, e.start);
            let (b, _) = line_col(text, e.end - 1);
            take(&old, a, b, '-', &mut out);
        }
    } else {
        take(&old, first_line, old_last, '-', &mut out);
        take(&new, first_line, new_last, '+', &mut out);
    }
    out
}
