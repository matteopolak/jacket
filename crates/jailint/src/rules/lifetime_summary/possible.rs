//! May provenance follows all recorded definitions, including loop-carried list
//! cursors. It never upgrades an ambiguous value into a guaranteed origin.
use super::{
    Def, Effect, HashMap, HashSet, Kind, Path, ProcId, Projection, Resolver, Root, Step, Summary,
    Val, value,
};

impl Resolver<'_> {
    pub(super) fn possible_effects(
        &self,
        summary: &Summary,
        args: &[super::LifetimeArgument],
        all: &HashMap<ProcId, Summary>,
    ) -> Vec<(Effect, Vec<super::Span>)> {
        let mut out = Vec::new();
        for (effect, witness) in &summary.may {
            if !matches!(effect.kind, Kind::Free | Kind::Resize) {
                continue;
            }
            let Root::Parameter(formal) = effect.target.root else {
                continue;
            };
            let Some(arg) = args.iter().find(|a| a.formal == formal) else {
                continue;
            };
            let Some(actual) = value(arg) else {
                continue;
            };
            if self
                .substitute(&effect.target, args, all, &mut HashSet::default(), 0)
                .is_some_and(|p| !matches!(p.root, Root::Local(_)))
            {
                continue;
            }
            let guard = if let Some((formal, expected)) = effect.guard {
                let Some(arg) = args.iter().find(|a| a.formal == formal) else {
                    continue;
                };
                let Some(guard) = self.mode_guard(arg, expected, all) else {
                    continue;
                };
                guard
            } else {
                None
            };
            for mut target in self.possible_value(actual, all, &mut HashSet::default(), 0) {
                let mut steps = effect.target.steps.clone();
                if effect.kind == Kind::Free
                    && steps.is_empty()
                    && self.compiler.ir_ty(arg.ty).is_none()
                {
                    steps.push(Step::Storage);
                }
                let Some(path) = target.append(&steps) else {
                    continue;
                };
                target = path;
                out.push((
                    Effect {
                        target,
                        guard,
                        kind: effect.kind,
                    },
                    witness.clone(),
                ));
            }
        }
        out
    }

    fn possible_value(
        &self,
        v: Val,
        all: &HashMap<ProcId, Summary>,
        seen: &mut HashSet<Val>,
        depth: usize,
    ) -> HashSet<Path> {
        if depth > 24 || !seen.insert(v) {
            return HashSet::default();
        }
        let mut out = HashSet::default();
        if let Some(path) = self.trace(v, all, &mut HashSet::default(), 0) {
            let exact = !matches!(path.root, Root::Local(_));
            out.insert(path);
            if exact {
                seen.remove(&v);
                return out;
            }
        }
        for def in self.values.get(&v).into_iter().flatten() {
            let paths = match def {
                Def::Copy(source) => self.possible_value(*source, all, seen, depth + 1),
                Def::Load(p, ty) => {
                    let mut paths = self.possible_location(*p, all, seen, depth + 1);
                    if self.compiler.ty.types.is_pointer(*ty) && !self.places.paths[*p].1.is_empty()
                    {
                        paths = paths
                            .into_iter()
                            .filter_map(|p| p.append(&[Step::Deref]))
                            .collect();
                    }
                    paths
                }
                Def::Address(p) => self.possible_location(*p, all, seen, depth + 1),
                _ => HashSet::default(),
            };
            out.extend(paths);
            if out.len() > 32 {
                break;
            }
        }
        seen.remove(&v);
        out
    }

    fn possible_location(
        &self,
        p: usize,
        all: &HashMap<ProcId, Summary>,
        seen: &mut HashSet<Val>,
        depth: usize,
    ) -> HashSet<Path> {
        if depth > 24 {
            return HashSet::default();
        }
        let mut out = HashSet::default();
        if let Some(path) = self.location(p, all, &mut HashSet::default(), 0) {
            out.insert(path);
        }
        let root = self.places.root_of(p);
        let mut projections = self.places.paths[p].1.as_slice();
        if projections.first() == Some(&Projection::Deref) {
            projections = &projections[1..];
        }
        let Some(steps) = projections
            .iter()
            .map(|p| match p {
                Projection::Field(n) => Some(Step::Field(*n)),
                Projection::Element(i) => Some(Step::Element(*i)),
                Projection::Deref => Some(Step::Deref),
                Projection::DynamicElement(_) => None,
            })
            .collect::<Option<Vec<_>>>()
        else {
            return out;
        };
        for v in self.writes.get(&root).into_iter().flatten() {
            for path in self.possible_value(*v, all, seen, depth + 1) {
                if let Some(path) = path.append(&steps) {
                    out.insert(path);
                }
                if out.len() > 32 {
                    return out;
                }
            }
        }
        out
    }
}
