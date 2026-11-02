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

mod contracts;
mod elements;
mod flow;
mod indirect;
mod infer;
mod reads;
#[cfg(test)]
mod tests;
mod writes;
use flow::{guaranteed, returning_blocks, sccs};

const MAX_PATH: usize = 8;
const MAX_CHAIN: usize = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) enum Step {
    Field(jaic::intern::Sym),
    Deref,
    Storage,
    Element(i128),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum Root {
    Parameter(usize),
    Local(usize),
    Heap,
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
    pub reads: HashSet<Path>,
    pub may_reads: HashSet<Path>,
    pub unknown_write: HashSet<usize>,
    pub escapes: HashSet<usize>,
    pub normal_return: Option<bool>,
    pub may: HashMap<Effect, Vec<Span>>,
    pub guaranteed: HashMap<Effect, Vec<Span>>,
    pub returns: HashMap<usize, (Path, Option<(usize, u64)>)>,
    pub elements: HashMap<usize, (Path, Selector)>,
}

#[derive(Default)]
pub(super) struct Summaries {
    pub targets: super::lifetime_targets::Universe,
    pub procs: HashMap<ProcId, Summary>,
    pub sccs: usize,
    pub iterations: usize,
}

#[derive(Clone, Debug)]
enum Def {
    Load(usize, TypeId),
    Copy(Val),
    Address(usize),
    Borrow(usize),
    Parameter(usize),
    Call(ProcId, Vec<LifetimeArgument>, usize),
    Resize(Val, Vec<LifetimeArgument>),
    Indirect(Val, Vec<LifetimeArgument>, usize),
}

pub(super) struct Resolver<'a> {
    pub places: Places,
    compiler: &'a Compiler,
    body: &'a LifetimeBody,
    values: HashMap<Val, Vec<Def>>,
    writes: HashMap<usize, Vec<Val>>,
    params: HashMap<usize, (usize, TypeId)>,
    exposed: HashSet<usize>,
    owned: HashSet<usize>,
    targets: &'a super::lifetime_targets::Universe,
    indirect_types: HashMap<Val, TypeId>,
}


fn value(arg: &LifetimeArgument) -> Option<Val> {
    match arg.operand {
        LifetimeOperand::Value(v) | LifetimeOperand::Place(v) => Some(v),
        LifetimeOperand::Other => None,
    }
}

