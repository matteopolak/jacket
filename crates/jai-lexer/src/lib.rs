//! Lossless token spans. Here-string bodies remain opaque to later parsing.
use jai_source::{Diagnostic, Span};
use std::borrow::Cow;
mod tokens;
pub use tokens::{Directive, Keyword, Punct};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Ident,
    Keyword(Keyword),
    Number,
    String,
    HereString,
    Directive(Directive),
    UnknownDirective,
    Note,
    Punctuation(Punct),
    Eof,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: Kind,
    pub span: Span,
}

pub fn lex(source: &str) -> Result<Vec<Token>, Diagnostic> {
    Lexer {
        source,
        at: 0,
        tokens: Vec::new(),
        comments: Vec::new(),
    }
    .run()
}

/// Keep byte offsets while tolerating legacy non-UTF8 bytes only in comments.
pub fn decode_source(bytes: &[u8]) -> Result<Cow<'_, str>, Diagnostic> {
    if let Ok(s) = std::str::from_utf8(bytes) {
        return Ok(Cow::Borrowed(s));
    }
    let mut normalized = bytes.to_vec();
    let mut invalid = Vec::new();
    let mut offset = 0;
    while let Err(e) = std::str::from_utf8(&normalized[offset..]) {
        let start = offset + e.valid_up_to();
        let end = start + e.error_len().unwrap_or(normalized.len() - start);
        for (i, byte) in normalized.iter_mut().enumerate().take(end).skip(start) {
            invalid.push(i);
            *byte = b' ';
        }
        offset = end;
    }
    let source = String::from_utf8(normalized).unwrap();
    let mut lexer = Lexer {
        source: &source,
        at: 0,
        tokens: Vec::new(),
        comments: Vec::new(),
    };
    lexer.run()?;
    for at in invalid {
        if !lexer.comments.iter().any(|s| s.start <= at && at < s.end) {
            return Err(Diagnostic::new(
                Span::new(at, at + 1),
                "invalid UTF-8 outside a comment",
            ));
        }
    }
    Ok(Cow::Owned(source))
}

