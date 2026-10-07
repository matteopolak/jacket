//! Here-strings whose terminator names a language (`#string WGSL`). Editors highlight such a
//! body as that language, so semantic tokens leave the whole here-string to them instead of
//! painting it as one string. See `docs/tools/vscode-extension.md` (*Embedded languages*).

/// Terminators that name a language, matched case-insensitively. Keep in step with
/// `EMBEDDED_LANGUAGES` in `editors/vscode/scripts/build-grammar.mjs`, which fails when the two
/// lists differ.
pub(crate) const LANGUAGE_TAGS: &[&str] = &[
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

fn is_ident_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c >= 0x80
}

/// The terminator of a here-string token's spelling (`#string,cr TAG\n...\nTAG`), read the way
/// the lexer reads the header: `#`, optional blanks, `string`, `,flag` or `,\x` modifiers, the
/// tag.
pub(crate) fn terminator(spelling: &str) -> Option<&str> {
    let bytes = spelling.as_bytes();
    let blanks = |mut at: usize| {
        while matches!(bytes.get(at), Some(b' ' | b'\t')) {
            at += 1;
        }
        at
    };
    let ident = |mut at: usize| {
        while bytes.get(at).is_some_and(|&c| is_ident_char(c)) {
            at += 1;
        }
        at
    };
    if bytes.first() != Some(&b'#') {
        return None;
    }
    let mut at = blanks(1);
    if !spelling[at..].starts_with("string") {
        return None;
    }
    at += "string".len();
    loop {
        at = blanks(at);
        if bytes.get(at) != Some(&b',') {
            break;
        }
        at = blanks(at + 1);
        if bytes.get(at) == Some(&b'\\') {
            // The escaped character is one byte in every flag the lexer knows.
            at = (at + 2).min(bytes.len());
        } else {
            at = ident(at);
        }
    }
    let end = ident(at);
    (end > at).then(|| &spelling[at..end])
}

/// Whether a here-string token's spelling has a terminator that names a language.
pub(crate) fn names_language(spelling: &str) -> bool {
    terminator(spelling)
        .is_some_and(|tag| LANGUAGE_TAGS.iter().any(|t| t.eq_ignore_ascii_case(tag)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_terminator_after_flags() {
        assert_eq!(terminator("#string END\nx\nEND"), Some("END"));
        assert_eq!(terminator("#string,cr WGSL\r\nx\r\nWGSL"), Some("WGSL"));
        assert_eq!(terminator("# string , \\% , cr\tsql\nx\nsql"), Some("sql"));
        assert_eq!(terminator("\"plain\""), None);
        assert_eq!(terminator("#string"), None);
    }

    #[test]
    fn known_tags_ignore_case_and_unknown_tags_do_not_count() {
        assert!(names_language("#string WGSL\nfn f() {}\nWGSL"));
        assert!(names_language("#string Json\n{}\nJson"));
        assert!(names_language("#string,cr glsl\nvoid main() {}\nglsl"));
        assert!(!names_language("#string END\nhello\nEND"));
        assert!(!names_language("#string WGSL_SOURCE\nx\nWGSL_SOURCE"));
        assert!(!names_language("\"WGSL\""));
    }
}
