//! The documentation model of a library: what its module page and item pages say, read from
//! the parsed files with the same rules the editor uses for hover. `jaic -doc_model DIR` prints
//! it as JSON for `jacket doc`, which renders the site and never reads Jai itself.
//!
//! An item is a public constant of the module: a procedure, struct, enum, type or plain
//! constant. Aliases (`split :: Impl.split;`) are followed to what they name, so the item has
//! the kind, signature and docs of its target; docs written on the alias itself win. Procedures
//! declared several times under one name are the overloads of one item.
use super::forward::Files;
use crate::ast::{Decl, DeclKind, EnumItem, ExprKind, ProcHeader, StmtKind, StructKind};
use crate::docs::{self, Resolver};
use crate::fxhash::HashMap;
use crate::parser::ide_facts::{self, DeclClass, TopLevelKind, Visibility};
use crate::parser::{IdeParse, parse_file_ide};
use crate::source::FileId;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// How many aliases are followed to the declaration they name.
const MAX_HOPS: usize = 8;

/// A longest constant value shown, in bytes.
const MAX_VALUE: usize = 160;

#[derive(Debug, Default)]
pub struct Param {
    pub name: String,
    pub ty: String,
    pub default: String,
    pub doc: String,
}

#[derive(Debug, Default)]
pub struct Overload {
    pub file: PathBuf,
    pub line: usize,
    /// The header as written, on one line, from its `(`: `(s: string, byte: u8) -> (...)`.
    pub signature: String,
    pub params: Vec<Param>,
    pub returns: Vec<Param>,
    /// Its docs, as Markdown with the parameter list items removed and links resolved.
    pub docs: String,
    pub docs_line: usize,
    /// Notes the header carries that change how it is called (`#c_call`, `inline`, a macro).
    pub flags: Vec<&'static str>,
}

/// A struct field or an enum member.
#[derive(Debug, Default)]
pub struct Member {
    pub name: String,
    /// The field's type, or the member's value.
    pub ty: String,
    pub doc: String,
}

#[derive(Debug, Default)]
pub struct Item {
    pub name: String,
    /// `procedure`, `struct`, `union`, `enum`, `enum_flags`, `type` or `constant`.
    pub kind: &'static str,
    pub file: PathBuf,
    pub line: usize,
    /// Docs written on the item itself, which for an alias is the alias's own comment.
    pub docs: String,
    pub overloads: Vec<Overload>,
    pub members: Vec<Member>,
    /// What a constant is set to, as written.
    pub value: String,
}

#[derive(Debug, Default)]
pub struct Module {
    pub file: PathBuf,
    pub docs: String,
    pub docs_line: usize,
    /// The targets the module asserts it builds for (`MACOS`), empty for any.
    pub platforms: Vec<String>,
    pub items: Vec<Item>,
}

/// A parsed file and its text, read once.
struct Parsed {
    text: Rc<str>,
    parse: IdeParse,
}

struct Reader<'a> {
    files: Files<'a>,
    parsed: HashMap<PathBuf, Option<Rc<Parsed>>>,
}

fn line_of(text: &str, at: usize) -> usize {
    text[..at].matches('\n').count() + 1
}

impl Reader<'_> {
    fn parsed(&mut self, file: &Path) -> Option<Rc<Parsed>> {
        if let Some(found) = self.parsed.get(file) {
            return found.clone();
        }
        let loaded = self.files.text(&file.to_path_buf()).map(|text| {
            let parse = parse_file_ide(FileId(0), &text);
            Rc::new(Parsed {
                text,
                parse,
            })
        });
        self.parsed.insert(file.to_path_buf(), loaded.clone());
        loaded
    }

    /// The declaration an alias at `at` of `file` finally names (itself when it names nothing
    /// that can be found).
    fn target(&mut self, file: &Path, at: usize) -> (PathBuf, usize) {
        let (mut file, mut at) = (file.to_path_buf(), at);
        for _ in 0..MAX_HOPS {
            let Some(text) = self.files.text(&file) else {
                break;
            };
            let Some(path) = docs::alias_target(&text, at) else {
                break;
            };
            match self.files.resolve(&file, at, &path) {
                Some(next) if next != (file.clone(), at) => (file, at) = next,
                _ => break,
            }
        }
        (file, at)
    }
}

fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn slice(text: &str, start: u32, end: u32) -> String {
    text.get(start as usize..end as usize)
        .map(collapse)
        .unwrap_or_default()
}

