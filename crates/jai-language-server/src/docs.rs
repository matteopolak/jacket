//! Doc comments, found by text so they work on any file (open or not) without parsing it.
//!
//! A declaration's docs are the run of `//` lines (or one `/* */` block) directly above its
//! line, with no blank line between, plus (for fields, enum members and parameters) a trailing
//! `// ...` on its own line. The text is Markdown: one space is stripped after each `//` and
//! nothing else. Extraction is lazy: callers ask about one declaration at a time.
//!
//! Also here: parameter docs (`- name: text` items in a procedure's docs, or comments on the
//! parameter itself), and `[name]` / `[text](name)` intra-doc links with the Markdown
//! rewriting that turns resolved links into real ones.

/// The most lines a doc comment shows.
pub(crate) const MAX_LINES: usize = 200;

/// The text of a `//` comment line, one leading space (and a `///` or `//!` marker) removed.
fn line_body(line: &str) -> Option<&str> {
    let rest = line.trim().strip_prefix("//")?;
    let rest = rest
        .strip_prefix('/')
        .or_else(|| rest.strip_prefix('!'))
        .unwrap_or(rest);
    Some(rest.strip_prefix(' ').unwrap_or(rest).trim_end())
}

/// A rule drawn in comments (`// ------`, `//====`): not documentation.
fn decoration(body: &str) -> bool {
    let body = body.trim();
    body.len() >= 4 && body.chars().all(|c| "/=*~#_-+".contains(c))
}

/// The start of the line holding `at`.
fn line_start(text: &str, at: usize) -> usize {
    text[..at].rfind('\n').map_or(0, |n| n + 1)
}

/// The line before the one starting at `start`, as (its start, its text without the newline).
fn previous_line(text: &str, start: usize) -> Option<(usize, &str)> {
    if start == 0 {
        return None;
    }
    let end = start - 1;
    let begin = text[..end].rfind('\n').map_or(0, |n| n + 1);
    Some((begin, text[begin..end].trim_end_matches('\r')))
}

/// The comment run directly above the line starting at `start`, as lines of Markdown.
fn comment_above(text: &str, start: usize) -> Vec<String> {
    let Some((mut begin, previous)) = previous_line(text, start) else {
        return Vec::new();
    };
    if previous.trim_end().ends_with("*/") {
        return block_above(text, begin, previous);
    }
    let mut lines: Vec<String> = Vec::new();
    let mut line = previous;
    while let Some(body) = line_body(line) {
        if decoration(body) {
            break;
        }
        lines.push(body.to_owned());
        if lines.len() >= MAX_LINES {
            break;
        }
        match previous_line(text, begin) {
            Some((b, l)) => {
                begin = b;
                line = l;
            }
            None => break,
        }
    }
    lines.reverse();
    lines
}

/// The `/* ... */` block whose last line is `last` (starting at `last_start`), as lines of
/// Markdown: the delimiters are dropped, and a ` * ` margin or the common indent is removed.
fn block_above(text: &str, last_start: usize, last: &str) -> Vec<String> {
    let mut begin = last_start;
    let mut line = last;
    let mut collected: Vec<&str> = vec![line];
    let mut depth = 0i32;
    loop {
        depth += line.matches("*/").count() as i32 - line.matches("/*").count() as i32;
        if depth <= 0 || collected.len() >= MAX_LINES {
            break;
        }
        match previous_line(text, begin) {
            Some((b, l)) => {
                begin = b;
                line = l;
                collected.push(line);
            }
            None => return Vec::new(),
        }
    }
    if depth > 0 {
        return Vec::new();
    }
    collected.reverse();
    let count = collected.len();
    let margin = collected.iter().skip(1).all(|l| {
        let t = l.trim_start();
        t.is_empty() || t.starts_with('*')
    });
    // Without a margin, the indent the continuation lines share is removed.
    let indent = if margin {
        0
    } else {
        collected
            .iter()
            .skip(1)
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.len() - l.trim_start().len())
            .min()
            .unwrap_or(0)
    };
    let mut lines: Vec<String> = Vec::with_capacity(count);
    for (i, raw) in collected.iter().enumerate() {
        let mut l: &str = raw;
        if i == 0 {
            l = l.trim_start();
            l = l.strip_prefix("/*").unwrap_or(l);
        }
        if i == count - 1 {
            l = l.trim_end();
            l = l.strip_suffix("*/").unwrap_or(l);
        }
        let text = if i == 0 {
            l.trim()
        } else if margin {
            let t = l.trim_start();
            let t = t.strip_prefix('*').unwrap_or(t);
            t.strip_prefix(' ').unwrap_or(t).trim_end()
        } else {
            l.get(indent..).unwrap_or(l.trim_start()).trim_end()
        };
        lines.push(text.to_owned());
    }
    while lines.first().is_some_and(|l| l.is_empty()) {
        lines.remove(0);
    }
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    lines
}

