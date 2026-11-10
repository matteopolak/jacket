//! Tokenizer. Keywords are ordinary identifiers; the parser gives them meaning.
mod api;
mod conditions;
mod floats;
mod here;
#[cfg(test)]
mod platform_tests;
mod punct;
mod quote;
mod scan;
mod value_map;

pub use self::api::{
    CPU_TAGS, Comment, CommentKind, Lexed, OS_TAGS, PlatformTarget, lex, lex_for_target, lex_ide,
    lex_stoppable, lex_with_trivia, lexical_regions,
};
pub use self::floats::{FloatLiteralPrecision, float_literal_precision, has_big_float32_exponent};
pub use self::here::{LANGUAGE_TAGS, here_string_tag, names_language};
pub use self::punct::{P, spellings as punctuation_spellings};
pub use self::quote::{QuoteMode, quote, quote_bytes};
pub use self::scan::STOP_POLL_TOKENS;
pub(crate) use self::scan::is_ident_char;
pub use self::value_map::string_value_map;
use crate::intern::Sym;
use crate::source::Span;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Ident(Sym),
    /// `#name`, stored without the hash.
    Directive(Sym),
    /// `@name` or `@"text"`, stored without the at-sign.
    Note(Arc<str>),
    /// Every literal the lexer accepts fits 64 bits (a larger one is `LexIntegerTooBig`); the
    /// narrow value keeps `Tok` from being 16-byte aligned, which would make a `Token` 48 bytes.
    Int(u64),
    Float(f64),
    Str(Arc<[u8]>),
    Punct(P),
    Eof,
}

