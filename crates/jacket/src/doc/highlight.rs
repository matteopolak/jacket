//! Syntax colours for Jai code on the site: keywords, types, strings, numbers, comments and
//! `#directives`. It looks at one token at a time, which is enough for ten lines of example.
use super::markdown::esc;

const KEYWORDS: &[&str] = &[
    "if",
    "else",
    "then",
    "for",
    "while",
    "return",
    "defer",
    "using",
    "struct",
    "enum",
    "enum_flags",
    "union",
    "cast",
    "xx",
    "inline",
    "break",
    "continue",
    "case",
    "remove",
    "true",
    "false",
    "null",
    "ifx",
    "push_context",
    "context",
    "operator",
    "interface",
];

const TYPES: &[&str] = &[
    "int", "float", "float32", "float64", "bool", "string", "void", "Type", "Any", "s8", "s16",
    "s32", "s64", "u8", "u16", "u32", "u64", "u128", "s128", "assert",
];

fn ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn span(out: &mut String, class: &str, text: &str) {
    out.push_str("<span class=\"");
    out.push_str(class);
    out.push_str("\">");
    out.push_str(&esc(text));
    out.push_str("</span>");
}

/// `code` as HTML. `link` gives the page of a procedure the example calls, so a call to
/// `trim(...)` can lead to it.
pub fn jai(code: &str, link: &dyn Fn(&str) -> Option<String>) -> String {
    let chars: Vec<(usize, char)> = code.char_indices().collect();
    let mut out = String::with_capacity(code.len() * 2);
    let mut i = 0;
    let at = |n: usize| chars.get(n).map_or(code.len(), |&(b, _)| b);
    while i < chars.len() {
        let (start, c) = chars[i];
        if c == '/' && chars.get(i + 1).is_some_and(|&(_, d)| d == '/') {
            let mut j = i;
            while j < chars.len() && chars[j].1 != '\n' {
                j += 1;
            }
            span(&mut out, "c", &code[start..at(j)]);
            i = j;
        } else if c == '"' {
            let mut j = i + 1;
            while j < chars.len() && chars[j].1 != '"' {
                if chars[j].1 == '\\' {
                    j += 1;
                }
                j += 1;
            }
            let j = (j + 1).min(chars.len());
            span(&mut out, "s", &code[start..at(j)]);
            i = j;
        } else if c == '#' && chars.get(i + 1).is_some_and(|&(_, d)| ident_start(d)) {
            let mut j = i + 1;
            while j < chars.len() && ident_char(chars[j].1) {
                j += 1;
            }
            span(&mut out, "d", &code[start..at(j)]);
            i = j;
        } else if c.is_ascii_digit() {
            let mut j = i;
            while j < chars.len()
                && (chars[j].1.is_ascii_alphanumeric() || chars[j].1 == '_' || chars[j].1 == '.')
            {
                // `1..3` is a range, not a number with dots.
                if chars[j].1 == '.' && chars.get(j + 1).is_some_and(|&(_, d)| d == '.') {
                    break;
                }
                j += 1;
            }
            span(&mut out, "n", &code[start..at(j)]);
            i = j;
        } else if ident_start(c) {
            let mut j = i;
            while j < chars.len() && ident_char(chars[j].1) {
                j += 1;
            }
            let word = &code[start..at(j)];
            let called = chars.get(j).is_some_and(|&(_, d)| d == '(');
            if KEYWORDS.contains(&word) {
                span(&mut out, "k", word);
            } else if TYPES.contains(&word) {
                span(&mut out, "t", word);
            } else if let Some(href) = called.then(|| link(word)).flatten() {
                out.push_str(&format!("<a href=\"{}\">{}</a>", esc(&href), esc(word)));
            } else {
                out.push_str(&esc(word));
            }
            i = j;
        } else {
            out.push_str(&esc(&code[start..at(i + 1)]));
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_get_classes_and_calls_link() {
        let html = jai(
            "found := trim(\"a<b\"); // note\nx: int = 0x1F; #char \"=\";",
            &|name| (name == "trim").then(|| "/trim.html".into()),
        );
        assert!(html.contains("<a href=\"/trim.html\">trim</a>("));
        assert!(html.contains("<span class=\"s\">&quot;a&lt;b&quot;</span>"));
        assert!(html.contains("<span class=\"c\">// note</span>"));
        assert!(html.contains("<span class=\"t\">int</span>"));
        assert!(html.contains("<span class=\"n\">0x1F</span>"));
        assert!(html.contains("<span class=\"d\">#char</span>"));
    }
}
