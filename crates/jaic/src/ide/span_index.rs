//! Lookups by source span over what the analysis recorded, built once per compile on first use.
//!
//! Each query below answers from a map that is filled the first time any query of its kind is
//! asked and kept until the record changes (a different count of references, entities or
//! procedures, or a sketch beginning). Features ask by the span of the syntax they hold:
//!
//! - [`Compiler::ide_entities_at`]: what a name (a use) resolved to;
//! - [`Compiler::ide_locals_declared_at`]: the variables a span declares;
//! - [`Compiler::ide_procs_with_header`]: the procedures made from a header;
//! - [`Compiler::ide_uses_of_proc`] and [`Compiler::ide_procs_named_at`]: where procedures are
//!   named, and the reverse;
//! - [`Compiler::ide_body_is_clean`]: whether a procedure body checked to the end in every
//!   instance.
//!
//! All of them stop at the sketch mark (`IdeFacts::sketch_from`): the references, entities and
//! procedures after it come from sketches with stand-in types, which say nothing about the
//! program. This is the one place that rule is applied to these lookups.
use super::IdeWhat;
use super::SketchMark;
use crate::fxhash::{HashMap, HashSet};
use crate::sema::procs::BodyState;
use crate::sema::scope::{EntityId, EntityKind, EntityState, Resolved};
use crate::sema::{Compiler, ProcId, Value};
use crate::source::{FileId, Span};
use std::cell::OnceCell;
use std::hash::Hash;
use std::ops::Deref;
use std::rc::Rc;

/// What the maps were built from: they are dropped when it changes.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Key {
    refs: usize,
    entities: usize,
    procs: usize,
    sketched: bool,
}

/// The maps of one state of the record, each built on first use.
#[derive(Default)]
struct Sections {
    idents: OnceCell<Map<Span, EntityId>>,
    locals: OnceCell<Map<Span, EntityId>>,
    headers: OnceCell<Map<Span, ProcId>>,
    uses: OnceCell<Map<ProcId, Span>>,
    named: OnceCell<Map<Span, ProcId>>,
    clean: OnceCell<HashSet<Span>>,
}

type Map<K, V> = HashMap<K, Rc<[V]>>;

#[derive(Default)]
pub struct SpanIndex {
    key: Option<Key>,
    sections: Rc<Sections>,
}

/// What a lookup found: a view of the index that stays valid whatever is asked next.
pub struct Found<V>(Option<Rc<[V]>>);

impl<V> Deref for Found<V> {
    type Target = [V];

    fn deref(&self) -> &[V] {
        self.0.as_deref().unwrap_or(&[])
    }
}

fn found<K: Hash + Eq, V>(map: &Map<K, V>, key: &K) -> Found<V> {
    Found(map.get(key).cloned())
}

fn grouped<K: Hash + Eq, V>(pairs: impl IntoIterator<Item = (K, V)>) -> Map<K, V> {
    let mut lists: HashMap<K, Vec<V>> = HashMap::default();
    for (k, v) in pairs {
        lists.entry(k).or_default().push(v);
    }
    lists.into_iter().map(|(k, v)| (k, Rc::from(v))).collect()
}

/// Whether the analysis records each file, asked once per file.
#[derive(Default)]
struct Wanted(HashMap<FileId, bool>);

impl Wanted {
    fn has(&mut self, c: &Compiler, file: FileId) -> bool {
        *self.0.entry(file).or_insert_with(|| {
            // The checker has already asked about most files.
            let known = c.ed.facts().and_then(|ide| ide.wanted.get(&file).copied());
            known.unwrap_or_else(|| c.ide_wants_now(file))
        })
    }
}

impl Compiler {
    /// The maps for the record as it stands, with where sketches began (the end of the record
    /// when none has run). `None` when nothing is being recorded.
    fn ide_sections(&self) -> Option<(Rc<Sections>, SketchMark)> {
        let ide = self.ed.facts()?;
        let key = Key {
            refs: ide.refs.len(),
            entities: self.ns.entities.len(),
            procs: self.pr.procs.len(),
            sketched: ide.sketch_from.is_some(),
        };
        let mark = ide.sketch_from.unwrap_or(SketchMark {
            refs: key.refs,
            entities: key.entities,
            procs: key.procs,
        });
        let mut index = self.ed.index.borrow_mut();
        if index.key != Some(key) {
            *index = SpanIndex {
                key: Some(key),
                sections: Rc::default(),
            };
        }
        Some((index.sections.clone(), mark))
    }

    /// The entities the name at `span` (a use, not a declaration) resolved to: several when a
    /// polymorphic body was checked once per instance.
    pub fn ide_entities_at(&self, span: Span) -> Found<EntityId> {
        let Some((sections, mark)) = self.ide_sections() else {
            return Found(None);
        };
        let idents = sections.idents.get_or_init(|| {
            let refs = self
                .ed
                .facts()
                .map_or(&[][..], |ide| &ide.refs[..mark.refs]);
            let mut seen: HashSet<(Span, EntityId)> = HashSet::default();
            let mut pairs: Vec<(Span, EntityId)> = Vec::new();
            for r in refs {
                if let (false, IdeWhat::Entity(e)) = (r.decl, &r.what)
                    && seen.insert((r.span, *e))
                {
                    pairs.push((r.span, *e));
                }
            }
            grouped(pairs)
        });
        found(idents, &span)
    }