struct Lexer<'a> {
    source: &'a str,
    at: usize,
    tokens: Vec<Token>,
    comments: Vec<Span>,
}
impl Lexer<'_> {
    fn rest(&self) -> &str {
        &self.source[self.at..]
    }
    fn bump(&mut self) {
        self.at += self.rest().chars().next().unwrap().len_utf8();
    }
    fn error(&self, start: usize, msg: &str) -> Diagnostic {
        Diagnostic::new(Span::new(start, self.at), msg)
    }
    fn push(&mut self, start: usize, kind: Kind) {
        self.tokens.push(Token {
            kind,
            span: Span::new(start, self.at),
        });
    }
    fn ident(&mut self) {
        while self
            .rest()
            .chars()
            .next()
            .is_some_and(|c| c == '_' || c == '\\' || c.is_alphanumeric())
        {
            self.bump();
        }
    }
    fn run(&mut self) -> Result<Vec<Token>, Diagnostic> {
        while self.at < self.source.len() {
            let start = self.at;
            let c = self.rest().chars().next().unwrap();
            if c.is_whitespace() || (start == 0 && c == '\u{feff}') {
                self.bump();
                continue;
            }
            if self.rest().starts_with("//") {
                self.at += self.rest().find('\n').unwrap_or(self.rest().len());
                self.comments.push(Span::new(start, self.at));
                continue;
            }
            if self.rest().starts_with("/*") {
                self.at += 2;
                let mut depth = 1usize;
                while self.at < self.source.len() && depth > 0 {
                    if self.rest().starts_with("/*") {
                        depth += 1;
                        self.at += 2;
                    } else if self.rest().starts_with("*/") {
                        depth -= 1;
                        self.at += 2;
                    } else {
                        self.bump();
                    }
                }
                if depth != 0 {
                    return Err(self.error(start, "unterminated block comment"));
                }
                self.comments.push(Span::new(start, self.at));
                continue;
            }
            let kind = if c == '"' {
                self.bump();
                let mut closed = false;
                while self.at < self.source.len() {
                    let c = self.rest().chars().next().unwrap();
                    self.bump();
                    if c == '"' {
                        closed = true;
                        break;
                    }
                    if c == '\\' {
                        if self.at == self.source.len() {
                            break;
                        }
                        self.bump();
                    }
                }
                if !closed {
                    return Err(self.error(start, "unterminated string"));
                }
                Kind::String
            } else if c == '#' {
                self.bump();
                self.ident();
                if self.at == start + 1 {
                    Kind::Punctuation(Punct::Hash)
                } else if Directive::from_spelling(&self.source[start..self.at])
                    == Some(Directive::String)
                {
                    self.here_string(start)?;
                    Kind::HereString
                } else {
                    Directive::from_spelling(&self.source[start..self.at])
                        .map(Kind::Directive)
                        .unwrap_or(Kind::UnknownDirective)
                }
            } else if c == '@' {
                self.bump();
                self.ident();
                Kind::Note
            } else if c == '_' || c == '\\' || c.is_alphabetic() {
                self.bump();
                self.ident();
                Keyword::from_spelling(&self.source[start..self.at])
                    .map(Kind::Keyword)
                    .unwrap_or(Kind::Ident)
            } else if c.is_ascii_digit()
                || (c == '.'
                    && self
                        .rest()
                        .as_bytes()
                        .get(1)
                        .is_some_and(u8::is_ascii_digit))
            {
                self.number();
                Kind::Number
            } else {
                if let Some(&(spelling, punctuation)) = Punct::SPELLINGS
                    .iter()
                    .find(|(spelling, _)| self.rest().starts_with(spelling))
                {
                    self.at += spelling.len();
                    Kind::Punctuation(punctuation)
                } else {
                    self.bump();
                    return Err(self.error(start, "unrecognized source character"));
                }
            };
            self.push(start, kind);
        }
        self.push(self.at, Kind::Eof);
        Ok(std::mem::take(&mut self.tokens))
    }
    fn number(&mut self) {
        let hexadecimal = self.rest().starts_with("0x");
        let no_exponent = self.rest().starts_with("0b") || self.rest().starts_with("0h");
        let mut exponent = false;
        while self.at < self.source.len() {
            let c = self.rest().as_bytes()[0];
            if c.is_ascii_alphanumeric() || c == b'_' {
                exponent = !no_exponent
                    && if hexadecimal {
                        matches!(c, b'p' | b'P')
                    } else {
                        matches!(c, b'e' | b'E')
                    };
                self.at += 1;
            } else if (c == b'.' && !self.rest().starts_with(".."))
                || (exponent && matches!(c, b'+' | b'-'))
            {
                exponent = false;
                self.at += 1;
            } else {
                break;
            }
        }
    }
    fn here_string(&mut self, start: usize) -> Result<(), Diagnostic> {
        // Jai accepts #string,modifier TERMINATOR and terminators at line starts.
        while self.at < self.source.len() && self.rest().chars().next().unwrap().is_whitespace() {
            self.bump();
        }
        while self.rest().starts_with(',') {
            self.bump();
            while self.rest().starts_with([' ', '\t']) {
                self.bump();
            }
            let modifier = self.at;
            if self.rest().starts_with('\\') {
                // Focus uses #string,\% TAG to escape formatting markers.
                self.bump();
                if self.at == self.source.len() || self.rest().starts_with([' ', '\t', '\r', '\n'])
                {
                    return Err(
                        self.error(start, "expected escaped here-string modifier character")
                    );
                }
                self.bump();
            } else {
                self.ident();
            }
            if modifier == self.at {
                return Err(self.error(start, "expected here-string modifier"));
            }
            while self.at < self.source.len() && self.rest().chars().next().unwrap().is_whitespace()
            {
                self.bump();
            }
        }
        let tag_start = self.at;
        self.ident();
        if tag_start == self.at {
            return Err(self.error(start, "expected here-string terminator"));
        }
        let tag = &self.source[tag_start..self.at];
        let tail = self.rest();
        let Some(newline) = tail.find('\n') else {
            return Err(self.error(start, "unterminated here-string"));
        };
        self.at += newline + 1;
        while self.at < self.source.len() {
            let line_len = self.rest().find('\n').unwrap_or(self.rest().len());
            let line = &self.rest()[..line_len];
            let trimmed = line.trim_start_matches([' ', '\t']);
            if trimmed.starts_with(tag)
                && trimmed[tag.len()..]
                    .chars()
                    .next()
                    .is_none_or(|c| !c.is_alphanumeric() && c != '_')
            {
                self.at += line.len() - trimmed.len() + tag.len();
                return Ok(());
            }
            self.at += line_len;
            if self.at < self.source.len() {
                self.at += 1;
            }
        }
        Err(self.error(start, "unterminated here-string"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn texts(s: &str) -> Vec<&str> {
        lex(s)
            .unwrap()
            .into_iter()
            .filter(|t| t.kind != Kind::Eof)
            .map(|t| t.span.text(s))
            .collect()
    }
    #[test]
    fn nested_comments() {
        assert_eq!(texts("a /* /* b */ c */ + // hi\n b"), ["a", "+", "b"]);
    }
    #[test]
    fn ranges_and_numbers() {
        assert_eq!(
            texts("1..10 .5 0xff 1e-5 0x1.fp+3"),
            ["1", "..", "10", ".5", "0xff", "1e-5", "0x1.fp+3"]
        );
    }
    #[test]
    fn hexadecimal_digits_are_not_decimal_exponents() {
        assert_eq!(
            texts("0xFE-1 0xE+2 0h800E-1 1e-5 0x1.fp-3"),
            [
                "0xFE", "-", "1", "0xE", "+", "2", "0h800E", "-", "1", "1e-5", "0x1.fp-3"
            ]
        );
    }
    #[test]
    fn maximal_operators() {
        assert_eq!(
            texts("a<<=2; x---; a,,b; foo.{a=1}"),
            [
                "a", "<<=", "2", ";", "x", "---", ";", "a", ",,", "b", ";", "foo", ".{", "a", "=",
                "1", "}"
            ]
        );
    }
    #[test]
    fn opaque_here_string() {
        let s = "s :: #string END\n{ not code @ }\nEND;";
        let t = lex(s).unwrap();
        assert_eq!(t[2].kind, Kind::HereString);
        assert_eq!(t[3].span.text(s), ";");
    }
    #[test]
    fn formatting_escape_here_string_modifier() {
        let source = "code := #string,\\% JAI\ncase \\% { }\nJAI;";
        let tokens = lex(source).unwrap();
        assert_eq!(tokens[2].kind, Kind::HereString);
        assert_eq!(tokens[3].span.text(source), ";");
    }
    #[test]
    fn utf8_spans() {
        assert_eq!(
            texts("café := \"é\\\"\";"),
            ["café", ":=", "\"é\\\"\"", ";"]
        );
    }
    #[test]
    fn errors_are_located() {
        for s in ["/*", "\"", "#string END\nbody"] {
            assert_eq!(lex(s).unwrap_err().span.start, 0);
        }
    }
    #[test]
    fn legacy_comment_bytes_preserve_offsets() {
        let bytes = b"// Casta\xf1o\nmain :: () {}";
        let source = decode_source(bytes).unwrap();
        assert_eq!(source.len(), bytes.len());
        assert_eq!(lex(&source).unwrap()[0].span.start, 11);
    }
    #[test]
    fn invalid_utf8_code_is_rejected() {
        for bytes in [
            b"x \xff y".as_slice(),
            b"\"//\xff\"".as_slice(),
            b"#string END\n//\xff\nEND".as_slice(),
        ] {
            assert!(decode_source(bytes).is_err());
        }
    }
}
