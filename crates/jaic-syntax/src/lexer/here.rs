//! The header of a here-string (`#string,cr TAG`), read by the lexer and by tools that ask
//! which terminator a token has.
use super::scan::is_ident_char;
use std::ops::Range;

/// Terminators that name a language, matched case-insensitively. Editors highlight a
/// here-string ending in one of these as that language. The editor grammars are generated
/// from this list (`editors/lexicon.json`).
pub const LANGUAGE_TAGS: &[&str] = &[
    "WGSL",
    "GLSL",
    "VERT",
    "FRAG",
    "COMP",
    "GEOM",
    "TESC",
    "TESE",
    "HLSL",
    "MSL",
    "METAL",
    "SQL",
    "JSON",
    "HTML",
    "CSS",
    "JS",
    "JAVASCRIPT",
    "TS",
    "TYPESCRIPT",
    "PY",
    "PYTHON",
    "SH",
    "BASH",
    "SHELL",
    "C",
    "CPP",
    "OBJC",
    "RS",
    "RUST",
    "XML",
    "YAML",
    "YML",
    "TOML",
    "MD",
    "MARKDOWN",
    "LUA",
    "JAI",
];

/// What follows `#string` up to the end of the terminator word.
pub(super) struct Header {
    /// `,\%`: `\%` in the body becomes the format-string escape byte.
    pub escape_percent: bool,
    /// `,cr`: the body's line endings are `\r\n`.
    pub cr: bool,
    /// The terminator's byte range in the source; empty when there is none.
    pub tag: Range<usize>,
    /// Where the header ends: just after the terminator word.
    pub end: usize,
}

fn blanks(src: &[u8], mut at: usize) -> usize {
    while matches!(src.get(at), Some(b' ' | b'\t')) {
        at += 1;
    }
    at
}

fn word(src: &[u8], mut at: usize) -> usize {
    while src.get(at).copied().is_some_and(is_ident_char) {
        at += 1;
    }
    at
}

/// Reads the flags and the terminator from `at`, just after `#string`.
pub(super) fn header(src: &[u8], mut at: usize) -> Header {
    let (mut escape_percent, mut cr) = (false, false);
    loop {
        at = blanks(src, at);
        if src.get(at) != Some(&b',') {
            break;
        }
        at = blanks(src, at + 1);
        if src.get(at) == Some(&b'\\') {
            at += 1;
            escape_percent |= src.get(at) == Some(&b'%');
            // The escaped character, unless the file ends at the backslash.
            at = (at + 1).min(src.len());
        } else {
            let end = word(src, at);
            cr |= &src[at..end] == b"cr";
            at = end;
        }
    }
    let end = word(src, at);
    Header {
        escape_percent,
        cr,
        tag: at..end,
        end,
    }
}

/// The terminator of a here-string token's spelling (`#string,cr TAG\n...\nTAG`), or `None`
/// for any other token.
pub fn here_string_tag(spelling: &str) -> Option<&str> {
    let src = spelling.as_bytes();
    if src.first() != Some(&b'#') {
        return None;
    }
    let name = blanks(src, 1);
    let after = word(src, name);
    if &src[name..after] != b"string" {
        return None;
    }
    let tag = header(src, after).tag;
    (!tag.is_empty()).then(|| &spelling[tag])
}

/// Whether `tag` names a language an editor can highlight.
pub fn names_language(tag: &str) -> bool {
    LANGUAGE_TAGS.iter().any(|t| t.eq_ignore_ascii_case(tag))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_terminator_after_flags() {
        assert_eq!(here_string_tag("#string END\nx\nEND"), Some("END"));
        assert_eq!(
            here_string_tag("#string,cr WGSL\r\nx\r\nWGSL"),
            Some("WGSL")
        );
        assert_eq!(
            here_string_tag("# string , \\% , cr\tsql\nx\nsql"),
            Some("sql")
        );
        assert_eq!(here_string_tag("\"plain\""), None);
        assert_eq!(here_string_tag("#string"), None);
        assert_eq!(here_string_tag("#stringy X"), None);
    }

    #[test]
    fn known_tags_ignore_case_and_unknown_tags_do_not_count() {
        assert!(names_language("WGSL") && names_language("Json") && names_language("glsl"));
        assert!(!names_language("END") && !names_language("WGSL_SOURCE"));
    }
}