    /// The runtime variables (locals, parameters, loop variables) declared at `span`: a local's
    /// name, a parameter's whole declaration, a loop's whole statement. Only files the analysis
    /// records are indexed.
    pub fn ide_locals_declared_at(&self, span: Span) -> Found<EntityId> {
        let Some((sections, mark)) = self.ide_sections() else {
            return Found(None);
        };
        let locals = sections.locals.get_or_init(|| {
            let mut wanted = Wanted::default();
            grouped(
                self.ns.entities[..mark.entities]
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| {
                        matches!(e.kind, EntityKind::Local { .. }) && wanted.has(self, e.span.file)
                    })
                    .map(|(i, e)| (e.span, EntityId(i as u32))),
            )
        });
        found(locals, &span)
    }

    /// The procedures made from the header at `span` (instances of a polymorphic one share it).
    /// Only files the analysis records are indexed.
    pub fn ide_procs_with_header(&self, span: Span) -> Found<ProcId> {
        let Some((sections, mark)) = self.ide_sections() else {
            return Found(None);
        };
        let headers = sections.headers.get_or_init(|| {
            let mut wanted = Wanted::default();
            grouped(
                self.pr.procs[..mark.procs]
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| wanted.has(self, p.lit.header.span.file))
                    .map(|(i, p)| (p.lit.header.span, ProcId(i as u32))),
            )
        });
        found(headers, &span)
    }

    /// Every (span, procedure) pair where a procedure is named, in the order recorded.
    fn ide_proc_names(&self, mark: SketchMark) -> Vec<(Span, ProcId)> {
        let Some(ide) = self.ed.facts() else {
            return Vec::new();
        };
        let mut pairs: Vec<(Span, ProcId)> = Vec::new();
        for r in ide.refs[..mark.refs].iter().filter(|r| !r.decl) {
            let named: Vec<ProcId> = match &r.what {
                IdeWhat::Procs(ps) => ps.clone(),
                IdeWhat::Entity(e) => match &self.entity(*e).state {
                    EntityState::Done(
                        Resolved::Proc(p)
                        | Resolved::Const {
                            value: Value::Proc(p),
                            ..
                        },
                    ) => vec![*p],
                    EntityState::Done(Resolved::ProcSet(ps)) => ps.clone(),
                    _ => Vec::new(),
                },
                _ => Vec::new(),
            };
            pairs.extend(named.into_iter().map(|p| (r.span, p)));
        }
        pairs
    }

    /// Where `proc` is named in the recorded files, in the order recorded.
    pub fn ide_uses_of_proc(&self, proc: ProcId) -> Found<Span> {
        let Some((sections, mark)) = self.ide_sections() else {
            return Found(None);
        };
        let uses = sections
            .uses
            .get_or_init(|| grouped(self.ide_proc_names(mark).into_iter().map(|(s, p)| (p, s))));
        found(uses, &proc)
    }

    /// The procedures the name at `span` refers to (the reverse of [`Self::ide_uses_of_proc`]).
    pub fn ide_procs_named_at(&self, span: Span) -> Found<ProcId> {
        let Some((sections, mark)) = self.ide_sections() else {
            return Found(None);
        };
        let named = sections.named.get_or_init(|| {
            let mut pairs = self.ide_proc_names(mark);
            let mut seen = HashSet::default();
            pairs.retain(|pair| seen.insert(*pair));
            grouped(pairs)
        });
        found(named, &span)
    }

    /// The procedure body (the block's span) was checked to the end in every instance of its
    /// procedure, so what was recorded inside it is the whole story. A body no instance needed,
    /// or one that failed or stopped in any instance, is not clean.
    pub fn ide_body_is_clean(&self, body: Span) -> bool {
        let Some((sections, mark)) = self.ide_sections() else {
            return false;
        };
        let clean = sections.clean.get_or_init(|| {
            let mut wanted = Wanted::default();
            let mut done: HashSet<Span> = HashSet::default();
            let mut failed: HashSet<Span> = HashSet::default();
            for p in &self.pr.procs[..mark.procs] {
                let Some(block) = &p.lit.body else {
                    continue;
                };
                if !wanted.has(self, block.span.file) {
                    continue;
                }
                match p.body_state {
                    BodyState::Done => {
                        done.insert(block.span);
                    }
                    BodyState::Queued | BodyState::Lowering => {
                        failed.insert(block.span);
                    }
                    BodyState::NotNeeded => {}
                }
            }
            done.difference(&failed).copied().collect()
        });
        clean.contains(&body)
    }
}
