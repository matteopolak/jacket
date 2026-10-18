//! Locating the call and argument under a cursor.
use crate::intern::Sym;
use crate::lexer::{P, Tok, Token};
use crate::source::Span;

/// An argument of the call an editor's cursor is in.
#[derive(Clone, Debug, PartialEq)]
pub struct CallArgument {
    /// `name = value`.
    pub name: Option<Sym>,
    /// `..value`.
    pub spread: bool,
    /// The value's text, without the name or `..`; `None` when nothing is written yet.
    pub value: Option<Span>,
    /// The value is a lone number (`3`, `-1.5`).
    pub literal: bool,
}

/// The call around a cursor, from tokens alone, so it holds while the call is unfinished.
#[derive(Clone, Debug, PartialEq)]
pub struct CallSite {
    /// The token before the `(`: the callee's name, or the end of a callee expression.
    pub callee: Span,
    /// The token before the `(` is a name (`f(`, `a.f(`), not a closing bracket (`g()(`).
    pub callee_is_name: bool,
    /// The `(`.
    pub open: u32,
    /// Past the `)`, or where the unfinished call stops (a `;`, a closing bracket, or a
    /// declaration on a following line).
    pub end: u32,
    pub args: Vec<CallArgument>,
    /// The argument the cursor is in.
    pub active: usize,
    /// The name being written at the cursor, through a `:` or `:=` typed after it (empty at
    /// the cursor when no name is started).
    pub word: Span,
    /// The cursor's argument already has its `=`.
    pub has_equals: bool,
}

/// The call whose argument list holds `byte`, when the cursor is where an argument name can be
/// written: at the start of an argument, in the name before its `=`, or in a name not followed
/// by anything else. `None` inside comments and literals, at lexical errors, and inside
/// brackets nested in the arguments.
pub fn call_at(text: &str, tokens: &[Token], lex_errors: &[Span], byte: u32) -> Option<CallSite> {
    // Comments and literals are opaque, commas and brackets included. The prefix also finds a
    // literal or comment not closed yet.
    let file = tokens
        .first()
        .map_or(crate::source::FileId(0), |t| t.span.file);
    let regions = crate::lexer::lexical_regions(file, text)
        .or_else(|_| crate::lexer::lexical_regions(file, &text[..byte as usize]))
        .ok()?;
    if regions.iter().any(|s| s.start <= byte && byte < s.end)
        || lex_errors.iter().any(|e| e.start <= byte && byte <= e.end)
    {
        return None;
    }
    let open = enclosing_paren(tokens, byte)?;
    let before = &tokens[open.checked_sub(1)?];
    if !matches!(
        before.tok,
        Tok::Ident(_) | Tok::Punct(P::RParen | P::RBracket | P::RBrace | P::DotStar)
    ) {
        return None;
    }
    let (groups, active, end) = argument_groups(tokens, open, byte, text.len() as u32)?;
    let current = &groups[active];
    let typed: Vec<&Token> = current
        .iter()
        .copied()
        .filter(|t| t.span.start < byte)
        .collect();
    let word = match typed.as_slice() {
        [] => current
            .first()
            .copied()
            .filter(|t| t.span.start == byte && matches!(t.tok, Tok::Ident(_))),
        [
            t @ Token {
                tok: Tok::Ident(_),
                ..
            },
        ] => Some(*t),
        _ => return None,
    };
    let start = word.map_or(byte, |t| t.span.start);
    let mut word_end = word.map_or(byte, |t| t.span.end);
    let separator = current.get(usize::from(word.is_some()));
    let has_equals = separator.is_some_and(|t| t.tok == Tok::Punct(P::Eq));
    match separator.map(|t| &t.tok) {
        None | Some(Tok::Punct(P::Eq)) => {}
        Some(Tok::Punct(P::Colon | P::ColonEq)) => word_end = separator?.span.end,
        Some(_) => return None,
    }
    Some(CallSite {
        callee: before.span,
        callee_is_name: matches!(before.tok, Tok::Ident(_)),
        open: tokens[open].span.start,
        end,
        args: groups.iter().map(|g| call_argument(g)).collect(),
        active,
        word: Span {
            file,
            start,
            end: word_end,
        },
        has_equals,
    })
}

