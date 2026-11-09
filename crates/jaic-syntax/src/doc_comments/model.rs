//! The documentation model of a library: what its module page and item pages say, read from
//! the parsed files with the same rules the editor uses for hover. `jaic -doc_model DIR` prints
//! it as JSON for `jacket doc`, which renders the site and never reads Jai itself.
//!
//! An item is a public constant of the module: a procedure, struct, enum, type or plain
//! constant. Aliases (`split :: Impl.split;`) are followed to what they name, so the item has
//! the kind, signature and docs of its target; docs written on the alias itself win. Procedures
//! declared several times under one name are the overloads of one item.
use super::forward::Files;
#[path = "model_declaration.rs"]
mod declaration;
#[path = "model_json.rs"]
mod output;
#[path = "model_overloads.rs"]
mod overloads;
#[path = "model_resolution.rs"]
mod resolution;
use super::refs::{TypeRef, refs_json, refs_named};
use crate::ast::{
    BinOp, Decl, DeclKind, EnumItem, Expr, ExprKind, File, ProcHeader, Stmt, StmtKind, StructKind,
    UnOp,
};
use crate::docs::{self, Resolver};
use crate::fxhash::HashMap;
use crate::parser::ide_facts::{self, DeclClass, TopLevelKind, Visibility};
use crate::parser::{IdeParse, parse_file_ide};
use crate::source::FileId;
pub use output::to_json;
pub(in crate::doc_comments) use output::{json, path};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

/// How many aliases are followed to the declaration they name.
const MAX_HOPS: usize = 8;

/// A longest constant value shown, in bytes.
const MAX_VALUE: usize = 160;

/// The operating systems a declaration can be limited to by an enclosing `#if`, one bit each.
const OSES: [(&str, u8); 6] = [
    ("MACOS", 1),
    ("LINUX", 2),
    ("WINDOWS", 4),
    ("ANDROID", 8),
    ("IOS", 16),
    ("WASM", 32),
];

/// Every operating system: no `#if` limits the declaration.
const EVERY_OS: u8 = 63;

/// Whether `expr` holds when `OS` is `os`: `None` when it depends on anything else (`CPU`, a
/// constant), which counts as possible.
fn holds_on(expr: &Expr, os: &str) -> Option<bool> {
    let is_os = |e: &Expr| matches!(&e.kind, ExprKind::Ident(n) if n.as_str() == "OS");
    match &expr.kind {
        ExprKind::Binary(op @ (BinOp::Eq | BinOp::Ne), left, right) if is_os(left) => {
            let ExprKind::InferredMember(target) = &right.kind else {
                return None;
            };
            Some((target.name.as_str() == os) == (*op == BinOp::Eq))
        }
        ExprKind::Binary(BinOp::And, left, right) => {
            match (holds_on(left, os), holds_on(right, os)) {
                (Some(false), _) | (_, Some(false)) => Some(false),
                (Some(true), Some(true)) => Some(true),
                _ => None,
            }
        }
        ExprKind::Binary(BinOp::Or, left, right) => {
            match (holds_on(left, os), holds_on(right, os)) {
                (Some(true), _) | (_, Some(true)) => Some(true),
                (Some(false), Some(false)) => Some(false),
                _ => None,
            }
        }
        ExprKind::Unary(UnOp::Not, inner) => holds_on(inner, os).map(|v| !v),
        _ => None,
    }
}

/// The systems left by taking the branch of `cond` (`taken` is false for the `else`).
fn narrowed(mask: u8, cond: &Expr, taken: bool) -> u8 {
    OSES.iter()
        .filter(|(name, _)| holds_on(cond, name) != Some(!taken))
        .fold(0, |acc, (_, bit)| acc | bit)
        & mask
}