fn class_kind(decl: &Decl) -> &'static str {
    match ide_facts::decl_class(decl) {
        DeclClass::Procedure => "procedure",
        DeclClass::Struct => match decl.value.as_ref().map(|v| &v.kind) {
            Some(ExprKind::Struct(lit)) if lit.kind == StructKind::Union => "union",
            _ => "struct",
        },
        DeclClass::Enum => match decl.value.as_ref().map(|v| &v.kind) {
            Some(ExprKind::Enum(lit)) if lit.flags_enum => "enum_flags",
            _ => "enum",
        },
        DeclClass::TypeAlias => "type",
        _ => "constant",
    }
}

fn clip(mut value: String) -> String {
    if value.len() > MAX_VALUE {
        let cut = value.floor_char_boundary(MAX_VALUE);
        value.truncate(cut);
        value.push_str(" ...");
    }
    value
}

/// The parameters and results of `header` with their docs, and the docs without the list items
/// that described them.
fn describe(
    header: &ProcHeader,
    text: &str,
    at: usize,
    doc: &str,
) -> (Vec<Param>, Vec<Param>, String) {
    let mut documented = docs::header_open(text, at)
        .map(|open| docs::parameters(text, open))
        .unwrap_or_default();
    // The list in the docs above the header can also name results.
    for (name, item) in docs::list_parameters(doc) {
        if !documented.iter().any(|(n, _)| *n == name) {
            documented.push((name, item));
        }
    }
    let doc_of = |name: &str| {
        documented
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, d)| d.clone())
            .unwrap_or_default()
    };
    let params: Vec<Param> = header
        .params
        .iter()
        .map(|p| {
            let name = p
                .name
                .as_ref()
                .map_or(String::new(), |n| n.name.as_str().to_owned());
            let mut ty =
                p.ty.as_ref()
                    .map_or(String::new(), |t| slice(text, t.span.start, t.span.end));
            if p.variadic {
                ty.insert_str(0, "..");
            }
            Param {
                doc: doc_of(&name),
                name: if p.baked {
                    format!("${name}")
                } else {
                    name
                },
                ty,
                default: p
                    .default
                    .as_ref()
                    .map_or(String::new(), |d| slice(text, d.span.start, d.span.end)),
            }
        })
        .collect();
    let returns: Vec<Param> = header
        .returns
        .iter()
        .map(|r| {
            let name = r
                .name
                .as_ref()
                .map_or(String::new(), |n| n.name.as_str().to_owned());
            Param {
                doc: if name.is_empty() {
                    String::new()
                } else {
                    doc_of(&name)
                },
                ty: r
                    .ty
                    .as_ref()
                    .map_or(String::new(), |t| slice(text, t.span.start, t.span.end)),
                default: r
                    .default
                    .as_ref()
                    .map_or(String::new(), |d| slice(text, d.span.start, d.span.end)),
                name,
            }
        })
        .collect();
    let used: Vec<&str> = params
        .iter()
        .chain(&returns)
        .filter(|p| !p.doc.is_empty())
        .map(|p| p.name.trim_start_matches('$'))
        .collect();
    let rest = docs::without_items(doc, &used);
    (params, returns, rest)
}

fn flags_of(header: &ProcHeader) -> Vec<&'static str> {
    let mut out = Vec::new();
    if header.flags.c_call() {
        out.push("#c_call");
    }
    if header.foreign.is_some() {
        out.push("#foreign");
    }
    out
}

fn overload(
    reader: &mut Reader<'_>,
    file: &Path,
    text: &str,
    decl: &Decl,
    header: &ProcHeader,
) -> Overload {
    let name = &decl.names[0];
    let at = name.span.start as usize;
    let doc = docs::declaration(text, at).unwrap_or_default();
    let (params, returns, docs_text) = describe(header, text, at, &doc);
    let docs_line = docs::located_above(text, at).map_or(0, |d| d.line);
    let _ = reader;
    Overload {
        file: file.to_path_buf(),
        line: line_of(text, at),
        signature: slice(text, header.span.start, header.span.end),
        params,
        returns,
        docs: docs_text,
        docs_line,
        flags: flags_of(header),
    }
}

