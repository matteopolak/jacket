//! The lexing entry points, comments and bracket matching.
use super::punct::P;
use super::scan::Lexer;
use super::{Tok, Token};
use crate::source::{Diagnostic, FileId, Span};

pub fn lex(file: FileId, text: &str) -> Result<Vec<Token>, Diagnostic> {
    let mut lx = Lexer {
        file,
        text,
        src: text.as_bytes(),
        at: 0,
        // Jai source averages four to five bytes per token. Reserving once (virtual memory the
        // pages of which are untouched until used) avoids the vector doubling to a 12 MiB
        // block for a 25k-line file and copying at every step.
        out: Vec::with_capacity(text.len() / 4 + 16),
        newline: true,
        regions: None,
        comments: None,
        unclosed: false,
        platform: None,
        braces: Vec::new(),
        opaque_braces: 0,
    };
    lx.run()?;
    Ok(lx.out)
}

/// The platform a library file is read for. File-level `#if` conditions made only of `OS` and
/// `CPU` tests against it are decided while lexing, and the body of a branch they rule out is
/// stepped over byte by byte instead of becoming tokens. The `{` and `}` stay, so the parser
/// sees an empty branch.
#[derive(Clone, Copy, Debug)]
pub struct PlatformTarget {
    pub os: &'static str,
    pub cpu: &'static str,
}

/// `lex`, for a library file (see [`PlatformTarget`]). A dead branch is not checked for lexical
/// errors, so use it only for files the project ships and tests on every platform.
pub fn lex_for_target(
    file: FileId,
    text: &str,
    target: PlatformTarget,
) -> Result<Vec<Token>, Diagnostic> {
    let mut lx = Lexer {
        file,
        text,
        src: text.as_bytes(),
        at: 0,
        out: Vec::with_capacity(text.len() / 4 + 16),
        newline: true,
        regions: None,
        comments: None,
        unclosed: false,
        platform: Some(target),
        braces: Vec::new(),
        opaque_braces: 0,
    };
    lx.run()?;
    Ok(lx.out)
}

/// Editor-only lexing. Retain good tokens, report lexical errors, and restart after
/// the malformed literal's line. Offsets always refer to the original UTF-8 text.
pub fn lex_ide(file: FileId, text: &str) -> (Vec<Token>, Vec<Diagnostic>) {
    let mut lx = Lexer {
        file,
        text,
        src: text.as_bytes(),
        at: 0,
        out: Vec::with_capacity(text.len() / 4 + 16),
        newline: true,
        regions: None,
        comments: None,
        unclosed: false,
        platform: None,
        braces: Vec::new(),
        opaque_braces: 0,
    };
    let errors = lx.recover();
    (lx.out, errors)
}

/// A comment found by `lex_with_trivia`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Comment {
    /// The whole comment, delimiters included. A line comment stops before its newline.
    pub span: Span,
    pub kind: CommentKind,
    /// False for a block comment the file ends inside (the span then runs to the end).
    pub terminated: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommentKind {
    Line,
    Block,
}

impl Comment {
    /// The comment's text without its delimiters.
    pub fn body<'t>(&self, text: &'t str) -> &'t str {
        let (s, e) = (self.span.start as usize, self.span.end as usize);
        match self.kind {
            CommentKind::Line => &text[s + 2..e],
            CommentKind::Block if self.terminated => &text[s + 2..e - 2],
            CommentKind::Block => &text[s + 2..e],
        }
    }
}

/// Editor lexing (as `lex_ide`) that also returns every comment, in source order. The token
/// stream is exactly `lex_ide`'s; comments inside strings are never reported, and an
/// unterminated block comment is returned with `terminated: false` besides its error.
pub fn lex_with_trivia(file: FileId, text: &str) -> (Vec<Token>, Vec<Comment>, Vec<Diagnostic>) {
    let mut lx = Lexer {
        file,
        text,
        src: text.as_bytes(),
        at: 0,
        out: Vec::with_capacity(text.len() / 4 + 16),
        newline: true,
        regions: None,
        comments: Some(Vec::new()),
        unclosed: false,
        platform: None,
        braces: Vec::new(),
        opaque_braces: 0,
    };
    let errors = lx.recover();
    (lx.out, lx.comments.unwrap_or_default(), errors)
}

/// Verified literal/comment boundaries for incremental IDE lexical recovery.
/// No tokens or semantic facts from an older version are reused.
pub fn lexical_regions(file: FileId, text: &str) -> Result<Vec<Span>, Diagnostic> {
    let mut lx = Lexer {
        file,
        text,
        src: text.as_bytes(),
        at: 0,
        out: Vec::new(),
        newline: true,
        regions: Some(Vec::new()),
        comments: None,
        unclosed: false,
        platform: None,
        braces: Vec::new(),
        opaque_braces: 0,
    };
    lx.run()?;
    Ok(lx.regions.unwrap())
}

/// `span` widened to balance its parentheses: the parser leaves grouping parentheses out of
/// an expression's span, so `(a + b) * c` spans `a + b) * c` and `a && (b` ends early.
/// `tokens` are the file's, from `lex`.
pub fn balanced(tokens: &[Token], span: Span) -> Span {
    let first = tokens.partition_point(|t| t.span.start < span.start);
    let last = tokens.partition_point(|t| t.span.end <= span.end);
    let (mut depth, mut lowest) = (0i32, 0i32);
    for t in &tokens[first..last.max(first)] {
        match t.tok {
            Tok::Punct(P::LParen) => depth += 1,
            Tok::Punct(P::RParen) => {
                depth -= 1;
                lowest = lowest.min(depth);
            }
            _ => {}
        }
    }
    let mut out = span;
    // Openers before the span for the closers it has too many of.
    let mut need = -lowest;
    for t in tokens[..first].iter().rev() {
        if need == 0 {
            break;
        }
        if matches!(t.tok, Tok::Punct(P::LParen)) {
            need -= 1;
            out.start = t.span.start;
        }
    }
    // Closers after it for the openers it leaves open.
    let mut open = depth - lowest;
    for t in &tokens[last.min(tokens.len())..] {
        if open == 0 {
            break;
        }
        if matches!(t.tok, Tok::Punct(P::RParen)) {
            open -= 1;
            out.end = t.span.end;
        }
    }
    out
}
