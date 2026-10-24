//! Inline holes of `@Interpolates` calls, as the editor sees them.
//!
//! The compiler records which string literals sit in an interpolating parameter and in which
//! mode (`Compiler::ide_interpolations`, from the callee's note alone: nothing is evaluated).
//! The holes are found here with the compiler's own scanner, so the editor and the compiler agree
//! on where code starts, and mapped back to document text through the literal's escapes.
use crate::analysis::{Span, Token, TokenKind, editor_tokens};
use crate::session::Session;
use crate::{DocumentUri, Position};
use jaic::interpolate::{self, Mode};
use jaic::lexer::Tok;
use jaic::source::FileId;
use std::rc::Rc;

/// One hole: `%name.field`, `%array[i]` or `%(expression)`.
pub(crate) struct Hole {
    /// The whole `%...`.
    pub whole: Span,
    /// The expression, without the `%` and, for `%(...)`, without the parentheses.
    pub expr: Span,
    /// The expression's tokens, with spans in the document.
    pub tokens: Vec<Token>,
}

/// An interpolated literal.
pub(crate) struct Site {
    pub string: Span,
    pub mode: Mode,
    pub holes: Vec<Hole>,
    value: Rc<[u8]>,
    /// For each byte of `value` (and one past it) its offset in the literal's spelling.
    map: Vec<u32>,
}

/// What a `%` typed in a literal is completing: the identifiers before the word, and the word.
pub(crate) struct HoleCursor {
    /// Where the first name of the chain starts.
    pub at: usize,
    pub chain: Vec<String>,
    pub prefix: String,
}

fn ident_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c >= 0x80
}

impl Site {
    fn new(text: &str, string: Span, value: Rc<[u8]>, mode: Mode) -> Option<Site> {
        let map = jaic::lexer::string_value_map(text.get(string.start..string.end)?, value.len())?;
        let source = |v: usize| string.start + map[v] as usize;
        let mut holes = Vec::new();
        for hole in interpolate::holes(&value, mode) {
            let range = hole.expr.clone();
            let mut tokens = Vec::new();
            if let Ok(inner) = std::str::from_utf8(&value[range.clone()])
                && let Ok(lexed) = jaic::lexer::lex(FileId(0), inner)
            {
                let moved = lexed
                    .into_iter()
                    .map(|mut t| {
                        t.span.start = source(range.start + t.span.start as usize) as u32;
                        t.span.end = source(range.start + t.span.end as usize) as u32;
                        t
                    })
                    .collect::<Vec<_>>();
                tokens = editor_tokens(&moved);
            }
            holes.push(Hole {
                whole: Span::new(source(hole.whole.start), source(hole.whole.end)),
                expr: Span::new(source(range.start), source(range.end)),
                tokens,
            });
        }
        Some(Site {
            string,
            mode,
            holes,
            value,
            map,
        })
    }

    /// The value offset a document byte is at, when it is between two value bytes.
    fn value_offset(&self, byte: usize) -> Option<usize> {
        let rel = byte.checked_sub(self.string.start)?;
        let at = self
            .map
            .partition_point(|&m| m as usize <= rel)
            .checked_sub(1)?;
        (self.map[at] as usize == rel).then_some(at)
    }

    fn source(&self, v: usize) -> usize {
        self.string.start + self.map[v] as usize
    }

