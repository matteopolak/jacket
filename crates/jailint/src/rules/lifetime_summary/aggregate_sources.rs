//! Pointer fields copied by a checked aggregate store are read at that store's epoch.
use super::physical_slots::Slot;
use super::value_writes::{Installed, Query, Source};
use super::{Def, HashMap, HashSet, Path, ProcId, Projection, Resolver, Root, Step, Summary, Val};
use crate::rules::lifetime_movement::facts::Site;
use jaic::types::TypeId;

pub(super) struct Read {
    pub slot: Option<Slot>,
    pub entry: Path,
}

impl Resolver<'_> {
    pub(super) fn aggregate_read(
        &self,
        value: Val,
        ty: TypeId,
        base: usize,
        place: usize,
    ) -> Option<Read> {
        let base_path = &self.places.paths[base];
        let path = &self.places.paths[place];
        if base_path.0 != path.0 || !path.1.starts_with(&base_path.1) {
            return None;
        }
        let steps = path.1[base_path.1.len()..]
            .iter()
            .map(|step| match step {
                Projection::Field(name) => Some(Step::Field(*name)),
                Projection::Element(index) => Some(Step::Element(*index)),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?;
        let width = self
            .compiler
            .ty
            .types
            .size_of(*self.places.types.get(&place)?);
        let (_, begin, _) = super::storage_versions::inline_range(
            self.compiler,
            self.body,
            &self.places,
            base,
            self.compiler.ty.types.size_of(ty),
        )?;
        let (_, offset, _) = super::storage_versions::inline_range(
            self.compiler,
            self.body,
            &self.places,
            place,
            width,
        )?;
        let delta = offset.checked_sub(begin)?;
        let mut visited = HashSet::default();
        self.aggregate_read_value(value, (ty, &steps, delta, width), &mut visited, 0)
    }

    fn aggregate_read_value(
        &self,
        value: Val,
        layout: (TypeId, &[Step], u64, u64),
        seen: &mut HashSet<Val>,
        depth: usize,
    ) -> Option<Read> {
        let (ty, steps, delta, width) = layout;
        if depth > 24 || !seen.insert(value) {
            return None;
        }
        if self.compiler.ir_ty(ty).is_none()
            && let Some(source) = self.places.existing(value, ty)
            && let Some(slot) = self.aggregate_place_slot(source, delta, width)
        {
            let entry = self.entry_path(slot)?;
            return Some(Read {
                slot: Some(slot),
                entry,
            });
        }
        let mut chosen: Option<Read> = None;
        for definition in self.values.get(&value)? {
            let read = match definition {
                Def::Parameter(formal, ..)
                    if self.params.values().any(|(n, t)| {
                        n == formal && self.compiler.ty.types.pointee(*t) == Some(ty)
                    }) =>
                {
                    let slot = self.aggregate_slot(value, delta, width)?;
                    Read {
                        slot: Some(slot),
                        entry: self.entry_path(slot)?,
                    }
                }

                Def::Parameter(formal, ..)
                    if self.params.values().any(|(n, t)| n == formal && *t == ty) =>
                {
                    Read {
                        slot: None,
                        entry: Path {
                            root: Root::Parameter(*formal),
                            steps: steps.to_vec(),
                        }
                        .append(&[Step::Deref])?,
                    }
                }
                Def::Copy(source) => {
                    self.aggregate_read_value(*source, layout, &mut seen.clone(), depth + 1)?
                }
                Def::Address(source) if self.places.types.get(source) == Some(&ty) => {
                    let slot = self.aggregate_slot(value, delta, width)?;
                    let entry = self.entry_path(slot)?;
                    Read {
                        slot: Some(slot),
                        entry,
                    }
                }
                _ => return None,
            };
            if chosen
                .as_ref()
                .is_some_and(|prior| prior.slot != read.slot || prior.entry != read.entry)
            {
                return None;
            }
            chosen = Some(read);
        }
        chosen
    }

    pub(super) fn aggregate_origin(
        &self,
        captured: (Val, TypeId, usize, usize, Site),
        all: &HashMap<ProcId, Summary>,
        seen: &mut HashSet<Val>,
        depth: usize,
    ) -> Option<Path> {
        let (value, ty, base, place, site) = captured;
        if depth > 24 {
            return None;
        }
        let Some(read) = self.aggregate_read(value, ty, base, place) else {
            return (self.return_mode.get() && !self.storage_mode.get())
                .then(|| self.aggregate_call_origin(captured, all, depth + 1))
                .flatten();
        };
        let Some(slot) = read.slot else {
            return Some(read.entry);
        };
        let sources = self.before(site, slot, all);
        let mut chosen = None;
        for source in sources {
            let path = match source {
                Source::Entry => read.entry.clone(),
                Source::Value(value) => self.trace(value, all, &mut seen.clone(), depth + 1)?,
                Source::Unknown | Source::Call(_) => return None,
            };
            if chosen.as_ref().is_some_and(|prior| prior != &path) {
                return None;
            }
            chosen = Some(path);
        }
        chosen
    }

    /// Immutable checked call results can carry layout-independent field origins.
    /// Local aggregate memory must still be read at its captured store epoch.
    fn aggregate_call_origin(
        &self,
        captured: (Val, TypeId, usize, usize, Site),
        all: &HashMap<ProcId, Summary>,
        depth: usize,
    ) -> Option<Path> {
        let (value, ty, base, place, site) = captured;
        let base_path = &self.places.paths[base];
        let path = &self.places.paths[place];
        if base_path.0 != path.0 || !path.1.starts_with(&base_path.1) {
            return None;
        }
        let projections = &path.1[base_path.1.len()..];
        if projections
            .iter()
            .any(|p| matches!(p, Projection::Deref | Projection::DynamicElement(_)))
        {
            return None;
        }
        self.aggregate_call_value(
            value,
            (ty, site),
            projections,
            all,
            &mut HashSet::default(),
            depth,
        )
    }

    fn aggregate_call_value(
        &self,
        value: Val,
        captured: (TypeId, Site),
        projections: &[Projection],
        all: &HashMap<ProcId, Summary>,
        seen: &mut HashSet<Val>,
        depth: usize,
    ) -> Option<Path> {
        let (ty, site) = captured;
        if depth > 24 || !seen.insert(value) {
            return None;
        }
        let [definition] = self.values.get(&value)?.as_slice() else {
            return None;
        };
        match definition {
            Def::Copy(source) => {
                self.aggregate_call_value(*source, captured, projections, all, seen, depth + 1)
            }
            Def::Call(callee, args, index)
                if self
                    .compiler
                    .proc(*callee)
                    .sig
                    .as_ref()
                    .and_then(|s| s.returns.get(*index))
                    .is_some_and(|actual| {
                        self.compiler.ty.types.repr(*actual) == self.compiler.ty.types.repr(ty)
                    }) =>
            {
                if !self.immediate_call_capture(value, *callee, *index, site) {
                    return None;
                }
                self.field_origin(value, ty, projections, all, depth + 1)
                    .or_else(|| {
                        if !self.builtin_data_projection(ty, projections) {
                            return None;
                        }
                        let (path, guard) = all.get(callee)?.returns.get(index)?;
                        if guard.is_some_and(|(formal, expected)| {
                            !args.iter().any(|a| {
                                a.formal == formal && !a.spread && a.constant == Some(expected)
                            })
                        }) {
                            return None;
                        }
                        let mut path = path.clone();
                        path.root = self.allocation_root(&path.root, value, args)?;
                        self.substitute(&path, args, all, &mut HashSet::default(), depth + 1)
                    })
                    .filter(|path| {
                        path.steps.is_empty()
                            && matches!(
                                path.root,
                                Root::Heap
                                    | Root::CallerAllocator
                                    | Root::AllocatorParameter(_)
                                    | Root::Temporary
                                    | Root::ScopedGlobal(..)
                                    | Root::Parameter(_)
                            )
                    })
            }
            _ => None,
        }
    }

    /// A checked aggregate result is captured before any possible intervening
    /// mutation, not re-read from a local header at its later use.
    fn immediate_call_capture(
        &self,
        value: Val,
        callee: ProcId,
        result: usize,
        captured: Site,
    ) -> bool {
        let Some(block) = self.body.blocks.get(captured.block) else {
            return false;
        };
        let mut origin = None;
        for (ordinal, event) in block.events.iter().enumerate().take(captured.event) {
            if matches!(event,super::Event::Call {callee:id,results,expands:false,..} if *id==callee && results.iter().any(|(index,v,_)|*index==result&&*v==value))
            {
                if origin.is_some() {
                    return false;
                }
                origin = Some(ordinal);
            }
        }
        let Some(origin) = origin else {
            return false;
        };
        block.events[origin + 1..captured.event]
            .iter()
            .all(|event| {
                !matches!(
                    event,
                    super::Event::Store { .. }
                        | super::Event::Initialize { .. }
                        | super::Event::Call { .. }
                        | super::Event::IndirectCall { .. }
                        | super::Event::Barrier
                        | super::Event::Grow { .. }
                        | super::Event::Address { .. }
                        | super::Event::ScopeEnd { .. }
                )
            })
    }

    /// Whole-result storage provenance applies only to the actual built-in data
    /// member. An arbitrary pointer member of a user record has no such meaning.
    fn builtin_data_projection(&self, ty: TypeId, projections: &[Projection]) -> bool {
        if !matches!(
            self.compiler.ty.types.kind(ty),
            jaic::types::TypeKind::String
                | jaic::types::TypeKind::Array {
                    kind: jaic::types::ArrayKind::View | jaic::types::ArrayKind::Resizable,
                    ..
                }
        ) {
            return false;
        }
        let [Projection::Field(name)] = projections else {
            return false;
        };
        super::physical_slots::field_projection(self.compiler, self.body, &self.places, ty, *name)
            .is_some_and(|(offset, field_ty)| {
                offset == jaic::layout::view::DATA && self.compiler.ty.types.is_pointer(field_ty)
            })
    }

    pub(super) fn aggregate_possible(
        &self,
        value: Val,
        ty: TypeId,
        base: usize,
        place: usize,
        site: Site,
        all: &HashMap<ProcId, Summary>,
    ) -> HashSet<Path> {
        let Some(read) = self.aggregate_read(value, ty, base, place) else {
            return HashSet::default();
        };
        let Some(slot) = read.slot else {
            return HashSet::from_iter([read.entry]);
        };
        let mut out = HashSet::default();
        for source in self.installed(Query::Before(site, slot), all) {
            match source {
                Installed::Value(path) => {
                    out.insert(path);
                }
                Installed::Entry(source) => {
                    if let Some(path) = self.entry_path(source) {
                        out.insert(path);
                    } else if source == slot {
                        out.insert(read.entry.clone());
                    }
                }
                Installed::Address(slot) => {
                    if let Some(path) = self.address_path(slot) {
                        out.insert(path);
                    }
                }
                Installed::Unknown => {}
            }
        }
        out
    }
}
