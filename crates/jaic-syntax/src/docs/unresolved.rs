//! The links in a doc comment that read as intra-doc links and lead nowhere.
use super::{links, meant_as_link};
use std::path::Path;

/// The links in `doc` written like intra-doc links that `resolve` has no page for, as written
/// (`[Name]`). `known` are the names the document's library declares: a plain lowercase word
/// (`[keys]`) is prose unless it is a close misspelling of one, and so is a dotted path
/// (`[table.header]`) whose first part is no name at all.
pub fn unresolved(
    doc: &str,
    resolve: &dyn Fn(&[String]) -> Option<String>,
    known: &[&str],
) -> Vec<String> {
    let mut out = Vec::new();
    if !doc.contains('[') {
        return out;
    }
    for link in links(doc) {
        if link.defined || resolve(&link.names).is_some() {
            continue;
        }
        let before = &doc[doc[..link.whole.0].rfind('\n').map_or(0, |n| n + 1)..link.whole.0];
        let after = &doc[link.whole.1
            ..doc[link.whole.1..]
                .find('\n')
                .map_or(doc.len(), |n| link.whole.1 + n)];
        if !meant_as_link(&link, before, after) {
            continue;
        }
        let whole = &doc[link.whole.0..link.whole.1];
        let word = link.names[0].as_str();
        let lower = |w: &str| w.chars().all(|c| c.is_lowercase() || c.is_ascii_digit());
        if link.names.len() == 1 {
            let plain = !link.targeted && !link.code && lower(word);
            if plain
                && (word.chars().count() < 4
                    || crate::suggest::closest_unique(word, known.iter().copied()).is_none())
            {
                continue;
            }
        } else if !whole.contains('`')
            && !whole.ends_with(')')
            && lower(word)
            && !known.contains(&word)
        {
            continue;
        }
        if !out.iter().any(|o| o == whole) {
            out.push(whole.to_owned());
        }
    }
    out
}

/// The line of `file` holding `text` in a comment, searching from `from`; `from` when the
/// text is not found (a link broken over two lines).
pub fn located_line(file: &Path, from: usize, text: &str) -> usize {
    let from = from.max(1);
    let Ok(source) = std::fs::read_to_string(file) else {
        return from;
    };
    source
        .lines()
        .enumerate()
        .skip(from - 1)
        .find(|(_, line)| {
            line.contains(text)
                && (line.contains("//")
                    || line.contains("/*")
                    || line.trim_start().starts_with('*'))
        })
        .map_or(from, |(n, _)| n + 1)
}
