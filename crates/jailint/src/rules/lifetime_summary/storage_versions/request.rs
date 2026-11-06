//! One query-owned prestate capture serviced by the ordinary storage transfer.
use super::*;
use crate::rules::lifetime_movement::facts::Site;
use crate::rules::lifetime_summary::physical_slots::Slot;
use crate::rules::lifetime_summary::{ProcId, Resolver, Summary};

pub(super) struct Capture<'a> {
    pub site: Option<Site>,
    pub slot: Option<usize>,
    pub range: Option<Slot>,
    pub live: crate::rules::lifetime_summary::flow::Live,
    pub sources: Sources,
    pub failed: bool,
    stores: HashMap<Site, (Slot, bool)>,
    event_work: Vec<Vec<(usize, usize)>>,
    ignored: HashSet<Site>,
    preserving_calls: HashSet<Site>,
    remaining: &'a mut usize,
    bound: Option<Bound>,
    bound_disjoint: bool,
    bound_target: bool,
    bound_active: bool,
    bound_sources: [Sources; 3],
}

/// One conditional returned-object byte request, not an owned/native root.
#[derive(Clone, Copy)]
pub(super) struct Bound {
    pub address: Val,
    pub ty: jaic::types::TypeId,
    fields: [Option<(usize, crate::rules::lifetime_facts::ByteRange)>; 3],
    pub image: Site,
    seed: Seed,
    root: usize,
    pub access: (i128, i128, u64),
}


#[derive(Clone, Copy)]
pub(super) enum Seed {
    Image,
    Call([Option<jaic::ide::lifetimes::LifetimeDefaultValue>; 3]),
}