    /// The hole being typed at `byte`, if a `%` there starts one.
    fn cursor(&self, byte: usize) -> Option<HoleCursor> {
        let v = self.value_offset(byte)?;
        let value = &self.value;
        // The word before the cursor: names and the dots between them.
        let mut j = v;
        while j > 0 && (ident_char(value[j - 1]) || value[j - 1] == b'.') {
            j -= 1;
        }
        let start = if j > 0 && value[j - 1] == b'%' {
            Some((j - 1, false))
        } else {
            // Inside `%(...)`: the nearest unclosed `(` right after a `%`.
            let mut depth = 0usize;
            let mut open = None;
            for q in (0..j).rev() {
                match value[q] {
                    b')' => depth += 1,
                    b'(' if depth == 0 => {
                        open = Some(q);
                        break;
                    }
                    b'(' => depth -= 1,
                    _ => {}
                }
            }
            open.filter(|&q| q > 0 && value[q - 1] == b'%')
                .map(|q| (q - 1, true))
        };
        let (p, paren) = start?;
        // The `%` must start a hole: not inside quoted SQL, not the second half of `%%`.
        let mut probe = value[..p].to_vec();
        probe.extend_from_slice(if paren {
            b"%(a)"
        } else {
            b"%a"
        });
        if !interpolate::holes(&probe, self.mode)
            .iter()
            .any(|h| h.whole.start == p)
        {
            return None;
        }
        let word = std::str::from_utf8(&value[j..v]).ok()?;
        let mut names: Vec<&str> = word.split('.').collect();
        let prefix = names.pop().unwrap_or("").to_owned();
        let valid = |n: &&str| n.bytes().next().is_some_and(|c| !c.is_ascii_digit());
        if !names.iter().all(valid) {
            return None;
        }
        Some(HoleCursor {
            at: self.source(j),
            chain: names.into_iter().map(str::to_owned).collect(),
            prefix,
        })
    }
}

impl Session {
    /// Whether `byte` is inside a string token: the only place a hole can be, and a question
    /// that needs no compile.
    fn in_string_token(&self, uri: &DocumentUri, byte: usize) -> bool {
        self.analysis_for(uri).is_some_and(|a| {
            let at = a.tokens.partition_point(|t| t.span.end <= byte);
            a.tokens.get(at).is_some_and(|t| {
                t.kind == TokenKind::String && t.span.start < byte && byte < t.span.end
            })
        })
    }

    /// The interpolated literals of the document, from the last check of its text.
    pub(crate) fn interpolation_sites(&self, uri: &DocumentUri) -> Vec<Site> {
        if self.environment.is_none() {
            return Vec::new();
        }
        let (Some(analysis), Ok(doc)) = (self.analysis_for(uri), self.document(uri)) else {
            return Vec::new();
        };
        let facts = self
            .checked(uri, |a, f| a.compiler.ide_interpolations(f))
            .unwrap_or_default();
        facts
            .iter()
            .filter_map(|fact| {
                let string = Span::new(fact.string.start as usize, fact.string.end as usize);
                let at = analysis
                    .tokens
                    .partition_point(|t| t.span.start < string.start);
                let token = analysis.tokens.get(at)?;
                if token.span != string || token.kind != TokenKind::String {
                    return None;
                }
                let Tok::Str(value) = &analysis.syntax_tokens.get(at)?.tok else {
                    return None;
                };
                Site::new(&doc.text, string, value.clone(), fact.mode)
            })
            .collect()
    }

    /// The name inside a hole at `byte`, as the identifier token it is.
    pub(crate) fn hole_word(&self, uri: &DocumentUri, byte: usize) -> Option<(usize, Token)> {
        if !self.in_string_token(uri, byte) {
            return None;
        }
        let sites = self.interpolation_sites(uri);
        let site = sites
            .iter()
            .find(|s| s.string.start < byte && byte < s.string.end)?;
        let hole = site
            .holes
            .iter()
            .find(|h| h.expr.start <= byte && byte <= h.expr.end)?;
        let token = hole
            .tokens
            .iter()
            .find(|t| t.kind == TokenKind::Ident && t.span.start <= byte && byte <= t.span.end)?;
        let analysis = self.analysis_for(uri)?;
        let at = analysis
            .tokens
            .partition_point(|t| t.span.start < site.string.start);
        Some((at, *token))
    }

    /// What a `%` before `byte` in an interpolated literal is completing, if it starts a hole.
    pub(crate) fn hole_cursor(&self, uri: &DocumentUri, byte: usize) -> Option<HoleCursor> {
        if !self.in_string_token(uri, byte) {
            return None;
        }
        self.interpolation_sites(uri)
            .iter()
            .find(|s| s.string.start < byte && byte < s.string.end)?
            .cursor(byte)
    }

    /// Whether `position` is in an interpolated literal's hole expression.
    pub fn in_interpolation_hole(&self, uri: &DocumentUri, position: Position) -> bool {
        let Ok(doc) = self.document(uri) else {
            return false;
        };
        let byte = doc.index.byte(&doc.text, position);
        self.hole_word(uri, byte).is_some()
    }
}
