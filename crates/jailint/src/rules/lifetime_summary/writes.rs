//! Conservative write and retention effects, solved after return provenance.
use super::{
    Effect, Event, HashMap, HashSet, Kind, LifetimeArgument, Path, ProcId, Resolver, Root, Step,
    Summary, Writes, chain, returning_blocks, value,
};

impl Resolver<'_> {
    fn argument_dependencies(
        &self,
        formals: &HashSet<usize>,
        args: &[LifetimeArgument],
        all: &HashMap<ProcId, Summary>,
    ) -> HashSet<usize> {
        let mut out = HashSet::default();
        for formal in formals {
            let Some(arg) = args.iter().find(|a| a.formal == *formal) else {
                continue;
            };
            match self.argument_trace(arg, all, &mut HashSet::default(), 0) {
                Some(Path {
                    root: Root::Parameter(formal),
                    ..
                }) => {
                    out.insert(formal);
                }
                None => {
                    if let Some(value) = value(arg) {
                        out.extend(self.potential(value));
                    }
                }
                _ => {}
            }
        }
        out
    }

    pub(super) fn infer_writes(&mut self, all: &HashMap<ProcId, Summary>) -> Writes {
        let body = self.body;
        let mut writes = Vec::new();
        let mut unknown_write = HashSet::default();
        let mut escapes = HashSet::default();
        let live = returning_blocks(body, all, self.targets);
        for event in body
            .blocks
            .iter()
            .enumerate()
            .filter(|(i, _)| live.contains(*i))
            .flat_map(|(_, b)| &b.events)
        {
            match event {
                Event::Initialize {
                    place,
                    ty,
                    span,
                } => {
                    let p = self.places.place(*place, *ty);
                    if let Some(target) = self.location(p, all, &mut HashSet::default(), 0)
                        && matches!(target.root, Root::Parameter(_))
                    {
                        writes.push((
                            Effect {
                                target,
                                kind: Kind::Write,
                                guard: None,
                            },
                            vec![*span],
                        ));
                    } else if self.places.unresolved(p) {
                        unknown_write.extend(self.potential(self.places.paths[p].0.0));
                    }
                }
                Event::Store {
                    place,
                    value,
                    ty,
                    span,
                    ..
                } if self.compiler.ty.types.is_pointer(*ty)
                    || self.compiler.ir_ty(*ty).is_none() =>
                {
                    let p = self.places.place(*place, *ty);
                    let resolved = self.location(p, all, &mut HashSet::default(), 0);
                    if self.places.unresolved(p)
                        && resolved
                            .as_ref()
                            .is_none_or(|p| !matches!(p.root, Root::Parameter(_)))
                    {
                        unknown_write.extend(self.potential(self.places.paths[p].0.0));
                    }
                    let owned =
                        !self.places.unresolved(p) && self.owned.contains(&self.places.root_of(p));
                    if let Some(Path {
                        root: Root::Parameter(formal),
                        ..
                    }) = self.value(*value, all)
                        && !owned
                        && resolved.as_ref().is_none_or(|target| match target.root {
                            Root::Local(root) => !self.owned.contains(&root),
                            Root::Parameter(_)
                            | Root::ScopedGlobal(..)
                            | Root::CallerAllocator
                            | Root::AllocatorParameter(_)
                            | Root::Heap
                            | Root::Temporary
                            | Root::Watermark
                            | Root::Until(_) => true,
                        })
                    {
                        let restored = resolved.as_ref().and_then(|target| {
                            if self.compiler.ty.types.is_pointer(*ty) {
                                target.clone().append(&[Step::Deref])
                            } else {
                                Some(target.clone())
                            }
                        });
                        if self.value(*value, all) != restored {
                            escapes.insert(formal);
                        }
                    }
                    if let Some(target) = resolved
                        && matches!(target.root, Root::Parameter(_))
                    {
                        let expected = if self.compiler.ty.types.is_pointer(*ty) {
                            target.clone().append(&[Step::Deref])
                        } else {
                            Some(target.clone())
                        };
                        if self.value(*value, all) != expected {
                            writes.push((
                                Effect {
                                    target,
                                    kind: Kind::Write,
                                    guard: None,
                                },
                                vec![*span],
                            ));
                        }
                    }
                }
                Event::Call {
                    args,
                    span,
                    expands: false,
                    ..
                }
                | Event::IndirectCall {
                    args,
                    span,
                    allocator: false,
                    ..
                } => {
                    for callee in self.call_targets(event).procedures {
                        if let Some(called) = all.get(&callee) {
                            let query = super::reads::address_query(self.compiler, callee, args);
                            let queried;
                            let accessed = if query {
                                queried = called
                                    .unknown_write
                                    .iter()
                                    .copied()
                                    .filter(|n| *n != 3)
                                    .collect();
                                &queried
                            } else {
                                &called.unknown_write
                            };
                            unknown_write.extend(self.argument_dependencies(accessed, args, all));
                            escapes.extend(self.argument_dependencies(&called.escapes, args, all));
                        }
                    }
                    for (effect, witness) in self.call_effects(event, all, false) {
                        if effect.kind == Kind::Write && effect.exported() {
                            writes.push((effect, chain(*span, &witness)));
                        }
                    }
                }
                _ => {}
            }
        }
        Writes {
            effects: writes,
            unknown: unknown_write,
            escapes,
        }
    }
}

impl super::Summaries {
    pub(super) fn propagate_writes<'a>(
        &mut self,
        cache: &mut super::prepared::Cache<'a>,
        compiler: &'a super::Compiler,
        valid: &HashMap<ProcId, &'a super::LifetimeBody>,
        components: &[Vec<ProcId>],
        graph: &HashMap<ProcId, Vec<ProcId>>,
    ) {
        // Returns are stable. Solve callees before callers, iterating only recursive SCCs.
        for component in components {
            let recursive = component.len() > 1 || graph[&component[0]].contains(&component[0]);
            let mut resolvers = HashMap::default();
            loop {
                let mut changed = false;
                for id in component {
                    let resolver = resolvers.entry(*id).or_insert_with(|| {
                        Resolver::cached(
                            cache,
                            compiler,
                            *id,
                            valid[id],
                            &self.procs,
                            &self.targets,
                            &self.allocators,
                        )
                    });
                    let Writes {
                        effects: writes,
                        unknown: unknown_write,
                        escapes,
                    } = resolver.infer_writes(&self.procs);
                    let summary = self.procs.get_mut(id).unwrap();
                    let previous = summary.unknown_write.len() + summary.escapes.len();
                    summary.unknown_write.extend(unknown_write);
                    summary.escapes.extend(escapes);
                    changed |= previous != summary.unknown_write.len() + summary.escapes.len();
                    let before = summary.may.len();
                    summary.may.retain(|e, _| !matches!(e.target.root, Root::Parameter(p) if e.kind == Kind::Write && summary.unknown_write.contains(&p)));
                    changed |= before != summary.may.len();
                    for (effect, witness) in writes {
                        if matches!(effect.target.root, Root::Parameter(p) if effect.kind == Kind::Write && summary.unknown_write.contains(&p))
                        {
                            continue;
                        }
                        changed |= super::insert(&mut summary.may, effect, witness);
                    }
                }
                if !changed || !recursive {
                    break;
                }
            }
        }
    }
}
