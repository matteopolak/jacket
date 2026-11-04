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
use jaic::ast::walk::walk;
use jaic::ast::{DeclKind, ExprKind as E, Stmt, StmtKind as S};
use jaic::fxhash::{HashMap, HashSet};
use jaic::source::FileId;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

type SourceKey = (u64, usize);
type TargetSources = HashMap<String, Option<(SourceKey, Arc<str>)>>;

struct CallTarget {
    path: String,
    source_key: SourceKey,
    site: Site,
}

struct CallFact {
    start: usize,
    end: usize,
    target: Rc<CallTarget>,
}

/// Resolved calls of one checked open document, independent of its compiler's lifetime.
pub(crate) struct CallGraph {
    generation: u64,
    text: Arc<str>,
    calls: Vec<CallFact>,
}

impl CallGraph {
    fn moved_call(
        &self,
        text: &str,
        shift: crate::semantic::Shift,
        call: &CallFact,
    ) -> Option<usize> {
        let start = shift.to_new(call.start)?;
        let end = shift.to_new(call.end)?;
        (self.text.get(call.start..call.end)? == text.get(start..end)?).then_some(start)
    }
}

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
    let file = jaic::parser::parse_file_ide(FileId(0), text).file;
    let mut out = Vec::new();
    collect(text, &file.stmts, &mut out, &mut HashSet::default());
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
            let name_span = jaic::parser::ide_facts::decl_name_span(d).unwrap_or(name.span);
            let body = lit
                .body
                .as_ref()
                .map(|b| (b.span.start as usize, b.span.end as usize));
            let from = value.span.start as usize;
            let to = body.map_or(b, |(x, _)| x);
            out.push(Site {
                name: name.name.as_str().to_string(),
                name_span: (name_span.start as usize, name_span.end as usize),
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
        .find(|c: char| !(jaic::lexicon::is_identifier_char(c) || c == '.'))
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
        let byte = doc.index.byte(&doc.text, position);
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
            let at = LineIndex::new(&text).byte(&text, location.range.start);
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

    /// Keep resolved calls when diagnostics check a program. Callee source is reduced to
    /// declarations and a content key, so these records retain no compiler or library text.
    pub(crate) fn remember_call_graphs(
        &self,
        checked: &DocumentUri,
        analysis: &mut crate::semantic::Analysis,
    ) {
        let root = self.root(checked);
        let generation = self.semantic_revision();
        let mut parsed: HashMap<String, (SourceKey, Vec<Site>)> = HashMap::default();
        let mut targets: HashMap<(String, usize), Rc<CallTarget>> = HashMap::default();
        for uri in self.documents.keys().filter(|uri| self.root(uri) == root) {
            let path = Path::new(uri.path());
            analysis.focus(path);
            let Some(file) = analysis.file(path) else {
                continue;
            };
            let Some(source) = analysis.compiler.src.sources.try_get(file) else {
                continue;
            };
            let text = source.text.clone();
            if self
                .call_graphs
                .borrow()
                .get(uri)
                .is_some_and(|old| old.generation == generation && Arc::ptr_eq(&old.text, &text))
            {
                continue;
            }
            let calls = analysis
                .compiler
                .ide_calls(file)
                .into_iter()
                .filter_map(|call| {
                    let (path, text, declared, _) =
                        analysis.location(call.signatures.get(call.active)?.span)?;
                    let key = (path.clone(), declared);
                    let target = if let Some(target) = targets.get(&key) {
                        target.clone()
                    } else {
                        let (source_key, known) = parsed
                            .entry(path.clone())
                            .or_insert_with(|| (crate::semantic::text_key(&text), sites(&text)));
                        let site = declared_at(known, declared)?.clone();
                        let target = Rc::new(CallTarget {
                            path,
                            source_key: *source_key,
                            site,
                        });
                        targets.insert(key, target.clone());
                        target
                    };
                    Some(CallFact {
                        start: call.span.start as usize,
                        end: call.span.end as usize,
                        target,
                    })
                })
                .collect();
            self.call_graphs.borrow_mut().insert(
                uri.clone(),
                Rc::new(CallGraph {
                    generation,
                    text,
                    calls,
                }),
            );
        }
    }

    fn call_graph(&self, uri: &DocumentUri) -> Option<Rc<CallGraph>> {
        let (uri, doc) = self.document_entry(uri).ok()?;
        let old = self.call_graphs.borrow().get(uri).cloned();
        if old.as_ref().is_some_and(|graph| {
            graph.generation == self.semantic_revision() && graph.text.as_ref() == doc.text
        }) {
            return old;
        }
        self.with_semantic(uri, &doc.text, |analysis, path| {
            self.remember_call_graphs(uri, analysis);
            analysis.focus(path);
            Some(())
        });
        self.call_graphs.borrow().get(uri).cloned().or(old)
    }

    /// Resolve a stored declaration against the current file. An external declaration whose
    /// file changed waits for a new check; a same-file declaration moves with its caller.
    fn call_target(
        &self,
        uri: &DocumentUri,
        here: &[Site],
        shift: crate::semantic::Shift,
        fact: &CallFact,
        sources: &mut TargetSources,
    ) -> Option<(CallHierarchyItem, usize)> {
        let target = self.client_uri(Path::new(&fact.target.path))?;
        if target == *uri {
            let doc = self.document(uri).ok()?;
            let declared = shift.to_new(fact.target.site.name_span.0)?;
            let site = declared_at(here, declared)?;
            if site.header != fact.target.site.header {
                return None;
            }
            return Some((
                site_item(target.as_str(), &doc.text, site)?,
                site.name_span.0,
            ));
        }
        let source = sources
            .entry(fact.target.path.clone())
            .or_insert_with(|| {
                let text: Arc<str> = self.source(&target)?.into();
                Some((crate::semantic::text_key(&text), text))
            })
            .as_ref()?;
        if source.0 != fact.target.source_key {
            return None;
        }
        let site = &fact.target.site;
        Some((
            site_item(target.as_str(), &source.1, site)?,
            site.name_span.0,
        ))
    }

    /// The procedures that call `item`, with where each calls it.
    pub fn incoming_calls(
        &self,
        item: &CallHierarchyItem,
    ) -> Result<Vec<CallHierarchyCall>, Error> {
        let requested = DocumentUri::parse(&item.uri)?;
        let Ok((uri, _)) = self.document_entry(&requested) else {
            return Ok(Vec::new());
        };
        self.call_graph(uri);
        let graphs: Vec<_> = self
            .call_graphs
            .borrow()
            .iter()
            .map(|(uri, graph)| (uri.clone(), graph.clone()))
            .collect();
        let mut grouped = Grouped::default();
        let mut sources = HashMap::default();
        for (caller_uri, graph) in graphs {
            let Ok(doc) = self.document(&caller_uri) else {
                continue;
            };
            let here = sites(&doc.text);
            let shift = crate::semantic::Shift::between(graph.text.as_bytes(), doc.text.as_bytes());
            for fact in &graph.calls {
                let Some(at) = graph.moved_call(&doc.text, shift, fact) else {
                    continue;
                };
                let Some((callee, _)) =
                    self.call_target(&caller_uri, &here, shift, fact, &mut sources)
                else {
                    continue;
                };
                if callee.uri != uri.as_str() || callee.selection_range != item.selection_range {
                    continue;
                }
                let Some(site) = calling(&here, at) else {
                    continue;
                };
                let Some(caller) = site_item(caller_uri.as_str(), &doc.text, site) else {
                    continue;
                };
                let len = callee_len(&doc.text, at);
                let Ok(from) = doc.index.range(&doc.text, Span::new(at, at + len)) else {
                    continue;
                };
                grouped.add(caller, site.name_span.0, from);
            }
        }
        Ok(grouped.finish())
    }

    /// The procedures `item` calls, with where.
    pub fn outgoing_calls(
        &self,
        item: &CallHierarchyItem,
    ) -> Result<Vec<CallHierarchyCall>, Error> {
        let requested = DocumentUri::parse(&item.uri)?;
        let Ok((uri, doc)) = self.document_entry(&requested) else {
            return Ok(Vec::new());
        };
        let here = sites(&doc.text);
        let start = doc.index.byte(&doc.text, item.selection_range.start);
        let Some(me) = here.iter().find(|site| contains(site.name_span, start)) else {
            return Ok(Vec::new());
        };
        let Some(graph) = self.call_graph(uri) else {
            return Ok(Vec::new());
        };
        let shift = crate::semantic::Shift::between(graph.text.as_bytes(), doc.text.as_bytes());
        let mut grouped = Grouped::default();
        let mut sources = HashMap::default();
        for fact in &graph.calls {
            let Some(at) = graph.moved_call(&doc.text, shift, fact) else {
                continue;
            };
            if calling(&here, at).is_none_or(|site| site.name_span != me.name_span) {
                continue;
            }
            let Some((callee, key)) = self.call_target(uri, &here, shift, fact, &mut sources)
            else {
                continue;
            };
            let len = callee_len(&doc.text, at);
            let Ok(from) = doc.index.range(&doc.text, Span::new(at, at + len)) else {
                continue;
            };
            grouped.add(callee, key, from);
        }
        Ok(grouped.finish())
    }
}