/// A `//` comment after the code at `from` on its line: (offset of the `//`, its body).
fn trailing_comment(text: &str, from: usize) -> Option<(usize, String)> {
    let end = text[from..].find('\n').map_or(text.len(), |n| from + n);
    let line = &text[from..end];
    let bytes = line.as_bytes();
    let mut in_string = false;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if in_string => i += 1,
            b'"' => in_string = !in_string,
            b'/' if !in_string && bytes.get(i + 1) == Some(&b'/') => {
                let body = line_body(&line[i..])?;
                return Some((from + i, body.to_owned()));
            }
            b'/' if !in_string && bytes.get(i + 1) == Some(&b'*') => return None,
            _ => {}
        }
        i += 1;
    }
    None
}

/// The docs above the declaration whose line holds `at`.
pub(crate) fn above(text: &str, at: usize) -> Option<String> {
    let lines = comment_above(text, line_start(text, at));
    let doc = lines.join("\n");
    (!doc.trim().is_empty()).then_some(doc)
}

/// The `// ...` after a field, member or parameter declared at `at` (its name), if what
/// comes before it on the line is a whole declaration rather than the start of a body.
pub(crate) fn trailing(text: &str, at: usize) -> Option<String> {
    let (comment_at, body) = trailing_comment(text, at)?;
    let code = text[at..comment_at].trim_end();
    if code.contains('{') {
        return None;
    }
    let ends = code.ends_with(';') || code.ends_with(',');
    if !(ends || !code.contains("::")) || body.trim().is_empty() {
        return None;
    }
    Some(body)
}

/// The docs of the declaration named at `at`: the comment lines above it, else its trailing
/// comment; for a parameter, its own comments, else its item in the procedure's docs.
pub(crate) fn declaration(text: &str, at: usize) -> Option<String> {
    if let Some(open) = enclosing_header(text, at) {
        let name = word_at(text, at)?;
        return parameter(text, open, name);
    }
    above(text, at).or_else(|| trailing(text, at))
}

/// The identifier starting at `at`.
fn word_at(text: &str, at: usize) -> Option<&str> {
    let rest = text.get(at..)?;
    let end = rest
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(rest.len());
    (end > 0).then(|| &rest[..end])
}

/// The `(` of the procedure header the parameter named at `at` is in, if it is in one: the
/// nearest unclosed `(` before `at`, without a `{`, `}` or `;` in between, that follows `::`
/// (or `#type`, `inline`).
pub(crate) fn enclosing_header(text: &str, at: usize) -> Option<usize> {
    const LOOKBACK_LINES: usize = 80;
    let mut depth = 0i32;
    let mut end = at;
    let mut start = line_start(text, at);
    for _ in 0..LOOKBACK_LINES {
        let line = &text[start..end];
        // Drop a trailing comment (outside strings).
        let code = match trailing_comment(text, start) {
            Some((comment_at, _)) if comment_at < end => &text[start..comment_at],
            _ => line,
        };
        for (i, c) in code.char_indices().rev() {
            match c {
                ')' | ']' => depth += 1,
                '[' => depth -= 1,
                '(' if depth > 0 => depth -= 1,
                '(' => {
                    let before = text[..start + i].trim_end();
                    let header = before.ends_with("::")
                        || before.ends_with("#type")
                        || before.ends_with("inline")
                        || before.ends_with("#type,distinct");
                    return header.then_some(start + i);
                }
                '{' | '}' | ';' => return None,
                _ => {}
            }
        }
        let (begin, _) = previous_line(text, start)?;
        end = start - 1;
        start = begin;
    }
    None
}

