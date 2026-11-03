//! Compilation-local, parameter-relative effects. Only checked instances enter the call
//! graph; SCCs are solved from callees upwards and recursion starts without guarantees.
use super::lifetime_movement::{self, Mutation, Selector};
use super::lifetime_places::{Places, Projection};
use jaic::fxhash::{HashMap, HashSet};
use jaic::ide::lifetimes::{
    LifetimeArgument, LifetimeBody, LifetimeEvent as Event, LifetimeOperand,
};
use jaic::ir::Val;
use jaic::sema::{Compiler, ProcId, procs::BodyState};
use jaic::source::Span;
use jaic::types::{ArrayKind, TypeId, TypeKind};

mod address_arithmetic;
mod allocation_effects;
mod allocator_context;
mod allocator_storage;
mod allocators;
mod captures;
mod contracts;
mod elements;
mod execution;
mod flow;
mod indirect;
mod infer;
mod interfaces;
mod nullable;
mod observable;
mod ordering;
mod origins;
mod possible;
mod prepared;
mod primitives;
mod reads;
mod return_fields;
mod return_may;
mod selected;
pub(in crate::rules) mod swap;
mod templates;
mod temporary;
#[cfg(test)]
mod tests;
mod writes;
use flow::{acyclic, guaranteed, returning_blocks, sccs};
pub(super) use ordering::Ordering;
pub(super) use prepared::Prepared;
pub(super) use reads::address_query;

const MAX_PATH: usize = 8;
const MAX_CHAIN: usize = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) enum Step {
    Field(jaic::intern::Sym),
    Deref,
    Storage,
    Element(i128),
}

