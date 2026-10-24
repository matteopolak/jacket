//! Where the bytes of a string token's value come from in its spelling.
//!
//! A quoted string with escapes (`"a\tb"`) or a here-string (which drops `\r`) has a value
//! whose byte offsets are not the spelling's. Tools that point into a string's text, such as
//! the holes of an interpolating call, map the value's offsets back with this.
use super::here;

/// For the string token spelled `spelling` (`"..."` or `#string ...`) with a value of `len`
/// bytes: for each value byte the offset in the spelling it comes from, then one more entry,
/// the offset just past the last byte consumed. `None` when the spelling does not produce
/// exactly `len` bytes.
pub fn string_value_map(spelling: &str, len: usize) -> Option<Vec<u32>> {
    let src = spelling.as_bytes();
    let mut map = Vec::with_capacity(len + 1);
    let end = match src.first()? {
        b'"' => quoted(src, len, &mut map)?,
        b'#' => here_string(src, len, &mut map)?,
        _ => return None,
    };
    (map.len() == len).then(|| {
        map.push(end as u32);
        map
    })
}

fn quoted(src: &[u8], len: usize, map: &mut Vec<u32>) -> Option<usize> {
    let mut i = 1;
    while map.len() < len {
        let c = *src.get(i)?;
        if c == b'"' {
            return None;
        }
        if c != b'\\' {
            map.push(i as u32);
            i += 1;
            continue;
        }
        let start = i as u32;
        let e = *src.get(i + 1)?;
        i += 2;
        let produced = match e {
            b'\n' => 0,
            b'x' => {
                i += 2;
                1
            }
            b'd' => {
                i += 3;
                1
            }
            b'u' | b'U' => {
                let digits = if e == b'u' {
                    4
                } else {
                    8
                };
                let hex = src.get(i..i + digits)?;
                i += digits;
                let value = u32::from_str_radix(std::str::from_utf8(hex).ok()?, 16).ok()?;
                char::from_u32(value)?.len_utf8()
            }
            _ => 1,
        };
        map.extend(std::iter::repeat_n(start, produced));
    }
    Some(i)
}

fn here_string(src: &[u8], len: usize, map: &mut Vec<u32>) -> Option<usize> {
    let mut at = 1;
    while matches!(src.get(at), Some(b' ' | b'\t')) {
        at += 1;
    }
    at += "string".len();
    let header = here::header(src, at);
    let mut i = header.end;
    while *src.get(i)? != b'\n' {
        i += 1;
    }
    i += 1;
    while map.len() < len {
        match *src.get(i)? {
            b'\r' => {}
            b'\n' => {
                map.push(i as u32);
                if header.cr {
                    map.push(i as u32);
                }
            }
            b'\\' if header.escape_percent && src.get(i + 1) == Some(&b'%') => {
                map.push(i as u32);
                i += 1;
            }
            _ => map.push(i as u32),
        }
        i += 1;
    }
    Some(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_quoted_string_maps_one_to_one() {
        assert_eq!(string_value_map("\"abc\"", 3), Some(vec![1, 2, 3, 4]));
    }

    #[test]
    fn escapes_map_to_their_backslash() {
        // value: a \t b \x41 c
        let map = string_value_map("\"a\\tb\\x41c\"", 5).unwrap();
        assert_eq!(map, vec![1, 2, 4, 5, 9, 10]);
        let map = string_value_map("\"\\u00e9x\"", 3).unwrap();
        assert_eq!(map, vec![1, 1, 7, 8]);
    }

    #[test]
    fn here_string_drops_carriage_returns() {
        let text = "#string END\r\nab\r\ncd\r\nEND";
        let map = string_value_map(text, 6).unwrap();
        let start = text.find("ab").unwrap() as u32;
        assert_eq!(map[0], start);
        assert_eq!(map[1], start + 1);
        assert_eq!(map[2], start + 3);
        assert_eq!(map[3], start + 4);
    }

    #[test]
    fn a_wrong_length_has_no_map() {
        assert_eq!(string_value_map("\"abc\"", 4), None);
    }
}