/// The docs of parameter `name` of the header opened at `open`: comments on the parameter
/// itself, else its item in the docs above the header.
fn parameter(text: &str, open: usize, name: &str) -> Option<String> {
    let name = name.trim_start_matches('$');
    header_parameters(text, open)
        .into_iter()
        .find(|(n, _)| n == name)
        .map(|(_, doc)| doc)
        .or_else(|| {
            let doc = above(text, open)?;
            list_parameters(&doc)
                .into_iter()
                .find(|(n, _)| n == name)
                .map(|(_, d)| d)
        })
}

/// Every parameter of the procedure whose header opens at `open` that has docs, from its own
/// comments (above it, or after it on its line) and from the list in the docs above the
/// header, as (name, docs). Own comments win.
pub(crate) fn parameters(text: &str, open: usize) -> Vec<(String, String)> {
    let mut found = header_parameters(text, open);
    if let Some(doc) = above(text, open) {
        for (name, doc) in list_parameters(&doc) {
            if !found.iter().any(|(n, _)| *n == name) {
                found.push((name, doc));
            }
        }
    }
    found
}

/// Where the parameter list of the procedure declared at `at` (its name, or its literal)
/// opens: the first `(` from there.
pub(crate) fn header_open(text: &str, at: usize) -> Option<usize> {
    let end = text.get(at..)?.find('\n').map_or(text.len(), |n| at + n);
    text[at..end].find('(').map(|n| at + n)
}

/// Parameters with a comment directly above them or after them on their line.
fn header_parameters(text: &str, open: usize) -> Vec<(String, String)> {
    const MAX_HEADER_LINES: usize = 80;
    let mut out: Vec<(String, String)> = Vec::new();
    let mut pending: Vec<String> = Vec::new();
    let mut depth = 0i32;
    let mut start = line_start(text, open);
    let mut first = true;
    for _ in 0..MAX_HEADER_LINES {
        let end = text[start..].find('\n').map_or(text.len(), |n| start + n);
        let line = text[start..end].trim_end_matches('\r');
        let from = if first {
            open - start
        } else {
            0
        };
        first = false;
        if depth == 0 && start != line_start(text, open) {
            break;
        }
        let comment = trailing_comment(text, start + from);
        let code_end = comment.as_ref().map_or(end, |(at, _)| *at);
        let code = &text[start + from..code_end];
        if code.trim().is_empty() && depth > 0 {
            match line_body(line) {
                Some(body) if comment.is_some() || line.trim_start().starts_with("//") => {
                    pending.push(body.to_owned());
                }
                _ => pending.clear(),
            }
        } else {
            // Split the code at top-level commas into parameters.
            let mut segments: Vec<(usize, String)> = Vec::new();
            let mut current = String::new();
            let mut began_at_depth_one = depth == 1 && from == 0;
            let mut seg_start = 0usize;
            let mut in_string = false;
            let mut closed = false;
            for (i, c) in code.char_indices() {
                if in_string {
                    current.push(c);
                    if c == '"' {
                        in_string = false;
                    }
                    continue;
                }
                match c {
                    '"' => {
                        in_string = true;
                        current.push(c);
                    }
                    '(' | '[' | '{' => {
                        depth += 1;
                        if depth > 1 {
                            current.push(c);
                        } else {
                            began_at_depth_one = true;
                            seg_start = i + 1;
                        }
                    }
                    ')' | ']' | '}' => {
                        depth -= 1;
                        if depth == 0 {
                            closed = true;
                            if began_at_depth_one {
                                segments.push((seg_start, std::mem::take(&mut current)));
                            }
                            break;
                        }
                        current.push(c);
                    }
                    ',' if depth == 1 => {
                        if began_at_depth_one {
                            segments.push((seg_start, std::mem::take(&mut current)));
                        }
                        began_at_depth_one = true;
                        seg_start = i + 1;
                    }
                    _ => current.push(c),
                }
            }
            if !closed && began_at_depth_one && !current.trim().is_empty() {
                segments.push((seg_start, std::mem::take(&mut current)));
            }
            let named: Vec<(String, bool)> = segments
                .iter()
                .filter_map(|(_, s)| {
                    let s = s.trim().trim_start_matches('$').trim_start_matches('$');
                    let end = s
                        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
                        .unwrap_or(s.len());
                    let rest = s[end..].trim_start();
                    (end > 0 && (rest.is_empty() || rest.starts_with(':')))
                        .then(|| (s[..end].to_owned(), false))
                })
                .collect();
            let last = named.len().saturating_sub(1);
            for (i, (name, _)) in named.iter().enumerate() {
                let mut parts: Vec<String> = Vec::new();
                if i == 0 {
                    parts.append(&mut pending);
                }
                if i == last
                    && let Some((_, body)) = &comment
                    && !body.trim().is_empty()
                {
                    parts.push(body.clone());
                }
                if !parts.is_empty() {
                    out.push((name.clone(), parts.join("\n")));
                }
            }
            pending.clear();
            if closed {
                break;
            }
        }
        if end >= text.len() {
            break;
        }
        start = end + 1;
    }
    out
}

