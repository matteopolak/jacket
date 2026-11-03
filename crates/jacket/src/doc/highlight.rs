//! Syntax colours for code on the site. Jai: keywords, types, strings, numbers, comments and
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
    tokens(code, &|w| link(w).map(|h| (w.len(), h)), Names::Called)
}

/// A short expression as HTML, such as a default value: `link` is asked about every name, not
/// only the called ones.
pub fn expr(code: &str, link: &dyn Fn(&str) -> Option<String>) -> String {
    tokens(code, &|w| link(w).map(|h| (w.len(), h)), Names::Every)
}

/// A declaration as written: `link` is asked about each written path (`Hash.Table`) that is not
/// being declared, and answers how much of its start (`Hash.Table`, or only `Hash`) leads to a
/// page, and where.
pub fn declaration(code: &str, link: &dyn Fn(&str) -> Option<(usize, String)>) -> String {
    tokens(code, link, Names::Paths)
}

/// Which names `tokens` offers to its resolver.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Names {
    /// Those a call is made to.
    Called,
    /// All of them.
    Every,
    /// All of them as paths, except the ones the code declares.
    Paths,
}

fn tokens(code: &str, link: &dyn Fn(&str) -> Option<(usize, String)>, names: Names) -> String {
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
            if KEYWORDS.contains(&word) {
                span(&mut out, "k", word);
                i = j;
                continue;
            }
            if TYPES.contains(&word) {
                span(&mut out, "t", word);
                i = j;
                continue;
            }
            // The rest of a written path: `Hash.Table`.
            while names == Names::Paths
                && chars.get(j).is_some_and(|&(_, d)| d == '.')
                && chars.get(j + 1).is_some_and(|&(_, d)| ident_start(d))
            {
                j += 1;
                while j < chars.len() && ident_char(chars[j].1) {
                    j += 1;
                }
            }
            let path = &code[start..at(j)];
            let declared = code[at(j)..]
                .trim_start_matches([' ', '\t'])
                .starts_with(':');
            let wanted = match names {
                Names::Called => chars.get(j).is_some_and(|&(_, d)| d == '('),
                Names::Every => true,
                Names::Paths => !declared,
            };
            match wanted.then(|| link(path)).flatten() {
                Some((len, href)) if len <= path.len() && path.is_char_boundary(len) => {
                    out.push_str(&format!(
                        "<a href=\"{}\">{}</a>{}",
                        esc(&href),
                        esc(&path[..len]),
                        esc(&path[len..])
                    ));
                }
                _ => out.push_str(&esc(path)),
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
    fn declarations_link_written_paths_but_not_what_they_declare() {
        let html = declaration(
            "Box :: struct(T: Type) {\n    next: *Box(T);\n    table: Hash.Table(int, T);\n    tag: Hash.Kind.ONE;\n}",
            &|path| match path {
                "Box" => Some((3, "/Box/".into())),
                "Hash.Table" => Some((10, "/Table/".into())),
                "Hash.Kind.ONE" => Some((9, "/Kind/".into())),
                _ => None,
            },
        );
        // `Box ::` declares, `*Box(T)` names.
        assert!(!html.contains("Box</a> ::"), "{html}");
        assert!(html.contains("*<a href=\"/Box/\">Box</a>(T)"), "{html}");
        assert!(
            html.contains("<a href=\"/Table/\">Hash.Table</a>("),
            "{html}"
        );
        assert!(
            html.contains("<a href=\"/Kind/\">Hash.Kind</a>.ONE"),
            "{html}"
        );
        // The field named like a type is not linked either.
        assert!(html.contains("table: "), "{html}");
        assert!(html.contains("<span class=\"k\">struct</span>"), "{html}");
    }

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

/// `code` as HTML when `lang` is a shell or TOML; `None` for other languages, which stay plain.
pub fn other(lang: &str, code: &str) -> Option<String> {
    match lang {
        "sh" | "bash" | "shell" | "zsh" | "console" | "powershell" | "ps1" | "pwsh" => {
            Some(code.lines().map(shell_line).collect::<Vec<_>>().join("\n"))
        }
        "toml" => Some(code.lines().map(toml_line).collect::<Vec<_>>().join("\n")),
        _ => None,
    }
}

/// The end of a quoted string starting at `start` (the quote), past the closing quote.
fn quoted(chars: &[char], start: usize) -> usize {
    let quote = chars[start];
    let mut i = start + 1;
    while i < chars.len() && chars[i] != quote {
        if chars[i] == '\\' && quote == '"' {
            i += 1;
        }
        i += 1;
    }
    (i + 1).min(chars.len())
}

/// One line of a shell session: the command name of each pipeline step, flags, variables, strings
/// and comments. A leading `$ ` prompt is kept as written.
fn shell_line(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let text = |a: usize, b: usize| chars[a..b].iter().collect::<String>();
    let mut out = String::new();
    let mut i = 0;
    let mut command = true;
    while i < chars.len() {
        let c = chars[i];
        let at_word = i == 0 || chars[i - 1].is_whitespace();
        if c == '#' && at_word {
            span(&mut out, "c", &text(i, chars.len()));
            break;
        }
        if c == '"' || c == '\'' {
            let end = quoted(&chars, i);
            span(&mut out, "s", &text(i, end));
            i = end;
            command = false;
            continue;
        }
        if c == '$'
            && chars
                .get(i + 1)
                .is_some_and(|n| ident_start(*n) || *n == '{')
        {
            let mut end = i + 1;
            while end < chars.len() && (ident_char(chars[end]) || matches!(chars[end], '{' | '}')) {
                end += 1;
            }
            span(&mut out, "n", &text(i, end));
            // `FOO=$HOME cmd`: the value of an assignment comes before the command.
            command = command && i > 0 && chars[i - 1] == '=';
            i = end;
            continue;
        }
        if c == '|' || c == ';' || c == '&' {
            out.push_str(&esc(&c.to_string()));
            command = true;
            i += 1;
            continue;
        }
        if c.is_whitespace() {
            out.push(c);
            i += 1;
            continue;
        }
        let mut end = i;
        while end < chars.len()
            && !chars[end].is_whitespace()
            && !matches!(chars[end], '|' | ';' | '"' | '\'' | '$')
        {
            end += 1;
        }
        let word = text(i, end);
        if command && word == "$" {
            out.push_str(&esc(&word));
        } else if command && !word.contains('=') {
            span(&mut out, "k", &word);
            command = false;
        } else if word.starts_with('-') && at_word {
            span(&mut out, "d", &word);
        } else {
            out.push_str(&esc(&word));
        }
        i = end;
    }
    out
}

/// One line of TOML: table headers, keys, strings, numbers, booleans and comments.
fn toml_line(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let text = |a: usize, b: usize| chars[a..b].iter().collect::<String>();
    let mut out = String::new();
    let trimmed = line.trim_start();
    let lead = chars.len() - trimmed.chars().count();
    out.push_str(&text(0, lead));
    let mut i = lead;
    if trimmed.starts_with('[') {
        let end = chars[i..]
            .iter()
            .position(|&c| c == ']')
            .map_or(chars.len(), |p| {
                let mut e = i + p + 1;
                if chars.get(e) == Some(&']') {
                    e += 1;
                }
                e
            });
        span(&mut out, "t", &text(i, end));
        i = end;
    } else if let Some(eq) = chars[i..].iter().position(|&c| c == '=') {
        let key = text(i, i + eq);
        let name = key.trim_end();
        if !name.is_empty() && !name.starts_with('#') {
            span(&mut out, "d", name);
            out.push_str(&key[name.len()..]);
            i += eq;
        }
    }
    while i < chars.len() {
        let c = chars[i];
        if c == '#' {
            span(&mut out, "c", &text(i, chars.len()));
            break;
        }
        if c == '"' || c == '\'' {
            let end = quoted(&chars, i);
            span(&mut out, "s", &text(i, end));
            i = end;
            continue;
        }
        if c.is_ascii_digit()
            || ((c == '-' || c == '+') && chars.get(i + 1).is_some_and(char::is_ascii_digit))
        {
            let mut end = i + 1;
            while end < chars.len()
                && (chars[end].is_ascii_alphanumeric()
                    || matches!(chars[end], '.' | '_' | ':' | '-'))
            {
                end += 1;
            }
            span(&mut out, "n", &text(i, end));
            i = end;
            continue;
        }
        if ident_start(c) {
            let mut end = i;
            while end < chars.len() && ident_char(chars[end]) {
                end += 1;
            }
            let word = text(i, end);
            if word == "true" || word == "false" {
                span(&mut out, "k", &word);
            } else {
                out.push_str(&esc(&word));
            }
            i = end;
            continue;
        }
        out.push_str(&esc(&c.to_string()));
        i += 1;
    }
    out
}

#[cfg(test)]
mod other_tests {
    use super::other;

    #[test]
    fn shell_marks_commands_flags_strings_and_comments() {
        let html = other(
            "sh",
            "jacket test --exact 'a b' | head # first\nFOO=$HOME ls",
        )
        .unwrap();
        assert!(html.contains("<span class=\"k\">jacket</span>"));
        assert!(html.contains("<span class=\"d\">--exact</span>"));
        assert!(
            html.contains("<span class=\"s\">&#39;a b&#39;</span>")
                || html.contains("<span class=\"s\">'a b'</span>")
        );
        assert!(html.contains("<span class=\"k\">head</span>"));
        assert!(html.contains("<span class=\"c\"># first</span>"));
        assert!(html.contains("<span class=\"n\">$HOME</span>"));
        assert!(html.contains("<span class=\"k\">ls</span>"));
    }

    #[test]
    fn toml_marks_tables_keys_and_values() {
        let html = other(
            "toml",
            "[[doc.page]]\nname = \"Guide\" # shown\nwidth = 2\nok = true",
        )
        .unwrap();
        assert!(html.contains("<span class=\"t\">[[doc.page]]</span>"));
        assert!(html.contains("<span class=\"d\">name</span> = <span class=\"s\">"));
        assert!(html.contains("<span class=\"c\"># shown</span>"));
        assert!(html.contains("<span class=\"n\">2</span>"));
        assert!(html.contains("<span class=\"k\">true</span>"));
        assert!(other("text", "x").is_none());
    }
}

/// A linter finding as the terminal shows it, in colour: the `error[rule]` header, the `-->`
/// place, the gutter, the `^^^` markers with their labels, and the `help:`/`note:` lines.
///
/// Classes: `de`/`dw` (error and warning, for the header and its markers), `dm` (the muted
/// frame), `dk` (secondary `---` markers), `dh` (`help:` and `note:` labels), `dc` (a fix line's
/// `~`), `da` (`+`).
pub fn diagnostic(text: &str) -> String {
    let mut level = "de";
    let mut out = Vec::new();
    for line in text.lines() {
        let mut html = String::new();
        let trimmed = line.trim_start();
        let indent = &line[..line.len() - trimmed.len()];
        let kind = ["error", "warning"].into_iter().find(|k| {
            trimmed
                .strip_prefix(k)
                .is_some_and(|r| r.starts_with(['[', ':']))
        });
        if let Some(kind) = kind {
            level = if kind == "error" {
                "de"
            } else {
                "dw"
            };
            let colon = trimmed.find(':').unwrap_or(trimmed.len());
            span(&mut html, level, &trimmed[..colon]);
            html.push_str(&esc(&trimmed[colon..]));
        } else if let Some(place) = trimmed.strip_prefix("--> ") {
            html.push_str(&esc(indent));
            span(&mut html, "dm", "-->");
            html.push(' ');
            html.push_str(&esc(place));
        } else if let Some((label, rest)) = ["= note:", "= help:", "help:", "note:"]
            .into_iter()
            .find_map(|l| trimmed.strip_prefix(l).map(|r| (l, r)))
        {
            html.push_str(&esc(indent));
            span(&mut html, "dh", label);
            html.push_str(&esc(rest));
        } else {
            gutter_line(&mut html, line, level);
        }
        out.push(html);
    }
    out.join("\n")
}

/// A line of the snippet: `  5 | code`, `    |   ^^^ label`, or a fix line `  5 ~ code`.
fn gutter_line(html: &mut String, line: &str, level: &str) {
    let body = line.trim_start();
    let number = body.len() - body.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    let after = &body[number..];
    let bar = after.trim_start();
    let Some(sign) = bar
        .chars()
        .next()
        .filter(|c| matches!(c, '|' | '~' | '+' | '-'))
    else {
        html.push_str(&esc(line));
        return;
    };
    let frame_end = line.len() - bar.len();
    html.push_str(&esc(&line[..frame_end]));
    let rest = &bar[1..];
    let class = match sign {
        '|' => "dm",
        '~' => "dc",
        '+' => "da",
        _ => level,
    };
    span(html, class, &sign.to_string());
    let marks = rest.trim_start();
    // Only a line without a number can be markers: `5 |     -x` is code.
    if number == 0 && sign == '|' && marks.starts_with(['^', '-']) {
        html.push_str(&esc(&rest[..rest.len() - marks.len()]));
        let run = marks.len() - marks.trim_start_matches(['^', '-']).len();
        let class = if marks.starts_with('^') {
            level
        } else {
            "dk"
        };
        span(html, class, &marks[..run]);
        let label = marks[run..].trim_start();
        if !label.is_empty() {
            html.push(' ');
            span(html, class, label);
        }
    } else {
        html.push_str(&esc(rest));
    }
}

#[cfg(test)]
mod diagnostic_tests {
    use super::diagnostic;

    #[test]
    fn colours_the_parts_of_a_finding() {
        let html = diagnostic(
            "warning[unused_variable]: `x` is never used\n  --> a.jai:2:5\n   |\n 2 |     x := 1;\n   |     ^ not read\n   |\nhelp: remove it\n 2 ~     \n   = note: `unused_variable` is `warn` by default",
        );
        assert!(html.starts_with("<span class=\"dw\">warning[unused_variable]</span>: "));
        assert!(html.contains("<span class=\"dm\">--&gt;</span> a.jai:2:5"));
        assert!(html.contains("<span class=\"dw\">^</span> <span class=\"dw\">not read</span>"));
        assert!(html.contains("<span class=\"dh\">help:</span> remove it"));
        assert!(html.contains("<span class=\"dc\">~</span>"));
        assert!(html.contains("<span class=\"dh\">= note:</span> `unused_variable`"));
        // Code that starts with `-` is not a marker.
        assert!(!diagnostic(" 2 |     -x").contains("dk"));
    }
}
