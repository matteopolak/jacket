//! Tokenizer. Keywords are ordinary identifiers; the parser gives them meaning.
use crate::intern::Sym;
use crate::source::{Diagnostic, FileId, Span};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum P {
    // Multi-character punctuation, longest first in `PUNCT`.
    RotlAssign,   // <<<=
    RotrAssign,   // >>>=
    Uninit,       // ---
    AndAndAssign, // &&=
    OrOrAssign,   // ||=
    ShlAssign,    // <<=
    ShrAssign,    // >>=
    Rotl,         // <<<
    Rotr,         // >>>
    ColonColon,   // ::
    ColonEq,      // :=
    Arrow,        // ->
    FatArrow,     // =>
    DotDot,       // ..
    DotBrace,     // .{
    DotBracket,   // .[
    DotStar,      // .*
    Shl,          // <<
    Shr,          // >>
    Le,           // <=
    Ge,           // >=
    EqEq,         // ==
    Ne,           // !=
    AndAnd,       // &&
    OrOr,         // ||
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    RemAssign,
    AndAssign,
    OrAssign,
    XorAssign,
    DollarDollar, // $$
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LParen,
    RParen,
    Semi,
    Comma,
    Colon,
    Dot,
    Dollar,
    Backtick,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    Bang,
    Lt,
    Gt,
    Pipe,
    Amp,
    Caret,
    Tilde,
    Question,
}

const PUNCT: &[(&str, P)] = &[
    ("<<<=", P::RotlAssign),
    (">>>=", P::RotrAssign),
    ("---", P::Uninit),
    ("&&=", P::AndAndAssign),
    ("||=", P::OrOrAssign),
    ("<<=", P::ShlAssign),
    (">>=", P::ShrAssign),
    ("<<<", P::Rotl),
    (">>>", P::Rotr),
    ("::", P::ColonColon),
    (":=", P::ColonEq),
    ("->", P::Arrow),
    ("=>", P::FatArrow),
    ("..", P::DotDot),
    (".{", P::DotBrace),
    (".[", P::DotBracket),
    (".*", P::DotStar),
    ("<<", P::Shl),
    (">>", P::Shr),
    ("<=", P::Le),
    (">=", P::Ge),
    ("==", P::EqEq),
    ("!=", P::Ne),
    ("&&", P::AndAnd),
    ("||", P::OrOr),
    ("+=", P::AddAssign),
    ("-=", P::SubAssign),
    ("*=", P::MulAssign),
    ("/=", P::DivAssign),
    ("%=", P::RemAssign),
    ("&=", P::AndAssign),
    ("|=", P::OrAssign),
    ("^=", P::XorAssign),
    ("$$", P::DollarDollar),
    ("{", P::LBrace),
    ("}", P::RBrace),
    ("[", P::LBracket),
    ("]", P::RBracket),
    ("(", P::LParen),
    (")", P::RParen),
    (";", P::Semi),
    (",", P::Comma),
    (":", P::Colon),
    (".", P::Dot),
    ("$", P::Dollar),
    ("`", P::Backtick),
    ("+", P::Plus),
    ("-", P::Minus),
    ("*", P::Star),
    ("/", P::Slash),
    ("%", P::Percent),
    ("=", P::Eq),
    ("!", P::Bang),
    ("<", P::Lt),
    (">", P::Gt),
    ("|", P::Pipe),
    ("&", P::Amp),
    ("^", P::Caret),
    ("~", P::Tilde),
    ("?", P::Question),
];

impl P {
    pub fn text(self) -> &'static str {
        PUNCT
            .iter()
            .find(|(_, p)| *p == self)
            .map(|(s, _)| *s)
            .unwrap()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Ident(Sym),
    /// `#name`, stored without the hash.
    Directive(Sym),
    /// `@name` or `@"text"`, stored without the at-sign.
    Note(Rc<str>),
    Int(u128),
    Float(f64),
    Str(Rc<[u8]>),
    Punct(P),
    Eof,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub tok: Tok,
    pub span: Span,
    /// True when a newline separates this token from the previous one.
    pub newline_before: bool,
}

