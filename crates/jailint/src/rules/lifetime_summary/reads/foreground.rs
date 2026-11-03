//! Ordinary calls need only the finite set of arguments whose pointees are read.
//! Detailed capture paths are solved separately for asynchronous callbacks.
use super::{Event, HashMap, HashSet, Path, ProcId, Resolver, Root, Summary};
use crate::rules::lifetime_summary::{Effect, Kind, Summaries, acyclic, guaranteed, sccs};
use jaic::ide::lifetimes::LifetimeOperand;
use jaic::sema::Compiler;

impl Resolver<'_> {
    fn foreground_direct(&self, event: &Event, all: &HashMap<ProcId, Summary>) -> Option<usize> {
        let path = match event {
            Event::ReadMemory {
                value, ..
            }
            | Event::Field {
                base: LifetimeOperand::Value(value),
                indirect: true,
                ..
            } => self.value(*value, all),
            Event::Field {
                base: LifetimeOperand::Place(value),
                base_ty,
                indirect: true,
                ..
            } => {
                let p = self.places.existing(*value, *base_ty)?;
                if let Some((formal, _)) = self.params.get(&p) {
                    Some(Path {
                        root: Root::Parameter(*formal),
                        steps: Vec::new(),
                    })
                } else {
                    let [value] = self.writes.get(&p)?.as_slice() else {
                        return None;
                    };
                    self.value(*value, all)
                }
            }
            _ => None,
        }?;
        let Root::Parameter(formal) = path.root else {
            return None;
        };
        Some(formal)
    }

    fn foreground_reads(&self, all: &HashMap<ProcId, Summary>, must: bool) -> HashSet<usize> {
        let effects: Vec<_> = self
            .body
            .blocks
            .iter()
            .map(|block| {
                let mut reads = HashSet::default();
                for event in &block.events {
                    if let Some(formal) = self.foreground_direct(event, all) {
                        reads.insert(formal);
                    }
                    for arg in self.read_arguments(event, all, must) {
                        if let Some(Path {
                            root: Root::Parameter(formal),
                            ..
                        }) = self.argument_trace(arg, all, &mut HashSet::default(), 0)
                        {
                            reads.insert(formal);
                        }
                    }
                }
                reads
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
}

pub(in crate::rules) fn solve(
    compiler: &Compiler,
    valid: &HashMap<ProcId, &jaic::ide::lifetimes::LifetimeBody>,
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
                    Resolver::new(
                        compiler,
                        valid[id],
                        &all.procs,
                        &all.targets,
                        &all.allocators,
                    )
                });
                let must = resolver.foreground_reads(&all.procs, true);
                let may = resolver.foreground_reads(&all.procs, false);
                let summary = all.procs.get_mut(id).unwrap();
                let before = summary.read_formals.len() + summary.may_read_formals.len();
                summary.read_formals.extend(must);
                summary.may_read_formals.extend(may);
                changed |= before != summary.read_formals.len() + summary.may_read_formals.len();
            }
            if !changed || once {
                break;
            }
        }
    }
}
