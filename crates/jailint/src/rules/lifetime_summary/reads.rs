//! Parameter-relative memory reads used to bound an asynchronous callback's borrow.
use super::{Event, HashMap, HashSet, Path, ProcId, Resolver, Root, Summary};

impl Resolver<'_> {
    pub(super) fn reads(&self, all: &HashMap<ProcId, Summary>, guaranteed: bool) -> HashSet<Path> {
        let mut effects = Vec::new();
        for block in &self.body.blocks {
            let mut paths = HashSet::default();
            for event in &block.events {
                match event {
                    Event::ReadMemory {
                        value, ..
                    } => {
                        if let Some(path) = self.value(*value, all)
                            && matches!(path.root, Root::Parameter(_))
                        {
                            paths.insert(path);
                        }
                    }
                    Event::Call {
                        args,
                        expands: false,
                        ..
                    }
                    | Event::IndirectCall {
                        args, ..
                    } => {
                        paths.extend(self.callee_reads(event, args, all, guaranteed));
                    }
                    _ => {}
                }
            }
            effects.push(
                paths
                    .into_iter()
                    .map(|target| {
                        (
                            super::Effect {
                                target,
                                kind: super::Kind::Unknown,
                                guard: None,
                            },
                            Vec::new(),
                        )
                    })
                    .collect::<Vec<_>>(),
            );
        }
        let candidates: HashSet<_> = effects.iter().flatten().map(|(e, _)| e.clone()).collect();
        candidates
            .into_iter()
            .filter(|e| !guaranteed || super::guaranteed(self.body, &effects, e).is_some())
            .map(|e| e.target)
            .collect()
    }

    fn callee_reads(
        &self,
        event: &Event,
        args: &[super::LifetimeArgument],
        all: &HashMap<ProcId, Summary>,
        guaranteed: bool,
    ) -> HashSet<Path> {
        let targets = self.call_targets(event);
        if guaranteed && targets.unknown {
            return HashSet::default();
        }
        let mut paths = HashSet::default();
        for (i, proc) in targets.ordered().iter().enumerate() {
            let current: HashSet<_> = all
                .get(proc)
                .into_iter()
                .flat_map(|s| {
                    if guaranteed {
                        &s.reads
                    } else {
                        &s.may_reads
                    }
                })
                .filter_map(|p| self.substitute(p, args, all, &mut HashSet::default(), 0))
                .filter(|p| matches!(p.root, Root::Parameter(_)))
                .collect();
            if !guaranteed {
                paths.extend(current);
            } else if i == 0 {
                paths = current;
            } else {
                paths.retain(|p| current.contains(p));
            }
        }
        paths
    }

    /// An object's address, as opposed to the allocation that owns it. Tokens for two
    /// elements of one container must not collapse into the container's allocation token.
    pub(in crate::rules) fn object(&mut self, arg: &super::LifetimeArgument) -> Option<usize> {
        let target = self.compiler.ty.types.pointee(arg.ty)?;
        let value = super::value(arg)?;
        for def in self.values.get(&value).into_iter().flatten() {
            if let super::Def::Address(p) = def
                && self.places.types.get(p) == Some(&target)
            {
                return Some(*p);
            }
        }
        if let Some(p) = self.stored_address(value, &mut HashSet::default())
            && self.places.types.get(&p) == Some(&target)
        {
            return Some(p);
        }
        let [super::Def::Load(slot, ty)] = self.values.get(&value)?.as_slice() else {
            return None;
        };
        if self.compiler.ty.types.pointee(*ty) != Some(target)
            || self.writes.get(slot).is_none_or(|w| w.len() != 1)
        {
            return None;
        }
        let p = self.places.project(*slot, super::Projection::Deref);
        self.places.types.insert(p, target);
        Some(p)
    }
}


/// Read summaries have only one consumer: captures of a registered asynchronous callback.
/// Solve its full callee closure, without paying for unrelated procedures in every root.
fn needed(
    compiler: &super::Compiler,
    valid: &HashMap<ProcId, &super::LifetimeBody>,
    all: &super::Summaries,
    graph: &HashMap<ProcId, Vec<ProcId>>,
) -> HashSet<ProcId> {
    let mut queue = std::collections::VecDeque::new();
    for (&proc, body) in valid {
        for event in body.blocks.iter().flat_map(|b| &b.events) {
            let Event::Call {
                callee,
                args,
                ..
            } = event
            else {
                continue;
            };
            let callback = super::lifetime_movement::notes(compiler, *callee).find_map(|note| {
                super::lifetime_movement::parameters(&note.text, "RegistersBorrower")?
                    .get(1)?
                    .parse::<usize>()
                    .ok()
            });
            if let Some(callback) = callback
                && let Some(v) = args
                    .iter()
                    .find(|a| a.formal == callback)
                    .and_then(super::value)
            {
                queue.extend(all.targets.get(proc, v).procedures.into_iter().filter(|p| {
                    valid.contains_key(p)
                        && all.procs.get(p).is_some_and(|s| {
                            s.unknown_write.is_empty()
                                && s.escapes.is_empty()
                                && !s.may.keys().any(|e| {
                                    matches!(
                                        e.kind,
                                        super::Kind::Unknown
                                            | super::Kind::Free
                                            | super::Kind::Write
                                    )
                                })
                        })
                }));
            }
        }
    }
    let mut needed = HashSet::default();
    while let Some(proc) = queue.pop_front() {
        if needed.insert(proc) {
            queue.extend(graph.get(&proc).into_iter().flatten());
        }
    }
    needed
}


pub(super) fn solve(
    compiler: &super::Compiler,
    valid: &HashMap<ProcId, &super::LifetimeBody>,
    graph: &HashMap<ProcId, Vec<ProcId>>,
    all: &mut super::Summaries,
) -> usize {
    let needed = needed(compiler, valid, all, graph);
    let mut ids: Vec<_> = needed
        .iter()
        .filter(|p| valid.contains_key(p))
        .copied()
        .collect();
    ids.sort_unstable_by_key(|p| p.0);
    for component in super::sccs(&ids, graph) {
        loop {
            let mut changed = false;
            for id in &component {
                let resolver = Resolver::new(compiler, valid[id], &all.procs, &all.targets);
                let guaranteed = resolver.reads(&all.procs, true);
                let may = resolver.reads(&all.procs, false);
                let summary = all.procs.get_mut(id).unwrap();
                let before = summary.reads.len() + summary.may_reads.len();
                summary.reads.extend(guaranteed);
                summary.may_reads.extend(may);
                changed |= summary.reads.len() + summary.may_reads.len() != before;
            }
            if !changed {
                break;
            }
        }
    }
    ids.len()
}
