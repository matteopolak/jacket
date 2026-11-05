//! Presentation-only interpreter for the checked-in coloring spec. No names are resolved here.
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Spec {
    schema: String,
    identifier: Identifier,
    keywords: Vec<String>,
    types: Vec<String>,
    rules: Vec<Rule>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Identifier {
    non_ascii: bool,
    excluded: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Rule {
    kind: String,
    role: Option<String>,
    open: Option<String>,
    close: Option<String>,
    escape: Option<String>,
    horizontal: Option<String>,
    line_bounded: Option<bool>,
    paint: Option<String>,
    semantic: Option<bool>,
    here_name: Option<String>,
    here_role: Option<String>,
    radix_prefixes: Option<Vec<String>>,
}

fn spec() -> &'static Spec {
    static SPEC: OnceLock<Spec> = OnceLock::new();
    SPEC.get_or_init(|| {
        let spec: Spec = serde_json::from_str(include_str!("coloring/spec.json"))
            .expect("checked coloring spec");
        assert_eq!(spec.schema, "jai-doc-coloring/1");
        spec
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Identifier,
    Other,
}

#[derive(Clone, Debug)]
pub(super) struct Lexical {
    pub start: usize,
    pub end: usize,
    pub role: Option<&'static str>,
    pub kind: Kind,
}

fn role(value: Option<&str>) -> Option<&'static str> {
    match value {
        Some("k") => Some("k"),
        Some("t") => Some("t"),
        Some("s") => Some("s"),
        Some("c") => Some("c"),
        Some("d") => Some("d"),
        Some("n") => Some("n"),
        None => None,
        _ => panic!("invalid checked color role"),
    }
}

fn byte(source: &[u8], at: usize) -> u8 {
    source.get(at).copied().unwrap_or(0)
}

fn ident(source: &str, at: usize, first: bool) -> bool {
    let Some(&c) = source.as_bytes().get(at) else {
        return false;
    };
    c.is_ascii_alphabetic()
        || c == b'_'
        || (!first && c.is_ascii_digit())
        || (c >= 128
            && spec().identifier.non_ascii
            && !spec()
                .identifier
                .excluded
                .iter()
                .any(|excluded| source.as_bytes()[at..].starts_with(excluded.as_bytes())))
}

fn word(source: &str, mut at: usize) -> usize {
    while ident(source, at, false) {
        at += 1;
    }
    at
}

fn blanks(source: &[u8], mut at: usize, characters: &str) -> usize {
    while at < source.len() && characters.as_bytes().contains(&source[at]) {
        at += 1;
    }
    at
}

fn quoted(source: &[u8], start: usize, quote: u8, escape: u8) -> usize {
    let mut at = start + 1;
    while at < source.len() {
        let c = source[at];
        at += 1;
        if c == quote {
            break;
        }
        if c == escape && at < source.len() {
            at += 1;
        }
    }
    at
}

fn note(source: &str, start: usize, line_bounded: bool) -> usize {
    let bytes = source.as_bytes();
    let mut at = start + 1;
    if byte(bytes, at) == b'"' {
        return quoted(bytes, at, b'"', b'\\');
    }
    let mut depth = 0usize;
    while at < bytes.len() {
        let c = bytes[at];
        if (line_bounded && c == b'\n') || (depth == 0 && c.is_ascii_whitespace()) {
            break;
        }
        if c == b'"' {
            at = quoted(bytes, at, b'"', b'\\');
            continue;
        }
        if c == b'(' {
            depth += 1;
        } else if c == b')' && depth > 0 {
            depth -= 1;
        }
        at += 1;
    }
    at
}

fn here(source: &str, after_name: usize, horizontal: &str) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut at = after_name;
    loop {
        at = blanks(bytes, at, horizontal);
        if byte(bytes, at) != b',' {
            break;
        }
        at = blanks(bytes, at + 1, horizontal);
        if byte(bytes, at) == b'\\' {
            at += 1;
            if at < bytes.len() {
                at += source[at..].chars().next().unwrap().len_utf8();
            }
        } else {
            let end = word(source, at);
            if end == at {
                return None;
            }
            at = end;
        }
    }
    let end = word(source, at);
    if end == at {
        return None;
    }
    let tag = &source[at..end];
    at = end;
    while at < bytes.len() && bytes[at] != b'\n' {
        at += 1;
    }
    at = (at + 1).min(bytes.len());
    while at < bytes.len() {
        let start = blanks(bytes, at, horizontal);
        if source.as_bytes()[start..].starts_with(tag.as_bytes())
            && !ident(source, start + tag.len(), false)
        {
            return Some(start + tag.len());
        }
        while at < bytes.len() && bytes[at] != b'\n' {
            at += 1;
        }
        at = (at + 1).min(bytes.len());
    }
    Some(bytes.len())
}

/// Reuse the same interpreter for note paint, with note recognition disabled inside it.
/// Notes are scanned at most twice; nesting uses counters, not recursive note interpretation.
pub(super) fn scan(source: &str) -> Vec<Lexical> {
    scan_region(source, false)
}

fn scan_region(source: &str, inside_note: bool) -> Vec<Lexical> {
    let bytes = source.as_bytes();
    let mut output = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let start = at;
        let mut matched = None;
        for rule in &spec().rules {
            let open = rule.open.as_deref().unwrap_or("");
            let prefix = bytes[at..].starts_with(open.as_bytes());
            let mut end = at;
            let mut color = role(rule.role.as_deref());
            let mut kind = Kind::Other;
            let mut note_paint = false;
            match rule.kind.as_str() {
                "line_comment" if prefix => {
                    end = at + open.len();
                    while end < bytes.len() && bytes[end] != b'\n' {
                        end += 1;
                    }
                }
                "nested_comment" if prefix => {
                    let close = rule.close.as_deref().unwrap();
                    end = at + open.len();
                    let mut depth = 1;
                    while end < bytes.len() && depth > 0 {
                        if bytes[end..].starts_with(open.as_bytes()) {
                            depth += 1;
                            end += open.len();
                        } else if bytes[end..].starts_with(close.as_bytes()) {
                            depth -= 1;
                            end += close.len();
                        } else {
                            end += 1;
                        }
                    }
                }
                "quoted" if prefix => {
                    end = quoted(
                        bytes,
                        at,
                        open.as_bytes()[0],
                        rule.escape.as_ref().unwrap().as_bytes()[0],
                    );
                }
                "note" if prefix && !inside_note => {
                    end = note(source, at, rule.line_bounded.unwrap());
                    note_paint = rule.paint.as_deref() == Some("lexical");
                }
                "directive" if prefix => {
                    let horizontal = rule.horizontal.as_deref().unwrap();
                    let name = blanks(bytes, at + open.len(), horizontal);
                    if ident(source, name, true) {
                        end = word(source, name);
                        if Some(&source[name..end]) == rule.here_name.as_deref()
                            && let Some(last) = here(source, end, horizontal)
                        {
                            end = last;
                            color = role(rule.here_role.as_deref());
                        }
                    }
                }
                "range" if prefix => {
                    end = at + open.len();
                }
                "number"
                    if bytes[at].is_ascii_digit()
                        || (bytes[at] == b'.' && byte(bytes, at + 1).is_ascii_digit()) =>
                {
                    end = at + 1;
                    let radix = rule
                        .radix_prefixes
                        .as_ref()
                        .unwrap()
                        .iter()
                        .any(|prefix| source[at..].starts_with(prefix));
                    while end < bytes.len() {
                        let c = bytes[end];
                        if c.is_ascii_alphanumeric()
                            || c == b'_'
                            || (c == b'.'
                                && byte(bytes, end + 1) != b'.'
                                && !ident(source, end + 1, true))
                            || (!radix
                                && matches!(c, b'+' | b'-')
                                && matches!(byte(bytes, end - 1), b'e' | b'E')
                                && byte(bytes, end + 1).is_ascii_digit())
                        {
                            end += 1;
                        } else {
                            break;
                        }
                    }
                }
                "identifier" if ident(source, at, true) => {
                    end = word(source, at);
                    kind = Kind::Identifier;
                    let name = &source[at..end];
                    color = if spec().keywords.iter().any(|word| word == name) {
                        Some("k")
                    } else if spec().types.iter().any(|word| word == name) {
                        Some("t")
                    } else {
                        None
                    };
                }
                "plain" => {
                    end = at + source[at..].chars().next().unwrap().len_utf8();
                }
                _ => {}
            }
            if end > at {
                matched = Some((end, color, kind, note_paint, rule.semantic.unwrap_or(true)));
                break;
            }
        }
        let (end, color, kind, note_paint, semantic) =
            matched.expect("checked spec has a consuming fallback");
        at = end;
        if note_paint {
            for mut token in scan_region(&source[start..end], true) {
                token.start += start;
                token.end += start;
                if !semantic {
                    token.kind = Kind::Other;
                }
                output.push(token);
            }
        } else {
            output.push(Lexical {
                start,
                end,
                role: color,
                kind,
            });
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::scan;

    #[test]
    fn partial_unicode_edits_always_partition_utf8_without_panicking() {
        for scalar in [
            '😀', '雪', '\u{00a0}', '\u{feff}', '"', '\\', '\n', ',', '@', '#',
        ] {
            for source in [
                format!("#string,\\{scalar} TAG\nbody\nTAG; return"),
                format!("/*{scalar}/*nested*/remaining*/"),
                format!("@note({scalar}\nreturn"),
                format!("\"\\{scalar}"),
            ] {
                let mut end = 0;
                for token in scan(&source) {
                    assert_eq!(token.start, end);
                    assert!(token.end > token.start && source.is_char_boundary(token.end));
                    end = token.end;
                }
                assert_eq!(end, source.len());
            }
        }
    }

    #[test]
    fn deep_comments_use_a_counter_without_recursive_stack_growth() {
        let source = format!("{}{}", "/*".repeat(8192), "*/".repeat(8192));
        let tokens = scan(&source);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].end, source.len());
        assert_eq!(tokens[0].role, Some("c"));
    }
}