impl<'a> Capture<'a> {
    pub fn new<'r>(
        r: &'r Resolver<'_>,
        at: Site,
        range: Slot,
        all: &HashMap<ProcId, Summary>,
        remaining: &'a mut usize,
        index: &crate::rules::lifetime_summary::captured_values::QueryIndex,
    ) -> Option<(std::borrow::Cow<'r, Places>, Self)> {
        Self::build(r, Some((at, range)), None, 0, all, remaining, index)
    }

    pub fn reads<'r>(
        r: &'r Resolver<'_>,
        slots: &HashSet<usize>,
        read_count: usize,
        all: &HashMap<ProcId, Summary>,
        remaining: &'a mut usize,
        index: &crate::rules::lifetime_summary::captured_values::QueryIndex,
    ) -> Option<(std::borrow::Cow<'r, Places>, Self)> {
        Self::build(r, None, Some(slots), read_count, all, remaining, index)
    }

    fn build<'r>(
        r: &'r Resolver<'_>,
        requested: Option<(Site, Slot)>,
        read_slots: Option<&HashSet<usize>>,
        read_count: usize,
        all: &HashMap<ProcId, Summary>,
        remaining: &'a mut usize,
        index: &crate::rules::lifetime_summary::captured_values::QueryIndex,
    ) -> Option<(std::borrow::Cow<'r, Places>, Self)> {
        if requested.is_some_and(|(_, range)| {
            range.width != r.compiler.ty.types.size_of(jaic::types::TypeId::VOID_PTR)
        }) || r.body.blocks.is_empty()
            || r.body.blocks.len() > 32
        {
            return None;
        }
        let events = r
            .body
            .blocks
            .iter()
            .try_fold(0usize, |n, b| n.checked_add(b.events.len()))?;
        let edges = r
            .body
            .blocks
            .iter()
            .try_fold(0usize, |n, b| n.checked_add(b.successors.len()))?;
        if events
            > if requested.is_some() {
                256
            } else {
                8192
            }
            || r.places.paths.len() > 256
            || requested.is_some_and(|(at, _)| {
                r.body
                    .blocks
                    .get(at.block)
                    .is_none_or(|b| at.event > b.events.len())
            })
            || r.body
                .blocks
                .iter()
                .any(|b| b.successors.iter().any(|i| *i >= r.body.blocks.len()))
        {
            return None;
        }
        *remaining = remaining.checked_sub(r.params.len())?;
        if let Some((_, range)) = requested {
            r.valid_slot(range)?;
        }
        *remaining = remaining.checked_sub(
            (if requested.is_some() {
                r.places.clone_work()?
            } else {
                0
            })
            .checked_add(events + edges + r.body.blocks.len())?,
        )?;
        let layout = layout_work(r, remaining)?;
        if requested.is_none() && jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!(
                "lifetime-object-capture layout={} remaining={}",
                layout, remaining
            );
        }
        let mut ignored = HashSet::default();
        let roots = if let Some(slots) = read_slots {
            *remaining = remaining
                .checked_sub(r.body.stack_roots.len() + r.body.slot_aliases.len() + slots.len())?;
            let roots = crate::rules::lifetime_facts::StackRoots::capture(r.body, 8192)?;
            for &slot in slots {
                if !r.places.paths.get(slot)?.1.is_empty() {
                    return None;
                }
                let root = roots.canonical(r.places.paths[slot].0.0);
                let bytes = roots.extent(root)??;
                if r.compiler.ty.types.size_of(r.places.selected_type(slot)?) > bytes {
                    return None;
                }
            }
            Some(roots)
        } else {
            None
        };
        if read_slots.is_some() {
            *remaining = remaining.checked_sub(r.body.overlay_accesses.len().checked_mul(4)?)?;
            for marker in &r.body.overlay_accesses {
                if marker.block != marker.projection_block
                    || marker.projection_event >= marker.event
                {
                    return None;
                }
                let block = r.body.blocks.get(marker.block)?;
                if !matches!(block.events.get(marker.event), Some(Event::Barrier)) {
                    return None;
                }
                let Event::FieldProjection {
                    result,
                    base_ty,
                    result_ty,
                    offset,
                    ..
                } = block.events.get(marker.projection_event)?
                else {
                    return None;
                };
                if *result != marker.result
                    || r.compiler.ty.types.repr(*base_ty)
                        != r.compiler.ty.types.repr(marker.descriptor_ty)
                    || offset
                        .checked_add(r.compiler.ty.types.size_of(*result_ty))
                        .is_none_or(|end| end > r.compiler.ty.types.size_of(*base_ty))
                {
                    return None;
                }
                let st = r
                    .compiler
                    .ty
                    .types
                    .as_struct(r.compiler.ty.types.repr_struct(marker.descriptor_ty))?;
                let info = r.compiler.ty.types.struct_info(st);
                *remaining = remaining.checked_sub(info.fields.len() + 1)?;
                if !info.is_union && !info.fields.iter().any(|field| field.overlay) {
                    return None;
                }
                if !ignored.insert(Site {
                    block: marker.block,
                    event: marker.event,
                }) {
                    return None;
                }
            }
        }
        let mut event_work = Vec::with_capacity(r.body.blocks.len());
        for (block, b) in r.body.blocks.iter().enumerate() {
            let mut costs = Vec::with_capacity(b.events.len());
            for (event, e) in b.events.iter().enumerate() {
                if ignored.contains(&Site {
                    block,
                    event,
                }) {
                    costs.push((1, 0));
                    continue;
                }
                let memory = match e {
                    Event::ContextSet {
                        place,
                        ty,
                        ..
                    }
                    | Event::Store {
                        place,
                        ty,
                        ..
                    }
                    | Event::Initialize {
                        place,
                        ty,
                        ..
                    }
                    | Event::DefaultImage {
                        place,
                        ty,
                        ..
                    } => Some((*place, *ty)),
                    _ => None,
                };
                let mut spill_work = 0;
                let mut cost = if let Some(slots) = read_slots {
                    match e {
                        Event::Barrier
                        | Event::Grow {
                            ..
                        } => {
                            spill_work = slots.len().checked_mul(3)?;
                            spill_work.checked_add(1)?
                        }
                        Event::Call {
                            expands: false, ..
                        }
                        | Event::IndirectCall {
                            ..
                        } => {
                            spill_work = slots.len().checked_mul(2)?;
                            spill_work.checked_add(1)?
                        }
                        Event::Load {
                            ..
                        }
                        | Event::ContextRead {
                            ..
                        } => {
                            spill_work = read_count;
                            read_count.checked_add(1)?
                        }
                        _ => 1,
                    }
                } else {
                    4
                };
                if let Some((place, ty)) = memory {
                    if matches!(e, Event::Store { .. } | Event::ContextSet { .. })
                        && let (Some(roots), Some(slots)) = (&roots, read_slots)
                    {
                        *remaining = remaining.checked_sub(slots.len() + 1)?;
                        let direct = roots.canonical(place);
                        if roots
                            .extent(direct)
                            .flatten()
                            .is_some_and(|bytes| r.compiler.ty.types.size_of(ty) <= bytes)
                            && slots
                                .iter()
                                .all(|slot| roots.canonical(r.places.paths[*slot].0.0) != direct)
                        {
                            ignored.insert(Site {
                                block,
                                event,
                            });
                            costs.push((1, 0));
                            continue;
                        }
                    }
                    let p = r.places.existing(place, ty)?;
                    let inline = if r.places.unresolved(p) {
                        false
                    } else {
                        *remaining = remaining.checked_sub(layout)?;
                        super::typed_inline_range(
                            r.compiler,
                            &r.places,
                            p,
                            r.compiler.ty.types.size_of(ty),
                        )
                        .is_some()
                    };
                    // A failed typed lookup under a private root could enter
                    // the legacy projection scan; this query has not reserved it.
                    if read_slots.is_some()
                        && !inline
                        && matches!(e, Event::DefaultImage { .. })
                        && r.private_bytes.contains_key(&r.places.root_of(p))
                    {
                        return None;
                    }
                    if !r.places.unresolved(p)
                        && !inline
                        && matches!(e, Event::Store { .. } | Event::ContextSet { .. })
                        && let (Some(roots), Some(slots)) = (&roots, read_slots)
                    {
                        *remaining = remaining.checked_sub(slots.len() + 1)?;
                        if let Some(object) = index.address(r, place, remaining)
                            && let crate::rules::lifetime_summary::address_objects::Base::Local(
                                root,
                            ) = object.base
                        {
                            let root = roots.canonical(root);
                            if roots.extent(root).flatten().is_some_and(|bytes| {
                                object
                                    .offset
                                    .checked_add(r.compiler.ty.types.size_of(ty))
                                    .is_some_and(|end| end <= bytes)
                            }) && slots
                                .iter()
                                .all(|slot| roots.canonical(r.places.paths[*slot].0.0) != root)
                            {
                                ignored.insert(Site {
                                    block,
                                    event,
                                });
                                costs.push((1, 0));
                                continue;
                            }
                        }
                    }
                    if !r.places.unresolved(p) && !inline {
                        if requested.is_none() && jacket_config::env::LIFETIME_STATS.is_set() {
                            eprintln!(
                                "lifetime-object-capture unresolved-memory ty={:?} place={:?} path={:?}",
                                ty, place, r.places.paths[p]
                            );
                        }
                        return None;
                    }
                    let range = if inline {
                        layout
                    } else {
                        events.checked_mul(r.places.paths.len().checked_add(layout)?)?
                    };
                    let target_work = if r.places.unresolved(p) {
                        0
                    } else if r.places.paths[p].1.is_empty() {
                        1
                    } else {
                        layout
                    };
                    cost = if let Some(slots) = read_slots {
                        let slot_work = slots.len().checked_mul(8)?.checked_add(8)?;
                        match e {
                            Event::Store {
                                value, ..
                            } => {
                                let per_slot = if r.places.unresolved(p) {
                                    2
                                } else {
                                    6
                                };
                                spill_work = slots.len().checked_mul(per_slot)?;
                                r.values.get(value).map_or(0, Vec::len).checked_add(
                                    target_work.checked_mul(3)?.checked_add(
                                        slots.len().checked_mul(per_slot)?.checked_add(8)?,
                                    )?,
                                )?
                            }
                            Event::Initialize {
                                ..
                            } => {
                                spill_work = slots.len().checked_mul(8)?;
                                target_work.checked_mul(3)?.checked_add(slot_work)?
                            }
                            Event::DefaultImage {
                                ..
                            } if !r.private_bytes.contains_key(&r.places.root_of(p)) => 1,
                            Event::DefaultImage {
                                ..
                            } => {
                                spill_work = slots.len().checked_mul(8)?;
                                target_work.checked_mul(4)?.checked_add(slot_work)?
                            }
                            _ => 1,
                        }
                    } else {
                        match e {
                            Event::Store {
                                value, ..
                            } => r.values.get(value).map_or(0, Vec::len).checked_add(
                                if r.places.unresolved(p) {
                                    1
                                } else {
                                    layout.checked_mul(3)?
                                },
                            )?,
                            Event::Initialize {
                                ..
                            } => {
                                if r.places.unresolved(p) {
                                    1
                                } else {
                                    layout.checked_mul(3)?
                                }
                            }
                            Event::DefaultImage {
                                ..
                            } => range.checked_mul(2)?,
                            _ => 1,
                        }
                    };
                }
                if let Event::IndirectCall {
                    callee,
                    allocator: false,
                    ..
                } = e
                {
                    *remaining = remaining
                        .checked_sub(r.targets.get(r.body.procedure, *callee).procedures.len())?;
                }
                costs.push((cost, spill_work));
            }
            event_work.push(costs);
        }
        if requested.is_none() && jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!(
                "lifetime-object-capture event-costs remaining={}",
                remaining
            );
        }
        if !r.owns_capture_index(index) {
            return None;
        }
        let mut stores = HashMap::default();
        let mut places = if requested.is_some() {
            std::borrow::Cow::Owned(r.places.clone())
        } else {
            std::borrow::Cow::Borrowed(&r.places)
        };
        let mut selected = None;
        if let Some((_, range)) = requested {
            *remaining = remaining.checked_sub(events.checked_mul(r.params.len())?)?;
            for (block, b) in r.body.blocks.iter().enumerate() {
                for (event, e) in b.events.iter().enumerate() {
                    let Event::Store {
                        place,
                        ty,
                        ..
                    } = e
                    else {
                        continue;
                    };
                    let Some(object) = index.address(r, *place, remaining) else {
                        continue;
                    };
                    let crate::rules::lifetime_summary::address_objects::Base::Input(formal) =
                        object.base
                    else {
                        continue;
                    };
                    let slot = Slot {
                        formal,
                        offset: object.offset,
                        width: r.compiler.ty.types.size_of(*ty),
                    };
                    if r.valid_slot(slot).is_some() {
                        stores.insert(
                            Site {
                                block,
                                event,
                            },
                            (slot, r.compiler.ty.types.is_pointer(*ty)),
                        );
                    }
                }
            }
            *remaining = remaining.checked_sub(r.params.len())?;
            let (_, param_ty) = r.params.values().find(|(n, _)| *n == range.formal)?;
            let incoming = r
                .body
                .blocks
                .iter()
                .flat_map(|b| &b.events)
                .find_map(|e| match e {
                    Event::Parameter {
                        formal,
                        incoming,
                        ..
                    } if *formal == range.formal => Some(*incoming),
                    _ => None,
                })?;
            let root = places.to_mut().place(incoming, *param_ty);
            let root = places
                .to_mut()
                .project(root, crate::rules::lifetime_summary::Projection::Deref);
            let slot = places.to_mut().project(
                root,
                crate::rules::lifetime_summary::Projection::Inline {
                    offset: range.offset,
                    ty: jaic::types::TypeId::VOID_PTR,
                },
            );
            places
                .to_mut()
                .types
                .insert(slot, jaic::types::TypeId::VOID_PTR);
            selected = Some(slot);
        }
        let live = if requested.is_some() {
            crate::rules::lifetime_summary::returning_blocks(r.body, all, r.targets)
        } else {
            crate::rules::lifetime_summary::flow::Live::all(r.body)
        };
        Some((
            places,
            Self {
                site: requested.map(|(site, _)| site),
                slot: selected,
                range: requested.map(|(_, range)| range),
                live,
                sources: HashSet::default(),
                failed: false,
                stores,
                event_work,
                ignored,
                preserving_calls: HashSet::default(),
                remaining,
                bound: None,
                bound_disjoint: false,
                bound_target: false,
                bound_active: false,
                bound_sources: std::array::from_fn(|_| Sources::default()),
            },
        ))
    }

    pub fn bind(
        &mut self,
        r: &Resolver<'_>,
        places: &mut std::borrow::Cow<'_, Places>,
        seed: Seed,
        (address, ty, fields, image, at): (
            Val,
            jaic::types::TypeId,
            &[(jaic::types::TypeId, crate::rules::lifetime_facts::ByteRange)],
            Site,
            Site,
        ),
    ) -> Option<()> {
        let block = r.body.blocks.get(image.block)?;
        let valid_seed = match seed {
            Seed::Image => {
                image.block == at.block
                    && image.event < at.event
                    && block.successors.is_empty()
                    && at.event <= block.events.len()
                    && matches!(block.events.get(image.event),Some(Event::Store{ty:selected,..}) if *selected==ty)
            }
            Seed::Call(_) => {
                matches!(block.events.get(image.event),Some(Event::Call{results,expands:false,..})
                if results.len()==1 && results[0].1==address && r.compiler.ty.types.pointee(results[0].2)==Some(ty))
            }
        };
        if self.slot.is_some()
            || fields.is_empty()
            || fields.len() > 3
            || !valid_seed
            || !self.charge(
                r.places
                    .clone_work()?
                    .checked_add(fields.len().checked_mul(8)?)?,
            )
        {
            return None;
        }
        let places = places.to_mut();
        let root = places.place(address, ty);
        let mut selected = [None; 3];
        for (i, &(field_ty, bytes)) in fields.iter().enumerate() {
            if bytes.width != r.compiler.ty.types.size_of(field_ty)
                || bytes.width != 8
                || bytes.offset.checked_add(bytes.width)? > r.compiler.ty.types.size_of(ty)
                || fields[..i].iter().any(|(_, old)| !old.separated(bytes))
            {
                return None;
            }
            let slot = places.project(
                root,
                crate::rules::lifetime_summary::Projection::Inline {
                    offset: bytes.offset,
                    ty: field_ty,
                },
            );
            places.types.insert(slot, field_ty);
            selected[i] = Some((slot, bytes));
        }
        self.site = Some(at);
        self.bound = Some(Bound {
            address,
            ty,
            fields: selected,
            image,
            seed,
            root,
            access: (0, 0, 1),
        });
        Some(())
    }

    pub fn bound_slots(&self) -> impl Iterator<Item = usize> + '_ {
        self.bound
            .iter()
            .filter(|_| self.bound_active)
            .flat_map(|bound| bound.fields.iter().flatten().map(|(slot, _)| *slot))
    }

    pub fn tracks(&self, slot: usize) -> bool {
        self.slot == Some(slot) || self.bound_slots().any(|other| slot == other)
    }

    pub fn tracked_slots(&self) -> impl Iterator<Item = usize> + '_ {
        self.slot.into_iter().chain(self.bound_slots())
    }

    /// Decode only the exact typed image values still held at the requested epoch.
    pub fn bound_values(
        &mut self,
        r: &Resolver<'_>,
        places: &Places,
    ) -> Option<Vec<jaic::ide::lifetimes::LifetimeDefaultLeaf>> {
        use jaic::ide::lifetimes::{LifetimeDefaultLeaf, LifetimeDefaultValue};
        let bound = self.bound?;
        let image_source = if matches!(bound.seed, Seed::Image) {
            let Event::Store {
                value,
                ty,
                ..
            } = r
                .body
                .blocks
                .get(bound.image.block)?
                .events
                .get(bound.image.event)?
            else {
                return None;
            };
            Some((*value, *ty))
        } else {
            None
        };
        let count = bound.fields.iter().flatten().count();
        if !self.charge(
            r.body
                .read_only_regions
                .len()
                .checked_add(count.checked_mul(3)?)?,
        ) {
            return None;
        }
        let mut values = [None; 3];
        for region in &r.body.read_only_regions {
            if image_source.is_none_or(|(value, _)| region.value != value) {
                continue;
            }
            if image_source.is_none_or(|(_, ty)| {
                region.ty != ty || region.bytes != r.compiler.ty.types.size_of(ty)
            }) || !self.charge(region.leaves.len().checked_mul(count.checked_add(1)?)?)
            {
                return None;
            }
            for (i, (slot, bytes)) in bound
                .fields
                .iter()
                .enumerate()
                .filter_map(|(i, entry)| entry.map(|entry| (i, entry)))
            {
                let field_ty = places.selected_type(slot)?;
                let mut found = None;
                for leaf in &region.leaves {
                    if leaf.offset == bytes.offset
                        && leaf.width == bytes.width
                        && leaf.ty == field_ty
                    {
                        if found.is_some_and(|old| old != leaf.value) {
                            return None;
                        }
                        found = Some(leaf.value);
                    }
                }
                let found = found?;
                if values[i].is_some_and(|old| old != found) {
                    return None;
                }
                values[i] = Some(found);
            }
        }
        let mut out = Vec::with_capacity(count);
        for (i, (slot, bytes)) in bound
            .fields
            .iter()
            .enumerate()
            .filter_map(|(i, entry)| entry.map(|entry| (i, entry)))
        {
            let sources = &self.bound_sources[i];
            if sources.len() != 1 {
                return None;
            }
            let field_ty = places.selected_type(slot)?;
            let selected = match sources.iter().next()? {
                Source::Aggregate(source, source_ty, base, selected, site)
                    if image_source == Some((*source, *source_ty))
                        && *base == bound.root
                        && *selected == slot
                        && *site == bound.image =>
                {
                    values[i]?
                }
                Source::Constant {
                    ty,
                    value,
                } if *ty == field_ty => *value,
                _ => return None,
            };
            if selected == LifetimeDefaultValue::Unknown {
                return None;
            }
            out.push(LifetimeDefaultLeaf {
                offset: bytes.offset,
                width: bytes.width,
                ty: field_ty,
                value: selected,
            });
        }
        self.charge(out.len()).then_some(out)
    }

    /// Return only already-validated address markers, never skipped writes.
    pub fn metadata_barriers(&mut self, r: &Resolver<'_>) -> Option<HashSet<Site>> {
        if !self.charge(self.ignored.len().checked_mul(2)?) {
            return None;
        }
        let mut markers = HashSet::default();
        for site in &self.ignored {
            if matches!(
                r.body.blocks.get(site.block)?.events.get(site.event),
                Some(Event::Barrier)
            ) {
                markers.insert(*site);
            }
        }
        Some(markers)
    }

    pub fn bound_access(&self) -> Option<(i128, i128, u64)> {
        self.bound.map(|bound| bound.access)
    }

    /// The selected full image overwrites every watched byte. Prefix effects
    /// remain in the separate lifetime audit and completed coordinate captures.
    pub fn bound_entry(&self) -> Option<Site> {
        self.bound
            .filter(|bound| self.bound_active && matches!(bound.seed, Seed::Image))
            .map(|bound| bound.image)
    }

    /// Current complete empty Frames preserve watched bytes without suppressing
    /// ordinary effects on any other cell. Receipts live only for this query.
    pub fn prepare_calls(
        &mut self,
        r: &Resolver<'_>,
        all: &HashMap<ProcId, Summary>,
    ) -> Option<()> {
        if !self.charge(r.body.blocks.len()) {
            return None;
        }
        for (block, b) in r.body.blocks.iter().enumerate() {
            if !self.charge(b.events.len()) {
                return None;
            }
            for (event, e) in b.events.iter().enumerate() {
                let Event::Call {
                    callee,
                    args,
                    results,
                    expands: false,
                    ..
                } = e
                else {
                    continue;
                };
                if !args.is_empty() || !results.is_empty() {
                    continue;
                }
                let Some(summary) = all.get(callee) else {
                    continue;
                };
                let Some(frame) = summary.frame.as_ref() else {
                    continue;
                };
                let sig = r.compiler.pr.procs.get(callee.0 as usize)?.sig.as_ref()?;
                if !sig.params.is_empty() || !sig.returns.is_empty() || frame.condition.is_some() {
                    continue;
                }
                let Some(typed) = frame.typed() else {
                    continue;
                };
                if !typed.reads.is_empty() || !typed.writes.is_empty() {
                    continue;
                }
                let count = frame.requirement_count();
                if count > 33 || !self.charge(count.checked_mul(2)?.checked_add(2)?) {
                    return None;
                }
                if frame.current(r.compiler) && frame.compatible(all) {
                    self.preserving_calls.insert(Site {
                        block,
                        event,
                    });
                }
            }
        }
        (!jaic::interrupt::should_stop()).then_some(())
    }

    /// Install only after ordinary call effects, within its proved nonnull query.
    pub fn seed_after(&mut self, site: Site, state: &mut State, places: &Places) -> bool {
        let Some(bound) = self
            .bound
            .filter(|bound| self.bound_active && bound.image == site)
        else {
            return true;
        };
        let Seed::Call(values) = bound.seed else {
            return true;
        };
        if !self.charge(bound.fields.iter().flatten().count() * 2 + 1) {
            return false;
        }
        for (i, entry) in bound.fields.iter().enumerate() {
            let Some((slot, _)) = entry else {
                continue;
            };
            let Some(ty) = places.selected_type(*slot) else {
                self.failed = true;
                return false;
            };
            let Some(value) = values[i] else {
                self.failed = true;
                return false;
            };
            state.insert(
                *slot,
                HashSet::from_iter([
                    if value == jaic::ide::lifetimes::LifetimeDefaultValue::Unknown {
                        Source::Unknown
                    } else {
                        Source::Constant {
                            ty,
                            value,
                        }
                    },
                ]),
            );
        }
        true
    }

    pub fn activate_bound(&mut self) {
        self.bound_active = true;
    }

    /// Project only exact same-cut byte coordinates into the already tracked cell.
    pub fn bound_event(
        &mut self,
        r: &Resolver<'_>,
        index: &crate::rules::lifetime_summary::captured_values::QueryIndex,
        loads: &CapturedLoads<'_>,
        site: Site,
        event: &Event,
    ) -> bool {
        self.bound_disjoint = false;
        self.bound_target = false;
        let Some(bound) = self.bound.filter(|_| self.bound_active) else {
            return true;
        };
        if matches!(
            event,
            Event::Store { .. }
                | Event::Initialize { .. }
                | Event::DefaultImage { .. }
                | Event::Call { .. }
                | Event::IndirectCall { .. }
                | Event::Grow { .. }
                | Event::Barrier
        ) && !self.charge(8 * bound.fields.iter().flatten().count())
        {
            return false;
        }
        if matches!(bound.seed, Seed::Image)
            && (site.block != bound.image.block || site.event < bound.image.event)
        {
            return true;
        }
        if matches!(event, Event::Call { .. }) && self.preserving_calls.contains(&site) {
            self.bound_disjoint = true;
            return true;
        }
        let (place, ty) = match event {
            Event::Store {
                place,
                ty,
                ..
            }
            | Event::ContextSet {
                place,
                ty,
                ..
            } => (*place, *ty),
            _ => return true,
        };
        let location = index.relative_write(
            r,
            (loads, Some(bound.address)),
            (place, ty),
            self.remaining,
            0,
        );
        if location.is_none() && index.get(place).is_none() {
            // Only the bytes of an actual retained GlobalAddr are static
            // storage; a pointer loaded from a global is not that object.
            let count = r
                .body
                .globals
                .len()
                .checked_add(r.body.stack_roots.len())
                .and_then(|n| n.checked_add(r.body.slot_aliases.len()))
                .and_then(|n| n.checked_add(index.captures(place).len()));
            let Some(count) = count else {
                self.failed = true;
                return false;
            };
            if !self.charge(count) {
                return false;
            }
            let mut globals = r.body.globals.iter().filter(|(value, _)| *value == place);
            if let Some((_, global)) = globals.next()
                && globals.all(|(_, other)| other == global)
                && r.body.stack_roots.iter().all(|root| root.value != place)
                && r.body
                    .slot_aliases
                    .iter()
                    .all(|(alias, root)| *alias != place && *root != place)
                && index.captures(place).iter().all(|row| {
                    matches!(
                        row,
                        crate::rules::lifetime_summary::captured_values::CaptureRow::Observation
                            | crate::rules::lifetime_summary::captured_values::CaptureRow::Metadata
                    )
                })
                && r.compiler
                    .out
                    .program
                    .globals
                    .get(global.0 as usize)
                    .is_some_and(|global| r.compiler.ty.types.size_of(ty) <= global.size)
            {
                self.bound_disjoint = true;
            }
        }
        if let Some((_, start, width)) = location {
            let Some(end) = start.checked_add(i128::from(width)) else {
                self.failed = true;
                return false;
            };

            let alignment = r.compiler.ty.types.align_of(ty);
            if alignment == 0 || start % i128::from(alignment) != 0 {
                self.failed = true;
                return false;
            }
            if let Some(bound) = &mut self.bound {
                bound.access.2 = bound.access.2.max(alignment);
                bound.access.0 = bound.access.0.min(start);
                bound.access.1 = bound.access.1.max(end);
            }
            self.bound_disjoint = bound.fields.iter().flatten().all(|(_, bytes)| {
                bytes
                    .offset
                    .checked_add(bytes.width)
                    .is_some_and(|watch_end| {
                        end <= i128::from(bytes.offset) || i128::from(watch_end) <= start
                    })
            });
            self.bound_target = site == bound.image
                && ty == bound.ty
                && start == 0
                && width == r.compiler.ty.types.size_of(bound.ty);
        }
        !jaic::interrupt::should_stop()
    }

    pub fn ignored(&self, site: Site) -> bool {
        self.ignored.contains(&site)
    }

    pub fn charge(&mut self, n: usize) -> bool {
        if let Some(next) = self.remaining.checked_sub(n)
            && !jaic::interrupt::should_stop()
        {
            *self.remaining = next;
            true
        } else {
            if jacket_config::env::LIFETIME_STATS.is_set() {
                eprintln!(
                    "lifetime-object-capture-budget needed={} remaining={}",
                    n, self.remaining
                );
            }
            self.failed = true;
            false
        }
    }
    pub fn observe(&mut self, site: Site, state: &State) -> bool {
        let (event, spill_work) = self
            .event_work
            .get(site.block)
            .and_then(|b| b.get(site.event))
            .copied()
            .unwrap_or((1, 0));
        let Some(event) = event.checked_sub(if self.bound_active {
            spill_work
        } else {
            0
        }) else {
            self.failed = true;
            return false;
        };
        if !self.charge(event) {
            return false;
        }
        if self.site == Some(site) && self.bound_active {
            if let Some(bound) = self.bound {
                for (i, (slot, _)) in bound
                    .fields
                    .iter()
                    .enumerate()
                    .filter_map(|(i, entry)| entry.map(|entry| (i, entry)))
                {
                    if let Some(sources) = state.get(&slot) {
                        if !self.charge(sources.len()) {
                            return false;
                        }
                        self.bound_sources[i].extend(sources);
                    }
                }
            }
        }
        if self.site == Some(site)
            && let Some(slot) = self.slot
            && let Some(sources) = state.get(&slot)
        {
            if !self.charge(sources.len()) {
                return false;
            }
            self.sources.extend(sources);
        }
        true
    }
    pub fn target(&self, site: Site, ordinary: Option<usize>) -> Option<usize> {
        if self.bound_target {
            return self.bound.map(|bound| bound.root);
        }
        if self
            .range
            .is_some_and(|range| self.stores.get(&site) == Some(&(range, true)))
        {
            self.slot
        } else {
            ordinary
        }
    }
    pub fn disjoint(&self, site: Site) -> bool {
        if self.bound.is_some() {
            return self.bound_active && self.bound_disjoint;
        }
        self.stores
            .get(&site)
            .is_some_and(|(s, _)| self.range.is_some_and(|range| s.disjoint(range)))
    }
}

