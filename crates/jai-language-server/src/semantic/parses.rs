//! Remembering which texts failed to parse.
use jaic::source::FileId;

thread_local! {
    /// Where the texts parsed lately fail (by hash and length): hover, references, diagnostics
    /// and lints each ask about the same open document, and a parse of a large file is a
    /// transient tree of tens of MiB each time.
    static PARSES: std::cell::RefCell<Vec<(u64, usize, Option<usize>)>> = const { std::cell::RefCell::new(Vec::new()) };
    /// The text asked about last and its answer: requests on one document ask about the same
    /// text again and again, and comparing bytes is several times faster than hashing them.
    static LAST_PARSE: std::cell::RefCell<(String, Option<usize>)> = const { std::cell::RefCell::new((String::new(), None)) };
}

/// Parse results remembered: one per open document and a few for the texts the repair of a
/// broken document tries. Each is 24 bytes; with fewer than there were open documents, every
/// new compile of a large project parsed all its files again to see that they parse.
pub(super) const PARSES_KEPT: usize = 1024;

pub(crate) fn text_key(text: &str) -> (u64, usize) {
    use std::hash::{Hash, Hasher};
    let mut h = crate::fast_hash::FastHasher::default();
    text.hash(&mut h);
    (h.finish(), text.len())
}

/// Remember that `text` parses (`None`) or fails at a byte offset, for `parse_error`.
pub fn note_parse(text: &str, error: Option<usize>) {
    let (hash, len) = text_key(text);
    PARSES.with(|p| {
        let mut p = p.borrow_mut();
        p.retain(|e| (e.0, e.1) != (hash, len));
        if p.len() >= PARSES_KEPT {
            p.remove(0);
        }
        p.push((hash, len, error));
    });
}

/// Where `text` fails to lex or parse (a byte offset), if it does.
pub fn parse_error(text: &str) -> Option<usize> {
    if let Some(found) = LAST_PARSE.with(|last| {
        let last = last.borrow();
        (last.0 == text).then_some(last.1)
    }) {
        return found;
    }
    let (hash, len) = text_key(text);
    let error = if let Some(found) = PARSES.with(|p| {
        p.borrow()
            .iter()
            .find(|e| (e.0, e.1) == (hash, len))
            .map(|e| e.2)
    }) {
        found
    } else {
        let error = jaic::parser::parse_file(FileId(0), text)
            .err()
            .map(|d| d.span.start as usize);
        note_parse(text, error);
        error
    };
    LAST_PARSE.with(|last| {
        let mut last = last.borrow_mut();
        last.0.clear();
        last.0.push_str(text);
        last.1 = error;
    });
    error
}
