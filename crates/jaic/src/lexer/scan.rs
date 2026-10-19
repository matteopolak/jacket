//! The scanner state and its token, number and string readers.
use super::api::{Comment, CommentKind, PlatformTarget};
use super::conditions::fold_or;
use super::punct::{INTEGER_TOO_BIG, P, puncts_starting_with};
use super::{Tok, Token};
use crate::intern::Sym;
use crate::source::{Diagnostic, FileId, Span};
use std::rc::Rc;

pub(super) struct Lexer<'a> {
    pub(super) file: FileId,
    pub(super) text: &'a str,
    pub(super) src: &'a [u8],
    pub(super) at: usize,
    pub(super) out: Vec<Token>,
    pub(super) newline: bool,
    pub(super) regions: Option<Vec<Span>>,
    pub(super) comments: Option<Vec<Comment>>,
    /// The last error left a block comment or here-string open: nothing after it lexes
    /// reliably.
    pub(super) unclosed: bool,
    /// Set by `lex_for_target`.
    pub(super) platform: Option<PlatformTarget>,
    /// For each open brace, whether it belongs to a file-level `#if` the platform decided as
    /// true (so statements inside are still file level).
    pub(super) braces: Vec<bool>,
    /// How many of `braces` are not such `#if` blocks.
    pub(super) opaque_braces: usize,
}

/// The offset just past the closing quote of a string whose body starts at `at`.
fn skip_string(src: &[u8], mut at: usize) -> usize {
    while at < src.len() && src[at] != b'"' {
        if src[at] == b'\\' {
            at += 1;
        }
        at += 1;
    }
    at
}

fn is_ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_' || c >= 0x80
}

pub(crate) fn is_ident_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c >= 0x80
}

impl<'a> Lexer<'a> {
    /// Run to the end, restarting after each error; the errors in order.
    pub(super) fn recover(&mut self) -> Vec<Diagnostic> {
        let text = self.text;
        let mut errors = Vec::new();
        while let Err(mut error) = self.run() {
            let start = error.span.start as usize;
            let end = text[start..].find('\n').map_or(text.len(), |i| start + i);
            error.span.end = end as u32;
            // An unclosed comment/here-string has no trustworthy internal token boundaries.
            let opaque = std::mem::take(&mut self.unclosed);
            errors.push(error);
            self.at = if opaque {
                text.len()
            } else {
                end
            };
            self.newline = true;
        }
        errors
    }

    fn unclosed_err(&mut self, start: usize, msg: &str) -> Diagnostic {
        self.unclosed = true;
        self.err(start, msg)
    }

    fn err(&self, start: usize, msg: &str) -> Diagnostic {
        let mut end = self.at.max(start + 1).min(self.src.len().max(start + 1));
        // A span covers whole characters: the renderer slices the text with it.
        while end < self.text.len() && !self.text.is_char_boundary(end) {
            end += 1;
        }
        Diagnostic::error(Span::new(self.file, start, end), msg)
    }

    fn peek(&self, n: usize) -> u8 {
        *self.src.get(self.at + n).unwrap_or(&0)
    }

    fn push(&mut self, tok: Tok, start: usize) {
        if matches!(tok, Tok::Str(_))
            && let Some(regions) = self.regions.as_mut()
        {
            regions.push(Span::new(self.file, start, self.at));
        }
        self.out.push(Token {
            tok,
            span: Span::new(self.file, start, self.at),
            newline_before: self.newline,
        });
        self.newline = false;
    }

