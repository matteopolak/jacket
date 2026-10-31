//! Text helpers for expansions: bounding, quoting, dedenting and substitution.
use super::MAX_TEXT;
use crate::fxhash::HashMap;
use crate::intern::Sym;
use crate::lexer::{P, Tok};
use crate::source::FileId;

pub(super) fn bounded(mut text: String) -> String {
    if text.len() > MAX_TEXT {
        let end = text.floor_char_boundary(MAX_TEXT);
        text.truncate(end);
        text.push('…');
    }
    text
}

/// `text` without the indentation its lines share (the first line, often unindented after a
/// brace, does not count).
pub fn dedent(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let indent = lines
        .iter()
        .skip(1)
        .chain(lines.first())
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    lines
        .iter()
        .map(|l| {
            if l.len() >= indent && l[..indent].trim().is_empty() {
                &l[indent..]
            } else {
                l.trim_start()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string()
}

/// An argument of a macro call, as text to put in the body.
pub(super) struct Argument {
    pub text: String,
    /// A `Code` parameter: `#insert` takes it as it is.
    pub code: bool,
    /// The variadic parameter: its arguments are listed in an array.
    pub variadic: bool,
    /// The argument reads as one unit (`Expr::is_atomic`): it needs no parentheses around it.
    pub atomic: bool,
}

/// `source` (a macro body) with each parameter replaced by its argument's text: `#insert c` of
/// a `Code` parameter by the code, `..args` of a variadic one by the arguments, and the
/// backtick of a caller-scope name dropped (after expansion it is the caller's name).
pub(super) fn substitute(source: &str, subst: &HashMap<Sym, Argument>) -> Option<String> {
    let tokens = crate::lexer::lex(FileId(0), source).ok()?;
    let mut out = String::with_capacity(source.len());
    let mut copied = 0usize;
    let mut i = 0;
    let keyword = |s: Sym| {
        matches!(
            s.as_str(),
            "return" | "defer" | "break" | "continue" | "remove"
        )
    };
    while i < tokens.len() {
        let t = &tokens[i];
        let (start, end) = (t.span.start as usize, t.span.end as usize);
        let prev = i.checked_sub(1).map(|p| &tokens[p].tok);
        let next = tokens.get(i + 1).map(|n| &n.tok);
        match &t.tok {
            // `#insert code_param`: the code itself.
            Tok::Directive(d) if d.as_str() == "insert" => {
                if let Some(Tok::Ident(n)) = next
                    && let Some(Argument {
                        text,
                        code: true,
                        ..
                    }) = subst.get(n)
                {
                    out.push_str(&source[copied..start]);
                    out.push_str(text);
                    copied = tokens[i + 1].span.end as usize;
                    i += 2;
                    continue;
                }
            }
            Tok::Punct(P::Backtick) => {
                if let Some(Tok::Ident(n)) = next
                    && !keyword(*n)
                {
                    out.push_str(&source[copied..start]);
                    copied = end;
                }
            }
            Tok::Punct(P::DotDot) => {
                if let Some(Tok::Ident(n)) = next
                    && let Some(Argument {
                        text,
                        code: false,
                        variadic: true,
                        ..
                    }) = subst.get(n)
                {
                    out.push_str(&source[copied..start]);
                    out.push_str(text);
                    copied = tokens[i + 1].span.end as usize;
                    i += 2;
                    continue;
                }
            }
            Tok::Ident(n) => {
                let member = matches!(prev, Some(Tok::Punct(P::Dot | P::Backtick)));
                let declared = matches!(
                    next,
                    Some(Tok::Punct(P::ColonEq | P::ColonColon | P::Colon))
                ) && !matches!(prev, Some(Tok::Punct(P::Dot)));
                if !member
                    && !declared
                    && let Some(Argument {
                        text,
                        code: false,
                        variadic,
                        atomic,
                    }) = subst.get(n)
                {
                    out.push_str(&source[copied..start]);
                    if *variadic {
                        out.push_str(&format!(".[{text}]"));
                    } else if *atomic {
                        out.push_str(text);
                    } else {
                        out.push('(');
                        out.push_str(text);
                        out.push(')');
                    }
                    copied = end;
                }
            }
            _ => {}
        }
        i += 1;
    }
    out.push_str(&source[copied.min(source.len())..]);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitution_replaces_parameters_and_drops_caller_backticks() {
        let mut subst = HashMap::default();
        let arg = |text: &str, code: bool, atomic: bool| Argument {
            text: text.to_string(),
            code,
            variadic: false,
            atomic,
        };
        subst.insert(Sym::intern("a"), arg("x + 1", false, false));
        subst.insert(Sym::intern("b"), arg("y", false, true));
        subst.insert(Sym::intern("body"), arg("print(\"hi\");", true, false));
        let text = substitute(
            "{ t := a; `total = t * b; p.a = 2; #insert body; `return; }",
            &subst,
        )
        .unwrap();
        assert_eq!(
            text,
            "{ t := (x + 1); total = t * y; p.a = 2; print(\"hi\");; `return; }"
        );
    }

    #[test]
    fn dedent_keeps_relative_indentation() {
        assert_eq!(
            dedent("    a;\n    if b {\n        c;\n    }"),
            "a;\nif b {\n    c;\n}"
        );
    }
}
