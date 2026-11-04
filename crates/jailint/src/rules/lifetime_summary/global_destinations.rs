//! Replay writes through closed, activation-owned global pointer bindings at their callers.
use super::{Event, HashSet, Path, ProcId, Resolver, Root, Step, TypeId};
use jaic::ir::GlobalId;

impl Resolver<'_> {
    pub(super) fn temporary_target(&self, mut path: Path) -> Option<Path> {
        if matches!(path.root, Root::Parameter(_) | Root::ScopedGlobal(..)) {
            return Some(path);
        }
        let Root::Local(place) = path.root else {
            return None;
        };
        let ((value, ty), projections) = &self.places.paths[place];
        if !projections.is_empty() || !matches!(path.steps.as_slice(), [Step::Deref, Step::Storage])
        {
            return None;
        }
        let (_, global) = self.body.globals.iter().find(|(v, _)| v == value)?;
        if !self.targets.scoped_pointers.contains(global) {
            return None;
        }
        path.root = Root::ScopedGlobal(*global, *ty);
        Some(path)
    }

    pub(super) fn global_destination(
        &mut self,
        event: &Event,
        global: GlobalId,
        ty: TypeId,
        steps: &[Step],
    ) -> Option<usize> {
        if steps.first() != Some(&Step::Deref) || self.rebinds_global(event, global) {
            return None;
        }
        let address = self.body.globals.iter().find(|(_, id)| *id == global)?.0;
        let slot = self.places.place(address, ty);
        let start = self.body.blocks.iter().enumerate().find_map(|(b, block)| {
            block
                .events
                .iter()
                .position(|e| std::ptr::eq(e, event))
                .map(|i| (b, i))
        })?;
        let mut incoming = vec![Vec::new(); self.body.blocks.len()];
        for (index, block) in self.body.blocks.iter().enumerate() {
            for &next in &block.successors {
                incoming[next].push(index);
            }
        }
        let mut queue = vec![start];
        let mut seen = HashSet::default();
        let mut source = None;
        // Every path to the call must agree on the most recent binding. A dominating
        // assignment alone is insufficient when a branch can overwrite the slot.
        while let Some((block, end)) = queue.pop() {
            if !seen.insert((block, end)) {
                continue;
            }
            let mut bound = false;
            for event in self.body.blocks[block].events[..end].iter().rev() {
                if let Event::Store {
                    place,
                    value,
                    ty: stored_ty,
                    constant,
                    ..
                } = *event
                    && self.places.existing(place, stored_ty) == Some(slot)
                {
                    if constant == Some(0)
                        || !self.body.stack_places.contains(&value)
                        || source.is_some_and(|prior| prior != value)
                    {
                        return None;
                    }
                    source = Some(value);
                    bound = true;
                    break;
                }
                if self.rebinds_global(event, global) {
                    return None;
                }
            }
            if !bound {
                if incoming[block].is_empty() {
                    return None;
                }
                queue.extend(
                    incoming[block]
                        .iter()
                        .map(|&previous| (previous, self.body.blocks[previous].events.len())),
                );
            }
        }
        let owner = self
            .places
            .place(source?, self.compiler.ty.types.pointee(ty)?);
        self.materialize_storage(&Path {
            root: Root::Local(owner),
            steps: steps[1..].to_vec(),
        })
    }

    fn rebinds_global(&self, event: &Event, global: GlobalId) -> bool {
        let mut queue: Vec<_> = self.call_targets(event).procedures.into_iter().collect();
        let mut seen = HashSet::<ProcId>::default();
        let Some(facts) = self.compiler.ed.facts() else {
            return true;
        };
        while let Some(id) = queue.pop() {
            if !seen.insert(id) {
                continue;
            }
            let Some(body) = facts.lifetime_bodies.get(&id) else {
                continue;
            };
            for event in body.blocks.iter().flat_map(|b| &b.events) {
                match event {
                    Event::Store {
                        place, ..
                    } if body
                        .globals
                        .iter()
                        .any(|(v, id)| v == place && *id == global) =>
                    {
                        return true;
                    }
                    Event::Call {
                        callee,
                        expands: false,
                        ..
                    } => queue.push(*callee),
                    Event::IndirectCall {
                        callee,
                        allocator: false,
                        ..
                    } => queue.extend(self.targets.get(body.procedure, *callee).procedures),
                    _ => {}
                }
            }
        }
        false
    }
}
