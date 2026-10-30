//! Names written in types, and the declarations they resolve to: what lets the site link a type
//! in a signature to the right page. The reading is `Reader::type_refs` in `model.rs`.
use super::model::{Module, json, path};
use std::collections::BTreeSet;
use std::path::PathBuf;

/// A name written in a type and the declaration it resolves to.
#[derive(Debug, Default)]
pub struct TypeRef {
    /// As written: `Table` or `Hash.Table`.
    pub path: String,
    pub file: PathBuf,
    pub line: usize,
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

/// A type declared once per target (`#if OS == .MACOS` and its other branch) is one item, which
/// stands for the declaration that is documented. A name that resolved to a branch that is not
/// documented (a binding's, say) is pointed at the item of that name.
pub(super) fn reconcile_refs(out: &mut Module) {
    let documented: Vec<(String, PathBuf, usize)> = out
        .items
        .iter()
        .filter(|i| matches!(i.kind, "struct" | "union" | "enum" | "enum_flags" | "type"))
        .map(|i| (i.name.clone(), i.file.clone(), i.line))
        .collect();
    let known: BTreeSet<(PathBuf, usize)> = out
        .items
        .iter()
        .flat_map(|i| std::iter::once((i.file.clone(), i.line)).chain(i.also.iter().cloned()))
        .collect();
    let fix = |r: &mut TypeRef| {
        if r.path.contains('.') || known.contains(&(r.file.clone(), r.line)) {
            return;
        }
        if let Some((_, file, line)) = documented.iter().find(|(name, ..)| *name == r.path) {
            r.file = file.clone();
            r.line = *line;
        }
    };
    for item in &mut out.items {
        for o in &mut item.overloads {
            for p in o.params.iter_mut().chain(o.returns.iter_mut()) {
                p.refs.iter_mut().for_each(fix);
            }
        }
        for m in &mut item.members {
            m.refs.iter_mut().for_each(fix);
        }
    }
}

/// `,"refs":[["Hash.Table","Hash_Table/table.jai",12]]`, or nothing when no name resolved.
pub(super) fn refs_json(refs: &[TypeRef]) -> String {
    if refs.is_empty() {
        return String::new();
    }
    let rows: Vec<String> = refs
        .iter()
        .map(|r| format!("[{},{},{}]", json(&r.path), path(&r.file), r.line))
        .collect();
    format!(",\"refs\":[{}]", rows.join(","))
}
