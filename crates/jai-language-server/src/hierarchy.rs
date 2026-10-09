//! Call hierarchy: who calls a procedure, and what it calls.
//!
//! Callers come from the references of the procedure's name (the calls among them), and callees
//! from the calls the type checker recorded in the procedure's body, each resolved to the
//! overload it chose. Procedures are found by parsing the file they are declared in, so a callee
//! in a module or the standard library works too. The call sites of an item have to be in a
//! document the program was compiled from, which means an open one.
use crate::analysis::Span;
use crate::position::LineIndex;
use crate::session::Session;
use crate::{CallHierarchyCall, CallHierarchyItem, DocumentUri, Error, Position, SymbolKind};
use jaic::ast::{DeclKind, ExprKind as E, Stmt, StmtKind as S};
use jaic::source::FileId;
use jailint::syntax::walk;
use std::collections::{HashMap, HashSet};

/// A procedure declared as `name :: (...) { ... }` (or without a body, as a foreign one).
#[derive(Clone, Debug)]
struct Site {
    name: String,
    name_span: (usize, usize),
    /// The whole declaration.
    extent: (usize, usize),
    /// The braces of the body.
    body: Option<(usize, usize)>,
    /// `(a: int) -> int`
    header: String,
    /// Where the header starts.
    header_start: usize,
}

/// Every procedure declaration in `text`, at any depth.
fn sites(text: &str) -> Vec<Site> {
    let Ok(file) = jaic::parser::parse_file(FileId(0), text) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    collect(text, &file.stmts, &mut out, &mut HashSet::new());
    out
}

fn collect(text: &str, stmts: &[Stmt], out: &mut Vec<Site>, seen: &mut HashSet<u32>) {
    walk(stmts, &mut |node, _| {
        if let Some(s) = node.stmt()
            && let S::Decl(d) = &s.kind
            && d.kind == DeclKind::Const
            && d.names.len() == 1
            && let Some(value) = &d.value
            && let E::Proc(lit) = &value.kind
        {
            seen.insert(lit.header.id.0);
            let (a, b) = crate::refactor::stmt_extent(text, s);
            let name = d.names[0];
            let body = lit
                .body
                .as_ref()
                .map(|b| (b.span.start as usize, b.span.end as usize));
            let from = value.span.start as usize;
            let to = body.map_or(b, |(x, _)| x);
            out.push(Site {
                name: name.name.as_str().to_string(),
                name_span: (name.span.start as usize, name.span.end as usize),
                extent: (a, b),
                body,
                header: text.get(from..to).unwrap_or("").trim().to_string(),
                header_start: from,
            });
            if let Some(body) = &lit.body {
                collect(text, &body.stmts, out, seen);
            }
            return true;
        }
        let Some(x) = node.expr() else {
            return true;
        };
        match &x.kind {
            E::Proc(lit) if !seen.contains(&lit.header.id.0) => {
                if let Some(body) = &lit.body {
                    collect(text, &body.stmts, out, seen);
                }
                false
            }
            // Declared procedures are collected from their statement.
            E::Proc(_) => false,
            E::Struct(st) => {
                collect(text, &st.body, out, seen);
                false
            }
            _ => true,
        }
    });
}

fn contains((a, b): (usize, usize), at: usize) -> bool {
    a <= at && at <= b
}

/// The procedure declared at `at`: its name or the start of its header.
fn declared_at(sites: &[Site], at: usize) -> Option<&Site> {
    sites
        .iter()
        .filter(|s| contains(s.name_span, at) || s.header_start == at)
        .min_by_key(|s| s.extent.1 - s.extent.0)
}

/// The procedure whose body holds `at`.
fn calling(sites: &[Site], at: usize) -> Option<&Site> {
    sites
        .iter()
        .filter(|s| s.body.is_some_and(|b| contains(b, at)))
        .min_by_key(|s| s.extent.1 - s.extent.0)
}

fn site_item(uri: &str, text: &str, site: &Site) -> Option<CallHierarchyItem> {
    let index = LineIndex::new(text);
    let range = |(a, b): (usize, usize)| index.range(text, Span::new(a, b)).ok();
    Some(CallHierarchyItem {
        name: site.name.clone(),
        detail: site.header.lines().next().unwrap_or("").trim().to_string(),
        kind: SymbolKind::Function,
        uri: uri.into(),
        range: range(site.extent)?,
        selection_range: range(site.name_span)?,
    })
}

/// A callee's name, with the `a.b.` before it: what a call is written as.
fn callee_len(text: &str, at: usize) -> usize {
    text[at..]
        .find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '.'))
        .unwrap_or(text.len() - at)
}

/// Collects the calls of one caller (or callee) item.
#[derive(Default)]
struct Grouped {
    order: Vec<(String, usize)>,
    entries: HashMap<(String, usize), (CallHierarchyItem, Vec<crate::Range>)>,
}

impl Grouped {
    fn add(&mut self, item: CallHierarchyItem, key: usize, from: crate::Range) {
        let key = (item.uri.clone(), key);
        if !self.entries.contains_key(&key) {
            self.order.push(key.clone());
        }
        self.entries
            .entry(key)
            .or_insert_with(|| (item, Vec::new()))
            .1
            .push(from);
    }