/// Items of a Markdown list in `doc` that document a parameter: `- name: text`,
/// ``- `name`: text`` or `- name — text`. Following indented lines continue the item.
pub(crate) fn list_parameters(doc: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut fenced = false;
    let mut open = false;
    for line in doc.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fenced = !fenced;
            open = false;
            continue;
        }
        if fenced {
            continue;
        }
        if let Some((name, rest)) = list_item(trimmed) {
            out.push((name, rest));
            open = true;
        } else if open && line.starts_with([' ', '\t']) && !trimmed.is_empty() {
            if let Some((_, text)) = out.last_mut() {
                text.push('\n');
                text.push_str(trimmed);
            }
        } else {
            open = false;
        }
    }
    out
}

/// `- name: text` and its variants as (name, text).
fn list_item(line: &str) -> Option<(String, String)> {
    let rest = line
        .strip_prefix("- ")
        .or_else(|| line.strip_prefix("* "))
        .or_else(|| line.strip_prefix("+ "))?
        .trim_start();
    let (name, rest) = if let Some(inner) = rest.strip_prefix('`') {
        let end = inner.find('`')?;
        (&inner[..end], &inner[end + 1..])
    } else {
        let rest = rest.strip_prefix('$').unwrap_or(rest);
        let end = rest
            .find(|c: char| !(c.is_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
        (&rest[..end], &rest[end..])
    };
    let name = name.trim_start_matches('$');
    if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }
    let rest = rest
        .strip_prefix(':')
        .or_else(|| rest.trim_start().strip_prefix('—'))
        .or_else(|| rest.trim_start().strip_prefix('–'))
        .or_else(|| rest.trim_start().strip_prefix("- "))?;
    Some((name.to_owned(), rest.trim().to_owned()))
}

// -------------------------------------------------------------------------------------------
// Intra-doc links
// -------------------------------------------------------------------------------------------

/// A `[name]`, `` [`name`] `` or `[text](name)` in a line of documentation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Link {
    /// The whole link, brackets (and target) included, as byte offsets into the line.
    pub(crate) whole: (usize, usize),
    /// The text between the brackets.
    pub(crate) label: (usize, usize),
    /// What it names: `print`, `Basic`, `print` for `Basic.print`'s last segment, ...
    pub(crate) names: Vec<String>,
}

/// A path such as `print` or `Basic.print` from link text (backticks and a trailing `()`
/// removed).
fn link_path(text: &str) -> Option<Vec<String>> {
    let text = text.trim();
    let text = text
        .strip_prefix('`')
        .and_then(|t| t.strip_suffix('`'))
        .unwrap_or(text)
        .trim();
    let text = text.strip_suffix("()").unwrap_or(text);
    if text.is_empty() {
        return None;
    }
    let mut names = Vec::new();
    for segment in text.split('.') {
        let mut chars = segment.chars();
        let first = chars.next()?;
        if !(first.is_alphabetic() || first == '_')
            || !chars.all(|c| c.is_alphanumeric() || c == '_')
        {
            return None;
        }
        names.push(segment.to_owned());
    }
    Some(names)
}

