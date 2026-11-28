//! What is wrong with the text of a `jacket.toml`, each with its place, for editors. The messages
//! are the ones [`crate::parse`] and the [`Check`]s give; nothing here judges the file itself.
use crate::{Check, Config, parse_lenient, unknown_note};
use std::ops::Range;
use std::path::Path;
use toml::de::{DeTable, DeValue};

/// One problem in the file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Issue {
    pub message: String,
    /// The bytes it is about; `None` when no key or value is to blame.
    pub span: Option<Range<usize>>,
}

/// Every problem of `text` (a file in `dir`): what does not read, the keys the schema does not
/// know, then what each of `checks` finds wrong. The checks run only on a file that reads.
///
/// ```
/// let text = "[lint]\nmax-nestng = 3\n";
/// let found = jacket_config::issues(text, std::path::Path::new("."), &[]);
/// assert_eq!(&text[found[0].span.clone().unwrap()], "max-nestng");
/// ```
pub fn issues(text: &str, dir: &Path, checks: &[Check]) -> Vec<Issue> {
    let Ok((config, unknown)) = parse_lenient(text) else {
        return vec![unreadable(text)];
    };

    if !unknown.is_empty() {
        return unknown
            .iter()
            .map(|path| Issue {
                message: unknown_note(path),
                span: key_span(text, path),
            })
            .collect();
    }

    check(text, &config, dir, checks)
}

fn unreadable(text: &str) -> Issue {
    let error = match toml::de::Deserializer::parse(text) {
        Err(error) => error,
        Ok(deserializer) => {
            match serde_ignored::deserialize::<_, _, Config>(deserializer, |_| {}) {
                Err(error) => error,
                Ok(_) => unreachable!("`parse_lenient` rejected this text"),
            }
        }
    };

    Issue {
        message: error.message().to_string(),
        span: error.span(),
    }
}

fn check(text: &str, config: &Config, dir: &Path, checks: &[Check]) -> Vec<Issue> {
    checks
        .iter()
        .filter_map(|check| check(config, dir).err())
        .map(|message| Issue {
            span: named_key(text, &message),
            message,
        })
        .collect()
}

/// The first key a message names in backticks, wherever it is in the file (an unknown rule
/// is named `unused_varible`, say), else nothing.
fn named_key(text: &str, message: &str) -> Option<Range<usize>> {
    let table = DeTable::parse(text).ok()?;

    message
        .split('`')
        .skip(1)
        .step_by(2)
        .find_map(|name| key_span(text, name).or_else(|| find_key(table.get_ref(), name)))
}

/// The span of the key at the dotted `path` (`lint.max-nesting`; a segment may be quoted, and an
/// array's entries are numbered: `lint.overrides.0.rules`).
pub fn key_span(text: &str, path: &str) -> Option<Range<usize>> {
    let root = DeTable::parse(text).ok()?;
    walk(root.get_ref(), &segments(path))
}

fn walk(table: &DeTable<'_>, path: &[String]) -> Option<Range<usize>> {
    let (key, value) = table.get_key_value(path.first()?.as_str())?;
    let rest = &path[1..];

    if rest.is_empty() {
        return Some(key.span());
    }

    match value.get_ref() {
        DeValue::Table(inner) => walk(inner, rest),
        DeValue::Array(items) => {
            let (entry, rest) = match rest[0].parse::<usize>() {
                Ok(n) => (items.get(n)?, &rest[1..]),
                Err(_) => (items.first()?, rest),
            };

            match entry.get_ref() {
                DeValue::Table(inner) => walk(inner, rest),
                _ => None,
            }
        }
        _ => None,
    }
}

fn segments(path: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut quoted = false;

    for c in path.chars() {
        match c {
            '"' => quoted = !quoted,
            '.' if !quoted => out.push(std::mem::take(&mut current)),
            c => current.push(c),
        }
    }

    out.push(current);
    out
}

/// The first key called `name` at any depth.
fn find_key(table: &DeTable<'_>, name: &str) -> Option<Range<usize>> {
    for (key, value) in table {
        if key.get_ref().as_ref() == name {
            return Some(key.span());
        }

        let inner = match value.get_ref() {
            DeValue::Table(inner) => find_key(inner, name),
            DeValue::Array(items) => items.iter().find_map(|item| match item.get_ref() {
                DeValue::Table(inner) => find_key(inner, name),
                _ => None,
            }),
            _ => None,
        };

        if inner.is_some() {
            return inner;
        }
    }

    None
}