fn members(decl: &Decl, text: &str) -> Vec<Member> {
    let mut out = Vec::new();
    match decl.value.as_ref().map(|v| &v.kind) {
        Some(ExprKind::Struct(lit)) => {
            for stmt in &lit.body {
                let StmtKind::Decl(field) = &stmt.kind else {
                    continue;
                };
                if field.kind != DeclKind::Var {
                    continue;
                }
                for name in &field.names {
                    out.push(Member {
                        name: name.name.as_str().to_owned(),
                        ty: field.ty.as_ref().map_or_else(
                            || {
                                field.value.as_ref().map_or(String::new(), |v| {
                                    format!(":= {}", slice(text, v.span.start, v.span.end))
                                })
                            },
                            |t| slice(text, t.span.start, t.span.end),
                        ),
                        doc: docs::declaration(text, name.span.start as usize).unwrap_or_default(),
                    });
                }
            }
        }
        Some(ExprKind::Enum(lit)) => {
            fn walk(items: &[EnumItem], text: &str, out: &mut Vec<Member>) {
                for item in items {
                    match item {
                        EnumItem::Member(m) => out.push(Member {
                            name: m.name.name.as_str().to_owned(),
                            ty: m
                                .value
                                .as_ref()
                                .map_or(String::new(), |v| slice(text, v.span.start, v.span.end)),
                            doc: docs::declaration(text, m.name.span.start as usize)
                                .unwrap_or_default(),
                        }),
                        EnumItem::If {
                            then_items,
                            else_items,
                            ..
                        } => {
                            walk(then_items, text, out);
                            walk(else_items, text, out);
                        }
                        _ => {}
                    }
                }
            }
            walk(&lit.items, text, &mut out);
        }
        _ => {}
    }
    out
}

/// Whether a file declares foreign procedures, which mirrors a native API documented by its
/// vendor; its declarations are not listed.
fn is_binding(parsed: &Parsed) -> bool {
    ide_facts::top_level(&parsed.parse.file).iter().any(|item| {
        matches!(item.kind, TopLevelKind::Decl(decl)
            if decl.kind == DeclKind::Const
                && ide_facts::decl_class(decl) == DeclClass::Procedure
                && !matches!(decl.value.as_ref().map(|v| &v.kind),
                    Some(ExprKind::Proc(lit)) if lit.body.is_some()))
    })
}

/// The documentation model of the library in `dir`. `stdlib` is where the `internal/` modules
/// and other imports are found.
pub fn module(dir: &Path, stdlib: &Path) -> Module {
    let mut own = Vec::new();
    super::own_files(dir, true, &mut own);
    let module_file = super::canonical(&crate::paths::module_entry(dir));
    let mut reader = Reader {
        files: Files::new(stdlib),
        parsed: HashMap::default(),
    };
    // The module's own file first, so items come in the order it lists them.
    own.sort_by_key(|f| super::canonical(f) != module_file);
    let mut queue: Vec<PathBuf> = own;
    queue.reverse();
    let mut seen = BTreeSet::new();
    let mut out = Module {
        file: module_file.clone(),
        ..Module::default()
    };
    let mut names: Vec<String> = Vec::new();
    // (item index, alias comment) to resolve links once every name is known.
    while let Some(file) = queue.pop() {
        let file = super::canonical(&file);
        if !seen.insert(file.clone()) {
            continue;
        }
        let Some(parsed) = reader.parsed(&file) else {
            continue;
        };
        let text = parsed.text.clone();
        let base = file.parent().unwrap_or(Path::new(".")).to_path_buf();
        let binding = is_binding(&parsed);
        let mut first_statement = None;
        let mut loads = Vec::new();
        for item in ide_facts::top_level(&parsed.parse.file) {
            first_statement.get_or_insert(item.span.start as usize);
            match item.kind {
                TopLevelKind::Load(path, _) => loads.push(base.join(path)),
                TopLevelKind::Assert(assert) if file == module_file => {
                    if let Some(targets) = ide_facts::asserted_targets(assert) {
                        for t in targets {
                            let t = t.as_str().to_owned();
                            if !out.platforms.contains(&t) {
                                out.platforms.push(t);
                            }
                        }
                    }
                }
                TopLevelKind::Decl(decl) if decl.kind == DeclKind::Const => {
                    if binding || item.visibility != Visibility::Export {
                        continue;
                    }
                    let Some(name) = decl.names.first() else {
                        continue;
                    };
                    let at = name.span.start as usize;
                    let own_docs = docs::above(&text, at).or_else(|| docs::trailing(&text, at));
                    let (tfile, tat) = reader.target(&file, at);
                    let Some(target) = reader.parsed(&tfile) else {
                        continue;
                    };
                    let tname = ide_facts::top_level(&target.parse.file)
                        .into_iter()
                        .find_map(|t| match t.kind {
                            TopLevelKind::Decl(d)
                                if d.names.iter().any(|n| n.span.start as usize == tat) =>
                            {
                                Some(d)
                            }
                            _ => None,
                        });
                    let Some(tdecl) = tname else {
                        // An alias of something this reader cannot find: a plain constant.
                        let value = decl
                            .value
                            .as_ref()
                            .map_or(String::new(), |v| slice(&text, v.span.start, v.span.end));
                        add(
                            &mut out,
                            &mut names,
                            Item {
                                name: name.name.as_str().to_owned(),
                                kind: "constant",
                                file: file.clone(),
                                line: line_of(&text, at),
                                docs: own_docs.unwrap_or_default(),
                                value: clip(value),
                                ..Item::default()
                            },
                        );
                        continue;
                    };
                    let kind = class_kind(tdecl);
                    let mut found = Item {
                        name: name.name.as_str().to_owned(),
                        kind,
                        file: tfile.clone(),
                        line: line_of(&target.text, tat),
                        docs: own_docs.unwrap_or_default(),
                        ..Item::default()
                    };
                    if kind == "procedure" {
                        // Every declaration of the name in the target's file is an overload.
                        for sibling in ide_facts::top_level(&target.parse.file) {
                            let TopLevelKind::Decl(d) = sibling.kind else {
                                continue;
                            };
                            if d.kind != DeclKind::Const
                                || !d
                                    .names
                                    .iter()
                                    .any(|n| n.name.as_str() == tdecl.names[0].name.as_str())
                                || sibling.visibility == Visibility::File
                            {
                                continue;
                            }
                            if let Some(header) = ide_facts::decl_header(d) {
                                let o = overload(&mut reader, &tfile, &target.text, d, header);
                                found.overloads.push(o);
                            }
                        }
                    } else {
                        found.members = members(tdecl, &target.text);
                        if found.docs.is_empty() {
                            found.docs = docs::declaration(&target.text, tat).unwrap_or_default();
                        }
                        if kind == "constant" || kind == "type" {
                            found.value = clip(tdecl.value.as_ref().map_or(String::new(), |v| {
                                slice(&target.text, v.span.start, v.span.end)
                            }));
                        }
                    }
                    add(&mut out, &mut names, found);
                }
                _ => {}
            }
        }
        queue.extend(loads.into_iter().rev());
        if file == module_file {
            let opening = docs::opening(&text).filter(|o| {
                first_statement
                    .and_then(|at| docs::located_above(&text, at))
                    .is_none_or(|first| first.line != o.line)
            });
            if let Some(o) = opening {
                out.docs = o.text;
                out.docs_line = o.line;
            }
        }
    }
    out
}