/// For each declaration inside an `#if` that limits the operating system, the systems it is
/// declared for, by the start of its first name.
fn limits(file: &File) -> HashMap<u32, u8> {
    fn walk(stmts: &[Stmt], mask: u8, out: &mut HashMap<u32, u8>) {
        for stmt in stmts {
            match &stmt.kind {
                StmtKind::StaticIf(branches) => {
                    walk(
                        &branches.then_branch,
                        narrowed(mask, &branches.cond, true),
                        out,
                    );
                    walk(
                        &branches.else_branch,
                        narrowed(mask, &branches.cond, false),
                        out,
                    );
                }
                StmtKind::Decl(decl) if mask != EVERY_OS => {
                    for name in &decl.names {
                        out.insert(name.span.start, mask);
                    }
                }
                _ => {}
            }
        }
    }
    let mut out = HashMap::default();
    walk(&file.stmts, EVERY_OS, &mut out);
    out
}

/// The systems named by `names`; every one for none.
fn os_mask(names: &[String]) -> u8 {
    if names.is_empty() {
        return EVERY_OS;
    }
    OSES.iter()
        .filter(|(name, _)| names.iter().any(|n| n == name))
        .fold(0, |acc, (_, bit)| acc | bit)
}

/// The names of the systems in `mask`; empty for every one.
fn os_names(mask: u8) -> Vec<String> {
    if mask == EVERY_OS {
        return Vec::new();
    }
    OSES.iter()
        .filter(|(_, bit)| mask & bit != 0)
        .map(|(name, _)| (*name).to_owned())
        .collect()
}

#[derive(Debug, Default)]
pub struct Param {
    pub name: String,
    pub ty: String,
    pub default: String,
    pub doc: String,
    /// The declarations the names in `ty` and `default` stand for.
    pub refs: Vec<TypeRef>,
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
    /// The systems an enclosing `#if` limits it to; empty for any.
    pub platforms: Vec<String>,
}

/// A struct field or an enum member.
#[derive(Debug, Default)]
pub struct Member {
    pub name: String,
    /// The field's type, or the member's value.
    pub ty: String,
    pub doc: String,
    /// The declarations the names in a field's type stand for.
    pub refs: Vec<TypeRef>,
    /// Where the name is written, as an offset in its file.
    pub at: usize,
    /// The line of the item's `declaration` that holds it, counted from the line of the item's
    /// name; set with the declaration.
    pub line: Option<usize>,
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
    /// Struct parameters whose names are bound locally rather than documented items.
    pub scope: Vec<String>,
    /// The declarations the names in `value` stand for (`Close_Proc :: #type (...) -> Io_Error`).
    pub refs: Vec<TypeRef>,
    /// The systems the `#if`s around it (and around what it names) limit it to; empty for any.
    pub platforms: Vec<String>,
    /// Whether its docs carry the `@experimental` tag (on the item or any overload).
    pub experimental: bool,
    /// Whether it is declared in a file of foreign procedures, a native API its vendor documents.
    pub binding: bool,
    /// Other declarations of the name (one per branch of an `#if`) that this item stands for.
    pub also: Vec<(PathBuf, usize)>,
    /// A struct, union or enum as written (`Channel :: struct(T: Type) { ... }`), without its
    /// field and member docs (those are on `members`); other comments stay.
    pub declaration: String,
    /// The declarations the type names in `declaration` stand for.
    pub decl_refs: Vec<TypeRef>,
    /// Not exported: a type that public items name, which cannot be named outside the module.
    pub private: bool,
    /// The systems this declaration alone is for; empty for any. `platforms` covers every
    /// declaration of the name.
    pub defined_on: Vec<String>,
    /// The definitions of the name that differ from this one, one per branch of an `#if`
    /// (`File_Watcher` is one struct on macOS and another elsewhere), each with its own
    /// `defined_on` in `platforms`. Empty when the name has one definition, or every branch
    /// says the same.
    pub variants: Vec<Item>,
}

/// A parameter of the module (`#module_parameters`).
#[derive(Debug, Default)]
pub struct Parameter {
    pub name: String,
    /// As written; empty when the type comes from the default.
    pub ty: String,
    pub default: String,
    pub doc: String,
    /// The declarations the names in `ty` and `default` stand for.
    pub refs: Vec<TypeRef>,
}

