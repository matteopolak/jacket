//! Parameters a body keeps in a field of another parameter.
//!
//! `keep :: (r: *Registry, n: *Node) { r.item = n; }` leaves `r.item` holding `n` on every
//! normal return, so a caller that frees `n` has left `r.item` dangling. The body says it: no
//! declaration note is needed.
use super::{
    Effect, Event, HashMap, HashSet, Kind, LifetimeArgument, Path, ProcId, Resolver, Root, Summary,
    returning_blocks, value,
};

impl Resolver<'_> {
    /// The fields of parameters that every normal return leaves holding another parameter
    /// itself, as (field path, holding parameter).
    ///
    /// Only a field this body writes exactly once counts: a second store, or a callee that also
    /// writes the field, makes the final value unclear. `writes` are this body's write
    /// effects, `unknown` the parameters it writes through in ways it cannot follow.
    pub(super) fn infer_retains(
        &self,
        all: &HashMap<ProcId, Summary>,
        writes: &[(Effect, Vec<super::Span>)],
        unknown: &HashSet<usize>,
    ) -> HashMap<Path, usize> {
        let body = self.body;
        let live = returning_blocks(body, all, self.targets);
        let mut effects = vec![Vec::new(); body.blocks.len()];
        for (b, block) in body.blocks.iter().enumerate() {
            if !live.contains(b) {
                continue;
            }
            for event in &block.events {
                for (dest, formal) in self.retained_by(event, all) {
                    let effect = Effect {
                        target: dest,
                        kind: Kind::Retain(formal),
                        guard: None,
                    };
                    effects[b].push((effect, Vec::new()));
                }
            }
        }
        let overlaps = |a: &Path, b: &Path| {
            a.root == b.root && (a.steps.starts_with(&b.steps) || b.steps.starts_with(&a.steps))
        };
        let candidates: HashSet<_> = effects
            .iter()
            .flatten()
            .map(|(effect, _)| effect.clone())
            .collect();
        let mut result = HashMap::default();
        for effect in &candidates {
            let Kind::Retain(formal) = effect.kind else {
                continue;
            };
            let Root::Parameter(owner) = effect.target.root else {
                continue;
            };
            let events = effects
                .iter()
                .flatten()
                .filter(|(e, _)| overlaps(&e.target, &effect.target))
                .count();
            let written = writes
                .iter()
                .filter(|(e, _)| e.kind == Kind::Write && overlaps(&e.target, &effect.target))
                .count();
            let other_formals = candidates
                .iter()
                .any(|e| e.target == effect.target && e.kind != effect.kind);
            if unknown.contains(&owner)
                || events != written
                || other_formals
                || super::flow::guaranteed(body, &effects, effect).is_none()
            {
                continue;
            }
            result.insert(effect.target.clone(), formal);
        }
        result
    }

    /// What one event leaves in the fields of this body's parameters: a store of a parameter,
    /// or a call that does so with one of this body's parameters as the stored value.
    fn retained_by(&self, event: &Event, all: &HashMap<ProcId, Summary>) -> Vec<(Path, usize)> {
        match event {
            Event::Store {
                place,
                value: stored,
                ty,
                ..
            } if self.holds_storage(*ty) => {
                let p = self.places.existing(*place, *ty);
                let Some(p) = p.filter(|p| self.write_sources.contains_key(p)) else {
                    return Vec::new();
                };
                let Some(dest) = self.location(p, all, &mut HashSet::default(), 0) else {
                    return Vec::new();
                };
                match self.value(*stored, all) {
                    Some(Path {
                        root: Root::Parameter(formal),
                        steps,
                    }) if steps.is_empty() && self.field_of_parameter(&dest) => {
                        vec![(dest, formal)]
                    }
                    _ => Vec::new(),
                }
            }
            Event::Call {
                callee,
                args,
                expands: false,
                ..
            } => {
                let Some(called) = all.get(callee) else {
                    return Vec::new();
                };
                called
                    .retains
                    .iter()
                    .filter_map(|(dest, formal)| {
                        let (dest, formal) = self.retained_in(dest, *formal, args, all)?;
                        self.field_of_parameter(&dest).then_some((dest, formal))
                    })
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    /// Whether a value of this type refers to storage it does not own: a pointer, or a string
    /// or array descriptor pointing at its data.
    fn holds_storage(&self, ty: jaic::types::TypeId) -> bool {
        let types = &self.compiler.ty.types;
        types.is_pointer(ty)
            || matches!(
                types.kind(ty),
                jaic::types::TypeKind::String
                    | jaic::types::TypeKind::Array {
                        kind: jaic::types::ArrayKind::View | jaic::types::ArrayKind::Resizable,
                        ..
                    }
            )
    }

    fn field_of_parameter(&self, dest: &Path) -> bool {
        matches!(dest.root, Root::Parameter(_)) && !dest.steps.is_empty()
    }

    /// A callee's retained field and holder, in terms of the call's arguments: the field as a
    /// path of the caller, and the caller's parameter that was stored (when it is one).
    fn retained_in(
        &self,
        dest: &Path,
        formal: usize,
        args: &[LifetimeArgument],
        all: &HashMap<ProcId, Summary>,
    ) -> Option<(Path, usize)> {
        let dest = self.substitute(dest, args, all, &mut HashSet::default(), 0)?;
        let arg = args.iter().find(|a| a.formal == formal)?;
        match self.value(value(arg)?, all)? {
            Path {
                root: Root::Parameter(formal),
                steps,
            } if steps.is_empty() => Some((dest, formal)),
            _ => None,
        }
    }

    /// The fields a call to a known procedure leaves holding one of its arguments: the field
    /// as a path in this body, and the argument that was stored.
    ///
    /// ```jai
    /// keep :: (r: *Registry, n: *Node) { r.item = n; }
    /// // keep(*local, node) retains `local.item` = `node`
    /// ```
    pub(in crate::rules) fn retained<'a>(
        &self,
        event: &'a Event,
        all: &HashMap<ProcId, Summary>,
    ) -> Vec<(Path, &'a LifetimeArgument)> {
        let Event::Call {
            callee,
            args,
            expands: false,
            ..
        } = event
        else {
            return Vec::new();
        };
        let Some(called) = all.get(callee) else {
            return Vec::new();
        };
        called
            .retains
            .iter()
            .filter_map(|(dest, formal)| {
                let dest = self.substitute(dest, args, all, &mut HashSet::default(), 0)?;
                let arg = args.iter().find(|a| a.formal == *formal)?;
                Some((dest, arg))
            })
            .collect()
    }
}