/// The links in one line of documentation, skipping inline code and things that are not
/// links (`a[i]`, images, `[x][y]`).
pub(crate) fn links(line: &str) -> Vec<Link> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'`' => {
                let run = bytes[i..].iter().take_while(|&&b| b == b'`').count();
                let ticks = &line[i..i + run];
                match line[i + run..].find(ticks) {
                    Some(close) => i += run + close + run,
                    None => i += run,
                }
            }
            b'[' => {
                let prev = line[..i].chars().next_back();
                let indexing =
                    prev.is_some_and(|c| c.is_alphanumeric() || matches!(c, '_' | ')' | ']' | '!'));
                let Some(close) = closing_bracket(line, i) else {
                    i += 1;
                    continue;
                };
                let label = (i + 1, close);
                let after = &line[close + 1..];
                if indexing || after.starts_with('[') {
                    i = close + 1;
                    continue;
                }
                if let Some(target) = after.strip_prefix('(') {
                    // `[text](name)`: only a name is a link; a URL is left alone.
                    if let Some(end) = target.find(')') {
                        if let Some(names) = link_path(&target[..end]) {
                            out.push(Link {
                                whole: (i, close + 2 + end),
                                label,
                                names,
                            });
                        }
                        i = close + 2 + end;
                        continue;
                    }
                } else if let Some(names) = link_path(&line[label.0..label.1]) {
                    out.push(Link {
                        whole: (i, close + 1),
                        label,
                        names,
                    });
                }
                i = close + 1;
            }
            _ => i += 1,
        }
    }
    out
}

/// The `]` matching the `[` at `open`, not counting brackets inside inline code.
fn closing_bracket(line: &str, open: usize) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = open + 1;
    while i < bytes.len() {
        match bytes[i] {
            b']' => return Some(i),
            b'[' => return None,
            b'`' => {
                let run = bytes[i..].iter().take_while(|&&b| b == b'`').count();
                let ticks = &line[i..i + run];
                match line[i + run..].find(ticks) {
                    Some(close) => i += run + close + run,
                    None => i += run,
                }
            }
            _ => i += 1,
        }
    }
    None
}

/// Whether `body` opens or closes a fenced code block.
fn fence(body: &str) -> bool {
    let t = body.trim_start();
    t.starts_with("```") || t.starts_with("~~~")
}

/// `doc` as Markdown for display: untagged code fences become `jai`, and links `resolve`
/// gives a URL for become Markdown links to it (the rest stay as written).
pub(crate) fn markdown(doc: &str, resolve: &dyn Fn(&[String]) -> Option<String>) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut fenced = false;
    for line in doc.lines() {
        if fence(line) {
            let t = line.trim_start();
            let indent = &line[..line.len() - t.len()];
            let mark = t.chars().next().unwrap_or('`');
            let run = t.chars().take_while(|&c| c == mark).count();
            let info = t[run..].trim();
            if !fenced {
                fenced = true;
                if info.is_empty() && mark == '`' {
                    out.push(format!("{indent}{}jai", &t[..run]));
                    continue;
                }
            } else if info.is_empty() {
                fenced = false;
            }
            out.push(line.to_owned());
            continue;
        }
        if fenced {
            out.push(line.to_owned());
            continue;
        }
        let mut rewritten = String::new();
        let mut at = 0;
        for link in links(line) {
            let Some(url) = resolve(&link.names) else {
                continue;
            };
            rewritten.push_str(&line[at..link.whole.0]);
            rewritten.push_str(&format!("[{}]({url})", &line[link.label.0..link.label.1]));
            at = link.whole.1;
        }
        rewritten.push_str(&line[at..]);
        out.push(rewritten);
    }
    out.join("\n")
}

/// Every link name path in `doc` (outside code fences), without repeats.
pub(crate) fn link_names(doc: &str) -> Vec<Vec<String>> {
    let mut out: Vec<Vec<String>> = Vec::new();
    let mut fenced = false;
    for line in doc.lines() {
        if fence(line) {
            fenced = !fenced;
            continue;
        }
        if fenced || !line.contains('[') {
            continue;
        }
        for link in links(line) {
            if !out.contains(&link.names) {
                out.push(link.names);
            }
        }
    }
    out
}