fn add(out: &mut Module, names: &mut Vec<String>, item: Item) {
    if let Some(i) = names.iter().position(|n| *n == item.name) {
        let have = &mut out.items[i];
        if have.kind == "procedure" && item.kind == "procedure" {
            for o in item.overloads {
                if !have
                    .overloads
                    .iter()
                    .any(|h| (&h.file, h.line) == (&o.file, o.line))
                {
                    have.overloads.push(o);
                }
            }
        }
        return;
    }
    names.push(item.name.clone());
    out.items.push(item);
}

/// `field` as a JSON string literal.
fn json(field: &str) -> String {
    let mut out = String::with_capacity(field.len() + 2);
    out.push('"');
    for c in field.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn path(p: &Path) -> String {
    json(&p.to_string_lossy())
}

fn params_json(list: &[Param]) -> String {
    let rows: Vec<String> = list
        .iter()
        .map(|p| {
            format!(
                "{{\"name\":{},\"type\":{},\"default\":{},\"doc\":{}}}",
                json(&p.name),
                json(&p.ty),
                json(&p.default),
                json(&p.doc)
            )
        })
        .collect();
    format!("[{}]", rows.join(","))
}

/// The model as JSON. `resolve` gives a URL for a link's names, as for [`docs::markdown`]; docs
/// come out as Markdown with the links it knows rewritten.
pub fn to_json(model: &Module, resolve: &dyn Fn(&[String]) -> Option<String>) -> String {
    let md = |doc: &str| docs::linked(doc, resolve);
    let mut items = Vec::new();
    for item in &model.items {
        let overloads: Vec<String> = item
            .overloads
            .iter()
            .map(|o| {
                format!(
                    "{{\"file\":{},\"line\":{},\"signature\":{},\"params\":{},\"returns\":{},\"docs\":{},\"docs_line\":{},\"flags\":[{}]}}",
                    path(&o.file),
                    o.line,
                    json(&o.signature),
                    params_json(&o.params),
                    params_json(&o.returns),
                    json(&md(&o.docs)),
                    o.docs_line,
                    o.flags.iter().map(|f| json(f)).collect::<Vec<_>>().join(",")
                )
            })
            .collect();
        let members: Vec<String> = item
            .members
            .iter()
            .map(|m| {
                format!(
                    "{{\"name\":{},\"type\":{},\"doc\":{}}}",
                    json(&m.name),
                    json(&m.ty),
                    json(&md(&m.doc))
                )
            })
            .collect();
        items.push(format!(
            "{{\"name\":{},\"kind\":{},\"file\":{},\"line\":{},\"docs\":{},\"value\":{},\"overloads\":[{}],\"members\":[{}]}}",
            json(&item.name),
            json(item.kind),
            path(&item.file),
            item.line,
            json(&md(&item.docs)),
            json(&item.value),
            overloads.join(","),
            members.join(",")
        ));
    }
    format!(
        "{{\"file\":{},\"docs\":{},\"docs_line\":{},\"platforms\":[{}],\"items\":[\n{}\n]}}\n",
        path(&model.file),
        json(&md(&model.docs)),
        model.docs_line,
        model
            .platforms
            .iter()
            .map(|p| json(p))
            .collect::<Vec<_>>()
            .join(","),
        items.join(",\n")
    )
}

/// What `jaic -doc_model` prints for the library in `dir`. Links in docs to a name of the
/// library (or `Module.name` of another library of `stdlib`) become `jaidoc:Module/name` and
/// `jaidoc:Module/name#member`, for the renderer to turn into pages.
pub fn print(dir: &Path, stdlib: &Path) -> String {
    let (dir, stdlib) = (&super::canonical(dir), &super::canonical(stdlib));
    let model = module(dir, stdlib);
    let key = {
        let canonical = super::canonical(dir);
        let root = super::canonical(stdlib);
        canonical.strip_prefix(&root).map_or_else(
            |_| {
                canonical
                    .file_name()
                    .map_or(String::new(), |n| n.to_string_lossy().into_owned())
            },
            crate::paths::module_key,
        )
    };
    let own: BTreeSet<&str> = model.items.iter().map(|i| i.name.as_str()).collect();
    let resolve = |names: &[String]| -> Option<String> {
        let first = names.first()?;
        if own.contains(first.as_str()) {
            return Some(match names.get(1) {
                Some(member) => format!("jaidoc:{key}/{first}#{member}"),
                None => format!("jaidoc:{key}/{first}"),
            });
        }
        let other = names.get(1)?;
        let module_dir = stdlib.join(first);
        let ext_dir = stdlib.join("Extensions").join(first);
        if crate::paths::module_entry(&module_dir).is_file() {
            return Some(format!("jaidoc:{first}/{other}"));
        }
        if crate::paths::module_entry(&ext_dir).is_file() {
            return Some(format!("jaidoc:Extensions/{first}/{other}"));
        }
        None
    };
    to_json(&model, &resolve)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn library(files: &[(&str, &str)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("jaic-doc-model-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for (file, text) in files {
            let path = dir.join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        dir
    }

    #[test]
    fn aliases_overloads_and_members_become_items() {
        let dir = library(&[
            (
                "module.jai",
                "// A module.\n\nImpl :: #import,file \"impl.jai\";\n\nsplit :: Impl.split;\nrenamed :: split;\nPoint :: Impl.Point;\nLIMIT :: Impl.LIMIT;\n\n#scope_module\n\nhidden :: () {}\n",
            ),
            (
                "impl.jai",
                "// Splits `s` at `byte`.\n//\n// - s: the text\n// - found: whether it was there\nsplit :: (s: string, byte: u8) -> (found: bool) { return true; }\n\n// Splits at a string.\nsplit :: (s: string, by: string, n := 1) -> bool { return true; }\n\nPoint :: struct {\n    x: int; // across\n    y: int;\n}\n\n// The most.\nLIMIT :: 4;\n",
            ),
        ]);
        let model = module(&dir, Path::new("/nowhere"));
        assert_eq!(model.docs, "A module.");
        let names: Vec<_> = model
            .items
            .iter()
            .map(|i| (i.name.as_str(), i.kind))
            .collect();
        assert_eq!(
            names,
            [
                ("split", "procedure"),
                ("renamed", "procedure"),
                ("Point", "struct"),
                ("LIMIT", "constant")
            ]
        );
        let split = &model.items[0];
        assert_eq!(split.overloads.len(), 2);
        let first = &split.overloads[0];
        assert_eq!(first.params[0].doc, "the text");
        assert_eq!(first.returns[0].doc, "whether it was there");
        assert_eq!(first.docs, "Splits `s` at `byte`.");
        assert_eq!(split.overloads[1].params[2].default, "1");
        assert_eq!(model.items[1].overloads.len(), 2);
        assert_eq!(model.items[2].members[0].doc, "across");
        assert_eq!(model.items[3].value, "4");
        assert_eq!(model.items[3].docs, "The most.");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