/// A built-in descriptor's own backing storage, excluding its elements' resources.
pub(super) fn descriptor_storage(steps: &[Step]) -> bool {
    matches!(steps, [Step::Storage])
        || matches!(steps, [Step::Field(name), Step::Deref] if name.as_str() == "data")
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum Root {
    Parameter(usize),
    Local(usize),
    Heap,
    CallerAllocator,
    AllocatorParameter(usize),
    Temporary,
    Watermark,
    Until(ProcId),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Path {
    pub root: Root,
    pub steps: Vec<Step>,
}

impl Path {
    fn append(mut self, steps: &[Step]) -> Option<Self> {
        if self.steps.len() + steps.len() > MAX_PATH {
            return None;
        }
        self.steps.extend_from_slice(steps);
        Some(self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Kind {
    Resize,
    Free,
    Write,
    Unknown,
    Advance,
    ResetTemporary,
    RestoreTemporary,
    StoreTemporary,
    Mutation(Mutation),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Effect {
    pub target: Path,
    pub kind: Kind,
    pub guard: Option<(usize, u64)>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Summary {
    pub memory: Option<jaic::ide::lifetimes::LifetimeMemoryEffects>,
    pub reads: HashSet<Path>,
    pub may_reads: HashSet<Path>,
    pub address_queries: HashSet<usize>,
    pub read_formals: HashSet<usize>,
    pub may_read_formals: HashSet<usize>,
    pub unknown_write: HashSet<usize>,
    pub escapes: HashSet<usize>,
    pub normal_return: Option<bool>,
    pub may: HashMap<Effect, Vec<Span>>,
    pub guaranteed: HashMap<Effect, Vec<Span>>,
    pub returns: HashMap<usize, (Path, Option<(usize, u64)>)>,
    pub may_returns: HashSet<usize>,
    pub may_return_fields: HashSet<(usize, Vec<Step>)>,
    pub return_fields: HashMap<(usize, Vec<Step>), Path>,
    pub allocations: HashMap<usize, (Path, Option<(usize, u64)>)>,
    pub changes_allocator: bool,
    pub observable: observable::Observable,
    pub pushes_context: bool,
    pub ordering: Option<Ordering>,
    pub rounded_address: Option<usize>,
    pub adjusted_returns: HashSet<usize>,
    pub allocation_effects: HashSet<(Root, Option<(usize, u64)>)>,
    pub elements: HashMap<usize, (Path, Selector)>,
}

impl Summary {
    pub fn preserves_address(&self, formal: usize) -> bool {
        !self.unknown_write.contains(&formal)
    }
}

#[derive(Default)]
pub(super) struct Summaries {
    pub compile_time_only: HashSet<ProcId>,
    pub frozen_returns: HashSet<ProcId>,
    pub call_sites: HashSet<Span>,
    pub allocators: allocators::Allocators,
    pub targets: super::lifetime_targets::Universe,
    pub procs: HashMap<ProcId, Summary>,
    pub sccs: usize,
    pub iterations: usize,
}

#[derive(Clone, Debug)]
enum Def<'a> {
    Load(usize, TypeId),
    Copy(Val),
    Address(usize),
    Borrow(usize),
    Parameter(usize),
    Call(ProcId, &'a [LifetimeArgument], usize),
    Resize(Val, &'a [LifetimeArgument]),
    Indirect(Val, &'a [LifetimeArgument], usize),
}

type FieldQuery = (Val, TypeId, Vec<Projection>);

pub(super) struct Resolver<'a> {
    pub places: Places,
    allocators: &'a allocators::Allocators,
    context: allocator_context::Context,
    allocation_mode: std::cell::Cell<bool>,
    return_mode: std::cell::Cell<bool>,
    return_definitions: std::cell::OnceCell<(HashSet<usize>, HashSet<usize>)>,
    may_field_origins: std::cell::RefCell<HashMap<FieldQuery, bool>>,
    field_origins: std::cell::RefCell<HashMap<FieldQuery, Path>>,
    observable_calls: observable::Calls<'a>,
    ordering_base: Option<Ordering>,
    rounded_address_base: Option<usize>,
    comparisons: std::cell::OnceCell<HashMap<Val, &'a jaic::ide::lifetimes::LifetimeComparison>>,
    return_cuts: std::cell::RefCell<HashMap<usize, bool>>,
    return_contracts: HashMap<usize, (Path, Option<(usize, u64)>)>,
    compiler: &'a Compiler,
    body: &'a LifetimeBody,
    core: std::rc::Rc<prepared::Core<'a>>,
    targets: &'a super::lifetime_targets::Universe,
}


fn value(arg: &LifetimeArgument) -> Option<Val> {
    match arg.operand {
        LifetimeOperand::Value(v) | LifetimeOperand::Place(v) => Some(v),
        LifetimeOperand::Other => None,
    }
}

impl Resolver<'_> {
    pub fn value(&self, v: Val, summaries: &HashMap<ProcId, Summary>) -> Option<Path> {
        if self
            .return_guard(v, summaries, &mut HashSet::default())?
            .is_some()
        {
            return None;
        }
        self.trace(v, summaries, &mut HashSet::default(), 0)
    }

    fn potential(&self, input: Val) -> HashSet<usize> {
        fn place(r: &Resolver<'_>, p: usize, seen: &mut HashSet<Val>, out: &mut HashSet<usize>) {
            let (root, _) = &r.places.paths[p];
            let id = r.places.root_of(p);
            if let Some((formal, _)) = r.params.get(&id) {
                out.insert(*formal);
            }
            for &value in r.writes.get(&id).into_iter().flatten() {
                walk(r, value, seen, out);
            }
            walk(r, root.0, seen, out);
        }
        fn walk(r: &Resolver<'_>, value_: Val, seen: &mut HashSet<Val>, out: &mut HashSet<usize>) {
            if !seen.insert(value_) {
                return;
            }
            for def in r.values.get(&value_).into_iter().flatten() {
                match def {
                    Def::Parameter(formal) => {
                        out.insert(*formal);
                    }
                    Def::Load(p, _) => {
                        place(r, *p, seen, out);
                        for &v in r.writes.get(p).into_iter().flatten() {
                            walk(r, v, seen, out);
                        }
                    }
                    Def::Address(p) | Def::Borrow(p) => place(r, *p, seen, out),
                    Def::Copy(value) => walk(r, *value, seen, out),
                    Def::Call(_, args, _) | Def::Indirect(_, args, _) => {
                        for arg in *args {
                            if let Some(v) = value(arg) {
                                walk(r, v, seen, out);
                            }
                        }
                    }
                    Def::Resize(_, args) => {
                        if let Some(v) = args.get(3).and_then(value) {
                            walk(r, v, seen, out);
                        }
                    }
                }
            }
        }
        let mut out = HashSet::default();
        walk(self, input, &mut HashSet::default(), &mut out);
        out
    }

    pub fn location(
        &self,
        p: usize,
        summaries: &HashMap<ProcId, Summary>,
        visiting: &mut HashSet<Val>,
        depth: usize,
    ) -> Option<Path> {
        if depth > 24 {
            return None;
        }
        let (root, projections) = &self.places.paths[p];
        let root_id = self.places.root_of(p);
        let parameter = self.params.contains_key(&root_id)
            || self
                .values
                .get(&root.0)
                .is_some_and(|defs| defs.iter().any(|d| matches!(d, Def::Parameter(_))));
        if !parameter
            && self.compiler.ir_ty(root.1).is_none()
            && !projections.is_empty()
            && let Some(path) =
                self.returned_field_origin(root.0, root.1, projections, summaries, depth + 1)
        {
            return Some(path);
        }
        let mut path;
        let mut rest = projections.as_slice();
        if rest.first() == Some(&Projection::Deref) {
            // A descriptor element Borrow uses this synthetic projection to mean storage.
            if matches!(
                self.compiler.ty.types.kind(root.1),
                TypeKind::Array { .. } | TypeKind::String
            ) {
                path = Path {
                    root: Root::Local(root_id),
                    steps: vec![Step::Storage],
                };
            } else if let Some([source]) = self.writes.get(&root_id).map(Vec::as_slice) {
                path = self.trace(*source, summaries, visiting, depth + 1)?;
            } else {
                path = self.trace(root.0, summaries, visiting, depth + 1)?;
            }
            rest = &rest[1..];
        } else if let Some((formal, ty)) = self.params.get(&root_id) {
            if self.compiler.ty.types.is_pointer(*ty) && !self.places.indirect.contains(&root_id) {
                return None;
            }
            path = Path {
                root: Root::Parameter(*formal),
                steps: Vec::new(),
            };
        } else {
            path = Path {
                root: Root::Local(root_id),
                steps: Vec::new(),
            };
            if self.compiler.ir_ty(root.1).is_none()
                && let Some(source) = self.return_source(root_id)
                && source != root.0
                && let Some(from) = self.trace(source, summaries, visiting, depth + 1)
                && (matches!(
                    from.root,
                    Root::Parameter(_)
                        | Root::Heap
                        | Root::CallerAllocator
                        | Root::AllocatorParameter(_)
                        | Root::Temporary
                        | Root::Watermark
                        | Root::Until(_)
                ) || matches!(
                    self.compiler.ty.types.kind(root.1),
                    TypeKind::Array {
                        kind: ArrayKind::View,
                        ..
                    } | TypeKind::String
                ) && from.steps.contains(&Step::Storage))
            {
                path = from;
            }
            for def in self.values.get(&root.0).into_iter().flatten() {
                match def {
                    Def::Copy(source) => {
                        path = self.trace(*source, summaries, visiting, depth + 1)?;
                        break;
                    }
                    Def::Borrow(region) => {
                        path = self
                            .location(*region, summaries, visiting, depth + 1)?
                            .append(&[Step::Storage])?;
                        break;
                    }
                    _ => {}
                }
                if let Def::Load(place, ty) = def
                    && self.compiler.ty.types.pointee(*ty) == Some(root.1)
                    && let Some([source]) = self.writes.get(place).map(Vec::as_slice)
                {
                    path = self.trace(*source, summaries, visiting, depth + 1)?;
                    break;
                }
            }
        }
        for projection in rest {
            path = path.append(&[match projection {
                Projection::Field(n) => Step::Field(*n),
                Projection::Element(i) => Step::Element(*i),
                Projection::DynamicElement(_) => return None,
                Projection::Deref => Step::Deref,
            }])?;
        }
        Some(path)
    }

    fn argument_trace(
        &self,
        arg: &LifetimeArgument,
        all: &HashMap<ProcId, Summary>,
        visiting: &mut HashSet<Val>,
        depth: usize,
    ) -> Option<Path> {
        let v = value(arg)?;
        // Offset-zero aggregate fields share the base pointer's SSA value. The typed
        // place identifies the descriptor being passed, rather than its containing object.
        if self.compiler.ir_ty(arg.ty).is_none()
            && let Some(p) = self.places.existing(v, arg.ty)
            && !self.places.paths[p].1.is_empty()
        {
            return self.location(p, all, visiting, depth + 1);
        }
        if let Some(p) = self.stored_address(v, &mut HashSet::default()) {
            return self.location(p, all, visiting, depth + 1);
        }
        if let Some(target) = self.compiler.ty.types.pointee(arg.ty) {
            let mut selected = None;
            for def in self.values.get(&v).into_iter().flatten() {
                if let Def::Address(p) = def
                    && self.places.types.get(p) == Some(&target)
                {
                    let path = self.location(*p, all, visiting, depth + 1)?;
                    if selected.as_ref().is_some_and(|old| old != &path) {
                        return None;
                    }
                    selected = Some(path);
                }
            }
            if selected.is_some() {
                return selected;
            }
        }
        self.trace(v, all, visiting, depth + 1)
    }

    fn substitute(
        &self,
        path: &Path,
        args: &[LifetimeArgument],
        summaries: &HashMap<ProcId, Summary>,
        visiting: &mut HashSet<Val>,
        depth: usize,
    ) -> Option<Path> {
        let Root::Parameter(formal) = path.root else {
            return matches!(
                path.root,
                Root::Heap
                    | Root::CallerAllocator
                    | Root::AllocatorParameter(_)
                    | Root::Temporary
                    | Root::Watermark
                    | Root::Until(_)
            )
            .then(|| path.clone());
        };
        let arg = args.iter().find(|a| a.formal == formal)?;
        self.argument_trace(arg, summaries, visiting, depth + 1)?
            .append(&path.steps)
    }

    pub fn effects(
        &self,
        callee: ProcId,
        args: &[LifetimeArgument],
        all: &HashMap<ProcId, Summary>,
        guaranteed: bool,
    ) -> Vec<(Effect, Vec<Span>)> {
        if reads::address_query(self.compiler, callee, args) {
            return Vec::new();
        }
        let Some(summary) = all.get(&callee) else {
            return Vec::new();
        };
        self.summary_effects(summary, args, all, guaranteed)
    }

    fn summary_effects(
        &self,
        summary: &Summary,
        args: &[LifetimeArgument],
        all: &HashMap<ProcId, Summary>,
        guaranteed: bool,
    ) -> Vec<(Effect, Vec<Span>)> {
        let map = if guaranteed {
            &summary.guaranteed
        } else {
            &summary.may
        };
        let mut result: Vec<_> = map
            .iter()
            .filter_map(|(effect, chain)| {
                let mut guard = None;
                if let Some((formal, constant)) = effect.guard {
                    let arg = args.iter().find(|a| a.formal == formal)?;
                    if let Some(actual) = arg.constant {
                        if actual != constant {
                            return None;
                        }
                    } else {
                        let path = self.value(value(arg)?, all)?;
                        let Root::Parameter(formal) = path.root else {
                            return None;
                        };
                        if !path.steps.is_empty() {
                            return None;
                        }
                        guard = Some((formal, constant));
                    }
                }
                let mut target =
                    self.substitute(&effect.target, args, all, &mut HashSet::default(), 0)?;
                if effect.kind == Kind::Free
                    && effect.target.steps.is_empty()
                    && let Root::Parameter(formal) = effect.target.root
                    && let Some(arg) = args.iter().find(|a| a.formal == formal)
                    && self.compiler.ir_ty(arg.ty).is_none()
                {
                    target = target.append(&[Step::Storage])?;
                }
                Some((
                    Effect {
                        target,
                        guard,
                        kind: match effect.kind {
                            Kind::Mutation(mut m) => {
                                let rebase = |s| self.selector_actual(s, args, all);
                                m.first = rebase(m.first);
                                m.second = rebase(m.second);
                                Kind::Mutation(m)
                            }
                            other => other,
                        },
                    },
                    chain.clone(),
                ))
            })
            .collect();
        if !guaranteed {
            result.extend(self.possible_effects(summary, args, all));
        }
        result
    }

    fn allocator(&self, args: &[LifetimeArgument], all: &HashMap<ProcId, Summary>) -> Vec<Effect> {
        let Some(mode) = args.first() else {
            return Vec::new();
        };
        [(1, Kind::Resize, 3), (2, Kind::Free, 3), (8, Kind::Free, 4)]
            .into_iter()
            .filter_map(|(constant, kind, memory)| {
                let guard = if let Some(actual) = mode.constant {
                    if actual != constant {
                        return None;
                    }
                    None
                } else {
                    let path = self.value(value(mode)?, all)?;
                    let Root::Parameter(formal) = path.root else {
                        return None;
                    };
                    if !path.steps.is_empty() {
                        return None;
                    }
                    Some((formal, constant))
                };
                let memory = args
                    .get(memory)
                    .and_then(value)
                    .and_then(|v| self.value(v, all))?;
                Some(Effect {
                    target: memory.clone(),
                    kind,
                    guard,
                })
            })
            .collect()
    }

    pub fn materialize(&mut self, path: &Path) -> Option<usize> {
        self.materialize_region(path, false)
    }

    pub fn materialize_storage(&mut self, path: &Path) -> Option<usize> {
        self.materialize_region(path, true)
    }

    fn materialize_region(&mut self, path: &Path, contents: bool) -> Option<usize> {
        let mut p = match path.root {
            Root::Local(p) => p,
            Root::Parameter(formal) => {
                let (&place, &(_, ty)) = self.params.iter().find(|(_, (n, _))| *n == formal)?;
                if let Some(target) = self.compiler.ty.types.pointee(ty) {
                    let incoming = self.values.iter().find_map(|(v, defs)| {
                        defs.iter()
                            .any(|d| matches!(d, Def::Parameter(n) if *n == formal))
                            .then_some(*v)
                    })?;
                    let root = self.places.place(incoming, ty);
                    let p = self.places.project(root, Projection::Deref);
                    self.places.types.insert(p, target);
                    p
                } else {
                    place
                }
            }
            _ => return None,
        };
        let mut owner = None;
        for step in &path.steps {
            match step {
                Step::Storage => {
                    owner = Some(p);
                    let element = self.places.types.get(&p).and_then(|&ty| {
                        match self.compiler.ty.types.kind(ty) {
                            TypeKind::Array {
                                elem, ..
                            } => Some(*elem),
                            TypeKind::String => Some(TypeId::U8),
                            _ => None,
                        }
                    });
                    if contents {
                        p = self.places.project(p, Projection::Deref);
                        if let Some(ty) = element {
                            self.places.types.insert(p, ty);
                        }
                    }
                }
                Step::Deref => {
                    owner = Some(self.places.data.get(&p).map_or(p, |(parent, _)| *parent));
                    p = self.places.project(p, Projection::Deref);
                }
                Step::Field(name) => {
                    if contents {
                        // A field beyond a dereference owns its own resource value.
                        owner = None;
                    }
                    let parent = p;
                    p = self.places.project(p, Projection::Field(*name));
                    if let Some(&ty) = self.places.types.get(&parent) {
                        let field_ty = match self.compiler.ty.types.kind(ty) {
                            TypeKind::Struct(st) => self
                                .compiler
                                .ty
                                .types
                                .struct_info(*st)
                                .fields
                                .iter()
                                .find(|f| f.name == Some(*name))
                                .map(|f| f.ty),
                            _ => None,
                        };
                        if let Some(ty) = field_ty {
                            self.places.types.insert(p, ty);
                        }
                        if name.as_str() == "data"
                            && matches!(
                                self.compiler.ty.types.kind(ty),
                                TypeKind::Array {
                                    kind: ArrayKind::Resizable | ArrayKind::View,
                                    ..
                                } | TypeKind::String
                            )
                        {
                            self.places.data.insert(p, (parent, Span::NONE));
                        }
                    }
                }
                Step::Element(index) => {
                    owner = Some(p);
                    p = self.places.project(p, Projection::Element(*index));
                }
            }
        }
        owner.or(Some(p))
    }
}

struct Writes {
    effects: Vec<(Effect, Vec<Span>)>,
    unknown: HashSet<usize>,
    escapes: HashSet<usize>,
}

fn insert(map: &mut HashMap<Effect, Vec<Span>>, effect: Effect, chain: Vec<Span>) -> bool {
    // Unknown reaches everything below its target; a parent write already forgets all
    // descendant places. Retain an antichain instead of enumerating redundant recursive
    // field paths. Allocation-specific resize/free effects deliberately do not use this.
    let covers = |a: &Effect, b: &Effect| {
        a.kind == b.kind
            && (a.guard.is_none() || a.guard == b.guard)
            && a.target.root == b.target.root
            && b.target.steps.starts_with(&a.target.steps)
    };
    if matches!(effect.kind, Kind::Unknown | Kind::Write) {
        if map.keys().any(|old| *old != effect && covers(old, &effect)) {
            return false;
        }
        map.retain(|old, _| *old == effect || !covers(&effect, old));
    }
    let prefer = map.get(&effect).is_none_or(|old| chain.len() < old.len());
    if prefer {
        map.insert(effect, chain);
    }
    prefer
}

fn chain(site: Span, rest: &[Span]) -> Vec<Span> {
    let mut result = vec![site];
    result.extend(rest.iter().copied().filter(|span| *span != site));
    if result.len() > MAX_CHAIN {
        // Preserve both ends, truncating only the middle of a long inferred chain.
        let tail = result.split_off(result.len() - 3);
        result.truncate(3);
        result.push(Span::NONE);
        result.extend(tail);
    }
    result
}

impl Effect {
    fn exported(&self) -> bool {
        matches!(self.target.root, Root::Parameter(_))
            || (self.kind == Kind::ResetTemporary && self.target.root == Root::Temporary)
            || (self.kind == Kind::Advance && matches!(self.target.root, Root::Until(_)))
    }
}

impl Summaries {
    #[cfg(test)]
    pub fn build(compiler: &Compiler) -> Self {
        Self::build_for(compiler, &HashSet::default())
    }

    pub fn build_for(compiler: &Compiler, files: &HashSet<jaic::source::FileId>) -> Self {
        let started = jaic::platform::Stopwatch::start();
        let Some(facts) = compiler.ed.facts() else {
            return Self::default();
        };
        let mut valid: HashMap<_, _> = facts
            .lifetime_bodies
            .iter()
            .filter(|(id, body)| {
                let proc = &compiler.pr.procs[id.0 as usize];
                proc.body_state == BodyState::Done
                    && std::sync::Arc::ptr_eq(&proc.lit, &body.syntax)
            })
            .map(|(&id, body)| (id, body))
            .collect();
        let allocators = allocators::Allocators::build(compiler, &valid);
        let needed = selected::retain(compiler, &mut valid, files);
        let mut all = contracts::seed(compiler, &valid, needed.as_ref());
        all.call_sites = valid
            .values()
            .flat_map(|body| &body.blocks)
            .flat_map(|block| &block.events)
            .filter_map(|event| match event {
                Event::Call {
                    span, ..
                }
                | Event::IndirectCall {
                    span, ..
                } => Some(*span),
                _ => None,
            })
            .collect();
        all.allocators = allocators;
        all.targets = super::lifetime_targets::Universe::build(compiler, &valid);
        (all.compile_time_only, all.frozen_returns) =
            execution::compile_time_only(compiler, &all.targets);
        // Tarjan's pop order is bottom-up for caller -> callee edges.
        let mut ids: Vec<_> = valid.keys().copied().collect();
        ids.sort_by_key(|id| id.0);
        let graph: HashMap<_, Vec<_>> = ids
            .iter()
            .map(|&id| {
                (
                    id,
                    valid[&id]
                        .blocks
                        .iter()
                        .flat_map(|b| &b.events)
                        .flat_map(|e| indirect::edges(e, id, &all.targets))
                        .filter(|p| valid.contains_key(p))
                        .collect(),
                )
            })
            .collect();
        let mut cache = prepared::Cache::default();
        let components = sccs(&ids, &graph);
        all.sccs = components.len();
        for component in &components {
            if jaic::interrupt::should_stop() {
                return Self::default();
            }
            let once = acyclic(component, &graph);
            if jacket_config::env::LIFETIME_STATS.is_set()
                && (component.len() > 1 || graph[&component[0]].contains(&component[0]))
            {
                eprintln!(
                    "lifetime-summary-scc {:?}",
                    component
                        .iter()
                        .map(|p| (p.0, compiler.pr.procs[p.0 as usize].name.as_str()))
                        .collect::<Vec<_>>()
                );
            }
            // Least fixed point for normal-return reachability, before guarantees. A
            // recursive cycle with no returning base must not make later caller uses live.
            loop {
                let mut changed = false;
                for id in component {
                    if jaic::interrupt::should_stop() {
                        return Self::default();
                    }
                    if all.procs[id].normal_return == Some(false)
                        && !returning_blocks(valid[id], &all.procs, &all.targets).is_empty()
                    {
                        all.procs.get_mut(id).unwrap().normal_return = Some(true);
                        changed = true;
                    }
                }
                if !changed || once {
                    break;
                }
            }
            // Body definitions are fixed once normal-return reachability settles.
            let mut resolvers = HashMap::default();
            loop {
                let mut changed = false;
                for id in component {
                    if jaic::interrupt::should_stop() {
                        return Self::default();
                    }
                    all.iterations += 1;
                    let resolver = resolvers.entry(*id).or_insert_with(|| {
                        Resolver::cached(
                            &mut cache,
                            compiler,
                            *id,
                            valid[id],
                            &all.procs,
                            &all.targets,
                            &all.allocators,
                        )
                    });
                    let next = resolver.infer(&all.procs[id], &all.procs);
                    if next != all.procs[id] {
                        all.procs.insert(*id, next);
                        changed = true;
                    }
                }
                if !changed || once {
                    break;
                }
            }
        }
        if jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!("lifetime-summary-writes-begin bodies={}", valid.len());
        }
        all.propagate_writes(&mut cache, compiler, &valid, &components, &graph);
        if jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!("lifetime-summary-writes-end");
        }
        reads::foreground::solve(&mut cache, compiler, &valid, &graph, &mut all);
        reads::solve_queries(&mut cache, compiler, &valid, &graph, &mut all);
        let read_summaries = reads::solve(&mut cache, compiler, &valid, &graph, &mut all);
        if jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!(
                "lifetime-summary-stats bodies={} sccs={} iterations={} may={} guaranteed={} read_summaries={} wall_ms={:.3}",
                valid.len(),
                all.sccs,
                all.iterations,
                all.procs.values().map(|s| s.may.len()).sum::<usize>(),
                all.procs
                    .values()
                    .map(|s| s.guaranteed.len())
                    .sum::<usize>(),
                read_summaries,
                started.elapsed().as_secs_f64() * 1000.0
            );
        }
        all
    }
}
