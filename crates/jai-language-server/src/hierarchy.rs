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

type SourceKey = Arc<str>;
type TargetSources = HashMap<String, Option<(SourceKey, Arc<str>)>>;

struct CallTarget {
    path: String,
    declared: usize,
    source_key: SourceKey,
    site: Site,
}

/// Selected session inputs, without compiler IDs, for staging a graph answer.
struct GraphContext {
    files: crate::semantic::Buffers,
    authority: Rc<()>,
    ignored: Option<std::path::PathBuf>,
    reads: crate::semantic::Reads,
}

struct CallFact {
    start: usize,
    end: usize,
    target: Rc<CallTarget>,
}

/// Resolved calls of one checked open document, independent of its compiler's lifetime.
pub(crate) struct CallGraph {
    generation: u64,
    input: Rc<crate::semantic::CheckedInputs>,
    facts_accepted: Rc<std::cell::Cell<bool>>,
    text: Arc<str>,
    calls: Vec<CallFact>,
}

impl CallGraph {
    pub(crate) fn accepted(&self) -> bool {
        self.facts_accepted.get()
    }

    fn belongs_to(&self, owner: &Rc<std::cell::Cell<bool>>) -> bool {
        Rc::ptr_eq(&self.facts_accepted, owner)
    }

    fn bytes(&self) -> usize {
        self.input.bytes
            + self.text.len()
            + std::mem::size_of::<Self>()
            + self.calls.capacity() * std::mem::size_of::<CallFact>()
            + self
                .calls
                .iter()
                .map(|call| {
                    let target = &call.target;
                    std::mem::size_of::<CallTarget>()
                        + target.path.capacity()
                        + target.source_key.len()
                        + target.site.name.capacity()
                        + target.site.header.capacity()
                        + 64
                })
                .sum::<usize>()
    }

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
        let _query = self.discovery_query();
        let mapping = std::cell::RefCell::new(crate::semantic::Observations::default());
        let base = self.config_filesystem();
        let fs = base.as_ref().map(|base| crate::semantic::ObservedFs {
            base,
            proof: Some(&mapping),
            secondary: None,
        });
        let (uri, doc) = match &fs {
            Some(fs) => self.projected_document_entry(fs, uri)?,
            None => self.document_entry(uri)?,
        };
        let byte = doc.index.byte(&doc.text, position);
        let here = sites(&doc.text);
        if let Some(site) = here.iter().find(|s| contains(s.name_span, byte)) {
            let items = site_item(uri.as_str(), &doc.text, site)
                .into_iter()
                .collect();
            return Ok(
                if self
                    .environment
                    .as_ref()
                    .is_none_or(|env| mapping.borrow().matches(&*env.fs))
                {
                    items
                } else {
                    Vec::new()
                },
            );
        }
        if self.word(uri, position)?.is_none() {
            return Ok(Vec::new());
        }
        if let Some((items, _)) = self.with_best_effort_semantic_at_projected(
            uri,
            &doc.text,
            byte,
            |a, path, at, shift| {
                let definitions = a.definition(path, at);
                if definitions.is_empty() {
                    return None;
                }
                let fs = fs.as_ref()?;
                let mut items: Vec<CallHierarchyItem> = Vec::new();
                for (path, text, start, _) in definitions {
                    let Some(target) = self.projected_uri(fs, Path::new(&path)) else {
                        continue;
                    };
                    let (text, at) = if target == *uri && !shift.is_none() {
                        let Some(start) = shift.to_new(start) else {
                            continue;
                        };
                        (Arc::from(doc.text.as_str()), start)
                    } else if let Some(open) = self
                        .documents
                        .get(&target)
                        .or_else(|| self.lint_configs.get(&target))
                    {
                        if !open.same_positions(&text) {
                            continue;
                        }
                        (Arc::from(open.text.as_str()), start)
                    } else {
                        (text, start)
                    };
                    if let Some(found) = declared_at(&sites(&text), at)
                        .and_then(|site| site_item(target.as_str(), &text, site))
                        && !items.iter().any(|item| {
                            item.uri == found.uri && item.selection_range == found.selection_range
                        })
                    {
                        items.push(found);
                    }
                }
                // A resolved nonprocedure is accepted emptiness, not an invitation to another target.
                Some(items)
            },
        ) {
            return Ok(items);
        }
        // Preserve lexical help from current client buffers when semantic facts are unavailable.
        // This traversal observes alias mapping and never reads external target text.
        let bindings = match &fs {
            Some(fs) => self.projected_bindings(fs, uri, position)?,
            None => self.bindings(uri, position)?,
        };
        let mut items: Vec<CallHierarchyItem> = Vec::new();
        for (target, row) in bindings {
            let Some(open) = self.documents.get(target) else {
                continue;
            };
            if let Some(found) = declared_at(&sites(&open.text), row.selection.start)
                .and_then(|site| site_item(target.as_str(), &open.text, site))
                && !items.iter().any(|item| {
                    item.uri == found.uri && item.selection_range == found.selection_range
                })
            {
                items.push(found);
            }
        }
        if self
            .environment
            .as_ref()
            .is_some_and(|env| !mapping.borrow().matches(&*env.fs))
        {
            return Ok(Vec::new());
        }
        Ok(items)
    }

    /// Keep resolved calls when diagnostics check a program. Callee source is reduced to
    /// declarations and exact source text, charged to the record budget, without a compiler.
    pub(crate) fn call_graph_owners(&self, owner: &Rc<std::cell::Cell<bool>>) -> Vec<DocumentUri> {
        let mut roots = std::collections::BTreeSet::new();
        self.call_graphs
            .borrow()
            .iter()
            .filter(|(uri, graph)| {
                graph.belongs_to(owner) && roots.insert(self.projected_root(uri))
            })
            .map(|(uri, _)| uri.clone())
            .collect()
    }

    pub(crate) fn refresh_call_graphs(
        &self,
        owners: &[DocumentUri],
        analysis: &mut crate::semantic::Analysis,
    ) {
        for uri in owners {
            let already_refreshed = self
                .call_graphs
                .borrow()
                .get(uri)
                .is_some_and(|graph| graph.belongs_to(&analysis.facts_accepted));
            if !already_refreshed {
                let _ = self.remember_call_graphs(uri, analysis);
            }
        }
    }

    pub(crate) fn remember_call_graphs(
        &self,
        checked: &DocumentUri,
        analysis: &mut crate::semantic::Analysis,
    ) -> Option<Rc<CallGraph>> {
        let root = self.projected_root(checked);
        let generation = self.semantic_revision();
        let mut parsed: HashMap<String, (SourceKey, Vec<Site>)> = HashMap::default();
        // Reuse only immutable syntax descriptions; call edges are rebuilt from current facts.
        let mut targets: HashMap<(String, usize), Rc<CallTarget>> = self
            .call_graphs
            .borrow()
            .values()
            .flat_map(|graph| {
                graph.calls.iter().map(|call| {
                    let target = call.target.clone();
                    ((target.path.clone(), target.declared), target)
                })
            })
            .collect();
        let mut graphs = Vec::new();
        for uri in self
            .documents
            .keys()
            .filter(|uri| self.projected_root(uri) == root)
        {
            let path = Path::new(uri.path());
            analysis.focus(path);
            let Some(file) = analysis.file(path) else {
                continue;
            };
            let Some(source) = analysis.compiler.src.sources.try_get(file) else {
                continue;
            };
            let text = source.text.clone();
            let calls = analysis
                .compiler
                .ide_calls(file)
                .into_iter()
                .filter_map(|call| {
                    let (path, text, declared, _) =
                        analysis.location(call.signatures.get(call.active)?.span)?;
                    let key = (path.clone(), declared);
                    let target = if let Some(target) =
                        targets.get(&key).filter(|target| target.source_key == text)
                    {
                        target.clone()
                    } else {
                        let (source_key, known) = parsed
                            .entry(path.clone())
                            .or_insert_with(|| (text.clone(), sites(&text)));
                        let site = declared_at(known, declared)?.clone();
                        let target = Rc::new(CallTarget {
                            path,
                            declared,
                            source_key: source_key.clone(),
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
            graphs.push((uri.clone(), text, calls));
        }
        let input = Rc::new(crate::semantic::CheckedInputs::capture(analysis));
        let mut kept = self.call_graphs.borrow_mut();
        let mut answer = None;
        for (uri, text, calls) in graphs {
            let graph = Rc::new(CallGraph {
                generation,
                input: input.clone(),
                facts_accepted: analysis.facts_accepted.clone(),
                text,
                calls,
            });
            if uri == *checked {
                answer = Some(graph.clone());
            }
            kept.insert(uri, graph);
        }
        let mut bytes = kept.values().map(|g| g.bytes()).sum::<usize>();
        while bytes > 16 << 20 {
            let key = kept
                .keys()
                .find(|uri| *uri != checked)
                .or_else(|| kept.keys().next())
                .cloned()
                .unwrap();
            bytes -= kept.remove(&key).unwrap().bytes();
        }
        answer
    }

    fn graph_current(&self, uri: &DocumentUri, graph: &CallGraph, allow_own_edit: bool) -> bool {
        self.graph_context(uri, graph, allow_own_edit).is_some()
    }

    /// Select current graph authority before projection; final acceptance uses the frozen context.
    fn graph_context(
        &self,
        uri: &DocumentUri,
        graph: &CallGraph,
        allow_own_edit: bool,
    ) -> Option<GraphContext> {
        if !graph.accepted() {
            return None;
        }
        let doc = self.document(uri).ok()?;
        let env = self.environment.as_ref()?;
        let (files, root, dirs) = self.compile_inputs(uri, &doc.text)?;
        let mut cache = self.semantic.borrow_mut();
        if cache
            .cached(env, &root, &dirs, &files)
            .is_some_and(|analysis| !graph.belongs_to(&analysis.facts_accepted))
        {
            return None;
        }
        let ignored = allow_own_edit.then(|| std::path::PathBuf::from(uri.path()));
        if graph.input.root != root || graph.input.dirs != dirs || graph.input.os.is_some() {
            return None;
        }
        let reads = graph.input.projected_reads(
            env,
            &files,
            cache.authority(),
            ignored.as_deref(),
            &self.input_proof.borrow(),
        )?;
        if !reads.is_current(&*env.fs) {
            return None;
        }
        Some(GraphContext {
            files,
            authority: cache.authority().clone(),
            ignored,
            reads,
        })
    }

    /// Validate all scanned positive and negative graph facts after target projection.
    /// Joining histories rejects contradictory snapshots even if separate replays could pass.
    fn graph_answer_current(&self, contexts: &[(Rc<CallGraph>, GraphContext)]) -> bool {
        let Some(env) = self.environment.as_ref() else {
            return false;
        };
        let mut joined = crate::semantic::Reads::default();
        for (graph, context) in contexts {
            // Preserve initial selection's first projection responses, including new aliases
            // that were not present in the independently retained graph snapshot.
            joined.merge(&context.reads);
            if !graph.accepted() {
                return false;
            }
            let Some(reads) = graph.input.projected_reads(
                env,
                &context.files,
                &context.authority,
                context.ignored.as_deref(),
                &crate::semantic::Observations::default(),
            ) else {
                return false;
            };
            joined.merge(&reads);
        }
        joined.disk.merge(&self.input_proof.borrow());
        !jaic::interrupt::fired()
            && joined.is_current(&*env.fs)
            && !jaic::interrupt::fired()
            && contexts.iter().all(|(graph, _)| graph.accepted())
    }

    fn call_graph(&self, uri: &DocumentUri) -> Option<Rc<CallGraph>> {
        let (uri, doc) = self.document_entry(uri).ok()?;
        let old = self.call_graphs.borrow().get(uri).cloned();
        if old.as_ref().is_some_and(|graph| {
            graph.generation == self.semantic_revision()
                && graph.text.as_ref() == doc.text
                && self.graph_current(uri, graph, false)
        }) {
            return old;
        }
        self.with_semantic(uri, &doc.text, |analysis, path| {
            let graph = self.remember_call_graphs(uri, analysis);
            analysis.focus(path);
            graph
        })
        .or_else(|| old.filter(|graph| self.graph_current(uri, graph, true)))
    }

    /// Resolve a stored declaration against the current file. An external declaration whose
    /// file changed waits for a new check; a same-file declaration moves with its caller.
    fn call_target(
        &self,
        fs: &crate::semantic::ObservedFs<'_>,
        uri: &DocumentUri,
        here: &[Site],
        shift: crate::semantic::Shift,
        fact: &CallFact,
        sources: &mut TargetSources,
    ) -> Option<(CallHierarchyItem, usize)> {
        let target = self.projected_uri(fs, Path::new(&fact.target.path))?;
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
                let text: Arc<str> = self
                    .documents
                    .get(&target)
                    .or_else(|| self.lint_configs.get(&target))
                    .map_or_else(
                        || fact.target.source_key.clone(),
                        |doc| Arc::from(doc.text.as_str()),
                    );
                Some((text.clone(), text))
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
        let _query = self.discovery_query();
        let Some(fs) = self.config_filesystem() else {
            return Ok(Vec::new());
        };
        let requested = DocumentUri::parse(&item.uri)?;
        let Ok((uri, _)) = self.projected_document_entry(&fs, &requested) else {
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
        let mut contexts = Vec::new();
        for (caller_uri, graph) in graphs {
            let Some(context) = self.graph_context(&caller_uri, &graph, true) else {
                continue;
            };
            contexts.push((graph.clone(), context));
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
                    self.call_target(&fs, &caller_uri, &here, shift, fact, &mut sources)
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
        if !self.graph_answer_current(&contexts) {
            return Ok(Vec::new());
        }
        Ok(grouped.finish())
    }

    /// The procedures `item` calls, with where.
    pub fn outgoing_calls(
        &self,
        item: &CallHierarchyItem,
    ) -> Result<Vec<CallHierarchyCall>, Error> {
        let _query = self.discovery_query();
        let Some(fs) = self.config_filesystem() else {
            return Ok(Vec::new());
        };
        let requested = DocumentUri::parse(&item.uri)?;
        let Ok((uri, doc)) = self.projected_document_entry(&fs, &requested) else {
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
        let Some(context) = self.graph_context(uri, &graph, true) else {
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
            let Some((callee, key)) = self.call_target(&fs, uri, &here, shift, fact, &mut sources)
            else {
                continue;
            };
            let len = callee_len(&doc.text, at);
            let Ok(from) = doc.index.range(&doc.text, Span::new(at, at + len)) else {
                continue;
            };
            grouped.add(callee, key, from);
        }
        if !self.graph_answer_current(&[(graph, context)]) {
            return Ok(Vec::new());
        }
        Ok(grouped.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutable_query_retires_only_its_own_analysis_call_graph_view() {
        let mut session = Session::with_environment(
            crate::Limits::default(),
            crate::Environment {
                fs: Rc::new(jaic::sema::NativeFs),
                services: None,
                options: Box::new(|_, _| {
                    let mut options = jaic::sema::Options::host();
                    options.runtime_support = false;
                    options.stack_trace = false;
                    options
                }),
            },
        );
        let uri = DocumentUri::parse("file:///fact-owner/main.jai").unwrap();
        let source = "target :: () {} main :: () { target(); }";
        session.open(uri.clone(), 1, source.into()).unwrap();
        let other = DocumentUri::parse("file:///fact-owner/other.jai").unwrap();
        session
            .open(other.clone(), 1, "other :: () {}".into())
            .unwrap();
        let first = session.call_graph(&uri).unwrap();
        assert_eq!(first.calls.len(), 1);
        session
            .with_semantic(&uri, source, |analysis, path| {
                analysis.hover(path, source.find("target").unwrap());
                Some(())
            })
            .unwrap();
        let after = session.call_graph(&uri).unwrap();
        assert!(
            !Rc::ptr_eq(&first, &after),
            "same input proof cannot establish mutable fact revision"
        );
        assert!(
            Rc::ptr_eq(&first.calls[0].target, &after.calls[0].target),
            "unchanged exact callee source shares its immutable parsed target"
        );
        session.diagnostics(&other).unwrap();
        let retained = session.call_graph(&uri).unwrap();
        assert!(
            Rc::ptr_eq(&after, &retained),
            "another analysis does not mutate this graph owner"
        );
        session.semantic.borrow_mut().forget_compiles();
        session
            .with_semantic(&uri, source, |analysis, path| {
                analysis.hover(path, 1);
                Some(())
            })
            .unwrap();
        let current = session.call_graph(&uri).unwrap();
        assert!(
            !Rc::ptr_eq(&retained, &current),
            "a different live owner takes precedence over an older retained view"
        );
        let changed = "target :: (argument: int) {} main :: () { target(1); }";
        session
            .change(
                &uri,
                2,
                &[crate::TextChange {
                    range: None,
                    range_length: None,
                    text: changed.into(),
                }],
            )
            .unwrap();
        session.diagnostics(&uri).unwrap();
        let updated = session.call_graph(&uri).unwrap();
        assert_eq!(updated.calls.len(), 1);
        assert!(
            !Rc::ptr_eq(&current.calls[0].target, &updated.calls[0].target),
            "same path and declaration offset cannot reuse changed source"
        );
        assert!(
            updated.calls[0]
                .target
                .site
                .header
                .contains("argument: int")
        );
    }

    #[cfg(unix)]
    #[test]
    fn combined_graph_answer_includes_conflicting_negative_graph_history() {
        use jaic::sema::{FileSystem, NativeFs};
        use std::{cell::Cell, path::PathBuf};
        struct SwitchFs {
            alias: PathBuf,
            second: PathBuf,
            armed: Cell<bool>,
        }
        impl FileSystem for SwitchFs {
            fn is_native(&self) -> bool {
                true
            }
            fn read(&self, path: &Path) -> Option<Vec<u8>> {
                NativeFs.read(path)
            }
            fn is_file(&self, path: &Path) -> bool {
                NativeFs.is_file(path)
            }
            fn is_dir(&self, path: &Path) -> bool {
                NativeFs.is_dir(path)
            }
            fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
                NativeFs.list_dir(path)
            }
            fn stamp(&self, path: &Path) -> Option<u64> {
                NativeFs.stamp(path)
            }
            fn canonical_exact(&self, path: &Path) -> Option<PathBuf> {
                NativeFs.canonical_exact(path)
            }
            fn canonical(&self, path: &Path) -> PathBuf {
                let answer = NativeFs.canonical(path);
                if path == self.alias && self.armed.replace(false) {
                    std::fs::remove_file(&self.alias).unwrap();
                    std::os::unix::fs::symlink(&self.second, &self.alias).unwrap();
                }
                answer
            }
        }
        let temp = test_support::TempDir::new("graph-history-union");
        let epoch = test_support::TempDir::new("graph-projection-epoch");
        let epoch_first = epoch.path().join("first");
        let epoch_second = epoch.path().join("second");
        let epoch_alias = epoch.path().join("alias");
        std::fs::write(&epoch_first, "first").unwrap();
        std::fs::write(&epoch_second, "second").unwrap();
        std::os::unix::fs::symlink(&epoch_first, &epoch_alias).unwrap();
        let first = epoch.path().join("history-first");
        let second = epoch.path().join("history-second");
        let alias = epoch.path().join("history-alias");
        std::fs::create_dir(&first).unwrap();
        std::fs::create_dir(&second).unwrap();
        std::os::unix::fs::symlink(&first, &alias).unwrap();
        let fs = Rc::new(SwitchFs {
            alias: alias.clone(),
            second: second.clone(),
            armed: Cell::new(false),
        });
        let mut session = Session::with_environment(
            crate::Limits::default(),
            crate::Environment {
                fs: fs.clone(),
                services: None,
                options: Box::new(|_, _| {
                    let mut options = jaic::sema::Options::host();
                    options.runtime_support = false;
                    options.stack_trace = false;
                    options
                }),
            },
        );
        let source = "target :: () {} main :: () { target(); }";
        let mut uris = Vec::new();
        for name in ["one.jai", "two.jai"] {
            let path = temp.path().join(name);
            std::fs::write(&path, source).unwrap();
            uris.push(DocumentUri::from_path(&path).unwrap());
        }
        for uri in &uris {
            session.open(uri.clone(), 1, source.into()).unwrap();
        }
        let mut graphs = Vec::new();
        for uri in &uris {
            let graph = session.call_graph(uri).unwrap();
            let context = session.graph_context(uri, &graph, false).unwrap();
            graphs.push((graph, context));
        }
        let epoch_graph = graphs[0].0.clone();
        let mut epoch_context = GraphContext {
            files: graphs[0].1.files.clone(),
            authority: graphs[0].1.authority.clone(),
            ignored: graphs[0].1.ignored.clone(),
            reads: graphs[0].1.reads.clone(),
        };
        // These are two genuine first canonical responses from different retained contexts.
        let observe = || {
            let proof = std::cell::RefCell::new(crate::semantic::Observations::default());
            let fs = crate::semantic::ObservedFs {
                base: &NativeFs,
                proof: Some(&proof),
                secondary: None,
            };
            jaic::paths::file_identity(&fs, &alias);
            proof.into_inner()
        };
        let a = observe();
        std::fs::remove_file(&alias).unwrap();
        std::os::unix::fs::symlink(&second, &alias).unwrap();
        let b = observe();
        for (at, (graph, _)) in graphs.iter_mut().enumerate() {
            let mut input = (*graph.input).clone();
            let mut reads = (*input.reads).clone();
            reads.disk.merge(if at == 0 {
                &a
            } else {
                &b
            });
            input.reads = Rc::new(reads);
            *graph = Rc::new(CallGraph {
                generation: graph.generation,
                input: Rc::new(input),
                facts_accepted: graph.facts_accepted.clone(),
                text: graph.text.clone(),
                calls: if at == 0 {
                    Vec::new()
                } else {
                    graph
                        .calls
                        .iter()
                        .map(|call| CallFact {
                            start: call.start,
                            end: call.end,
                            target: call.target.clone(),
                        })
                        .collect()
                },
            });
        }
        assert!(
            graphs[0].0.calls.is_empty(),
            "negative graph still contributes authority"
        );
        assert!(!graphs[1].0.calls.is_empty());
        let reset = || {
            std::fs::remove_file(&alias).unwrap();
            std::os::unix::fs::symlink(&first, &alias).unwrap();
            fs.armed.set(true);
        };
        reset();
        assert!(graphs[0].0.input.reads.is_current(&*fs));
        assert!(
            graphs[1].0.input.reads.is_current(&*fs),
            "separate replays can both succeed"
        );
        reset();
        assert!(
            !session.graph_answer_current(&graphs),
            "one answer cannot combine conflicting positive and negative histories"
        );
        fs.armed.set(false);
        let unprojected = epoch_context.reads.clone();
        let observation = std::cell::RefCell::new(crate::semantic::Observations::default());
        let observed = crate::semantic::ObservedFs {
            base: &*fs,
            proof: Some(&observation),
            secondary: None,
        };
        jaic::paths::file_identity(&observed, &epoch_alias);
        epoch_context.reads.disk.merge(&observation.into_inner());
        std::fs::remove_file(&epoch_alias).unwrap();
        std::os::unix::fs::symlink(&epoch_second, &epoch_alias).unwrap();
        let discarded = GraphContext {
            files: epoch_context.files.clone(),
            authority: epoch_context.authority.clone(),
            ignored: epoch_context.ignored.clone(),
            reads: unprojected,
        };
        assert!(
            session.graph_answer_current(&[(epoch_graph.clone(), discarded)]),
            "without first projection history the old graph still independently validates"
        );
        assert!(
            !session.graph_answer_current(&[(epoch_graph, epoch_context)]),
            "initial projection responses cannot be discarded and reblessed later"
        );
    }
}