/// A link in a source file: where its label is and what it names, resolved from `scope_at`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Site {
    pub(crate) label: (usize, usize),
    pub(crate) names: Vec<String>,
    /// An offset in the declaration the comment documents (its scope resolves the names).
    pub(crate) scope_at: usize,
}

/// The links in the doc comments of `text`: comment runs and blocks directly above a
/// declaration line, and trailing comments.
pub(crate) fn sites(text: &str) -> Vec<Site> {
    let mut out = Vec::new();
    // (offset of the line, its text)
    let mut run: Vec<(usize, &str)> = Vec::new();
    let mut block_depth = 0i32;
    let mut offset = 0;
    for raw in text.split_inclusive('\n') {
        let line = raw.trim_end_matches(['\n', '\r']);
        let at = offset;
        offset += raw.len();
        let trimmed = line.trim_start();
        if block_depth > 0 {
            run.push((at, line));
            block_depth += line.matches("/*").count() as i32 - line.matches("*/").count() as i32;
            continue;
        }
        if trimmed.starts_with("/*") {
            let depth = trimmed.matches("/*").count() as i32 - trimmed.matches("*/").count() as i32;
            run.clear();
            run.push((at, line));
            block_depth = depth.max(0);
            continue;
        }
        if trimmed.starts_with("//") {
            run.push((at, line));
            continue;
        }
        if trimmed.is_empty() {
            run.clear();
            continue;
        }
        let scope_at = at + (line.len() - trimmed.len());
        if !run.is_empty() {
            collect_sites(&run, scope_at, &mut out);
            run.clear();
        }
        if let Some((comment_at, _)) = trailing_comment(text, at + (line.len() - trimmed.len())) {
            let end = at + line.len();
            collect_sites(&[(comment_at, &text[comment_at..end])], scope_at, &mut out);
        }
    }
    out
}

/// The site whose label holds `byte`.
pub(crate) fn site_at(text: &str, byte: usize) -> Option<Site> {
    // Only a comment can hold a link: skip the scan for other lines.
    let start = line_start(text, byte);
    let end = text[byte..].find('\n').map_or(text.len(), |n| byte + n);
    let line = &text[start..end];
    if !line.contains("//")
        && !line.contains("*/")
        && !line.contains("/*")
        && !line.trim_start().starts_with('*')
    {
        return None;
    }
    sites(text)
        .into_iter()
        .find(|s| s.label.0 <= byte && byte <= s.label.1)
}

fn collect_sites(run: &[(usize, &str)], scope_at: usize, out: &mut Vec<Site>) {
    let mut fenced = false;
    for &(at, line) in run {
        // The part of the line that is comment text.
        let (body_at, body) = match line.find("//") {
            Some(n) if line[..n].trim().is_empty() || !line.trim_start().starts_with("/*") => {
                (n + 2, &line[n + 2..])
            }
            _ => (0, line),
        };
        if fence(body.trim_start_matches(['/', '!', '*'])) || fence(body.trim_start()) {
            fenced = !fenced;
            continue;
        }
        if fenced || !body.contains('[') {
            continue;
        }
        for link in links(body) {
            out.push(Site {
                label: (at + body_at + link.label.0, at + body_at + link.label.1),
                names: link.names,
                scope_at,
            });
        }
    }
}

