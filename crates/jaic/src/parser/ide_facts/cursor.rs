//! What surrounds a cursor position: strings, member accesses and declarations.
use super::call_site::closer;
use crate::lexer::{Lexed, P, Tok, Token};
use crate::source::Span;

/// The directive a string literal is the argument of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathDirective {
    /// `#import "Module"`, `#import,file "path"`.
    Import,
    /// `#load "file.jai"`.
    Load,
}

/// A string literal around the cursor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StringAt {
    /// The literal with its quotes; an unclosed one runs to the end of its line.
    pub span: Span,
    /// Where the text inside the quotes starts: just past the opening quote.
    pub start: u32,
    pub closed: bool,
    /// The `#import` / `#load` the literal names a path for.
    pub directive: Option<PathDirective>,
}

/// A member access `receiver.word` with the cursor in `word` (possibly empty).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemberAt {
    /// The `.`.
    pub dot: u32,
    /// The name being written, empty at the cursor when none is started.
    pub word: Span,
    /// The expression before the dot; `None` when the dot starts an inferred member
    /// (`.Name` where the context decides the type: `paint(.`, `c == .`).
    pub receiver: Option<Span>,
    /// The names of `a.b.c` when the receiver is such a chain; empty for any other receiver
    /// (a call, an index, a number), which has nothing to offer without evaluating it.
    pub chain: Vec<Span>,
}

/// What the cursor is in, read from the tokens of its file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CursorContext {
    /// Code none of the other cases describes.
    Code,
    /// A line or block comment (an unclosed block runs to the end of the file).
    Comment(Span),
    String(StringAt),
    Member(MemberAt),
    /// One of the names a declaration introduces (`a`, `b` in `a, b := f()`), before its `:`.
    DeclName {
        /// Every name of the declaration.
        names: Vec<Span>,
        /// The index of the one holding the cursor.
        at: usize,
    },
}

/// What `offset` is in. Built from `lex_with_trivia`, so a literal or comment that is not
/// closed yet, or other broken text, still gives an answer; quotes after escapes, comment
/// delimiters inside strings and quotes inside comments are told apart by the lexer, not by
/// looking at bytes.
pub fn cursor_context(text: &str, offset: u32) -> CursorContext {
    cursor_context_in(text, offset, &Lexed::new(crate::source::FileId(0), text))
}

/// [`cursor_context`] over `text` lexed already (as `Lexed::new` with file 0).
pub fn cursor_context_in(text: &str, offset: u32, lexed: &Lexed) -> CursorContext {
    let file = crate::source::FileId(0);
    let Lexed {
        tokens,
        comments,
        errors,
    } = lexed;
    if let Some(c) = comments.iter().find(|c| {
        c.span.start < offset
            && (offset < c.span.end
                || (offset == c.span.end
                    && (!c.terminated || c.kind == crate::lexer::CommentKind::Line)))
    }) {
        return CursorContext::Comment(c.span);
    }
    let literal = tokens
        .iter()
        .filter(|t| matches!(t.tok, Tok::Str(_)))
        .map(|t| (t.span, true))
        .chain(errors.iter().filter_map(|e| {
            // An unclosed literal stops at its line's end; so does a bad escape's error.
            (text.as_bytes().get(e.span.start as usize) == Some(&b'"')).then_some((e.span, false))
        }))
        .find(|(span, closed)| {
            span.start < offset
                && if *closed {
                    offset < span.end
                } else {
                    offset <= span.end
                }
        });
    if let Some((span, closed)) = literal {
        let before = tokens.partition_point(|t| t.span.end <= span.start);
        return CursorContext::String(StringAt {
            span,
            start: span.start + 1,
            closed,
            directive: path_directive(&tokens[..before]),
        });
    }
    // The name the cursor is in or at the end of, else the position between tokens.
    let hit = tokens.iter().position(|t| {
        matches!(t.tok, Tok::Ident(_)) && t.span.start <= offset && offset <= t.span.end
    });
    let first = hit.unwrap_or_else(|| tokens.partition_point(|t| t.span.end <= offset));
    let word = hit.map_or(
        Span {
            file,
            start: offset,
            end: offset,
        },
        |i| tokens[i].span,
    );
    if let Some(dot) = first
        .checked_sub(1)
        .filter(|&d| tokens[d].tok == Tok::Punct(P::Dot))
    {
        let (receiver, chain) = receiver_before(tokens, dot);
        return CursorContext::Member(MemberAt {
            dot: tokens[dot].span.start,
            word,
            receiver,
            chain,
        });
    }
    if let Some(i) = hit
        && let Some(names) = declared_names(tokens, i)
    {
        let at = names
            .iter()
            .position(|n| n.start == tokens[i].span.start)
            .unwrap_or(0);
        return CursorContext::DeclName {
            names,
            at,
        };
    }
    CursorContext::Code
}

