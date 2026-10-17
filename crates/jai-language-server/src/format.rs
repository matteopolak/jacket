//! Format strings in the calls of a document, found from tokens alone (no type checking), so
//! they work while the text does not parse. A call here is only a candidate: whether its callee
//! takes a format string is the callee's `@PrintLike` note, read by `Session::format_calls`
//! (from the compile, or from this document's own declarations when there is no compile).
//!
//! Formatting follows the bundled `Basic` module: `%` (or `%0`) takes the next argument, `%N`
//! takes argument N (1-based) and a following `%` continues after it, `%00` prints nothing,
//! and `%%` is two arguments in a row. A literal percent sign is written `\%`.
use crate::analysis::Span;
use jaic::lexer::{P, Tok, Token};

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
    /// The name written before the `(` of the call.
    pub callee: String,
    /// The format string literal, quotes included.
    pub string: Span,
    pub specs: Vec<Spec>,
    /// The positional arguments after the format string.
    pub args: Vec<Span>,
    /// A `..spread` argument: how many values are passed is not known from the text.
    pub spread: bool,
}

/// Calls in `text` with a literal format string among their first two arguments.
pub fn calls(tokens: &[Token], text: &str) -> Vec<FormatCall> {
    let mut out = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        let Tok::Ident(name) = &token.tok else {
            continue;
        };
        if !matches!(
            tokens.get(i + 1).map(|t| &t.tok),
            Some(Tok::Punct(P::LParen))
        ) {
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
            callee: name.as_str().to_string(),
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

/// Names this document declares with the `@PrintLike` note: `name :: (...) ... @PrintLike {`.
pub fn local_print_like(tokens: &[Token]) -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    for (i, token) in tokens.iter().enumerate() {
        let Tok::Ident(name) = &token.tok else {
            continue;
        };
        if !matches!(
            (
                tokens.get(i + 1).map(|t| &t.tok),
                tokens.get(i + 2).map(|t| &t.tok)
            ),
            (Some(Tok::Punct(P::ColonColon)), Some(Tok::Punct(P::LParen)))
        ) {
            continue;
        }
        // Past the parameter list, then through the return type to the body.
        let mut depth = 0usize;
        let mut j = i + 2;
        while j < tokens.len() {
            match &tokens[j].tok {
                Tok::Punct(P::LParen) => depth += 1,
                Tok::Punct(P::RParen) => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                Tok::Eof => break,
                _ => {}
            }
            j += 1;
        }
        // Through the return type to the body, past it, and over the notes that follow.
        let mut k = j + 1;
        let (mut parens, mut braces) = (0usize, 0usize);
        let mut after_body = false;
        while k < tokens.len() {
            match &tokens[k].tok {
                Tok::Punct(P::LParen) => parens += 1,
                Tok::Punct(P::RParen) => parens = parens.saturating_sub(1),
                Tok::Punct(P::LBrace) if parens == 0 => braces += 1,
                Tok::Punct(P::RBrace) if braces > 0 => {
                    braces -= 1;
                    if braces == 0 {
                        after_body = true;
                        k += 1;
                        break;
                    }
                }
                Tok::Punct(P::Semi) if braces == 0 => {
                    after_body = true;
                    k += 1;
                    break;
                }
                Tok::Eof => break,
                _ => {}
            }
            k += 1;
        }
        if after_body {
            while let Some(Tok::Note(n)) = tokens.get(k).map(|t| &t.tok) {
                if n.trim() == "PrintLike" {
                    out.insert(name.as_str().to_string());
                }
                k += 1;
            }
        }
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
    // `cast,trunc(u8) x` and `xx,no_check x`: the commas after `cast`/`xx` and between its
    // modifiers do not separate arguments. 1: after `cast`/`xx` or a modifier, 2: after such a comma.
    let mut cast_modifiers = 0u8;
    while i < tokens.len() {
        let modifier_comma = cast_modifiers == 1 && matches!(tokens[i].tok, Tok::Punct(P::Comma));
        cast_modifiers = match &tokens[i].tok {
            Tok::Ident(name) if matches!(name.as_str(), "cast" | "xx") => 1,
            Tok::Punct(P::Comma) if modifier_comma => 2,
            Tok::Ident(_) if cast_modifiers == 2 => 1,
            _ => 0,
        };
        if modifier_comma {
            i += 1;
            continue;
        }
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

/// The `%` directives of the string literal at `string` of `text` (read as `Basic` reads them).
pub fn specs(text: &str, string: Span) -> Vec<Spec> {
    jailint::format_string::specs(text, string.start, string.end)
        .into_iter()
        .map(|s| Spec {
            span: Span::new(s.start, s.end),
            index: s.index,
        })
        .collect()
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

    #[test]
    fn cast_modifier_commas_do_not_split_arguments() {
        let text =
            "f :: () { print(\"% % % %\", cast,trunc(u8) a, xx,no_check b, cast(u8) c, xx d); }";
        let tokens = jaic::lexer::lex(jaic::source::FileId(0), text).unwrap();
        let calls = calls(&tokens, text);
        assert_eq!(calls.len(), 1);
        let args: Vec<&str> = calls[0]
            .args
            .iter()
            .map(|a| &text[a.start..a.end])
            .collect();
        assert_eq!(
            args,
            ["cast,trunc(u8) a", "xx,no_check b", "cast(u8) c", "xx d"]
        );
    }
}
