//! Ordinary calls need only the finite set of arguments whose pointees are read.
//! Detailed capture paths are solved separately for asynchronous callbacks.
use super::{Event, HashMap, HashSet, Path, ProcId, Resolver, Root, Summary};
use crate::rules::lifetime_summary::{Effect, Kind, Summaries, acyclic, guaranteed, sccs};
use jaic::ide::lifetimes::LifetimeOperand;
use jaic::intern::Sym;
use jaic::sema::Compiler;

/// The member names at the start of `steps`, and whether `steps` ends there.
fn leading_members(steps: &[super::super::Step]) -> (Vec<Sym>, bool) {
    let chain: Vec<Sym> = steps
        .iter()
        .map_while(|step| match step {
            super::super::Step::Field(name) => Some(*name),
            _ => None,
        })
        .collect();
    let complete = chain.len() == steps.len();
    (chain, complete)
}

impl Resolver<'_> {
    /// The parameter an event reads through, and the member chain it reads.
    fn foreground_direct(
        &self,
        event: &Event,
        all: &HashMap<ProcId, Summary>,
    ) -> Option<(usize, Vec<Sym>)> {
        let (path, member) = match event {
            Event::ReadMemory {
                value, ..
            } => (self.value(*value, all), None),
            Event::Field {
                base: LifetimeOperand::Value(value),
                indirect: true,
                name,
                ..
            } => (self.value(*value, all), Some(*name)),
            Event::Field {
                base: LifetimeOperand::Place(value),
                base_ty,
                indirect: true,
                name,
                ..
            } => {
                let p = self.places.existing(*value, *base_ty)?;
                if let Some((formal, _)) = self.params.get(&p) {
                    (
                        Some(Path {
                            root: Root::Parameter(*formal),
                            steps: Vec::new(),
                        }),
                        Some(*name),
                    )
                } else {
                    let [value] = self.writes.get(&p)?.as_slice() else {
                        return None;
                    };
                    (self.value(*value, all), Some(*name))
                }
            }
            _ => return None,
        };
        let path = path?;
        let Root::Parameter(formal) = path.root else {
            return None;
        };
        let (mut chain, complete) = leading_members(&path.steps);
        if complete && let Some(name) = member {
            chain.push(name);
        }
        Some((formal, chain))
    }

    /// The member chains a call may read through `arg`, from what its targets read; `None` when
    /// any target is unknown or reads the whole pointee.
    pub(in crate::rules) fn read_members(
        &self,
        event: &Event,
        arg: &super::super::LifetimeArgument,
        all: &HashMap<ProcId, Summary>,
    ) -> Option<Vec<Vec<Sym>>> {
        let targets = self.call_targets(event);
        if targets.unknown || targets.procedures.is_empty() {
            return None;
        }
        let mut chains = Vec::new();
        for proc in &targets.procedures {
            let known = all.get(proc)?.read_members.get(&arg.formal)?;
            chains.extend(known.iter().cloned());
        }
        (!chains.iter().any(Vec::is_empty)).then_some(chains)
    }

    /// What a body reads through its parameters, per block: the parameter and member chain.
    fn foreground_facts(
        &self,
        all: &HashMap<ProcId, Summary>,
        must: bool,
    ) -> Vec<Vec<(usize, Vec<Sym>)>> {
        self.body
            .blocks
            .iter()
            .map(|block| {
                let mut facts = Vec::new();
                for event in &block.events {
                    facts.extend(self.foreground_direct(event, all));
                    for arg in self.read_arguments(event, all, must) {
                        let Some(Path {
                            root: Root::Parameter(formal),
                            steps,
                        }) = self.argument_trace(arg, all, &mut HashSet::default(), 0)
                        else {
                            continue;
                        };
                        let (prefix, complete) = leading_members(&steps);
                        match self.read_members(event, arg, all).filter(|_| complete) {
                            Some(chains) => facts.extend(chains.into_iter().map(|chain| {
                                (formal, prefix.iter().copied().chain(chain).collect())
                            })),
                            None => facts.push((formal, prefix)),
                        }
                    }
                }
                facts
            })
            .collect()
    }

    fn foreground_reads(&self, all: &HashMap<ProcId, Summary>, must: bool) -> HashSet<usize> {
        let effects: Vec<_> = self
            .foreground_facts(all, must)
            .into_iter()
            .map(|facts| {
                facts
                    .into_iter()
                    .map(|(formal, _)| formal)
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .map(|formal| {
                        (
                            Effect {
                                target: Path {
                                    root: Root::Parameter(formal),
                                    steps: Vec::new(),
                                },
                                kind: Kind::Unknown,
                                guard: None,
                            },
                            Vec::new(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        let candidates: HashSet<_> = effects.iter().flatten().map(|(e, _)| e.clone()).collect();
        candidates
            .into_iter()
            .filter(|e| !must || guaranteed(self.body, &effects, e).is_some())
            .filter_map(|e| match e.target.root {
                Root::Parameter(n) => Some(n),
                _ => None,
            })
            .collect()
    }

    /// Every member chain this body may read through each parameter.
    fn foreground_members(
        &self,
        all: &HashMap<ProcId, Summary>,
    ) -> HashMap<usize, HashSet<Vec<Sym>>> {
        let mut members = HashMap::<usize, HashSet<Vec<Sym>>>::default();
        for (formal, chain) in self.foreground_facts(all, false).into_iter().flatten() {
            members.entry(formal).or_default().insert(chain);
        }
        members
    }
}

pub(in crate::rules) fn solve<'a>(
    cache: &mut super::super::prepared::Cache<'a>,
    compiler: &'a Compiler,
    valid: &HashMap<ProcId, &'a jaic::ide::lifetimes::LifetimeBody>,
    graph: &HashMap<ProcId, Vec<ProcId>>,
    all: &mut Summaries,
) {
    let mut ids: Vec<_> = valid.keys().copied().collect();
    ids.sort_unstable_by_key(|p| p.0);
    for component in sccs(&ids, graph) {
        let once = acyclic(&component, graph);
        let mut resolvers = HashMap::default();
        loop {
            let mut changed = false;
            for id in &component {
                let resolver = resolvers.entry(*id).or_insert_with(|| {
                    Resolver::cached(
                        cache,
                        compiler,
                        *id,
                        valid[id],
                        &all.procs,
                        &all.targets,
                        &all.allocators,
                    )
                });
                let must = resolver.foreground_reads(&all.procs, true);
                let may = resolver.foreground_reads(&all.procs, false);
                let members = resolver.foreground_members(&all.procs);
                let summary = all.procs.get_mut(id).unwrap();
                let size = |s: &Summary| {
                    s.read_formals.len()
                        + s.may_read_formals.len()
                        + s.read_members.values().map(HashSet::len).sum::<usize>()
                };
                let before = size(summary);
                summary.read_formals.extend(must);
                summary.may_read_formals.extend(may);
                for (formal, chains) in members {
                    summary
                        .read_members
                        .entry(formal)
                        .or_default()
                        .extend(chains);
                }
                changed |= before != size(summary);
            }
            if !changed || once {
                break;
            }
        }
    }
}
