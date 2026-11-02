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
    /// This hop belongs to a private/internal/prelude or module-parameter helper scope.
    pub private: bool,
}

/// Types the language provides, which have no declaration to link to.
pub(super) const BUILTIN_TYPES: &[&str] = &[
    "int", "float", "float32", "float64", "bool", "string", "void", "Type", "Any", "s8", "s16",
    "s32", "s64", "u8", "u16", "u32", "u64", "s128", "u128", "code", "Code",
];

/// The names in a type as written, `Hash.Table` as one: `[3] Hash.Table(string, T)` has
/// `Hash.Table`, `string` and `T`. A `$T` that the call binds is left out.
pub(super) fn type_paths(ty: &str) -> Vec<Vec<String>> {
    let chars: Vec<char> = ty.chars().collect();
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
        if !baked {
            out.push(path);
        }
    }
    out
}

/// `,"refs":[["Hash.Table","Hash_Table/table.jai",12]]`, or nothing when no name resolved.
pub(super) fn refs_json(refs: &[TypeRef]) -> String {
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
    format!(",\"refs\":[{}]", rows.join(","))
}