impl Tok {
    /// The name of a directive token, without the `#`.
    pub fn directive(&self) -> Option<&str> {
        match self {
            Tok::Directive(name) => Some(name.as_str()),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    pub tok: Tok,
    pub span: Span,
    /// True when a newline separates this token from the previous one.
    pub newline_before: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::FileId;

    fn kinds(s: &str) -> Vec<Tok> {
        lex(FileId(0), s)
            .unwrap()
            .into_iter()
            .map(|t| t.tok)
            .collect()
    }

    #[test]
    fn basics() {
        let t = kinds("x := 1..2; y :: 0x_ff; z := 1.5e3; #run foo(); @note");
        assert_eq!(t[0], Tok::Ident(Sym::intern("x")));
        assert_eq!(t[1], Tok::Punct(P::ColonEq));
        assert_eq!(t[2], Tok::Int(1));
        assert_eq!(t[3], Tok::Punct(P::DotDot));
        assert_eq!(t[8], Tok::Int(255));
        assert_eq!(t[12], Tok::Float(1500.0));
        assert_eq!(t[14], Tok::Directive(Sym::intern("run")));
        assert!(matches!(&t[19], Tok::Note(n) if &**n == "note"));
    }

    #[test]
    fn hex_float_precision_follows_the_bit_pattern_width() {
        for (text, float64) in [
            ("0h3c00", false),
            ("0h3f800000", false),
            ("0h3f80_0000", false),
            ("0h3ff0000000000000", true),
            ("0H3FF0_0000_0000_0000", true),
        ] {
            let precision = float_literal_precision(text, 1.0);
            assert_eq!(precision.defaults_to_float64, float64, "{text}");
            assert!(!precision.requires_float64, "{text}");
        }
    }

    #[test]
    fn decimal_float_precision_counts_fractional_digits() {
        for (text, value, defaults, requires) in [
            ("1.5", 1.5, false, false),
            ("1.1234567", 1.123_456_7, false, false),
            ("1.12345678", 1.123_456_78, true, false),
            ("1.123456780", 1.123_456_78, true, false),
            ("0.123456789", 0.123_456_789, false, true),
            ("12342345234.0", 12_342_345_234.0, false, false),
            ("1.0e39", 1.0e39, false, true),
        ] {
            let precision = float_literal_precision(text, value);
            assert_eq!(
                (precision.defaults_to_float64, precision.requires_float64),
                (defaults, requires),
                "{text}"
            );
        }
    }

    #[test]
    fn identifier_separators() {
        let t = kinds("to\\ _pt := f(a.to\\, b);");
        assert_eq!(t[0], Tok::Ident(Sym::intern("to_pt")));
        assert_eq!(t[6], Tok::Ident(Sym::intern("to")));
        assert_eq!(t[7], Tok::Punct(P::Comma));
    }

    #[test]
    fn a_note_runs_to_the_next_whitespace() {
        let t = kinds("@short=j @range(1,64) @tag; x");
        assert!(matches!(&t[0], Tok::Note(n) if &**n == "short=j"));
        assert!(matches!(&t[1], Tok::Note(n) if &**n == "range(1,64)"));
        assert!(matches!(&t[2], Tok::Note(n) if &**n == "tag;"));
        assert_eq!(t[3], Tok::Ident(Sym::intern("x")));
    }

    #[test]
    fn a_call_style_note_reads_to_its_closing_parenthesis() {
        let t = kinds("@skip_on(.MACOS, reason = \"a ) b\") @a((b c)d) x");
        assert!(matches!(&t[0], Tok::Note(n) if &**n == "skip_on(.MACOS, reason = \"a ) b\")"));
        assert!(matches!(&t[1], Tok::Note(n) if &**n == "a((b c)d)"));
        assert_eq!(t[2], Tok::Ident(Sym::intern("x")));
    }

    #[test]
    fn a_note_parenthesis_must_close_on_its_line() {
        for text in ["@note(", "@note(a b\nc)", "@note(\"a)"] {
            let error = lex(FileId(0), text).unwrap_err();
            assert_eq!(
                error.code,
                crate::diagnostics::Code::LexUnterminatedNote,
                "{text:?}"
            );
        }
    }

    #[test]
    fn leading_dot_floats() {
        let t = kinds("x := .5; y := a.b; z := 1..2;");
        assert_eq!(t[2], Tok::Float(0.5));
        assert_eq!(t[7], Tok::Punct(P::Dot));
        assert_eq!(t[13], Tok::Punct(P::DotDot));
    }

    #[test]
    fn here_strings() {
        let t = kinds("s :: #string END\nhello\n  world\nEND;");
        assert_eq!(t[2], Tok::Str(b"hello\n  world\n".as_slice().into()));
        assert_eq!(t[3], Tok::Punct(P::Semi));
        let t = kinds("s :: #string,cr END\r\na\r\nb\n  END");
        assert_eq!(t[2], Tok::Str(b"a\r\nb\r\n".as_slice().into()));
    }

    #[test]
    fn here_string_names_may_follow_blank_lines_and_comments() {
        for src in [
            "s :: #string\n        END\nbody\nEND;",
            "s :: #string\n\n\tEND\nbody\nEND;",
            "s :: #string // note\n    // more\n    END\nbody\nEND;",
            "s :: #string /* a /* nested */ b */\n END\nbody\nEND;",
            "s :: #string,cr\n END\nbody\nEND;",
        ] {
            let t = kinds(src);
            let body: &[u8] = if src.contains(",cr") {
                b"body\r\n"
            } else {
                b"body\n"
            };
            assert_eq!(t[2], Tok::Str(body.into()), "{src:?}");
            assert_eq!(t[3], Tok::Punct(P::Semi), "{src:?}");
        }
    }

    #[test]
    fn here_string_names_end_their_line() {
        for src in [
            "s :: #string END junk\nbody\nEND;",
            "s :: #string END // note\nbody\nEND;",
            "s :: #string\n    END junk\nbody\nEND;",
        ] {
            let e = lex(FileId(0), src).unwrap_err();
            assert_eq!(
                e.code,
                crate::diagnostics::Code::LexTextAfterStringTerminator,
                "{src:?}"
            );
        }
        // Whitespace after the name is fine.
        assert_eq!(
            kinds("s :: #string END \t\nx\nEND;")[2],
            Tok::Str(b"x\n".as_slice().into())
        );
    }

    #[test]
    fn here_string_flags_cut_off_by_the_end_of_file() {
        // Found by the `lexer` fuzz target: a trailing `\` flag stepped past the end.
        for src in ["#string,\t\t\\", "#string,\\", "#string, \\%"] {
            let e = lex(FileId(0), src).unwrap_err();
            assert!(e.message.contains("here-string"), "{src:?}: {}", e.message);
        }
    }

    #[test]
    fn trivia_reports_comments_but_not_slashes_in_strings() {
        let text = "a // one\n\"http://x\" /* b /* c */ d */ e";
        let (tokens, comments, errors) = lex_with_trivia(FileId(0), text);
        assert!(errors.is_empty());
        assert_eq!(tokens.len(), lex(FileId(0), text).unwrap().len());
        let bodies: Vec<_> = comments.iter().map(|c| (c.kind, c.body(text))).collect();
        assert_eq!(
            bodies,
            [
                (CommentKind::Line, " one"),
                (CommentKind::Block, " b /* c */ d ")
            ]
        );
    }

    #[test]
    fn trivia_keeps_an_unterminated_block_comment() {
        let text = "a /* open\n// x";
        let (tokens, comments, errors) = lex_with_trivia(FileId(0), text);
        assert_eq!(errors.len(), 1);
        assert_eq!(tokens.len(), lex_ide(FileId(0), text).0.len());
        assert_eq!(comments.len(), 1);
        assert!(!comments[0].terminated);
        assert_eq!(comments[0].span.end as usize, text.len());
        assert_eq!(comments[0].body(text), " open\n// x");
    }

    #[test]
    fn nested_comments_and_escapes() {
        let t = kinds("/* a /* b */ c */ \"\\x41\\%\\u00e9\"");
        assert_eq!(t[0], Tok::Str(vec![b'A', 0x1f, 0xc3, 0xa9].into()));
    }

    #[test]
    fn unknown_escape_of_a_multibyte_character() {
        // Found by the `check` fuzz target: the error span ended inside the escaped character
        // (here U+FFFD, which invalid UTF-8 in a file turns into), and rendering it panicked.
        for src in ["x := \"\\\u{fffd}\";", "x := \"\\é", "\"\\\u{1d11e}"] {
            let e = lex(FileId(0), src).unwrap_err();
            let (start, end) = (e.span.start as usize, e.span.end as usize);
            assert!(src.is_char_boundary(end), "{src:?}: {start}..{end}");
            let escape = &src[start..end];
            assert!(
                escape.starts_with('\\') && escape.chars().count() == 2,
                "{escape:?}"
            );
            assert_eq!(e.message, format!("unknown escape sequence '{escape}'"));
        }
    }

    /// Lex `text` with a stop hook that says yes on its first question; how often it was asked.
    fn asked_before_stopping(text: &str) -> (bool, u32) {
        let mut asked = 0;
        let lexed = lex_stoppable(FileId(0), text, &mut || {
            asked += 1;
            true
        });
        (lexed.is_ok_and(|tokens| tokens.is_none()), asked)
    }

    #[test]
    fn a_huge_block_comment_can_be_stopped() {
        let size = 8 << 20;
        let plain = format!("/*{}*/ x := 1;", " ".repeat(size));
        let nested = format!("/*{}*/ x := 1;", "/* a */ ".repeat(size / 8));
        let lines = format!("/*{}*/ x := 1;", "a\n".repeat(size / 2));
        for text in [plain, nested, lines] {
            assert_eq!(asked_before_stopping(&text), (true, 1));
        }
    }

    #[test]
    fn a_huge_string_literal_can_be_stopped() {
        let size = 8 << 20;
        let plain = format!("x := \"{}\";", "a".repeat(size));
        let escaped = format!("x := \"{}\";", "\\n".repeat(size / 2));
        for text in [plain, escaped] {
            assert_eq!(asked_before_stopping(&text), (true, 1));
        }
    }

    #[test]
    fn a_huge_here_string_can_be_stopped() {
        let text = format!("x := #string END\n{}END;", "line of text\n".repeat(1 << 20));
        assert_eq!(asked_before_stopping(&text), (true, 1));
    }

    #[test]
    fn long_scans_that_are_not_stopped_still_lex_in_full() {
        let text = format!(
            "/*{}*/ x := \"{}\"; y := #string END\n{}END;",
            "/* a */ ".repeat(1 << 20),
            "a".repeat(1 << 22),
            "line\n".repeat(1 << 20),
        );
        let lexed = lex_stoppable(FileId(0), &text, &mut || false);
        assert!(lexed.is_ok_and(|tokens| tokens.is_some_and(|t| t.len() > 8)));
    }
}
