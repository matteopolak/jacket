//! Inline holes in the string literal of an `@Interpolates("mode")` call.
//!
//! A procedure marked `@Interpolates("sql")` (or `"plain"`) takes `%name`, `%a.b`, `%a[i]` and
//! `%(expression)` inside a literal and the compiler rewrites the call to its positional form
//! before type checking: `f("x = %a", ..)` becomes `f("x = %", .., a)`. This module is the
//! scanner: it finds the holes of a string value and nothing more. Mapping the holes back to
//! source text and checking them is the compiler's job (`sema::interpolate`), and editors use the
//! same scanner to know where a hole is.
use std::ops::Range;

/// What the scanner leaves alone, named by the note's argument.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Nothing is skipped: every `%` that starts a hole is one.
    Plain,
    /// SQL quoting and comments are skipped: `'...'` (with `''`), `E'...'` (backslash escapes),
    /// `$tag$...$tag$`, `"..."`, `--` to the end of the line and nested `/* */`.
    Sql,
}

impl Mode {
    pub fn from_name(name: &str) -> Option<Mode> {
        match name {
            "plain" => Some(Mode::Plain),
            "sql" => Some(Mode::Sql),
            _ => None,
        }
    }
}

/// The mode of the note `@Interpolates("mode")`, given the note's text (without the `@`).
/// `None` when the note is another one or names no known mode.
pub fn note_mode(note: &str) -> Option<Mode> {
    let rest = note.trim().strip_prefix(NOTE)?;
    let name = rest.strip_prefix("(\"")?.strip_suffix("\")")?;
    Mode::from_name(name)
}

/// The note's name.
pub const NOTE: &str = "Interpolates";

/// One hole in a string value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hole {
    /// The whole `%...` in the value, which the rewritten string replaces with a single `%`.
    pub whole: Range<usize>,
    /// The expression's text in the value: the name and its `.field`/`[index]` chain, or what
    /// is inside the parentheses of `%(...)`.
    pub expr: Range<usize>,
}

fn ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_' || c >= 0x80
}

fn ident_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c >= 0x80
}