// Upper bound on typed path and field-list work, not another address authority.
fn layout_work(r: &Resolver<'_>, remaining: &mut usize) -> Option<usize> {
    use crate::rules::lifetime_summary::Projection;
    let mut total = 1usize;
    for ((_, root_ty), path) in &r.places.paths {
        *remaining = remaining.checked_sub(path.len() + 1)?;
        total = total.checked_add(path.len() + 1)?;
        let mut ty = *root_ty;
        for step in path {
            ty = match step {
                Projection::Deref => match r.compiler.ty.types.pointee(ty) {
                    Some(ty) => ty,
                    None => break,
                },
                Projection::Inline {
                    ty, ..
                } => *ty,
                Projection::Field(name) => {
                    let Some(st) = r.compiler.ty.types.as_struct(ty) else {
                        break;
                    };
                    let fields = &r.compiler.ty.types.struct_info(st).fields;
                    *remaining = remaining.checked_sub(fields.len())?;
                    total = total.checked_add(fields.len())?;
                    let Some(field) = fields.iter().find(|f| f.name == Some(*name)) else {
                        break;
                    };
                    field.ty
                }
                Projection::Element(_) => match r.compiler.ty.types.kind(ty) {
                    jaic::types::TypeKind::Array {
                        elem, ..
                    } => *elem,
                    _ => break,
                },
                Projection::DynamicElement(_) => break,
            };
        }
    }
    Some(total)
}