pub(super) fn closer(p: P) -> Option<P> {
    match p {
        P::LParen => Some(P::RParen),
        P::LBrace | P::DotBrace => Some(P::RBrace),
        P::LBracket | P::DotBracket => Some(P::RBracket),
        _ => None,
    }
}

/// The index of the innermost `(` still open at `byte`.
fn enclosing_paren(tokens: &[Token], byte: u32) -> Option<usize> {
    let mut stack: Vec<(usize, P)> = Vec::new();
    for (i, t) in tokens
        .iter()
        .enumerate()
        .take_while(|(_, t)| t.span.start < byte)
    {
        let Tok::Punct(p) = t.tok else {
            continue;
        };
        if let Some(close) = closer(p) {
            stack.push((i, close));
        } else if matches!(p, P::RParen | P::RBrace | P::RBracket) {
            if stack.last().is_some_and(|&(_, close)| close == p) {
                stack.pop();
            } else {
                stack.clear();
            }
        } else if p == P::Semi {
            // A semicolon ends a statement, even after an unfinished call.
            while stack.last().is_some_and(|&(_, close)| close != P::RBrace) {
                stack.pop();
            }
        }
    }
    match stack.last()? {
        &(open, P::RParen) => Some(open),
        _ => None,
    }
}

/// The tokens of each argument after the `(` at `open`, the argument holding `byte`, and where
/// the call ends.
fn argument_groups(
    tokens: &[Token],
    open: usize,
    byte: u32,
    text_end: u32,
) -> Option<(Vec<Vec<&Token>>, usize, u32)> {
    let mut groups: Vec<Vec<&Token>> = vec![Vec::new()];
    let mut nested = Vec::new();
    let mut active = 0;
    for (i, t) in tokens.iter().enumerate().skip(open + 1) {
        if nested.is_empty() && t.span.start >= byte && declaration_starts(tokens, i) {
            // A declaration on a following line is where recovery stops, not more of the
            // unfinished argument.
            return Some((groups, active, t.span.start));
        }
        if let Tok::Punct(p) = t.tok {
            if nested.is_empty() {
                match p {
                    P::RParen => return Some((groups, active, t.span.end)),
                    P::Semi | P::RBrace | P::RBracket => {
                        return Some((groups, active, t.span.start));
                    }
                    // `f(a,, b)` overrides the context: no argument names there.
                    P::Comma if groups.len() > 1 && groups.last()?.is_empty() => return None,
                    P::Comma => {
                        groups.push(Vec::new());
                        if t.span.start < byte {
                            active = groups.len() - 1;
                        }
                        continue;
                    }
                    _ => {}
                }
            }
            if let Some(close) = closer(p) {
                nested.push(close);
            } else if nested.last() == Some(&p) {
                nested.pop();
            }
        }
        if t.tok != Tok::Eof {
            groups.last_mut()?.push(t);
        }
    }
    Some((groups, active, text_end))
}

/// `name :`, `name :=` or `name ::` first on its line.
fn declaration_starts(tokens: &[Token], i: usize) -> bool {
    tokens[i].newline_before
        && matches!(tokens[i].tok, Tok::Ident(_))
        && tokens.get(i + 1).is_some_and(|next| {
            matches!(next.tok, Tok::Punct(P::Colon | P::ColonEq | P::ColonColon))
        })
}

fn call_argument(tokens: &[&Token]) -> CallArgument {
    let name = match tokens {
        [
            Token {
                tok: Tok::Ident(name),
                ..
            },
            Token {
                tok: Tok::Punct(P::Eq),
                ..
            },
            ..,
        ] => Some(*name),
        _ => None,
    };
    let rest = &tokens[(usize::from(name.is_some()) * 2).min(tokens.len())..];
    let spread = rest.first().is_some_and(|t| t.tok == Tok::Punct(P::DotDot));
    let value = &rest[usize::from(spread)..];
    let number = |t: &Token| matches!(t.tok, Tok::Int(_) | Tok::Float(_));
    let sign = |t: &Token| matches!(t.tok, Tok::Punct(P::Minus | P::Plus));
    let literal = match value {
        [n] => number(n),
        [s, n] => sign(s) && number(n),
        _ => false,
    };
    CallArgument {
        name,
        spread,
        value: value
            .first()
            .zip(value.last())
            .map(|(first, last)| first.span.to(last.span)),
        literal,
    }
}
