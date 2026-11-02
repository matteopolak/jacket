//! Logical-entry return contracts and selector substitution.
use super::{
    Def, HashMap, HashSet, LifetimeArgument, Path, ProcId, Resolver, Root, Selector, Summary, Val,
    value,
};

impl Resolver<'_> {
    pub(super) fn swap_effect(
        &self,
        event: &super::Event,
        all: &HashMap<ProcId, Summary>,
        facts: &crate::rules::lifetime_movement::facts::Facts,
        site: crate::rules::lifetime_movement::facts::Site,
    ) -> Option<super::Effect> {
        let super::Event::Call {
            callee,
            args,
            ..
        } = event
        else {
            return None;
        };
        if !self.compiler.proc_has_note(*callee, "SwapsArguments") {
            return None;
        }
        let position =
            |formal| facts.position(value(args.iter().find(|a| a.formal == formal)?)?, site);
        let a = position(0)?;
        let b = position(1)?;
        if a.region != b.region || !a.bounded || !b.bounded {
            return None;
        }
        let p = self.places.lookup(&facts.places, a.region)?;
        let mut target = self.location(p, all, &mut HashSet::default(), 0)?;
        if !matches!(target.root, Root::Parameter(_)) {
            return None;
        }
        target.steps.push(super::Step::Storage);
        Some(super::Effect {
            target,
            guard: None,
            kind: super::Kind::Mutation(super::lifetime_movement::Mutation {
                action: super::lifetime_movement::Action::Swap,
                first: a.selector,
                second: b.selector,
            }),
        })
    }

    /// Writing a built-in descriptor count changes its logical extent. A by-value
    /// descriptor's header is private to the callee and must not shrink the caller's view.
    pub(super) fn count_effect(
        &self,
        event: &super::Event,
        all: &HashMap<ProcId, Summary>,
    ) -> Option<super::Effect> {
        let super::Event::Store {
            place,
            ty,
            constant: Some(n),
            ..
        } = *event
        else {
            return None;
        };
        let p = self.places.existing(place, ty)?;
        if !matches!(self.places.paths[p].1.last(), Some(super::Projection::Field(name)) if name.as_str() == "count")
        {
            return None;
        }
        let parent = *self.places.descriptor_fields.get(&p)?;
        let mut target = self.location(parent, all, &mut HashSet::default(), 0)?;
        let Root::Parameter(formal) = target.root else {
            return None;
        };
        let by_pointer = self
            .params
            .values()
            .any(|(n, ty)| *n == formal && self.compiler.ty.types.is_pointer(*ty));
        if !by_pointer && !target.steps.contains(&super::Step::Deref) {
            return None;
        }
        target.steps.push(super::Step::Storage);
        Some(super::Effect {
            target,
            kind: super::Kind::Mutation(super::lifetime_movement::Mutation::new(
                super::lifetime_movement::Action::Count,
                Selector::Constant(n as i64 as i128),
            )),
            guard: None,
        })
    }

    pub(super) fn selector_actual(
        &self,
        selector: Selector,
        args: &[LifetimeArgument],
        all: &HashMap<ProcId, Summary>,
    ) -> Selector {
        match selector {
            Selector::Formal(formal) => {
                args.iter()
                    .find(|a| a.formal == formal)
                    .map_or(Selector::Unknown, |a| {
                        if let Some(n) = a.constant {
                            return Selector::Constant(n as i64 as i128);
                        }
                        let Some(v) = value(a) else {
                            return Selector::Unknown;
                        };
                        match self.value(v, all) {
                            Some(Path {
                                root: Root::Parameter(p),
                                steps,
                            }) if steps.is_empty() => Selector::Formal(p),
                            _ => Selector::Local(v),
                        }
                    })
            }
            Selector::Local(_) => Selector::Unknown,
            other => other,
        }
    }

    pub fn element(&self, value: Val, all: &HashMap<ProcId, Summary>) -> Option<(Path, Selector)> {
        self.element_inner(value, all, &mut HashSet::default())
    }

    fn element_inner(
        &self,
        v: Val,
        all: &HashMap<ProcId, Summary>,
        seen: &mut HashSet<Val>,
    ) -> Option<(Path, Selector)> {
        if seen.len() >= 24 || !seen.insert(v) {
            return None;
        }
        let defs = self.values.get(&v)?;
        match defs.as_slice() {
            [Def::Call(callee, args, index)] => {
                let (path, key) = all.get(callee)?.elements.get(index)?;
                Some((
                    self.substitute(path, args, all, &mut HashSet::default(), 0)?,
                    self.selector_actual(*key, args, all),
                ))
            }
            [Def::Copy(source)] => self.element_inner(*source, all, seen),
            [Def::Load(place, _)] => {
                let [source] = self.writes.get(place)?.as_slice() else {
                    return None;
                };
                self.element_inner(*source, all, seen)
            }
            _ => None,
        }
    }
}