#[derive(Debug, Default)]
pub struct Module {
    pub file: PathBuf,
    /// Every program gets it without importing it (the prelude).
    pub implicit: bool,
    pub docs: String,
    pub docs_line: usize,
    /// The targets the module asserts it builds for (`MACOS`), empty for any.
    pub platforms: Vec<String>,
    /// Whether the opening comment carries the `@experimental` tag.
    pub experimental: bool,
    pub parameters: Vec<Parameter>,
    pub items: Vec<Item>,
    /// The file and line of each top-level `#insert` that is not private to its file. What it
    /// declares is not in `items`: the model reads source and never runs it.
    pub inserts: Vec<(PathBuf, usize)>,
}

/// A parsed file and its text, read once.
struct Parsed {
    text: Arc<str>,
    parse: IdeParse,
    /// Declarations limited by an `#if` on `OS`; see [`limits`].
    limits: HashMap<u32, u8>,
    private: BTreeSet<usize>,
}

impl Parsed {
    /// The systems the declaration named at `start` is for.
    fn os(&self, start: u32) -> u8 {
        self.limits.get(&start).copied().unwrap_or(EVERY_OS)
    }
}

/// Ordered declaration identities and whether each hop has no independent public page.
type RefChain = Vec<(PathBuf, usize, bool, usize)>;

struct Reader<'a> {
    files: Files<'a>,
    parsed: HashMap<PathBuf, Option<Rc<Parsed>>>,
    /// The library's entry file, where names of files it loads are also in scope.
    module_file: PathBuf,
    /// Every file of the library: a library is its folder, whether or not the entry loads them.
    library: Vec<PathBuf>,
    /// What a name leads to anywhere in the library, found once.
    anywhere: HashMap<String, Option<(PathBuf, usize)>>,
    /// Where a name written in a file leads, found once.
    refs: HashMap<(PathBuf, String), RefChain>,
    /// Where each line starts in a file's text, keyed by the text's address: `files` keeps
    /// every text it read alive, so an address names one text for the reader's whole life.
    lines: HashMap<(usize, usize), Rc<Vec<usize>>>,
}

impl Reader<'_> {
    /// The line `at` is on, counting from 1.
    fn line_of(&mut self, text: &str, at: usize) -> usize {
        let starts = self
            .lines
            .entry((text.as_ptr() as usize, text.len()))
            .or_insert_with(|| {
                Rc::new(
                    std::iter::once(0)
                        .chain(text.match_indices('\n').map(|(i, _)| i + 1))
                        .collect(),
                )
            });
        starts.partition_point(|&start| start <= at)
    }
}

impl Reader<'_> {
    fn parsed(&mut self, file: &Path) -> Option<Rc<Parsed>> {
        if let Some(found) = self.parsed.get(file) {
            return found.clone();
        }
        let loaded = self.files.text(&file.to_path_buf()).map(|text| {
            let parse = parse_file_ide(FileId(0), &text);
            Rc::new(Parsed {
                private: ide_facts::top_level(&parse.file)
                    .iter()
                    .filter(|item| item.visibility != Visibility::Export)
                    .filter_map(|item| match item.kind {
                        TopLevelKind::Decl(d) => Some(d.names.iter()),
                        _ => None,
                    })
                    .flatten()
                    .map(|n| n.span.start as usize)
                    .collect(),
                limits: limits(&parse.file),
                text,
                parse,
            })
        });
        self.parsed.insert(file.to_path_buf(), loaded.clone());
        loaded
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
    // A header without a body is a procedure type, not a callable overload.
    if matches!(
        decl.value.as_ref().map(|v| &v.kind),
        Some(ExprKind::ProcType(_))
    ) {
        return "type";
    }
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
                refs: Vec::new(),
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
                refs: Vec::new(),
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
    let (mut params, mut returns, docs_text) = describe(header, text, at, &doc);
    for p in params.iter_mut().chain(returns.iter_mut()) {
        p.refs = reader.type_refs(file, &format!("{} {}", p.ty, p.default));
    }
    let docs_line = docs::located_above(text, at).map_or(0, |d| d.line);
    let limit = reader
        .parsed(file)
        .and_then(|parsed| parsed.limits.get(&name.span.start).copied())
        .unwrap_or(EVERY_OS);
    Overload {
        file: file.to_path_buf(),
        line: reader.line_of(text, at),
        signature: slice(text, header.span.start, header.span.end),
        params,
        returns,
        docs: docs_text,
        docs_line,
        flags: flags_of(header),
        platforms: os_names(limit),
    }
}

