//! `%` directives of a print-style format string, as the bundled `Basic` module reads them:
//! `%` (or `%0`) takes the next argument, `%N` takes argument N (1-based) and a following `%`
//! continues after it, `%00` prints nothing, and `%%` is two arguments in a row. A literal
//! percent sign is written `\%`.
//!
//! Shared by the `format_arg_count` rule and the language server's format-string features.

/// One directive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spec {
    /// Byte range in the text that was scanned.
    pub start: usize,
    pub end: usize,
    /// The argument it formats (0-based among the arguments after the format string), or
    /// `None` for `%00`.
    pub index: Option<usize>,
}

/// The directives of the string literal at `start..end` of `text` (quotes included).
pub fn specs(text: &str, start: usize, end: usize) -> Vec<Spec> {
    let bytes = text.as_bytes();
    let end = end.saturating_sub(1).min(bytes.len());
    let mut at = start + 1;
    let mut next = 0usize;
    let mut out = Vec::new();
    while at < end {
        match bytes[at] {
            b'\\' => at += 2,
            b'%' => {
                let from = at;
                at += 1;
                if at + 1 < end && bytes[at] == b'0' && bytes[at + 1] == b'0' {
                    at += 2;
                    out.push(Spec {
                        start: from,
                        end: at,
                        index: None,
                    });
                    continue;
                }
                let index = if at < end && bytes[at].is_ascii_digit() && bytes[at] != b'0' {
                    let mut n = 0usize;
                    while at < end && bytes[at].is_ascii_digit() {
                        n = n
                            .saturating_mul(10)
                            .saturating_add((bytes[at] - b'0') as usize);
                        at += 1;
                    }
                    n.saturating_sub(1)
                } else {
                    if at < end && bytes[at] == b'0' {
                        at += 1;
                    }
                    next
                };
                next = index.saturating_add(1);
                out.push(Spec {
                    start: from,
                    end: at,
                    index: Some(index),
                });
            }
            _ => at += 1,
        }
    }
    out
}

/// Arguments the directives use: one past the highest index.
pub fn required(specs: &[Spec]) -> usize {
    specs
        .iter()
        .filter_map(|s| s.index)
        .map(|i| i + 1)
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn indices(source: &str) -> Vec<Option<usize>> {
        let text = format!("\"{source}\"");
        specs(&text, 0, text.len())
            .into_iter()
            .map(|s| s.index)
            .collect()
    }

    #[test]
    fn directives_follow_basic_print() {
        assert_eq!(indices("a % b %"), [Some(0), Some(1)]);
        assert_eq!(indices("%2 %1 %"), [Some(1), Some(0), Some(1)]);
        assert_eq!(indices("%%"), [Some(0), Some(1)]);
        assert_eq!(indices("100\\%"), []);
        assert_eq!(indices("%00x%0"), [None, Some(0)]);
    }
}
