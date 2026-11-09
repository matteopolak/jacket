//! Names written in types, and the declarations they resolve to: what lets the site link a type
//! in a signature to the right page. The reading is `Reader::type_refs` in `model.rs`.
use super::model::{json, path};
use std::path::PathBuf;

/// One hop of a written name's alias chain. Repeated paths are ordered nearest first;
/// the site chooses the first hop whose declaration has a page.
#[derive(Debug, Default)]
pub struct TypeRef {
    /// As written: `Table` or `Hash.Table`.
    pub path: String,
    pub file: PathBuf,
    pub line: usize,
    /// Where the declared name starts in `file`, 0 when it is not known.
    pub at: usize,
    /// This hop belongs to a private/internal/prelude scope.
    pub private: bool,
}

/// Types the language provides, which have no declaration to link to.
pub(super) const BUILTIN_TYPES: &[&str] = &[
    "int", "float", "float32", "float64", "bool", "string", "void", "Type", "Any", "s8", "s16",
    "s32", "s64", "u8", "u16", "u32", "u64", "s128", "u128", "code", "Code",
];

/// The names in a type as written, `Hash.Table` as one: `[3] Hash.Table(string, T)` has
/// `Hash.Table`, `string` and `T`. Directive identifiers are syntax: `*#Context` contributes no
/// declaration name. Names the type binds for itself are not references either: a `$T` and every
/// later use of `T` in it, a parameter's name in `(a: T)`, and a named argument's in `F(key = V)`.
pub(super) fn type_paths(ty: &str) -> Vec<Vec<String>> {
    let chars: Vec<char> = ty.chars().collect();
    let bound = bound_names(&chars);
    // The first character after the spaces from `at`.
    let after = |at: usize| {
        let at = (at..chars.len()).find(|&n| chars[n] != ' ')?;
        Some((chars[at], chars.get(at + 1).copied()))
    };
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        // Literal text is data, even when it happens to spell a declaration's name.
        if c == '"' {
            i += 1;
            while i < chars.len() {
                if chars[i] == '\\' {
                    i = (i + 2).min(chars.len());
                } else if chars[i] == '"' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
            continue;
        }
        // A directive is language syntax, not a same-spelled declaration: #Context
        // must not resolve to a user type named Context. Its operands are still read.
        if c == '#' {
            i += 1;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            continue;
        }
        if c.is_ascii_digit() {
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            continue;
        }
        if !(c.is_alphabetic() || c == '_') {
            i += 1;
            continue;
        }
        let baked = i > 0 && chars[i - 1] == '$';
        let first = i;
        // `.NONE` is a member of a type the context knows, not a name of its own; `..T` is variadic.
        let leading_dot = i > 0 && chars[i - 1] == '.' && !(i > 1 && chars[i - 2] == '.');
        let mut path = Vec::new();
        loop {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            path.push(chars[start..i].iter().collect::<String>());
            let more = chars.get(i) == Some(&'.')
                && chars
                    .get(i + 1)
                    .is_some_and(|n| n.is_alphabetic() || *n == '_');
            if !more {
                break;
            }
            i += 1;
        }
        // `a: T` declares `a`; `key = V` names a parameter. `::`, `:=` and `==` are neither.
        let named_argument = matches!(after(i), Some(('=', next)) if next != Some('='))
            && chars[..first]
                .iter()
                .rev()
                .find(|c| **c != ' ')
                .is_some_and(|c| matches!(c, '(' | ','));
        let declares = matches!(after(i), Some((':', next)) if !matches!(next, Some(':' | '=')))
            || named_argument;
        if !baked && !declares && !leading_dot && !bound.contains(&path[0]) {
            out.push(path);
        }
    }
    out
}

/// The names `$Name` and `$$Name` bind in a type written as `chars`.
fn bound_names(chars: &[char]) -> Vec<String> {
    let mut names = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '$' {
            i += 1;
            continue;
        }
        while chars.get(i) == Some(&'$') {
            i += 1;
        }
        let start = i;
        while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
            i += 1;
        }
        if i > start {
            names.push(chars[start..i].iter().collect());
        }
    }
    names
}

/// `,"refs":[["Hash.Table","Hash_Table/table.jai",12]]`, or nothing when no name resolved.
pub(super) fn refs_json(refs: &[TypeRef]) -> String {
    refs_named("refs", refs)
}

/// [`refs_json`] under another key.
pub(super) fn refs_named(key: &str, refs: &[TypeRef]) -> String {
    if refs.is_empty() {
        return String::new();
    }
    let rows: Vec<String> = refs
        .iter()
        .map(|r| {
            format!(
                "[{},{},{},{}]",
                json(&r.path),
                path(&r.file),
                r.line,
                r.private
            )
        })
        .collect();
    format!(",\"{key}\":[{}]", rows.join(","))
}
