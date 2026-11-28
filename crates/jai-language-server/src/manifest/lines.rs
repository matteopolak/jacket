//! A `jacket.toml` read a line at a time: which table a line is in, which keys a line holds, and
//! what the cursor is on. Nothing here needs the text to be valid TOML.
use crate::analysis::Span;

/// What is wanted at the cursor.
pub(super) enum Place {
    /// Inside the brackets of a table header: `start..end` is the name typed.
    Header {
        start: usize,
        end: usize,
        prefix: String,
    },
    /// Where a key goes in `table`; `present` are the keys the table has already.
    Key {
        start: usize,
        end: usize,
        table: String,
        prefix: String,
        present: Vec<String>,
        /// The key can be followed by ` = ` (a line with no value yet).
        assign: bool,
    },
    /// Inside a string that names a path: `start..end` is the part after the last `/` typed,
    /// `folder` what comes before it.
    Path {
        start: usize,
        end: usize,
        folder: String,
        kind: PathKind,
    },
    Nothing,
}

/// What a path-valued key names.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PathKind {
    Files,
    Folders,
    /// Folders that hold a `module.jai`.
    Modules,
}

fn path_kind(table: &str, key: &str) -> Option<PathKind> {
    match (table, key) {
        ("project", "entry") => Some(PathKind::Files),
        ("project" | "package", "import-path") | ("test", "paths") => Some(PathKind::Folders),
        ("package", "module") => Some(PathKind::Modules),
        (table, "path") if table.starts_with("dependencies.") => Some(PathKind::Folders),
        _ => None,
    }
}

fn is_key_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

/// The dotted name in a header line (`[lint.portability]`, `[[lint.overrides]]`), or `None` for
/// a line that is not one.
fn header_name(line: &str) -> Option<String> {
    let rest = line.trim_start().strip_prefix('[')?;
    let rest = rest.strip_prefix('[').unwrap_or(rest);
    let name = rest.split(']').next()?;

    Some(
        name.split('.')
            .map(|part| part.trim().trim_matches('"'))
            .collect::<Vec<_>>()
            .join("."),
    )
}

/// `(start of line, line without its newline)` for each line of `text`.
fn lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut start = 0;

    text.split_inclusive('\n').map(move |raw| {
        let at = start;

        start += raw.len();
        (
            at,
            raw.strip_suffix('\n').unwrap_or(raw).trim_end_matches('\r'),
        )
    })
}

/// The table the line starting at `line_start` is in.
fn table_at(text: &str, line_start: usize) -> String {
    lines(text)
        .take_while(|(start, _)| *start < line_start)
        .filter_map(|(_, line)| header_name(line))
        .last()
        .unwrap_or_default()
}

/// Where the scan of a line prefix ends up.
struct State {
    in_string: Option<usize>,
    expect_key: bool,
    /// The key whose value is being written.
    key: Option<String>,
    /// The key an inline table is the value of.
    outer: Option<String>,
    inline: bool,
}

fn scan(before: &str, base: usize) -> State {
    let mut state = State {
        in_string: None,
        expect_key: true,
        key: None,
        outer: None,
        inline: false,
    };
    let mut brackets = 0usize;
    let mut word = String::new();
    let mut last_key: Option<String> = None;
    let mut chars = before.char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        if state.in_string.is_some() {
            match c {
                '\\' => {
                    chars.next();
                }
                '"' => state.in_string = None,
                _ => {}
            }

            continue;
        }

        if is_key_char(c) {
            word.push(c);
            continue;
        }

        if !word.is_empty() && state.expect_key && !c.is_whitespace() && c != '=' {
            word.clear();
        }

        match c {
            '"' => state.in_string = Some(base + i),
            '#' => break,
            '=' => {
                if state.expect_key && !word.is_empty() {
                    state.key = Some(std::mem::take(&mut word));

                    if !state.inline {
                        last_key.clone_from(&state.key);
                    }
                }

                state.expect_key = false;
            }
            '{' => {
                state.outer.clone_from(&last_key);
                state.inline = true;
                state.expect_key = true;
                state.key = None;
            }
            '}' => {
                state.inline = false;
                state.expect_key = false;
            }
            '[' => brackets += 1,
            ']' => brackets = brackets.saturating_sub(1),
            ',' if state.inline && brackets == 0 => {
                state.expect_key = true;
                state.key = None;
            }
            _ => {}
        }

        if !is_key_char(c) && !c.is_whitespace() {
            word.clear();
        }
    }

    state
}