    pub(super) fn run(&mut self) -> Result<(), Diagnostic> {
        // Skip a UTF-8 byte order mark.
        if self.at == 0 && self.src.starts_with(&[0xEF, 0xBB, 0xBF]) {
            self.at = 3;
        }
        // A `#!` line at the very start (`#!/usr/bin/env jai`) is ignored, so a script can be run
        // directly.
        if self.src[self.at..].starts_with(b"#!") {
            while self.at < self.src.len() && self.src[self.at] != b'\n' {
                self.at += 1;
            }
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
                let sym = self.ident_with_separators();
                self.push(Tok::Ident(sym), start);
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
                // A note is every character up to the next whitespace, whatever it is
                // (`@range(1,64)`, `@short=j`; also a `;` or `{` that follows it directly).
                let from = self.at;
                // The corpus's working programs write help text as `@"?If set ..."` (Argparse
                // style), so a note that starts with a quote is read as a string.
                if self.peek(0) == b'"' {
                    self.at += 1;
                    let s = self.string_body(start)?;
                    let note: Rc<str> = String::from_utf8_lossy(&s).into();
                    self.push(Tok::Note(note), start);
                    continue;
                }
                while self.at < self.src.len() && !self.src[self.at].is_ascii_whitespace() {
                    self.at += 1;
                }
                let text = &self.src[from..self.at];
                // `@help("two words")`: the note stops at the space, and the leftover quote
                // would start a string.
                #[expect(
                    clippy::naive_bytecount,
                    reason = "the text is one short word; a bytecount dependency is not worth it"
                )]
                let odd_quotes = text.iter().filter(|&&b| b == b'"').count() % 2 == 1;
                if odd_quotes {
                    return Err(self
                        .err(
                            start,
                            "a note ends at the first whitespace, so it cannot be quoted",
                        )
                        .with_help(
                            "write the note without spaces, such as `@help_text` or `@range(1,64)`",
                        ));
                }
                let note: Rc<str> = String::from_utf8_lossy(text).into();
                self.push(Tok::Note(note), start);
            } else {
                let rest = &self.src[self.at..];
                let Some(&(text, p)) = puncts_starting_with(c)
                    .iter()
                    .find(|(t, _)| rest.starts_with(t.as_bytes()))
                else {
                    self.at += 1;
                    return Err(self.err(start, &format!("unexpected character `{}`", c as char)));
                };
                self.at += text.len();
                self.push(Tok::Punct(p), start);
                if self.platform.is_some() {
                    self.track_brace(p);
                }
            }
        }
    }

    /// Follows the nesting of braces so a file-level `#if` can be recognized, and steps over
    /// the body of one the platform rules out.
    fn track_brace(&mut self, p: P) {
        match p {
            P::DotBrace => {
                self.braces.push(false);
                self.opaque_braces += 1;
            }
            P::RBrace => {
                if self.braces.pop() == Some(false) {
                    self.opaque_braces -= 1;
                }
            }
            P::LBrace => {
                let decided = if self.opaque_braces == 0 {
                    self.platform
                        .and_then(|target| self.condition_before_brace(target))
                } else {
                    None
                };
                match decided {
                    Some(false) if self.skip_block() => {}
                    Some(true) => self.braces.push(true),
                    _ => {
                        self.braces.push(false);
                        self.opaque_braces += 1;
                    }
                }
            }
            _ => {}
        }
    }

    /// The value of the `#if` condition that ends at the `{` just pushed, when it is only
    /// `OS` and `CPU` tests (`==`, `!=`, `!(..)`, `&&`, `||`, parentheses).
    fn condition_before_brace(&self, target: PlatformTarget) -> Option<bool> {
        let brace = self.out.len().checked_sub(1)?;
        let limit = brace.saturating_sub(64);
        let start = (limit..brace)
            .rev()
            .find(|&i| matches!(&self.out[i].tok, Tok::Directive(d) if d.as_str() == "if"))?;
        let cond = &self.out[start + 1..brace];
        let mut at = 0;
        let value = fold_or(cond, &mut at, target)?;
        (at == cond.len()).then_some(value)
    }

    /// Steps over the rest of a `{` block just pushed, up to its matching `}`, which is pushed.
    /// False (with the position unchanged) when the body holds something this does not
    /// follow exactly as `run` does: a here-string.
    fn skip_block(&mut self) -> bool {
        let src = self.src;
        let mut at = self.at;
        let mut depth = 1usize;
        let mut newline = false;
        while at < src.len() {
            match src[at] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        self.at = at + 1;
                        self.newline = newline;
                        self.push(Tok::Punct(P::RBrace), at);
                        return true;
                    }
                }
                b'\n' => newline = true,
                b'"' => at = skip_string(src, at + 1),
                b'@' => {
                    if src.get(at + 1) == Some(&b'"') {
                        at = skip_string(src, at + 2);
                    } else {
                        // A note runs to the next whitespace, whatever it holds.
                        while at + 1 < src.len() && !src[at + 1].is_ascii_whitespace() {
                            at += 1;
                        }
                    }
                }
                b'/' if src.get(at + 1) == Some(&b'/') => {
                    while at < src.len() && src[at] != b'\n' {
                        at += 1;
                    }
                    continue;
                }
                b'/' if src.get(at + 1) == Some(&b'*') => {
                    at += 2;
                    let mut nesting = 1;
                    while nesting > 0 && at < src.len() {
                        if src[at] == b'/' && src.get(at + 1) == Some(&b'*') {
                            nesting += 1;
                            at += 2;
                        } else if src[at] == b'*' && src.get(at + 1) == Some(&b'/') {
                            nesting -= 1;
                            at += 2;
                        } else {
                            if src[at] == b'\n' {
                                newline = true;
                            }
                            at += 1;
                        }
                    }
                    continue;
                }
                b'#' => {
                    let mut name = at + 1;
                    while matches!(src.get(name), Some(b' ' | b'\t')) {
                        name += 1;
                    }
                    if src[name.min(src.len())..].starts_with(b"string") {
                        return false;
                    }
                }
                _ => {}
            }
            at += 1;
        }
        false
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
        // Identifiers end at an ASCII byte (or the end), so this is a char boundary.
        self.text.get(s..self.at).unwrap_or("?")
    }

    /// Identifiers may contain `\\` as an ignored visual separator: `group\\_fraction`. A trailing
    /// backslash pads a short name to line up with its neighbours (`arrow.to\\, x` next to
    /// `arrow.from, x`) and is dropped too.
    fn ident_with_separators(&mut self) -> Sym {
        let first = self.ident();
        if self.peek(0) != b'\\' {
            return Sym::intern(first);
        }
        let mut name = String::from(first);
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
        Sym::intern(&name)
    }

    fn note_comment(&mut self, start: usize, kind: CommentKind, terminated: bool) {
        let span = Span::new(self.file, start, self.at);
        if let Some(comments) = self.comments.as_mut() {
            comments.push(Comment {
                span,
                kind,
                terminated,
            });
        }
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
                let start = self.at;
                while self.at < self.src.len() && self.src[self.at] != b'\n' {
                    self.at += 1;
                }
                if let Some(regions) = self.regions.as_mut() {
                    regions.push(Span::new(self.file, start, self.at));
                }
                self.note_comment(start, CommentKind::Line, true);
            } else if c == b'/' && self.peek(1) == b'*' {
                let start = self.at;
                self.at += 2;
                let mut depth = 1;
                while depth > 0 {
                    if self.at >= self.src.len() {
                        self.note_comment(start, CommentKind::Block, false);
                        return Err(self.unclosed_err(start, "unterminated block comment"));
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
                if let Some(regions) = self.regions.as_mut() {
                    regions.push(Span::new(self.file, start, self.at));
                }
                self.note_comment(start, CommentKind::Block, true);
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
            let mut too_big = false;
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
                too_big |= value > u64::MAX as u128;
                self.at += 1;
            }
            if too_big {
                return Err(self.err(start, INTEGER_TOO_BIG));
            }
            if self.at == digits_start {
                return Err(self.err(start, "expected digits after numeric prefix"));
            }
            if radix == 0 {
                let ndigits = self.src[digits_start..self.at]
                    .iter()
                    .filter(|&&c| c != b'_')
                    .count();
                if !matches!(ndigits, 4 | 8 | 16) {
                    return Err(self.err(
                        start,
                        "a hex float bit pattern requires 4, 8 or 16 hex digits",
                    ));
                }
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
            match text.parse::<u128>() {
                Ok(v) if v <= u64::MAX as u128 => Ok(Tok::Int(v)),
                _ => Err(self.err(start, INTEGER_TOO_BIG)),
            }
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
                            // The escaped character may be several bytes long.
                            let ch = self.text[self.at - 1..].chars().next().unwrap_or('?');
                            self.at += ch.len_utf8() - 1;
                            return Err(self.err(
                                self.at - 1 - ch.len_utf8(),
                                &format!("unknown escape sequence '\\{ch}'"),
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
        let header = super::here::header(self.src, self.at);
        self.at = header.end;
        if header.tag.is_empty() {
            return Err(self.unclosed_err(start, "expected here-string terminator"));
        }
        let (escape_percent, cr) = (header.escape_percent, header.cr);
        let src = self.src;
        let tag = &src[header.tag];
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
        Err(self.unclosed_err(start, "unterminated here-string"))
    }
}
