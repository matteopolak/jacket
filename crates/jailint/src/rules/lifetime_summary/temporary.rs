//! Stores through a caller-provided destination are checked in the caller's storage context.
use super::{
    Effect, Event, HashMap, HashSet, Kind, Path, ProcId, Resolver, Root, Span, Step, Summary, value,
};

impl Resolver<'_> {
    pub(in crate::rules) fn temporary_writes(
        &mut self,
        event: &Event,
        all: &HashMap<ProcId, Summary>,
    ) -> Vec<(Effect, Span, bool)> {
        let Some((target, source, ty, span)) = self.temporary_source(event, all) else {
            return Vec::new();
        };
        if !matches!(target.root, Root::Parameter(_)) {
            return Vec::new();
        }
        let definite = self
            .value(source, all)
            .is_some_and(|p| p.root == Root::Temporary);
        let mut paths = Vec::new();
        if definite {
            paths.push((target, true));
        } else if matches!(self.compiler.ty.types.kind(ty), super::TypeKind::String) {
            if self.string_may_temporary(source, ty, all) {
                paths.push((target, false));
            }
        } else {
            if self.compiler.ir_ty(ty).is_none() {
                for projections in self.storage_fields(ty, 0) {
                    let definite = self
                        .field_origin(source, ty, &projections, all, 0)
                        .is_some_and(|p| p.root == Root::Temporary);
                    if !definite && !self.field_may_temporary(source, ty, &projections, all, 0) {
                        continue;
                    }
                    let steps: Vec<_> = projections
                        .iter()
                        .map(|p| match p {
                            super::Projection::Field(name) => Step::Field(*name),
                            _ => unreachable!(),
                        })
                        .collect();
                    if let Some(path) = target.clone().append(&steps) {
                        // A returned field can be nullable or conditionally initialized.
                        // Preserve the risk without turning a field class into a must write.
                        paths.push((path, false));
                    }
                }
                let source = self.places.place(source, ty);
                let prefix = self.places.paths[source].1.len();
                for field in self.places.descendants(source) {
                    let Some(&ty) = self.places.types.get(&field) else {
                        return Vec::new();
                    };
                    if self.compiler.ir_ty(ty).is_some()
                        && !self.compiler.ty.types.is_pointer(ty)
                        && !matches!(self.compiler.ty.types.kind(ty), super::TypeKind::Proc(_))
                    {
                        continue;
                    }
                    if !self
                        .writes
                        .get(&field)
                        .into_iter()
                        .flatten()
                        .any(|v| self.may_temporary(*v, all, &mut HashSet::default(), 0))
                    {
                        continue;
                    }
                    let steps: Option<Vec<_>> = self.places.paths[field].1[prefix..]
                        .iter()
                        .map(|p| match p {
                            super::Projection::Field(n) => Some(Step::Field(*n)),
                            super::Projection::Element(i) => Some(Step::Element(*i)),
                            super::Projection::Deref => Some(Step::Deref),
                            super::Projection::DynamicElement(_) => None,
                        })
                        .collect();
                    let Some(path) = steps.and_then(|s| target.clone().append(&s)) else {
                        // Never defer only part of an unsupported aggregate copy.
                        return Vec::new();
                    };
                    // A field can have conditional definitions before the aggregate copy.
                    if !paths.iter().any(|(old, _)| old == &path) {
                        paths.push((path, false));
                    }
                }
            }
            if paths.is_empty() && self.may_temporary(source, all, &mut HashSet::default(), 0) {
                paths.push((target, false));
            }
        }
        paths
            .into_iter()
            .map(|(target, definite)| {
                (
                    Effect {
                        target,
                        kind: Kind::StoreTemporary,
                        guard: None,
                    },
                    span,
                    definite,
                )
            })
            .collect()
    }

    // A string is one view, including when its ABI rvalue is its header's address.
    // Its pointed-to bytes are not additional fields copied into the destination.
    fn string_may_temporary(
        &mut self,
        source: super::Val,
        ty: super::TypeId,
        all: &HashMap<ProcId, Summary>,
    ) -> bool {
        if self.may_temporary(source, all, &mut HashSet::default(), 0) {
            return true;
        }
        let header = self.places.place(source, ty);
        let data = self.places.projected(
            header,
            super::Projection::Field(jaic::intern::Sym::intern("data")),
        );
        std::iter::once(header).chain(data).any(|p| {
            self.writes
                .get(&p)
                .into_iter()
                .flatten()
                .any(|v| self.may_temporary(*v, all, &mut HashSet::default(), 0))
        })
    }

    fn temporary_source(
        &mut self,
        event: &Event,
        all: &HashMap<ProcId, Summary>,
    ) -> Option<(Path, super::Val, super::TypeId, Span)> {
        match event {
            Event::Store {
                place,
                value,
                ty,
                span,
                ..
            } => {
                if self.compiler.ir_ty(*ty).is_some()
                    && !self.compiler.ty.types.is_pointer(*ty)
                    && !matches!(self.compiler.ty.types.kind(*ty), super::TypeKind::Proc(_))
                {
                    return None;
                }
                let p = self.places.place(*place, *ty);
                Some((
                    self.location(p, all, &mut HashSet::default(), 0)?,
                    *value,
                    *ty,
                    *span,
                ))
            }
            Event::Call {
                callee,
                args,
                span,
                expands: false,
                ..
            } if self.compiler.proc_has_note(*callee, "AddsToArray") => {
                let array = args.iter().find(|a| a.formal == 0)?;
                let source = args.iter().find(|a| a.formal == 1)?;
                Some((
                    self.argument_trace(array, all, &mut HashSet::default(), 0)?
                        .append(&[Step::Storage])?,
                    value(source)?,
                    source.ty,
                    *span,
                ))
            }
            _ => None,
        }
    }
}

