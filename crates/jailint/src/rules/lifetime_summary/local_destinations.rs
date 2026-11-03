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
