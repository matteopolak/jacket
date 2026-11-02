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
        let live = returning_blocks(body, all);
        for event in body
            .blocks
            .iter()
            .enumerate()
            .filter(|(i, _)| live.contains(i))
            .flat_map(|(_, b)| &b.events)
        {
            match event {
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
                    let owned = !self.places.unresolved(p)
                        && self
                            .places
                            .paths
                            .iter()
                            .position(|(root, path)| {
                                root == &self.places.paths[p].0 && path.is_empty()
                            })
                            .is_some_and(|root| self.owned.contains(&root));
                    if let Some(Path {
                        root: Root::Parameter(formal),
                        ..
                    }) = self.value(*value, all)
                        && !owned
                        && resolved.as_ref().is_none_or(|target| match target.root {
                            Root::Local(root) => !self.owned.contains(&root),
                            Root::Parameter(_) | Root::Heap | Root::Temporary | Root::Watermark => {
                                true
                            }
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
                    callee,
                    args,
                    span,
                    expands: false,
                    ..
                } => {
                    if let Some(called) = all.get(callee) {
                        unknown_write.extend(self.argument_dependencies(
                            &called.unknown_write,
                            args,
                            all,
                        ));
                        escapes.extend(self.argument_dependencies(&called.escapes, args, all));
                    }
                    for (effect, witness) in self.effects(*callee, args, all, false) {
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