    fn finish(mut self) -> Vec<CallHierarchyCall> {
        self.order
            .into_iter()
            .filter_map(|key| self.entries.remove(&key))
            .map(|(item, mut from_ranges)| {
                from_ranges.sort_by_key(|r| (r.start, r.end));
                from_ranges.dedup();
                CallHierarchyCall {
                    item,
                    from_ranges,
                }
            })
            .collect()
    }
}

impl Session {
    /// The procedures named at `position`: the one declared there, or the ones a use names.
    pub fn prepare_call_hierarchy(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Vec<CallHierarchyItem>, Error> {
        let doc = self.document(uri)?;
        let byte = doc.index.byte(&doc.text, position)?;
        let here = sites(&doc.text);
        if let Some(site) = here.iter().find(|s| contains(s.name_span, byte)) {
            return Ok(site_item(uri.as_str(), &doc.text, site)
                .into_iter()
                .collect());
        }
        if self.word(uri, position)?.is_none() {
            return Ok(Vec::new());
        }
        let mut items: Vec<CallHierarchyItem> = Vec::new();
        for location in self.definition(uri, position)? {
            let Ok(target) = DocumentUri::parse(&location.uri) else {
                continue;
            };
            let Some(text) = self.source(&target) else {
                continue;
            };
            let Ok(at) = LineIndex::new(&text).byte(&text, location.range.start) else {
                continue;
            };
            if let Some(found) =
                declared_at(&sites(&text), at).and_then(|s| site_item(&location.uri, &text, s))
                && !items
                    .iter()
                    .any(|i| i.uri == found.uri && i.selection_range == found.selection_range)
            {
                items.push(found);
            }
        }
        Ok(items)
    }

    /// The procedures that call `item`, with where each calls it.
    pub fn incoming_calls(
        &self,
        item: &CallHierarchyItem,
    ) -> Result<Vec<CallHierarchyCall>, Error> {
        let uri = DocumentUri::parse(&item.uri)?;
        if !self.documents.contains_key(&uri) {
            return Ok(Vec::new());
        }
        let mut grouped = Grouped::default();
        let mut parsed: HashMap<String, (String, Vec<Site>)> = HashMap::new();
        for reference in self.references(&uri, item.selection_range.start, false)? {
            let Ok(target) = DocumentUri::parse(&reference.uri) else {
                continue;
            };
            let Some(text) = self.source(&target) else {
                continue;
            };
            let index = LineIndex::new(&text);
            let (Ok(a), Ok(b)) = (
                index.byte(&text, reference.range.start),
                index.byte(&text, reference.range.end),
            ) else {
                continue;
            };
            // Calls only: a name followed by its arguments.
            if !text[b..].trim_start_matches([' ', '\t']).starts_with('(') {
                continue;
            }
            let entry = parsed
                .entry(reference.uri.clone())
                .or_insert_with(|| (text.clone(), sites(&text)));
            let Some(site) = calling(&entry.1, a) else {
                continue;
            };
            if let Some(caller) = site_item(&reference.uri, &entry.0, site) {
                grouped.add(caller, site.name_span.0, reference.range);
            }
        }
        Ok(grouped.finish())
    }

    /// The procedures `item` calls, with where.
    pub fn outgoing_calls(
        &self,
        item: &CallHierarchyItem,
    ) -> Result<Vec<CallHierarchyCall>, Error> {
        let uri = DocumentUri::parse(&item.uri)?;
        let Ok(doc) = self.document(&uri) else {
            return Ok(Vec::new());
        };
        let here = sites(&doc.text);
        let start = doc.index.byte(&doc.text, item.selection_range.start)?;
        let Some(me) = here.iter().find(|s| contains(s.name_span, start)) else {
            return Ok(Vec::new());
        };
        let Some(body) = me.body else {
            return Ok(Vec::new());
        };
        let calls = self
            .checked(&uri, |a, f| {
                a.compiler
                    .ide_calls(f)
                    .into_iter()
                    .filter(|c| (body.0..=body.1).contains(&(c.span.start as usize)))
                    .filter_map(|c| {
                        let target = a.location(c.signatures.get(c.active)?.span)?;
                        Some((c.span.start as usize, target))
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let mut grouped = Grouped::default();
        let mut parsed: HashMap<String, Vec<Site>> = HashMap::new();
        for (at, (path, text, declared, _)) in calls {
            // Not the calls of a procedure nested in this one.
            if calling(&here, at).is_none_or(|s| s.name_span != me.name_span) {
                continue;
            }
            let Ok(target) = DocumentUri::parse(&format!("file://{path}")) else {
                continue;
            };
            let known = parsed.entry(path.clone()).or_insert_with(|| sites(&text));
            let Some(site) = declared_at(known, declared) else {
                continue;
            };
            let Some(callee) = site_item(target.as_str(), &text, site) else {
                continue;
            };
            let len = callee_len(&doc.text, at);
            let Ok(from) = doc.index.range(&doc.text, Span::new(at, at + len)) else {
                continue;
            };
            grouped.add(callee, site.name_span.0, from);
        }
        Ok(grouped.finish())
    }
}
