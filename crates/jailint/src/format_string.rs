//! `%` directives of a print-style format string, as the bundled `Basic` module reads them:
//! `%` (or `%0`) takes the next argument, `%N` takes argument N (1-based) and a following `%`
//! continues after it, `%00` prints nothing, and `%%` is two arguments in a row. A literal
//! percent sign is written `\%`.
//!
//! Used by the language server's format-string features (hover and diagnostics that work
//! without type checking); `jaic` makes the same count check at compile time.

/// The largest argument index a directive reports. A string can ask for argument
/// 77777777777777777777777777, which no call passes; clamping keeps `index + 1` and the counts
/// built from it from overflowing in every consumer.
pub const MAX_INDEX: usize = i32::MAX as usize;

/// One directive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spec {
    /// Byte range in the text that was scanned.
    pub start: usize,
    pub end: usize,
    /// The argument it formats (0-based among the arguments after the format string), or
    /// `None` for `%00`. At most `MAX_INDEX`.
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
                    n.saturating_sub(1).min(MAX_INDEX)
                } else {
                    if at < end && bytes[at] == b'0' {
                        at += 1;
                    }
                    next
                };
                next = (index + 1).min(MAX_INDEX);
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

    #[test]
    fn huge_argument_numbers_are_clamped() {
        let huge = indices("%77777777777777777777777777 % %");
        assert_eq!(huge, [Some(MAX_INDEX); 3]);
        let text = "\"%77777777777777777777777777 %\"";
        assert_eq!(required(&specs(text, 0, text.len())), MAX_INDEX + 1);
    }
}