pub fn lex(file: FileId, text: &str) -> Result<Vec<Token>, Diagnostic> {
    let mut lx = Lexer {
        file,
        src: text.as_bytes(),
        at: 0,
        out: Vec::new(),
        newline: true,
    };
    lx.run()?;
    Ok(lx.out)
}

struct Lexer<'a> {
    file: FileId,
    src: &'a [u8],
    at: usize,
    out: Vec<Token>,
    newline: bool,
}

fn is_ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_' || c >= 0x80
}
fn is_ident_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c >= 0x80
}

impl<'a> Lexer<'a> {
    fn err(&self, start: usize, msg: &str) -> Diagnostic {
        Diagnostic::error(
            Span::new(
                self.file,
                start,
                self.at.max(start + 1).min(self.src.len().max(start + 1)),
            ),
            msg,
        )
    }
    fn peek(&self, n: usize) -> u8 {
        *self.src.get(self.at + n).unwrap_or(&0)
    }
    fn push(&mut self, tok: Tok, start: usize) {
        self.out.push(Token {
            tok,
            span: Span::new(self.file, start, self.at),
            newline_before: self.newline,
        });
        self.newline = false;
    }
    fn run(&mut self) -> Result<(), Diagnostic> {
        // Skip a UTF-8 byte order mark.
        if self.src.starts_with(&[0xEF, 0xBB, 0xBF]) {
            self.at = 3;
        }
        loop {
            self.skip_trivia()?;
            if self.at >= self.src.len() {
                let at = self.at;
                self.push(Tok::Eof, at);
                return Ok(());
            }
            let start = self.at;
            let c = self.src[self.at];
            if is_ident_start(c) {
                let name = self.ident_with_separators();
                self.push(Tok::Ident(Sym::intern(&name)), start);
            } else if c.is_ascii_digit() || self.at_leading_dot_float() {
                let tok = self.number(start)?;
                self.push(tok, start);
            } else if c == b'"' {
                self.at += 1;
                let s = self.string_body(start)?;
                self.push(Tok::Str(s.into()), start);
            } else if c == b'#' && {
                let mut skip = 1;
                while matches!(self.peek(skip), b' ' | b'\t') {
                    skip += 1;
                }
                is_ident_start(self.peek(skip))
            } {
                // `# import` (with spaces) is accepted as `#import`.
                self.at += 1;
                while matches!(self.peek(0), b' ' | b'\t') {
                    self.at += 1;
                }
                let name = self.ident();
                if name == "string" {
                    let s = self.here_string(start)?;
                    self.push(Tok::Str(s.into()), start);
                } else {
                    let sym = Sym::intern(name);
                    self.push(Tok::Directive(sym), start);
                }
            } else if c == b'@' {
                self.at += 1;
                let note: Rc<str> = if self.peek(0) == b'"' {
                    self.at += 1;
                    let s = self.string_body(start)?;
                    String::from_utf8_lossy(&s).into()
                } else {
                    // Notes may contain any non-space characters, e.g. @Bindings(name) or @foo.bar
                    let s = self.at;
                    let mut depth = 0i32;
                    while self.at < self.src.len() {
                        let ch = self.src[self.at];
                        if ch == b'(' {
                            depth += 1;
                        } else if ch == b')' {
                            if depth == 0 {
                                break;
                            }
                            depth -= 1;
                        } else if depth == 0
                            && !(is_ident_char(ch)
                                || ch == b'.'
                                || ch == b'-'
                                || ch == b'/'
                                || ch == b':'
                                || ch == b'=')
                        {
                            break;
                        }
                        self.at += 1;
                    }
                    String::from_utf8_lossy(&self.src[s..self.at]).into()
                };
                self.push(Tok::Note(note), start);
            } else {
                let rest = &self.src[self.at..];
                let Some(&(text, p)) = PUNCT.iter().find(|(t, _)| rest.starts_with(t.as_bytes()))
                else {
                    self.at += 1;
                    return Err(self.err(start, &format!("unexpected character '{}'", c as char)));
                };
                self.at += text.len();
                self.push(Tok::Punct(p), start);
            }
        }
    }
    /// `.5` is a float literal unless the dot continues an expression (`x.5`, `1..5`).
    fn at_leading_dot_float(&self) -> bool {
        let after_operand = self.at > 0 && {
            let prev = self.src[self.at - 1];
            is_ident_char(prev) || matches!(prev, b')' | b']' | b'.' | b'"')
        };
        self.peek(0) == b'.' && self.peek(1).is_ascii_digit() && !after_operand
    }
    fn ident(&mut self) -> &'a str {
        let s = self.at;
        while self.at < self.src.len() && is_ident_char(self.src[self.at]) {
            self.at += 1;
        }
        std::str::from_utf8(&self.src[s..self.at]).unwrap_or("?")
    }
    /// Identifiers may contain `\\` as an ignored visual separator: `group\\_fraction`. A trailing
    /// backslash pads a short name to line up with its neighbours (`arrow.to\\, x` next to
    /// `arrow.from, x`) and is dropped too.
    fn ident_with_separators(&mut self) -> String {
        let mut name = String::from(self.ident());
        while self.peek(0) == b'\\' {
            let mut skip = 1;
            while matches!(self.peek(skip), b' ' | b'\t') {
                skip += 1;
            }
            self.at += skip;
            if !is_ident_char(self.peek(0)) {
                break;
            }
            name.push_str(self.ident());
        }
        name
    }
    fn skip_trivia(&mut self) -> Result<(), Diagnostic> {
        loop {
            let c = self.peek(0);
            if c == b'\n' {
                self.newline = true;
                self.at += 1;
            } else if c == b' ' || c == b'\t' || c == b'\r' || c == 0x0c || c == 0x0b {
                self.at += 1;
            } else if c == 0xc2 && self.peek(1) == 0xa0 {
                // U+00A0 (no-break space) separates tokens like a space.
                self.at += 2;
            } else if c == b'/' && self.peek(1) == b'/' {
                while self.at < self.src.len() && self.src[self.at] != b'\n' {
                    self.at += 1;
                }
            } else if c == b'/' && self.peek(1) == b'*' {
                let start = self.at;
                self.at += 2;
                let mut depth = 1;
                while depth > 0 {
                    if self.at >= self.src.len() {
                        return Err(self.err(start, "unterminated block comment"));
                    }
                    if self.peek(0) == b'/' && self.peek(1) == b'*' {
                        depth += 1;
                        self.at += 2;
                    } else if self.peek(0) == b'*' && self.peek(1) == b'/' {
                        depth -= 1;
                        self.at += 2;
                    } else {
                        if self.peek(0) == b'\n' {
                            self.newline = true;
                        }
                        self.at += 1;
                    }
                }
            } else {
                return Ok(());
            }
        }
    }
    fn number(&mut self, start: usize) -> Result<Tok, Diagnostic> {
        let radix_prefix = |c: u8| match c {
            b'x' | b'X' => Some(16),
            b'b' | b'B' => Some(2),
            b'h' | b'H' => Some(0), // hex float bit pattern
            _ => None,
        };
        if self.peek(0) == b'0'
            && let Some(radix) = radix_prefix(self.peek(1))
        {
            self.at += 2;
            let digits_start = self.at;
            let base = if radix == 0 {
                16
            } else {
                radix
            };
            let mut value: u128 = 0;
            while self.at < self.src.len() {
                let ch = self.src[self.at];
                if ch == b'_' {
                    self.at += 1;
                    continue;
                }
                let Some(d) = (ch as char).to_digit(base) else {
                    break;
                };
                value = value.wrapping_mul(base as u128).wrapping_add(d as u128);
                self.at += 1;
            }
            if self.at == digits_start {
                return Err(self.err(start, "expected digits after numeric prefix"));
            }
            if radix == 0 {
                let ndigits = self.src[digits_start..self.at]
                    .iter()
                    .filter(|&&c| c != b'_')
                    .count();
                return Ok(if ndigits <= 8 {
                    Tok::Float(f32::from_bits(value as u32) as f64)
                } else {
                    Tok::Float(f64::from_bits(value as u64))
                });
            }
            return Ok(Tok::Int(value));
        }
        let mut text = String::new();
        let mut is_float = false;
        while self.at < self.src.len() {
            let ch = self.src[self.at];
            if ch.is_ascii_digit() {
                text.push(ch as char);
            } else if ch == b'_' {
            } else if ch == b'.' && !is_float && self.peek(1).is_ascii_digit() {
                is_float = true;
                text.push('.');
            } else if ch == b'.'
                && !is_float
                && self.peek(1) != b'.'
                && !is_ident_start(self.peek(1))
            {
                // `1.` is a float literal; `1..2` is a range; `1.foo` is a member access.
                is_float = true;
                text.push('.');
            } else if (ch == b'e' || ch == b'E')
                && (self.peek(1).is_ascii_digit()
                    || ((self.peek(1) == b'-' || self.peek(1) == b'+')
                        && self.peek(2).is_ascii_digit()))
            {
                is_float = true;
                text.push('e');
                self.at += 1;
                text.push(self.src[self.at] as char);
            } else {
                break;
            }
            self.at += 1;
        }
        if is_float {
            text.parse::<f64>()
                .map(Tok::Float)
                .map_err(|_| self.err(start, "invalid float literal"))
        } else {
            text.parse::<u128>()
                .map(Tok::Int)
                .map_err(|_| self.err(start, "integer literal too large"))
        }
    }
    fn string_body(&mut self, start: usize) -> Result<Vec<u8>, Diagnostic> {
        let mut out = Vec::new();
        loop {
            let Some(&c) = self.src.get(self.at) else {
                return Err(self.err(start, "unterminated string literal"));
            };
            self.at += 1;
            match c {
                b'"' => return Ok(out),
                b'\\' => {
                    let Some(&e) = self.src.get(self.at) else {
                        return Err(self.err(start, "unterminated string literal"));
                    };
                    self.at += 1;
                    match e {
                        b'n' => out.push(b'\n'),
                        b'r' => out.push(b'\r'),
                        b't' => out.push(b'\t'),
                        b'0' => out.push(0),
                        b'e' => out.push(0x1b),
                        b'a' => out.push(0x07),
                        b'b' => out.push(0x08),
                        b'f' => out.push(0x0c),
                        b'v' => out.push(0x0b),
                        b'%' => out.push(0x1f),
                        b'\\' | b'"' | b'\'' => out.push(e),
                        b'\n' => {}
                        b'x' => {
                            let v = self.hex_digits(2, start)?;
                            out.push(v as u8);
                        }
                        b'd' => {
                            let mut v = 0u32;
                            for _ in 0..3 {
                                let d = self.peek(0);
                                if !d.is_ascii_digit() {
                                    return Err(
                                        self.err(start, "expected three decimal digits after \\d")
                                    );
                                }
                                v = v * 10 + (d - b'0') as u32;
                                self.at += 1;
                            }
                            out.push(v as u8);
                        }
                        b'u' | b'U' => {
                            let n = if e == b'u' {
                                4
                            } else {
                                8
                            };
                            let v = self.hex_digits(n, start)?;
                            let ch = char::from_u32(v)
                                .ok_or_else(|| self.err(start, "invalid unicode escape"))?;
                            let mut buf = [0; 4];
                            out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                        }
                        _ => {
                            return Err(self.err(
                                self.at - 2,
                                &format!("unknown escape sequence '\\{}'", e as char),
                            ));
                        }
                    }
                }
                b'\n' => {
                    self.newline = true;
                    out.push(b'\n');
                }
                _ => out.push(c),
            }
        }
    }
    fn hex_digits(&mut self, n: usize, start: usize) -> Result<u32, Diagnostic> {
        let mut v = 0u32;
        for _ in 0..n {
            let d = (self.peek(0) as char)
                .to_digit(16)
                .ok_or_else(|| self.err(start, "expected hex digits in escape"))?;
            v = v * 16 + d;
            self.at += 1;
        }
        Ok(v)
    }
    /// `#string TERMINATOR` (optionally `#string,cr TERM` or `#string,\% TERM`).
    fn here_string(&mut self, start: usize) -> Result<Vec<u8>, Diagnostic> {
        let mut escape_percent = false;
        let mut cr = false;
        loop {
            while matches!(self.peek(0), b' ' | b'\t') {
                self.at += 1;
            }
            if self.peek(0) != b',' {
                break;
            }
            self.at += 1;
            while matches!(self.peek(0), b' ' | b'\t') {
                self.at += 1;
            }
            if self.peek(0) == b'\\' {
                self.at += 1;
                if self.peek(0) == b'%' {
                    escape_percent = true;
                }
                // The escaped character, unless the file ends at the backslash.
                self.at = (self.at + 1).min(self.src.len());
            } else {
                cr |= self.ident() == "cr";
            }
        }
        let tag = self.ident();
        if tag.is_empty() {
            return Err(self.err(start, "expected here-string terminator"));
        }
        let tag = tag.as_bytes();
        // Body starts on the next line.
        while self.at < self.src.len() && self.src[self.at] != b'\n' {
            self.at += 1;
        }
        self.at += 1;
        let body_start = self.at.min(self.src.len());
        while self.at < self.src.len() {
            let line_end = self.src[self.at..]
                .iter()
                .position(|&c| c == b'\n')
                .map_or(self.src.len(), |p| self.at + p);
            let line = &self.src[self.at..line_end];
            let indent = line
                .iter()
                .take_while(|&&c| c == b' ' || c == b'\t')
                .count();
            let trimmed = &line[indent..];
            if trimmed.starts_with(tag) && trimmed.get(tag.len()).is_none_or(|&c| !is_ident_char(c))
            {
                // Every line ending, including the one before the terminator line, is part of
                // the string, normalized to `\n` (`\r\n` with `#string,cr`).
                let mut body = Vec::with_capacity(self.at - body_start);
                for &c in &self.src[body_start..self.at] {
                    match c {
                        b'\r' => {}
                        b'\n' if cr => body.extend_from_slice(b"\r\n"),
                        c => body.push(c),
                    }
                }
                if escape_percent {
                    let mut out = Vec::with_capacity(body.len());
                    let mut i = 0;
                    while i < body.len() {
                        if body[i] == b'\\' && body.get(i + 1) == Some(&b'%') {
                            out.push(0x1f);
                            i += 2;
                        } else {
                            out.push(body[i]);
                            i += 1;
                        }
                    }
                    body = out;
                }
                self.at += indent + tag.len();
                self.newline = true;
                return Ok(body);
            }
            self.at = line_end + 1;
        }
        self.at = self.src.len();
        Err(self.err(start, "unterminated here-string"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn identifier_separators() {
        let t = kinds("to\\ _pt := f(a.to\\, b);");
        assert_eq!(t[0], Tok::Ident(Sym::intern("to_pt")));
        assert_eq!(t[6], Tok::Ident(Sym::intern("to")));
        assert_eq!(t[7], Tok::Punct(P::Comma));
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
    fn here_string_flags_cut_off_by_the_end_of_file() {
        // Found by the `lexer` fuzz target: a trailing `\` flag stepped past the end.
        for src in ["#string,\t\t\\", "#string,\\", "#string, \\%"] {
            let e = lex(FileId(0), src).unwrap_err();
            assert!(e.message.contains("here-string"), "{src:?}: {}", e.message);
        }
    }
    #[test]
    fn nested_comments_and_escapes() {
        let t = kinds("/* a /* b */ c */ \"\\x41\\%\\u00e9\"");
        assert_eq!(t[0], Tok::Str(vec![b'A', 0x1f, 0xc3, 0xa9].into()));
    }
}