/// What the cursor at byte `at` is wanted for.
pub(super) fn place(text: &str, at: usize) -> Place {
    let Some((line_start, line)) =
        lines(text).find(|(start, line)| at >= *start && at <= start + line.len())
    else {
        return Place::Key {
            start: at,
            end: at,
            table: table_at(text, at),
            prefix: String::new(),
            present: Vec::new(),
            assign: true,
        };
    };
    let table = table_at(text, line_start);
    let before = &line[..at - line_start];
    let after = &line[at - line_start..];

    if line.trim_start().starts_with('[') {
        let open = line.find('[').unwrap_or(0);
        let name_start = line_start + line[open..].find(|c| c != '[').map_or(open, |n| open + n);
        let typed_end = at.max(name_start);
        let word_end = typed_end
            + text[typed_end..line_start + line.len()]
                .find(|c: char| !(is_key_char(c) || c == '.'))
                .unwrap_or(line_start + line.len() - typed_end);

        return Place::Header {
            start: name_start,
            end: word_end,
            prefix: text[name_start..typed_end].to_string(),
        };
    }

    let state = scan(before, line_start);
    let word_end = at + after.find(|c: char| !is_key_char(c)).unwrap_or(after.len());
    let table_here = match (&state.outer, state.inline) {
        (Some(outer), true) => format!("{table}.{outer}"),
        _ => table.clone(),
    };

    if let Some(quote) = state.in_string {
        let key = state.key.as_deref().unwrap_or("");
        let Some(kind) = path_kind(&table_here, key) else {
            return Place::Nothing;
        };
        let typed = &text[quote + 1..at];
        let (folder, partial) = typed.rsplit_once('/').unwrap_or(("", typed));
        let end = at + after.find('"').unwrap_or(after.len());

        return Place::Path {
            start: at - partial.len(),
            end,
            folder: folder.to_string(),
            kind,
        };
    }

    if !state.expect_key {
        return Place::Nothing;
    }

    let start = at - before.len() + before.rfind(|c: char| !is_key_char(c)).map_or(0, |n| n + 1);
    let start = start.min(at);
    let mut present: Vec<String> = Vec::new();

    if state.inline {
        present.extend(
            keys_of_line(line_start, line, &table)
                .into_iter()
                .map(|k| k.key),
        );
    } else {
        for (start_of, other) in lines(text) {
            if start_of == line_start {
                continue;
            }

            let (header, in_table) = (header_name(other), table_at(text, start_of));

            if header.is_none() && in_table == table {
                present.extend(
                    keys_of_line(start_of, other, &table)
                        .into_iter()
                        .filter(|k| k.table == table)
                        .map(|k| k.key),
                );
            }
        }
    }

    Place::Key {
        start,
        end: word_end,
        table: table_here,
        prefix: text[start..at].to_string(),
        present,
        assign: !state.inline && !after[word_end - at..].contains('='),
    }
}

/// A key written on a line.
struct KeyAt {
    start: usize,
    end: usize,
    key: String,
    /// The table it belongs to: an inline table's keys are in `table.outer`.
    table: String,
}

fn keys_of_line(line_start: usize, line: &str, table: &str) -> Vec<KeyAt> {
    let mut found = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0;
    let mut depth = 0usize;
    let mut outer: Option<String> = None;

    while i < bytes.len() {
        match bytes[i] {
            b'#' => break,
            b'"' => {
                i += 1;

                while i < bytes.len() && bytes[i] != b'"' {
                    i += if bytes[i] == b'\\' {
                        2
                    } else {
                        1
                    };
                }
            }
            b'{' => depth += 1,
            b'}' => depth = depth.saturating_sub(1),
            c if is_key_char(c as char) => {
                let start = i;

                while i < bytes.len() && is_key_char(bytes[i] as char) {
                    i += 1;
                }

                let rest = line[i..].trim_start();

                if rest.starts_with('=') {
                    let key = line[start..i].to_string();
                    let owner = match (&outer, depth) {
                        (Some(outer), 1..) => format!("{table}.{outer}"),
                        _ => table.to_string(),
                    };

                    if depth == 0 {
                        outer = Some(key.clone());
                    }

                    found.push(KeyAt {
                        start: line_start + start,
                        end: line_start + i,
                        key,
                        table: owner,
                    });
                }

                continue;
            }
            _ => {}
        }

        i += 1;
    }

    found
}

/// The span, name and description of the key or table the cursor at `at` is on.
pub(super) fn describe_at(text: &str, at: usize) -> Option<(Span, String, &'static str)> {
    let (line_start, line) =
        lines(text).find(|(start, line)| at >= *start && at <= start + line.len())?;

    if let Some(name) = header_name(line) {
        let open = line.find(|c| c != ' ' && c != '[')?;
        let mut cursor = line_start + open;
        let mut path = Vec::new();

        for part in name.split('.') {
            let raw = text[cursor..].find(part)? + cursor;
            let (start, end) = (raw, raw + part.len());

            path.push(part);

            if (start..=end).contains(&at) {
                let table = path.join(".");
                let text = jacket_config::schema::describe_table(&table)?;

                return Some((Span::new(start, end), table, text));
            }

            cursor = end;
        }

        return None;
    }

    let table = table_at(text, line_start);
    let key = keys_of_line(line_start, line, &table)
        .into_iter()
        .find(|k| (k.start..=k.end).contains(&at))?;
    let description = jacket_config::schema::describe(&key.table, &key.key)?;
    let name = if key.table.is_empty() {
        key.key.clone()
    } else {
        format!("{}.{}", key.table, key.key)
    };

    Some((Span::new(key.start, key.end), name, description))
}

/// The table names a header can complete to (`dependencies`, not `dependencies.<name>`).
pub(super) fn table_names() -> Vec<String> {
    jacket_config::schema::tables()
        .into_iter()
        .filter(|name| !name.is_empty())
        .map(|name| match name.strip_suffix(".<name>") {
            Some(parent) => parent.to_string(),
            None => name,
        })
        .collect()
}

/// The byte where the line that defines dependency `name` ends: its `[dependencies.name]` header
/// or its `name = ...` line under `[dependencies]`.
pub(super) fn dependency_line_end(text: &str, name: &str) -> Option<usize> {
    let header = format!("dependencies.{name}");

    lines(text).find_map(|(start, line)| {
        let defines = match header_name(line) {
            Some(found) => found == header,
            None => {
                table_at(text, start) == "dependencies"
                    && keys_of_line(start, line, "dependencies")
                        .iter()
                        .any(|k| k.table == "dependencies" && k.key == name)
            }
        };
        defines.then(|| start + line.trim_end().len())
    })
}
