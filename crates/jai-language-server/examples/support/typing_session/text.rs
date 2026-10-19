//! Text helpers: positions, identifiers and bracket balance.

use super::Position;

pub(super) fn pos_of(text: &str, byte: usize) -> Position {
    let before = &text[..byte];
    Position {
        line: before.bytes().filter(|&b| b == b'\n').count() as u32,
        character: before
            .rsplit('\n')
            .next()
            .map_or(0, jaic::source::utf16_len),
    }
}

/// The byte offset of an in-range position, or `None`.
pub(super) fn byte_of(text: &str, position: Position) -> Option<usize> {
    let mut start = 0;
    for _ in 0..position.line {
        start += text[start..].find('\n')? + 1;
    }
    let line = &text[start..];
    let line = &line[..line.find('\n').unwrap_or(line.len())];
    let mut units = 0;
    for (byte, ch) in line.char_indices() {
        if units == position.character {
            return Some(start + byte);
        }
        units += ch.len_utf16() as u32;
    }
    (units == position.character).then_some(start + line.len())
}

/// Offsets of identifier starts (not inside a longer word or number).
pub(super) fn identifier_starts(text: &str) -> Vec<usize> {
    let word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let bytes = text.as_bytes();
    (0..bytes.len())
        .filter(|&i| {
            (bytes[i].is_ascii_alphabetic() || bytes[i] == b'_') && (i == 0 || !word(bytes[i - 1]))
        })
        .collect()
}

/// Whether brackets, strings and block comments in `region` all close inside it. A string that
/// runs to the end of a line counts as unclosed, and so does a here-string.
pub(super) fn balanced(region: &str) -> bool {
    if region.contains("#string") {
        return false;
    }
    let bytes = region.as_bytes();
    let (mut depth, mut comment, mut i) = (0i32, 0usize, 0);
    while i < bytes.len() {
        let two = bytes.get(i..i + 2).unwrap_or_default();
        match bytes[i] {
            _ if comment > 0 && two == b"*/" => {
                comment -= 1;
                i += 1;
            }
            _ if comment > 0 && two == b"/*" => {
                comment += 1;
                i += 1;
            }
            _ if comment > 0 => {}
            b'/' if two == b"/*" => {
                comment += 1;
                i += 1;
            }
            b'/' if two == b"//" => {
                i += region[i..].find('\n').unwrap_or(bytes.len() - i);
                continue;
            }
            b'"' => match string_end(bytes, i) {
                Some(end) => i = end,
                None => return false,
            },
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            _ => {}
        }
        if depth < 0 {
            return false;
        }
        i += 1;
    }
    depth == 0 && comment == 0
}

/// The offset of the quote closing the string opened at `open`, if it closes on the same line.
pub(super) fn string_end(bytes: &[u8], open: usize) -> Option<usize> {
    let mut i = open + 1;
    while let Some(&b) = bytes.get(i) {
        match b {
            b'\\' => i += 1,
            b'"' => return Some(i),
            b'\n' => return None,
            _ => {}
        }
        i += 1;
    }
    None
}