/// A declaration's first line as a signature to show: from `at` to the `{` or the end of the
/// line, bounded.
pub(crate) fn signature_line(text: &str, at: usize) -> String {
    let start = line_start(text, at);
    let end = text[at..].find('\n').map_or(text.len(), |n| at + n);
    let line = text[start..end].trim();
    let line = line.split(" //").next().unwrap_or(line);
    let line = line.trim_end_matches('{').trim_end();
    if line.len() > 160 {
        let cut = line.floor_char_boundary(160);
        format!("{} ...", &line[..cut])
    } else {
        line.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(text: &str, name: &str) -> Option<String> {
        declaration(text, text.find(name).unwrap())
    }

    #[test]
    fn line_comments_directly_above_are_the_docs() {
        let text = "x :: 1;\n\n// First.\n//  Indented *md*.\n//\n// Last.\nf :: () {}\n";
        assert_eq!(
            doc(text, "f ::").unwrap(),
            "First.\n Indented *md*.\n\nLast."
        );
        let gap = "// Not docs.\n\nf :: () {}\n";
        assert_eq!(doc(gap, "f ::"), None);
        let rule = "// ------\nf :: () {}\n";
        assert_eq!(doc(rule, "f ::"), None);
    }

    #[test]
    fn block_comments_are_docs() {
        let text = "/* Adds.\n * Twice.\n */\nf :: () {}\n";
        assert_eq!(doc(text, "f ::").unwrap(), "Adds.\nTwice.");
        let one = "/* One line. */\nf :: () {}\n";
        assert_eq!(doc(one, "f ::").unwrap(), "One line.");
    }

    #[test]
    fn fields_and_members_take_trailing_comments() {
        let text = "P :: struct {\n    x: int; // the x\n    // the y\n    y: int;\n}\n";
        assert_eq!(doc(text, "x:").unwrap(), "the x");
        assert_eq!(doc(text, "y:").unwrap(), "the y");
        assert_eq!(doc("f :: () { // not docs\n}\n", "f ::"), None);
    }

    #[test]
    fn parameters_use_their_own_comments_or_the_list() {
        let text = "\
// Does it.
// - a: the first
// - `b` — the second
//   continued
f :: (a: int, b: int, /* c */ c: int) {}
g :: (
    // the left
    left: int,
    right: int, // the right
) {}
";
        let at = |n: &str| text.find(n).unwrap();
        assert_eq!(declaration(text, at("a: int")).unwrap(), "the first");
        assert_eq!(
            declaration(text, at("b: int")).unwrap(),
            "the second\ncontinued"
        );
        assert_eq!(declaration(text, at("left")).unwrap(), "the left");
        assert_eq!(declaration(text, at("right")).unwrap(), "the right");
        let open = header_open(text, at("g ::")).unwrap();
        assert_eq!(
            parameters(text, open),
            [
                ("left".to_owned(), "the left".to_owned()),
                ("right".to_owned(), "the right".to_owned())
            ]
        );
    }

    #[test]
    fn locals_are_not_parameters() {
        let text = "// Doc.\nf :: (a: int) {\n    local := 1;\n}\n";
        assert_eq!(enclosing_header(text, text.find("local").unwrap()), None);
        assert!(enclosing_header(text, text.find("a:").unwrap()).is_some());
    }

    #[test]
    fn links_in_prose() {
        let names =
            |s: &str| -> Vec<Vec<String>> { links(s).into_iter().map(|l| l.names).collect() };
        assert_eq!(
            names("see [print] and [`Basic.print`]"),
            [vec!["print"], vec!["Basic", "print"]]
        );
        assert_eq!(
            names("[the text](Thing.x) `[code]` a[i] [0] [url](http://x)"),
            [vec!["Thing", "x"]]
        );
        assert_eq!(names("![img](x) [a b]"), Vec::<Vec<String>>::new());
    }

    #[test]
    fn markdown_tags_fences_and_links() {
        let resolve =
            |names: &[String]| (names == ["print"]).then(|| "file:///a.jai#L3".to_owned());
        let doc = "See [`print`] and [x].\n\n```\nlet [print]\n```\n\n```c\nint x;\n```";
        assert_eq!(
            markdown(doc, &resolve),
            "See [`print`](file:///a.jai#L3) and [x].\n\n```jai\nlet [print]\n```\n\n```c\nint x;\n```"
        );
    }

    #[test]
    fn sites_cover_runs_blocks_and_trailing_comments() {
        let text = "// Uses [helper].\nf :: () {}\nx: int; // see [Mode.IDLE]\n\n// stray [nothing]\n\ng :: 1;\n";
        let found = sites(text);
        let labels: Vec<&str> = found.iter().map(|s| &text[s.label.0..s.label.1]).collect();
        assert_eq!(labels, ["helper", "Mode.IDLE"]);
        assert_eq!(found[0].scope_at, text.find("f ::").unwrap());
        assert!(site_at(text, text.find("helper").unwrap() + 2).is_some());
        assert!(site_at(text, text.find("stray").unwrap()).is_none());
    }
}
