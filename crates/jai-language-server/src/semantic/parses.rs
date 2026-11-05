//! Remembering which texts failed to parse.
use jaic::source::FileId;

struct Parsed {
    key: (u64, usize),
    source: std::sync::Arc<str>,
    error: Option<usize>,
}

thread_local! {
    static PARSES: std::cell::RefCell<Vec<Parsed>> = std::cell::RefCell::new(Vec::with_capacity(PARSES_KEPT));
}

pub(super) const PARSES_KEPT: usize = 1024;
const PARSE_BYTES: usize = 16 << 20;

pub(crate) fn text_key(text: &str) -> (u64, usize) {
    use std::hash::{Hash, Hasher};
    let mut h = jaic::fxhash::FxHasher::default();
    text.hash(&mut h);
    (h.finish(), text.len())
}

/// Remember that `text` parses (`None`) or fails at a byte offset, for `parse_error`.
pub fn note_parse(text: &str, error: Option<usize>) {
    let cost = text.len() + 32;
    let storage = PARSES_KEPT * std::mem::size_of::<Parsed>();
    if storage + cost > PARSE_BYTES {
        return;
    }
    let key = text_key(text);
    PARSES.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.retain(|entry| entry.key != key || &*entry.source != text);
        let mut bytes = cache
            .iter()
            .map(|entry| entry.source.len() + 32)
            .sum::<usize>()
            + storage
            + cost;
        let mut removed = 0;
        while cache.len() - removed >= PARSES_KEPT || bytes > PARSE_BYTES {
            bytes -= cache[removed].source.len() + 32;
            removed += 1;
        }
        cache.drain(..removed);
        cache.push(Parsed {
            key,
            source: text.into(),
            error,
        });
    });
}

/// Where `text` fails to lex or parse. Hashes only select exact-source candidates.
pub fn parse_error(text: &str) -> Option<usize> {
    if let Some(found) = PARSES.with(|cache| {
        let cache = cache.borrow();
        cache
            .last()
            .filter(|entry| &*entry.source == text)
            .map(|entry| entry.error)
    }) {
        return found;
    }
    let key = text_key(text);
    if let Some(found) = PARSES.with(|cache| {
        cache
            .borrow()
            .iter()
            .rev()
            .find(|entry| entry.key == key && &*entry.source == text)
            .map(|entry| entry.error)
    }) {
        return found;
    }
    let error = jaic::parser::parse_file(FileId(0), text)
        .err()
        .map(|d| d.span.start as usize);
    note_parse(text, error);
    error
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_parse_hash_candidate_cannot_establish_source_identity() {
        let good = "main :: () {}";
        let bad = "main :: (  {}";
        assert_eq!(good.len(), bad.len());
        let error = jaic::parser::parse_file(FileId(0), bad)
            .unwrap_err()
            .span
            .start as usize;
        note_parse(bad, Some(error));
        let key = text_key(good);
        PARSES.with(|p| {
            let mut cache = p.borrow_mut();
            let entry = cache.last_mut().unwrap();
            entry.key = key;
        });
        assert_eq!(
            parse_error(good),
            None,
            "candidate hash is not the parsed source"
        );
    }
}
