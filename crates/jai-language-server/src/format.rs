//! Format strings of print-family calls, found from tokens alone (no type checking), so they
//! work while the text does not parse.
//!
//! Formatting follows the bundled `Basic` module: `%` (or `%0`) takes the next argument, `%N`
//! takes argument N (1-based) and a following `%` continues after it, `%00` prints nothing,
//! and `%%` is two arguments in a row. A literal percent sign is written `\%`.
use crate::analysis::Span;
use jaic::lexer::{P, Tok, Token};

/// Procedures whose first string-literal argument (among the first two) is a format string.
pub const PRINT_FAMILY: &[&str] = &[
    "print",
    "sprint",
    "tprint",
    "print_to_builder",
    "print_color",
    "log",
    "log_error",
    "log_warning",
    "assert",
];

/// One `%` directive of a format string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spec {
    pub span: Span,
    /// The argument it formats (0-based among the arguments after the format string), or
    /// `None` for `%00`.
    pub index: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct FormatCall {
    pub callee: String,
    /// The format string literal, quotes included.
    pub string: Span,
    pub specs: Vec<Spec>,
    /// The positional arguments after the format string.
    pub args: Vec<Span>,
    /// A `..spread` argument: how many values are passed is not known from the text.
    pub spread: bool,
}

impl FormatCall {
    /// Arguments the format string uses.
    pub fn required(&self) -> usize {
        self.specs
            .iter()
            .filter_map(|s| s.index)
            .map(|i| i + 1)
            .max()
            .unwrap_or(0)
    }
}

/// Print-family calls of `text` with a literal format string.
pub fn calls(tokens: &[Token], text: &str) -> Vec<FormatCall> {
    let mut out = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        let Tok::Ident(name) = &token.tok else {
            continue;
        };
        if !PRINT_FAMILY.contains(&name.as_str())
            || !matches!(tokens.get(i + 1).map(|t| &t.tok), Some(Tok::Punct(P::LParen)))
            // `x.print(` or a declaration `print :: (`.
            || matches!(
                i.checked_sub(1).map(|p| &tokens[p].tok),
                Some(Tok::Punct(P::Dot | P::ColonColon))
            )
        {
            continue;
        }
        let args = arguments(tokens, i + 2);
        // The format string: a lone string literal among the first two arguments.
        let Some(at) = args.iter().take(2).position(|a| {
            a.named.is_none()
                && a.tokens.len() == 1
                && matches!(tokens[a.tokens.start].tok, Tok::Str(_))
                && text[span(&tokens[a.tokens.start]).start..].starts_with('"')
        }) else {
            continue;
        };
        let string = span(&tokens[args[at].tokens.start]);
        let rest = &args[at + 1..];
        out.push(FormatCall {
            callee: name.as_str().into(),
            string,
            specs: specs(text, string),
            args: rest
                .iter()
                .filter(|a| a.named.is_none())
                .map(|a| a.span)
                .collect(),
            spread: rest.iter().any(|a| a.spread),
        });
    }
    out
}

fn span(token: &Token) -> Span {
    Span::new(token.span.start as usize, token.span.end as usize)
}

struct Argument {
    tokens: std::ops::Range<usize>,
    span: Span,
    named: Option<String>,
    spread: bool,
}

/// The arguments of a call whose first argument token is at `from`, up to its `)` or a `,,`
/// (context arguments follow that).
fn arguments(tokens: &[Token], from: usize) -> Vec<Argument> {
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut start = from;
    let mut i = from;
    let push = |start: usize, end: usize, out: &mut Vec<Argument>| {
        if start >= end {
            return false;
        }
        let named = match (&tokens[start].tok, tokens.get(start + 1).map(|t| &t.tok)) {
            (Tok::Ident(n), Some(Tok::Punct(P::Eq))) => Some(n.as_str().to_string()),
            _ => None,
        };
        out.push(Argument {
            tokens: start..end,
            span: Span::new(
                tokens[start].span.start as usize,
                tokens[end - 1].span.end as usize,
            ),
            named,
            spread: matches!(tokens[start].tok, Tok::Punct(P::DotDot)),
        });
        true
    };
    while i < tokens.len() {
        match &tokens[i].tok {
            Tok::Punct(P::LParen | P::LBracket | P::LBrace | P::DotBrace | P::DotBracket) => {
                depth += 1
            }
            Tok::Punct(P::RParen | P::RBracket | P::RBrace) => {
                if depth == 0 {
                    push(start, i, &mut out);
                    return out;
                }
                depth -= 1;
            }
            Tok::Punct(P::Comma) if depth == 0 => {
                if !push(start, i, &mut out) {
                    // `,,`: context overrides follow.
                    return out;
                }
                start = i + 1;
            }
            // A statement ended: the call is unfinished.
            Tok::Punct(P::Semi) if depth == 0 => return out,
            Tok::Eof => return out,
            _ => {}
        }
        i += 1;
    }
    out
}

/// The `%` directives of the string literal at `string` of `text`.
pub fn specs(text: &str, string: Span) -> Vec<Spec> {
    let bytes = text.as_bytes();
    let end = string.end.saturating_sub(1).min(bytes.len());
    let mut at = string.start + 1;
    let mut implicit = 0usize;
    let mut out = Vec::new();
    while at < end {
        match bytes[at] {
            b'\\' => at += 2,
            b'%' => {
                let start = at;
                at += 1;
                if at + 1 < end && bytes[at] == b'0' && bytes[at + 1] == b'0' {
                    at += 2;
                    out.push(Spec {
                        span: Span::new(start, at),
                        index: None,
                    });
                    continue;
                }
                let index = if at < end && bytes[at].is_ascii_digit() && bytes[at] != b'0' {
                    let mut n = 0usize;
                    while at < end && bytes[at].is_ascii_digit() {
                        n = n
                            .saturating_mul(10)
                            .saturating_add((bytes[at] - b'0') as usize);
                        at += 1;
                    }
                    n.saturating_sub(1)
                } else {
                    if at < end && bytes[at] == b'0' {
                        at += 1;
                    }
                    implicit
                };
                implicit = index.saturating_add(1);
                out.push(Spec {
                    span: Span::new(start, at),
                    index: Some(index),
                });
            }
            _ => at += 1,
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn indices(source: &str) -> Vec<Option<usize>> {
        let text = format!("\"{source}\"");
        specs(&text, Span::new(0, text.len()))
            .into_iter()
            .map(|s| s.index)
            .collect()
    }

    #[test]
    fn directives_follow_basic_print() {
        assert_eq!(indices("a % b %"), [Some(0), Some(1)]);
        assert_eq!(indices("%2 %1 %"), [Some(1), Some(0), Some(1)]);
        assert_eq!(indices("%%"), [Some(0), Some(1)]);
        assert_eq!(indices("100\\%"), []);
        assert_eq!(indices("%00x%0"), [None, Some(0)]);
    }
}