impl Resolver<'_> {
    pub(in crate::rules) fn temporary_destinations(
        &mut self,
        event: &Event,
        all: &HashMap<ProcId, Summary>,
        guaranteed: bool,
    ) -> Vec<usize> {
        let (Event::Call {
            args,
            expands: false,
            ..
        }
        | Event::IndirectCall {
            args,
            allocator: false,
            ..
        }) = event
        else {
            return Vec::new();
        };
        let targets = self.call_targets(event);
        if guaranteed && targets.unknown {
            return Vec::new();
        }
        let mut selected: Option<HashSet<Effect>> = None;
        for callee in targets.ordered() {
            let Some(summary) = all.get(&callee) else {
                return Vec::new();
            };
            let map = if guaranteed {
                &summary.guaranteed
            } else {
                &summary.may
            };
            let effects: HashSet<_> = map
                .keys()
                .filter(|e| e.kind == Kind::StoreTemporary && e.guard.is_none())
                .cloned()
                .collect();
            if let Some(prior) = &mut selected {
                if guaranteed {
                    prior.retain(|e| effects.contains(e));
                } else {
                    prior.extend(effects);
                }
            } else {
                selected = Some(effects);
            }
        }
        let mut effects: Vec<_> = selected
            .unwrap_or_default()
            .into_iter()
            .filter_map(|e| {
                let Root::Parameter(formal) = e.target.root else {
                    return None;
                };
                Some((formal, e.target.steps))
            })
            .collect();
        effects.sort_unstable();
        effects
            .into_iter()
            .filter_map(|(formal, steps)| {
                let arg = args.iter().find(|a| a.formal == formal)?;
                let p = self
                    .compiler
                    .ty
                    .types
                    .pointee(arg.ty)
                    .filter(|ty| *ty == jaic::types::TypeId::VOID)
                    .and_then(|_| self.stored_address(value(arg)?, &mut HashSet::default()))
                    .or_else(|| self.object(arg))?;
                self.materialize(&Path {
                    root: Root::Local(p),
                    steps,
                })
            })
            .collect()
    }
}

impl Resolver<'_> {
    // A join of temporary and non-temporary values is a may effect, never a guarantee.
    pub(super) fn may_temporary(
        &self,
        v: super::Val,
        all: &HashMap<ProcId, Summary>,
        seen: &mut HashSet<super::Val>,
        depth: usize,
    ) -> bool {
        if depth > 24 || !seen.insert(v) {
            return false;
        }
        let temporary = self
            .value(v, all)
            .is_some_and(|p| p.root == Root::Temporary)
            || self
                .values
                .get(&v)
                .into_iter()
                .flatten()
                .any(|def| match def {
                    super::Def::Call(callee, _, index) => all
                        .get(callee)
                        .is_some_and(|s| s.may_returns.contains(index)),
                    super::Def::Copy(source) => self.may_temporary(*source, all, seen, depth + 1),
                    super::Def::Address(p)
                        if !self.exposed.contains(p)
                            && self
                                .compiler
                                .ir_ty(
                                    self.places
                                        .types
                                        .get(p)
                                        .copied()
                                        .unwrap_or(self.places.paths[*p].0.1),
                                )
                                .is_none() =>
                    {
                        self.writes
                            .get(p)
                            .into_iter()
                            .flatten()
                            .any(|source| self.may_temporary(*source, all, seen, depth + 1))
                    }
                    super::Def::Load(p, _) | super::Def::Borrow(p) => {
                        let (root, projections) = &self.places.paths[*p];
                        self.writes
                            .get(p)
                            .into_iter()
                            .flatten()
                            .any(|source| self.may_temporary(*source, all, seen, depth + 1))
                            || !projections.is_empty()
                                && self.field_may_temporary(
                                    root.0,
                                    root.1,
                                    projections,
                                    all,
                                    depth + 1,
                                )
                    }
                    _ => false,
                });
        seen.remove(&v);
        temporary
    }
}
