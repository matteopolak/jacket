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

/// `text` as a Jai string literal.
pub(super) fn quote(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\0' => out.push_str("\\0"),
            '%' => out.push_str("\\%"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
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

/// A substituted argument needs parentheses unless it is one token-like atom.
fn atom(text: &str) -> bool {
    let simple = |c: char| c.is_alphanumeric() || c == '_' || c == '.';
    if text.chars().all(simple) {
        return true;
    }
    if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        return !text[1..text.len() - 1].contains('"');
    }
    // `name(...)` / `(...)` with one outer pair of parentheses.
    let open = text.find('(');
    if let Some(open) = open
        && text.ends_with(')')
        && text[..open].chars().all(simple)
    {
        let mut depth = 0i32;
        for (i, c) in text[open..].char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        return open + i == text.len() - 1;
                    }
                }
                _ => {}
            }
        }
    }
    false
}

/// `source` (a macro body) with each parameter replaced by its argument's text: `#insert c` of
/// a `Code` parameter by the code, `..args` of a variadic one by the arguments, and the
/// backtick of a caller-scope name dropped (after expansion it is the caller's name).
pub(super) fn substitute(
    source: &str,
    subst: &HashMap<Sym, (String, bool, bool)>,
) -> Option<String> {
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
                    && let Some((text, true, _)) = subst.get(n)
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
                    && let Some((text, false, true)) = subst.get(n)
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
                    && let Some((text, code, variadic)) = subst.get(n)
                    && !code
                {
                    out.push_str(&source[copied..start]);
                    if *variadic {
                        out.push_str(&format!(".[{text}]"));
                    } else if atom(text) {
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
        subst.insert(Sym::intern("a"), ("x + 1".to_string(), false, false));
        subst.insert(Sym::intern("b"), ("y".to_string(), false, false));
        subst.insert(
            Sym::intern("body"),
            ("print(\"hi\");".to_string(), true, false),
        );
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

    #[test]
    fn atoms_need_no_parentheses() {
        assert!(atom("x"));
        assert!(atom("a.b"));
        assert!(atom("f(1, g(2))"));
        assert!(atom("\"text\""));
        assert!(!atom("x + 1"));
        assert!(!atom("f(1) + g(2)"));
    }
}