fn members(decl: &Decl, text: &str) -> Vec<Member> {
    let mut out = Vec::new();
    match decl.value.as_ref().map(|v| &v.kind) {
        Some(ExprKind::Struct(lit)) => {
            // Both branches of an `#if` are listed: a field is documented wherever it is declared.
            fn walk(stmts: &[Stmt], text: &str, out: &mut Vec<Member>) {
                for stmt in stmts {
                    match &stmt.kind {
                        StmtKind::StaticIf(branches) => {
                            walk(&branches.then_branch, text, out);
                            walk(&branches.else_branch, text, out);
                        }
                        StmtKind::Decl(field) if field.kind == DeclKind::Var => {
                            let ty = match (&field.ty, &field.value) {
                                (Some(t), _) => slice(text, t.span.start, t.span.end),
                                (None, Some(v)) => {
                                    format!(":= {}", slice(text, v.span.start, v.span.end))
                                }
                                (None, None) => String::new(),
                            };
                            for name in &field.names {
                                out.push(Member {
                                    refs: Vec::new(),
                                    name: name.name.as_str().to_owned(),
                                    ty: ty.clone(),
                                    doc: docs::declaration(text, name.span.start as usize)
                                        .unwrap_or_default(),
                                    at: name.span.start as usize,
                                    line: None,
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
            walk(&lit.body, text, &mut out);
        }
        Some(ExprKind::Enum(lit)) => {
            fn walk(items: &[EnumItem], text: &str, out: &mut Vec<Member>) {
                for item in items {
                    match item {
                        EnumItem::Member(m) => out.push(Member {
                            refs: Vec::new(),
                            name: m.name.name.as_str().to_owned(),
                            ty: m
                                .value
                                .as_ref()
                                .map_or(String::new(), |v| slice(text, v.span.start, v.span.end)),
                            doc: docs::declaration(text, m.name.span.start as usize)
                                .unwrap_or_default(),
                            at: m.name.span.start as usize,
                            line: None,
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

/// Whether a file exports `#foreign` procedures, which mirrors a native API documented by its
/// vendor, so its types and constants are listed as bindings too. `#intrinsic` and `#compiler`
/// procedures have no body either but are the language's own, so they don't count.
/// Private helpers (for example, after `#scope_file`) do not classify the file.
pub(super) fn is_binding_file(file: &crate::ast::File) -> bool {
    ide_facts::top_level(file).iter().any(|item| {
        item.visibility == Visibility::Export
            && matches!(item.kind, TopLevelKind::Decl(decl) if is_foreign_proc(decl))
    })
}

/// A `#foreign` (or `#elsewhere`) procedure constant: one a native library provides.
pub(super) fn is_foreign_proc(decl: &Decl) -> bool {
    decl.kind == DeclKind::Const
        && matches!(decl.value.as_ref().map(|v| &v.kind),
            Some(ExprKind::Proc(lit)) if lit.body.is_none() && lit.header.foreign.is_some())
}

/// The documentation model of the library in `dir`. `stdlib` is where the `internal/` modules
/// and other imports are found.
/// A file's top level with its constants found by name and by where a name starts, so each item
/// of a large binding file costs a lookup rather than a pass over the whole file.
struct TopIndex<'a> {
    tops: Vec<ide_facts::TopLevel<'a>>,
    /// The first declaration with a name starting at an offset.
    at: HashMap<usize, &'a Decl>,
    /// Each constant declaring a name, in source order, with its visibility.
    named: HashMap<&'a str, Vec<(&'a Decl, Visibility)>>,
}

impl<'a> TopIndex<'a> {
    fn new(parsed: &'a Parsed) -> Self {
        let tops = ide_facts::top_level(&parsed.parse.file);
        let mut at: HashMap<usize, &'a Decl> = HashMap::default();
        let mut named: HashMap<&'a str, Vec<(&'a Decl, Visibility)>> = HashMap::default();
        for t in &tops {
            let TopLevelKind::Decl(d) = t.kind else {
                continue;
            };
            for n in &d.names {
                at.entry(n.span.start as usize).or_insert(d);
            }
            if d.kind == DeclKind::Const {
                let mut seen: Vec<&str> = Vec::new();
                for n in &d.names {
                    let name = n.name.as_str();
                    if !seen.contains(&name) {
                        seen.push(name);
                        named.entry(name).or_default().push((d, t.visibility));
                    }
                }
            }
        }
        Self {
            tops,
            at,
            named,
        }
    }

    fn constants(&self, name: &str) -> &[(&'a Decl, Visibility)] {
        self.named.get(name).map_or(&[], Vec::as_slice)
    }
}

/// The parameter names of a parameterized struct, which its members may refer to.
fn struct_params(decl: &Decl) -> Vec<String> {
    let Some(Expr {
        kind: ExprKind::Struct(lit),
        ..
    }) = &decl.value
    else {
        return Vec::new();
    };
    lit.params
        .iter()
        .filter_map(|p| p.name.as_ref())
        .map(|n| n.name.as_str().to_owned())
        .collect()
}

pub fn module(dir: &Path, stdlib: &Path) -> Module {
    let mut own = Vec::new();
    super::own_files(dir, true, &mut own);
    let module_file = super::canonical(&crate::paths::module_entry(dir));
    let mut reader = Reader {
        files: Files::new(stdlib),
        parsed: HashMap::default(),
        module_file: module_file.clone(),
        library: own.iter().map(|f| super::canonical(f)).collect(),
        anywhere: HashMap::default(),
        refs: HashMap::default(),
        lines: HashMap::default(),
    };
    // The module's own file first, so items come in the order it lists them.
    own.sort_by_key(|f| super::canonical(f) != module_file);
    let mut queue: Vec<PathBuf> = own;
    queue.reverse();
    let mut seen = BTreeSet::new();
    let mut out = Module {
        file: module_file.clone(),
        implicit: super::canonical(dir) == super::canonical(&stdlib.join("prelude")),
        ..Module::default()
    };
    let mut names: Vec<String> = Vec::new();
    let mut binding_files: HashMap<PathBuf, bool> = HashMap::default();
    let internal = super::canonical(&stdlib.join("internal"));
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
        let mut first_statement = None;
        let mut loads = Vec::new();
        let index = TopIndex::new(&parsed);
        // What an `#insert` declares is known only by running it, which the model never does.
        for (span, visibility) in ide_facts::top_level_inserts(&parsed.parse.file) {
            if visibility == Visibility::Export {
                let line = reader.line_of(&text, span.start as usize);
                out.inserts.push((file.clone(), line));
            }
        }
        for item in &index.tops {
            first_statement.get_or_insert(item.span.start as usize);
            match item.kind {
                TopLevelKind::Load(path, _) => loads.push(base.join(path)),
                TopLevelKind::Assert(assert) if file == module_file => {
                    // `OS_IS_UNIX || OS == .WINDOWS` says more than the targets it names.
                    let only_os = OSES
                        .iter()
                        .all(|(name, _)| holds_on(&assert.cond, name).is_some());
                    if let Some(targets) = ide_facts::asserted_targets(assert).filter(|_| only_os) {
                        for t in targets {
                            let t = t.as_str().to_owned();
                            if !out.platforms.contains(&t) {
                                out.platforms.push(t);
                            }
                        }
                    }
                }
                TopLevelKind::ModuleParameters(stmt) => {
                    for p in stmt.params.iter().chain(&stmt.runtime_params) {
                        let Some(name) = &p.name else {
                            continue;
                        };
                        let written = |e: &crate::ast::Expr| slice(&text, e.span.start, e.span.end);
                        let ty = p.ty.as_ref().map(written).unwrap_or_default();
                        let default = p.default.as_ref().map(written).unwrap_or_default();
                        let refs = reader.type_refs(&file, &format!("{ty} {default}"));
                        out.parameters.push(Parameter {
                            name: name.name.as_str().to_owned(),
                            ty,
                            default,
                            doc: docs::declaration(&text, name.span.start as usize)
                                .unwrap_or_default(),
                            refs,
                        });
                    }
                }
                TopLevelKind::Decl(decl) if decl.kind == DeclKind::Const => {
                    if item.visibility != Visibility::Export {
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
                    let binding = *binding_files
                        .entry(tfile.clone())
                        // The standard library's own `internal/` code may call a native
                        // function, but mirrors no vendor's API.
                        .or_insert_with(|| {
                            !tfile.starts_with(&internal) && is_binding_file(&target.parse.file)
                        });
                    let elsewhere;
                    let tindex = if Rc::ptr_eq(&target, &parsed) {
                        &index
                    } else {
                        elsewhere = TopIndex::new(&target);
                        &elsewhere
                    };
                    let tname = tindex.at.get(&tat).copied();
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
                                line: reader.line_of(&text, at),
                                docs: own_docs.unwrap_or_default(),
                                value: clip(value),
                                binding,
                                platforms: os_names(parsed.os(name.span.start)),
                                defined_on: os_names(parsed.os(name.span.start)),
                                ..Item::default()
                            },
                        );
                        continue;
                    };
                    let mut kind = class_kind(tdecl);
                    let target_is_type =
                        matches!(kind, "struct" | "union" | "enum" | "enum_flags" | "type")
                            || (kind == "constant"
                                && tdecl.value.as_ref().is_some_and(|value| {
                                    reader.value_is_type(&tfile, value, MAX_HOPS)
                                }));
                    let alias_type = (tfile != file || tat != at)
                        && target_is_type
                        && name.name != tdecl.names[0].name;
                    if alias_type
                        || (kind == "constant"
                            && decl
                                .value
                                .as_ref()
                                .is_some_and(|value| reader.value_is_type(&file, value, MAX_HOPS)))
                    {
                        kind = "type";
                    }
                    let alias_limit = parsed.os(name.span.start);
                    // A name declared in each branch of an `#if` is available wherever one is.
                    let target_limit = tindex
                        .constants(tdecl.names[0].name.as_str())
                        .iter()
                        .map(|(d, _)| target.os(d.names[0].span.start))
                        .fold(0, |acc, mask| acc | mask);
                    let mut found = Item {
                        name: name.name.as_str().to_owned(),
                        kind,
                        file: tfile.clone(),
                        line: reader.line_of(&target.text, tat),
                        docs: own_docs.unwrap_or_default(),
                        // A foreign procedure is a binding wherever it is; the rest of a
                        // declaration in a file that mirrors a native API is too.
                        binding: is_foreign_proc(tdecl)
                            || (binding && ide_facts::decl_class(tdecl) != DeclClass::Procedure),
                        ..Item::default()
                    };
                    if !alias_type && (tfile != file || tat != at) {
                        // A thin re-export represents both declaration identities. This also
                        // lets references written in the provider's files find its public page.
                        found.also.push((file.clone(), reader.line_of(&text, at)));
                    }
                    found.scope = struct_params(tdecl);
                    if alias_type {
                        found.file.clone_from(&file);
                        found.line = reader.line_of(&text, at);
                        found.value = clip(
                            decl.value
                                .as_ref()
                                .map_or(String::new(), |v| slice(&text, v.span.start, v.span.end)),
                        );
                        found.refs = reader.type_refs(&file, &found.value);
                        if found.docs.is_empty() {
                            found.docs = docs::declaration(&target.text, tat).unwrap_or_default();
                        }
                    } else if kind == "procedure" {
                        // The comment above a procedure (not an alias of one) is its first
                        // overload's, which carries it with the parameter lines taken out.
                        if tfile == file && tat == at {
                            found.docs.clear();
                        }
                        // Every declaration of the name in the target's file is an overload.
                        for &(d, visibility) in tindex.constants(tdecl.names[0].name.as_str()) {
                            if visibility == Visibility::File {
                                continue;
                            }
                            if let Some(header) = ide_facts::decl_header(d) {
                                let o = overload(&mut reader, &tfile, &target.text, d, header);
                                found.overloads.push(o);
                            }
                        }
                    } else {
                        reader.item_members(&tfile, &target.text, tdecl, &mut found);
                        if matches!(kind, "struct" | "union" | "enum" | "enum_flags") {
                            reader.item_declaration(&tfile, &target.text, tdecl, &mut found);
                        }
                        if found.docs.is_empty() {
                            found.docs = docs::declaration(&target.text, tat).unwrap_or_default();
                        }
                        if kind == "constant" || kind == "type" {
                            found.value = clip(tdecl.value.as_ref().map_or(String::new(), |v| {
                                slice(&target.text, v.span.start, v.span.end)
                            }));
                            found.refs = reader.type_refs(&tfile, &found.value);
                        }
                    }
                    let own_limit = if alias_type {
                        alias_limit
                    } else {
                        target.os(tdecl.names[0].span.start) & alias_limit
                    };
                    found.defined_on = os_names(own_limit);
                    found.platforms = os_names(if kind == "procedure" {
                        found
                            .overloads
                            .iter()
                            .fold(0, |acc, o| acc | os_mask(&o.platforms))
                            & alias_limit
                    } else {
                        target_limit & alias_limit
                    });
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
    reader.add_private_types(&mut out, &mut names);
    for item in &mut out.items {
        overloads::merge_alternatives(&mut item.overloads);
    }
    take_experimental(&mut out);
    out
}

/// Moves the `@experimental` tag lines out of the docs and into the flags.
fn take_experimental(module: &mut Module) {
    (module.docs, module.experimental) = docs::strip_experimental(&module.docs);
    for item in &mut module.items {
        let (text, mut tagged) = docs::strip_experimental(&item.docs);
        item.docs = text;
        for o in &mut item.overloads {
            let (text, t) = docs::strip_experimental(&o.docs);
            o.docs = text;
            tagged |= t;
        }
        item.experimental = tagged;
    }
}

/// Whether two declarations of a name say the same, whatever branch and file they are in.
fn same_definition(a: &Item, b: &Item) -> bool {
    let members = |i: &Item| -> Vec<(String, String, String)> {
        i.members
            .iter()
            .map(|m| (m.name.clone(), m.ty.clone(), m.doc.clone()))
            .collect()
    };
    a.kind == b.kind
        && a.docs == b.docs
        && a.value == b.value
        && a.declaration == b.declaration
        && members(a) == members(b)
}

fn add(out: &mut Module, names: &mut Vec<String>, mut item: Item) {
    if let Some(i) = names.iter().position(|n| *n == item.name) {
        let have = &mut out.items[i];
        have.platforms = os_names(os_mask(&have.platforms) | os_mask(&item.platforms));
        if have.kind != "procedure" && (&have.file, have.line) != (&item.file, item.line) {
            have.also.push((item.file.clone(), item.line));
            if same_definition(have, &item) {
                have.defined_on = os_names(os_mask(&have.defined_on) | os_mask(&item.defined_on));
            } else if let Some(same) = have.variants.iter_mut().find(|v| same_definition(v, &item))
            {
                same.defined_on = os_names(os_mask(&same.defined_on) | os_mask(&item.defined_on));
            } else {
                item.platforms.clone_from(&item.defined_on);
                have.variants.push(item);
                return;
            }
        }
        have.also.extend(item.also);
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
    let own: BTreeSet<&str> = model
        .items
        .iter()
        .filter(|i| !i.private)
        .map(|i| i.name.as_str())
        .collect();
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
#[path = "model_tests.rs"]
mod tests;
