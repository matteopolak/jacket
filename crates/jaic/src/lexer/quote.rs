//! Spelling a value as a Jai string literal: the inverse of the lexer's string escapes.

/// How a string literal is going to be used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuoteMode {
    /// An ordinary string.
    Plain,
    /// A format string (the first argument of `print` and friends): `%` is written `\%` so
    /// it is not read as a placeholder.
    FormatString,
}

/// `text` as a Jai string literal with the same value. Non-ASCII text is kept as written.
pub fn quote(text: &str, mode: QuoteMode) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match u8::try_from(c) {
            Ok(byte) if c.is_ascii() => escape_ascii(byte, mode, &mut out),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

/// `value` as a Jai string literal with the same bytes. Bytes outside printable ASCII are
/// written as `\xNN`, so the result is ASCII whatever the bytes hold.
pub fn quote_bytes(value: &[u8], mode: QuoteMode) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for &byte in value {
        if byte.is_ascii() {
            escape_ascii(byte, mode, &mut out);
        } else {
            out.push_str(&format!("\\x{byte:02x}"));
        }
    }
    out.push('"');
    out
}

fn escape_ascii(byte: u8, mode: QuoteMode, out: &mut String) {
    match byte {
        b'"' => out.push_str("\\\""),
        b'\\' => out.push_str("\\\\"),
        b'\n' => out.push_str("\\n"),
        b'\r' => out.push_str("\\r"),
        b'\t' => out.push_str("\\t"),
        0 => out.push_str("\\0"),
        b'%' if mode == QuoteMode::FormatString => out.push_str("\\%"),
        0x20..=0x7e => out.push(byte as char),
        _ => out.push_str(&format!("\\x{byte:02x}")),
    }
}

#[cfg(test)]
mod tests {
    use super::{QuoteMode, quote, quote_bytes};
    use crate::lexer::{Tok, lex};
    use crate::source::FileId;

    /// The value the lexer reads back from `literal`.
    fn lexed(literal: &str) -> Vec<u8> {
        let tokens = lex(FileId(0), literal).expect("the literal lexes");
        match tokens.first().map(|t| &t.tok) {
            Some(Tok::Str(value)) => value.to_vec(),
            other => panic!("expected one string token, got {other:?}"),
        }
    }

    #[test]
    fn common_escapes() {
        assert_eq!(
            quote("a\\b\"c\nd\re\tf\0g", QuoteMode::Plain),
            r#""a\\b\"c\nd\re\tf\0g""#
        );
        assert_eq!(quote("50%", QuoteMode::Plain), "\"50%\"");
        assert_eq!(quote("50%", QuoteMode::FormatString), "\"50\\%\"");
    }

    #[test]
    fn other_control_characters_use_hex() {
        assert_eq!(
            quote("\u{1}\u{1b}\u{7f}", QuoteMode::Plain),
            r#""\x01\x1b\x7f""#
        );
    }

    #[test]
    fn non_ascii_text_is_kept() {
        assert_eq!(quote("héllo ✓", QuoteMode::Plain), "\"héllo ✓\"");
    }

    #[test]
    fn bytes_outside_ascii_use_hex() {
        assert_eq!(
            quote_bytes(b"a\xffb\x80", QuoteMode::Plain),
            r#""a\xffb\x80""#
        );
    }

    #[test]
    fn quoted_text_lexes_back_to_the_same_string() {
        let everything: String = (0u32..0x250).filter_map(char::from_u32).collect();
        for text in [
            "",
            "plain",
            "tab\there",
            "nul\0inside",
            "a\\b \"q\" \n\r\n",
            "100% sure",
            "héllo ✓ 😀",
            &everything,
        ] {
            assert_eq!(lexed(&quote(text, QuoteMode::Plain)), text.as_bytes());
        }
    }

    #[test]
    fn quoted_bytes_lex_back_to_the_same_bytes() {
        let all: Vec<u8> = (0..=255).collect();
        assert_eq!(lexed(&quote_bytes(&all, QuoteMode::Plain)), all);
    }

    #[test]
    fn format_string_percent_is_the_lexers_literal_percent_marker() {
        // `\%` lexes to the byte the formatter reads as a literal `%`.
        assert_eq!(lexed(&quote("a%b", QuoteMode::FormatString)), b"a\x1fb");
    }
}