impl<'a> Resolver<'a> {
    pub fn new(
        compiler: &'a Compiler,
        body: &'a LifetimeBody,
        all: &HashMap<ProcId, Summary>,
        targets: &'a super::lifetime_targets::Universe,
    ) -> Self {
        let mut this = Self {
            places: Places::new(compiler, body),
            compiler,
            body,
            values: HashMap::default(),
            writes: HashMap::default(),
            params: HashMap::default(),
            exposed: HashSet::default(),
            owned: HashSet::default(),
            targets,
            indirect_types: body
                .blocks
                .iter()
                .flat_map(|b| &b.events)
                .filter_map(|e| match e {
                    Event::IndirectCall {
                        callee,
                        ty,
                        ..
                    } => Some((*callee, *ty)),
                    _ => None,
                })
                .collect(),
        };
        let live = returning_blocks(body, all, targets);
        for e in body
            .blocks
            .iter()
            .enumerate()
            .filter(|(i, _)| live.contains(i))
            .flat_map(|(_, b)| &b.events)
        {
            match e {
                Event::Expression {
                    operand: LifetimeOperand::Place(v),
                    ty,
                    entity: Some(id),
                    ..
                } if matches!(
                    compiler.entity(*id).kind,
                    jaic::sema::scope::EntityKind::Local { .. }
                ) =>
                {
                    let p = this.places.place(*v, *ty);
                    let (root, _) = &this.places.paths[p];
                    if let Some(root_id) = this
                        .places
                        .paths
                        .iter()
                        .position(|(r, path)| r == root && path.is_empty())
                    {
                        this.owned.insert(root_id);
                    }
                }
                Event::Parameter {
                    formal,
                    place,
                    incoming,
                    ty,
                    ..
                } => {
                    let p = this.places.place(*place, *ty);
                    this.params.insert(p, (*formal, *ty));
                    this.owned.insert(p);
                    this.values
                        .entry(*incoming)
                        .or_default()
                        .push(Def::Parameter(*formal));
                }
                Event::Load {
                    place,
                    value,
                    ty,
                    ..
                } => {
                    let p = this.places.place(*place, *ty);
                    if value == place && compiler.ir_ty(*ty).is_none() {
                        // An aggregate rvalue is its address. Preserve plain aggregate
                        // copies, but do not add a self-reference to a call's SSA result.
                        if !this.values.get(value).is_some_and(|defs| {
                            defs.iter()
                                .any(|d| matches!(d, Def::Call(..) | Def::Resize(..)))
                        }) {
                            this.values.entry(*value).or_default().push(Def::Address(p));
                        }
                        continue;
                    }
                    this.values
                        .entry(*value)
                        .or_default()
                        .push(Def::Load(p, *ty));
                }
                Event::Store {
                    place,
                    value,
                    ty,
                    ..
                } => {
                    let p = this.places.place(*place, *ty);
                    this.writes.entry(p).or_default().push(*value);
                    if compiler.ir_ty(*ty).is_none()
                        && place != value
                        && !this.values.contains_key(place)
                    {
                        this.values.insert(*place, vec![Def::Address(p)]);
                    }
                }
                Event::Copy {
                    value,
                    source,
                }
                | Event::Offset {
                    value,
                    source,
                    ..
                } if value != source => this
                    .values
                    .entry(*value)
                    .or_default()
                    .push(Def::Copy(*source)),
                Event::Address {
                    place,
                    ty,
                    ..
                } => {
                    let p = this.places.place(*place, *ty);
                    this.values.entry(*place).or_default().push(Def::Address(p));
                    this.exposed.insert(p);
                }
                Event::View {
                    value,
                    source,
                    from,
                    to,
                    ..
                } => {
                    let p = this.places.place(*source, *from);
                    let target = this.places.place(*value, *to);
                    if target != p {
                        this.values.entry(*value).or_default().push(Def::Borrow(p));
                    }
                }
                Event::Borrow {
                    value,
                    region,
                    region_ty,
                    ..
                } => {
                    let p = this.places.place(*region, *region_ty);
                    this.values.entry(*value).or_default().push(Def::Borrow(p));
                }
                Event::Index {
                    base,
                    result,
                    base_ty,
                    ..
                } => {
                    let p = this.places.place(*base, *base_ty);
                    this.values.entry(*result).or_default().push(Def::Borrow(p));
                }
                Event::Call {
                    callee,
                    args,
                    results,
                    ..
                } => {
                    for &(i, v, _) in results {
                        this.values
                            .entry(v)
                            .or_default()
                            .push(Def::Call(*callee, args.clone(), i));
                    }
                }
                Event::IndirectCall {
                    allocator: true,
                    callee,
                    args,
                    results,
                    ..
                } => {
                    for &(_, v, _) in results {
                        this.values
                            .entry(v)
                            .or_default()
                            .push(Def::Resize(*callee, args.clone()));
                    }
                }
                Event::IndirectCall {
                    callee,
                    args,
                    results,
                    allocator: false,
                    ..
                } => {
                    for &(index, v, _) in results {
                        this.values.entry(v).or_default().push(Def::Indirect(
                            *callee,
                            args.clone(),
                            index,
                        ));
                    }
                }
                _ => {}
            }
        }
        for (id, ((value, _), path)) in this.places.paths.iter().enumerate() {
            if path.is_empty() && body.stack_places.contains(value) {
                this.owned.insert(id);
            }
        }
        this
    }

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
            if let Some(id) = r
                .places
                .paths
                .iter()
                .position(|(base, path)| base == root && path.is_empty())
            {
                if let Some((formal, _)) = r.params.get(&id) {
                    out.insert(*formal);
                }
                for &value in r.writes.get(&id).into_iter().flatten() {
                    walk(r, value, seen, out);
                }
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
                        for arg in args {
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

    fn return_guard(
        &self,
        v: Val,
        all: &HashMap<ProcId, Summary>,
        visiting: &mut HashSet<Val>,
    ) -> Option<Option<(usize, u64)>> {
        if !visiting.insert(v) {
            return None;
        }
        let mut guard = None;
        for def in self.values.get(&v)? {
            let next = match def {
                Def::Load(p, _) => {
                    if !self.places.paths[*p].1.is_empty() {
                        None
                    } else if let Some(writes) = self.writes.get(p) {
                        let [value] = writes.as_slice() else {
                            return None;
                        };
                        self.return_guard(*value, all, visiting)?
                    } else {
                        None
                    }
                }
                Def::Copy(value) => self.return_guard(*value, all, visiting)?,
                Def::Resize(_, args) => {
                    if args.first()?.constant == Some(0) {
                        None
                    } else {
                        self.mode_guard(args.first()?, 1, all)?
                    }
                }
                Def::Call(callee, args, index) => {
                    let (_, condition) = all.get(callee)?.returns.get(index)?;
                    if let Some((formal, expected)) = condition {
                        self.mode_guard(args.iter().find(|a| a.formal == *formal)?, *expected, all)?
                    } else {
                        None
                    }
                }
                Def::Indirect(callee, args, index) => {
                    self.indirect_return(*callee, args, *index, all, &mut HashSet::default(), 0)?;
                    None
                }
                Def::Address(_) | Def::Borrow(_) | Def::Parameter(_) => None,
            };
            if guard.is_some() && next != guard {
                return None;
            }
            guard = next;
        }
        visiting.remove(&v);
        Some(guard)
    }

    fn mode_guard(
        &self,
        mode: &LifetimeArgument,
        expected: u64,
        all: &HashMap<ProcId, Summary>,
    ) -> Option<Option<(usize, u64)>> {
        if let Some(actual) = mode.constant {
            return (actual == expected).then_some(None);
        }
        let path = self.trace(value(mode)?, all, &mut HashSet::default(), 0)?;
        let Root::Parameter(formal) = path.root else {
            return None;
        };
        path.steps.is_empty().then_some(Some((formal, expected)))
    }

    fn trace(
        &self,
        v: Val,
        summaries: &HashMap<ProcId, Summary>,
        visiting: &mut HashSet<Val>,
        depth: usize,
    ) -> Option<Path> {
        if depth > 24 || !visiting.insert(v) {
            return None;
        }
        let defs = self.values.get(&v)?;
        let mut result = None;
        let derived = defs.iter().any(|d| matches!(d, Def::Borrow(_)));
        for def in defs {
            if derived && !matches!(def, Def::Borrow(_)) {
                continue;
            }
            let path = match def {
                Def::Parameter(formal) => Path {
                    root: Root::Parameter(*formal),
                    steps: Vec::new(),
                },
                Def::Address(p) => self.location(*p, summaries, visiting, depth + 1)?,
                Def::Borrow(p) => self
                    .location(*p, summaries, visiting, depth + 1)?
                    .append(&[Step::Storage])?,
                Def::Copy(v) => self.trace(*v, summaries, visiting, depth + 1)?,
                Def::Load(p, ty) => {
                    if let Some((formal, _)) = self.params.get(p) {
                        if self.exposed.contains(p)
                            || self.writes.get(p).is_none_or(|w| w.len() != 1)
                        {
                            return None;
                        }
                        Path {
                            root: Root::Parameter(*formal),
                            steps: Vec::new(),
                        }
                    } else if !self.compiler.ty.types.is_pointer(*ty) {
                        self.location(*p, summaries, visiting, depth + 1)?
                    } else if !self.places.paths[*p].1.is_empty() {
                        self.location(*p, summaries, visiting, depth + 1)?
                            .append(&[Step::Deref])?
                    } else if let Some(writes) = self.writes.get(p) {
                        let [v] = writes.as_slice() else {
                            return None;
                        };
                        self.trace(*v, summaries, visiting, depth + 1)?
                    } else {
                        let path = self.location(*p, summaries, visiting, depth + 1)?;
                        if self.compiler.ty.types.is_pointer(*ty) {
                            path.append(&[Step::Deref])?
                        } else {
                            path
                        }
                    }
                }
                Def::Call(callee, args, i) => {
                    let (path, guard) = summaries.get(callee)?.returns.get(i)?;
                    if guard.is_some_and(|(formal, constant)| {
                        args.iter()
                            .find(|a| a.formal == formal)
                            .and_then(|a| a.constant)
                            .is_some_and(|c| c != constant)
                    }) {
                        return None;
                    }
                    self.substitute(path, args, summaries, visiting, depth + 1)?
                }
                Def::Indirect(callee, args, index) => {
                    self.indirect_return(*callee, args, *index, summaries, visiting, depth + 1)?
                }
                Def::Resize(callee, args) => {
                    let mode = args.first()?;
                    if mode.constant == Some(0) {
                        self.indirect_return(*callee, args, 0, summaries, visiting, depth + 1)
                            .unwrap_or(Path {
                                root: Root::Heap,
                                steps: Vec::new(),
                            })
                    } else {
                        if mode.constant.is_some_and(|c| c != 1) {
                            return None;
                        }
                        self.trace(value(args.get(3)?)?, summaries, visiting, depth + 1)?
                    }
                }
            };
            if result.as_ref().is_some_and(|old| old != &path) {
                return None;
            }
            result = Some(path);
        }
        visiting.remove(&v);
        result
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
        let root_id = self
            .places
            .paths
            .iter()
            .position(|(r, steps)| r == root && steps.is_empty())?;
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
            if self.compiler.ty.types.is_pointer(*ty) {
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
                && let Some([source]) = self.writes.get(&root_id).map(Vec::as_slice)
                && *source != root.0
                && let Some(from) = self.trace(*source, summaries, visiting, depth + 1)
                && (matches!(
                    from.root,
                    Root::Parameter(_)
                        | Root::Heap
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
                Root::Heap | Root::Temporary | Root::Watermark | Root::Until(_)
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
        let Some(summary) = all.get(&callee) else {
            return Vec::new();
        };
        let map = if guaranteed {
            &summary.guaranteed
        } else {
            &summary.may
        };
        map.iter()
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
                let target =
                    self.substitute(&effect.target, args, all, &mut HashSet::default(), 0)?;
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
            .collect()
    }

    fn allocator(&self, args: &[LifetimeArgument], all: &HashMap<ProcId, Summary>) -> Vec<Effect> {
        let Some(mode) = args.first() else {
            return Vec::new();
        };
        let Some(memory) = args.get(3).and_then(value).and_then(|v| self.value(v, all)) else {
            return Vec::new();
        };
        [(1, Kind::Resize), (2, Kind::Free)]
            .into_iter()
            .filter_map(|(constant, kind)| {
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
                Some(Effect {
                    target: memory.clone(),
                    kind,
                    guard,
                })
            })
            .collect()
    }

    pub fn materialize(&mut self, path: &Path) -> Option<usize> {
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
                Step::Storage => owner = Some(p),
                Step::Deref => {
                    owner = Some(self.places.data.get(&p).map_or(p, |(parent, _)| *parent));
                    p = self.places.project(p, Projection::Deref);
                }
                Step::Field(name) => {
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

fn insert(map: &mut HashMap<Effect, Vec<Span>>, effect: Effect, chain: Vec<Span>) {
    let prefer = map.get(&effect).is_none_or(|old| chain.len() < old.len());
    if prefer {
        map.insert(effect, chain);
    }
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
    fn propagate_writes(&mut self, compiler: &Compiler, valid: &HashMap<ProcId, &LifetimeBody>) {
        // Returns are stable before writes are classified. A RESIZE result stored back
        // into its own field preserves the owner; an unrelated replacement must forget it.
        loop {
            let mut changed = false;
            for (&id, body) in valid {
                let mut resolver = Resolver::new(compiler, body, &self.procs, &self.targets);
                let Writes {
                    effects: writes,
                    unknown: unknown_write,
                    escapes,
                } = resolver.infer_writes(&self.procs);
                let summary = self.procs.get_mut(&id).unwrap();
                let previous = summary.unknown_write.len() + summary.escapes.len();
                summary.unknown_write.extend(unknown_write);
                summary.escapes.extend(escapes);
                changed |= previous != summary.unknown_write.len() + summary.escapes.len();
                for (effect, witness) in writes {
                    if let std::collections::hash_map::Entry::Vacant(entry) =
                        summary.may.entry(effect)
                    {
                        entry.insert(witness);
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
    }

    pub fn build(compiler: &Compiler) -> Self {
        let Some(facts) = compiler.ed.facts() else {
            return Self::default();
        };
        let valid: HashMap<_, _> = facts
            .lifetime_bodies
            .iter()
            .filter(|(id, body)| {
                let proc = &compiler.pr.procs[id.0 as usize];
                proc.body_state == BodyState::Done
                    && std::sync::Arc::ptr_eq(&proc.lit, &body.syntax)
            })
            .map(|(&id, body)| (id, body))
            .collect();
        let mut all = contracts::seed(compiler, &valid);
        all.targets = super::lifetime_targets::Universe::build(compiler, &valid);
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
        let components = sccs(&ids, &graph);
        all.sccs = components.len();
        for component in components {
            // Least fixed point for normal-return reachability, before guarantees. A
            // recursive cycle with no returning base must not make later caller uses live.
            loop {
                let mut changed = false;
                for id in &component {
                    if all.procs[id].normal_return == Some(false)
                        && !returning_blocks(valid[id], &all.procs, &all.targets).is_empty()
                    {
                        all.procs.get_mut(id).unwrap().normal_return = Some(true);
                        changed = true;
                    }
                }
                if !changed {
                    break;
                }
            }
            loop {
                let mut changed = false;
                for id in &component {
                    all.iterations += 1;
                    let body = valid[id];
                    let resolver = Resolver::new(compiler, body, &all.procs, &all.targets);
                    let next = resolver.infer(&all.procs[id], &all.procs);
                    if next != all.procs[id] {
                        all.procs.insert(*id, next);
                        changed = true;
                    }
                }
                if !changed {
                    break;
                }
            }
        }
        all.propagate_writes(compiler, &valid);
        if jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!(
                "lifetime-summary-stats bodies={} sccs={} iterations={} may={} guaranteed={}",
                valid.len(),
                all.sccs,
                all.iterations,
                all.procs.values().map(|s| s.may.len()).sum::<usize>(),
                all.procs
                    .values()
                    .map(|s| s.guaranteed.len())
                    .sum::<usize>()
            );
        }
        all
    }
}
