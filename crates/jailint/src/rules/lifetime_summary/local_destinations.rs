//! Resolve a store through a private, singly assigned local pointer to its stack owner.
use super::{
    Def, HashMap, HashSet, ProcId, Projection, Resolver, Root, Step, Summary, TypeId, Val,
};

impl Resolver<'_> {
    pub(in crate::rules) fn local_destination(
        &mut self,
        value: Val,
        ty: TypeId,
        all: &HashMap<ProcId, Summary>,
    ) -> Option<usize> {
        let p = self.places.place(value, ty);
        self.local_destination_of(p, all)
    }

    pub(in crate::rules) fn local_destination_of(
        &mut self,
        p: usize,
        all: &HashMap<ProcId, Summary>,
    ) -> Option<usize> {
        let (root, projections) = &self.places.paths[p];
        let slot = self.places.root_of(p);
        if self.exposed.contains(&slot) {
            return None;
        }
        let stable = if let Some([source]) = self.writes.get(&slot).map(Vec::as_slice) {
            self.private_pointer(*source, &mut HashSet::default())
        } else {
            self.private_pointer(root.0, &mut HashSet::default())
        };
        if projections.first() != Some(&Projection::Deref) || !stable {
            return None;
        }
        let path = self.location(p, all, &mut HashSet::default(), 0)?;
        if let Root::Local(owner) = path.root
            && self.local_fixed_array(owner)
            && path.steps.first() == Some(&Step::Storage)
            && path.steps[1..]
                .iter()
                .all(|s| matches!(s, Step::Field(_) | Step::Element(_)))
        {
            // A fixed array iterator borrows inline storage. Its destination has
            // the array's lifetime, rather than an unknown external pointer's.
            return Some(owner);
        }
        if !matches!(path.root, Root::Local(_))
            || path
                .steps
                .iter()
                .any(|s| !matches!(s, Step::Field(_) | Step::Element(_)))
        {
            return None;
        }
        self.materialize_storage(&path)
    }

    fn local_fixed_array(&self, p: usize) -> bool {
        self.owned.contains(&self.places.root_of(p))
            && self.places.types.get(&p).is_some_and(|&ty| {
                matches!(
                    self.compiler.ty.types.kind(ty),
                    super::TypeKind::Array {
                        kind: jaic::types::ArrayKind::Fixed(_),
                        ..
                    }
                )
            })
    }

    fn private_pointer(&self, value: Val, seen: &mut HashSet<Val>) -> bool {
        if seen.len() >= 24 || !seen.insert(value) {
            return false;
        }
        let Some(defs) = self.values.get(&value) else {
            return false;
        };
        let valid = !defs.is_empty()
            && defs.iter().all(|def| match def {
                Def::Address(_) => true,
                Def::Borrow(p) => self.local_fixed_array(*p),
                Def::Copy(v) => self.private_pointer(*v, seen),
                Def::Load(p, _)
                    if self.owned.contains(&self.places.root_of(*p))
                        && !self.exposed.contains(p) =>
                {
                    self.writes
                        .get(p)
                        .is_some_and(|writes| match writes.as_slice() {
                            [v] => self.private_pointer(*v, seen),
                            _ => false,
                        })
                }
                _ => false,
            });
        seen.remove(&value);
        valid
    }
}