/// The index just past the bracket that closes the one at `open` (`(` or `[`), counting nested
/// brackets and stepping over string literals; `None` when it is never closed.
fn closing(v: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut i = open;
    while i < v.len() {
        match v[i] {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            b'"' => {
                i += 1;
                while i < v.len() && v[i] != b'"' {
                    if v[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// The hole starting at the `%` at `at`, if there is one: its extent in `v`.
fn hole_at(v: &[u8], at: usize) -> Option<Hole> {
    let next = *v.get(at + 1)?;
    if next == b'(' {
        let end = closing(v, at + 1)?;
        return Some(Hole {
            whole: at..end,
            expr: at + 2..end - 1,
        });
    }
    if !ident_start(next) {
        return None;
    }
    let mut end = at + 1;
    while v.get(end).is_some_and(|&c| ident_char(c)) {
        end += 1;
    }
    loop {
        match v.get(end) {
            Some(b'.') if v.get(end + 1).is_some_and(|&c| ident_start(c)) => {
                end += 1;
                while v.get(end).is_some_and(|&c| ident_char(c)) {
                    end += 1;
                }
            }
            Some(b'[') => match closing(v, end) {
                Some(close) => end = close,
                None => break,
            },
            _ => break,
        }
    }
    Some(Hole {
        whole: at..end,
        expr: at + 1..end,
    })
}

/// The index past the quoted text, comment or dollar quote of SQL starting at `at`, or `None`
/// when `v[at]` starts none.
fn sql_skip(v: &[u8], at: usize) -> Option<usize> {
    let rest = &v[at..];
    let after_word = at == 0 || !ident_char(v[at - 1]);
    match rest[0] {
        b'-' if rest.get(1) == Some(&b'-') => Some(
            rest.iter()
                .position(|&c| c == b'\n')
                .map_or(v.len(), |n| at + n + 1),
        ),
        b'/' if rest.get(1) == Some(&b'*') => {
            let (mut depth, mut i) = (0usize, at);
            while i < v.len() {
                if v[i..].starts_with(b"/*") {
                    depth += 1;
                    i += 2;
                } else if v[i..].starts_with(b"*/") {
                    depth -= 1;
                    i += 2;
                    if depth == 0 {
                        return Some(i);
                    }
                } else {
                    i += 1;
                }
            }
            Some(v.len())
        }
        b'\'' => Some(quoted(
            v,
            at,
            b'\'',
            at > 0 && matches!(v[at - 1], b'e' | b'E') && { at < 2 || !ident_char(v[at - 2]) },
        )),
        b'"' => Some(quoted(v, at, b'"', false)),
        b'$' if after_word => {
            let tag_end = rest[1..]
                .iter()
                .position(|&c| !ident_char(c))
                .map_or(rest.len(), |n| n + 1);
            if rest.get(tag_end) != Some(&b'$') || rest.get(1).is_some_and(u8::is_ascii_digit) {
                return None;
            }
            let tag = &rest[..=tag_end];
            let close = rest[tag.len()..]
                .windows(tag.len())
                .position(|w| w == tag)
                .map_or(v.len(), |n| at + tag.len() + n + tag.len());
            Some(close)
        }
        _ => None,
    }
}

/// The index past the `quote`d text opening at `at`, where the quote doubled stands for itself
/// and, with `backslash`, a backslash escapes the next byte.
fn quoted(v: &[u8], at: usize, quote: u8, backslash: bool) -> usize {
    let mut i = at + 1;
    while i < v.len() {
        match v[i] {
            b'\\' if backslash => i += 1,
            c if c == quote => {
                if v.get(i + 1) == Some(&quote) {
                    i += 1;
                } else {
                    return i + 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    v.len()
}

/// The holes of the string value `v`, in order.
pub fn holes(v: &[u8], mode: Mode) -> Vec<Hole> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < v.len() {
        if mode == Mode::Sql
            && let Some(end) = sql_skip(v, i)
        {
            i = end.max(i + 1);
            continue;
        }
        if v[i] == b'%' {
            if v.get(i + 1) == Some(&b'%') {
                i += 2;
                continue;
            }
            if let Some(hole) = hole_at(v, i) {
                i = hole.whole.end;
                out.push(hole);
                continue;
            }
        }
        i += 1;
    }
    out
}

/// `v` with every hole replaced by a plain `%`.
pub fn rewrite(v: &[u8], holes: &[Hole]) -> Vec<u8> {
    let mut out = Vec::with_capacity(v.len());
    let mut at = 0;
    for hole in holes {
        out.extend_from_slice(&v[at..hole.whole.start]);
        out.push(b'%');
        at = hole.whole.end;
    }
    out.extend_from_slice(&v[at..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(text: &str, mode: Mode) -> Vec<String> {
        holes(text.as_bytes(), mode)
            .iter()
            .map(|h| text[h.expr.clone()].to_string())
            .collect()
    }

    #[test]
    fn note_modes() {
        assert_eq!(note_mode("Interpolates(\"sql\")"), Some(Mode::Sql));
        assert_eq!(note_mode("Interpolates(\"plain\")"), Some(Mode::Plain));
        assert_eq!(note_mode("Interpolates(\"other\")"), None);
        assert_eq!(note_mode("PrintLike"), None);
    }

    #[test]
    fn hole_forms() {
        let all = found(
            "a %x b %u.name.first c %arr[i + 1] d %(a + b) e",
            Mode::Plain,
        );
        assert_eq!(all, ["x", "u.name.first", "arr[i + 1]", "a + b"]);
    }

    #[test]
    fn plain_percent_signs_stay() {
        assert!(found("a % b, 100%, %%x, %1, % ", Mode::Plain).is_empty());
        assert_eq!(found("end of %id.", Mode::Plain), ["id"]);
    }

    #[test]
    fn rewriting_replaces_each_hole_with_a_percent() {
        let text = "x = %a.b and y = %(c) %%";
        let h = holes(text.as_bytes(), Mode::Plain);
        assert_eq!(rewrite(text.as_bytes(), &h), b"x = % and y = % %%");
    }

    #[test]
    fn sql_skips_quotes_and_comments() {
        let q = "select '%a', E'\\'%b', \"%c\", $t$ %d $t$, $$%e$$ -- %f\n/* %g /* %h */ %i */ %j";
        assert_eq!(found(q, Mode::Sql), ["j"]);
        assert_eq!(found("'it''s %a' %b", Mode::Sql), ["b"]);
        assert_eq!(found("a $1 %x", Mode::Sql), ["x"]);
        assert_eq!(found("'%a'", Mode::Plain), ["a"]);
    }

    #[test]
    fn brackets_in_holes_must_close() {
        assert_eq!(found("%a[1 %b", Mode::Plain), ["a", "b"]);
        assert_eq!(found("%(a[\")\"]) x", Mode::Plain), ["a[\")\"]"]);
        assert!(found("%(a", Mode::Plain).is_empty());
    }
}
