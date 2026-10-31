//! The `@experimental` tag: a doc comment line that marks a module or an item.
use std::borrow::Cow;

/// The doc comment line that marks a module or an item as experimental.
pub const EXPERIMENTAL: &str = "@experimental";

fn tag_line(line: &str) -> bool {
    line.trim() == EXPERIMENTAL
}

/// `doc` without its `@experimental` lines, and whether it had one.
pub fn strip_experimental(doc: &str) -> (String, bool) {
    if !doc.lines().any(tag_line) {
        return (doc.to_owned(), false);
    }
    let kept: Vec<&str> = doc.lines().filter(|l| !tag_line(l)).collect();
    (kept.join("\n").trim().to_owned(), true)
}

/// `doc` with each `@experimental` line written as *Experimental*, for display.
pub fn noted(doc: &str) -> Cow<'_, str> {
    if !doc.lines().any(tag_line) {
        return Cow::Borrowed(doc);
    }
    let lines: Vec<&str> = doc
        .lines()
        .map(|l| {
            if tag_line(l) {
                "*Experimental*"
            } else {
                l
            }
        })
        .collect();
    Cow::Owned(lines.join("\n"))
}