/// The `#import` / `#load` a string at the end of `before` is the argument of, past options
/// such as `,file` or `,dir`.
fn path_directive(before: &[Token]) -> Option<PathDirective> {
    for t in before.iter().rev().take(8) {
        match &t.tok {
            Tok::Punct(P::Comma) | Tok::Ident(_) => {}
            Tok::Directive(name) => {
                return match name.as_str() {
                    "import" => Some(PathDirective::Import),
                    "load" => Some(PathDirective::Load),
                    _ => None,
                };
            }
            _ => return None,
        }
    }
    None
}

/// Whether `t` can end an expression a `.` follows.
fn ends_expression(t: &Tok) -> bool {
    // `#filepath.count`, `#Context.name`: directives that stand for a value.
    if let Tok::Directive(d) = t {
        return matches!(
            d.as_str(),
            "filepath" | "file" | "line" | "this" | "Context"
        );
    }
    matches!(
        t,
        Tok::Ident(_)
            | Tok::Str(_)
            | Tok::Int(_)
            | Tok::Float(_)
            | Tok::Punct(P::RParen | P::RBracket | P::RBrace | P::DotStar)
    )
}

/// The receiver of the `.` at `tokens[dot]`: its span, and the names when it is `a.b.c`.
/// Nothing is a receiver unless it touches the dot, so `case .` and `== .` are inferred members.
fn receiver_before(tokens: &[Token], dot: usize) -> (Option<Span>, Vec<Span>) {
    // `#Context .name` is written with a space, so the dot is not taken for a member of `#`.
    let touches = |i: usize| {
        tokens[i].span.end == tokens[i + 1].span.start
            || matches!(&tokens[i].tok, Tok::Directive(d) if d.as_str() == "Context")
    };
    let Some(mut i) = dot
        .checked_sub(1)
        .filter(|&i| touches(i) && ends_expression(&tokens[i].tok))
    else {
        return (None, Vec::new());
    };
    let last = tokens[i].span;
    let mut chain = Vec::new();
    let mut pure = true;
    let start = loop {
        match tokens[i].tok {
            Tok::Ident(_) => {
                chain.push(tokens[i].span);
                match i.checked_sub(2) {
                    Some(j)
                        if tokens[i - 1].tok == Tok::Punct(P::Dot)
                            && touches(i - 1)
                            && touches(j)
                            && ends_expression(&tokens[j].tok) =>
                    {
                        i = j;
                    }
                    _ => break tokens[i].span.start,
                }
            }
            Tok::Punct(P::RParen | P::RBracket | P::RBrace) => {
                pure = false;
                let Some(open) = matching_open(tokens, i) else {
                    break tokens[i].span.start;
                };
                match open.checked_sub(1) {
                    Some(before)
                        if touches(before) && matches!(tokens[before].tok, Tok::Ident(_)) =>
                    {
                        i = before;
                    }
                    _ => break tokens[open].span.start,
                }
            }
            Tok::Punct(P::DotStar) => {
                pure = false;
                match i
                    .checked_sub(1)
                    .filter(|&j| touches(j) && ends_expression(&tokens[j].tok))
                {
                    Some(j) => i = j,
                    None => break tokens[i].span.start,
                }
            }
            _ => {
                pure = false;
                break tokens[i].span.start;
            }
        }
    };
    chain.reverse();
    let chain = if pure {
        chain
    } else {
        Vec::new()
    };
    (
        Some(Span {
            start,
            end: last.end,
            ..last
        }),
        chain,
    )
}

/// The index of the bracket that opens the closer at `close`.
fn matching_open(tokens: &[Token], close: usize) -> Option<usize> {
    let mut depth = 0usize;
    for i in (0..=close).rev() {
        let Tok::Punct(p) = tokens[i].tok else {
            continue;
        };
        if matches!(p, P::RParen | P::RBracket | P::RBrace) {
            depth += 1;
        } else if closer(p).is_some() {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

/// The names `a, b` of the declaration whose list holds the name at `tokens[at]`, when the
/// list starts its statement and is followed by `:`, `:=` or `::`.
fn declared_names(tokens: &[Token], at: usize) -> Option<Vec<Span>> {
    let mut first = at;
    while first >= 2
        && tokens[first - 1].tok == Tok::Punct(P::Comma)
        && matches!(tokens[first - 2].tok, Tok::Ident(_))
    {
        first -= 2;
    }
    let starts_statement = first == 0
        || tokens[first].newline_before
        || matches!(
            tokens[first - 1].tok,
            Tok::Punct(P::Semi | P::LBrace | P::RBrace)
        );
    if !starts_statement || !matches!(tokens[first].tok, Tok::Ident(_)) {
        return None;
    }
    let mut names = vec![tokens[first].span];
    let mut i = first + 1;
    while tokens.get(i).is_some_and(|t| t.tok == Tok::Punct(P::Comma))
        && tokens
            .get(i + 1)
            .is_some_and(|t| matches!(t.tok, Tok::Ident(_)))
    {
        names.push(tokens[i + 1].span);
        i += 2;
    }
    matches!(
        tokens.get(i)?.tok,
        Tok::Punct(P::Colon | P::ColonEq | P::ColonColon)
    )
    .then_some(names)
}
